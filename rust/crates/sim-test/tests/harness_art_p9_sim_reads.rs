//! Source migration boundary: every script, including mixed fake/real tests.
use std::{fs, path::{Path, PathBuf}};

const PROTECTED: &[&str] = &[
    "get_agent_snapshot", "get_settlement_snapshot", "get_construction_snapshot",
    "get_building_snapshot", "get_resource_snapshot", "get_time_snapshot", "get_tallies",
    "get_recent_deaths", "get_society_snapshot", "get_intervention_state", "get_world_info",
    "get_terrain_snapshot", "apply_intervention", "advance_ticks", "reset_world",
    "reset_world_stress", "on_building_placed", "get_activity_snapshot",
];

#[derive(Debug)]
struct Token { value: String, quoted: bool, line: usize, function: String }

// Comments and strings are lexical units, so fake definitions and quoted source
// replacement fixtures cannot be mistaken for executable member calls.
fn tokens(source: &str) -> Vec<Token> {
    let mut result = Vec::new();
    let mut function = String::new();
    for (line_number, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        if let Some(signature) = trimmed.strip_prefix("func ").or_else(|| trimmed.strip_prefix("static func ")) {
            function = signature.split('(').next().expect("function name").to_owned();
        } else if !line.starts_with([' ', '\t']) && !trimmed.is_empty() && !trimmed.starts_with('#') {
            function.clear();
        }
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == '#' { break; }
            if c.is_whitespace() { i += 1; continue; }
            let quoted = c == '"' || c == '\'';
            let mut value = String::new();
            if quoted {
                i += 1;
                while i < chars.len() && chars[i] != c {
                    if chars[i] == '\\' && i + 1 < chars.len() { i += 1; }
                    value.push(chars[i]); i += 1;
                }
                i += 1;
            } else if c.is_alphanumeric() || c == '_' {
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    value.push(chars[i]); i += 1;
                }
            } else { value.push(c); i += 1; }
            result.push(Token { value, quoted, line: line_number + 1, function: function.clone() });
        }
    }
    result
}

fn protected(name: &str) -> bool { PROTECTED.contains(&name) }

fn safe_counter_dispatch(ts: &[Token]) -> bool {
    let allowed = ["get_drawn_head_dot_count", "get_drawn_icon_count", "get_drawn_icon_agent_ids",
        "capture_metadata", "get_feed_log", "get_visible_feed", "capture_feed_metadata", "get_death_total"];
    let mut calls = 0;
    for (i, t) in ts.iter().enumerate().filter(|(_, t)| t.value == "_draw_counter" && !t.quoted) {
        if i > 0 && ts[i - 1].value == "func" { continue; }
        let Some(args) = ts.get(i + 1..i + 7) else { return false; };
        if args[0].value != "(" || !args[1].quoted || args[2].value != ","
            || !args[3].quoted || !allowed.contains(&args[3].value.as_str()) || args[4].value != ")" {
            return false;
        }
        if args[1].value.contains("WorldSim") || t.function == "_draw_counter" { return false; }
        calls += 1;
    }
    calls > 0
}

fn violations(path: &str, source: &str) -> Vec<String> {
    if path == "scripts/ui/sim_reads.gd" { return Vec::new(); }
    let ts = tokens(source);
    let safe_counter = matches!(path, "scripts/test/capture_framings.gd" | "scripts/test/capture_framings_p9.gd" | "scripts/test/capture_framings_p10.gd" | "scripts/test/capture_framings_p11.gd")
        && safe_counter_dispatch(&ts);
    let mut errors = Vec::new();
    for (i, t) in ts.iter().enumerate() {
        if t.quoted || t.value != "." { continue; }
        let Some(call) = ts.get(i + 1..i + 4) else { continue; };
        if call[1].value != "(" { continue; }
        let method = &call[0].value;
        let old_path = path == "scripts/test/art/test_sim_reads_equivalence_p9.gd"
            && matches!(t.function.as_str(), "old_read" | "old_mutate");
        if old_path { continue; }
        let dynamic = matches!(method.as_str(), "call" | "callv");
        let arg = &call[2];
        let assembled = ts.get(i + 4).is_some_and(|next| next.value == "+");
        let safe_helper = safe_counter && t.function == "_draw_counter"
            && i > 0 && ts[i - 1].value == "node" && arg.value == "method";
        let typed_reader = i > 0 && ts[..i].windows(4).any(|decl|
            decl[0].value == "var" && decl[1].value == ts[i - 1].value
                && decl[2].value == ":" && decl[3].value == "SimReads"
                && decl[0].function == t.function);
        if (!typed_reader && protected(method)) || (dynamic && (protected(&arg.value)
            || (arg.value != ")" && (!arg.quoted || assembled) && !safe_helper))) {
            errors.push(format!("{path}:{} {}: direct or unresolved dispatch {}", t.line, t.function, method));
        }
    }
    errors
}

fn scripts(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("script directory") {
        let path = entry.expect("script entry").path();
        if path.is_dir() { scripts(&path, out); }
        else if path.extension().is_some_and(|ext| ext == "gd") { out.push(path); }
    }
}

#[test]
fn every_game_test_capture_and_transitive_support_script_routes_world_reads() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut files = Vec::new();
    scripts(&root.join("scripts"), &mut files);
    files.sort();
    assert!(files.len() > 100, "all-script coverage must not collapse");
    let mut errors = Vec::new();
    for file in &files {
        let relative = file.strip_prefix(&root).expect("repo script").to_string_lossy();
        errors.extend(violations(&relative, &fs::read_to_string(file).expect("script source")));
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

#[test]
fn scanner_detects_calls_comments_fakes_and_dynamic_helpers() {
    for source in ["world.get_agent_snapshot()", "world . reset_world ( 1 )",
        "world.get_activity_snapshot()", "world.call(\"get_activity_snapshot\")",
        "world.call(\"get_tallies\")", "world.callv('advance_ticks', [1])",
        "func _snapshot(method):\n return world.call(method)",
        "world.call(\"get_\" + name)",
        "func get_tallies():\n return {}\nfunc run():\n world.get_tallies()"] {
        assert!(!violations("scripts/test/mixed.gd", source).is_empty(), "missed: {source}");
    }
    for source in ["# world.get_tallies()", "func get_tallies():\n return {}",
        "world.has_method('get_tallies')", "var source = 'world.get_tallies()'",
        "SimReads.read(world, 'get_tallies')", "world.get_agent_detail(1)"] {
        assert!(violations("scripts/test/mixed.gd", source).is_empty(), "false positive: {source}");
    }
    let helper = "static func old_read(world, getter):\n return world.call(getter)\nstatic func read(world, getter):\n return world.call(getter)";
    assert_eq!(violations("scripts/test/art/test_sim_reads_equivalence_p9.gd", helper).len(), 1);
    assert_eq!(violations("scripts/test/not_equivalence.gd", helper).len(), 2);
    assert!(!safe_counter_dispatch(&tokens("_draw_counter('WorldSim', 'get_tallies')")));
    assert!(!safe_counter_dispatch(&tokens("_draw_counter('UI/HudTopbar', method)")));
}

#[test]
fn hearth_hoisting_keeps_original_snapshot_inputs_and_elapsed_clock() {
    let source = include_str!("../../../../scripts/ui/building_renderer.gd");
    let animate = source.split("func _animate_hearths()").nth(1).expect("hearth animation")
        .split("\nfunc ").next().expect("function body");
    let (before, body) = animate.split_once("for sprite: Sprite2D in _buildings.values():").expect("hearth loop");
    for input in ["SimReads.read(_world_sim, \"get_settlement_snapshot\")",
        "SimReads.read(_world_sim, \"get_time_snapshot\")", "float(clock.get(\"time_of_day\", 12.0))",
        "int(clock.get(\"winter_severity\", -1))"] { assert!(before.contains(input), "lost input: {input}"); }
    for consumer in ["WOOD.settlement(settlements, tile)", "WOOD.hearth_frame(state, hour, _elapsed, winter_severity)",
        "WOOD.cold_ash(state, hour, winter_severity)"] { assert!(body.contains(consumer), "lost consumer: {consumer}"); }
    assert!(!body.contains("SimReads.read"), "reads must remain outside hearth loop");
}

#[test]
fn reader_resource_and_node_are_additive_immediately_after_world() {
    let scene = include_str!("../../../../scenes/main.tscn");
    assert_eq!(scene.matches("path=\"res://scripts/ui/sim_reads.gd\"").count(), 1);
    assert!(scene.contains("[ext_resource type=\"Script\" path=\"res://scripts/ui/sim_reads.gd\" id=\"21_sim_reads\"]"));
    assert!(scene.contains("[node name=\"WorldSim\" type=\"WorldSimNode\" parent=\".\"]\n\n[node name=\"SimReads\" type=\"Node\" parent=\".\"]\nscript = ExtResource(\"21_sim_reads\")"));
}
