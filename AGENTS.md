# AGENTS.md — WorldSim

WorldSim is a Dwarf Fortress–style stone-age settlement sim: a deterministic
Rust simulation (hecs ECS) behind a Godot 4.6 GDExtension bridge, with
GDScript only for drawing, input and UI. Direction: [docs/VISION.md](docs/VISION.md).
Structure: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). This file is canonical
for every coding agent; `CLAUDE.md` imports it.

## Branches and pushing

- Work and commit on **`v8/wip`**. Push with an explicit target: `git push origin v8/wip`, then check `git ls-remote origin v8/wip` equals local HEAD.
- **`v8/main`** moves only by fast-forward to a commit whose full gate is green.
- Never push `lead/main`, `main` or any `codex/*` branch. Never force-push, rewrite pushed history, or delete or move a remote branch or tag.
- Never discard work with a hard reset; inspect with `git status` / `git diff` and fix forward.
- The V7 lines and research are pinned: tags `v7-final-sim`, `v7-final-art`; world-model work lives on `codex/worldmodel-2026-10`.

## Staging and the working tree

- Stage explicit paths only (`git add path …`), never `git add -A` / `git add .`.
- Never stage `.gitignore`, `.ignore` or local tool state (`graft/`, `.omc/`); never touch the local `.claude/settings.json` or its skip-worktree flag.
- Raw measurement data (probe runs, captures, logs) lives outside the repository.

## Code rules

- Integers and permille only in `sim-core`, `sim-systems`, `sim-engine` — no `exp/ln/powf/sin/…` (the gate greps).
- Iterate maps in key order: `BTreeMap`/`BTreeSet` or sort first. Never insert or remove hecs components while iterating a hash collection.
- Keyed draws consume no stream state (`sim-core/src/rng.rs`); a skipped keyed draw changes nothing else.
- GDScript never writes simulation state; it reads the bridge (`SimReads`) and sends commands.
- No `.unwrap()` in production Rust you touch; return `Result`/`Option` or use a safe default.
- Every `src/**/*.rs` starts with a `//!` header (systems: priority, interval, reads, writes, events); every `.gd` starts with a comment block (purpose, bridge methods called).
- A change that is meant to keep behaviour must pass `tools/v8/identity.sh` before it is called behaviour-preserving.

## Skills (read before working in the area)

| skill | read when | path |
|---|---|---|
| worldsim-architecture | any Rust change; adding a system or a bridge read; crate boundaries, determinism, integer math | `.claude/skills/worldsim-architecture/SKILL.md` |
| worldsim-godot | any GDScript, scene, shader, import, capture or gdext/FFI work | `.claude/skills/worldsim-godot/SKILL.md` |
| worldsim-testing | writing tests, identity, determinism, the gate, perf comparisons | `.claude/skills/worldsim-testing/SKILL.md` |
| worldsim-pitfalls | before any non-trivial change: the log of problems that bit us | `.claude/skills/worldsim-pitfalls/SKILL.md` |
| worldsim-docs-upkeep | at the end of every pass | `.claude/skills/worldsim-docs-upkeep/SKILL.md` |
| systematic-debugging | a bug, failing test or unexpected behaviour | `.claude/skills/systematic-debugging/SKILL.md` |
| verification-before-completion | before claiming anything is done or fixed | `.claude/skills/verification-before-completion/SKILL.md` |

Codex reads the same skills through `.agents/skills/<name>` (symlinks).

## The gate

```bash
bash tools/v8/gate.sh <log_dir>        # 20 steps; every step except perf must pass
```

Steps: fmt, bridge build, Godot import, tests (debug, release), clippy
(all targets), forbidden-math grep, GDScript strict check, headless smoke,
identity, determinism, perf (report only), bridge API, docs check,
inventory closure, Python unit tests, M1 tests, colony determinism,
command replay, `.uid` check. Rebuild the bridge (`cargo build -p
sim-bridge`) before any Godot run — `cargo test` does not refresh the library
Godot loads.

## End-of-pass checklist

1. `CHANGELOG.md` entry (Added / Changed / Removed).
2. `docs/ROADMAP.md` row status.
3. `docs/HISTORY.md` line when a milestone ends or something is deleted.
4. An ADR in `docs/decisions/` for each design decision.
5. `docs/ARCHITECTURE.md`: regenerate the system table (`python3 tools/v8/system_table.py --write`), update maps if structure changed.
6. Skills: add a lesson when a problem needed more than one attempt, a better method was found, or Godot/gdext surprised you — in the same pass.
7. `bash tools/v8/check_docs.sh` green.

Details: `.claude/skills/worldsim-docs-upkeep/SKILL.md`.

## Where things are

- Owner-facing (Korean): `README.md`, `docs/VISION.md`, `docs/ROADMAP.md`, `docs/HISTORY.md`.
- Agent-facing (English): `docs/ARCHITECTURE.md`, `docs/decisions/`, `CHANGELOG.md`, this file, `.claude/skills/`.
- Pass goals and their records: `docs/<line>/` (V8-0: `docs/v8/`; M1: `docs/v8/m1/`).
- Deleted material and where it lives: `docs/archive/INDEX.md`.
- Tooling: `tools/v8/` (gate, identity, perf, smoke, capture, inventory, docs checks).
