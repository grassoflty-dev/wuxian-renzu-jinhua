import type { RuntimeAsset } from "../assets/AssetRegistry.js";
import type { SceneBoundsM, SceneSpritePlacement } from "./ScenePresentation.js";
export interface FacilityFloor {
    bounds: SceneBoundsM;
    walls: readonly (readonly (readonly [
        number,
        number
    ])[])[];
    asset: RuntimeAsset;
    topQuad: readonly (readonly [
        number,
        number
    ])[];
}
const FACILITY_SCENES = new Set(["gh_entry_maintenance", "gh_power_room", "gh_gate_a", "gh_central_shaft", "gh_lockdown", "gh_bio_isolation", "gh_gate_b", "gh_deep_decon", "gh_sentinel_arena", "gh_beacon", "gh_exit", "rs_core_room"]);
const PREFIX = "runtime2d.grey_hive.";
const SHEETS = new Set(["floor_tiles.v1", "wall_tiles.v1", "pipes_cables.v1", "clutter.v1"]);
export function isFacilityCollection(assetId: string): boolean { return assetId.startsWith(PREFIX) && SHEETS.has(assetId.slice(PREFIX.length)); }
/** Subframes reference admitted atlas pixels; the underlying PNG/WebP bytes never change. */
export function facilitySlice(asset: RuntimeAsset, rect: readonly [
    number,
    number,
    number,
    number
], anchor: readonly [
    number,
    number
]): RuntimeAsset {
    const [x, y, w, h] = rect, [ax, ay, aw, ah] = asset.atlasFrame;
    if (!isFacilityCollection(asset.assetId) || ![x, y, w, h].every(Number.isSafeInteger) || x < 0 || y < 0 || w <= 0 || h <= 0 || x + w > aw || y + h > ah || !anchor.every(v => Number.isFinite(v) && v >= 0 && v <= 1))
        throw new Error("E_FACILITY_SUBFRAME");
    return { ...asset, atlasFrame: [ax + x, ay + y, w, h], anchorX: anchor[0], anchorY: anchor[1] };
}
export function planFacilityMap(worldId: string, sceneId: string, bounds: SceneBoundsM, sprites: readonly SceneSpritePlacement[], collision: unknown): {
    sprites: SceneSpritePlacement[];
    floor?: FacilityFloor;
} {
    const floor = sprites.find(p => p.asset.assetId === PREFIX + "floor_tiles.v1");
    if (!FACILITY_SCENES.has(sceneId) || (worldId !== "grey_hive" && worldId !== "return_station") || !floor)
        return { sprites: [...sprites] };
    if (bounds.width > 128 || bounds.depth > 128 || !Array.isArray(collision))
        throw new Error("E_FACILITY_GEOMETRY");
    const walls = collision.map(item => {
        const polygon = (item as {
            polygon?: unknown;
        }).polygon;
        if (!Array.isArray(polygon) || polygon.length < 3 || polygon.some(p => !Array.isArray(p) || p.length !== 2 || !p.every(Number.isFinite) || p[0] < bounds.x || p[0] > bounds.x + bounds.width || p[1] < bounds.z || p[1] > bounds.z + bounds.depth))
            throw new Error("E_FACILITY_GEOMETRY");
        return polygon.map(p => [p[0], p[1]] as const);
    });
    const result = sprites.filter(p => !isFacilityCollection(p.asset.assetId));
    // Only the planar top face is sampled; the sheet margins and raised side edges are excluded.
    facilitySlice(floor.asset, [10, 170, 350, 300], [.5, .435]);
    const topQuad = [[188, 179], [347, 302], [187, 423], [22, 302]] as const;
    const wall = sprites.find(p => p.asset.assetId === PREFIX + "wall_tiles.v1");
    if (wall) {
        const panel = facilitySlice(wall.asset, [36, 0, 310, 470], [.5, .9]);
        // Rear boundary panels only. Other solid boundaries remain readable cutaway rails.
        for (const [index, polygon] of walls.entries()) {
            const xs = polygon.map(p => p[0]), zs = polygon.map(p => p[1]);
            const minX = Math.min(...xs), maxX = Math.max(...xs), minZ = Math.min(...zs), maxZ = Math.max(...zs);
            if (minZ !== bounds.z || maxZ - minZ > 1 || maxX - minX < 1)
                continue;
            for (let x = minX + .75; x < maxX; x += 1.5)
                result.push({ ...wall, id: `${wall.id}:panel:${index}:${x}`, asset: panel, position: { xM: x, yM: 0, zM: (minZ + maxZ) / 2 }, displaySizeM: { width: 1.65, height: 2.6 } });
        }
    }
    for (const p of sprites) {
        if (p.asset.assetId === PREFIX + "pipes_cables.v1")
            result.push({ ...p, asset: facilitySlice(p.asset, [0, 220, 380, 300], [.5, .8]), displaySizeM: { width: 1.7, height: 1.1 } });
        if (p.asset.assetId === PREFIX + "clutter.v1")
            result.push({ ...p, asset: facilitySlice(p.asset, [30, 40, 390, 305], [.5, .9]), displaySizeM: { width: 1.15, height: .75 } });
    }
    return { sprites: result, floor: { bounds: { ...bounds }, walls, asset: floor.asset, topQuad } };
}
