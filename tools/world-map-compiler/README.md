# World map compiler core

The compiler reads Tiled `.tmj` source maps and emits deterministic `SceneDefinition` JSON. It does not evaluate game state. Rust remains the authority for collision, navigation, objectives, hazards, transitions, and saves; Pixi consumes the presentation section.

```powershell
python tools/world-map-compiler/compile_maps.py `
  --maps design/maps `
  --runtime-manifest governance/assets/RUNTIME_ASSET_MANIFEST.json `
  --release-manifest governance/assets/AI_ASSET_RELEASE_MANIFEST.json `
  --progression server-rs/data/world_progression_v1.json `
  --entity-catalog content/enemies/entity-types.json `
  --out content/scenes/compiled
```

Use `--validate-all` instead of `--out` for CI. It validates and writes nothing. A campaign build with `--progression` requires scenes for each registered world and an emitting interaction or trigger for every required event. For a scoped package, `--allow-partial-campaign` permits `return_station` plus one or more progression-catalog worlds; it still checks all local links and requires each included catalog world to emit at least one canonical event. This mode does not weaken the default full-campaign gate or the separate one-world `--allow-incomplete-world-slice` mode, and those two modes cannot be combined.

Every TMJ map needs `worldId`, `sceneId`, and positive `pixelsPerMeter` map properties. Coordinates convert Tiled x/y pixels to world x/z meters. Its layer names are fixed:

```text
visual.background, visual.floor, visual.floor_detail, visual.props_back,
visual.props_dynamic, visual.foreground, visual.occluders, visual.vfx_markers
logic.collision, logic.navigation, logic.spawn, logic.interaction,
logic.door, logic.trigger, logic.hazard, logic.checkpoint, logic.camera,
logic.objective, logic.transition, logic.traversal, logic.walkable,
logic.exploration, logic.terrain, authoring.only
```

The optional Mist Harbor authoring layers compile `logic.walkable` objects to top-level `walkablePolygons`, `logic.exploration` objects to `explorationRegions`, and `logic.terrain` objects to `terrainRegions`. These keys are omitted when a map has no authored entries so older compiled scenes stay byte-for-byte unchanged. Terrain tags are closed to `terrain.water_deep` and `terrain.water_shallow`. A hazard may carry the same optional `terrainTag`; the compiler keeps its existing hazard projection and derives one matching terrain region from that same object, avoiding duplicate source geometry. `authoring.only` objects are checked for unique IDs, in-bounds coordinates, and an allowlisted authoring kind, then omitted from runtime output. This layer is for editor notes such as the pump level indicator and the deferred shortcut marker, not interactions, doors, or transitions.

This first compiler version accepts `objectgroup` layers with stable object names or `id` properties. Visual objects reference an admitted runtime `assetId`. The source release status and SHA must match the canonical 94-image ledger, the runtime item must have output provenance and a valid atlas page, and pages must obey the release texture limits. Enemy/NPC spawns require an `entityType` present in the optional catalog; if any enemy/NPC exists and no catalog is supplied, compilation fails. Navigation objects use comma-separated `neighbors` and the player spawn names its starting `navNode`. Checkpoints must be reachable from that node. Objectives name a `completingEvent` emitted by an interaction or trigger in the same scene. Doors and transitions name a target scene and spawn; cross-scene targets are checked across all input maps. A transition may set `requiresEvent` to a canonical progression event, which Rust checks before changing scenes.

`logic.traversal` objects compile into the canonical `logic.traversal` array. Their Tiled object position is `from`; `toX` and `toY` are destination pixel coordinates. `rangeM` is frozen to 1.2 and `cooldownMs` to 350. `requiresCapabilities` is a comma-separated list of canonical capability IDs (for example `mobility.air_step_i`) or an empty string. The compiler emits stable `id`, 3D `from`/`to` coordinates, range, cooldown, and sorted capability requirements. It rejects unknown or duplicate requirements, invalid bounds, collision-blocked endpoints, and paths whose 0.35 m player clearance intersects authored collision polygons. Traversal moves only between authored markers; it is not free air movement.

Tiled tile layers, image layers, and rotated objects are rejected until they receive explicit conversion rules. The compiler fails rather than dropping data silently. It also rejects duplicate IDs, invalid or out-of-bounds polygons, unknown assets or entity types, missing spawns, broken nav links, unreachable checkpoints, missing targets, and missing required progression events.

## Moving machinery authoring

A `logic.hazard` object with `kind=moving_machinery` requires integer `damage`
(1–10000), `periodMs` (100–60000), `warningMs` (1–10000), `phase=phase3`, meter
properties `translationXM` and `translationZM`, `speedMps` (0.1–20), and integer
`endpointHoldMs` (100–10000). The compiler emits `translationM: [x, z]` and the
other tuning fields unchanged. Translation must be finite, at least 0.01 m long,
and keep every polygon vertex in bounds at both ends of its linear path. Motion
properties are rejected on other hazard kinds. Collision and phase activation
remain Rust-owned; a hazard is not a terrain or movement-capability requirement.

The `cw_regulator_reciprocating_press` implementation TUNE uses a 2 × 1.5 m
polygon at x=8..10, z=11.5..13, translated 6 m east at 2 m/s, then back on the
same path. It deals 16 damage with a 750 ms damage interval while travelling,
starts with a safe 900 ms warning, and has a safe 600 ms warning/hold at each
endpoint. The repeating motion cycle is derived as
`2 * distance / speed + 2 * endpointHold = 7200 ms`; no redundant cycle field is
stored. Phase 3 begins at the implementation TUNE of 30% Boss HP. These are
implementation choices, not values frozen by the source authority. The whole
sweep remains in the south side lane, clear of spawn and the z=7..9 central
Ground route. The core console retains its established authored ID and emits
`clockworks_core` only through the Rust defeat-gated interaction; full compiler
event coverage does not promote Clockworks out of staged admission.

Run unit tests:

```powershell
python -m unittest discover -s tools/world-map-compiler -p 'test_*.py' -v
```

## Periodic environment authoring

`logic.hazard` can carry an `environment` JSON-string property containing a strict
nested periodic warning/active/recovery configuration. `logic.interaction` with
`kind=environment_control` requires an `environmentControl` JSON-string property,
explicit `rangeM` and `cooldownMs`. It may target only same-scene environmental
hazards and cannot emit progression events or join an interaction aggregate.
Encounter timing/motion fields cannot be mixed with an environmental config.
See [the engine, schema and authored TUNE](../../docs/ENVIRONMENT_HAZARDS_01_IMPLEMENTATION.md).
Omitting these properties keeps existing maps byte-identical.

## Generic vertical support authoring

`logic.standing_deck` polygon objects require `heightM` (0..3 m).
`logic.moving_support` polygons require `lowerM`, `upperM`, `travelMs`
(250..60000), and `endpointHoldMs` (0..30000); their span is at least 0.1 m.
Each accepts an optional stable `id`, otherwise its object name is used. Unknown
support properties, nonconvex polygons, and a combined catalog above 64 fail.

Point layers may use `heightM`; contextual traversal uses `toHeightM` for its
endpoint. Actionable layers use both `heightMinM` and `heightMaxM`, compiling to
`heightRangeM` (or traversal `fromHeightRangeM`). All actionable objects in a
standing-deck scene require explicit ranges. Elevated stable anchors/endpoints
need full-foot standing-deck clearance. Wrong-layer or misspelled vertical
properties fail rather than being discarded. The lower y=0 recovery floor stays
present. See `docs/STANDING_DECK_HEIGHT_STAGE_01.md` for the authoritative contract,
compatibility limits, and remaining projection/combat/authored-content gates.

Hazards also accept paired `heightMinM`/`heightMaxM`, compiled to `heightRangeM`.
Standing-deck scenes must supply the range for every hazard. The authority uses
finite inclusive footpoint bands; legacy floor hazards default to [0,1] m. This
adds no hazard to existing content. See `docs/VERTICAL_COMBAT_HEIGHT_STAGE_02.md`.

Gear Shaft now authors one lift, two upper decks, a no-capability main-route gap,
an upper checkpoint and an earned Air Step shortcut. Exact development tuning,
save compatibility and remaining genuine-route/visual gates are in
[GEAR_SHAFT_AUTHORED_LIFT_01.md](../../docs/GEAR_SHAFT_AUTHORED_LIFT_01.md).
