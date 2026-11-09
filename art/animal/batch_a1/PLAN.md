# A1 finite production plan — 2026-11-10

Status: **ART CANDIDATE DELIVERY COMPLETE**. The original scope below is fulfilled:
184 required + 12 optional new frames accepted, with 20 unchanged reference frames
in the 216-frame delivery. See `docs/v8/artbatch_a1/production_summary.json` and
`docs/v8/artbatch_a1/animal_manifest.json`; owner approval and game import remain pending.

Scope: the A1 animal family from `/Users/rexxa/work/art-batch/BRIEF.md`, content
bible [#690](https://github.com/hyunlord/new-world/issues/690), and C2 §3.5 / issue
[#695](https://github.com/hyunlord/new-world/issues/695). The bible's climate table
is §4.5; §4.6 is its quantity summary. Source snapshot:
`/Users/rexxa/work/art-batch/A1/sources/issue-690.json`.

## Required matrix

All rows contain **stand, walk, run, graze, dead × S/E**. Rows marked attack add
**attack × S/E**. Each pose-direction entry is one key pose, not a claim of a
multi-frame animation cycle. `catalog.json` is the machine-readable plan.

| Order | Set | Content era / role | Attack | Frames |
|---:|---|---|---|---:|
| 1 | cattle | Bronze livestock | No | 10 |
| 2 | aurochs | Wild source for bronze cattle | Yes | 12 |
| 3 | sheep_fleeced | Neolithic sheep; bronze wool-chain priority | Yes, wild source role | 12 |
| 4 | sheep_shorn | Sheep wool-production variant | No | 10 |
| 5 | calf | Bronze cattle young | No | 10 |
| 6 | goat | Neolithic livestock and wild source | Yes | 12 |
| 7 | kid | Neolithic goat young | No | 10 |
| 8 | lamb | Neolithic sheep young | No | 10 |
| 9 | wild_boar | Wild source for neolithic pig | Yes | 12 |
| 10 | pig | Neolithic livestock | No | 10 |
| 11 | piglet | Neolithic capture/breeding young | No | 10 |
| 12 | reindeer | Neolithic harsh-climate livestock and wild source | Yes | 12 |
| 13 | dog | Neolithic livestock; wolf is existing source/reference | No | 10 |
| 14 | chicken | Bronze livestock from climate catalogue | No | 10 |
| 15 | junglefowl | Wild source for bronze chicken | Yes | 12 |
| 16 | donkey | Bronze cart-use candidate; origin unspecified | No | 10 |
| 17 | horse | Iron livestock and wild source | Yes | 12 |
| | **Required total** | **17 sets** | **7 attack sets** | **184** |

Bronze cattle and wool-chain art come first, then the requested neolithic gaps and
young. Chicken/junglefowl close the bible's bronze animal omission before donkey
and iron horse. The explicit brief list accounts for 14 sets/152 frames; piglet
and chicken/junglefowl add 3 sets/32 frames. Piglet is supported by §4.2 capture of
goat/pig young. Chicken and its wild source appear in §4.5.

Goat, fleeced sheep, reindeer and horse may share body art between wild-source and
domestic roles. Wild/penned status is a separate marker. The catalogue records
these roles without inventing runtime IDs. Aurochs/cattle and boar/pig remain
distinct shapes. Existing deer/wolf are comparators, not new generation targets.

After all required rows pass, generate **winter reindeer** as an optional 12-frame
coat variant with unchanged size/camera/anchors. The bible requires reindeer in
harsh climates but does not require seasonal sprites; this extension must remain
labelled optional. No animal count-bucket, filled/empty or depleted variants are
specified. Bees/hives belong to A2's structure/resource work; the inert falconry
node does not specify an additional bird species for this animal batch.

## Production sequence

1. Freeze the style and proposed sizes in `STYLE.md` and `catalog.json`. Record
   actual attached references and their hashes, exact prompt and tool response.
2. Generate complete coherent pose sheets, preserving each raw attempt outside
   the repository. Reject anatomy/style failures; record reason before retry.
3. Measure S-stand landmarks. Normalize by existing whole-figure rules; write
   per-frame canvas, anchor, measured alpha footprint, source and hash sidecars.
4. Run structural checks, review native/2× contact sheets against the references
   and family, and persist visual verdicts. Update actual adoption separately from
   planned scope. Never mark an unreviewed or structurally failed frame adopted.
5. Commit accepted assets/manifests/contact sheets in small batches on
   `v8/art-batch-a1`. Explicit paths only. Push only this branch about every two
   hours and at completion; compare remote SHA to local HEAD.
6. Write Korean `/Users/rexxa/work/art-batch/A1/REPORT.md` with generated/adopted/
   rejected counts by set, fixed style rules, evidence sheets, branch/SHA and
   draft-only `art_import.json` / `bindings.json` import rows for C2/C3.

Write progress sheets to `/Users/rexxa/work/art-batch/A1/sheets/<HHMM>_<what>.png`
every 1–2 hours. Keep raw outputs, rejected candidates and logs under that external
evidence root. Repository assets, sidecars and curated sheets stay in the animal
art layout; no game import, `rust/` or `scripts/` edits occur in this pass.

## Verification and honest limits

Completion requires every required pose-direction entry to have a reviewed asset,
its provenance, structural result and explicit adoption disposition. Rejecting
an attempt does not satisfy that entry. The family comparison must cover the
whole 184-frame set, not just attractive stand frames. Review young/adult scale,
fleeced/shorn identity, wild attacks and dead poses explicitly.

Separate the existing animal references' historical approval state, this pass's
candidate/adoption decisions, and C2/C3 runtime integration. Record unavailable
metadata as unavailable. No runtime, gameplay, determinism or owner-approval claim
is established by art-only checks.
