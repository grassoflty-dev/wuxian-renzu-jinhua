import { projectWorldPoint, type CameraFrame } from "./CameraModel.js";
import type { FacilityFloor } from "./FacilityMapModel.js";
/** Maps the selected top diamond directly to the authoritative horizontal plane. */
export function facilityFloorGeometry(plan: FacilityFloor, camera: CameraFrame, atlasWidth: number, atlasHeight: number): {
    positions: Float32Array;
    uvs: Float32Array;
    indices: Uint32Array;
} {
    if (!Number.isFinite(atlasWidth) || !Number.isFinite(atlasHeight) || atlasWidth <= 0 || atlasHeight <= 0 || plan.topQuad.length !== 4)
        throw new Error("E_FACILITY_ATLAS");
    const [ax, ay, aw, ah] = plan.asset.atlasFrame;
    for (const [u, v] of plan.topQuad)
        if (!Number.isFinite(u) || !Number.isFinite(v) || u < 0 || v < 0 || u > aw || v > ah || ax + u > atlasWidth || ay + v > atlasHeight)
            throw new Error("E_FACILITY_ATLAS");
    const positions: number[] = [], uvs: number[] = [], indices: number[] = [];
    const { x, z, width, depth } = plan.bounds;
    const source = (a: number, b: number) => {
        const [tl, tr, br, bl] = plan.topQuad as readonly [
            readonly [
                number,
                number
            ],
            readonly [
                number,
                number
            ],
            readonly [
                number,
                number
            ],
            readonly [
                number,
                number
            ]
        ];
        return [(ax + tl[0] * (1 - a) * (1 - b) + tr[0] * a * (1 - b) + br[0] * a * b + bl[0] * (1 - a) * b) / atlasWidth,
            (ay + tl[1] * (1 - a) * (1 - b) + tr[1] * a * (1 - b) + br[1] * a * b + bl[1] * (1 - a) * b) / atlasHeight];
    };
    for (let zi = z; zi < z + depth; zi += 2)
        for (let xi = x; xi < x + width; xi += 2) {
            const w = Math.min(2, x + width - xi), d = Math.min(2, z + depth - zi), base = positions.length / 2;
            for (const [dx, dz] of [[0, 0], [w, 0], [w, d], [0, d]]) {
                const p = projectWorldPoint({ xM: xi + dx!, yM: 0, zM: zi + dz! }, camera);
                positions.push(p.x, p.y);
                uvs.push(...source(dx! / 2, dz! / 2));
            }
            indices.push(base, base + 1, base + 2, base, base + 2, base + 3);
        }
    return { positions: new Float32Array(positions), uvs: new Float32Array(uvs), indices: new Uint32Array(indices) };
}
