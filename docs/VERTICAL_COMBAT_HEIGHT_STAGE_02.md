# Symmetric vertical combat and environmental bands: stage 2

This source slice is based on hosted `3397c599d03e2f7ab4b551b41e8c95a3e19b639f`,
local `1699ca17d387ab9dac209d26c1ca3503f594fd24`, tree
`9332b0a4c57fdd9f77a003db08780193d4104830`. It retains the accepted standing-deck
stage. Later native Tidebound placements/presentation and Wraith work are outside
this isolated baseline and must be retained when replaying narrow integration
hunks. No actor profile, HP, damage, cooldown, reward, progression gate, or
canonical scene is tuned by this slice.

## Combat height contract

`continuous_combat::combat_vertical_overlap` is the shared finite, symmetric
predicate for a maximum inclusive 1.0 m footpoint-height difference. This is
explicit development tuning, not a number supplied by the six source documents
or a claim of physically exact 3D capsules. The ordinary 5.5 m/s jump under
16 m/s² gravity remains below 1 m. A 2 m deck separates attacks between floors.

The predicate applies to player legacy/Primary/Pulse/Pierce selection and impact,
living group-member targets, ordinary actor sensing and every Bite/Shot/Leap/
Charge/Slam/Lunge/Swing hit, legacy melee/pressure waves, Sentinel attacks, and
both committed Warden attacks. Existing horizontal ranges, sectors, directional
commitment, LOS/collision, damage and Guard rules remain intact. An attack that
was legitimately warned on one floor rechecks height at impact. A different-floor
player neither takes damage nor consumes a pressure projectile; the shot retains
its captured origin Y and can subsequently hit an eligible target.

## Stable actor surfaces

Actor standing checks are separate from LOS and projectile occupancy. Ground
controllers retain their existing floor collision behavior. Elevated actors need
full-foot support from a static standing deck, at their authored home elevation.
They do not board moving lifts, fall, or gain free flight in this bounded model.
Ordinary walking and body motion stop before leaving supported geometry; legacy
and Warden candidates and Sentinel movement receive the corresponding guard.

Initial elevated spawns validate the actor's actual profile footprint, not only
the generic player-radius scene check. Capture and both file/slot Continue reject
unsupported or cross-plane actor positions before installing state/serializing a
new save. Sentinel restored Y is bound to its source spawn's plane. These are
validation rules over existing fields; no new actor save field is introduced.

## Environmental height

Hazards optionally declare `heightRangeM`, using the same strict finite 0..3 m
range type as route authority. Standing-deck scenes require it explicitly for
every hazard. Old floor hazards default to [0, 1] m footpoint eligibility. The
predicate is checked before environmental exposure/damage, phase-2 heat damage,
and moving-machinery damage/cooldown. Phase clocks continue normally while the
player is outside the band. Explicit ranges are projected with the hazard's
height and range; default flat JSON retains its previous optional-field shape.

Tiled hazard `heightMinM`/`heightMaxM` compile to this range. Terrain profile
sampling and slowdown also use the floor band, so an actor on a dry raised deck
does not inherit floor-water combat tuning. Deep-water admission/collision
permissions remain conservative existing restrictions; this slice does not
silently enable a bridge over forbidden deep water or define a new falling-into-
water contract. No native scene currently authors such a bridge.

## Verification boundary

Focused geometry tests cover symmetric same-floor hits/cross-floor misses,
committed warnings, every ordinary attack kind, living group-member aim, both
Warden attacks, legacy pressure/melee/Sentinel, projectile non-consumption,
actor edge support and dry raised Tidebound profile selection. Public-runtime
fixtures physically board/ride/dock, demonstrate separated attacks above and real
counterplay after returning to the lower floor, exercise explicit hazard bands,
and reject malformed source or forged file/slot actor positions atomically.
Private unit fixtures test direct capture and machinery cooldown without using
health injection in a campaign driver. Initial fixture aim at a Swarm center
missed Pierce; the test now aims at the actual public living member position,
with the original failure retained.

Selected baseline combat/support/scene/save/profile regressions, strict compiler
checks, and 30 canonical byte-preservation checks are recorded in the final
manifest. The source is exercised through the explicit headless adapter; no new
Windows execution, renderer visual or authored Gear Shaft route claim follows.

Before lift admission, stage 3 still needs an atomic support+rider projection,
matched presentation/readiness, then authored pre-AirStep Gear Shaft progression
and its earned post-Shutdown shortcut, followed by genuine route validation.
