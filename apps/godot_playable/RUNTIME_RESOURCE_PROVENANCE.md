# Runtime resource bytes and public-use scope

This addition places the 31 PNG files listed by `tools/resource-preflight/resource-map-v1.json` at their existing Godot consumer paths. It does not change consumer code, resource selection or runtime admission flags.

## Existing public resources

Twenty-seven files are byte-identical copies of existing public repository resources. Their original source paths, Git blob IDs and SHA256 values are recorded in [the resource map](../../tools/resource-preflight/resource-map-v1.json). Existing [public asset lineage](../../governance/assets/PUBLIC_ASSET_LINEAGE.json) remains the source of their recorded provenance and project-use scope. No new artwork or altered version is introduced by these copies.

## Four skill icons

The Pulse, Guard, Pierce and Dash 128-pixel PNGs were derived from independently authored SVG artwork commissioned for this project. The user has explicitly authorized inclusion of these four PNG files in this project's public repository. This authorization covers these four runtime icon files; it does not extend to other reference images, failed candidates or unrelated private assets. No new third-party licensing representation is made here.

The exact bytes match the checksums already recorded in the public skill consumer manifest and resource map. Their previously recorded candidate/internal-use flags are retained unchanged as historical integration and validation metadata. Those older flags do not negate this newly granted public-repository inclusion permission, and this permission does not itself establish Godot import, visual display, gameplay or release acceptance.

## Validation boundary

Byte integrity and repository path placement can be verified independently of runtime behavior. Godot import, display, native build, startup and gameplay remain unverified by this resource-only addition. In particular, the resource preflight checker at commit `7b2dd4dfe65ae07974c79f5d82fba27367c9e60d` has a reported Windows metadata-comparison failure pending correction; adding these PNGs does not resolve or bypass that defect. No placeholder substitutions, regeneration, recoloring or image cleanup are included.
