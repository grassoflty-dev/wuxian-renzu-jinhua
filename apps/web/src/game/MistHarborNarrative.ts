import type { WorldSnapshotV3 } from "../protocol/types.js";

type BeaconSnapshot = Pick<WorldSnapshotV3, "worldId" | "sceneId" | "worldEpoch" | "progression">;
type MappingSnapshot = BeaconSnapshot & Pick<WorldSnapshotV3, "capabilities">;
type BeaconReceipt = {
  applied: boolean;
  alreadyApplied?: boolean;
  errorCode: string | null;
  snapshot: BeaconSnapshot;
};
type MappingReceipt = Omit<BeaconReceipt, "snapshot"> & { snapshot: MappingSnapshot };

const WORLD = "mist_harbor";
const WEST_EVENT = "mist_beacon_west";
const EAST_EVENT = "mist_beacon_east";
const BEACON_SYNC_LINE = "双基准建立，中心干扰源可定位。";
const MAPPING_LINE = "这次我不是在看路——是在听。";
const MAPPING_EVENT = "mist_signal";
const MAPPING_CAPABILITY = "perception.acoustic_mapping_i";
const SIGNAL_LINE = "不是没信号，是有人在用噪声盖住它。";
const SIGNAL_SCENE = "mh_signal_yard";
const SIGNAL_HAZARD = "mh_signal_interference_region";
const BEACONS = [
  { sceneId: "mh_tidal_warehouse", targetId: "mh_west_beacon", eventId: WEST_EVENT, otherEventId: EAST_EVENT },
  { sceneId: "mh_breakwater", targetId: "mh_east_beacon", eventId: EAST_EVENT, otherEventId: WEST_EVENT },
] as const;

function completedEvents(snapshot: BeaconSnapshot): ReadonlySet<string> | null {
  if (snapshot.progression?.currentWorldId !== WORLD ||
      !Array.isArray(snapshot.progression.worlds)) return null;
  const matching = snapshot.progression.worlds.filter(row =>
    !!row && typeof row === "object" && (row as Record<string, unknown>).worldId === WORLD);
  if (matching.length !== 1) return null;
  const events = (matching[0] as Record<string, unknown>).completedEvents;
  if (!Array.isArray(events) || !events.every(event => typeof event === "string")) return null;
  return new Set(events);
}

/** The catalog's pending combination hook is satisfied only by a fresh second Beacon receipt. */
export function confirmedMistHarborBeaconSync(
  before: BeaconSnapshot,
  targetId: string,
  kind: string,
  receipt: BeaconReceipt,
): string | null {
  const after = receipt.snapshot;
  const beacon = BEACONS.find(item => item.sceneId === before.sceneId && item.targetId === targetId);
  if (before.worldId !== WORLD || !beacon || kind !== "beacon" ||
      receipt.applied !== true || receipt.alreadyApplied === true || receipt.errorCode !== null ||
      after.worldId !== WORLD || after.sceneId !== beacon.sceneId ||
      after.worldEpoch !== before.worldEpoch ||
      !Number.isSafeInteger(before.progression?.eventSeq) ||
      !Number.isSafeInteger(after.progression?.eventSeq) ||
      after.progression.eventSeq <= before.progression.eventSeq) return null;
  const prior = completedEvents(before);
  const current = completedEvents(after);
  if (!prior?.has(beacon.otherEventId) || prior.has(beacon.eventId) ||
      !current?.has(WEST_EVENT) || !current.has(EAST_EVENT)) return null;
  return BEACON_SYNC_LINE;
}

/** Show the confirmed line only for the console receipt that newly grants selected mapping. */
export function confirmedMistHarborAcousticMappingLine(
  before: MappingSnapshot,
  targetId: string,
  kind: string,
  receipt: MappingReceipt,
): string | null {
  const after = receipt.snapshot;
  if (before.worldId !== WORLD || before.sceneId !== "mh_resonance_tower" ||
      targetId !== "mh_signal_console_staged" || kind !== "terminal" ||
      receipt.applied !== true || receipt.alreadyApplied === true || receipt.errorCode !== null ||
      after.worldId !== WORLD || after.sceneId !== before.sceneId ||
      !Number.isSafeInteger(before.worldEpoch) || after.worldEpoch !== before.worldEpoch ||
      !Number.isSafeInteger(before.progression?.eventSeq) ||
      !Number.isSafeInteger(after.progression?.eventSeq) ||
      after.progression.eventSeq <= before.progression.eventSeq) return null;
  const prior = completedEvents(before);
  const current = completedEvents(after);
  if (!prior || !current || prior.has(MAPPING_EVENT) || !current.has(MAPPING_EVENT) ||
      current.size !== prior.size + 1 || [...prior].some(event => !current.has(event))) return null;
  const beforeItems = before.capabilities?.items;
  const afterItems = after.capabilities?.items;
  if (!Array.isArray(beforeItems) || !Array.isArray(afterItems) ||
      beforeItems.some(item => item?.capabilityId === MAPPING_CAPABILITY && item.granted === true)) return null;
  const mappingItems = afterItems.filter(item => item?.capabilityId === MAPPING_CAPABILITY);
  const mappingItem = mappingItems[0];
  if (mappingItems.length !== 1 || mappingItem?.granted !== true ||
      mappingItem.selected !== true) return null;
  return MAPPING_LINE;
}

type SignalSnapshot = Pick<WorldSnapshotV3, "worldId" | "sceneId" | "worldEpoch" | "hazards">;

function signalHazardActive(snapshot: SignalSnapshot): boolean | null {
  if (!Array.isArray(snapshot.hazards)) return null;
  const matches = snapshot.hazards.filter(hazard => hazard?.entityId === SIGNAL_HAZARD);
  if (matches.length !== 1) return null;
  const hazard = matches[0];
  const position = hazard?.transform?.positionM;
  if (hazard?.kind !== "signal_interference_zone" || typeof hazard.active !== "boolean" ||
      !position || ![position.xM, position.yM, position.zM, hazard.transform.yawRad].every(Number.isFinite)) return null;
  return hazard.active;
}

/** Session-local narrative state; authoritative hazard snapshots are its only input. */
export class MistHarborSignalLineState {
  private identity: string | null = null;
  private previousActive: boolean | null = null;
  private readonly shownEpochs = new Set<number>();
  private pendingEpoch: number | null = null;

  constructor(initial: SignalSnapshot) {
    this.establishBaseline(initial);
  }

  private establishBaseline(snapshot: SignalSnapshot): void {
    this.pendingEpoch = null;
    if (snapshot.worldId !== WORLD || snapshot.sceneId !== SIGNAL_SCENE ||
        !Number.isSafeInteger(snapshot.worldEpoch)) {
      this.identity = null;
      this.previousActive = null;
      return;
    }
    this.identity = `${snapshot.worldId}\0${snapshot.sceneId}\0${snapshot.worldEpoch}`;
    this.previousActive = signalHazardActive(snapshot);
  }

  accept(snapshot: SignalSnapshot, canDisplay = true): string | null {
    const nextIdentity = snapshot.worldId === WORLD && snapshot.sceneId === SIGNAL_SCENE &&
      Number.isSafeInteger(snapshot.worldEpoch)
      ? `${snapshot.worldId}\0${snapshot.sceneId}\0${snapshot.worldEpoch}` : null;
    if (nextIdentity === null || nextIdentity !== this.identity) {
      this.establishBaseline(snapshot);
      return null;
    }
    const active = signalHazardActive(snapshot);
    const firstEntry = !this.shownEpochs.has(snapshot.worldEpoch) &&
      this.previousActive === false && active === true;
    this.previousActive = active;
    if (firstEntry) this.pendingEpoch = snapshot.worldEpoch;
    if (active !== true) this.pendingEpoch = null;
    if (!canDisplay || this.pendingEpoch !== snapshot.worldEpoch) return null;
    this.pendingEpoch = null;
    this.shownEpochs.add(snapshot.worldEpoch);
    return SIGNAL_LINE;
  }
}
