# Direction-2 slice 2-5c — deposit/consume feed (snapshot-diff, matches existing HUD pattern)

> Repo: hyunlord/new-world · Branch: lead/main · Expected HEAD: 7daf28c1
> Final visual slice. Surface deposit/consume in the HUD notification feed by diffing 2-5a's
> per-settlement `food_stocks` — pure GDScript + locale. No Rust, no sim, no EventBus.

NO ENV-BYPASS. No Rust / sim-core / sim-systems / sim-engine change. No new EventBus / FFI surface.

## Section 1: Implementation Intent

The supply chain is now visible at rest (carry indicator 2-5b, stockpile HUD 2-5a) but the
EVENTS — a deposit landing, a famine draw — scroll by only as silent number changes. 2-5c adds
them to the existing HUD notification feed so the player reads
"Settlement 3 stored 4 Food" / "Settlement 3 used 2 Food (famine)" alongside "Settlement formed",
"Agent born", etc.

`hud_status_panel.gd` already polls `get_settlement_snapshot` every frame and diffs id-sets for
notifications (its header documents: "No global causal-events FFI; notifications derive from
snapshot id-set deltas only"). 2-5a added `food_stocks` (parallel to the settlement id array).
2-5c tracks the previous per-settlement food_stock and pushes a feed line when it crosses a
threshold — UP ⇒ deposit, DOWN ⇒ famine consume. Pure GDScript reading existing data + the
existing feed mechanism. No sim/Rust/EventBus change, consistent with the deliberate
snapshot-diff design.

## Section 2: What to Build

### GDScript — feed lines from food_stock deltas
`scripts/ui/panels/hud_status_panel.gd`:
- `const FEED_FOOD_DELTA_THRESHOLD: int = 3` — minimum |delta| to surface a line; below it the
  change accumulates against last-notified so routine per-tick churn aggregates into one line.
- `var _last_notified_food: Dictionary` keyed by settlement id → food_stock at last notification.
- Cache the Locale autoload: `var _locale: Node` set in `_ready()` via
  `get_node_or_null("/root/Locale")`, plus a `_loc(key)` helper returning
  `_locale.ltr(key) if _locale != null else key`. RATIONALE (verified): a bare `Locale.` global
  identifier does NOT compile under the static `--check-only` gate (it does not register autoload
  singletons). This is the same gate-safe pattern `world_renderer.gd` uses for its 2-5a Food label.
- In `_ingest_snapshots()`, integrate into the EXISTING settlement-snapshot loop. That loop already
  reads the `ids` key (`PackedInt64Array`) — the correct, type-matching id array (entity_bits ==
  settlement_id). Read `food_stocks` (`PackedInt32Array`, parallel to `ids`, same sorted order /
  length per the 2-5a contract) and, inside the existing `for i in sids_arr.size()` loop:
  - `if i >= foods_arr.size(): continue` (defensive, short/absent array);
  - new settlement id (not in `_last_notified_food`) → baseline it, no line;
  - `delta >= THRESHOLD` → push deposit line `_loc("HUD_FEED_STOCKPILE_DEPOSIT") % [sid, delta]`,
    update last-notified;
  - `delta <= -THRESHOLD` → push consume line `_loc("HUD_FEED_STOCKPILE_CONSUME") % [sid, -delta]`,
    update last-notified;
  - below threshold → leave last-notified unchanged so it accumulates.
- After the existing `_prev_settlement_ids = cur_s` cleanup, prune `_last_notified_food` for ids no
  longer present (mirror the dissolved-settlement cleanup).
- Reuse existing `_push(...)` + `_prune_old()` feed plumbing. No new feed UI.

NOTE (code-reality correction vs the original snippet): the original draft referenced a
`settlement_ids` key typed `PackedInt64Array`. The actual FFI `settlement_ids` key is
`PackedInt32Array`; the `PackedInt64Array` id array is the `ids` key (which the existing loop
already consumes). Both keys carry the same numeric value (settlement_id). Using `ids` keeps the
displayed "Settlement %d" consistent with the existing "Settlement %d formed/dissolved" lines and
needs only one array read.

### Out of scope (do NOT build)
- A real CausalEvent/EventBus FFI for deposits/consumes (contradicts the deliberate design).
- Per-agent attribution. Water/Wood/Stone feed (Food only).
- Fixing the EXISTING hardcoded-English feed strings — leave them (separate cleanup).
- Any Rust / sim / FFI change.

## Section 3: How to Implement

The localization SOURCE OF TRUTH is the fluent files `localization/fluent/{en,ko}/messages.ftl`
(`source_format: fluent_preferred`); the `localization/{en,ko}/<category>.json` files are kept in
parallel (2-5a touched both). Add the two keys to the fluent files (format `KEY = value`, `%d`
literal) AND the `ui` category JSON, then run `python3 tools/localization_compile.py
--project-root .` to regenerate `compiled/{en,ko}.json` + `key_registry.json`.

```gdscript
# in _ingest_snapshots(), within the existing settlement block:
var foods: Variant = sd.get("food_stocks", null)
var foods_arr: PackedInt32Array = PackedInt32Array()
if foods is PackedInt32Array:
    foods_arr = foods
if sids is PackedInt64Array:
    var sids_arr: PackedInt64Array = sids
    for i in sids_arr.size():
        var sid: int = int(sids_arr[i])
        cur_s[sid] = true
        if i >= foods_arr.size():
            continue
        var food: int = int(foods_arr[i])
        if not _last_notified_food.has(sid):
            _last_notified_food[sid] = food
            continue
        var delta: int = food - int(_last_notified_food[sid])
        if delta >= FEED_FOOD_DELTA_THRESHOLD:
            _push(_loc("HUD_FEED_STOCKPILE_DEPOSIT") % [sid, delta])
            _last_notified_food[sid] = food
        elif delta <= -FEED_FOOD_DELTA_THRESHOLD:
            _push(_loc("HUD_FEED_STOCKPILE_CONSUME") % [sid, -delta])
            _last_notified_food[sid] = food
```

`FEED_FOOD_DELTA_THRESHOLD` chosen value: **3**. `_loc(...)` returns a plain `String`, so `%`
positional formatting works. Args are `[id, amount]`, same order in en and ko.

**No data crosses the FFI boundary that 2-5a didn't already provide.** No sim mutation, no new
snapshot field.

## Section 4: Dispatch Plan

| # | Ticket | File/Concern | Language | Mode | Depends On |
|---|--------|-------------|----------|:----:|:----------:|
| T1 | Locale keys en+ko (fluent + ui json + recompile) | localization/ | data | DISPATCH | — |
| T2 | Feed lines from food_stock deltas | scripts/ui/panels/hud_status_panel.gd | GDScript | DISPATCH | T1 |

No Rust ticket (pure GDScript + locale data).

## Section 5: Localization Checklist

| Key | Source | en value | ko value |
|-----|--------|----------|----------|
| `HUD_FEED_STOCKPILE_DEPOSIT` | fluent/{en,ko}/messages.ftl + {en,ko}/ui.json | Settlement %d stored %d Food | 정착지 %d 식량 %d 적재 |
| `HUD_FEED_STOCKPILE_CONSUME` | fluent/{en,ko}/messages.ftl + {en,ko}/ui.json | Settlement %d used %d Food (famine) | 정착지 %d 기근 식량 %d 소비 |

- Both templates take args in the SAME positional order `[id, amount]`.
- Recompile via `tools/localization_compile.py`. Keys must resolve via Locale (not the literal key).
- a17 (registry↔compiled consistency, count-independent) holds across the +2 keys; a18
  (`localization/` lock released) holds. en/ko symmetric.

## Section 6: Verification & Harness

**Gate:** `cd rust && cargo test --workspace && cargo clippy --workspace -- -D warnings`
**Harness gate:** `cargo test -p sim-test harness_ -- --nocapture`

**No new simulation harness** — GDScript + locale only, no sim-core/systems/engine change.
- Regression baseline is EMPTY (post-c03ae2bc) → it stays empty; ANY failure blocks.
- a14/a15 (13-keys-present), a17 (registry consistency, +2 keys), a18 (localization released),
  en/ko symmetry → all PASS.
- **settlement regression** → STOP + report.
- Smoke: deposit past threshold → one deposit line; famine draw past threshold → one consume line;
  small per-tick changes aggregate; feed prunes as before; existing lines unaffected.
- GDScript: the two new visible strings route through the Locale autoload (gate-safe `_loc`).

## Governance chain
7daf28c1 → 2-5c
