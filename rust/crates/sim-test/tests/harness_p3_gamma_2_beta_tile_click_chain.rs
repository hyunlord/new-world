//! P3-γ-2-β harness — Tile Click + Causal Chain Rendering (V7 Phase 3-γ).
//!
//! γ-2-β consumes the γ-1 FFI (`get_tile_causal_history`) from a left-click
//! on the influence overlay sprite, mapped to a tile index via the locked
//! SPRITE_ORIGIN_X/SPRITE_ORIGIN_Y constants, and routes the resulting
//! `Array<Dictionary>` to `CausalPanel.display_history`, which renders a
//! header + per-event list in a `VBoxContainer`.
//!
//! This harness is purely source-token (Type S/A/D) — no Godot runtime. The
//! companion Godot-headless harness lives at
//! `scripts/test/p3_gamma_2_beta/harness_tile_click_chain.gd`.
//!
//! Run: `cargo test -p sim-test --test harness_p3_gamma_2_beta_tile_click_chain -- --nocapture`

// ── A1: world_renderer_sprite_origin_x ───────────────────────────────────────

/// Type A: `world_renderer.gd` declares `SPRITE_ORIGIN_X := 448` exactly once (P3γ2β-D3).
#[test]
fn harness_p3_gamma_2_beta_world_renderer_sprite_origin_x() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    let count = src.matches("SPRITE_ORIGIN_X := 448").count();
    assert_eq!(
        count, 1,
        "world_renderer.gd must declare `SPRITE_ORIGIN_X := 448` exactly once \
         (locked fact P3γ2β-D3); got {count}"
    );
    println!("A1 PASS: SPRITE_ORIGIN_X := 448 declared exactly once");
}

// ── A2: world_renderer_sprite_origin_y ───────────────────────────────────────

/// Type A: `world_renderer.gd` declares `SPRITE_ORIGIN_Y := 28` exactly once (P3γ2β-D3).
#[test]
fn harness_p3_gamma_2_beta_world_renderer_sprite_origin_y() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    let count = src.matches("SPRITE_ORIGIN_Y := 28").count();
    assert_eq!(
        count, 1,
        "world_renderer.gd must declare `SPRITE_ORIGIN_Y := 28` exactly once \
         (locked fact P3γ2β-D3); got {count}"
    );
    println!("A2 PASS: SPRITE_ORIGIN_Y := 28 declared exactly once");
}

// ── A3: world_renderer_defines_handle_tile_click ─────────────────────────────

/// Type A: `_handle_tile_click(` defined (P3γ2β-D4).
#[test]
fn harness_p3_gamma_2_beta_world_renderer_defines_handle_tile_click() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(
        src.contains("func _handle_tile_click("),
        "world_renderer.gd must define `_handle_tile_click(` (locked fact P3γ2β-D4)"
    );
    println!("A3 PASS: _handle_tile_click defined");
}

// ── A4: world_renderer_defines_fetch_causal_history ──────────────────────────

/// Type A: `_fetch_causal_history(` defined (P3γ2β-D4).
#[test]
fn harness_p3_gamma_2_beta_world_renderer_defines_fetch_causal_history() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(
        src.contains("func _fetch_causal_history("),
        "world_renderer.gd must define `_fetch_causal_history(` (locked fact P3γ2β-D4)"
    );
    println!("A4 PASS: _fetch_causal_history defined");
}

// ── A5: world_renderer_left_mouse_button_only ────────────────────────────────

/// Type A: `MOUSE_BUTTON_LEFT` present — left-click restriction (P3γ2β-D1).
#[test]
fn harness_p3_gamma_2_beta_world_renderer_left_mouse_button_only() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(
        src.contains("MOUSE_BUTTON_LEFT"),
        "world_renderer.gd must restrict click handling to `MOUSE_BUTTON_LEFT` \
         (locked fact P3γ2β-D1)"
    );
    println!("A5 PASS: MOUSE_BUTTON_LEFT branch present");
}

// ── A6: world_renderer_invokes_get_tile_causal_history ───────────────────────

/// Type A: `get_tile_causal_history(` invoked via FFI (P3γ2β-S1 / D4).
#[test]
fn harness_p3_gamma_2_beta_world_renderer_invokes_get_tile_causal_history() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(
        src.contains("get_tile_causal_history("),
        "world_renderer.gd must call `get_tile_causal_history(` (γ-1 FFI per P3γ2β-S1/D4)"
    );
    println!("A6 PASS: get_tile_causal_history invocation present");
}

// ── A7: world_renderer_does_not_consume_get_event_chain ──────────────────────

/// Type A: γ-2-β explicitly excludes `get_event_chain` (P3γ2β-S1).
#[test]
fn harness_p3_gamma_2_beta_world_renderer_does_not_consume_get_event_chain() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(
        !src.contains("get_event_chain"),
        "world_renderer.gd must NOT consume `get_event_chain` in γ-2-β (P3γ2β-S1)"
    );
    println!("A7 PASS: get_event_chain not consumed");
}

// ── A8: causal_panel_defines_display_history ─────────────────────────────────

/// Type A: `display_history(` public API (P3γ2β-D5).
#[test]
fn harness_p3_gamma_2_beta_causal_panel_defines_display_history() {
    let src = include_str!("../../../../scripts/ui/panels/causal_panel.gd");
    assert!(
        src.contains("func display_history("),
        "causal_panel.gd must define `display_history(` (locked fact P3γ2β-D5)"
    );
    println!("A8 PASS: display_history defined");
}

// ── A9: causal_panel_uses_vbox_container ─────────────────────────────────────

/// Type A: VBoxContainer used for chain layout (P3γ2β-D5).
#[test]
fn harness_p3_gamma_2_beta_causal_panel_uses_vbox_container() {
    let src = include_str!("../../../../scripts/ui/panels/causal_panel.gd");
    assert!(
        src.contains("VBoxContainer"),
        "causal_panel.gd must use `VBoxContainer` for chain rendering (locked fact P3γ2β-D5)"
    );
    println!("A9 PASS: VBoxContainer used");
}

// ── A10: causal_panel_defines_format_event ───────────────────────────────────

/// Type A: `_format_event(` helper present (P3γ2β-D5).
#[test]
fn harness_p3_gamma_2_beta_causal_panel_defines_format_event() {
    let src = include_str!("../../../../scripts/ui/panels/causal_panel.gd");
    assert!(
        src.contains("func _format_event("),
        "causal_panel.gd must define `_format_event(` (locked fact P3γ2β-D5)"
    );
    println!("A10 PASS: _format_event defined");
}

// ── A11: causal_panel_defines_channel_name ───────────────────────────────────

/// Type A: `_channel_name(` helper present (P3γ2β-D5).
#[test]
fn harness_p3_gamma_2_beta_causal_panel_defines_channel_name() {
    let src = include_str!("../../../../scripts/ui/panels/causal_panel.gd");
    assert!(
        src.contains("func _channel_name("),
        "causal_panel.gd must define `_channel_name(` (locked fact P3γ2β-D5)"
    );
    println!("A11 PASS: _channel_name defined");
}

// ── A12: causal_panel_reads_old_value_key ────────────────────────────────────

/// Type D: γ-1 FFI writes `old_value` (NOT `old`). Regression guard.
#[test]
fn harness_p3_gamma_2_beta_causal_panel_reads_old_value_key() {
    let src = include_str!("../../../../scripts/ui/panels/causal_panel.gd");
    assert!(
        src.contains("\"old_value\""),
        "causal_panel.gd must read `\"old_value\"` (γ-1 FFI key, NOT `\"old\"`)"
    );
    println!("A12 PASS: old_value key consumed");
}

// ── A13: causal_panel_reads_new_value_key ────────────────────────────────────

/// Type D: γ-1 FFI writes `new_value` (NOT `new`). Regression guard.
#[test]
fn harness_p3_gamma_2_beta_causal_panel_reads_new_value_key() {
    let src = include_str!("../../../../scripts/ui/panels/causal_panel.gd");
    assert!(
        src.contains("\"new_value\""),
        "causal_panel.gd must read `\"new_value\"` (γ-1 FFI key, NOT `\"new\"`)"
    );
    println!("A13 PASS: new_value key consumed");
}

// ── A14: compiled_en_has_all_13_keys ─────────────────────────────────────────

const NEW_KEYS: [&str; 13] = [
    "UI_CAUSAL_EVENT_BUILDING_PLACED",
    "UI_CAUSAL_EVENT_STAMP_DIRTY",
    "UI_CAUSAL_EVENT_INFLUENCE_CHANGED",
    "UI_CAUSAL_CHANNEL_WARMTH",
    "UI_CAUSAL_CHANNEL_LIGHT",
    "UI_CAUSAL_CHANNEL_NOISE",
    "UI_CAUSAL_CHANNEL_FOOD_AROMA",
    "UI_CAUSAL_CHANNEL_DANGER",
    "UI_CAUSAL_CHANNEL_SOCIAL",
    "UI_CAUSAL_CHANNEL_SPIRITUAL",
    "UI_CAUSAL_CHANNEL_BEAUTY",
    "UI_CAUSAL_NO_HISTORY",
    "UI_CAUSAL_TILE_HEADER",
];

/// Type A: All 13 keys present in compiled EN (P3γ2β-D6).
#[test]
fn harness_p3_gamma_2_beta_compiled_en_has_all_13_keys() {
    let src = include_str!("../../../../localization/compiled/en.json");
    let mut hits = 0;
    for k in NEW_KEYS.iter() {
        if src.contains(k) {
            hits += 1;
        } else {
            panic!("compiled/en.json missing key `{}` (locked fact P3γ2β-D6)", k);
        }
    }
    assert_eq!(hits, 13, "expected 13 distinct keys in compiled/en.json, got {}", hits);
    println!("A14 PASS: all 13 keys present in compiled/en.json");
}

// ── A15: compiled_ko_has_all_13_keys ─────────────────────────────────────────

/// Type A: All 13 keys present in compiled KO (P3γ2β-D6).
#[test]
fn harness_p3_gamma_2_beta_compiled_ko_has_all_13_keys() {
    let src = include_str!("../../../../localization/compiled/ko.json");
    let mut hits = 0;
    for k in NEW_KEYS.iter() {
        if src.contains(k) {
            hits += 1;
        } else {
            panic!("compiled/ko.json missing key `{}` (locked fact P3γ2β-D6)", k);
        }
    }
    assert_eq!(hits, 13, "expected 13 distinct keys in compiled/ko.json, got {}", hits);
    println!("A15 PASS: all 13 keys present in compiled/ko.json");
}

// ── A16: ko_translations_have_hangul ─────────────────────────────────────────

/// Type A: KO translation values contain Hangul characters (U+AC00–U+D7AF).
/// Catches accidental English fallback.
#[test]
fn harness_p3_gamma_2_beta_ko_translations_have_hangul() {
    let src = include_str!("../../../../localization/compiled/ko.json");
    // Parse minimally: find `"KEY": "value"` for each key, verify ≥1 Hangul.
    let mut hangul_pass = 0;
    for k in NEW_KEYS.iter() {
        let needle = format!("\"{}\":", k);
        let Some(idx) = src.find(&needle) else {
            panic!("KO key `{}` not found at all", k);
        };
        // Find next `"` after the colon to get value start.
        let rest = &src[idx + needle.len()..];
        let val_start = rest.find('"').expect("missing opening quote for value");
        let val_rest = &rest[val_start + 1..];
        let val_end = val_rest.find('"').expect("missing closing quote for value");
        let value = &val_rest[..val_end];
        // Check for at least one Hangul codepoint.
        let has_hangul = value.chars().any(|c| {
            let cp = c as u32;
            (0xAC00..=0xD7AF).contains(&cp)
        });
        if has_hangul {
            hangul_pass += 1;
        } else {
            panic!(
                "KO key `{}` value `{}` has no Hangul codepoint (looks like EN fallback)",
                k, value
            );
        }
    }
    assert_eq!(hangul_pass, 13, "expected 13 KO values with Hangul, got {}", hangul_pass);
    println!("A16 PASS: all 13 KO values contain Hangul");
}

// ── A17: key_registry_active_count_5116 ──────────────────────────────────────

/// Type A: `active_key_count` == 5116 (5103 + 13). Locked fact P3γ2β-D6.
#[test]
fn harness_p3_gamma_2_beta_key_registry_active_count_5116() {
    let src = include_str!("../../../../localization/key_registry.json");
    assert!(
        src.contains("\"active_key_count\": 5116"),
        "key_registry.json must have `active_key_count: 5116` (5103 + 13 per P3γ2β-D6)"
    );
    println!("A17 PASS: active_key_count == 5116");
}

// ── B18: world_renderer_space_branch_preserved ───────────────────────────────

/// Type D: SPACE channel-cycle branch preserved byte-for-byte (P3γ2β-D1).
#[test]
fn harness_p3_gamma_2_beta_world_renderer_space_branch_preserved() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(
        src.contains("if event.keycode == KEY_SPACE:"),
        "world_renderer.gd must preserve `if event.keycode == KEY_SPACE:` (locked fact P3γ2β-D1)"
    );
    println!("B18 PASS: SPACE branch preserved");
}

// ── B19: world_renderer_channel_switched_log ─────────────────────────────────

/// Type D: existing channel-cycle print preserved (P3γ2β-D1).
#[test]
fn harness_p3_gamma_2_beta_world_renderer_channel_switched_log() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    assert!(
        src.contains("Channel switched: "),
        "world_renderer.gd must preserve `Channel switched: ` log marker"
    );
    println!("B19 PASS: Channel switched log preserved");
}

// ── B20: causal_panel_q_toggle_preserved ─────────────────────────────────────

/// Type D: γ-2-α Q-toggle handler preserved (P3γ2β-D1).
#[test]
fn harness_p3_gamma_2_beta_causal_panel_q_toggle_preserved() {
    let src = include_str!("../../../../scripts/ui/panels/causal_panel.gd");
    assert!(
        src.contains("if event.keycode == KEY_Q:"),
        "causal_panel.gd must preserve `if event.keycode == KEY_Q:` (γ-2-α regression guard)"
    );
    println!("B20 PASS: Q-toggle preserved");
}

// ── B21: locale_compiled_loader_path_preserved ───────────────────────────────

/// Type D: Locale compiled-loader path preserved (γ-2-α regression).
#[test]
fn harness_p3_gamma_2_beta_locale_compiled_loader_path_preserved() {
    let src = include_str!("../../../../scripts/core/locale.gd");
    assert!(
        src.contains("localization/compiled/%s.json"),
        "locale.gd must preserve `localization/compiled/%s.json` loader path"
    );
    println!("B21 PASS: Locale loader path preserved");
}

// ── B22: main_tscn_causal_panel_mount_preserved ──────────────────────────────

/// Type D: γ-2-α scene mount preserved.
#[test]
fn harness_p3_gamma_2_beta_main_tscn_causal_panel_mount_preserved() {
    let src = include_str!("../../../../scenes/main.tscn");
    assert!(
        src.contains("2_causal"),
        "main.tscn must preserve `2_causal` ext_resource id (γ-2-α regression guard)"
    );
    println!("B22 PASS: scenes/main.tscn CausalPanel mount preserved");
}

// ── B23: total_ui_causal_keys_at_least_15 ────────────────────────────────────

/// Type D: total UI_CAUSAL_* occurrences in compiled/en.json ≥ 15.
/// γ-2-α added 2 (UI_CAUSAL_PANEL_*), γ-2-β adds 13. Total ≥ 15.
#[test]
fn harness_p3_gamma_2_beta_total_ui_causal_keys_at_least_15() {
    let src = include_str!("../../../../localization/compiled/en.json");
    let count = src.matches("UI_CAUSAL_").count();
    assert!(
        count >= 15,
        "compiled/en.json must have ≥15 UI_CAUSAL_ occurrences (got {})",
        count
    );
    println!("B23 PASS: UI_CAUSAL_ count={} (≥15)", count);
}
