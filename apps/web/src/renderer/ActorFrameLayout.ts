import type { AtlasFrame, RuntimeAsset } from "../assets/AssetRegistry.js";

interface ActorCrop {
  readonly sourceSha256: string;
  readonly sheetSize: readonly [number, number];
  readonly rect: AtlasFrame;
  readonly foot: readonly [number, number];
}

/** Single front idle pose measured in each approved sheet. This is not an action animation. */
export const TEMPORARY_ACTOR_CROPS: Readonly<Record<string, ActorCrop>> = Object.freeze({
  "runtime2d.enemy.infected_maintenance_worker.v1": {
    sourceSha256: "6b7055df24acfa58edfdf0342c67efb60f9fac892c80110d449700bf7021477d",
    sheetSize: [1447, 1086], rect: [17, 3, 428, 522], foot: [266, 511],
  },
  "runtime2d.enemy.infected_security.v1": {
    sourceSha256: "7bd9d4cf19f36d189cdf81530c3dee48450179696a7dbceb24eaf375714de4e4",
    sheetSize: [1407, 1072], rect: [14, 0, 332, 542], foot: [190, 537],
  },
  "runtime2d.enemy.sentinel.base.v1": {
    sourceSha256: "f7ced75a0fcf35afacf796f23630a40c309682c86ef67af79de5414a8a16fbe2",
    sheetSize: [1442, 1072], rect: [10, 0, 386, 541], foot: [205, 531],
  },
});

// All remaining enemy sources selected by the shipped native profiles are already
// single portraits. Unknown actor sheets must receive explicit frame review first.
export const REVIEWED_SINGLE_ACTOR_ASSETS = new Set([
  "runtime2d.enemy.mistharbor.drowned.v2",
  "runtime2d.enemy.mistharbor.signal_wraith.v2",
  "runtime2d.enemy.clockworks.forged_guard.v2",
]);

export function isolatedActorFrame(asset: RuntimeAsset): { atlasFrame: AtlasFrame; anchor: readonly [number, number] } | null {
  const crop = TEMPORARY_ACTOR_CROPS[asset.assetId];
  if (!crop) {
    if ((asset.kind === "Actor" || asset.kind === "Boss") && !REVIEWED_SINGLE_ACTOR_ASSETS.has(asset.assetId)) {
      throw new Error(`E_RENDERER_ACTOR_FRAME_UNREVIEWED:${asset.assetId}`);
    }
    return null;
  }
  const [x, y, width, height] = crop.rect;
  if (asset.sourceSha256 !== crop.sourceSha256 || asset.atlasFrame[2] !== crop.sheetSize[0] ||
      asset.atlasFrame[3] !== crop.sheetSize[1] || x + width > asset.atlasFrame[2] || y + height > asset.atlasFrame[3]) {
    throw new Error(`E_RENDERER_ACTOR_CROP_SOURCE:${asset.assetId}`);
  }
  return { atlasFrame: [asset.atlasFrame[0] + x, asset.atlasFrame[1] + y, width, height],
    anchor: [(crop.foot[0] - x) / width, (crop.foot[1] - y) / height] };
}
