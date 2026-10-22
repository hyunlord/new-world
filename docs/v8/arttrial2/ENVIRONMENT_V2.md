# TRIAL2b environment acceptance and accounting

Status: **Final style join:30/38 first attempts,35/38 after retries;35 promoted.** Fresh raw model reviews pass38/38; retry reviews pass5/5. `ENV_ACCEPTANCE_V2.json` records `promoted=true`; all35published PNGs and complete provenance sidecars have been hash-verified. Semantic QA is separately PASS. This is style acceptance, not automatic seam/readability/game acceptance.

## Reproducible acceptance

`art/pipeline2/accept_v2.py` keeps preregistered style acceptance (numeric AND model) separate from consumer-promotion eligibility:

- Fresh `ENV_MODEL_REVIEW_V2.json`: raw candidate identity, exactly two distinct
  same-family reference hashes matching frozen calibration, actual reference bytes,
  scorecard bytes/hash, usable transcription, all four scores≥4. Legacy reviews
  and candidate/reference identity mismatches fail closed.
- Semantic QA JSON `records`: `raw_sha256`, boolean `passed`, `evidence` path and
  `evidence_sha256`. Missing subject/parts, defective extraction or other content
  failures can withhold consumer promotion, but never alter C1/C2 style acceptance. Missing semantic evidence withholds promotion only. Repeat `--semantic-qa` to join original and retry manifests.
- Optional retry candidate JSON `records`: `name`, historical `attempt`2 or3,
  `raw`, `raw_sha256`, `working`, `working_sha256`. Attempt numbers never reset.
  First38 candidate/working identities come from the frozen re-score evidence.

Style acceptance and C1/C2 use numeric AND model only, exactly as preregistered.
Semantic PASS is an additional consumer-promotion condition, recorded separately as
`promotion_eligible`; prompt-only counts or details not required by the actual goal
are not new acceptance criteria.

The command re-runs numeric checks using frozen calibration, verifies complete
passingLOO and re-runs all three person-family negative controls. It first writes
a reviewable report; `--promote` explicitly enables copying accepted working
sprites into `art/style05/env/<family>/`. Every copied byte must match the recorded
working SHA and1.5MB cap. Existing different accepted files and sidecars are archived
under `env/history/trial2/<family>/<name>-<hash>.*` before replacement, preserving
the two historical ground acceptances. New sidecars say exactly
“accepted under judge v2, TRIAL2b”. The lowest accepted historical attempt wins.

Example (paths are required evidence inputs, not yet-created completion evidence):

```bash
uv run --with pillow --with numpy --with scikit-image python -m art.pipeline2.accept_v2 \
  --reviews docs/v8/arttrial2/ENV_MODEL_REVIEW_V2.json \
  --semantic-qa /Users/rexxa/work/arttrial2-data/trial2b/semantic-original.json \
  --semantic-qa /Users/rexxa/work/arttrial2-data/trial2b/env-retries/semantic_qa.json \
  --retry-candidates /Users/rexxa/work/arttrial2-data/trial2b/retry-working-manifest.json \
  --output docs/v8/arttrial2/ENV_ACCEPTANCE_V2.json
```

Append `--promote` only after inspecting the dry-run evidence. Generation records
must be closed: every new retry raw has a generation sidecar, including failures.
The script refuses to report a cost numerator from an incomplete retry ledger.

## Denominators and costs

C2 denominator is always38 distinct original assets. First-pass numerator only
counts numeric+model accepted attempt1; after-retry numerator counts each style-accepted asset once.
Neither numerator depends on semantic eligibility or consumer-promotion count.
These remain separate from final calm-ground replacement acceptance.

Actual historical environment cost is **59 generation attempts**:
ground12 + plants18 + buildings12 + items10 + misc7. Relative to38 first attempts,
this is **21 extra historical generations**, not20. A counterfactual avoided-retry
estimate must identify which specific attempts would have been avoided; it never
reduces actual C1 cost. The original failed and unnecessary generations remain in
actual accounting.

The report emits both:

1. Environment re-score C1 = `(59 + all new environment retry generations) /
   numeric+model accepted distinct environment assets`.
2. Final delivered-portfolio C1 including calm-ground replacement =
   `(59 + new environment retry generations + all calm-ground generations) /
   numeric+model accepted distinct environment assets`.

Use the second when presenting a portfolio that includes the six regenerated calm
grounds. Calm generation counts include rejected selections and are not reset to
one per selected texture. A zero accepted denominator yields `null`, never zero or
an artificially favorable estimate. Model-review calls are recorded separately
from asset-generation tries. Billed cost is unavailable unless exposed by tools.

## Verification at scaffold checkpoint

Three tests pass: wrong raw/reference/card provenance rejection; numeric/model
style acceptance independent of semantic promotion eligibility; and real temporary-filesystem promotion
preserving historical sprite bytes/sidecars. CLI help executes. Ruff and diff check
pass. Basedpyright reports0 errors/97 warnings for the join script; strict typecheck
is NOT_GREEN. No production files were promoted by these tests.


## Final measured result

| Production family | Original assets | First numeric AND fresh model | After retries |
|---|---:|---:|---:|
| ground | 6 | 6 | 6 |
| plants | 9 | 9 | 9 |
| buildings | 6 | 5 | 5 |
| items | 10 | 4 | 9 |
| misc | 7 | 6 | 6 |

C2:30/38=78.95%,35/38=92.11%, both bars PASS. C1 actual70/35=2.000; including12calm-ground calls82/35=2.343, both below2.5. The denominator is35distinct original environment identities, never41by adding six replacements. No historical failed generation is subtracted.

Unaccepted after the shared three-attempt ceiling: drying_rack, firewood_bundle, deadwood. All38first model results passed; numeric failures alone drive these rejections. Five accepted retries:flint_pile2,log_pile_large2,log_pile_small2,stone_axe2,hide3.

The historical first-ground working images are the style-rescore cohort; old seam/busyness problems remain. Calmer replacements live separately in `ground_v2/` and are separately evaluated. Acceptance here does not erase ground seams, approve a whole runtime scene, or establish the model judge's specificity. The plant-reference leakage sensitivity is28/36first style passes (77.78%) after excluding oak_large/birch_large from evaluation only; the required38denominator is unchanged.

## Full accepted-sidecar provenance

Promotion now builds every sidecar before copying any asset. The helper
`accept_provenance_v2.py` preserves the original generation and processing records
in full, while separating their historical statuses/scores from the current v2
verdict. Top-level fields include exact prompt, generation reference paths/hashes,
try count, recorded generation ID or explicit null/reason, original identity,
actual working dimensions, ordered operations/parameters, recorded placement
coordinates, generation timestamps/wall time, all eight numeric checks and family
thresholds, calibration identity, two actual model-reference hashes, all scores,
verbatim reason, full model context/timestamps and acceptance evidence identity.

Original candidates use the adjacent working sidecar plus raw-generation sidecar.
Retries can explicitly pin `provenance` and `provenance_sha256` in their working
manifest. Missing/changed processing provenance, prompt/operations, generation
reference identities or full model context prevents promotion. Unknown values stay
null with an explanation; generation times are never inferred from model-review
times. Retry generation UTC timestamps were not recorded and remain explicitly
null while recorded wall seconds survive.

Thirty numeric-passing original candidates were joined with their actual fresh
model records and passed in-memory full-sidecar construction. No asset was copied;
external evidence is `trial2b/sidecar-provenance-preflight.json`. Four tests pass,
including rejection of missing/changed processing provenance; Ruff and diff checks
pass. Calibration feature values and hashes are unchanged by this extension.
The provenance extension uses shared immutable types in `accept_types_v2.py` to
avoid import cycles. Its final three-file typecheck has0 errors/152 warnings;
strict typing remains NOT_GREEN. Promotion prepares every full sidecar before any
working-file copy, so missing required provenance cannot cause partial promotion.
