/** Bound, source-confirmed Grey Hive lines only. Pending catalog entries have no runtime text. */
const CONFIRMED_LINES: ReadonlyArray<{
  sceneId: string;
  targetId: string;
  kind: string;
  body: string;
}> = [
  {
    sceneId: "gh_beacon", targetId: "gh_beacon_deploy_marker", kind: "beacon_collect",
    body: "这就是信号源。带回去，也许能解释这里为什么还在呼叫。",
  },
  {
    sceneId: "gh_entry_maintenance", targetId: "gh_entry_checkpoint", kind: "scene_checkpoint",
    body: "B-17 链路仅保持最低功率。",
  },
  {
    sceneId: "gh_power_room", targetId: "gh_power_console", kind: "power_console",
    body: "主电还活着，只是被人为切断。",
  },
  {
    sceneId: "gh_lockdown", targetId: "gh_lockdown_terminal", kind: "lockdown_terminal",
    body: "隔离协议已被局部覆盖。",
  },
  {
    sceneId: "gh_sentinel_arena", targetId: "gh_sys_sentinel_01", kind: "facility_log",
    body: "自动防卫单元已接管本区。",
  },
];

export function confirmedGreyHiveNarrative(
  worldId: string,
  sceneId: string,
  targetId: string,
  kind: string,
  applied: boolean,
): string | null {
  if (!applied || worldId !== "grey_hive") return null;
  return CONFIRMED_LINES.find(line => line.sceneId === sceneId && line.targetId === targetId && line.kind === kind)?.body ?? null;
}
