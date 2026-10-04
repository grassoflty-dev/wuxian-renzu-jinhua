import type { BossEncounterProjection, Vec3, WorldSnapshotEnvelope } from "../protocol/types.js";
export const WARDEN_ID = "mh_resonance_warden_staged_marker";
const ENTITY_TYPE = "enemy.mist_harbor.resonance_warden";
const TEMP_RENDER_TYPE = "runtime2d.enemy.mistharbor.signal_wraith.v2";
const states = new Set(["idle", "chase", "decoy", "windup", "recovery", "stagger"]);
export interface WardenEncounterFrame {
    text: string;
    hpRatio: number;
    phase: number;
    source: Vec3 | null;
    warning: {
        kind: "strike" | "pulse" | "decoy";
        polygonM: Array<[
            number,
            number
        ]>;
        color: number;
        alpha: number;
        label: string;
    } | null;
}
const counter = (n: unknown): n is number => Number.isSafeInteger(n) && (n as number) >= 0;
const finite = (n: unknown): n is number => typeof n === "number" && Number.isFinite(n);
function mappingAuthorized(snapshot: Extract<WorldSnapshotEnvelope, {
    protocolVersion: 3;
}>): boolean {
    if (snapshot.capabilities.acousticMappingAuthorized !== undefined)
        return snapshot.capabilities.acousticMappingAuthorized === true;
    const rows = snapshot.capabilities.items?.filter(row => row.capabilityId === "perception.acoustic_mapping_i") ?? [];
    return rows.length === 1 && rows[0]?.granted === true && rows[0]?.selected === true;
}
/** Ground warnings and a Boss bar from the exact Rust-authored encounter. No damage or progression. */
export function projectWardenEncounter(snapshot: WorldSnapshotEnvelope, reducedMotion = true): WardenEncounterFrame | null {
    if (snapshot.protocolVersion !== 3 || snapshot.worldId !== "mist_harbor" || snapshot.sceneId !== "mh_warden_arena" ||
        !counter(snapshot.worldEpoch) || snapshot.worldEpoch === 0 || !counter(snapshot.serverTick))
        return null;
    const boss: BossEncounterProjection | undefined = snapshot.bossEncounter;
    const actors = snapshot.actors.filter(actor => actor?.entityId === WARDEN_ID);
    if (!boss || boss.entityId !== WARDEN_ID || boss.entityType !== ENTITY_TYPE || actors.length !== 1 ||
        actors[0]?.entityType !== TEMP_RENDER_TYPE || actors[0]?.active !== true || !counter(boss.currentHp) || !counter(boss.maxHp) ||
        boss.currentHp === 0 || boss.currentHp > boss.maxHp || boss.maxHp > 100000 || !states.has(boss.state) ||
        (boss.phase !== 1 && boss.phase !== 2) || boss.temporaryVisual !== true || boss.publicReleaseEligible !== false || typeof boss.mappedTrueSource !== "boolean")
        return null;
    const w = boss.warning;
    let warning: WardenEncounterFrame["warning"] = null;
    let source: Vec3 | null = null;
    if (w) {
        if (typeof w.ordinaryVisible !== "boolean" || !counter(w.attackSerial) || w.attackSerial === 0 || !counter(w.remainingMs) || w.remainingMs === 0 || w.remainingMs > 60000 ||
            !Array.isArray(w.originM) || w.originM.length !== 3 || !w.originM.every(finite) || w.originM[0] < .5 || w.originM[0] > 23.5 ||
            w.originM[1] !== 0 || w.originM[2] < .5 || w.originM[2] > 15.5 || !finite(w.directionRad) || Math.abs(w.directionRad) > Math.PI + 1e-6 ||
            !finite(w.radiusM) || w.radiusM <= 0 || w.radiusM > 12 || !finite(w.halfAngleRad) || w.halfAngleRad <= 0 || w.halfAngleRad > Math.PI + 1e-6 ||
            !["strike", "pulse", "decoy"].includes(w.kind) || (w.kind === "decoy") !== (boss.state === "decoy") ||
            (w.kind !== "decoy" && boss.state !== "windup") || (boss.mappedTrueSource && (w.kind === "decoy" || !mappingAuthorized(snapshot))))
            return null;
        // Losing the effective permission removes early disclosure immediately.
        if (w.ordinaryVisible || boss.mappedTrueSource && mappingAuthorized(snapshot)) {
            const [x, , z] = w.originM;
            const cone = w.kind === "strike";
            const polygonM: Array<[
                number,
                number
            ]> = cone ? [[x, z]] : [];
            const start = cone ? w.directionRad - w.halfAngleRad : 0;
            const span = cone ? w.halfAngleRad * 2 : Math.PI * 2;
            for (let i = 0; i <= 32; i++) {
                const a = start + span * i / 32;
                polygonM.push([x + Math.sin(a) * w.radiusM, z + Math.cos(a) * w.radiusM]);
            }
            warning = { kind: w.kind, polygonM, color: w.kind === "decoy" ? 0x84a8ba : w.kind === "strike" ? 0xffbe67 : 0xf08464,
                alpha: reducedMotion ? .16 : .16 + Math.sin(snapshot.serverTick / 5) * .025,
                label: w.kind === "decoy" ? "回声" : w.kind === "strike" ? "定向共鸣 · 移出扇区" : "扩散共鸣 · 离开圆环" };
            if (boss.mappedTrueSource && mappingAuthorized(snapshot))
                source = { xM: x, yM: 0, zM: z };
        }
    }
    else if (boss.mappedTrueSource)
        return null;
    return { text: `共鸣守望者 · 阶段 ${boss.phase} · ${boss.currentHp} / ${boss.maxHp}`,
        hpRatio: boss.currentHp / boss.maxHp, phase: boss.phase, source, warning };
}
