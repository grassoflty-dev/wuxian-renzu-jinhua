# Runtime resource bytes and public-use scope

This addition places the 31 PNG files listed by `tools/resource-preflight/resource-map-v1.json` at their existing Godot consumer paths. It does not change consumer code, resource selection or runtime admission flags.

## Existing public resources

Twenty-seven files are byte-identical copies of existing public repository resources. Their original source paths, Git blob IDs and SHA256 values are recorded in [the resource map](../../tools/resource-preflight/resource-map-v1.json). Existing [public asset lineage](../../governance/assets/PUBLIC_ASSET_LINEAGE.json) remains the source of their recorded provenance and project-use scope. No new artwork or altered version is introduced by these copies.

## Four skill icons

The Pulse, Guard, Pierce and Dash 128-pixel PNGs were derived from independently authored SVG artwork commissioned for this project. The user has explicitly authorized inclusion of these four PNG files in this project's public repository. This authorization covers these four runtime icon files; it does not extend to other reference images, failed candidates or unrelated private assets. No new third-party licensing representation is made here.

The exact bytes match the checksums already recorded in the public skill consumer manifest and resource map. Their previously recorded candidate/internal-use flags are retained unchanged as historical integration and validation metadata. Those older flags do not negate this newly granted public-repository inclusion permission, and this permission does not itself establish Godot import, visual display, gameplay or release acceptance.

## Validation boundary

Byte integrity and repository path placement can be verified independently of runtime behavior. Godot import, display, native build, startup and gameplay remain unverified by this resource-only addition. In particular, the resource preflight checker at commit `7b2dd4dfe65ae07974c79f5d82fba27367c9e60d` has a reported Windows metadata-comparison failure pending correction; adding these PNGs does not resolve or bypass that defect. No placeholder substitutions, regeneration, recoloring or image cleanup are included.

## Return-station floor r2 integration

The user has authorized project-required runtime images in this game's public repository. This addition uses the existing project-commissioned floor candidate `ART-RS-CORE-FLOOR-BASE-20261014-001/r2` as `assets/world/rs_core_floor_base_r2.png`. It is a byte-identical copy of `scene-rs-core-floor-base-orthographic-original-v2.png`, with no regeneration, crop, resize or recoloring. This permission does not publish unrelated private references, discussions or rejected artwork and does not make a new third-party licensing representation.

- Original PNG: 1536 x 1024, RGB, opaque, 2,655,885 bytes.
- SHA256: `b19728536bb4edeea0cb63de736d18ce5788153b398d630169317a50fba905d9`.
- Git blob: `45834dde586df0a54b972c35859593de480e0937`.
- The image was generated for this project's floor brief. Its 12 x 8 panel layout covers the existing 24 x 16 metre floor once, at y=0. It is not a repeating tile or an additional collision surface.
- Mapping: world corners (0,0,0), (24,0,0), (24,0,16), (0,0,16) correspond to UV (0,0), (1,0), (1,1), (0,1).

`rs_floor_visual.gd` loads only this fixed imported texture through `ResourceLoader`, and only for matching `return_station/rs_core_room` snapshot and scene IDs with exact 0/0/24/16 bounds. The renderer retains its original floor, draws the texture over it when available, then draws the existing collision and other visuals. Missing or invalid imports keep the original floor; success or failure is cached for the consumer instance. After a failed import, restart the instance after repairing/reimporting the resource. No private selection file is required.

Byte identity is checked before publication, not by hashing engine-transformed texture data at runtime. Godot must import the PNG normally; any future selective export must include this texture. Resource remapping support is not proof of a tested export or packaged resource list.

The historical artwork status remains `PARTIAL_CANDIDATE / NOT_ADMITTED`; public-source permission is separate from visual or release acceptance. Known static QA risks remain: local bright/bevel bands around 4-8 source pixels, and low contrast between the floor and dark trouser legs in the earlier 1280-pixel static probe. Two of 96 panel low-frequency samples exceeded 12 (maximum 12.86), within the brief's 5% allowance. The old probes predate the current safe-rectangle projection and are not gameplay screenshots. Godot import, actual filtering, HUD/character contrast, movement, high-resolution display and export are `NOT_RUN` for this integration.
