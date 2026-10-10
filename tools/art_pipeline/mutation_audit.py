#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 tools/art_pipeline/mutation_audit.py inventory --root .
# Execution is deferred until the final target is committed.
# run --root /Users/rexxa/work/art-production-audit --expected-head <full SHA> --output <evidence>
# Copied from tools/revitalize/mutation_audit.py; original is read-only.
# Mac lane: eight cargo jobs, separate worktree, no Linux /proc memory probe.
# Stdlib CLI follows repository scripts; no dependency installation is required.
# noqa: SIZE_OK — ownership is explicitly limited to this standalone audit script; splitting needs leader scope extension.
"""Conservative source-token mutation inventory and isolated, resumable executor."""
from __future__ import annotations
import argparse
import dataclasses
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import shutil
import subprocess
import time
from typing import Final

LEX: Final = re.compile(r'//[^\n]*|/\*[\s\S]*?\*/|r(?P<h>\#*)"[\s\S]*?"(?P=h)|"(?:\\[\s\S]|[^"\\])*"|\x27(?:\\.|[^\x27\\])\x27')
OPS: Final = re.compile(r'\.(?:contains|find|rfind|matches|starts_with|ends_with|split|split_once|strip_prefix|strip_suffix)\s*\(|Regex::new\s*\(')

@dataclasses.dataclass(frozen=True, slots=True)
class Candidate:
    """One token/file/test tuple; unresolved tuples remain visible."""
    id: str
    test_file: str
    function: str
    token: str
    source_file: str
    status: str
    reason: str
    test_line: int
    source_sha256: str
    occurrence_count: int
    operation: str = "replace"

def decode_literal(raw: str) -> str:
    """Decode Rust ordinary/raw strings without corrupting Unicode."""
    if raw.startswith("r"):
        return raw[raw.index('"') + 1:raw.rindex('"')]
    text = re.sub(r'\\\r?\n\s*', '', raw[1:-1])
    return re.sub(r'\\(u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)',
                  lambda m: chr(int(m[1][2:-1].replace("_", ""), 16)) if m[1].startswith("u{")
                  else chr(int(m[1][1:], 16)) if m[1].startswith("x")
                  else {"n": "\n", "r": "\r", "t": "\t", "0": "\0", '"': '"', "'": "'", "\\": "\\"}.get(m[1], m[0]), text)

def literals(source: str) -> list[tuple[int, str]]:
    """Return offsets and exact decoded string values, excluding comments."""
    return [(m.start(), decode_literal(m[0])) for m in LEX.finditer(source)
            if m[0].startswith(('"', 'r'))]

def masked(source: str) -> str:
    """Preserve positions and newlines while hiding strings and comments."""
    return LEX.sub(lambda m: re.sub(r'[^\n]', ' ', m[0]), source)

def functions(source: str) -> dict[str, tuple[int, str]]:
    """Extract Rust function bodies with string-safe brace balancing."""
    result: dict[str, tuple[int, str]] = {}
    clean = masked(source)
    for m in re.finditer(r'\bfn\s+(\w+)\s*\(', clean):
        start = clean.find("{", m.end())
        if start < 0:
            continue
        depth = 1
        end = start + 1
        while end < len(clean) and depth:
            depth += (clean[end] == "{") - (clean[end] == "}")
            end += 1
        result[m[1]] = (source.count("\n", 0, m.start()) + 1, source[m.start():end])
    return result

def git(root: Path, *args: str) -> str:
    """Run bounded Git reads in the explicitly selected root."""
    return subprocess.check_output(["git", "-C", str(root), *args], text=True).strip()

def selected(root: Path) -> set[tuple[str, str]]:
    """Read every ledger row, including appended production-stage rows."""
    ledger = (root / "docs/art/TEST_LEDGER.md").read_text()
    pairs = set(re.findall(r'(rust/crates/sim-test/tests/[\w.-]+\.rs)\s*\x60?\s*\|\s*\x60?(\w+)', ledger))
    if not pairs:
        raise RuntimeError("Ledger contains no parseable test-function rows")
    return pairs

def inventory(root: Path) -> list[Candidate]:
    """Over-approximate helper dependencies; expose every unresolved mapping."""
    candidates: list[Candidate] = []
    for file, name in sorted(selected(root)):
        source = (root / file).read_text()
        fns = functions(source)
        if name not in fns:
            raise RuntimeError(f"Ledger function missing: {file}::{name}")
        seen: set[str] = set()
        todo = [name]
        bodies: list[str] = []
        while todo:
            current = todo.pop()
            if current in seen:
                continue
            seen.add(current)
            body = fns[current][1]
            bodies.append(body)
            todo.extend(c for c in re.findall(r'\b(\w+)\s*\(', masked(body)) if c in fns and c not in seen)
        scope = "\n".join(bodies)
        values = literals(scope)
        paths: set[str] = set()
        # Helpers often build paths from ["scripts", "ui", "file.gd"].
        path_values = [v for _, v in values]
        path_values += ["/".join(v for _, v in literals(m[0])) for m in re.finditer(r'\[[^\]]*\]', scope)]
        path_values += ["/".join(v for _, v in literals(m[0])) for m in re.finditer(r'(?:\.join\("[^"]+"\)\s*)+', scope)]
        for value in path_values:
            for path in (root / value, (root / file).parent / value):
                resolved = path.resolve()
                if resolved.is_relative_to(root) and resolved.is_file() and resolved.suffix in {".gd", ".tscn", ".gdshader", ".rs", ".ftl"}:
                    paths.add(str(resolved.relative_to(root)))
        body = fns[name][1]
        # Include all literals in the function: arrays, declaration helpers and
        # counted patterns would be missed by contains-only extraction.
        tokens = {v for _, v in literals(scope) if v}
        negative: set[str] = set()
        for m in re.finditer(r'!\s*\w+\.contains\(\s*((?:r\#*)?"(?:\\.|[^"\\])*"(?:\#*)?)\s*\)', body):
            negative.add(decode_literal(m[1]))
        for m in re.finditer(r'for\s+(\w+)\s+in\s+\[([^\]]*)\]', body):
            if re.search(r'!\s*\w+\.contains\(\s*' + re.escape(m[1]) + r'\s*\)', body):
                negative.update(v for _, v in literals(m[2]))
        direct = {decode_literal(m[1]) for m in re.finditer(r'\.(?:contains|find|matches|starts_with|ends_with)\s*\(\s*("(?:\\.|[^"\\])*")', body)}
        for token in sorted(tokens):
            matches = [(p, (root / p).read_text()) for p in sorted(paths)]
            present = [(p, s) for p, s in matches if token in s]
            reason = "literal in target function; helper-expanded source candidates"
            status = "eligible" if present else "absent_or_dynamic"
            if not present and re.search(r"(?:^A\d+|^a\d+|^\[|must |expected |got |not found|failed|read \{)", token):
                status = "diagnostic_literal"
            if token in path_values and ("/" in token or token.endswith((".gd", ".rs", ".tscn"))):
                status = "path_literal"
            if re.search(r'!\s*\w+\.contains\(\s*' + re.escape(json.dumps(token)) + r'\s*\)', body):
                reason = "negative guard: absent token cannot be removed; insertion needs separate audit"
            if not paths:
                reason = "source file unresolved; manual mapping required"
            if token not in direct:
                reason += "; indirect/array/regex/message literal needs interpretation"
            if token in negative and not present:
                for path, text in matches:
                    key = "\0".join((file, name, token, path, "insert"))
                    candidates.append(Candidate(hashlib.sha256(key.encode()).hexdigest()[:20], file, name, token, path,
                        "eligible_negative", "supplemental absence guard: inject a code line, not a stripped comment; source mapping needs review",
                        fns[name][0], hashlib.sha256(text.encode()).hexdigest(), 0, "insert"))
                continue
            for path, text in present or [("", "")]:
                key = "\0".join((file, name, token, path))
                candidates.append(Candidate(hashlib.sha256(key.encode()).hexdigest()[:20], file, name, token, path,
                    status, reason, fns[name][0], hashlib.sha256(text.encode()).hexdigest() if path else "", text.count(token) if token else 0))
        # Dynamic call arguments are visible even when no literal was extracted.
        for m in OPS.finditer(masked(body)):
            expression = body[m.end():body.find(")", m.end())]
            if expression.strip() and not expression.lstrip().startswith(('"', 'r#')):
                key = file + name + str(m.start())
                candidates.append(Candidate(hashlib.sha256(key.encode()).hexdigest()[:20], file, name,
                    expression, "", "dynamic_expression", "resolve argument/regex/conditional before claiming complete coverage",
                    fns[name][0], "", 0))
    return candidates

def test_outcome(code: int, output: str) -> str:
    """Zero selected tests and compile errors are never mutation kills."""
    if code < 0 or "running 1 test\n" not in output:
        return "invalid"
    if code == 0 and "1 passed; 0 failed" in output:
        return "passed"
    if code != 0 and "0 passed; 1 failed" in output:
        return "failed"
    return "invalid"

def mutated(before: bytes, token: str, operation: str) -> bytes:
    """Separate replacement from supplemental non-comment token insertion."""
    if operation.startswith("insert@"):
        name = operation.split("@", 1)[1]
        text = before.decode()
        start = re.search(r"\bfn\s+" + re.escape(name) + r"\s*\(", text)
        if start is None:
            raise RuntimeError(f"Missing insertion scope: {name}")
        opening = text.index("{", start.end()) + 1
        return (text[:opening] + "\n// " + token + "\n" + text[opening:]).encode()
    if operation == "insert_rust":
        return before + b"\n// " + token.encode() + b"\n"
    if operation == "insert":
        return before + b"\n" + token.encode() + b"\n"
    if operation == "replace_normalized":
        pattern = r"\s*".join(re.escape(char) for char in token if not char.isspace())
        return re.sub(pattern, "__MUTATED__", before.decode()).encode()
    if operation == "replace":
        return before.replace(token.encode(), b"__MUTATED__")
    raise RuntimeError(f"Unsupported mutation operation: {operation}")

def execute(root: Path, entry: Candidate, logfile: Path) -> str:
    """Rebuild in the audit tree, including include_str dependencies."""
    env = dict(os.environ, CARGO_BUILD_JOBS="8", CARGO_TARGET_DIR=str(root / "rust/target"))
    source = masked((root / entry.test_file).read_text())
    fn_start = re.search(r"\bfn\s+" + re.escape(entry.function) + r"\s*\(", source)
    if fn_start is None:
        raise RuntimeError(f"Missing test function: {entry.function}")
    stack: list[str] = []
    for part in re.finditer(r"\bmod\s+(\w+)\s*\{|[{}]", source[:fn_start.start()]):
        if part[0].endswith("{"):
            stack.append(part[1] or "")
        elif stack:
            stack.pop()
    qualified = "::".join([p for p in stack if p] + [entry.function])
    cmd = ["cargo", "test", "-p", "sim-test", "--test", Path(entry.test_file).stem, qualified,
           "--", "--exact", "--test-threads=1", "--nocapture"]
    with logfile.open("w") as log:
        process = subprocess.Popen(cmd, cwd=root / "rust", env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            code = process.wait(timeout=600)
        except subprocess.TimeoutExpired:
            code = -1
        finally:
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGTERM)
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait()
    return test_outcome(code, logfile.read_text())

def signature_for(root: Path, entry: Candidate) -> str:
    """Bind resume evidence to source, test body, operation and committed head."""
    source_hash = hashlib.sha256((root / entry.source_file).read_bytes()).hexdigest()
    test_hash = hashlib.sha256((root / entry.test_file).read_bytes()).hexdigest()
    return hashlib.sha256((entry.id + source_hash + git(root, "rev-parse", "HEAD") + test_hash).encode()).hexdigest()

def recover(root: Path, output: Path) -> None:
    """Validate isolation and restore interrupted mutation before source parsing."""
    if root != Path("/Users/rexxa/work/art-production-audit"):
        raise RuntimeError("Execution requires /Users/rexxa/work/art-production-audit worktree")
    if not (root / ".git").is_file():
        raise RuntimeError("Audit root must be a Git worktree")
    common = Path(git(root, "rev-parse", "--path-format=absolute", "--git-common-dir"))
    if root.parent != common.parent.parent:
        raise RuntimeError("Audit worktree must be a sibling of its main clone")
    active = output / "active-mutant.json"
    if active.exists():
        recovery = json.loads(active.read_text())
        path = root / recovery["source_file"]
        before = (output / recovery["backup"]).read_bytes()
        if path.read_bytes() not in (before, mutated(before, recovery["token"], recovery["operation"])):
            raise RuntimeError("Interrupted mutant has unrelated edits; preserve and review")
        path.write_bytes(before)
        active.unlink()


def run(root: Path, entries: list[Candidate], output: Path) -> None:
    """Run under the caller-held audit lock after guarded recovery."""
    active = output / "active-mutant.json"
    if git(root, "status", "--porcelain", "--untracked-files=no"):
        raise RuntimeError("Audit worktree has tracked changes; restore/review before run")
    if shutil.disk_usage(root).free < 15 * 1024**3:
        raise RuntimeError("Less than 15 GiB disk available; defer heavy phase")
    results = output / "results.jsonl"
    previous = [json.loads(line) for line in results.read_text().splitlines()] if results.exists() else []
    latest = {row["signature"]: row["outcome"] for row in previous}
    completed = {signature for signature, outcome in latest.items() if outcome in {"killed", "survived"}}
    baselines: dict[tuple[str, str], str] = {}
    baseline_logs: dict[tuple[str, str], str] = {}
    for entry in entries:
        if entry.status not in {"eligible", "eligible_negative"}:
            continue
        path = root / entry.source_file
        before = path.read_bytes()
        signature = signature_for(root, entry)
        if signature in completed:
            continue
        key = (entry.test_file, entry.function)
        prefix = entry.id + "-" + signature[:12] + "-" + str(time.time_ns())
        if key not in baselines:
            baseline_logs[key] = prefix + "-baseline.log"
            baselines[key] = execute(root, entry, output / baseline_logs[key])
        outcome = "baseline_invalid"
        backup = output / (entry.id + ".backup")
        if baselines[key] == "passed":
            backup.write_bytes(before)
            active.write_text(json.dumps({"source_file": entry.source_file, "backup": backup.name, "token": entry.token, "operation": entry.operation}))
            try:
                path.write_bytes(mutated(before, entry.token, entry.operation))
                result = execute(root, entry, output / (prefix + "-mutant.log"))
                outcome = {"passed": "survived", "failed": "killed", "invalid": "invalid"}[result]
            finally:
                path.write_bytes(before)
                active.unlink()
            backup.unlink()
        with results.open("a") as stream:
            stream.write(json.dumps(dict(dataclasses.asdict(entry), signature=signature, outcome=outcome, baseline=baselines[key], baseline_log=baseline_logs[key], mutant_log=prefix + "-mutant.log", survivor_reason="pending contract diagnosis" if outcome == "survived" else "", time=time.time())) + "\n")
            stream.flush()
            os.fsync(stream.fileno())


def resolved_inventory(root: Path) -> tuple[list[Candidate], list[dict[str, str]]]:
    """Resolve this ledger's source bindings; fail visibly for unfamiliar tests."""
    result: dict[str, Candidate] = {}
    dispositions: list[dict[str, str]] = []
    camera = "scripts/ui/camera_controller.gd"
    world = "scripts/ui/world_renderer.gd"
    building = "scripts/ui/building_renderer.gd"
    terrain = "scripts/ui/terrain_renderer.gd"
    status = "scripts/ui/panels/hud_status_panel.gd"
    topbar = "scripts/ui/panels/hud_topbar.gd"
    scene = "scenes/main.tscn"
    helpers = {"read_camera_controller_src": camera, "read_world_renderer_src": world,
               "read_agent_renderer_src": "scripts/ui/agent_renderer.gd",
               "read_palette_shader_src": "shaders/palette_swap.gdshader",
               "read_trail_renderer_src": "scripts/ui/activity_trail_renderer.gd",
               "read_main_tscn_src": scene, "read_hud_status_panel_src": status,
               "read_hud_topbar_src": topbar}
    for file, name in sorted(selected(root)):
        line, body = functions((root / file).read_text())[name]
        clean = masked(body)
        strings = {p: t for p, t in literals(body)}
        bindings: list[tuple[int, str, str]] = []
        used: set[str] = set()
        def add(path: str, token: str, operation: str = "replace", why: str = "direct source predicate") -> None:
            text = (root / path).read_text()
            actuals = [token] if token in text or operation.startswith("insert") else []
            if not actuals and "Vector2(" in token:
                actuals = sorted(set(re.findall(r"\s*".join(re.escape(c) for c in token), text)))
                why += "; exact source spelling resolved through no_ws"
            if not actuals:
                dispositions.append({"file": file, "function": name, "token": token,
                                     "classification": "absent_alternative", "reason": why + "; no matching source occurrence"})
            for actual in actuals:
                key = "\0".join((file, name, path, actual, operation))
                ident = hashlib.sha256(key.encode()).hexdigest()[:20]
                result[ident] = Candidate(ident, file, name, actual, path,
                    "eligible_negative" if operation.startswith("insert") else "eligible", why, line,
                    hashlib.sha256(text.encode()).hexdigest(), text.count(actual), operation)
            used.add(token)
        # Bind each shadowed local at its definition rather than mixing files.
        for m in re.finditer(r"\blet\s+(\w+)\s*=\s*", clean):
            end = clean.find(";", m.end())
            expr = body[m.end():end]
            path = ""
            if "include_str!" in expr:
                values = literals(expr)
                if values:
                    path = str(((root / file).parent / values[0][1]).resolve().relative_to(root))
            for helper, target in helpers.items():
                if helper + "(" in expr:
                    path = target
            if "read_file(&[" in expr:
                pieces = [value for _, value in literals(expr)]
                relative = "/".join(pieces)
                if relative.endswith((".gd", ".tscn", ".gdshader")) and (root / relative).is_file():
                    path = relative
            if "MAIN_TSCN_PATH" in expr or 'read_file(&["scenes", "main.tscn"])' in expr:
                path = scene
            if "WORLD_RENDERER_PATH" in expr:
                path = world
            if not path:
                identifiers = set(re.findall(r"\b\w+\b", masked(expr)))
                prior = [p for _, var, p in bindings if var in identifiers]
                if prior:
                    path = prior[-1]
            if path:
                bindings.append((m.start(), m[1], path))
        def source_for(var: str, position: int) -> str:
            choices = [p for off, v, p in bindings if off < position and v == var]
            return choices[-1] if choices else ""
        # Ordinary positive/negative contains/find/matches and counted patterns.
        for m in re.finditer(r"(!?)\s*(\w+)\s*\.\s*(contains|find|matches|starts_with|ends_with|split_once)\s*\(", clean):
            p = m.end()
            while p < len(body) and body[p].isspace():
                p += 1
            if p not in strings:
                continue
            token = strings[p]
            path = source_for(m[2], m.start())
            if path and token:
                add(path, token, "insert" if m[1] else "replace",
                    f"{m[2]}.{m[3]} at test offset {m.start()}; lexical source binding")
        # Resolve literal arrays used by a loop's source predicate (for example reset state).
        for loop in re.finditer(r"for\s+(\w+)\s+in\s*\[([^\]]*)\]\s*\{", body):
            variable = loop[1]
            tail = body[loop.end():]
            predicate = re.search(r"(\w+)\.contains\(\s*" + re.escape(variable) + r"\s*\)", tail)
            if predicate:
                path = source_for(predicate[1], loop.start())
                if path:
                    for _, token in literals(loop[2]):
                        add(path, token, why="literal loop array source predicate")
        if name == "harness_needs_a5_renderer_and_bridge_contract":
            add("scripts/ui/need_bar_renderer.gd", "0.75", why="parsed visual warning threshold equality")
            dispositions.append({"file": file, "function": name, "token": "bridge mapping",
                "classification": "unchanged_bridge_contract",
                "reason": "FFI dict key/field assertions are untouched by art production; no bridge mutants are authorized or needed for the repinned rendering assertions."})
        # Include expressions used directly inside assertions.
        for m in re.finditer(r'include_str!\(\s*("(?:\\.|[^"\\])*")\s*\)\.contains\(\s*("(?:\\.|[^"\\])*")', body):
            path = str(((root / file).parent / decode_literal(m[1])).resolve().relative_to(root))
            add(path, decode_literal(m[2]), why="inline include_str source binding")
        # Resolve helper-parsed camera declarations and numeric alternatives.
        if "unique_decl_rhs" in body and "ZOOM_" in body:
            for token in ("ZOOM_MIN", "ZOOM_MAX", "ZOOM_DEFAULT"):
                if '"' + token + '"' in body:
                    add(camera, token, why="unique_decl_rhs identifier")
                    value = {"ZOOM_MIN": "Vector2(1.0,1.0)", "ZOOM_MAX": "Vector2(8.0,8.0)", "ZOOM_DEFAULT": "Vector2(4.0,4.0)"}[token]
                    add(camera, value, why="no_ws RHS equality/accepted-list contract")
            add(camera, "const ", why="unique_decl_rhs const/var declaration parser")
        if name == "harness_p12_alpha_a5_zoom_default_constant":
            add(camera, "ZOOM_DEFAULT", why="declaration search then accepted literal array")
            add(camera, "Vector2(4.0, 4.0)", why="accepted.iter().any contains(c); actual active alternative")
            add(camera, "\n", why="find newline bounds the ZOOM_DEFAULT source line")
        if name in {"harness_g_a13_existing_p12_alpha_a4_updated_to_eight", "harness_g_a14_existing_p13_alpha_a3_updated_to_eight"}:
            target = next(t for t in strings.values() if t.endswith(".rs"))
            path = "rust/crates/sim-test/tests/" + target
            for t in strings.values():
                if t.startswith("fn "):
                    add(path, t, why="split_once scopes the source test function")
            if "a15_" not in name:
                add(path, "\n}", why="split_once terminates the inspected maximum-zoom function")
            if "a15_" in name:
                for t in ("Vector2(8.0,8.0)", "Vector2(1.0,1.0)", "Vector2(4.0,4.0)"):
                    add(path, t, why="no_ws whole test-file positive literal")
                add(path, "Vector2(5.0,5.0)", "insert_rust", "whole test-file absence; comment is observed by no_ws")
            else:
                anchor = next(t[3:t.index("(")] for t in strings.values() if t.startswith("fn "))
                add(path, "Vector2(8.0,8.0)", why="no_ws scoped maximum-zoom literal")
                add(path, "Vector2(4.0,4.0)", "insert@" + anchor, "insert Rust comment inside inspected function slice; no comment stripping")
        if name.endswith("phase12_alpha_and_beta1_invariants_preserved") or name.endswith("phase12_alpha_beta1_beta2_invariants_preserved"):
            for t in ("zoom = Vector2(2, 2)", "camera_controller.gd", "script = ExtResource("):
                add(scene, t, why="read_file MAIN_TSCN_PATH; zoom OR actual spelling")
            for t in ("OVERLAY_ALPHA", "const ", "0.65"):
                add(world, t, why="line predicate and parsed OVERLAY_ALPHA float")
            add(world, "=", why="rfind equals delimiter before OVERLAY_ALPHA numeric parse")
        if name == "harness_p13_delta_a3_hud_topbar_declares_four_label_members":
            for t in ("var ", ": Label", "shelter", "agent", "settlement", "site"):
                add(topbar, t, why="Label declaration parser; no_ws ':Label'; casefolded want array")
        if name == "harness_p12_alpha_a3_zoom_min_constant":
            add(camera, "Vector2(1.0, 1.0)", why="ZOOM_MIN line equality checked after whitespace normalization")
        if name == "harness_p14_delta_a6_anchored_top_right_with_mouse_filter_ignore":
            for t in ("ArtUI.panel(self, Rect2(12, 836, 620, 232))", "mouse_filter = Control.MOUSE_FILTER_IGNORE"):
                add(status, t, why="needles array requires panel geometry and mouse pass-through")
        if name == "harness_p14_delta_a9_notification_feed_constants_present":
            for t in ("MAX_VISIBLE_NOTIFICATIONS", "NOTIF_FADE_FRAMES", "4", "480"):
                add(status, t, why="unique declaration and numeric parse feed limit/fade assertions")
        if name == "harness_p14_delta_a4_resource_count_consistency_with_phase_14_beta":
            for t in ("get_settlement_snapshot", "food_stocks", "get_resource_snapshot", "amounts", "get_tallies"):
                add(topbar, t, why="for source in literal array -> stripped.contains(source)")
        if name == "harness_p14_delta_a5_resource_labels_array_exact_order":
            for t in ("HUD_STOCKPILE_FOOD", "HUD_BERRIES", "UI_BIRTHS", "UI_DEATHS", "_loc(keys[index])"):
                add(topbar, t, why="literal key array or no_ws source predicate")
        if "hud_status_panel_phase_14_delta_intact" in name or name == "harness_zoom_a23_phase14d_hud_status_panel_intact":
            for t in ("get_time_snapshot", "get_resource_snapshot", "get_tallies"):
                add(topbar, t, why="needles.iter() missing-list must be empty")
        if name == "harness_zoom_a19_phase12a_13a_camera_controller_zoom_invariants_intact":
            for t in ("ZOOM_MIN", "ZOOM_MAX", "ZOOM_DEFAULT", "Vector2(1.0,1.0)", "Vector2(8.0,8.0)", "Vector2(4.0,4.0)"):
                add(camera, t, why="tokens/lits missing-list; no_ws for vector literals")
        if name == "harness_p14_epsilon_a4_trail_alpha_constant_equals_0_4":
            for t in ("TRAIL_ALPHA", "const ", "0.8"):
                add("scripts/ui/activity_trail_renderer.gd", t, why="unique_decl_rhs then parse_float_rhs exact0.8")
        if "main_tscn_load_steps_equals" in name or name in {"harness_needs_a6_scene_wiring_additive", "harness_death_a6_scene_wiring_additive"}:
            for t in ("[gd_scene", "load_steps", "19", "="):
                add(scene, t, why="tscn header parser; exact numeric load_steps19")
        if name in {"harness_needs_a6_scene_wiring_additive", "harness_death_a6_scene_wiring_additive"}:
            for t in strings.values():
                if t.startswith(('name="', 'type="', '[ext_resource', '[node ', 'path="res://', 'script =', 'id="')) or re.fullmatch(r"\w+\.gd", t):
                    add(scene, t, why="scene node/resource predicate or for-token preservation array")
            if name.startswith("harness_death"):
                death_line = next(l for l in (root / scene).read_text().splitlines() if 'path="res://scripts/ui/death_viz_renderer.gd"' in l)
                ext_id = re.search(r'id="([^"]+)"', death_line)[1]
                add(scene, '"', why="quote delimiter used to parse the death script resource id")
                add(scene, f'ExtResource("{ext_id}")', why="format!(ExtResource(ext_id)) resolved from exact death resource line")
        if name == "harness_building_renderer_reads_snapshot_only":
            for p in (world, building, terrain):
                for t in ("on_building_placed(", "RESOURCE_SEED", "VILLAGE_FIXTURE", "SOURCE_KIND_COLORS"):
                    add(p, t, "insert", "nested source and forbidden arrays explicitly quantify these three files")
        if name == "harness_economy_hud_reads_sim_time_and_tallies":
            for p in (topbar, status):
                for t in ("TICKS_PER_DAY", '"Berry', '"Wood', '"Tick %d'):
                    add(p, t, "insert", "nested HUD source/forbidden arrays")
        if name == "harness_p14_gamma_a23_world_renderer_no_extra_ffi_added":
            for t in ("world_sim.", "world_sim.get_agent_detail", "world_sim.connect(", 'world_sim.connect("world_reset", refresh_snapshot)'):
                add(world, t, why="manual FFI-name scanner / required member / conditional connection-count equality")
            add(world, "world_sim.unlisted_method()", "insert", "supplemental disallowed-call insertion exercises allowed-set difference")
            dispositions.append({"file": file, "function": name, "token": "allowed",
                "classification": "allowlist_not_required_presence", "reason": "12 allowed names authorize calls; only get_agent_detail presence is required. Optional names cannot be removed when absent."})
        count = sum(e.test_file == file and e.function == name for e in result.values())
        behavioral = name in {"harness_scarcity_a16_production_init_seeds_all_three_kinds_finite",
            "harness_resource_depletion_a9_production_scene_registers_finite_capacities",
            "harness_resource_depletion_a10_live_depletion_surfaces"}
        dispositions.append({"file": file, "function": name, "token": "",
            "classification": "behavioral_not_source_text" if behavioral else "resolved" if count else "unresolved_function",
            "reason": "historical provenance wording only; ECS behavioral predicates have no guarded source tokens" if behavioral else f"{count} precise source/function/token mutants"})
        for t in strings.values():
            if t in used:
                continue
            if t == "\n}":
                kind, why = "scope_delimiter", "split_once function-boundary delimiter has an explicit mutant"
            elif t == ":Label":
                kind, why = "normalized_token", "no_ws transforms ': Label' into ':Label'; source spelling mutant supplied"
            elif re.fullmatch("[0-9a-f]{64}", t):
                kind, why = "hash_expectation", "expected digest is test data, not source text; supplemental byte-change mutant supplied"
            elif name == "harness_p14_gamma_a23_world_renderer_no_extra_ffi_added":
                kind, why = "allowlist_or_required_member", "optional allowed names need not occur; scanner, required get_agent_detail and disallowed-call mutants supplied"
            elif t in {"", "Hill and Mountain atlas rows differ from Rust codes", "resource steps include the scene itself"} or t.startswith("B3 PASS"):
                kind, why = "diagnostic_or_parser_default", "assertion diagnostic, println output or empty unwrap_or parser fallback"
            elif t.startswith(("A", "a", "[")) or any(x in t for x in ("must", "missing", "expected", "{", "maximum zoom", "renderer contains", "HUD contains", "digit parse")):
                kind, why = "diagnostic_or_format", "assert/panic/expect/println diagnostic or format expression; resolved dynamic contracts listed separately"
            elif t.endswith((".gd", ".rs", ".tscn", ".gdshader")) or t in {"scripts", "ui", "panels", "scenes"}:
                kind, why = "path", "source-loading operand, not a source predicate"
            elif t.startswith("Vector2(") or t == "v_tint.rgb * palette_color.rgb":
                kind, why = "accepted_alternative", "OR/accepted-array variant absent from current guarded source; active spelling has a mutant"
            else:
                kind, why = "nonrequired_literal", "reviewed test literal; not an additional mandatory source occurrence; see resolved dynamic/helper contracts"
            dispositions.append({"file": file, "function": name, "token": t, "classification": kind, "reason": why})
        known_dynamic = {"'#'", "reset", "'\\n'", "c", "|l| {\\n            let t = l.trim_start(", "'='", "&compact.as_str(",
            "w", "&default_compact.as_str(", "source", "key", "n", "l", "&v", "forbidden", "token", "'\\\"'",
            '&format!("ExtResource(\\"{ext_id}\\"'}
        for call in OPS.finditer(clean):
            arg = body[call.end():body.find(")", call.end())]
            if arg.strip() and not arg.lstrip().startswith(('"', 'r#')):
                normalized = re.sub(r"\s+", " ", arg)
                known = (name == "harness_needs_a5_renderer_and_bridge_contract" and normalized.startswith("&format!")) or normalized in {"'\\n'", "'\"'"} or normalized in {re.sub(r"\s+", " ", x.replace("\\n", "\n")) for x in known_dynamic}
                dispositions.append({"file": file, "function": name, "token": arg,
                    "classification": "resolved_dynamic" if known else "unresolved_dynamic",
                    "reason": "reviewed array/scoped parser/numeric-range helper; concrete source tokens above" if known else "new dynamic expression requires recipe review"})
    return list(result.values()), dispositions

def assertion_accounting(root: Path, entries: list[Candidate]) -> list[dict]:
    """Link each historical ledger row to current token targets, never infer kills."""
    rows: list[dict] = []
    for number, line in enumerate((root / "docs/art/TEST_LEDGER.md").read_text().splitlines(), 1):
        if not line.startswith('| `rust/crates/sim-test/tests/'):
            continue
        cells = line.strip('| ').split(' | ')
        if len(cells) != 6:
            raise RuntimeError(f"Ambiguous ledger columns at line {number}")
        rows.append(dict(ledger_line=number, file=cells[0].strip('`'), function=cells[1].strip('`'),
                         old=cells[2][1:-1] if cells[2].startswith('`') and cells[2].endswith('`') else cells[2],
                         new=cells[3][1:-1] if cells[3].startswith('`') and cells[3].endswith('`') else cells[3], reason=cells[5],
                         row_sha256=hashlib.sha256(line.encode()).hexdigest()))
    compact = lambda text: re.sub(r'\s+', '', text)
    for index, row in enumerate(rows):
        effective = row['new']
        lineage: list[int] = []
        for later in rows[index + 1:]:
            if (later['file'], later['function']) != (row['file'], row['function']):
                continue
            old = later['old']
            if old in effective:
                effective = effective.replace(old, later['new'])
                lineage.append(later['ledger_line'])
            else:
                # Later ledger fragments may omit assert! wrappers of the original row.
                old_values = [value for _, value in literals(old)]
                new_values = [value for _, value in literals(later['new'])]
                if len(old_values) == len(new_values) == 1 and old_values[0] in effective:
                    effective = effective.replace(old_values[0], new_values[0])
                    lineage.append(later['ledger_line'])
        related = [entry for entry in entries if (entry.test_file, entry.function) == (row['file'], row['function'])]
        values = [value for _, value in literals(effective)]
        # Non-Rust prose rows quote the guarded fragment with Markdown backticks.
        if not values:
            values = re.findall(r'`([^`]+)`', effective) or [effective]
        review_reason = ''
        if effective.startswith('Zoom minimum'):
            values = ['Vector2(1.0,1.0)']
        elif effective.startswith('Reset invokes'):
            values = ['_retire_missing({})']
        elif any('hearth_frame(_hour(), _elapsed), "building"' in value for value in values):
            values = [value.replace('hearth_frame(_hour(), _elapsed), "building"', 'desired, "building"') for value in values]
            review_reason = 'Historical BUILDING_IDS argument migrated through Part4; final call passes desired, preserving snapshot coordinates.'
        elif row['function'].startswith('harness_p14_delta_a17_'):
            values = ['_locale.ltr(key)', '_activity_legend', 'event.keycode == KEY_L', '_locale.call("set_language",']
        elif 'stem and' in effective:
            values = ['U1_12', 'U1_11', 'U1_13']
            review_reason = 'Part2 fallback glyph was superseded by Part6 native warning icons; final icon predicates replace the historical shape checks.'
        elif effective == 'Renderer visual gate equals 0.75':
            values = ['0.75']
        elif effective == 'read_hud_topbar_src()':
            values = [entry.token for entry in related if entry.source_file == 'scripts/ui/panels/hud_topbar.gd']
            review_reason = 'Owner relocation affects the existing predicates in this function; each corresponding topbar token is separately targeted.'
        elif effective == 'max_n, 4,':
            values = ['4']
        elif effective == 'assert_eq!(n, 19,':
            values = ['19']
        elif effective.startswith('assert_eq!(ext_count_after, 18,'):
            values = ['[ext_resource type=']
            review_reason = 'Replacing the counted ext_resource header changes the actual source count; it targets the asserted resource total rather than the diagnostic text.'
        elif effective in {'must equal 4', 'load_steps must equal 19',
                           'renderer follows the rendered agent by identity at its native head anchor',
                           'renderer draws native warning atlas regions'}:
            row.update(effective_assertion=effective, superseded_by_lines=lineage, targets=[],
                       unmapped_predicates=[], assertion_disposition='non_predicate_reason',
                       accounting_reason='This row changes only assertion diagnostic wording; the adjacent ledger row for the same function changes and targets the actual predicate. Diagnostic wording does not inspect runtime source and has no independent source mutant.')
            continue
        targets = []
        missing = []
        for value in values:
            matches = [entry.id for entry in related if compact(value) == compact(entry.token)
                       or (len(value) > 3 and compact(value) in compact(entry.token))]
            if matches:
                targets.append(dict(predicate=value, candidate_ids=matches))
            else:
                missing.append(value)
        row.update(effective_assertion=effective, superseded_by_lines=lineage, accounting_reason=review_reason,
                   targets=targets, unmapped_predicates=missing,
                   assertion_disposition='mapped_pending_execution' if targets and not missing else 'needs_review')
    return rows

def score_assertions(rows: list[dict], records: list[dict]) -> bool:
    """Every mapped predicate needs a killed candidate; diagnostics need no mutant."""
    outcomes = {record["id"]: record["outcome"] for record in records}
    for row in rows:
        if row["assertion_disposition"] == "non_predicate_reason":
            continue
        if row["unmapped_predicates"] or not row["targets"]:
            row["assertion_disposition"] = "needs_review"
            continue
        for target in row["targets"]:
            target["outcomes"] = {ident: outcomes.get(ident, "not_run") for ident in target["candidate_ids"]}
            target["killed"] = any(value == "killed" for value in target["outcomes"].values())
        row["assertion_disposition"] = "killed" if all(target["killed"] for target in row["targets"]) else "mapped_pending_execution"
    return all(row["assertion_disposition"] in {"killed", "non_predicate_reason"} for row in rows)

def main() -> None:
    """Inventory is read-only; run requires an explicit isolated-tree command."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["inventory", "run"])
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--expected-head", default="", help="Required committed SHA for execution; inventory may inspect a working tree")
    parser.add_argument("--limit", type=int, default=0, help="Pilot: first N resolved mutants; 0 runs all")
    args = parser.parse_args()
    root = args.root.resolve()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    if args.mode == "run":
        if not args.expected_head or git(root, "rev-parse", "HEAD") != args.expected_head:
            raise RuntimeError("Run requires --expected-head matching the final committed target")
        signal.signal(signal.SIGTERM, lambda *_: (_ for _ in ()).throw(SystemExit(143)))
        with (output / "audit.lock").open("w") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            recover(root, output)
            process_inventory(root, output, args.mode, args.limit)
    else:
        process_inventory(root, output, args.mode, args.limit)


def process_inventory(root: Path, output: Path, mode: str, limit: int) -> None:
    """Parse clean sources, reconcile contracts, then optionally run mutations."""
    lexical = inventory(root)
    entries, dispositions = resolved_inventory(root)
    unresolved = [d for d in dispositions if d["classification"].startswith("unresolved")]
    reconciliation = []
    source_parts = {"CARGO_MANIFEST_DIR", "scripts", "ui", "panels", "scenes", "shaders", "rust", "crates", "sim-test", "tests"}
    for old in lexical:
        matches = [e for e in entries if e.test_file == old.test_file and e.function == old.function
                   and "".join(e.token.split()) == "".join(old.token.split())]
        notes = [d for d in dispositions if d["file"] == old.test_file and d["function"] == old.function and d["token"] == old.token]
        reason = ""
        if matches:
            wrong_file = old.source_file and all(e.source_file != old.source_file for e in matches)
            classification = "eliminated_cartesian_pair" if wrong_file else "resolved_contract"
            reason = "lexical source binding selects " + ", ".join(sorted({e.source_file for e in matches}))
        elif notes:
            classification = notes[0]["classification"]
            reason = notes[0]["reason"]
        elif old.token in source_parts or old.token.endswith((".gd", ".rs", ".tscn", ".gdshader")):
            classification, reason = "helper_source_loader", "path-building/helper input, not an inspected source token"
        elif old.token in {":=", "var ", "const "}:
            classification, reason = "inactive_parser_alternative", "declaration helper accepts const/var and =/:=; inspected declarations use const with =, covered explicitly"
        elif old.token == "//":
            classification, reason = "comment_stripping_syntax", "shader helper removes comments; comment delimiter presence is not required"
        elif old.token == "{v:08x}":
            classification, reason = "digest_format", "SHA256 hexadecimal formatter; supplemental whole-file byte mutation supplied"
        elif old.token.startswith("project root") or "{" in old.token:
            classification, reason = "helper_diagnostic", "helper expect/panic formatting text, not guarded source content"
        else:
            classification, reason = "unresolved_lexical", "requires source review"
            unresolved.append({"classification": classification, "token": old.token})
        reconciliation.append({"original_id": old.id, "function": old.function, "token": old.token,
                               "original_source_file": old.source_file, "classification": classification,
                               "reason": reason, "resolved_ids": [e.id for e in matches]})
    ledger_rows = assertion_accounting(root, entries)
    (output / "assertion-accounting.json").write_text(json.dumps(ledger_rows, indent=2) + "\n")
    (output / "reconciliation.json").write_text(json.dumps(reconciliation, indent=2) + "\n")
    (output / "resolution.json").write_text(json.dumps(dispositions, indent=2) + "\n")
    (output / "lexical-inventory.json").write_text(json.dumps([dataclasses.asdict(e) for e in lexical], indent=2) + "\n")
    (output / "inventory.json").write_text(json.dumps({"head": git(root, "rev-parse", "HEAD"), "functions": sorted(selected(root)),
        "lexical_resolution_complete": not unresolved, "assertion_coverage_complete": False, "audit_executed": False, "unresolved": unresolved, "limitation": "Lexical recipes are not exhaustive assertion coverage. Every ledger row still requires a killed candidate or an individually reviewed written reason; numeric/helper assertions require manual reconciliation.",
        "function_coverage": [{"file": f, "function": n, "test_body_sha256": hashlib.sha256(functions((root / f).read_text())[n][1].encode()).hexdigest(), "candidate_count": sum(e.test_file == f and e.function == n for e in entries)} for f, n in sorted(selected(root))],
        "candidates": [dataclasses.asdict(e) for e in entries]}, indent=2) + "\n")
    if mode == "run":
        if any(row["assertion_disposition"] == "needs_review" for row in ledger_rows):
            raise RuntimeError("Resolve every ledger assertion mapping before execution")
        if unresolved:
            raise RuntimeError("Resolve inventory blockers before running scored audit")
        run(root, entries[:limit] if limit > 0 else entries, output)
    if (output / "results.jsonl").exists():
        attempts = [json.loads(line) for line in (output / "results.jsonl").read_text().splitlines()]
        current_signatures = {signature_for(root, e) for e in entries}
        records = list({row["id"]: row for row in attempts if row["signature"] in current_signatures}.values())
        counts = {status: sum(row["outcome"] == status for row in records)
                  for status in ("killed", "survived", "invalid", "baseline_invalid")}
        scored = counts["killed"] + counts["survived"]
        assertion_complete = score_assertions(ledger_rows, records)
        (output / "assertion-accounting.json").write_text(json.dumps(ledger_rows, indent=2) + "\n")
        (output / "summary.json").write_text(json.dumps({"outcomes": counts, "attempts": len(attempts), "latest_mutants": len(records),
            "assertion_accounting_complete": assertion_complete,
            "kill_rate_scored_only": counts["killed"] / scored if scored else None,
            "lexical_resolution_complete": not unresolved, "assertion_coverage_complete": assertion_complete, "audit_complete": len(records) == len(entries) and not counts["invalid"] and not counts["baseline_invalid"], "unresolved_inventory": sum(e.status not in {"eligible", "eligible_negative", "diagnostic_literal", "path_literal"} for e in entries)}, indent=2) + "\n")
    print(json.dumps({"functions": len(selected(root)), "candidates": len(entries),
                      "eligible_replacements": sum(e.status == "eligible" for e in entries),
                      "eligible_insertions": sum(e.status == "eligible_negative" for e in entries), "lexical_resolution_complete": not unresolved, "assertion_coverage_complete": False, "audit_executed": mode == "run"}))

if __name__ == "__main__":
    main()
