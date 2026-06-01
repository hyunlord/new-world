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

// ── B1.S4: three-state cycle assignment (T7.10.C extension) ──────────────────

/// Type S: SPACE cycles Warmth → Light → Noise → Warmth via an explicit
/// if/elif/else chain over the three CHANNEL_* constants.
///
/// T7.10.B1 originally used a two-state ternary
/// (`CHANNEL_LIGHT if … else CHANNEL_WARMTH`). T7.10.C added Noise to the
/// stamped + propagated channels, and N4-a extended the SPACE toggle to a
/// 3-state cycle so all three backend wirings (Warmth BFS, Light shadowcast,
/// Noise linear-decay) are reachable from a single F6 session. The cycle is
/// strict: each press advances exactly one step, no skips.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_t7_10_b1_three_state_cycle_assignment_present() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");

    // Warmth → Light transition (first branch).
    assert!(
        src.contains("if current_channel == CHANNEL_WARMTH:")
            && src.contains("current_channel = CHANNEL_LIGHT"),
        "world_renderer.gd must contain the Warmth→Light branch \
         (`if current_channel == CHANNEL_WARMTH:` followed by \
         `current_channel = CHANNEL_LIGHT`)"
    );
    // Light → Noise transition (elif branch).
    assert!(
        src.contains("elif current_channel == CHANNEL_LIGHT:")
            && src.contains("current_channel = CHANNEL_NOISE"),
        "world_renderer.gd must contain the Light→Noise branch \
         (`elif current_channel == CHANNEL_LIGHT:` followed by \
         `current_channel = CHANNEL_NOISE`) — T7.10.C cycle extension"
    );
    // Noise → Warmth wrap (else branch).
    assert!(
        src.contains("else:") && src.contains("current_channel = CHANNEL_WARMTH"),
        "world_renderer.gd must contain the Noise→Warmth wrap \
         (`else:` followed by `current_channel = CHANNEL_WARMTH`) — \
         closes the 3-state cycle"
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
    // T7.10.C extends the cycle to 3 channels; each branch assigns
    // channel_name directly to one of the three string literals.
    assert!(
        src.contains("channel_name = \"Warmth\""),
        "world_renderer.gd must assign `channel_name = \"Warmth\"` in the wrap branch \
         (T7.10.C 3-state cycle — Warmth name must surface)"
    );
    assert!(
        src.contains("channel_name = \"Light\""),
        "world_renderer.gd must assign `channel_name = \"Light\"` in the Warmth→Light branch \
         (T7.10.C 3-state cycle — Light name must surface)"
    );
    assert!(
        src.contains("channel_name = \"Noise\""),
        "world_renderer.gd must assign `channel_name = \"Noise\"` in the Light→Noise branch \
         (T7.10.C 3-state cycle — Noise name must surface)"
    );
}

// ── B1.S6: initial channel is OFF (clean launch screen) ──────────────────────

/// Type S: `current_channel: int = CHANNEL_OFF` is the initial value, and
/// `CHANNEL_OFF := -1` exists as a sentinel constant.
///
/// fix-overlay-darkness re-point: the influence overlay used to default to
/// Warmth (`current_channel: int = CHANNEL_WARMTH`), which washed the launch
/// screen with a dim 0.65-alpha disc and dimmed the terrain beneath. The fix
/// defaults to OFF so the first visible frame is clean terrain + agents +
/// resource markers; the overlay becomes an opt-in analysis tool reached via
/// SPACE. The negative assertion (old Warmth default absent) prevents leaving
/// both initialiser lines present so two `contains` spuriously pass. The
/// `CHANNEL_LIGHT := 1` / `CHANNEL_WARMTH := 0` const-existence checks are
/// kept (regression guard — the OFF sentinel is added, the real channel
/// indices must survive).
///
/// ticks: 0 (source-only check)
#[test]
fn harness_t7_10_b1_initial_channel_is_off() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");

    assert!(
        src.contains("current_channel: int = CHANNEL_OFF"),
        "world_renderer.gd must initialise `current_channel: int = CHANNEL_OFF` \
         (clean launch screen — the influence overlay is opt-in, not always-on; \
         this is the fix for the dark-screen bug)"
    );
    assert!(
        !src.contains("current_channel: int = CHANNEL_WARMTH"),
        "world_renderer.gd must NOT keep `current_channel: int = CHANNEL_WARMTH` \
         (the old always-on default this change replaces — leaving it present \
         means the dark-screen bug is unfixed)"
    );
    assert!(
        src.contains("CHANNEL_OFF := -1"),
        "world_renderer.gd must declare `CHANNEL_OFF := -1` \
         (OFF sentinel — distinct from every real InfluenceChannel index; \
         -1 must never collide with a channel passed to get_influence_overlay)"
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

// ── B1.S7: overlay sprite starts hidden in _ready ────────────────────────────

/// Extract the body of a GDScript function from the renderer source, from the
/// signature line up to (but not including) the next top-level `func ` decl.
/// Used by the OFF-gate assertions below to scope token / ordering checks to a
/// single function instead of the whole file (a token could otherwise satisfy a
/// `_process` check by appearing in `_unhandled_input` and vice-versa).
fn extract_fn_body<'a>(src: &'a str, signature: &str) -> &'a str {
    let start = src
        .find(signature)
        .unwrap_or_else(|| panic!("signature not found in world_renderer.gd: {signature}"));
    let after = &src[start + signature.len()..];
    match after.find("\nfunc ") {
        Some(end) => &after[..end],
        None => after,
    }
}

/// Type A (Assertion 4): the influence-overlay `sprite` is hidden inside
/// `_ready` after it is created/configured.
///
/// Default OFF means the first visible frame must show NO overlay. Godot's
/// `Sprite2D` is visible by default, so without an explicit `sprite.visible =
/// false` at creation the dark 0.65-alpha Warmth wash flashes on frame 0 before
/// `_process` runs — i.e. the dark-screen bug persists for the launch frame.
/// Scoping the check to the `_ready` body proves the hide happens at creation,
/// not (only) inside `_process`.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_t7_10_b1_ready_hides_overlay_sprite() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    let ready = extract_fn_body(src, "func _ready() -> void:");

    assert!(
        ready.contains("sprite.visible = false"),
        "world_renderer.gd `_ready` must set `sprite.visible = false` after the \
         overlay sprite is created (default channel is OFF — the launch frame \
         must render no overlay; without this the Warmth wash flashes on frame 0)"
    );
}

// ── B1.S8: SPACE cycle includes the OFF state (entry + wrap) ──────────────────

/// Type A (Assertion 5): the SPACE cycle in `_unhandled_input` makes OFF both
/// *enterable* (Beauty→OFF wrap) and *exitable* (OFF→Warmth entry).
///
/// This is the assertion that defeats the circular-pass failure mode: a
/// Generator could satisfy the default-flip assertions and never make OFF
/// reachable from the keyboard, leaving the user permanently unable to re-hide
/// the overlay after one SPACE press (a one-way OFF trap). Both halves must be
/// present so the net cycle is
/// OFF→Warmth→Light→Noise→Danger→Spiritual→Beauty→OFF.
///
/// ticks: 0 (source-only check)
#[test]
fn harness_t7_10_b1_cycle_includes_off_entry_and_wrap() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    let handler = extract_fn_body(src, "func _unhandled_input(event: InputEvent) -> void:");

    // (a) OFF → Warmth entry branch: enterable means there is a branch keyed on
    //     `current_channel == CHANNEL_OFF` that advances to Warmth.
    assert!(
        handler.contains("current_channel == CHANNEL_OFF")
            && handler.contains("current_channel = CHANNEL_WARMTH")
            && handler.contains("channel_name = \"Warmth\""),
        "world_renderer.gd `_unhandled_input` must contain the OFF→Warmth entry \
         branch (`current_channel == CHANNEL_OFF` → `current_channel = \
         CHANNEL_WARMTH` with `channel_name = \"Warmth\"`) so the overlay can be \
         turned ON from the clean default screen"
    );
    // (b) Beauty → OFF wrap branch: enterable into OFF means the final wrap sets
    //     the channel back to CHANNEL_OFF with the "Off" name.
    assert!(
        handler.contains("current_channel = CHANNEL_OFF")
            && handler.contains("channel_name = \"Off\""),
        "world_renderer.gd `_unhandled_input` must contain the Beauty→OFF wrap \
         branch (`current_channel = CHANNEL_OFF` with `channel_name = \"Off\"`) \
         so the cycle can return to the clean screen — without it OFF is \
         unreachable from the keyboard and the toggle is one-way"
    );
}

// ── B1.S9: _process gates the overlay draw on the channel ─────────────────────

/// Type A (Assertion 6): `_process` gates the overlay upload on the channel —
/// the OFF path hides the sprite and SKIPS the FFI call, the non-OFF path shows
/// the sprite and draws.
///
/// Without this per-frame gate the overlay re-appears every frame regardless of
/// `current_channel`, so the default flip would be cosmetically reverted by
/// `_process` on tick 1. It also enforces the FFI-safety rule:
/// `get_influence_overlay(CHANNEL_OFF = -1)` must never be called (−1 is a
/// sentinel, not a valid channel index). Verified structurally: the OFF
/// discriminator and the `sprite.visible = false` both precede the
/// `get_influence_overlay` FFI call, proving the OFF branch short-circuits
/// before the FFI; and `sprite.visible = true` is also present (non-OFF show).
///
/// ticks: 0 (source-only check)
#[test]
fn harness_t7_10_b1_process_gates_overlay_on_channel() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    let process = extract_fn_body(src, "func _process(_delta: float) -> void:");

    // (a) OFF discriminator exists inside _process.
    let off_idx = process.find("current_channel == CHANNEL_OFF").expect(
        "world_renderer.gd `_process` must discriminate on `current_channel == \
         CHANNEL_OFF` to gate the overlay draw",
    );
    // (b) sprite.visible set in BOTH directions.
    let hide_idx = process.find("sprite.visible = false").expect(
        "world_renderer.gd `_process` OFF path must set `sprite.visible = false`",
    );
    assert!(
        process.contains("sprite.visible = true"),
        "world_renderer.gd `_process` non-OFF path must set `sprite.visible = true`"
    );
    // (c) The OFF branch precedes / skips the FFI call: both the OFF
    //     discriminator and the hide must appear before `get_influence_overlay`.
    let ffi_idx = process.find("get_influence_overlay(current_channel)").expect(
        "world_renderer.gd `_process` non-OFF path must call \
         `get_influence_overlay(current_channel)`",
    );
    assert!(
        off_idx < ffi_idx && hide_idx < ffi_idx,
        "world_renderer.gd `_process` must gate the FFI: the \
         `current_channel == CHANNEL_OFF` check and `sprite.visible = false` \
         must precede `get_influence_overlay` so the OFF branch short-circuits \
         before the FFI call (get_influence_overlay(-1) must never fire)"
    );
}

// ── B1.S10: substrate render calls remain unconditional after the OFF gate ────

/// Type D (Assertion 7): the three substrate render calls
/// (`_update_construction_sites`, `_update_settlement_furniture`,
/// `_render_resource_sources`) stay reachable in `_process` for every channel
/// INCLUDING OFF — they are not collateral-damaged by the OFF skip or the
/// overlay's data-size early-return.
///
/// The fix turns OFF the *overlay only*. If the Generator wraps these calls
/// inside the OFF early-return (the natural mistake when adding the gate),
/// construction sites / settlement furniture / resource markers stop rendering
/// on launch — a worse regression than the original bug. Verified structurally:
/// all three calls are present, they appear AFTER the OFF discriminator (so they
/// are outside the OFF-vs-non-OFF gate block), and there is no `return` between
/// the OFF gate and those calls (the OFF path falls through to them).
///
/// ticks: 0 (source-only check)
#[test]
fn harness_t7_10_b1_substrate_calls_survive_off_gate() {
    let src = include_str!("../../../../scripts/ui/world_renderer.gd");
    let process = extract_fn_body(src, "func _process(_delta: float) -> void:");

    let off_idx = process
        .find("current_channel == CHANNEL_OFF")
        .expect("`_process` must contain the OFF discriminator");
    let construction_idx = process.find("_update_construction_sites()").expect(
        "world_renderer.gd `_process` must call `_update_construction_sites()` \
         every frame",
    );
    assert!(
        process.contains("_update_settlement_furniture()"),
        "world_renderer.gd `_process` must call `_update_settlement_furniture()` \
         every frame (substrate render, not the overlay)"
    );
    assert!(
        process.contains("_render_resource_sources()"),
        "world_renderer.gd `_process` must call `_render_resource_sources()` \
         every frame (substrate render, not the overlay)"
    );
    // The substrate calls must be OUTSIDE the channel gate: they appear after
    // the OFF discriminator and after the non-OFF FFI draw block.
    assert!(
        construction_idx > off_idx,
        "the substrate render calls must come AFTER the overlay channel gate \
         (so they are not nested inside the OFF skip)"
    );
    // The OFF path must fall through to the substrate calls — no `return`
    // between the OFF gate and the first substrate call.
    let gate_region = &process[off_idx..construction_idx];
    assert!(
        !gate_region.contains("return"),
        "the OFF branch must NOT `return` before the substrate render calls \
         (otherwise construction/settlement/resource markers vanish on launch — \
         a worse regression than the original dark-screen bug)"
    );
}
