import { execFile } from "node:child_process";
import { promisify } from "node:util";
import path from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { bounded } from "./browser-cleanup.mjs";

const execute = promisify(execFile);
export const ownershipError = (code, message, cause) => Object.assign(new Error(message, { cause }), { code });
const key = p => `${p.pid}:${p.created}`;
const canonical = value => path.resolve(value).toLowerCase();
export function commandFlag(command, name) {
  const match = command?.match(new RegExp(`(?:^|\\s)(?:"--${name}=([^"]*)"|--${name}=(?:"([^"]*)"|(\\S+)))`));
  return match?.slice(1).find(value => value !== undefined);
}
export function sameProcess(a, b) {
  return !!a && !!b && a.pid === b.pid && a.created === b.created && a.exe === b.exe && a.command === b.command;
}

export async function windowsProcesses() {
  const script = '$ErrorActionPreference="Stop"; @(Get-CimInstance Win32_Process | ForEach-Object { [pscustomobject]@{pid=[int]$_.ProcessId;parent=[int]$_.ParentProcessId;name=$_.Name;created=$_.CreationDate.ToUniversalTime().ToString("o");exe=$_.ExecutablePath;command=$_.CommandLine} }) | ConvertTo-Json -Compress';
  try {
    const { stdout } = await execute("powershell.exe", ["-NoProfile", "-NonInteractive", "-Command", script],
      { windowsHide: true, timeout: 1000, maxBuffer: 8 * 1024 * 1024 });
    const rows = JSON.parse(stdout);
    if (!Array.isArray(rows) || rows.some(p => !Number.isInteger(p.pid) || !Number.isInteger(p.parent) || !Number.isFinite(Date.parse(p.created)))) throw new Error("Invalid process inventory");
    return rows;
  } catch (error) { throw ownershipError("E_PROCESS_QUERY", "Cannot verify process ownership", error); }
}

export async function windowsListener(port) {
  try {
    const { stdout } = await execute("netstat.exe", ["-ano", "-p", "TCP"], { windowsHide: true, timeout: 1000, maxBuffer: 4 * 1024 * 1024 });
    return [...new Set(stdout.split(/\r?\n/).map(line => line.trim().split(/\s+/))
      .filter(fields => fields[0] === "TCP" && fields[1]?.endsWith(`:${port}`) && fields[3] === "LISTENING")
      .map(fields => Number(fields[4])))];
  } catch (error) { throw ownershipError("E_PORT_QUERY", "Cannot verify CDP listener", error); }
}

export async function stopWindowsProcess(expected) {
  // Recheck inside the same process which performs termination. Get-Process opens
  // the process handle; its start time must still agree with the CIM identity.
  const payload = Buffer.from(JSON.stringify(expected)).toString("base64");
  const script = `$ErrorActionPreference="Stop"; $e=[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('${payload}'))|ConvertFrom-Json; $p=Get-CimInstance Win32_Process -Filter "ProcessId=$($e.pid)"; if(!$p){exit 0}; if($p.CreationDate.ToUniversalTime().ToString("o") -ne $e.created -or $p.ExecutablePath -cne $e.exe -or $p.CommandLine -cne $e.command){throw "Process identity changed"}; $handle=Get-Process -Id $e.pid; if([Math]::Abs(($handle.StartTime.ToUniversalTime()-$p.CreationDate.ToUniversalTime()).TotalMilliseconds) -gt 1){throw "Process handle identity changed"}; $handle.Kill()`;
  try { await execute("powershell.exe", ["-NoProfile", "-NonInteractive", "-Command", script], { windowsHide: true, timeout: 1000 }); }
  catch (error) { throw ownershipError("E_PROCESS_TERMINATE", "Owned process termination failed", error); }
}

// A registry is per session. A launcher PID is only a lead; exact profile,
// executable, creation identity and debug port authorize a relaunched root.
export function createProcessOwnership({ profile, binary, port, startedAt, before,
  query = windowsProcesses, stop = stopWindowsProcess, listener = windowsListener,
  workerRoot, workerExecutable }) {
  if (!Array.isArray(before) || !Number.isFinite(startedAt)) throw new TypeError("Ownership requires the pre-launch inventory and start time");
  const prior = new Set(before.map(key));
  const known = new Map();
  let launcher;
  const valid = p => p.exe && p.command && Number.isFinite(Date.parse(p.created));
  const newer = p => Date.parse(p.created) >= startedAt && !prior.has(key(p));
  const exactProfile = p => {
    const value = commandFlag(p.command, "user-data-dir");
    return profile && value && canonical(value) === canonical(profile);
  };
  async function refresh() {
    const rows = await bounded(query, 1000, "E_PROCESS_QUERY_TIMEOUT");
    if (!Array.isArray(rows) || rows.some(p => !Number.isInteger(p.pid) || !Number.isInteger(p.parent) || !Number.isFinite(Date.parse(p.created)))) throw ownershipError("E_PROCESS_QUERY", "Invalid process query result");
    const byPid = new Map(rows.map(p => [p.pid, p]));
    for (const [pid, identity] of known) {
      if (byPid.has(pid) && !sameProcess(byPid.get(pid), identity)) throw ownershipError("E_PROCESS_IDENTITY", "Owned PID was reused or changed");
    }
    for (const p of rows) {
      if (!newer(p)) continue;
      if (profile && ["msedge.exe", "identity_helper.exe"].includes(p.name?.toLowerCase()) && !valid(p)) throw ownershipError("E_PROCESS_INCOMPLETE", `New browser process ${p.pid} has incomplete identity`);
      if (exactProfile(p) && !valid(p)) throw ownershipError("E_PROCESS_INCOMPLETE", `Profile process ${p.pid} has incomplete identity`);
      if (profile && exactProfile(p) && valid(p) && canonical(p.exe) === canonical(binary)
        && !commandFlag(p.command, "type") && commandFlag(p.command, "remote-debugging-port") === String(port)) known.set(p.pid, p);
      if (launcher?.pid === p.pid && newer(p)) {
        if (!valid(p)) throw ownershipError("E_PROCESS_INCOMPLETE", `Launcher process ${p.pid} has incomplete identity`);
        if (canonical(p.exe) !== canonical(binary) || !exactProfile(p) || commandFlag(p.command, "remote-debugging-port") !== String(port)) throw ownershipError("E_PROCESS_IDENTITY", "Launcher identity does not match its session");
        known.set(p.pid, p);
      }
      if (workerRoot && p.parent === workerRoot.pid && newer(p) && valid(p) && canonical(p.exe) === canonical(workerExecutable)) {
        const parent = byPid.get(workerRoot.pid);
        if (!sameProcess(parent, workerRoot)) throw ownershipError("E_PROCESS_IDENTITY", "Worker parent identity changed");
        known.set(p.pid, p);
      }
    }
    let added;
    do {
      added = false;
      for (const p of rows) {
        if (known.has(p.pid) || !newer(p)) continue;
        const parent = known.get(p.parent);
        if (!parent || !sameProcess(byPid.get(parent.pid), parent) || Date.parse(p.created) < Date.parse(parent.created)) continue;
        if (!valid(p)) throw ownershipError("E_PROCESS_INCOMPLETE", `Owned descendant ${p.pid} has incomplete identity`);
        known.set(p.pid, p); added = true;
      }
    } while (added);
    // A same-profile orphan which was never verified cannot prove safe removal.
    if (profile && rows.some(p => exactProfile(p) && !known.has(p.pid))) throw ownershipError("E_PROCESS_IDENTITY", "Unverified process still references the session profile");
    return rows.filter(p => known.has(p.pid) && sameProcess(p, known.get(p.pid)));
  }
  return {
    registerLauncher(child) { launcher = child; },
    refresh,
    async verifyPort() {
      const owned = await refresh();
      const pids = await listener(port);
      if (!pids.length) return false;
      if (pids.length !== 1 || !owned.some(p => p.pid === pids[0] && !commandFlag(p.command, "type"))) throw ownershipError("E_CDP_OWNER", "CDP listener does not belong to the session root");
      return true;
    },
    async hasExited() { return (await refresh()).length === 0 && (!launcher || launcher.pid == null || launcher.exitCode !== null || launcher.signalCode !== null); },
    async waitForExit(timeoutMs) {
      const end = Date.now() + timeoutMs;
      do { if (await this.hasExited()) return true; if (Date.now() >= end) return false; await delay(Math.min(50, end - Date.now())); } while (Date.now() < end);
      return false;
    },
    async terminate(timeoutMs = 10000) {
      const end = Date.now() + timeoutMs;
      const owned = await refresh();
      // Children first. Never terminate a fresh, unregistered PID or process group.
      for (const p of owned.sort((a, b) => Date.parse(b.created) - Date.parse(a.created))) {
        if (Date.now() >= end) throw ownershipError("E_PROCESS_TERMINATE_TIMEOUT", "Owned termination budget exhausted");
        const fresh = (await bounded(query, 1000, "E_PROCESS_QUERY_TIMEOUT")).find(candidate => candidate.pid === p.pid);
        if (!fresh) continue;
        if (!sameProcess(fresh, p)) throw ownershipError("E_PROCESS_IDENTITY", "Process changed before termination");
        await stop(p);
      }
    },
  };
}
