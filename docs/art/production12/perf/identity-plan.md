# Part 0 identity execution plan

Status: **NOT_RUN**. This is an offline plan, not a game_v25 identity result. No Godot or cargo was launched while preparing it. Prior P11 results and seed/tick choices are hypotheses only after the merge.

## Required framing derivation

P11 used `probe_baseline_p11.gd --mode=framings --case=P2/P3` plus baseline collector rows for P1/P4/P5/P6. Its derivation is `/Users/rexxa/work/art-production11-evidence/p3-runtime/framing-derivation.json`; its census outputs include complete/failures, world clock, population, settlement member counts, and fire state. Preserve the same evidence on the Part 0 tooling commit. Count **occupied** villages from positive `member_counts`, not just settlement IDs.

| Framing | Initial seed / tick hypothesis | Required semantics |
|---|---|---|
| P1 | 20261010 / 40560 | inhabited live village, Z4, lowest settlement ID hearth |
| P2 | 20261010 / 31320 | lit night, Z4, chosen hearth lit; record clock and fire flags |
| P3 | 20261013 / 46189 | at least two occupied villages, FV all-village framing |
| P4 | 20261013 / 126994 | at least two occupied villages, Z1 |
| P5 | 15100 / 1440 / stress 500 | stress world, Z4, 500 at reset; record actual capture population |
| P6 | 20261010 / 40560 | living person inspector and GodHand panel visible, Z4 |

P12 `capture_identity_p12.gd` currently retains these old values. Its new census records world info, settlements and clock, but screenshot presence alone cannot prove lit night or inhabited villages. `capture_framings_p12.gd` takes an external P3 plan and `P12_P3_TARGET_TICK`. Update these together after census. Do not relabel a stale census as game_v25.

## Preparation and census commands

Run exclusively through the leader's serialized Mac scheduler, outside quiet holds. After **each checkout**, build bridge then import before Godot. No parallel cargo/Godot. The following assumes the base worktree has already been created at the Part 0 tooling commit.

```bash
export GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot
export CARGO_BUILD_JOBS=6
export CAPTURE_EVIDENCE=/Users/rexxa/work/art-production12-evidence
base=/Users/rexxa/work/art-p12-base
identity="$CAPTURE_EVIDENCE/identity"
mkdir -p "$identity/derivation"
cd "$base"
while [[ -d /Users/rexxa/work/.mac_quiet.lock ]]; do sleep 30; done
(cd rust && cargo build -p sim-bridge)
bash tools/art_pipeline/run_bounded.sh 1800 "$GODOT_BIN" --headless --path "$base" --import
```

Tooling owner should copy `probe_baseline_p11.gd` to `scripts/test/art/probe_baseline_p12.gd` and replace P11 paths/env labels, without changing census meanings. Then:

```bash
for framing in P1 P2 P3 P4 P5 P6; do
  while [[ -d /Users/rexxa/work/.mac_quiet.lock ]]; do sleep 30; done
  env P12_BASELINE_OUT="$identity/derivation/$framing.json" \
    bash tools/art_pipeline/with_window_lock.sh 1800 \
    "$GODOT_BIN" --headless --path "$base" \
    --script res://scripts/test/art/probe_baseline_p12.gd -- --mode=framings --case="$framing" \
    > "$identity/derivation/$framing.log" 2>&1 || break
done
```

If P2 or P3/P4 fails its semantic predicate, the existing census supports exact searches. Use `--mode=search --purpose=night` or `--purpose=villages --min-villages=2`, with `--seed=20261011 --last-seed=20261042 --start-tick=0 --max-tick=172800 --stride=1`. Same wrapper/env/log pattern applies. A stride larger than 1 proves only first sample, never first tick. Stop/retry within timebox; preserve missing framing as NOT_VERIFIED rather than inventing a date.

Record a derivation JSON with tooling commit, actual loaded dylib SHA256, census collector SHA256, each raw census path/hash, seed/tick/population/occupied-village count/fire state, and why each framing qualifies. `get_world_info` need not expose a profile name: record the source build provenance that makes `reset_world` game_v25, and retain the stress profile separately.

## P3 plan and six independent captures

Convert a verified P3 census tick to calendar using its **own** `ticks_per_day` and `clock_offset_ticks`. The capture plan's day is 1-based: `day = floor((tick + offset)/ticks_per_day)+1`; minute is the remaining fraction multiplied by 1440. Existing P11 example was `[["RESET",20261013],[33,"09:49","FV","all"]]`. That old plan is not evidence for this pass. A non-minute-aligned tick must first be checked against the capture script's date-to-tick conversion; do not silently round it.

Save the derived plan as `$identity/derivation/p3-plan.json` and its exact tick as `$identity/derivation/p3-tick.txt`. Commit any changed CASES / census FRAMINGS in the tooling commit, repoint/rebuild/import base, then recheck derivation at that exact commit before reference capture.

```bash
export P12_P3_PLAN="$identity/derivation/p3-plan.json"
export P12_P3_TARGET_TICK="$(cat "$identity/derivation/p3-tick.txt")"
bash tools/art_pipeline/run_identity_p12.sh "$base" "$identity/six" A
bash tools/art_pipeline/run_identity_p12.sh "$base" "$identity/six" Aprime
python3 tools/art_pipeline/pixel_identity_p12.py \
  "$identity/six/A" "$identity/six/Aprime" "$identity/six/A" \
  "$identity/reference-pixels"
```

Fresh output directories are mandatory on retries. The comparator using A as B checks reference reproducibility only; it does not prove candidate identity. Require all six control `pixels` and `max_channel` equal zero, complete exact draw-clock event equality, 1920×1080 framebuffers and matching seed/tick/camera/zoom/state/capture frames. The inherited comparator previously allowed nonzero control bounds outside P5; P12 must strengthen this to exact zero on **all six**, or assert zero explicitly before any PASS. Inspect actual current comparator before reliance.

Record per-launch HEAD/source/loaded-library hashes before, inside the acquired window lock and after, plus terminal exit, log errors and artifacts. P11 six-frame supplementary provenance was captured during A only; do not silently inherit that limitation. No watermark, mask, tolerance or recolouring.

## Running and pan references

Reuse the provenance wrapper `/Users/rexxa/work/art-production11-evidence/p3-prep/run_running_identity_p11.sh` as `tools/art_pipeline/run_running_identity_p12.sh`, replacing P11 labels and script paths only. It already snapshots HEAD, source tree and loaded bridge before/inside/after each launch; uses fresh output; checks logs/artifacts; waits outside quiet holds; serializes the window. Its source snapshot should also include assets/localization so renderer inputs cannot drift unnoticed.

```bash
bash tools/art_pipeline/run_running_identity_p12.sh "$base" "$identity/running-A"
python3 tools/art_pipeline/compare_running_identity_p12.py \
  "$identity/running-A" "$identity/running-A" "$identity/running-reference-validation.json"
```

This self-comparison proves only completeness, schedule and checksum structure. A candidate must be captured independently and compared to the frozen accepted reference.

| World | Seed/start tick | Frames | Schedule |
|---|---|---|---|
| W0 | 20261010 / 40560 | 600 | one SimReads advance every 2 frames; final tick 40860 |
| W200 | 15100 / 1440 / stress 200 | 600 | one SimReads advance every 2 frames; final tick 1740 |
| W500_FROZEN | 15100 / 1440 / stress 500 | 240 | no advance; final tick 1440 |
| W500_2X | 15100 / 1440 / stress 500 | 240 | one advance each frame; final tick 1680 |

All use fixed 60 FPS, 120 startup + 120 warmup frames, captured process frames starting 241, Z4. Pan starts 1200 world px left of lowest-ID hearth and moves 10 px per frame. Dump scope is viewport plus 64 screen px. Compare exact per-frame sprite position, texture/region, flip, visibility, modulate/self_modulate, z and child visibility; require nonempty observed agent set, correct populations, hashes and schedule. Running collector uses normal render loop, **not** the six-framing `--disable-render-loop` argument.

Candidate (b) additionally requires W-multi 600-frame moving-porter dumps. Current running CASES/comparator enumerate only the four rows above. Extend both with derived W-multi seed/tick and explicit schedule/provenance before evaluating (b); do not claim its four-world PASS covers porters.

## Evidence still needed

All game_v25 framing census, A/Aprime screenshots/clocks, running/pan dumps, per-launch provenance and validation remain runtime work. Existing `production11/perf/identity-reference/reference-validation.md` is historical PASS only. No P12 H7 or candidate result is established by this plan.
