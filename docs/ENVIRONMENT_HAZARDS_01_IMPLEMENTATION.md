# ENVIRONMENT-HAZARDS-01: authored periodic hazards and optional cooling

## Authority and limits

This implements the bounded missing environmental layer in exactly three existing
scenes. [Gameplay specification](RELEASE_V1_GAMEPLAY_ENGINEERING_SPEC_2026_09_28_SOURCE.md)
§11.4 requires directional short steam, area mist and optional Decon valves;
§16.4 requires Boiler heat accumulation, coolant and periodic vent warnings;
§16.6 requires Furnace Cooling/Heat; §20.4 makes live heat-resistance Effects
apply automatically. [Engineering supplement](RELEASE_V1_ENGINEERING_SUPPLEMENT_V1_SOURCE.md)
§§30–31 and 60–62 require generic HazardRules and data rather than item-ID branches.
The [three-world plan](RELEASE_V1_THREE_WORLDS_PLAN_V1_SOURCE.md),
[visual specification](RELEASE_V1_VISUAL_SPEC_V1_SOURCE.md) and
[UI specification](RELEASE_V1_UI_VISUAL_SPEC_V1_SOURCE.md) govern warning clarity,
layering and acceptance. The [Mist Harbor geometry/pump freeze](MIST_HARBOR_V1_GEOMETRY_PUMP_FREEZE_SOURCE.md)
is unchanged. No authority source text was modified.

Clockworks remains staged. No gate, required event, reward, resource progression,
actor roster, asset admission or completed campaign claim is added. The visual
feedback is explicitly `temporary_visual=true` procedural region/phase text;
missing approved final hazard artwork is not promoted or replaced by this work.

## Runtime and persistence

The generic Rust engine projects warning → active → recovery cycles and optional
suppression, using only authoritative simulation time. Heat exposure integrates
chronological phase segments in integer milliunits, preserving the same exposure
under subdivided input intervals, including caps and cooling. Damage requires
both current active phase and player containment; Heat also requires its exposure
threshold. At most one hit is emitted per authoritative step, without overdue
catch-up bursts. Live resolved HazardRules apply damage resistance, including
independent equipment/bloodline stacking and removal. No equipment or lineage ID
appears in the generic consumer.

Cooling is range/epoch/request validated, transactional, reusable after its own
cooldown and cannot complete progression. An independent shorter cooling control
cannot shorten an already earned suppression deadline; it can still lower heat.
After suppression, a complete safe warning begins before active danger. Revisit
time can dissipate heat without applying off-screen damage. Pause freezes the
owner clock, exposure, damage cooldowns and controls.

Save profile 2 gains optional, default-omitted environment state. IDs are checked
against the statically embedded, SHA-pinned canonical scene definitions; invalid
clock origins, values, unknown controls/hazards and nested extra fields are
rejected. This is schema/authority validation, not a cryptographic authenticity
claim about user-editable files. The unversioned legacy Save V6 profile remains
frozen: even null, empty or otherwise valid new environment fields are rejected.
Untouched legacy saves normalize in memory without rewriting original bytes.

## Epoch-bound transport

`formal_environment_control` sends only `id`, `requestId`, `worldEpoch` and requires
the target to be an authored environment control. It uses the existing scene
interaction transaction and returns its captured view/counters as one receipt,
without a later snapshot read. Legacy `formal_interact` refuses this new kind;
other existing interaction kinds retain their policy. A delayed Boiler command
cannot be rebound to Furnace even though both retain `cw_coolant_valve_staged`.
Replayed request IDs, payload changes, wrong kind, out-of-range, paused and stale
epoch calls cannot apply cooling. Frontend receipt identity and current-session
checks remain active.

## Authored implementation TUNE (not frozen specification values)

All values are editable TMJ JSON-string properties (`environment` and
`environmentControl`) and compiled deterministically. No runtime values are
chosen from scene IDs.

| Scene / stable hazard ID | Tag | Warning / active / recovery ms | Damage / interval ms | Heat exposure units |
|---|---|---|---|---|
| Decon / gh_decon_steam_jet_zone | pressure | 900 / 700 / 2400 | 8 / 700 | none |
| Decon / gh_decon_mist_zone | toxin | 1200 / 4000 / 1500 | 6 / 1000 | none |
| Boiler / cw_heat_accumulation_staged | heat | 1000 / 8000 / 3000 | 8 / 1000 | max100, +25/s, −20/s, threshold50 |
| Boiler / cw_boiler_vent_staged | pressure | 900 / 700 / 3400 | 10 / 700 | none |
| Furnace / cw_heat_accumulation_staged | heat | 1200 / 10000 / 3500 | 10 / 1000 | max100, +30/s, −18/s, threshold60 |

Decon valves retain their two existing IDs and positions; each suppresses its own
region for 6000ms, cooldown8000ms. Boiler coolant retains its ID/position, suppresses
both local hazards for6000ms and reduces Heat by40, cooldown8000ms. Furnace cooling
switch suppresses Heat for9000ms and reduces it by70, cooldown12000ms; Furnace
coolant suppresses6000ms/reduces40, cooldown8000ms. All ranges are2.5m.

The Decon south-directed steam footprint is x7..10,z4..6.5m, mist keeps
x14..18,z9..14m. Boiler Heat is x11..17,z2..6m and the vent x11..13,z10..13m.
Furnace Heat is x8..14,z2..6m. All player spawns and a0.35m-clear central Ground
lane at z8 stay outside these regions. Optional valves do not become hard locks.
Existing heat/vent marker IDs are moved to the hazard layer rather than duplicated.

## Compiler and presentation

Compiler and Rust validation reject incomplete, duplicate-key, unknown-field,
boolean/noninteger, out-of-range and mixed encounter/environment configurations;
kind/tag mismatches; unknown/repeated target IDs; progression events or aggregate
membership on environmental controls. Only three compiled JSON files change;
all other27 scenes reproduce byte-for-byte. Corresponding native SHA pins are
updated through the existing canonical compiler outputs.

Ground region text distinguishes warning, active, recovery and suppressed states
without relying on color. Heat percentage and optional control F prompts also
have DOM HUD text. Reduced motion disables pulsing; visible countdowns never
create or advance authority. Frozen paused overlays remain readable, and scene
invalidation/destroy releases both graphics and labels. No texture acquisition
or global asset ownership policy changes.

## Verification status

Final local validation is recorded with the candidate manifest and logs. It
covers generic boundaries, fixed-point exposure, overflow atomicity, controls,
actual owner-step damage, the real paused owner loop, authored regions/cooling,
live heat Effects, disk Save/Close/Continue in Decon, staged Clockworks disk
restoration plus explicit Continue refusal, strict legacy/current profiles,
compiler reproduction, frontend malformed projections, reduced motion, HUD,
cleanup and delayed source-epoch transport.

Real-source headless Rust tests exclude Tauri linking; browser-independent Node
checks are not Windows/native acceptance. Exact Windows CI must run after the
reviewed commit. Native GPU60FPS, visual approval, install/restart and complete
campaign playthrough remain separate outstanding gates.
