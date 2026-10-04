# AI Asset Release Review Policy

The authoritative decisions are the 94 rows in `AI_ASSET_RELEASE_MANIFEST.json`, transcribed from the verified v1 plan. The source report SHA-256 is `fcd027320a01a0fe083b09321deebc85b33598b4efc8642a6e5d624f5aaf51a1`; source PNG identity is pinned to Git commit `030eb9bf8194ad5534dc77315567ff339cd2c625`.

- `release_approved`: eligible as a technical-production source only when the exact asset ID, relative path, source Git object and SHA-256 all match. This decision does not claim that a complete runtime asset, scene, rig, animation set or game art package exists.
- `concept_only`: reference only. The release gate always rejects it for source admission.
- `reject`: prohibited from source admission and reference use through this gate.
- Historical source manifests and any `pending_review` record are `ARCHIVE`, have no runtime authority, and fail closed.

The release gate does not decode or render image data. It reads bytes only to compute SHA-256. Any missing manifest/file, unrecognized status, mismatched source path or SHA-256, duplicate identity, wrong authority, or malformed count is a hard failure. Only `release_approved` plus the matching digest passes.

The current decision set is 43 `release_approved`, 30 `concept_only`, and 21 `reject`. A SHA match is a provenance check, not a claim that the 43 approved sources amount to complete formal game art.
