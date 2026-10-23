# ART-POLISH2 decisions

## D1 — Owner feedback and bounded scope (2026-10-24)

Owner: "나무벽 질감 말고 나머지 검정 부분은 나무벽이랑 너무 이질적이다."
Owner: "너무 비슷한 얼굴이 많다."

Implement construction style × simulation material and six face variants with
matching sprite tints. Only `art/**` and `docs/v8/artpolish2/**`; no game code,
no merge, and no claim of owner approval. The task-specific branch instruction
overrides the general `v8/wip` convention: base `18ac947b45963ecb54e2c4a2b1b7692d13ecfafb`,
delivery branch `v8/art-polish2`. Unrelated worktree changes remain untouched.

## D2 — Material tops supersede the dark-cap default

The previous ×0.45 dark caps remain owner comparison (a). Proposed (b) uses
material tops with `top.value = 0.75`, saturation 0.85, blur 2 px, exposed ink
edges 2 px, width 64 px and no side shadow. Proposed (c) centres width 44 px
with a soft side shadow. Brightness survives as a parameter (`top_value` in
handoff terminology, `top.value` in the schema), never a material-specific
branch. The owner will choose a/b/c; no choice is inferred here.

## D3 — Authoritative look counts and preserved fallbacks

The goal's §3.3 says 17 adult looks, but the adopted `route_i/MANIFEST.json`
has 16 accepted adult and 18 accepted child looks. The goal's 204-face total
matches these 34 unique sources × six variants. Produce all 204 unique
variants and expose 216 logical slots through the existing two fallbacks:
`adult_M_fur_cloak_long_tied` → `adult_M_fur_cloak_bun`, and
`adult_M_hide_wrap_long_tied` → `adult_M_hide_wrap_bun`. No missing source is
invented or promoted. This preserves the current adoption decision.

## D4 — Execution and evidence boundaries

Raw generations, reviewer packets/keys, logs, and measurements live under
`/Users/rexxa/work/artpolish2-data/`. Committed deliverables are each ≤1.5 MB.
Generation ceiling is 320 calls including failed calls and scorecards; wall
assets get at most three tries and portraits at most two. Blind-test failure
allows one regeneration round followed by two new reviewers. Original failed
scores remain in the report. General end-of-pass edits outside the permitted
paths are superseded by this pass's explicit scope.

## D5 — Implementation plan and commit separation

The supplied goal is an approved execution contract. Plan/approval skill
defaults do not introduce another permission handoff. Generic schema,
resolver, rendering, and tests are committed before the new-era demonstration.
The later commit that introduces `mud_brick` includes only data, generated
assets, and documentation; an executable test inspects that commit's actual
file list and rejects every `.py` path. No tests are weakened to make a visual
gate pass.
