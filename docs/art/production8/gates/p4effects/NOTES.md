# Part 4 A24 effect verification

Owned product files: `scripts/rendering/art/god_hand_effects.gd`, `scripts/test/art/test_intervention_world_effects.gd`, `scripts/test/art/test_god_hand.gd`.

- Spoil food: six native-pixel brown (#6B4E2A) rising particles and brown sprite tint, 1.2 seconds.
- Rot wood: four native-pixel grey particles and sprite base color multiplied by #5A4A3A, 1.2 seconds.
- Both gifts: six light falling particles on the actual respective pile, 1.2 seconds.
- Food targeting mirrors renderer Chebyshev nearest-stockpile assignment with building-ID tie order; wood uses the actual camp prop belonging to target formation's hearth. No absent pile is fabricated.
- `notify_cast` caches the existing real sprite location after the synchronous intervention but before renderer refresh. A depleted/freed pile retains particles at its last real location; no phantom sprite is created. Newly rendered gift piles are resolved during observation if no pile was cached.
- Absolute `log_ids` and actual `log_settlement_ids` prevent same-tick cast collisions. Missing-ID overlap fallback is retained for existing legacy fixtures only.
- Windfall dust stays at alpha 1 for 1.0 second then fades for 0.3 seconds. Drop/bounce still ends at 0.8 seconds. Existing rain, cold snap and stockpile-label contracts retained.

Verification: scoped strict check passed for all three owned scripts (16 FFI calls matched); `INTERVENTION_WORLD_EFFECTS failures=0`, `GOD_HAND failures=0`, `SETTLEMENT_SELECT failures=0`. Raw logs in this directory. No Rust/cargo, full gate, windowed capture, commit or push was run in this slice. Actual screenshot acceptance remains with parent integration.

`intervention_effects.world_effects[]` exports event_id, kind, settlement_id, age, duration, target_found, pile_tile, pile_art_id, sprite_alive, particles (world-global position, native size, RGB color and alpha), and current tint. Existing metadata keys are preserved. Windfall dust adds hold_seconds and fade_seconds.
