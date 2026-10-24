#!/usr/bin/env python3
"""V8-0 inventory: verdict rows and the closure check (goal 6.2, 11.1 step 15).

`rows(root)` yields one row per inventory unit, with the first matching rule
(`RULES`, ordered; a pattern is an exact path, a prefix ending in `/`, or a
`re:` regular expression):

- every top-level path;
- every file under `scripts/`, `scenes/`, `tools/`;
- every module `rust/crates/*/src/**.rs` and every `rust/crates/sim-test/tests/*.rs`
  (test verdicts come from the test triage table, `TEST_VERDICTS`);
- every file under `shaders/`;
- every directory under `assets/` and `localization/`, and every directory
  under `docs/` down to depth 3 (`docs/a/b`); deeper `docs/` directories are
  covered by their depth-3 parent row;
- every file whose own verdict differs from the verdict of its directory row
  (an exception row, e.g. one kept file in a deleted directory).

Rows are built from the union of `git ls-tree v7-final-art` and `git ls-files`,
so a deleted path keeps its row and a new path gets one.

usage: inventory.py table [ROOT]           print the markdown verdict table
       inventory.py write INVENTORY.md [ROOT]   replace the generated table section
       inventory.py check INVENTORY.md [ROOT]
           closure: every `git ls-files` path is covered (by prefix) by a row
           of the table in INVENTORY.md; exits 1 and lists uncovered paths.
"""
import re
import subprocess
import sys
from pathlib import Path

E_REACH = "tools/v8/gd_reach.py (keep-set closure, Part 1)"
E_GIT = "git ls-files at b1ca032f"

# (pattern, verdict, reason, evidence). First match wins.
RULES = [
    # ---- M1 additions (docs/v8/m1/M1_GOAL.md) -------------------------------------------
    ("sim_bridge.gdextension.uid", "KEEP", "the extension's committed Godot UID (M1 5.3)", "M1 DECISIONS D3"),
    ("scripts/world/colony/", "KEEP", "M1 colony view: terrain, trees, buildings, order overlay, people, order tool", "M1 goal 9.4; DECISIONS D25"),
    ("scripts/ui/hud/order_bar.gd", "KEEP", "M1 order bar", "M1 goal 9.6"),
    ("scripts/ui/hud/order_bar.gd.uid", "KEEP", "M1 order bar UID", "M1 5.3"),
    ("assets/v8/", "KEEP", "M1 colony tile and sprite sheet drawn by tools/v8/gen_tiles.py (M1-ART)", "M1 goal 9.2; M1-ART goal 5"),
    ("re:^tools/v8/(verify_v8_0\\.gd|v7_ui_deleted\\.txt|compare_chronicles\\.py|test_compare_chronicles\\.py|gen_tiles\\.py|test_gen_tiles\\.py|m1_gui_test\\.gd|test_locale_trf\\.gd|u_script\\.json)$", "KEEP", "M1 tooling (screen check, chronicle comparison, tile sheet, GUI test, trf test, U-script)", "M1 goal 5.1, 9.2, 9.7, 9.8, 10.1; DECISIONS D21"),
    ("re:^rust/crates/sim-core/src/(colony/.*|rules/colony)\\.rs$", "KEEP", "M1 colony data: items, orders, commands, jobs, the colony rule", "M1 goal 6.1; DECISIONS D8"),
    ("re:^rust/crates/sim-systems/src/(commands|worldgen/colony|runtime/jobs/.*)\\.rs$", "KEEP", "M1 commands, colony worldgen and the job layer", "M1 goal 6.3, 7, 8.1; ADR 0012, 0013"),
    ("rust/crates/sim-bridge/src/ffi/colony.rs", "KEEP", "M1 bridge orders and colony reads", "M1 goal 6.5"),
    ("rust/crates/sim-test/src/probe/colony.rs", "KEEP", "M1 colony probe (commands, U-bars, replay)", "M1 goal 6.4, 10"),
    ("re:^rust/crates/sim-test/tests/(colony_.*|engine_colony_ffi)\\.rs$", "KEEP", "M1 tests (Parts 1-3)", "M1 goal 6.6, 7.8, 8.5"),
    ("rust/crates/sim-test/tests/colony_support/", "KEEP", "M1 test fixtures", "M1 goal 7.8"),
    # ---- top-level files and dot-dirs -------------------------------------------------
    (".claude/settings.json", "KEEP", "local file neutralized ({}); committed blob replaced by it (7.6)", "goal 0.1, 7.6"),
    (".claude/agents/", "DELETE", "V7 harness pipeline agents (drafter/challenger/evaluator…); pipeline retired", "goal 6.3"),
    (".claude/skills/worldsim-code/", "DELETE", "absorbed into worldsim-architecture (10.2)", "goal 10.2"),
    (".claude/skills/worldsim-harness/", "DELETE", "absorbed into worldsim-testing minus pipeline/hook parts (10.2)", "goal 10.2"),
    (".claude/", "KEEP", "agent configuration; canonical skills location (10.1)", "goal 10.1"),
    (".agents/skills/systematic-debugging/", "MOVE→.claude/skills/systematic-debugging/", "generic skill; canonical copy moves, .agents keeps a symlink (10.1)", "goal 6.3, 10.1"),
    (".agents/skills/verification-before-completion/", "MOVE→.claude/skills/verification-before-completion/", "generic skill; moved with a symlink back (10.1)", "goal 6.3, 10.1"),
    (".agents/skills/godot/", "MOVE→.claude/skills/worldsim-godot/", "absorbed into worldsim-godot; generic references kept under references/ (10.2)", "goal 10.2"),
    (".agents/skills/worldsim-code/", "DELETE", "absorbed into worldsim-architecture (10.2)", "goal 10.2"),
    (".agents/", "KEEP", "Codex skill links (symlinks to .claude/skills, 10.1)", "goal 10.1"),
    (".claudeignore", "KEEP", "agent search hygiene file", E_GIT),
    (".editorconfig", "KEEP", "planner KEEP", "goal 6.3"),
    (".gitattributes", "KEEP", "planner KEEP", "goal 6.3"),
    (".gitignore", "KEEP", "not changed by this pass (proposals in the report)", "goal section 2"),
    (".harness/", "DELETE", "V7 harness pipeline state (plans, prompts, audit, baselines); pipeline retired", "goal 6.3; harvested in removed_index.md"),
    ("AGENTS.md", "KEEP", "rewritten for V8 (10.5)", "goal 10.5"),
    ("CLAUDE.md", "KEEP", "rewritten as @AGENTS.md + Claude notes (10.5)", "goal 10.5"),
    ("README.md", "KEEP", "rewritten in Korean + English (9.1)", "goal 9.1"),
    ("CHANGELOG.md", "KEEP", "new in V8-0 (9.7)", "goal 9.7"),
    ("DESIGN.md", "KEEP", "M-UX interface design system: theme, layout and interaction guidance", "DESIGN.md; docs/v8/mux/MUX_GOAL.md"),
    ("addons/", "DELETE", "harness MCP addon; project.godot has no [editor_plugins] section, nothing loads it", "grep editor_plugins project.godot → none; " + E_REACH),
    ("hooks/", "DELETE", "V7 git pre-commit harness hook", "goal 6.3"),
    ("icon.svg", "KEEP", "project icon (project.godot config/icon)", E_REACH),
    ("project.godot", "KEEP", "Godot project root", E_REACH),
    ("sim_bridge.gdextension", "KEEP", "GDExtension manifest loading libsim_bridge", E_REACH),
    ("stats/", "DELETE", "pre-V7 stat definition JSON; nothing in rust/ or reachable GDScript reads it", "grep -rn 'stats/' rust scripts → none; " + E_REACH),
    ("vlm_interactive_test_results.zip", "DELETE", "V7 VLM test artefact", "goal 6.3"),
    # ---- shaders, scenes, assets -----------------------------------------------------
    ("shaders/villager_ramp.gdshader", "KEEP", "palette-ramp shader used by art_sprite_library.gd (people colours; VISION §5 keeps palette swap)", E_REACH),
    ("shaders/settlement_outline.gdshader", "KEEP", "used by agent_renderer.gd (selection outline)", E_REACH),
    ("shaders/palette_swap.gdshader", "ASK", "unreferenced V7 MultiMesh palette-swap shader; VISION §5 keeps a palette-swap shader for clothing colours", "VISION §5; " + E_REACH),
    ("shaders/", "DELETE", "not referenced by the kept scripts/scenes", E_REACH),
    ("scenes/main.tscn", "KEEP", "main scene; deleted UI nodes removed (7.3)", E_REACH),
    ("scenes/ui/god_hand_panel.tscn", "DELETE", "god-hand panel scene (7.2)", "goal 7.2"),
    ("scenes/test/", "DELETE", "art calibration scene / Godot test code (6.3)", "goal 6.3"),
    ("scenes/", "KEEP", "scene root", E_GIT),
    ("assets/fonts/", "KEEP", "Galmuri pixel fonts (7.5 keeps assets/fonts/galmuri/**)", "goal 7.5; " + E_REACH),
    ("assets/ui/", "KEEP", "worldsim_theme.tres: project theme (project.godot gui/theme/custom)", E_REACH),
    ("assets/art/atlas/region_atlas.json", "DELETE", "atlas pipeline metadata; nothing at runtime reads it", E_REACH),
    ("assets/art/terrain/provenance.json", "DELETE", "terrain pipeline provenance; nothing at runtime reads it", E_REACH),
    ("assets/art/atlas/", "KEEP", "runtime atlas pages + region maps loaded by art_sprite_library.gd / carry_load_layer.gd", E_REACH),
    ("assets/art/terrain/", "KEEP", "terrain sheets + terrain_manifest.json loaded by terrain_renderer.gd", E_REACH),
    ("assets/art/contact_shadow.png", "KEEP", "loaded by agent_renderer.gd / building_renderer.gd", E_REACH),
    ("assets/art/villager_ramps.png", "KEEP", "palette ramps loaded by art_sprite_library.gd", E_REACH),
    ("assets/art/production/U1/04.png", "KEEP", "preloaded by activity_icons_p11.gd (dependency of agent_activity.gd)", E_REACH),
    ("assets/art/production/", "DELETE", "atlas source PNGs (packed into assets/art/atlas); art pipeline retired", E_REACH + "; atlas source_path fields"),
    ("assets/art/direction/", "DELETE", "V7 art direction references", E_REACH),
    ("assets/art/manifest.json", "DELETE", "art pipeline manifest; not read at runtime", E_REACH),
    ("assets/art/", "KEEP", "runtime art root", E_REACH),
    ("assets/sprites/palette_lut.png", "ASK", "palette LUT of the unreferenced palette_swap.gdshader (kept with it)", "VISION §5"),
    ("assets/sprites/agent_base.png", "ASK", "base sprite of the palette-swap kit (kept with it)", "VISION §5"),
    ("assets/sprites/buildings/", "ASK", "pre-art-line building sprites; VISION §6.3 asks to review reuse of furniture/buildings at the new size", "VISION §6.3; " + E_REACH),
    ("assets/sprites/furniture/", "ASK", "pre-art-line furniture sprites; VISION §6.3 review", "VISION §6.3; " + E_REACH),
    ("assets/sprites/", "DELETE", "unreferenced pre-art-line sprites (walls/floors are to be redrawn per VISION §6.3)", "VISION §6.3; " + E_REACH),
    ("assets/audio/", "DELETE", "unreferenced ambience loops", E_REACH),
    ("assets/tilesets/", "DELETE", "unreferenced V7 TileSet", E_REACH),
    ("assets/", "KEEP", "asset root", E_GIT),
    # ---- localization ---------------------------------------------------------------
    ("localization/fluent/", "DELETE", "Fluent source; V8 compiles compiled/*.json from the category JSONs (manifest source_format json); locale.gd never reads it", "scripts/core/locale.gd; tools/localization_compile.py run in scratch (Part 1)"),
    ("localization/key_registry.json", "KEEP", "written (and read) by the kept generator tools/localization_compile.py on every run; regenerated with only the kept keys (preserve_key_ids false)", "tools/localization_compile.py _write_key_registry"),
    ("localization/key_owners.json", "DELETE", "duplicate-key owner policy; the generator treats a missing file as no policy, and the pruned category JSONs carry no duplicates", "tools/localization_compile.py _load_key_owners; scratch dry run (Part 1)"),
    ("localization/", "KEEP", "runtime pipeline: manifest.json, compiled/*.json, category JSONs (keys pruned to the kept UI, 7.4)", "scripts/core/locale.gd; tools/v8/locale_keys.py"),
    # ---- scripts --------------------------------------------------------------------
    ("scripts/test/", "DELETE", "Godot test code (6.3); the V8 smoke is written fresh in tools/v8", "goal 6.3"),
    ("scripts/core/locale.gd", "KEEP", "Locale autoload (7.3 keep set)", "goal 7.3; project.godot [autoload]"),
    ("scripts/core/settlement_selection.gd", "KEEP", "dependency of kept renderers/panels (selected village state); M1 replaces", E_REACH),
    # V8-0 layout (8.2 renames, 8.7 moves; DECISIONS D24, D29)
    ("scripts/core/sim_reads.gd", "KEEP", "SimReads: every bridge read (moved from scripts/ui/ in 8.7)", "goal 8.7"),
    ("scripts/ui/hud/", "KEEP", "hud_topbar.gd (7.3 keep set; moved from scripts/ui/panels/ in 8.7)", "goal 7.3, 8.7"),
    ("re:^scripts/world/(nature_renderer|nature_canopy)\\.gd$", "KEEP", "live: main.tscn NatureRenderer draws the world's trees, berry bushes and deadwood; the M1 tile layer replaces it (ROADMAP M1)", "scenes/main.tscn; diff review round 1; DECISIONS D11, D31"),
    ("re:^scripts/world/(world_renderer|terrain_renderer|building_renderer|agent_renderer|camera_controller|daylight_controller)\\.gd$", "KEEP", "7.3 keep set (moved from scripts/ui/ in 8.7)", "goal 7.3, 8.7"),
    ("scripts/world/", "KEEP", "renderer helpers kept as dependencies of the 7.3 keep set (moved from scripts/ui/ in 8.7; M1 replaces where they draw V7 presentation)", E_REACH),
    ("scripts/rendering/", "DELETE", "god-hand effects renderer (7.2/7.3)", "goal 7.2"),
    ("re:^scripts/ui/(camera_controller|world_renderer|terrain_renderer|building_renderer|agent_renderer|daylight_controller)\\.gd$", "KEEP", "7.3 keep set", "goal 7.3"),
    ("re:^scripts/ui/panels/(hud_topbar|chronicle_panel|agent_inspector_panel|causal_panel)\\.gd$", "KEEP", "7.3 keep set", "goal 7.3"),
    ("re:^scripts/ui/(sim_reads|art_sprite_library|art_hash|art_season_tint|agent_activity|agent_frame_choices_p10|agent_pose_moments|agent_selection_index_p10|art_declump_p12|art_walker_p13b|camp_seats|carry_load_layer|chronicle_text|cold_effects|morning_chill|nature_layout|person_names|person_read_cache|shelter_cold|wood_visuals)\\.gd$",
     "KEEP", "transitive dependency of the 7.3 keep set (M1 replaces where it draws V7 presentation)", E_REACH),
    ("scripts/ui/cold_audit.json", "KEEP", "data read by cold_effects.gd (dependency); moved to scripts/world/ in 8.7", E_REACH),
    ("re:^scripts/ui/(activity_icons_p11|porter_presentation_p12|offscreen_marker)\\.gd$", "KEEP", "dependency of agent_renderer.gd / agent_activity.gd that cannot be cut without rewriting the renderer; listed under 'M1 replaces'", E_REACH),
    ("re:^scripts/ui/(nature_renderer|nature_canopy)\\.gd$", "KEEP", "live (main.tscn NatureRenderer: trees, berry bushes, deadwood); moved to scripts/world/ in 8.7; the M1 tile layer replaces it", "scenes/main.tscn; DECISIONS D11, D31"),
    ("scripts/ui/panels/art_ui.gd", "KEEP", "shared panel styling used by every kept panel (dependency)", E_REACH),
    ("re:^scripts/ui/.*\\.gd$", "DELETE", "UI outside the 7.3 keep set (overlays, badges, god hand, status/summary panels, LOD controller)", "goal 7.3; " + E_REACH),
    ("scripts/", "KEEP", "GDScript root", E_GIT),
    # ---- tools ----------------------------------------------------------------------
    ("tools/v8/", "KEEP", "V8-0 tooling (identity, gate, inventory, reachability, docs checks)", "goal 4.1, 6.1, 11.1"),
    ("tools/harness/gdscript_strict_check.sh", "MOVE→tools/v8/gdscript_strict_check.sh", "kept gate step 8 (6.3)", "goal 6.3"),
    ("tools/revitalize/run_bounded.sh", "MOVE→tools/v8/run_bounded.sh", "portable timeout used by the gate (6.3)", "goal 6.3"),
    ("tools/localization_compile.py", "KEEP", "the generator of localization/compiled/*.json (reproduced byte-identically in scratch at base)", "Part 1 run: cmp identical ko/en"),
    ("tools/localization_compile_fluent.py", "DELETE", "writes fluent .ftl from compiled JSON; fluent retired", "tool docstring"),
    ("tools/generate_locale_from_json.py", "DELETE", "pre-V7 trait locale generator (reads data/species/…, which no longer exists)", "tool source; ls data → absent"),
    ("tools/localization_audit.py", "DELETE", "V7 audit for inline *_en/*_ko data fields and fluent parity", "tool docstring"),
    ("tools/revitalize/", "DELETE", "pass-specific revitalize tooling (identity/gate/perf per pass); reusable parts copied to tools/v8", "goal 6.3"),
    ("tools/art_pipeline/", "DELETE", "art production pipeline (generation ledgers, boards, packers)", "goal 6.3"),
    ("tools/worldmodel/", "DELETE", "world-model research tooling (lives on codex/worldmodel-2026-10)", "goal 3.2, 6.3"),
    ("tools/harness/", "DELETE", "V7 harness pipeline and hooks", "goal 6.3"),
    ("tools/scripts/", "DELETE", "one-off script (6.3)", "goal 6.3"),
    ("tools/art/", "DELETE", "one-off art script (6.3)", "goal 6.3"),
    ("tools/.gdignore", "KEEP", "keeps Godot from importing tools/", E_GIT),
    ("re:^tools/(migrate_i18n|trait_migration|gen_trait_v3|fix_trait_centers|data_localization_extract|derive_composite_violation_stress|generate_building_sprites|find_unused_files|rust_shadow_cutover_check)\\.py$", "DELETE", "one-off migration script (6.3)", "goal 6.3"),
    ("re:^tools/(migration_verify.*|rust_shadow_.*)\\.sh$", "DELETE", "one-off migration script (6.3)", "goal 6.3"),
    ("tools/", "KEEP", "tools root", E_GIT),
    # ---- docs -----------------------------------------------------------------------
    ("docs/v8/harvest/", "DELETE", "harvest drafts, consumed by docs/skills and deleted at the end of Part 5", "goal 6.4, 10.5"),
    ("docs/v8/", "KEEP", "V8-0 goal, direction, decisions, inventory, report", "goal 0.1"),
    ("docs/archive/", "KEEP", "INDEX.md: permanent index of deleted docs (7.1)", "goal 7.1"),
    ("docs/decisions/", "KEEP", "ADRs (9.6)", "goal 9.6"),
    ("docs/images/", "KEEP", "at most 12 curated images (11.1 step 14)", "goal 11.1"),
    ("docs/VISION.md", "KEEP", "new (9.2)", "goal 9.2"),
    ("docs/ROADMAP.md", "KEEP", "new (9.3)", "goal 9.3"),
    ("docs/HISTORY.md", "KEEP", "new (9.4)", "goal 9.4"),
    ("docs/ARCHITECTURE.md", "KEEP", "new (9.5)", "goal 9.5"),
    ("re:^docs/(revitalize|art|worldmodel|analysis|design|guides|plans|reference|traits|localization|architecture)(/|$)", "DELETE", "pre-decided (6.3); harvested into docs/v8/harvest → HISTORY/skills; index in docs/archive/INDEX.md", "goal 6.3, 6.4"),
    ("re:^docs/(CHANGELOG\\.md|llm-setup\\.md|sprite_session_handoff_wildlife_a4\\.md|\\.gitkeep)$", "DELETE", "pre-decided (6.3); CHANGELOG harvested into HISTORY (pre-V7 paragraph)", "goal 6.3"),
    ("docs/", "KEEP", "docs root", E_GIT),
    # ---- rust -----------------------------------------------------------------------
    ("rust/crates/sim-systems/src/runtime/intervention.rs", "DELETE", "god hand (7.2)", "goal 7.2"),
    ("rust/crates/sim-engine/src/intervention.rs", "DELETE", "god hand (7.2)", "goal 7.2"),
    ("rust/crates/sim-core/src/rules.rs", "MOVE→rust/crates/sim-core/src/rules/mod.rs (+ baseline.rs, one file per sub-rule struct)", "8.1", "goal 8.1"),
    ("rust/crates/sim-core/src/material/", "KEEP", "material data model (not live; kept for M2)", "goal 6.3"),
    ("rust/crates/sim-core/src/variety.rs", "KEEP", "live: WorldVariety::draw_v18 feeds game_v26 rules/worldgen", "rules.rs game_v20; worldgen game_v18"),
    ("rust/crates/sim-core/src/", "KEEP", "reached by the live engine/bridge (every pub module has users in sim-systems/sim-engine/sim-bridge)", "Part 1 symbol scan $DATA/rust_module_refs.txt"),
    ("rust/crates/sim-systems/src/runtime/combat/", "ASK", "registered (priority 137) but changed no outcome in the base ablation (seeds 0-3, 120 days); M5 animal threats / drafting may reuse it", "V8_0_INVENTORY.md 2.3; $DATA/ablation"),
    ("rust/crates/sim-systems/src/runtime/", "KEEP", "registered in register_default_runtime_systems (live engine) or used by a registered system", "rust/crates/sim-systems/src/lib.rs register_default_runtime_systems"),
    ("rust/crates/sim-systems/src/worldgen/", "KEEP", "live world generation (build_world_with); old game_vNN configs removed in 8.1", "goal 8.1"),
    ("rust/crates/sim-systems/src/subprofile.rs", "KEEP", "feature-gated timing scopes used by worldsim_perf --features subprofile", "rust/crates/sim-test/Cargo.toml [features]"),
    ("rust/crates/sim-systems/src/", "KEEP", "sim-systems crate root", E_GIT),
    ("rust/crates/sim-engine/src/", "KEEP", "tick loop, resources, RuntimeSystem trait", E_GIT),
    ("rust/crates/sim-bridge/src/ffi/world_node.rs", "KEEP", "WorldSimNode (#[func] API); intervention methods removed (7.2); split by concern (8.5)", "goal 7.2, 8.5"),
    ("rust/crates/sim-bridge/src/", "KEEP", "GDExtension bridge", E_GIT),
    ("rust/crates/sim-test/src/worldmodel/", "DELETE", "world-model research (lives on codex/worldmodel-2026-10)", "goal 3.2"),
    ("rust/crates/sim-test/src/bin/worldsim_wm.rs", "DELETE", "world-model recorder binary", "goal 3.2"),
    ("rust/crates/sim-test/src/probe/ablation.rs", "DELETE", "ablation/bisection scenes of past passes", "goal 6.3"),
    ("re:^rust/crates/sim-test/src/probe/pass17_(batch|death)?_?tests\\.rs$", "KEEP", "unit tests of the kept probe modules (renamed in 8.2)", "src/probe/mod.rs"),
    ("re:^rust/crates/sim-test/src/probe/pass[0-9]+(_[a-z]+)?\\.rs$", "KEEP", "computes blocks of the probe summary/raw rows that identity compares (6.3 lists probe/pass*.rs for deletion, but deleting them would change the summary); renamed by content in 8.2", "src/probe/runner.rs, summary.rs; DECISIONS D9"),
    ("rust/crates/sim-test/src/subprofile_report.rs", "KEEP", "worldsim_perf --features subprofile report (perf tooling, not pass-only)", "src/perf.rs:306"),
    ("rust/crates/sim-test/src/", "KEEP", "test crate: probe, perf, scenes (reduced in 8.1)", "goal 7.7"),
    ("re:^rust/crates/sim-test/tests/pass17_(family|routine)_support/", "KEEP", "shared fixture module of kept (KEEP-ADAPT) pass17 test files; follows its users in 8.4", "V8_0_INVENTORY.md section 3.1"),
    ("rust/crates/sim-test/tests/", "TRIAGE", "see section 3.1", "V8_0_INVENTORY.md section 3.1"),
    ("rust/crates/sim-bridge/tests/fixtures/", "DELETE", "art_p12/art_p13b numeric fixtures written by deleted scripts/test/art capture tools", "fixture json provenance → scripts/test/art/*"),
    ("rust/crates/sim-test/benches/", "DELETE", "V7 Criterion harness for the Phase 2 influence hard gate; nothing runs it, and VISION §5 keeps one perf check (tools/v8/perf.sh)", "diff review round 1 F3; DECISIONS D31"),
    ("rust/crates/sim-core/benches/", "KEEP", "Criterion bench of the material model (`cargo bench -p sim-core`); the material model is kept as data for M2", "rust/crates/sim-core/Cargo.toml [[bench]]; DECISIONS D31"),
    ("rust/crates/sim-core/tests/data/", "KEEP", "v7_final rules/worldgen dumps (Part 0)", "goal 5.3"),
    ("rust/crates/sim-core/CLAUDE.md", "DELETE", "V7-era per-crate agent notes; superseded by docs/ARCHITECTURE.md + skills", "goal 10.5"),
    ("re:^rust/.*CLAUDE\\.md$", "DELETE", "V7-era agent notes; superseded by ARCHITECTURE + skills", "goal 10.5"),
    ("rust/", "KEEP", "Rust workspace", E_GIT),
]

TEST_VERDICTS = {}  # filled from docs/v8/harvest/test_triage.md when present


BASE_REF = "v7-final-art"
TABLE_MARKER = "<!-- verdict table: generated by tools/v8/inventory.py write; do not edit below -->"


def git_files(root):
    """Tracked files now."""
    return subprocess.run(["git", "ls-files"], cwd=root, capture_output=True, text=True, check=True).stdout.splitlines()


def inventory_files(root):
    """Files at the base tag plus files tracked now (deleted rows stay, new files get rows)."""
    base = subprocess.run(["git", "ls-tree", "-r", "--name-only", BASE_REF], cwd=root,
                          capture_output=True, text=True).stdout.splitlines()
    return sorted(set(base) | set(git_files(root)))


def match(path):
    for pattern, verdict, reason, evidence in RULES:
        if pattern.startswith("re:"):
            if re.search(pattern[3:], path):
                return verdict, reason, evidence
        elif path == pattern or (pattern.endswith("/") and (path + "/").startswith(pattern)):
            return verdict, reason, evidence
    return "UNMATCHED", "", ""


def load_test_verdicts(root):
    """Per-file test verdicts from section 3 of docs/v8/V8_0_INVENTORY.md (above the generated table)."""
    inventory = Path(root, "docs/v8/V8_0_INVENTORY.md")
    if not inventory.exists():
        return {}
    text = inventory.read_text().split(TABLE_MARKER)[0]
    out = {}
    for line in text.splitlines():
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) >= 5 and cells[0].strip("`").endswith(".rs") and cells[1].split() \
                and cells[1].split()[0] in {"KEEP", "DELETE", "KEEP-TRIM", "KEEP-ADAPT"}:
            name = cells[0].strip("`")
            if not name.startswith("rust/"):
                name = "rust/crates/sim-test/tests/" + name
            out[name] = (cells[1], cells[4], "V8_0_INVENTORY.md section 3.1 (test triage)")
    return out


def units(files):
    out = set()
    for f in files:
        parts = f.split("/")
        out.add(parts[0] + ("/" if len(parts) > 1 else ""))
        if parts[0] in ("scripts", "scenes", "tools", "shaders"):
            out.add(f)
        if f.startswith("rust/crates/") and ("/src/" in f or re.match(r"rust/crates/sim-test/tests/[^/]+\.rs$", f)) and f.endswith(".rs"):
            out.add(f)
        if parts[0] in ("assets", "localization"):
            for depth in range(2, len(parts)):
                out.add("/".join(parts[:depth]) + "/")
        if parts[0] == "docs":
            for depth in range(2, min(len(parts), 4)):
                out.add("/".join(parts[:depth]) + "/")
            if len(parts) == 2:
                out.add(f)
    return sorted(out)


def exceptions(files, unit_set):
    """Files whose verdict differs from the nearest directory unit containing them."""
    out = set()
    for f in files:
        if f in unit_set:
            continue
        parts = f.split("/")
        for depth in range(len(parts) - 1, 0, -1):
            d = "/".join(parts[:depth]) + "/"
            if d in unit_set:
                if match(f)[0] != match(d.rstrip("/"))[0]:
                    out.add(f)
                break
    return out


def rows(root):
    tests = load_test_verdicts(root)
    files = inventory_files(root)
    unit_set = set(units(files))
    def verdict_of(path):
        return tests[path] if path in tests else match(path)

    for unit in sorted(unit_set | exceptions(files, unit_set)):
        key = unit.rstrip("/")
        verdict, reason, evidence = verdict_of(key)
        if unit.endswith("/"):
            inside = {}
            for f in files:
                if f.startswith(unit):
                    v = verdict_of(f)[0].split()[0]
                    inside[v] = inside.get(v, 0) + 1
            if len(inside) > 1:
                summary = ", ".join(f"{n} {v}" for v, n in sorted(inside.items()))
                verdict, reason = "MIXED", f"files inside: {summary}; see the file/exception rows. Directory rule: {reason}"
        yield unit, verdict, reason, evidence


def table(root):
    lines = ["| path | verdict | reason | evidence |", "|---|---|---|---|"]
    for unit, verdict, reason, evidence in rows(root):
        lines.append(f"| `{unit}` | {verdict} | {reason.replace('|', '/')} | {evidence.replace('|', '/')} |")
    return "\n".join(lines)


def covered_prefixes(inventory_text):
    prefixes = set()
    for line in inventory_text.splitlines():
        m = re.match(r"\|\s*`([^`]+)`\s*\|", line)
        if m:
            prefixes.add(m.group(1))
    return prefixes


def check(inventory, root):
    prefixes = covered_prefixes(Path(inventory).read_text())
    uncovered = []
    for f in git_files(root):
        if not any(f == p or (p.endswith("/") and f.startswith(p)) for p in prefixes):
            uncovered.append(f)
    print(f"inventory closure: {len(git_files(root))} tracked paths, {len(prefixes)} rows, {len(uncovered)} uncovered")
    for f in uncovered[:50]:
        print(f"  uncovered: {f}")
    return 0 if not uncovered else 1


def write(inventory, root):
    path = Path(inventory)
    head = path.read_text().split(TABLE_MARKER)[0].rstrip() + "\n\n" if path.exists() else ""
    path.write_text(head + TABLE_MARKER + "\n\n" + table(root) + "\n")
    print(f"wrote {path}")


def main(argv):
    if len(argv) >= 3 and argv[1] == "write":
        write(argv[2], argv[3] if len(argv) > 3 else ".")
        return 0
    if len(argv) >= 2 and argv[1] == "table":
        print(table(argv[2] if len(argv) > 2 else "."))
        return 0
    if len(argv) >= 3 and argv[1] == "check":
        return check(argv[2], argv[3] if len(argv) > 3 else ".")
    print(__doc__.strip(), file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
