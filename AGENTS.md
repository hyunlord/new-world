# AGENTS.md — WorldSim

WorldSim is a Dwarf Fortress–style stone-age settlement sim: a deterministic
Rust simulation (hecs ECS) behind a Godot 4.6 GDExtension bridge, with
GDScript only for drawing, input and UI. Direction: [docs/VISION.md](docs/VISION.md).
Structure: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). This file is canonical
for every coding agent; `CLAUDE.md` imports it.

**Extensibility contract (ADR 0033, standing owner directive 2026-10-24):** kinds are data, behaviour is code.
Content (items, materials, styles, recipes, knowledge, apparel, looks, animals, pressures, eras, labels) lives in
versioned data with a registry, a resolver and a category fallback; code branches on tags and categories, never on a
content name; no new content enum (an unavoidable one is listed as M-EXT debt in the pass's DECISIONS); every family
gets a "data only" test; every review checks for new hard-coded content names (`scripts/world/**` is grepped for
manifest ids in the gate).

**Colony mind authoring:** follow [MIND_AUTHORING](docs/v8/core/MIND_AUTHORING.md) for facts, registered lifecycle effects and `mind.ron` nodes. Keep policy out of providers and baseline imports out of colony code; [ADR 0046](docs/decisions/0046-colony-mind.md) and [ADR 0047](docs/decisions/0047-baseline-isolation.md) record the committed colony switch and the separate baseline module isolation; preserve supported baseline constructors/save fixtures and keep final identity checks distinct from source moves.

## Branches and pushing

- M6a owner-approved exception: work and commit only in `/Users/rexxa/work/new-world-m6a-dev` on **`v8/m6a`**; follow `docs/v8/m6a/M6A_GOAL.md` for its publication procedure.
- Otherwise work and commit on **`v8/wip`**. Push with an explicit target: `git push origin v8/wip`, then check `git ls-remote origin v8/wip` equals local HEAD.
- **`v8/main`** moves only by fast-forward to a commit whose full gate is green.
- Never push `lead/main`, `main` or any `codex/*` branch. Never force-push, rewrite pushed history, or delete or move a remote branch or tag.
- Never discard work with a hard reset; inspect with `git status` / `git diff` and fix forward.
- The V7 lines and research are pinned: tags `v7-final-sim`, `v7-final-art`; world-model work lives on `codex/worldmodel-2026-10`.

## Pass coordination (depth roadmap v2 §7)

- Only one performance measurement job may run on this Mac at a time; overlapping measurements are both invalid. Admit each repetition at 1-minute load < 8 and interleave old/new binaries in the same session.
- Use pass-local temporary issue IDs (for example M6A-1) before publication; at publication assign the next free ISS numbers after checking the current ledger.
- Preserve both sides of documentation conflicts; stop and report code/data conflicts. A red gate means diagnose, fix and re-run; stop for code/data conflicts, an unfixable identity failure or the pass time limit.
- A bar that scores 0/8 on the first calibration round is a suspected specification defect: stop, report the mechanism, and get the bar confirmed or withdrawn before freezing and spending a held-out set (owner decision O-2, 2026-10-31).
- Held-out seed register: 1200–1231 M-UX2; 1232–1263 M-CORE; 1264–1295 M-CORE P-3d; 1296–1327 M6a. Allocate subsequent passes 32 seeds at a time from 1328 and record the allocation before evaluation. Calibrate on seeds 0–7 for at most four logged rounds; freeze before held-out and never tune on its results.

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
- The simulation contract (C1 reproducible, C2 view-independent, C3 measured budget, C4 closed-form/integer/keyed, C5 structure) is [ADR 0016](docs/decisions/0016-simulation-contract.md); each clause names its gate step.

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
bash tools/v8/gate.sh <log_dir>        # 36 steps; every step except perf and nav_bench must pass
```

Steps: fmt, bridge build, Godot import, tests (debug, release), clippy
(all targets), forbidden-math grep, GDScript strict check, headless smoke,
identity, determinism, perf (report only), bridge API, docs check,
inventory closure, Python unit tests, M1 tests, colony determinism,
command replay, `.uid` check, play check (`tools/v8/test_play.sh`), and since M1.5 (ADR 0016) snapshot schema
docs, renderer agreement, save round trip and the standing benchmark against `docs/perf/PERF_LEDGER.md`, since M3 the economy soak (`tools/v8/soak.sh`, since M4 also 0 `KnowledgeLost`) and the nav benchmark (report only), since M4 the knowledge step (the knowledge tests and the F10 loss fixture), since M5 the story step (two worlds × 48 days with the storyteller: F1's schedule, ledger and reservations exact; the soak runs without beasts), and since M5.5 the neighbours step (two worlds × 48 days: waves 1 and 2 accepted, a trade, ledger exact with the traded terms) and the bench's RSS outer bound (ADR 0016 amendment), and since M-ART2 (ADR 0033) the art manifest check, resolver parity with the art line, the extensibility grep tests and the headless renderer tests (plus appearance agreement inside renderer agreement), and since M-UX the steward fixture (two seeds ×24 days ON: deterministic, exact ledger and reservations, no panics), and Stage B adds `data_only_ext` (seven-family selected pack plus real-scene headless UI). Rebuild the bridge (`cargo build -p
sim-bridge`) before any Godot run — `cargo test` does not refresh the library
Godot loads.

## End-of-pass checklist

1. `CHANGELOG.md` entry (Added / Changed / Removed).
2. `docs/ROADMAP.md` row status.
3. `docs/HISTORY.md` line when a milestone ends or something is deleted.
4. An ADR in `docs/decisions/` for each design decision.
5. `docs/ARCHITECTURE.md`: regenerate the system table (`python3 tools/v8/system_table.py --write`), update maps if structure changed.
6. Skills: add a lesson when a problem needed more than one attempt, a better method was found, or Godot/gdext surprised you — in the same pass.
7. `tools/play.sh --headless-check main` passes; replace the current `docs/images/<pass>_contact_sheet.png` (now `mux_contact_sheet.png`, with `mux_compare_sheet.png` beside it) with the new pass's sheet when the view changed (≤ 12 images; delete the previous sheet).
8. `bash tools/v8/check_docs.sh` green.

Details: `.claude/skills/worldsim-docs-upkeep/SKILL.md`.

## Where things are

- Owner-facing (Korean): `README.md`, `docs/VISION.md`, `docs/ROADMAP.md`, `docs/HISTORY.md`.
- Agent-facing (English): `docs/ARCHITECTURE.md`, `docs/decisions/`, `CHANGELOG.md`, this file, `.claude/skills/`.
- Pass goals and their records: `docs/<line>/` (V8-0: `docs/v8/`; M1: `docs/v8/m1/`).
- Deleted material and where it lives: `docs/archive/INDEX.md`.
- Tooling: `tools/v8/` (gate, identity, perf, smoke, capture, contact sheet, inventory, docs checks); `tools/play.sh` and `tools/play/*.command` run any version for playing.
