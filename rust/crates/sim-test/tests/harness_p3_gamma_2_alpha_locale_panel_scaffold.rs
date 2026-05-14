//! P3-γ-2-α harness — Locale autoload + CausalPanel scaffold (V7 Phase 3-γ).
//!
//! γ-2-α delivers the first-ever UI substrate in the V7 reset:
//!   * Locale autoload (`scripts/core/locale.gd`) loading
//!     `localization/{lang}/*.json` into one flat key→value Dictionary.
//!   * CausalPanel `Control` scaffold (`scripts/ui/panels/causal_panel.gd`)
//!     mounted in `scenes/main.tscn` under a `UI` `CanvasLayer`, hidden by
//!     default, toggled visible with Q.
//!   * Paired locale keys `UI_CAUSAL_PANEL_TITLE` and
//!     `UI_CAUSAL_PANEL_PLACEHOLDER` in both `en/ui.json` and `ko/ui.json`.
//!
//! No Rust code or simulation behaviour is touched; the FFI surface from
//! γ-1 (af4a9c7e) is preserved unmodified.
//!
//! All assertions are Type S (source identity) using `include_str!` — they
//! fail at compile time if a file is missing and at runtime if required
//! tokens are absent. Godot parse-level + boot verification is performed
//! out-of-band (see plan §Verification, points 1–12).
//!
//! Run: `cargo test -p sim-test --test harness_p3_gamma_2_alpha_locale_panel_scaffold -- --nocapture`

// ── A1: project_godot_locale_autoload_registered ─────────────────────────────

/// Type S: `project.godot` registers the Locale autoload as a singleton.
///
/// The `*` prefix marks it as an autoload Node accessible via `/root/Locale`.
/// First-ever autoload in the V7 reset — establishes the single-autoload
/// precedent for γ-2-α (locked fact P3γ2α-L1).
///
/// ticks: 0 (source-only check)
#[test]
fn harness_p3_gamma_2_alpha_project_godot_locale_autoload_registered() {
    let src = include_str!("../../../../project.godot");

    assert!(
        src.contains("[autoload]"),
        "project.godot must contain an [autoload] section (γ-2-α: first autoload)"
    );
    assert!(
        src.contains("Locale=\"*res://scripts/core/locale.gd\""),
        "project.godot must register `Locale=\"*res://scripts/core/locale.gd\"` \
         (the leading `*` marks it as an autoload Node singleton — locked fact P3γ2α-L1)"
    );

    println!("A1 PASS: [autoload] Locale singleton registered");
}

// ── A2: locale_gd_loads_json_with_fallback ───────────────────────────────────

/// Type S: `scripts/core/locale.gd` loads `localization/{lang}/*.json` into
/// one flat `Dictionary` and exposes `ltr(key)` with key-fallback semantics.
///
/// Locked facts P3γ2α-L2 (flat-JSON loader, default `en`) and P3γ2α-L3
/// (`ltr(key)` returns key on miss — no Godot `tr()` involvement).
///
/// ticks: 0 (source-only check)
#[test]
fn harness_p3_gamma_2_alpha_locale_gd_loads_json_with_fallback() {
    let src = include_str!("../../../../scripts/core/locale.gd");

    assert!(
        src.contains("extends Node"),
        "locale.gd must `extends Node` (autoload singleton lives in the scene tree)"
    );
    assert!(
        src.contains("_current_lang: String = \"en\""),
        "locale.gd must default `_current_lang` to \"en\" (locked fact P3γ2α-L2)"
    );
    assert!(
        src.contains("res://localization/%s/"),
        "locale.gd must scan `res://localization/{{lang}}/` for JSON files \
         (locked fact P3γ2α-L2)"
    );
    assert!(
        src.contains(".ends_with(\".json\")"),
        "locale.gd must filter directory entries to `.json` files"
    );
    assert!(
        src.contains("JSON.parse_string"),
        "locale.gd must use `JSON.parse_string` to parse each locale file"
    );
    assert!(
        src.contains("func ltr(key: String) -> String"),
        "locale.gd must expose `func ltr(key: String) -> String` (locked fact P3γ2α-L3)"
    );
    assert!(
        src.contains("_strings.get(key, key)"),
        "locale.gd `ltr` must fall back to the key itself on miss \
         (dev visibility fallback — locked fact P3γ2α-L3)"
    );

    println!("A2 PASS: locale.gd flat-JSON loader + ltr() fallback");
}

// ── A3: locale_gd_public_api_complete ────────────────────────────────────────

/// Type S: `locale.gd` exposes the three γ-2-α public methods.
///
/// `ltr(key)` is consumed by γ-2-α; `set_language(lang)` and `key_count()`
/// exist for γ-2-β and beyond (locked fact at the end of the "Locked facts"
/// list — "exposes 3 public methods").
///
/// ticks: 0 (source-only check)
#[test]
fn harness_p3_gamma_2_alpha_locale_gd_public_api_complete() {
    let src = include_str!("../../../../scripts/core/locale.gd");

    assert!(
        src.contains("func ltr(key: String) -> String"),
        "locale.gd must expose `func ltr(key: String) -> String`"
    );
    assert!(
        src.contains("func set_language(lang: String) -> void"),
        "locale.gd must expose `func set_language(lang: String) -> void` (γ-2-β consumer)"
    );
    assert!(
        src.contains("func key_count() -> int"),
        "locale.gd must expose `func key_count() -> int` (γ-2-β consumer)"
    );

    println!("A3 PASS: locale.gd exposes ltr + set_language + key_count");
}

// ── A4: causal_panel_gd_scaffold_present ─────────────────────────────────────

/// Type S: `scripts/ui/panels/causal_panel.gd` is a `Control` subclass that
/// hides itself, refuses mouse capture, and builds its layout dynamically.
///
/// Locked facts P3γ2α-U1 (Control subclass, dynamic add_child layout, no
/// `.tscn`), P3γ2α-U3 (default `visible = false`), and P3γ2α-U4
/// (`mouse_filter = MOUSE_FILTER_IGNORE` on root + background).
///
/// ticks: 0 (source-only check)
#[test]
fn harness_p3_gamma_2_alpha_causal_panel_gd_scaffold_present() {
    let src = include_str!("../../../../scripts/ui/panels/causal_panel.gd");

    assert!(
        src.contains("extends Control"),
        "causal_panel.gd must `extends Control` (locked fact P3γ2α-U1)"
    );
    assert!(
        src.contains("visible = false"),
        "causal_panel.gd `_ready` must set `visible = false` \
         (locked fact P3γ2α-U3: hidden by default until Q is pressed)"
    );
    assert!(
        src.contains("mouse_filter = Control.MOUSE_FILTER_IGNORE"),
        "causal_panel.gd root must set `mouse_filter = Control.MOUSE_FILTER_IGNORE` \
         (locked fact P3γ2α-U4: γ-2-α scaffold does not capture clicks)"
    );
    assert!(
        src.contains("ColorRect.new()"),
        "causal_panel.gd must instantiate a `ColorRect` background dynamically \
         (locked fact P3γ2α-U1: no .tscn, layout built in code)"
    );
    assert!(
        src.contains("Label.new()"),
        "causal_panel.gd must instantiate Labels dynamically for title and placeholder"
    );

    println!("A4 PASS: causal_panel.gd Control scaffold (hidden, ignore mouse, dynamic layout)");
}

// ── A5: causal_panel_gd_q_toggle_wired ───────────────────────────────────────

/// Type S: causal_panel.gd toggles visibility on a non-echo Q key press.
///
/// Locked fact P3γ2α-U3 — `_unhandled_input` recognises
/// `InputEventKey.pressed && !echo && keycode == KEY_Q`.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_p3_gamma_2_alpha_causal_panel_gd_q_toggle_wired() {
    let src = include_str!("../../../../scripts/ui/panels/causal_panel.gd");

    assert!(
        src.contains("func _unhandled_input(event: InputEvent) -> void"),
        "causal_panel.gd must override `_unhandled_input(event: InputEvent)`"
    );
    assert!(
        src.contains("event is InputEventKey"),
        "_unhandled_input must check `event is InputEventKey`"
    );
    assert!(
        src.contains("event.pressed") && src.contains("not event.echo"),
        "_unhandled_input must require `event.pressed and not event.echo` \
         (locked fact P3γ2α-U3: ignore key repeats)"
    );
    assert!(
        src.contains("KEY_Q"),
        "_unhandled_input must match `KEY_Q` (locked fact P3γ2α-U3: Q toggles panel)"
    );
    assert!(
        src.contains("func toggle_visible() -> void"),
        "causal_panel.gd must expose `func toggle_visible() -> void`"
    );
    assert!(
        src.contains("visible = not visible"),
        "toggle_visible must flip `visible = not visible`"
    );
    assert!(
        src.contains("func is_panel_visible() -> bool"),
        "causal_panel.gd must expose `func is_panel_visible() -> bool` \
         (read-only probe for tests + γ-2-β consumers)"
    );

    println!("A5 PASS: causal_panel.gd Q-key toggle wired through _unhandled_input");
}

// ── A6: causal_panel_gd_geometry_locked ──────────────────────────────────────

/// Type S: causal_panel.gd uses the locked geometry constants — 320×200 box
/// at margin 16 in the top-left of the CanvasLayer.
///
/// Plan §"Other locked facts" — "fixed 320×200 box at margin 16, positioned
/// in the top-left of the viewport (CanvasLayer default origin)". γ-2-β may
/// relocate/resize, but γ-2-α scaffold values are the contract.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_p3_gamma_2_alpha_causal_panel_gd_geometry_locked() {
    let src = include_str!("../../../../scripts/ui/panels/causal_panel.gd");

    assert!(
        src.contains("PANEL_WIDTH := 320.0") || src.contains("PANEL_WIDTH: float = 320.0"),
        "causal_panel.gd must declare PANEL_WIDTH = 320.0 (γ-2-α locked geometry)"
    );
    assert!(
        src.contains("PANEL_HEIGHT := 200.0") || src.contains("PANEL_HEIGHT: float = 200.0"),
        "causal_panel.gd must declare PANEL_HEIGHT = 200.0 (γ-2-α locked geometry)"
    );
    assert!(
        src.contains("PANEL_MARGIN := 16.0") || src.contains("PANEL_MARGIN: float = 16.0"),
        "causal_panel.gd must declare PANEL_MARGIN = 16.0 (γ-2-α locked geometry)"
    );
    assert!(
        src.contains("Color(0.0, 0.0, 0.0, 0.78)"),
        "causal_panel.gd background must be RGBA(0, 0, 0, 0.78) \
         (plan §In-game verification)"
    );

    println!("A6 PASS: causal_panel.gd geometry locked (320×200 @ margin 16, RGBA 0,0,0,0.78)");
}

// ── A7: causal_panel_gd_uses_locale_keys ─────────────────────────────────────

/// Type S: causal_panel.gd labels its title and placeholder via the
/// `UI_CAUSAL_PANEL_TITLE` and `UI_CAUSAL_PANEL_PLACEHOLDER` Locale keys.
///
/// Locked fact P3γ2α-K1 — keys are consumed via `Locale.ltr(KEY)`, not
/// hardcoded strings (CLAUDE.md localisation rule).
///
/// ticks: 0 (source-only check)
#[test]
fn harness_p3_gamma_2_alpha_causal_panel_gd_uses_locale_keys() {
    let src = include_str!("../../../../scripts/ui/panels/causal_panel.gd");

    assert!(
        src.contains("UI_CAUSAL_PANEL_TITLE"),
        "causal_panel.gd must reference the `UI_CAUSAL_PANEL_TITLE` locale key \
         (locked fact P3γ2α-K1)"
    );
    assert!(
        src.contains("UI_CAUSAL_PANEL_PLACEHOLDER"),
        "causal_panel.gd must reference the `UI_CAUSAL_PANEL_PLACEHOLDER` locale key \
         (locked fact P3γ2α-K1)"
    );
    assert!(
        src.contains("/root/Locale") || src.contains("Locale"),
        "causal_panel.gd must resolve text through the Locale autoload \
         (no hardcoded UI strings — CLAUDE.md localisation rule)"
    );
    assert!(
        !src.contains(".tr("),
        "causal_panel.gd must NOT use Godot's built-in `tr()` \
         (project rule: custom Locale autoload only)"
    );

    println!("A7 PASS: causal_panel.gd consumes UI_CAUSAL_PANEL_* via Locale (no hardcoded text)");
}

// ── A8: main_tscn_mounts_causal_panel ────────────────────────────────────────

/// Type S: `scenes/main.tscn` mounts CausalPanel under `UI` `CanvasLayer`.
///
/// Locked fact P3γ2α-U2 — `Main` gains a `UI` `CanvasLayer` child, and
/// `CausalPanel` `Control` is mounted under `UI` with the
/// `causal_panel.gd` script as `ext_resource id="2_causal"`. `load_steps`
/// must reflect the new resource.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_p3_gamma_2_alpha_main_tscn_mounts_causal_panel() {
    let src = include_str!("../../../../scenes/main.tscn");

    assert!(
        src.contains("load_steps=3"),
        "main.tscn must declare `load_steps=3` after γ-2-α adds the panel ext_resource \
         (locked fact P3γ2α-U2)"
    );
    assert!(
        src.contains("res://scripts/ui/panels/causal_panel.gd")
            && src.contains("id=\"2_causal\""),
        "main.tscn must register `causal_panel.gd` as `ext_resource id=\"2_causal\"` \
         (locked fact P3γ2α-U2)"
    );
    assert!(
        src.contains("[node name=\"UI\" type=\"CanvasLayer\" parent=\".\"]"),
        "main.tscn must add `UI` CanvasLayer under `Main` \
         (locked fact P3γ2α-U2: CanvasLayer renders in screen space)"
    );
    assert!(
        src.contains("[node name=\"CausalPanel\" type=\"Control\" parent=\"UI\"]"),
        "main.tscn must mount `CausalPanel` Control under `UI` \
         (locked fact P3γ2α-U2: /root/Main/UI/CausalPanel)"
    );
    assert!(
        src.contains("script = ExtResource(\"2_causal\")"),
        "main.tscn `CausalPanel` node must attach `ExtResource(\"2_causal\")` \
         (the causal_panel.gd script reference)"
    );

    println!("A8 PASS: main.tscn mounts /root/Main/UI/CausalPanel with causal_panel.gd");
}

// ── A9: locale_en_panel_keys_present ─────────────────────────────────────────

/// Type S: `localization/en/ui.json` contains both `UI_CAUSAL_PANEL_TITLE`
/// and `UI_CAUSAL_PANEL_PLACEHOLDER` with the exact English copy from
/// locked fact P3γ2α-K1.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_p3_gamma_2_alpha_locale_en_panel_keys_present() {
    let src = include_str!("../../../../localization/en/ui.json");

    assert!(
        src.contains("\"UI_CAUSAL_PANEL_TITLE\": \"Why? — Causal History\""),
        "en/ui.json must contain `UI_CAUSAL_PANEL_TITLE: \"Why? — Causal History\"` \
         (locked fact P3γ2α-K1)"
    );
    assert!(
        src.contains("\"UI_CAUSAL_PANEL_PLACEHOLDER\""),
        "en/ui.json must contain `UI_CAUSAL_PANEL_PLACEHOLDER` (locked fact P3γ2α-K1)"
    );
    assert!(
        src.contains("(γ-2-β)"),
        "en/ui.json placeholder copy must reference the γ-2-β follow-up land \
         (locked fact P3γ2α-K1: copy mentions consumption path)"
    );

    println!("A9 PASS: en/ui.json carries UI_CAUSAL_PANEL_TITLE + UI_CAUSAL_PANEL_PLACEHOLDER");
}

// ── A10: locale_ko_panel_keys_paired ─────────────────────────────────────────

/// Type S: `localization/ko/ui.json` contains the same two keys with their
/// Korean translations (locked fact P3γ2α-K1 — paired translations are a
/// project mandate).
///
/// ticks: 0 (source-only check)
#[test]
fn harness_p3_gamma_2_alpha_locale_ko_panel_keys_paired() {
    let src = include_str!("../../../../localization/ko/ui.json");

    assert!(
        src.contains("\"UI_CAUSAL_PANEL_TITLE\": \"왜? — 인과 기록\""),
        "ko/ui.json must contain `UI_CAUSAL_PANEL_TITLE: \"왜? — 인과 기록\"` \
         (locked fact P3γ2α-K1)"
    );
    assert!(
        src.contains("\"UI_CAUSAL_PANEL_PLACEHOLDER\""),
        "ko/ui.json must contain `UI_CAUSAL_PANEL_PLACEHOLDER` (locked fact P3γ2α-K1)"
    );
    assert!(
        src.contains("(γ-2-β)"),
        "ko/ui.json placeholder copy must reference the γ-2-β follow-up land"
    );

    println!("A10 PASS: ko/ui.json carries paired translations for both keys");
}

// ── A11: gamma_1_ffi_surface_preserved ───────────────────────────────────────

/// Type S: γ-1's FFI surface (`get_tile_causal_history`, `get_event_chain`)
/// is unchanged.
///
/// Locked fact (Other locked facts §1) — "γ-2-α touches NO Rust code. The
/// sim-bridge FFI surface from γ-1 is unchanged." γ-2-β will consume these
/// methods; γ-2-α must leave them untouched.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_p3_gamma_2_alpha_gamma_1_ffi_surface_preserved() {
    let src = include_str!("../../sim-bridge/src/ffi/world_node.rs");

    assert!(
        src.contains("fn get_tile_causal_history(&self, x: i32, y: i32) -> VarArray"),
        "γ-1 `get_tile_causal_history` signature must be byte-identical \
         (γ-2-α must not touch sim-bridge)"
    );
    assert!(
        src.contains("fn get_event_chain(&self, x: i32, y: i32, event_id: i64) -> VarArray"),
        "γ-1 `get_event_chain` signature must be byte-identical \
         (γ-2-α must not touch sim-bridge)"
    );

    println!("A11 PASS: γ-1 FFI surface (get_tile_causal_history, get_event_chain) preserved");
}
