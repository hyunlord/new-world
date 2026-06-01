//! V7 Phase 12-γ — Settlement Centroid Furniture Placeholder harness.
//!
//! Static-inspection + smoke-test harness for Phase 12-γ:
//!   - `rust/crates/sim-bridge/src/ffi/world_node.rs` — adds
//!     `SettlementSnapshotRow`, `collect_settlement_snapshot`,
//!     `settlement_rows_to_dict`, and a `#[func] get_settlement_snapshot`
//!     method on `WorldSimNode`.
//!   - `rust/crates/sim-bridge/src/ffi/mod.rs` — re-exports the new symbols.
//!   - `scripts/ui/world_renderer.gd` (modified) — polls the new snapshot,
//!     maintains a keyed dictionary of furniture Sprite2D nodes, reaps
//!     dissolved settlements.
//!   - Regression guards for Phase 4-γ, Phase 11-α + D1, Phase 12-α,
//!     12-β.1, and 12-β.2 A3 invariants.
//!
//! Plan locked thresholds: see plan_final.md for this feature.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use hecs::World;
use sim_bridge::ffi::{collect_settlement_snapshot, SettlementSnapshotRow};
use sim_core::components::{Agent, Position, Settlement, SettlementId};

// ── helpers ───────────────────────────────────────────────────────────────

fn project_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .expect("project root above sim-test crate")
}

fn read_file(rel: &str) -> String {
    let path = project_root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

/// Strip Rust line comments (`// …` to EOL). Preserves `//` inside string
/// literals (single or double quoted).
fn strip_rs_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    for line in src.lines() {
        let mut in_str: Option<char> = None;
        let bytes = line.as_bytes();
        let mut keep_end = line.len();
        let mut i = 0;
        while i < bytes.len() {
            let c = bytes[i] as char;
            match in_str {
                Some(q) if c == q => in_str = None,
                None if c == '"' || c == '\'' => in_str = Some(c),
                None if c == '/' && i + 1 < bytes.len() && bytes[i + 1] as char == '/' => {
                    keep_end = i;
                    break;
                }
                _ => {}
            }
            i += 1;
        }
        out.push_str(&line[..keep_end]);
        out.push('\n');
    }
    out
}

/// Strip GDScript line comments (`# …` to EOL). Preserves `#` inside string
/// literals (single or double quoted).
fn strip_gd_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    for line in src.lines() {
        let mut in_str: Option<char> = None;
        let mut keep_end = line.len();
        let bytes = line.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            let c = b as char;
            match in_str {
                Some(q) if c == q => in_str = None,
                None if c == '"' || c == '\'' => in_str = Some(c),
                None if c == '#' => {
                    keep_end = i;
                    break;
                }
                _ => {}
            }
        }
        out.push_str(&line[..keep_end]);
        out.push('\n');
    }
    out
}

const WORLD_NODE_PATH: &str = "rust/crates/sim-bridge/src/ffi/world_node.rs";
const WORLD_RENDERER_PATH: &str = "scripts/ui/world_renderer.gd";
const AGENT_RENDERER_PATH: &str = "scripts/ui/agent_renderer.gd";
const MAIN_TSCN_PATH: &str = "scenes/main.tscn";

// ─── Assertion 1: settlement_snapshot_collector_exists ────────────────────
#[test]
fn harness_p12_gamma_a1_settlement_snapshot_collector_exists() {
    // Type: A — exactly 1 `pub fn collect_settlement_snapshot` symbol
    // accepting a hecs world reference in sim-bridge.
    let src = read_file(WORLD_NODE_PATH);
    let stripped = strip_rs_comments(&src);
    let count = stripped
        .matches("pub fn collect_settlement_snapshot")
        .count();
    assert_eq!(
        count, 1,
        "A1: must have exactly 1 `pub fn collect_settlement_snapshot` declaration; found {count}"
    );
    println!("[P12-γ A1] collect_settlement_snapshot symbol present ✓");
}

// ─── Assertion 2: snapshot_row_struct_has_five_required_fields ────────────
#[test]
fn harness_p12_gamma_a2_snapshot_row_struct_has_five_required_fields() {
    // Type: A — `pub struct SettlementSnapshotRow` declared with all 5
    // fields: entity_bits, settlement_id, centroid_x, centroid_y, member_count.
    let src = read_file(WORLD_NODE_PATH);
    let stripped = strip_rs_comments(&src);

    let struct_pos = stripped
        .find("pub struct SettlementSnapshotRow")
        .expect("A2.1: `pub struct SettlementSnapshotRow` must be declared");
    let end = stripped[struct_pos..]
        .find('}')
        .map(|i| struct_pos + i + 1)
        .unwrap_or(stripped.len());
    let body = &stripped[struct_pos..end];
    for field in [
        "entity_bits",
        "settlement_id",
        "centroid_x",
        "centroid_y",
        "member_count",
    ] {
        assert!(
            body.contains(field),
            "A2.2: SettlementSnapshotRow must declare field `{field}`; body=\n{body}"
        );
    }
    println!("[P12-γ A2] all 5 SettlementSnapshotRow fields present ✓");
}

// ─── Assertion 3: collector_iterates_settlement_store_and_agent_position ──
#[test]
fn harness_p12_gamma_a3_collector_queries_settlement_and_agent_position() {
    // Type: A — RE-POINTED (fix-settlements-zero-hud). The collector no longer
    // queries the always-empty world `Settlement` table; it iterates the
    // authoritative `resources.settlements` store. A3.1 now asserts that
    // `settlements.values()` iteration; A3.2's `(Agent, Position)` join is
    // unchanged.
    let src = read_file(WORLD_NODE_PATH);
    let stripped = strip_rs_comments(&src);

    // Settlement store iteration — replaces the removed world query.
    let settlement_iter_present = stripped.contains("settlements.values()");
    assert!(
        settlement_iter_present,
        "A3.1: collector must iterate the settlement store \
         (`settlements.values()`) — the always-empty world `query::<&Settlement>` \
         was the bug and is removed"
    );

    // (Agent, Position) join query — whitespace-tolerant.
    let agent_pos_acceptable = [
        "query::<(&Agent, &Position)>",
        "query::<(&Agent,&Position)>",
    ];
    let agent_pos_present = agent_pos_acceptable.iter().any(|p| stripped.contains(p));
    assert!(
        agent_pos_present,
        "A3.2: collector must contain `query::<(&Agent, &Position)>` join"
    );
    println!("[P12-γ A3] collector contains Settlement and (Agent, Position) queries ✓");
}

// ─── Assertion 4: get_settlement_snapshot_func_attribute_and_signature ────
#[test]
fn harness_p12_gamma_a4_get_settlement_snapshot_func_attribute_and_signature() {
    // Type: A — `#[func]` immediately preceding
    // `fn get_settlement_snapshot(&self) -> VarDictionary`, AND the body
    // forwards to `collect_settlement_snapshot` against `self.engine.world`.
    let src = read_file(WORLD_NODE_PATH);
    let stripped = strip_rs_comments(&src);

    let sig = "fn get_settlement_snapshot(&self) -> VarDictionary";
    let sig_pos = stripped
        .find(sig)
        .unwrap_or_else(|| panic!("A4.1: `{sig}` must be declared"));

    let prefix = &stripped[..sig_pos];
    let prev_lines: Vec<&str> = prefix.lines().collect();
    let mut last_nonempty: Option<&str> = None;
    for line in prev_lines.iter().rev() {
        if !line.trim().is_empty() {
            last_nonempty = Some(line.trim());
            break;
        }
    }
    let preceding = last_nonempty.unwrap_or("");
    assert_eq!(
        preceding, "#[func]",
        "A4.2: line immediately preceding `{sig}` must be `#[func]`; got `{preceding}`"
    );

    // Body forwards to collect_settlement_snapshot with engine world.
    let after_sig = &stripped[sig_pos..];
    let open_brace = after_sig
        .find('{')
        .unwrap_or_else(|| panic!("A4.3: function body opening `{{` not found after signature"));
    let mut depth = 0i32;
    let mut close_idx = open_brace;
    for (i, c) in after_sig[open_brace..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    close_idx = open_brace + i;
                    break;
                }
            }
            _ => {}
        }
    }
    let body = &after_sig[open_brace..=close_idx];
    assert!(
        body.contains("collect_settlement_snapshot"),
        "A4.4: function body must call `collect_settlement_snapshot`; body=\n{body}"
    );
    assert!(
        body.contains("self.engine.world"),
        "A4.5: function body must forward `self.engine.world` to the collector; body=\n{body}"
    );
    println!("[P12-γ A4] #[func] get_settlement_snapshot forwards to collector ✓");
}

// ─── Assertion 5: dict_marshaller_sets_five_named_keys ────────────────────
#[test]
fn harness_p12_gamma_a5_dict_marshaller_sets_five_named_keys() {
    // Type: A — `dict.set` calls for all five string keys: "ids",
    // "settlement_ids", "centroid_xs", "centroid_ys", "member_counts".
    let src = read_file(WORLD_NODE_PATH);
    let stripped = strip_rs_comments(&src);

    let needles = [
        "dict.set(\"ids\"",
        "dict.set(\"settlement_ids\"",
        "dict.set(\"centroid_xs\"",
        "dict.set(\"centroid_ys\"",
        "dict.set(\"member_counts\"",
    ];
    for needle in needles.iter() {
        assert!(
            stripped.contains(needle),
            "A5: must contain `{needle}` somewhere in sim-bridge source"
        );
    }
    println!("[P12-γ A5] all 5 dict.set keys present ✓");
}

// ─── Assertion 6: collector_returns_correct_centroid_for_known_membership ─
#[test]
fn harness_p12_gamma_a6_collector_returns_correct_centroid_for_known_membership() {
    // Type: A — spawn 3 agents at known positions + a Settlement holding
    // those 3 agent ids; assert centroid is floor mean and member_count==3.
    let mut world = World::new();

    // Three distinct positions: (10, 20), (12, 22), (14, 24).
    // sum_x = 36 → /3 = 12; sum_y = 66 → /3 = 22.
    let positions = [(10u32, 20u32), (12u32, 22u32), (14u32, 24u32)];
    let agent_ids = [101u64, 102u64, 103u64];

    for ((ax, ay), &aid) in positions.iter().zip(agent_ids.iter()) {
        world.spawn((Agent { id: aid }, Position { x: *ax, y: *ay }));
    }

    let mut settlement = Settlement::new_with_id(7u32, 0);
    for &aid in agent_ids.iter() {
        settlement.add_member_agent(aid);
    }
    // Re-pointed: settlements live in `resources.settlements`
    // (HashMap<SettlementId, Settlement>), NOT as ECS world entities.
    let mut settlements: HashMap<SettlementId, Settlement> = HashMap::new();
    settlements.insert(settlement.settlement_id, settlement);

    let rows: Vec<SettlementSnapshotRow> = collect_settlement_snapshot(&world, &settlements);

    assert_eq!(rows.len(), 1, "A6.1: expected 1 settlement row; got {}", rows.len());
    let row = rows[0];

    let expected_x: i32 = (10 + 12 + 14) / 3;
    let expected_y: i32 = (20 + 22 + 24) / 3;
    assert_eq!(
        row.centroid_x, expected_x,
        "A6.2: centroid_x must equal floor((10+12+14)/3)={expected_x}; got {}",
        row.centroid_x
    );
    assert_eq!(
        row.centroid_y, expected_y,
        "A6.3: centroid_y must equal floor((20+22+24)/3)={expected_y}; got {}",
        row.centroid_y
    );
    assert_eq!(
        row.member_count, 3,
        "A6.4: member_count must equal 3; got {}",
        row.member_count
    );
    assert_eq!(
        row.settlement_id, 7,
        "A6.5: settlement_id must equal 7; got {}",
        row.settlement_id
    );
    assert_eq!(
        row.entity_bits,
        7u64,
        "A6.6: entity_bits must equal settlement_id (7) — no ECS entity exists for a \
         HashMap-sourced settlement; the field remains a stable unique downstream key"
    );
    println!("[P12-γ A6] centroid + member_count + ids correct ✓");
}

// ─── Assertion 7: zero_member_settlement_produces_no_row ──────────────────
#[test]
fn harness_p12_gamma_a7_zero_member_settlement_produces_no_row() {
    // Type: A — Settlement with empty member_agents must be skipped.
    // Re-pointed: settlement lives in the HashMap store; world is empty.
    let world = World::new();
    let empty_settlement = Settlement::new_with_id(99, 0);
    let mut settlements: HashMap<SettlementId, Settlement> = HashMap::new();
    settlements.insert(empty_settlement.settlement_id, empty_settlement);

    let rows = collect_settlement_snapshot(&world, &settlements);
    assert_eq!(
        rows.len(),
        0,
        "A7: empty-member Settlement must produce 0 rows; got {}",
        rows.len()
    );
    println!("[P12-γ A7] empty-member Settlement skipped ✓");
}

// ─── Assertion 8: settlement_with_missing_member_position_is_handled ──────
#[test]
fn harness_p12_gamma_a8_settlement_with_missing_member_position_is_handled() {
    // Type: A — Settlement references one resolvable agent + one stale id.
    // member_count must reflect only the resolvable member; centroid must
    // be only that member's position.
    let mut world = World::new();
    world.spawn((Agent { id: 501 }, Position { x: 5, y: 9 }));

    let mut settlement = Settlement::new_with_id(11, 0);
    settlement.add_member_agent(501); // resolvable
    settlement.add_member_agent(9999); // stale — no Agent with this id
    // Re-pointed: settlement lives in the HashMap store; agents stay in world.
    let mut settlements: HashMap<SettlementId, Settlement> = HashMap::new();
    settlements.insert(settlement.settlement_id, settlement);

    let rows = collect_settlement_snapshot(&world, &settlements);
    assert_eq!(rows.len(), 1, "A8.1: expected 1 row; got {}", rows.len());
    let row = rows[0];
    assert_eq!(
        row.member_count, 1,
        "A8.2: member_count must equal resolvable count (1); got {}",
        row.member_count
    );
    assert_eq!(
        row.centroid_x, 5,
        "A8.3: centroid_x must equal only resolvable member's x=5; got {}",
        row.centroid_x
    );
    assert_eq!(
        row.centroid_y, 9,
        "A8.4: centroid_y must equal only resolvable member's y=9; got {}",
        row.centroid_y
    );
    println!("[P12-γ A8] stale member references handled without contamination ✓");
}

// ─── Assertion 9: world_renderer_declares_furniture_sprite_path_and_z ─────
#[test]
fn harness_p12_gamma_a9_world_renderer_declares_furniture_sprite_path_and_z_constants() {
    // Type: A — FURNITURE_SPRITE_PATH points under
    // assets/sprites/furniture/hearth/; Z_FURNITURE ∈ {1, 2, 3, 4}.
    let src = read_file(WORLD_RENDERER_PATH);
    let stripped = strip_gd_comments(&src);

    let sprite_line = stripped
        .lines()
        .find(|l| {
            l.trim_start().starts_with("const ") && l.contains("FURNITURE_SPRITE_PATH")
        })
        .unwrap_or_else(|| {
            panic!("A9.1: world_renderer.gd must declare `const FURNITURE_SPRITE_PATH`")
        });
    assert!(
        sprite_line.contains("assets/sprites/furniture/hearth/"),
        "A9.2: FURNITURE_SPRITE_PATH must reference `assets/sprites/furniture/hearth/`; line=`{sprite_line}`"
    );

    let z_line = stripped
        .lines()
        .find(|l| {
            let t = l.trim_start();
            t.starts_with("const ") && t.contains("Z_FURNITURE")
        })
        .unwrap_or_else(|| panic!("A9.3: const Z_FURNITURE must be declared"));
    let after_eq = z_line.rfind('=').map(|i| &z_line[i + 1..]).unwrap_or("");
    let z_parsed: i64 = after_eq
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("A9.4: Z_FURNITURE must parse as int; line=`{z_line}`"));
    assert!(
        (1..=4).contains(&z_parsed),
        "A9.5: Z_FURNITURE must be in (0, 5) exclusive — i.e. ∈ {{1,2,3,4}} — to sit between terrain (z=0) and ConstructionSite (z=5); got {z_parsed}"
    );
    println!("[P12-γ A9] FURNITURE_SPRITE_PATH + Z_FURNITURE constants valid ✓");
}

// ─── Assertion 10: world_renderer_polls_ffi_and_reaps_stale_sprites ───────
#[test]
fn harness_p12_gamma_a10_world_renderer_polls_ffi_and_reaps_stale_sprites() {
    // Type: A — world_renderer.gd calls world_sim.get_settlement_snapshot()
    // AND iterates _furniture_sprites.keys() with a queue_free() reaper.
    let src = read_file(WORLD_RENDERER_PATH);
    let stripped = strip_gd_comments(&src);

    assert!(
        stripped.contains("world_sim.get_settlement_snapshot()"),
        "A10.1: world_renderer.gd must call `world_sim.get_settlement_snapshot()`"
    );
    assert!(
        stripped.contains("_furniture_sprites.keys()"),
        "A10.2: world_renderer.gd must iterate `_furniture_sprites.keys()` for reaping"
    );
    assert!(
        stripped.contains("queue_free()"),
        "A10.3: world_renderer.gd must contain `queue_free()` (reaper loop)"
    );
    println!("[P12-γ A10] FFI poll + reaper loop present ✓");
}

// ─── Assertion 11: phase4_gamma_and_d1_invariants_preserved ───────────────
#[test]
fn harness_p12_gamma_a11_phase4_gamma_and_d1_invariants_preserved() {
    // Type: D — regression guard. SPRITE_SCALE = 0.25 + 4 D1 STATE_TINTS
    // Color literals must persist.
    let src = read_file(AGENT_RENDERER_PATH);
    let stripped = strip_gd_comments(&src);

    let scale_forms = [
        "SPRITE_SCALE := 0.25",
        "SPRITE_SCALE: float = 0.25",
        "SPRITE_SCALE :float = 0.25",
        "SPRITE_SCALE: float= 0.25",
        "SPRITE_SCALE = 0.25",
    ];
    let scale_ok = scale_forms.iter().any(|c| stripped.contains(c));
    assert!(
        scale_ok,
        "A11.1: agent_renderer.gd must preserve SPRITE_SCALE = 0.25 (Phase 4-γ)"
    );

    let required_tints = [
        "Color(0.55, 0.70, 0.95, 1.0)",
        "Color(1.0, 0.85, 0.15, 1.0)",
        "Color(1.0, 0.40, 0.75, 1.0)",
        "Color(0.30, 0.95, 0.35, 1.0)",
    ];
    for needle in required_tints.iter() {
        assert!(
            stripped.contains(needle),
            "A11.2: agent_renderer.gd MUST preserve D1 STATE_TINTS literal `{needle}`"
        );
    }
    println!("[P12-γ A11] Phase 4-γ SPRITE_SCALE + D1 STATE_TINTS preserved ✓");
}

// ─── Assertion 12: phase12_alpha_beta1_beta2_invariants_preserved ─────────
#[test]
fn harness_p12_gamma_a12_phase12_alpha_beta1_beta2_invariants_preserved() {
    // Type: D — regression guard for the chain α + β.1 + β.2 A3.
    let tscn = read_file(MAIN_TSCN_PATH);
    let zoom_ok = tscn.contains("zoom = Vector2(2, 2)")
        || tscn.contains("zoom = Vector2(2.0, 2.0)");
    assert!(
        zoom_ok,
        "A12.1: main.tscn must preserve Camera2D `zoom = Vector2(2, 2)` (Phase 12-α)"
    );
    let camera_script_ok =
        tscn.contains("camera_controller.gd") && tscn.contains("script = ExtResource(");
    assert!(
        camera_script_ok,
        "A12.2: main.tscn must preserve camera_controller.gd ExtResource attachment (Phase 12-α)"
    );

    let renderer = read_file(WORLD_RENDERER_PATH);
    let stripped = strip_gd_comments(&renderer);
    assert!(
        stripped.contains("TERRAIN_TILESET_PATH"),
        "A12.3: world_renderer.gd must still declare TERRAIN_TILESET_PATH (Phase 12-β.1)"
    );
    assert!(
        stripped.contains("BUILDING_SPRITE_PATH"),
        "A12.4: world_renderer.gd must still declare BUILDING_SPRITE_PATH (Phase 12-β.1)"
    );

    let alpha_line = stripped
        .lines()
        .find(|l| {
            let t = l.trim_start();
            t.starts_with("const ") && t.contains("OVERLAY_ALPHA")
        })
        .unwrap_or_else(|| panic!("A12.5: const OVERLAY_ALPHA must remain declared"));
    let after_eq = alpha_line.rfind('=').map(|i| &alpha_line[i + 1..]).unwrap_or("");
    let val: f64 = after_eq
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("A12.6: OVERLAY_ALPHA must parse as float; line=`{alpha_line}`"));
    assert!(
        (val - 0.65).abs() < 1e-9,
        "A12.7: OVERLAY_ALPHA must equal exactly 0.65; got {val}"
    );

    // β.2 A3: CONSTRUCTION_SPRITE_PATH + Z_CONSTRUCTION = 5.
    assert!(
        stripped.contains("CONSTRUCTION_SPRITE_PATH"),
        "A12.8: world_renderer.gd must still declare CONSTRUCTION_SPRITE_PATH (Phase 12-β.2 A3)"
    );
    let z_line = stripped
        .lines()
        .find(|l| {
            let t = l.trim_start();
            t.starts_with("const ") && t.contains("Z_CONSTRUCTION")
        })
        .unwrap_or_else(|| panic!("A12.9: const Z_CONSTRUCTION must remain declared"));
    let z_after_eq = z_line.rfind('=').map(|i| &z_line[i + 1..]).unwrap_or("");
    let z_parsed: i64 = z_after_eq
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("A12.10: Z_CONSTRUCTION must parse as int; line=`{z_line}`"));
    assert_eq!(
        z_parsed, 5,
        "A12.11: Z_CONSTRUCTION must remain exactly 5 (Phase 12-β.2 A3); got {z_parsed}"
    );
    println!("[P12-γ A12] Phase 12-α + β.1 + β.2 A3 invariants preserved ✓");
}
