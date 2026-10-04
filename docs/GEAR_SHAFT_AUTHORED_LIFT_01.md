# Gear Shaft authored lift and ground-save compatibility

## Scope and authority

This isolated slice starts from hosted `f00c4645fbbb38a6cba3fe6217534c45c948cb57`,
local `4694fa7d1e8e7b4ac56c75f0e1d01576aca4f5c5`, tree
`530235c09e7b702ebba71b0c2090ce17663f4b0c`. It uses the accepted generic support,
standing-deck, vertical combat and atomic presentation/readiness primitives.

The six-document gameplay specification (section 16.5, lines 986–990) requires
Gear Shaft lifts/platforms and contextual traversal before Air Step is earned.
The three-world plan's Gear Shaft row requires a checkpoint and retains three
Pressure Drones plus two Guards. Gameplay section 18 and engineering section 27
restrict Air Step to authored endpoints, capability checks and cooldowns. These
sources prescribe no exact dimensions or timings; all numbers below are
explicit development tuning.

## Authored route

- Existing five enemy IDs, types, home coordinates and all combat tuning stay
  unchanged. No new kill gate or world-completion event is added.
- `cw_gear_main_lift` occupies x=10..15, z=6..10. It cycles between y=0 and y=2,
  with 4-second travel and 2-second holds at each endpoint. Its phase comes from
  the retained authoritative clock, not a new scene-local clock.
- The upper docking deck occupies x=14..18.5, z=6..10 at y=2. Its 1m overlap
  with the lift supports full-foot dismount; the existing static-deck priority
  keeps a dismounted player on the deck when the lift descends.
- The Furnace landing occupies x=19.5..24, z=6..10 at y=2. The 1m gap has the
  ordinary `cw_gear_upper_gap` contextual traversal from (17.8,2,8) to
  (20.2,2,8), with no capability requirement. The established 1.2m activation
  range and 350ms cooldown are retained.
- The Furnace transition requires foot height 1.9..2.1. Standing at its X/Z on
  the lower floor cannot activate it. The existing Boiler return and lower
  checkpoint stay at ground level; an additional upper checkpoint is at
  (16,2,8). It does not heal, grant progression, or teleport a falling player.
- The lower y=0 recovery floor remains present. A missed jump or gap crossing
  falls onto that floor and requires another real ascent. This slice adds no
  lethal pit, falling damage, or height-extruded wall system.
- `cw_gear_earned_air_step` goes from (11,0,8) to (16,2,8), only from the
  0..0.1m band and only with `mobility.air_step_i`. It skips waiting for the
  lift after the genuine Shutdown reward; the main route never needs it.
  The existing Rust capability/source, endpoint and cooldown checks remain
  authoritative. This slice adds no acquisition or reward path.

The old three eventless staged interactions are replaced by actual support and
traversal definitions. Procedural support footprints, height posts and coherent
rider drawing remain temporary visuals. A later visual pass still needs lift,
chain/gear/deck art and final motion/audio polish; those assets do not block the
bounded functional implementation.

## Exact old ground-save upgrade

Current native installation records `gearShaftSupportVersion=1` in Clockworks
persistent state. Save V6's version number and existing support-frame schema
are unchanged. The absent-effect-sources-discriminator legacy profile explicitly
rejects presence of this new key, including zero or null.

Only the exact canonical Gear Shaft scene in the complete native registry can
upgrade version zero with an absent support frame. Every prerequisite must hold:

- grounded foot y=0, vertical velocity zero and no active dash/traversal;
- valid current KCC full-foot occupancy and contact;
- no upper checkpoint or new scene-completion record;
- the projected starting contact is unambiguously `floor`, with no support ID.

In particular, an old position inside the new lift's footprint while the lift
is in its lower hold is rejected. Continue cannot silently turn that old floor
save into a boarding state. The same X/Z can be admitted while the lift is
raised, because it remains on the actual lower floor.

The upgrade itself changes only the in-memory layout-version marker. It leaves
position, HP, time, actor records, grants, route state and original file/slot
bytes intact. It neither inserts a boarding record nor resets phase. Subsequent
projection/capture derives the current frame from the retained clock. Ordinary
Continue still performs its established epoch rebase and horizontal-input
clear; that lifecycle behavior is not a migration adjustment. The held-entry
snapshot must be drawn and acknowledged before simulation resumes.

Version one requires the exact existing support frame. Missing or mismatched
frames, future versions, unsafe old states and a noncanonical registry fail
closed before candidate installation. Generic support fixtures retain their
existing missing-frame rejection. There is no repositioning fallback.

## Verification and remaining gates

The accompanying tests pin the exact compiled geometry, all five actor homes,
main-route permission and earned shortcut declaration, repeatability of all 30
compiled scenes, source-bound presentation phases, old/current save acceptance
and rejection, original-byte preservation, and candidate-only mutation.

The public movement/save component fixture explicitly arranges the canonical
actors as already defeated. It exercises real input, lift contact, pause,
mid-ride and upper-deck slot Continue, checkpoint, ordinary contextual traversal,
safe fall/recovery and the height-bound exit. This is a mechanics test, not an
earned combat or complete journey proof. An earlier unarmed live-enemy fixture
waited on the lift without counterplay and died; its failed log is retained and
is not relabeled as success. Production HP, damage, spawns and warning timing
were not changed to make a test pass.

Final genuine acceptance must run the cue-aware journey with real enemy defeats,
pre-Air-Step ascent, genuine singleton Shutdown source, earned shortcut and
save/revisit assertions on the exact converged source. Native Tidebound and
Wraith encounters also remain explicit proof obligations. Earlier campaign and
Windows build results belong to their recorded source trees and are not
extended to this authored route. GPU/browser/Windows visual acceptance remains
separate from controlled renderer tests.
