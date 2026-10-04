import type { InteractableView } from "../protocol/types.js";

// Mirrors mist_harbor.revisitPolicy in server-rs/data/world_progression_v1.json.
const MAX_REVISITS = 2;

/** Completed-world presentation only; Rust still authorizes every revisit. */
export function mistHarborRevisitStatus(
  revisitCount: unknown, gate: Pick<InteractableView, "active"> | undefined,
): string {
  if (typeof revisitCount !== "number" || !Number.isSafeInteger(revisitCount) ||
      revisitCount < 0 || revisitCount > MAX_REVISITS) return "已完成 · 复访状态未报告";
  if (revisitCount >= MAX_REVISITS) return "已完成 · 复访次数用尽";
  // Gates are scene-local, and a projected gate can be inactive while paused.
  return gate ? gate.active === true ? "已完成 · 限次复访入口可用" : "已完成 · 入口当前不可用"
    : "已完成 · 前往归航站查询复访";
}
