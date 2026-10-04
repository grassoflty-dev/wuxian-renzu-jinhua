import type { WorldSnapshotV3 } from "../../protocol/types.js";

export interface MistHarborPumpStatus {
  state: "ready" | "draining" | "drained";
  text: string;
}

/** Displays only the Rust-owned discrete state in the pump station. */
export function deriveMistHarborPumpStatus(snapshot: WorldSnapshotV3): MistHarborPumpStatus | null {
  if (snapshot.worldId !== "mist_harbor" || snapshot.sceneId !== "mh_pump_station") return null;
  switch (snapshot.mistHarborPump?.state) {
    case "ready": return { state: "ready", text: "排水泵待命" };
    case "draining": return { state: "draining", text: "排水进行中" };
    case "drained": return { state: "drained", text: "排水系统已完成" };
    default: return null;
  }
}
