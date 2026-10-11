#!/usr/bin/env node
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { basename, join, relative, resolve } from "node:path";

export function parseLock(text) {
  return text.split(/\r?\n\[\[package\]\]\r?\n/).slice(1).map((block) => {
    const value = (key) => block.match(new RegExp(`^${key} = "([^"]*)"`, "m"))?.[1] ?? null;
    return { name: value("name"), version: value("version"), source: value("source"), checksum: value("checksum") };
  }).filter((item) => item.name && item.version);
}

function manifestValue(text, key) {
  return text.match(new RegExp(`^${key}\\s*=\\s*"([^"]+)"`, "m"))?.[1] ?? null;
}

function sha256(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function evidenceFiles(dir, licenseFile) {
  const wanted = /^(licen[cs]e|copying|notice|copyright)(\..*)?$/i;
  const names = readdirSync(dir).filter((name) => statSync(join(dir, name)).isFile() && (wanted.test(name) || name === licenseFile));
  return [...new Set(names)].sort().map((name) => ({
    path: `cargo-registry/${basename(dir)}/${name}`,
    sha256: sha256(join(dir, name)),
  }));
}

export function inspectPackage(pkg, registryRoots) {
  if (!pkg.source?.startsWith("registry+")) {
    return { ...pkg, actualSpdx: null, status: "pending_review", evidence: [], attribution: null,
      distributionNotes: "Non-registry package: inspect the resolved source tree before distribution.", unknowns: ["No local registry evidence rule applies."] };
  }
  const folderName = `${pkg.name}-${pkg.version}`;
  const exact = registryRoots.map((root) => join(root, folderName)).find(existsSync);
  if (!exact) {
    const alternates = registryRoots.flatMap((root) => existsSync(root) ? readdirSync(root).filter((n) => n.startsWith(`${pkg.name}-`)) : []);
    return { ...pkg, actualSpdx: null, status: alternates.length ? "wrong_version_only" : "pending_review", evidence: [], attribution: null,
      distributionNotes: "Do not distribute until the exact locked source and its license material are reviewed.",
      unknowns: [alternates.length ? `Exact version absent; local alternatives: ${alternates.sort().join(", ")}` : "Exact locked crate is absent from the local cache."] };
  }
  const manifestPath = join(exact, "Cargo.toml");
  if (!existsSync(manifestPath)) {
    return { ...pkg, actualSpdx: null, status: "pending_review", evidence: [], attribution: null,
      distributionNotes: "Do not distribute until the exact source manifest and license material are reviewed.", unknowns: ["Exact source directory has no Cargo.toml."] };
  }
  const manifest = readFileSync(manifestPath, "utf8");
  const declaredLicenseExpression = manifestValue(manifest, "license");
  const actualSpdx = declaredLicenseExpression && !declaredLicenseExpression.includes("/") ? declaredLicenseExpression : null;
  const licenseFile = manifestValue(manifest, "license-file");
  const evidence = evidenceFiles(exact, licenseFile);
  const unknowns = [];
  if (!declaredLicenseExpression) unknowns.push("Cargo.toml has no license expression.");
  else if (!actualSpdx) unknowns.push(`Cargo.toml uses a non-SPDX legacy expression: ${declaredLicenseExpression}`);
  if (!evidence.length) unknowns.push("No top-level license/notice text found in the exact cached source.");
  return { ...pkg, declaredLicenseExpression, actualSpdx, status: unknowns.length ? "pending_review" : "evidence_found", evidence,
    attribution: evidence.length ? "Preserve applicable copyright/license text from the evidence files." : null,
    distributionNotes: unknowns.length ? "Review before distribution; metadata or license text is incomplete." : "Include applicable license and attribution text with binary/source distribution.", unknowns };
}

function arg(name, fallback = null) {
  const i = process.argv.indexOf(name);
  return i >= 0 ? process.argv[i + 1] : fallback;
}

function localCandidates(candidateRoot) {
  if (!candidateRoot) return [];
  const rel = (name) => join("server-rs", "release-ui", "vendor", name);
  const file = (name) => join(candidateRoot, rel(name));
  const evidence = (name) => existsSync(file(name)) ? [{ path: `original-workspace/${rel(name).replaceAll("\\", "/")}`, sha256: sha256(file(name)) }] : [];
  return [
    { name: "babylon.js", version: "9.26.2 (observed embedded version string)", classification: "vendored-code-candidate-for-INT-01-move", actualSpdx: null,
      status: "pending_review", evidence: evidence("babylon.js"), attribution: null,
      upstreamReference: { repository: "https://github.com/BabylonJS/Babylon.js", tag: "9.26.2", releaseCommitShort: "1c0985a", packagePath: "packages/public/umd/babylonjs/package.json", claimedSpdx: "Apache-2.0",
        license: { url: "https://raw.githubusercontent.com/BabylonJS/Babylon.js/9.26.2/license.md", sha256: "9362ea9ea17cb221a20cbbd63f404710834cc273d2a9a7b60314fd68ab702cd6" },
        notice: { url: "https://raw.githubusercontent.com/BabylonJS/Babylon.js/9.26.2/NOTICE.md", sha256: "225463efd81558f120a6e0cdd7f4c1007ed486006d1228c4849fee0c49715823" } },
      distributionNotes: "Official tag metadata says Apache-2.0, but the local minified bytes were not matched to the official npm artifact. Keep pending; if accepted later, ship the matching Apache-2.0 license and NOTICE and retain notices.",
      unknowns: ["No adjacent Babylon license text was found.", "The local dist SHA-256 has not been matched to the official 9.26.2 npm artifact, so the official tag cannot yet prove provenance of these exact bytes."] },
    { name: "three.min.js", version: null, classification: "legacy-vendored-code-not-a-BASE-release-dependency", actualSpdx: "MIT (from adjacent text only)",
      status: "excluded_not_base_dependency", evidence: [...evidence("three.min.js"), ...evidence("THREE-LICENSE.txt")],
      attribution: "If independently distributed, preserve THREE-LICENSE.txt.",
      distributionNotes: "Recorded for boundary clarity only. Do not infer that this legacy file is a dependency of the BASE_SHA release.", unknowns: ["Exact Three.js version was not established."] },
  ];
}

export function buildInventory({ repo, commit, cargoHome, candidateRoot = null }) {
  const lockPath = "server-rs/Cargo.lock";
  const lockText = execFileSync("git", ["show", `${commit}:${lockPath}`], { cwd: repo, encoding: "utf8" });
  const rootsBase = join(cargoHome, "registry", "src");
  const registryRoots = existsSync(rootsBase) ? readdirSync(rootsBase).map((n) => join(rootsBase, n)).filter((p) => statSync(p).isDirectory()) : [];
  const packages = parseLock(lockText).filter((pkg) => pkg.source !== null).map((pkg) => inspectPackage(pkg, registryRoots))
    .sort((a, b) => a.name.localeCompare(b.name) || a.version.localeCompare(b.version));
  const counts = packages.reduce((acc, pkg) => (acc[pkg.status] = (acc[pkg.status] || 0) + 1, acc), {});
  return { schemaVersion: 1, generatedFrom: { commit, lockPath, lockSha256: createHash("sha256").update(lockText).digest("hex") },
    evidencePolicy: "Exact locked version and local files only; no network fetch and no license inference.", counts: { total: packages.length, ...counts }, packages,
    additionalCandidates: localCandidates(candidateRoot) };
}

if (process.argv[1] && resolve(process.argv[1]) === resolve(new URL(import.meta.url).pathname.replace(/^\/(.:)/, "$1"))) {
  const repo = resolve(arg("--repo", "."));
  const commit = arg("--commit");
  const output = arg("--output");
  const cargoHome = resolve(arg("--cargo-home", process.env.CARGO_HOME || join(process.env.USERPROFILE || "", ".cargo")));
  const candidateRootArg = arg("--candidate-root");
  const candidateRoot = candidateRootArg ? resolve(candidateRootArg) : null;
  if (!commit || !output) throw new Error("Usage: inventory.mjs --repo PATH --commit SHA --output FILE [--cargo-home PATH]");
  const inventory = buildInventory({ repo, commit, cargoHome, candidateRoot });
  writeFileSync(resolve(output), `${JSON.stringify(inventory, null, 2)}\n`, "utf8");
  console.log(JSON.stringify(inventory.counts));
}
