export const ADAPTER_ERRORS = Object.freeze({ MISSING_SCALE: 'E_ADAPTER_TILE_SCALE', MISSING_ORIGIN: 'E_ADAPTER_ORIGIN', MISSING_AXIS: 'E_ADAPTER_AXIS', INVALID_INPUT: 'E_ADAPTER_INPUT', NONFINITE: 'E_ADAPTER_NONFINITE' });

export function migrateV2ToV3(input, config) {
  if (!config || !Number.isFinite(config.tileSizeM) || config.tileSizeM <= 0) throw new Error(ADAPTER_ERRORS.MISSING_SCALE);
  if (!config.originM || !Number.isFinite(config.originM.x) || !Number.isFinite(config.originM.y) || !Number.isFinite(config.originM.z)) throw new Error(ADAPTER_ERRORS.MISSING_ORIGIN);
  if (config.axis !== 'x-z') throw new Error(ADAPTER_ERRORS.MISSING_AXIS);
  if (!input || !Number.isInteger(input.px) || !Number.isInteger(input.py) || !input.worldId || !input.zone2d) throw new Error(ADAPTER_ERRORS.INVALID_INPUT);
  const positionM = { x: config.originM.x + input.px * config.tileSizeM, y: config.originM.y, z: config.originM.z + input.py * config.tileSizeM };
  if (!Object.values(positionM).every(Number.isFinite)) throw new Error(ADAPTER_ERRORS.NONFINITE);
  return { schemaVersion: 3, worldId: input.worldId, entityId: `entity.player.${input.worldId}`, roomId: input.zone2d.roomId, zoneId: input.zone2d.zoneId, transform: { positionM, yawRad: 0 }, velocityMps: { x: 0, y: 0, z: 0 } };
}

export function signature(value) { return JSON.stringify(value); }
