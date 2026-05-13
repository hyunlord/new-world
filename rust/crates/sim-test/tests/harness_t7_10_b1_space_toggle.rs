//! T7.10.B1 — SPACE-key channel toggle anti-circular source guard.
//!
//! T7.10.B1 wires a SPACE-key toggle in `scripts/ui/world_renderer.gd` that
//! flips the visible influence channel between Warmth (T7.10.A) and Light
//! (T7.10.B). The toggle is a pure GDScript edit — no Rust / FFI / scene
//! change — so the Rust-side B1 backend harness (Warmth & Light concurrent
//! non-trivial state) passes without the input handler being present. That
//! gap is the "circular" failure mode the Evaluator flagged: deleting the
//! handler from `world_renderer.gd` would NOT trip any existing test, yet
//! the feature (visual toggle) would silently regress.
//!
//! This file closes the gap with a source-token sweep over the renderer
//! script. Tokens are deliberately strict: each one corresponds to a
//! specific line/expression in the handler — removing the handler removes
//! the tokens, and removing any single guard (echo, KEY_SPACE, two-state
//! assignment, print) trips a distinct assertion.
//!
//! Run: `cargo test -p sim-test --test harness_t7_10_b1_space_toggle -- --nocapture`

// ── B1.S1: input handler signature present ───────────────────────────────────

/// Type S: `world_renderer.gd` declares an `_unhandled_input(event: InputEvent)`
/// handler. Without this entrypoint Godot never delivers key events, and the
/// SPACE toggle is dead code.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_t7_10_b1_unhandled_input_handler_present() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");

    assert!(
        src.contains("func _unhandled_input(event: InputEvent) -> void:"),
        "world_renderer.gd must declare \
         `func _unhandled_input(event: InputEvent) -> void:` \
         (Godot input callback for the T7.10.B1 SPACE toggle). \
         Removing this function silently kills the channel switch."
    );
}

// ── B1.S2: input event type and pressed/echo guards ──────────────────────────

/// Type S: handler discriminates on `InputEventKey`, requires
/// `event.pressed`, and rejects `event.echo`.
///
/// `event.echo` is the OS auto-repeat flag: without `not event.echo` a held
/// SPACE rapid-flips the channel every frame. The Evaluator listed each of
/// these tokens as a mandatory source-level guard.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_t7_10_b1_input_event_guards_present() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");

    assert!(
        src.contains("event is InputEventKey"),
        "world_renderer.gd must type-check `event is InputEventKey` \
         (only key events should drive the channel toggle)"
    );
    assert!(
        src.contains("event.pressed"),
        "world_renderer.gd must require `event.pressed` \
         (fire on key-down, not key-up)"
    );
    assert!(
        src.contains("not event.echo"),
        "world_renderer.gd must require `not event.echo` \
         (echo guard — holding SPACE must NOT rapid-flip the channel)"
    );
}

// ── B1.S3: SPACE keycode binding ─────────────────────────────────────────────

/// Type S: handler matches `event.keycode == KEY_SPACE`.
///
/// The keybind is hard-coded by design (Phase 2 disclosure §3 in the prompt).
/// Any future migration to InputMap actions must update this assertion
/// alongside the renderer.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_t7_10_b1_space_keycode_binding_present() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");

    assert!(
        src.contains("event.keycode == KEY_SPACE"),
        "world_renderer.gd must match `event.keycode == KEY_SPACE` \
         (T7.10.B1 keybind; documented as hard-coded in the dispatch prompt)"
    );
}

// ── B1.S4: two-state toggle assignment ───────────────────────────────────────

/// Type S: the toggle assigns `current_channel` to `CHANNEL_LIGHT` iff it was
/// `CHANNEL_WARMTH`, otherwise back to `CHANNEL_WARMTH`.
///
/// The exact ternary `CHANNEL_LIGHT if current_channel == CHANNEL_WARMTH else
/// CHANNEL_WARMTH` is what makes the toggle a strict two-state cycle. A
/// regression to `current_channel += 1` or `current_channel = CHANNEL_LIGHT`
/// (one-way) would still leave SPACE bound, but the visible behaviour would
/// silently break.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_t7_10_b1_two_state_toggle_assignment_present() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");

    assert!(
        src.contains(
            "current_channel = CHANNEL_LIGHT if current_channel == CHANNEL_WARMTH else CHANNEL_WARMTH"
        ),
        "world_renderer.gd must use the exact two-state toggle \
         `current_channel = CHANNEL_LIGHT if current_channel == CHANNEL_WARMTH else CHANNEL_WARMTH` \
         (T7.10.B1 toggle invariant — strict Warmth↔Light cycle)"
    );
}

// ── B1.S5: console feedback path ─────────────────────────────────────────────

/// Type S: handler emits the `Channel switched: <name>` print line.
///
/// The console line is the only user-observable signal in headless / F6
/// sessions that the toggle fired (no on-screen HUD label per Phase 2
/// disclosure §2). Removing the print would make the toggle's effect
/// indistinguishable from a dead key.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_t7_10_b1_channel_switched_print_present() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");

    assert!(
        src.contains("print(\"Channel switched: \""),
        "world_renderer.gd must call `print(\"Channel switched: \", channel_name)` \
         (T7.10.B1 console feedback — the only visible toggle signal in F6 sessions)"
    );
    assert!(
        src.contains("\"Light\" if current_channel == CHANNEL_LIGHT else \"Warmth\""),
        "world_renderer.gd must derive the printed name with \
         `\"Light\" if current_channel == CHANNEL_LIGHT else \"Warmth\"` \
         (both channel names must surface, not just one)"
    );
}

// ── B1.S6: initial channel preserves T7.10.A baseline ────────────────────────

/// Type S: `current_channel: int = CHANNEL_WARMTH` is the initial value, and
/// `CHANNEL_LIGHT := 1` exists as a constant.
///
/// Initial = Warmth means the first visible frame matches the T7.9.B/T7.10.A
/// baseline screenshot (a Warmth disc), so the existing visual regression
/// guard does not trip on T7.10.B1's pure renderer edit.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_t7_10_b1_initial_channel_is_warmth() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");

    assert!(
        src.contains("current_channel: int = CHANNEL_WARMTH"),
        "world_renderer.gd must initialise `current_channel: int = CHANNEL_WARMTH` \
         (first visible frame matches T7.10.A Warmth baseline so existing \
         visual regression guards do not trip)"
    );
    assert!(
        src.contains("CHANNEL_LIGHT := 1"),
        "world_renderer.gd must declare `CHANNEL_LIGHT := 1` \
         (toggle target; Light = InfluenceChannel index 1)"
    );
    assert!(
        src.contains("CHANNEL_WARMTH := 0"),
        "world_renderer.gd must declare `CHANNEL_WARMTH := 0` \
         (toggle baseline; Warmth = InfluenceChannel index 0)"
    );
}
