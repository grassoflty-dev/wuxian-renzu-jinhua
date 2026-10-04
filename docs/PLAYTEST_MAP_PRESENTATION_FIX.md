# Map presentation correction for the first Windows playtest

This patch starts from the history-free preview source `43035ecc`. It changes presentation only: the admitted images, runtime manifest, all 30 scene definitions, collision, navigation, progression and save code retain their bytes. It does not include later Beacon or Sentinel gameplay.

## Collection images and physical scale

The four Grey Hive environment assets contain multiple separate objects. Rendering those complete sheets as room-sized sprites was incorrect. The eleven canonical Grey Hive scenes and Return Station now use a presentation adapter over their unchanged definitions. Individual wall, straight-pipe and toolbox rectangles stay strictly inside their approved atlas frame. They have explicit display dimensions in metres; the CameraModel no longer fits these collection IDs to a room-sized background.

The floor is one Pixi Mesh, not overlapping raised tile sprites. Each 2 m cell samples the first approved floor tile's planar top, using source-relative points `(188,179), (347,302), (187,423), (22,302)`. Those four points map to the exact fixed-camera world-plane corners. Partial cells end at the scene bounds. The side thickness and other seven pieces are never sampled. A continuous floor underlay covers any antialias seams. Solid footprints come from the original collision polygons; their entrance/exit gaps remain open. Rear wall art is confined to actual north-boundary solids. Other bounds remain cutaway footprints rather than invented walk-blocking walls.

Gate A and the power console choose a single cropped state from the accepted public door/progression view. Unlocking alone does not imply an open door. Gate B has only approved closed art, so its closed panel is removed only when the authoritative door is open. A single masked supply crate replaces the three-crate sheet. Beacon and Return Station terminal images were visually checked as single objects and retain their pixels, with explicit physical sizes. No new PNG is generated.

## Feet and layering

Physical props and actors share one sortable parent. Their camera foot depths and stable keys determine front/back order, rather than every dynamic prop being unconditionally above every actor. Public Wraith alternatives and Swarm member positions participate independently; this never queries hidden actor state. Floor, declared atmospheric foreground, explicit occluder logic, effects and HUD retain separate roles. Existing capability filtering is unchanged; no through-wall enemy outline is added.

The right-hand diagnostic panel is hidden by default and does not reserve a column. `?developer=1` explicitly enables it. Return remains a normal HUD button, including while loading; save and enhancement status remain available through pause.

## Verification scope

Focused tests cover all 30 admitted scenes, all 12 facility conversions, rectangle bounds, exact floor coverage and partial edges, real Pixi Mesh resources and shared object parenting, live depth crossings, authoritative prop states and default diagnostics. A CPU composition made from the same model and approved pixels is a diagnostic, not a running-game screenshot. Browser/Windows GPU appearance and play feel require actual execution; unit tests do not replace that acceptance.
