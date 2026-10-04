import type { RuntimeAsset } from '../assets/AssetRegistry.js';
import type { DoorView, WorldSnapshotEnvelope } from '../protocol/types.js';
export interface PropPresentation {
    asset: RuntimeAsset;
    heightM: number;
    cutout?: readonly (readonly [
        number,
        number
    ])[];
}
const HEIGHTS: Readonly<Record<string, number>> = {
    'runtime2d.prop.gate_a.closed_open.v1': 2.6, 'runtime2d.prop.gate_b.v1': 2.6,
    'runtime2d.prop.power_console.off_on.v1': 1.45, 'runtime2d.prop.supply_crates.v1': 1,
    'runtime2d.prop.beacon.folded.v1': .55, 'runtime2d.prop.beacon.deployed.v1': 1.25,
    'runtime2d.prop.lockdown_terminal.v1': 1.4, 'runtime2d.prop.bio_pod.v1': 2.4,
    'runtime2d.prop.medical_station.v1': 1.8, 'runtime2d.world.returnstation.terminals.v1': 1.5,
    'runtime2d.world.returnstation.terminal.v1': 1.5,
};
export function isMappedProp(id: string): boolean { return Object.hasOwn(HEIGHTS, id); }
function slice(asset: RuntimeAsset, rect: readonly [
    number,
    number,
    number,
    number
]): RuntimeAsset {
    const [x, y, w, h] = rect, [ax, ay, aw, ah] = asset.atlasFrame;
    if (![x, y, w, h].every(Number.isSafeInteger) || x < 0 || y < 0 || w <= 0 || h <= 0 || x + w > aw || y + h > ah)
        throw new Error('E_PROP_SUBFRAME');
    return { ...asset, atlasFrame: [ax + x, ay + y, w, h], anchorX: .5, anchorY: 1 };
}
/** State comes only from accepted public door/progression views, never a predicted input. */
export function projectPropPresentation(asset: RuntimeAsset, snapshot: WorldSnapshotEnvelope, doors: readonly DoorView[]): PropPresentation | null | undefined {
    const heightM = HEIGHTS[asset.assetId];
    if (heightM === undefined)
        return undefined;
    if (asset.assetId === 'runtime2d.prop.gate_a.closed_open.v1') {
        const open = doors.find(d => d.doorId === 'gh_gate_a')?.open === true;
        return { asset: slice(asset, open ? [724, 216, 683, 625] : [0, 216, 690, 625]), heightM };
    }
    if (asset.assetId === 'runtime2d.prop.gate_b.v1' && doors.find(d => d.doorId === 'gh_gate_b')?.open === true)
        return null;
    if (asset.assetId === 'runtime2d.prop.power_console.off_on.v1') {
        const progression = snapshot.protocolVersion === 3 ? snapshot.progression : snapshot.view.progression;
        const gh = progression?.worlds.filter((r): r is {
            worldId: string;
            completedEvents: string[];
        } => typeof r === 'object' && r !== null && (r as {
            worldId?: unknown;
        }).worldId === 'grey_hive');
        const on = gh?.length === 1 && Array.isArray(gh[0]!.completedEvents) && gh[0]!.completedEvents.includes('hive_power');
        return { asset: slice(asset, on ? [752, 2, 696, 918] : [24, 2, 718, 918]), heightM };
    }
    if (asset.assetId === 'runtime2d.prop.supply_crates.v1')
        return { asset: slice(asset, [0, 640, 540, 550]), heightM, cutout: [[250, 10], [510, 100], [539, 440], [295, 549], [15, 420], [15, 140]] };
    return { asset, heightM };
}
