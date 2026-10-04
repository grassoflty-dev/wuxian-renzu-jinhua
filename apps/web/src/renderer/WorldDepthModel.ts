import type { RenderLayerId } from "./LayerModel.js";
const ATMOSPHERE = new Set(["runtime2d.world.mistharbor.fog_bank.v1", "runtime2d.world.mistharbor.fog_vfx.v1", "runtime2d.world.mistharbor.water_edge.v1"]);
export function sharesWorldDepth(layer: RenderLayerId, assetId = ""): boolean {
    if (ATMOSPHERE.has(assetId))
        return false;
    return layer === "L2_BACK_PROPS" || layer === "L3_ACTORS" || layer === "L4_DYNAMIC_PROPS" || (layer === "L5_FRONT_PROPS" && assetId === "runtime2d.grey_hive.pipes_cables.v1");
}
/** Physical objects cross in one foot-depth ordering; stable keys settle ties. */
export function worldDepthRanks(items: readonly {
    key: string;
    footY: number;
    layer: RenderLayerId;
    asset?: {
        assetId: string;
    };
}[]): ReadonlyMap<string, number> {
    const ordered = items.filter(item => sharesWorldDepth(item.layer, item.asset?.assetId));
    if (ordered.some(item => !Number.isFinite(item.footY)))
        throw new Error("E_WORLD_OBJECT_DEPTH");
    ordered.sort((a, b) => a.footY - b.footY || (a.key < b.key ? -1 : a.key > b.key ? 1 : 0));
    return new Map(ordered.map((item, index) => [item.key, index * 10]));
}
