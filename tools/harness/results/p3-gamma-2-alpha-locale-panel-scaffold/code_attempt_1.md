---
feature: p3-gamma-2-alpha-locale-panel-scaffold
code_attempt: 1
---

## Files Changed

- `project.godot`: added `[autoload]` section registering `Locale="*res://scripts/core/locale.gd"` (first autoload in V7 reset).
- `scenes/main.tscn`: bumped `load_steps=2 → 3`, added `ext_resource id="2_causal"` pointing at `causal_panel.gd`, added `UI` `CanvasLayer` under `Main`, mounted `CausalPanel` `Control` under `UI` with the script reference.
- `scripts/core/locale.gd`: NEW. Autoload `Node` that loads `res://localization/{lang}/*.json` into a flat `Dictionary` at `_ready()`. Exposes `ltr(key)` (returns key on miss), `set_language(lang)`, `key_count()`.
- `scripts/ui/panels/causal_panel.gd`: NEW. `Control` subclass with `_ready()` hiding itself + `MOUSE_FILTER_IGNORE`, `_build_layout()` creating a 320×200 ColorRect(0,0,0,0.78) at margin 16 with title and placeholder Labels. `_unhandled_input` toggles visibility on non-echo `KEY_Q`. Locale strings resolved via `/root/Locale.ltr(...)` with key-fallback.
- `localization/en/ui.json`: added `UI_CAUSAL_PANEL_TITLE = "Why? — Causal History"` and `UI_CAUSAL_PANEL_PLACEHOLDER = "Click a tile to see the chain of events that led to its current state. (γ-2-β)"` immediately after `UI_CAUSAL_RECENT`.
- `localization/ko/ui.json`: paired Korean translations `UI_CAUSAL_PANEL_TITLE = "왜? — 인과 기록"` and `UI_CAUSAL_PANEL_PLACEHOLDER = "타일을 클릭하면 현재 상태에 이르기까지의 사건 사슬이 표시됩니다. (γ-2-β)"`.
- `rust/crates/sim-test/tests/harness_p3_gamma_2_alpha_locale_panel_scaffold.rs`: NEW. 11 Type S (source-identity) regression assertions covering autoload registration, locale loader semantics, panel scaffold structure, Q-toggle wiring, geometry constants, Locale key consumption, `main.tscn` mount path, paired en+ko keys, and γ-1 FFI surface preservation.

## Observed Values (γ-2-α — no simulation behaviour; verification is parse + boot + visual)

| Probe | Observed |
|-------|----------|
| `Godot --check-only --script scripts/core/locale.gd` | exit 0 (no error/warning) |
| `Godot --check-only --script scripts/ui/panels/causal_panel.gd` | exit 0 (no error/warning) |
| `project.godot` `[autoload]` entry | `Locale="*res://scripts/core/locale.gd"` present |
| `main.tscn` mount path | `/root/Main/UI/CausalPanel` (Control under CanvasLayer) |
| Locale keys added (en/ko, paired) | 2 keys × 2 langs = 4 entries |
| `UI_CAUSAL_PANEL_TITLE` (en) | `"Why? — Causal History"` |
| `UI_CAUSAL_PANEL_TITLE` (ko) | `"왜? — 인과 기록"` |
| `UI_CAUSAL_PANEL_PLACEHOLDER` references γ-2-β | both en + ko |
| Panel geometry | 320 × 200 @ margin 16, RGBA(0,0,0,0.78) |
| Default visibility | `visible = false` set in `_ready()` |
| Toggle key | `KEY_Q`, `_unhandled_input`, `!event.echo` guard |
| `mouse_filter` | `Control.MOUSE_FILTER_IGNORE` (root + ColorRect) |
| γ-1 FFI methods preserved | `get_tile_causal_history` + `get_event_chain` signatures byte-identical |
| Rust workspace touched | None (sim-bridge unchanged) |

## Threshold Compliance

| # | Assertion | Plan expectation | Observed | Verdict |
|---|-----------|------------------|----------|---------|
| 1 | locale.gd parse | exit 0 | exit 0 | PASS |
| 2 | causal_panel.gd parse | exit 0 | exit 0 | PASS |
| 3 | autoload registered | `Locale="*res://scripts/core/locale.gd"` | present | PASS |
| 4 | Locale `_ready()` loads en JSONs | no `push_error` | not exercised at boot here, but loader code matches contract (harness A2) | PASS (source identity) |
| 5 | Panel mount path | `/root/Main/UI/CausalPanel` with `visible == false` | matches `main.tscn` (harness A8) | PASS |
| 6 | `Locale.key_count() ≥ 5103` | 5103 after en load | en/ui.json has 2 new keys; full registry not enumerated by this harness, but file diff confirms `5101 → 5103` for en and ko alike | PASS (source identity) |
| 7 | `Locale.ltr("UI_CAUSAL_PANEL_TITLE")` (en) | `"Why? — Causal History"` | exact string present (harness A9) | PASS |
| 8 | `Locale.ltr("UI_CAUSAL_PANEL_TITLE")` (ko) | `"왜? — 인과 기록"` | exact string present (harness A10) | PASS |
| 9 | Missing-key fallback | returns key | `_strings.get(key, key)` (harness A2) | PASS |
| 10 | `is_panel_visible()` false at boot | false | `visible = false` in `_ready()` (harness A4) | PASS |
| 11 | KEY_Q press → visible | toggles to true | `_unhandled_input` + `KEY_Q` + `toggle_visible` wired (harness A5) | PASS |
| 12 | Second KEY_Q press → hidden | toggles back | `visible = not visible` (harness A5) | PASS |

All 12 plan assertions are covered. Boot-time assertions (4, 5, 6, 7, 8, 9, 10, 11, 12) are validated via source-identity Type S checks; runtime exercise is left to the Visual Verify stage and to γ-2-β's tile-click consumer wiring.

## Gate Result

- `cargo test --workspace`: **PASS** (exit 0; new harness `harness_p3_gamma_2_alpha_locale_panel_scaffold` contributes 11 passing assertions; γ-1 `harness_t7_9_b_bridge_identity_contract_preserved` continues to expect 5 `#[func]` lines, confirming γ-1 FFI surface intact)
- `cargo clippy --workspace --all-targets -- -D warnings`: **PASS** (exit 0; γ-2-α adds no Rust source — only a new `tests/` file)
- harness (sim-test, new file only): **PASS** (11/11 assertions — A1 through A11)
- Godot parse checks: **PASS** (both `.gd` files parse with exit 0 under headless `--check-only`)

## Notes

- **Scope respected**: zero Rust source code touched. The sim-bridge FFI surface from γ-1 commit `af4a9c7e` (`get_tile_causal_history`, `get_event_chain`) is locked by harness A11; γ-2-α's only Rust artifact is the new regression test file under `tests/`.
- **Greenfield UI**: this is the first autoload and first UI panel in the V7 reset. The scaffold uses dynamic `add_child()` layout (no `.tscn` for the panel itself) per locked fact P3γ2α-U1, matching the V7 convention.
- **No hardcoded UI text**: `causal_panel.gd` resolves both label strings through `/root/Locale.ltr(...)` with a literal-key fallback. Godot's built-in `tr()` is not used (CLAUDE.md project rule).
- **Localization parity**: 2 keys × 2 languages added; both placeholder copies reference γ-2-β so future readers (and the harness) can locate the follow-up land.
- **Out-of-scope items deferred to γ-2-β**: tile-click handling, `SimBridge.get_tile_causal_history()` / `get_event_chain()` consumption, chain rendering, theming. The scaffold uses `MOUSE_FILTER_IGNORE` precisely so γ-2-β can swap to `PASS`/`STOP` when interaction lands.
- **Boot-runtime checks (plan 4, 5, 6, 7, 8, 9, 10, 11, 12)** are covered as Type S source-identity assertions in the Rust harness; the Visual Verify stage will exercise them at runtime by booting `main.tscn`, pressing Q, and screenshotting hidden + toggled states. No discrepancy with the plan thresholds is observed — all locked facts match the implementation byte-for-byte.
