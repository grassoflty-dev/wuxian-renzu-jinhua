# Standing decks and height-qualified route commands: stage 1

## Scope and source

This generic server/compiler slice extends the moving-support primitive from hosted
`493e10a263e1b1eecf5e5507ad10bd264cb71376`. Its exact source baseline is hosted
`8543a9dc2a7ccb9113dc8c107b086e4a52e3a115`, tree
`b035ea3fffb86d41d5cf67a53f65df29c3c32e7a`; the later Swarm/Tidebound source is
outside this isolated verification. Integration must replay narrow hunks and keep
those later changes.

The six-document requirements call for a pre-AirStep Gear Shaft main route using
vertical traversal/lifts/platforms, with an optional authored AirStep route after
Shutdown. They do not supply deck heights, docking tolerances, or timing values.
The 2 m test deck, 1 m overlapping dock, [1.9, 2.1] m command bands, and lift timing
in these fixtures are development-test tuning, not final authored level design.

No canonical scene, enemy, reward, progression gate, combat damage, hazard, or
rendering asset is changed. The lower recovery floor remains at y=0. Collision
walls remain full-height XZ blockers. This is not a general 3D wall/ceiling system,
free flight, or the authored Gear Shaft lift admission.

## Authoritative support contract

- `standingDecks` contains immutable `{id, polygon, heightM}` records. The shared
  moving/static catalog is limited to 64 unique IDs. Polygons must be finite,
  strictly convex, nondegenerate, fully in bounds, and contain no repeated points;
  the footpoint envelope is 0 through 3 m.
- The player's whole circular foot must fit on a surface. Walking below a deck
  does not snap upward. Existing jump and dash cannot reach the test's 2 m deck.
  A supported landing is resolved on descent, and stepping off falls to the
  retained lower floor. Horizontal collision sweeps remain authoritative.
- At a coplanar overlap, a standing deck takes precedence over a moving lift,
  followed by stable ID ordering. A descending lift also lands on a crossed
  standing top. This prevents a docked rider from being pulled through the deck.
- Dock transfer needs sufficient safe footprint overlap; this primitive does not
  compute unions of adjacent polygons. The tests exercise the overlap at fixed
  60 Hz and test a missed dock. Authored content must validate its own geometry.

## Height-qualified route authority

Interactions, doors, triggers, checkpoints, and transitions may declare an
inclusive `heightRangeM: [minimum, maximum]`; contextual traversals use
`fromHeightRangeM`. Values must be finite and ordered within 0..3 m. Actionable
point anchors must lie in their band. Every actionable item in a scene containing
standing decks must explicitly declare its band. Existing flat and moving-only
scenes retain their prior optional-field behavior.

Elevated actionable anchors, spawns/navigation nodes, and traversal endpoints
require a stable standing deck with full-foot clearance. A moving pose is not a
stable spawn or endpoint. Command execution and corresponding route affordances
use the same band predicate. Traversal selection and execution both recheck the
start height. The old standalone traversal loader remains flat-only; supported
raised traversal is loaded through the validated full scene registry.

## Save and lifecycle contract

The existing optional SaveV5 `movingSupportFrame` now includes `stationary` poses
for standing decks. Stationary velocity and elapsed phase are exactly zero.
The fingerprint covers only authoritative support geometry/timing. The previous
moving-only catalog hash is byte-for-byte unchanged when no decks exist.
Changing or removing a static catalog rejects both file and slot Continue before
installing state or altering save bytes. Static-only scenes also preserve valid
airborne vertical velocity and exact next-step evolution through the explicit
entry readiness hold. Existing support envelope, epoch, shape, and frozen legacy
profile guards remain in force. No persisted schema discriminator is changed.

## Compiler contract

`logic.standing_deck` emits `standingDecks`; `logic.moving_support` emits
`movingSupports`. The compiler permits an optional stable `id` property, otherwise
using the object name. It rejects unknown support properties, malformed geometry,
combined count/ID violations, invalid timing, unsupported elevated endpoints,
and incomplete/reversed/nonfinite/wrong-layer vertical properties.

Point-layer `heightM` and traversal `toHeightM` are preserved in world Y rather
than flattened. Actionable `heightMinM`/`heightMaxM` compile to the exact range
fields. The new compiler's 30 canonical outputs were regenerated separately and
all 30 remain byte-identical. One new compiler output is pinned as a Rust-loader
fixture; Python checks its exact bytes and Rust checks its accepted height data.

## Verification and remaining gates

On the isolated headless adapter, the existing production Rust source is used;
only the native Tauri shell/dependency entry and native Web identity build gate
are excluded. Debug information and incremental compilation are disabled, without
changing game semantics. Each process uses a fresh temporary save directory.

- 9 new physics cases
- 8 new public-runtime/loader cases, including actual boarding/ride/docking,
  same-XZ below denial, legitimate raised traversal, file/slot restoration and
  malformed/catalog-change rollback
- 300 existing library cases, 11 moving-support physics, 10 moving-support runtime,
  3 contextual traversal, 8 conveyor physics, 5 conveyor runtime, 9 scene-runtime,
  10 SaveV5, and 23 capability/save-profile cases
- 42 compiler cases and exact 30/30 canonical-byte preservation

An initial fixture-only client timestamp regression and an initial compiler
property-parser mistake were corrected and their failed logs retained. The final
result must be read from the pinned package manifest, not those earlier logs.

Before authored admission, separate reviewed slices still need symmetric vertical
combat/hazard eligibility, atomic support+rider public projection and readiness,
placeholder presentation, actual Gear Shaft main-route and post-reward shortcut
content, followed by genuine traversal/campaign proof. No native rendering or
Windows runtime acceptance is claimed here.
