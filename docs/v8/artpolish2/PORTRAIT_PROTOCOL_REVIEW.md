# ART-POLISH2 portrait protocol review

Reviewed 2026-10-24 KST; evidence snapshot ended 2026-10-23 18:10:25 UTC.
Scope: read-only review of the portrait preparation, processing, packet and
scoring pipeline. Only this report was changed. No production portrait image
was viewed, no blind reviewer role was performed, and no owner approval is
claimed. All reproduction images were synthetic temporary fixtures.

**Initial disposition: CHANGES_REQUIRED for evidence integrity.** The numeric
thresholds, current fixed sample and ordinary shuffle/response paths are
correct, but the initial pipeline cannot establish immutable cross-phase
candidate identity, fresh retry reviewers, or preservation of prior scores and
observations. These findings were handed to the portrait-pipeline executor;
any later correction requires a separate verification entry below. They are
not retrospective claims about misconduct or a production blind result.

## Findings

### P1 — Matching can use portraits different from those described

- Location: `art/pipeline_polish2/blind_matching.py:54`, `:59`, `:70`, `:75`;
  `art/pipeline_polish2/blind_models.py:57`.
- The builder verifies the old description **panel** hash, then loads the
  current `candidates.json` and resolves the semantic variant keys to current
  portrait references. The private key does not bind a frozen candidate-set
  or face-plan reference. A normal candidate update between the two phases
  therefore changes the pictures without invalidating the descriptions.
- Reproduction: create synthetic A/B packets, save B's six descriptions for
  each row, replace the source portrait and update its valid `AssetRef` in
  `candidates.json`, then call `build_matching(base, a, b_descriptions)`.
  All eight matching rows were accepted. The old description panel's portrait
  pixel was `[170,136,102]`; the matching panel's was `[32,32,32]`. Every old
  description-panel hash still verified.
- Source hashes changed from
  `870145bf456b0c62bb5804e9831f386b1d014ae5a7b19f2e488d6cdbb0e91a0c` to
  `13a171fc38f58f1e70f21f0a9ab0adf143c0a5db23c2d618eb3e7d98feeaeba7`.
- Required closure: bind the exact candidate sources and sample plan for the
  entire round, including both reviewers, description, matching and scoring;
  reject a changed source instead of silently following the new selection.

### P1 — Two NEW reviewers in round1 are not evidenced

- Location: `art/pipeline_polish2/blind_models.py:15`, `:57`;
  `art/pipeline_polish2/blind_packets.py:121`;
  `art/pipeline_polish2/blind_score.py:111`.
- The model records only role `A`/`B`, round and random seed. There is no actual
  agent/session/person identity, no distinctness check between the two roles,
  and no comparison with round0. A different seed demonstrates a new shuffle,
  not a new reviewer. `score_reviewer()` also does not compare the key's round
  to the requested scoring round.
- Required closure: preserve reviewer/session evidence; require two distinct
  identities per round and identities disjoint from round0 for round1. The
  execution record must establish that those reviewers received only their
  assigned packets, with no inherited production context. A/B labels alone
  cannot establish this claim. No real retry review had occurred at snapshot.

### P1 — Reprocessing can erase a prior structural failure

- Location: `art/pipeline_polish2/portrait_process.py:76`, `:90`, `:91`;
  `art/pipeline_polish2/portrait_models.py:104`.
- Different raw bytes under an existing attempt are correctly rejected, but
  rerunning the same raw with a different inspection overwrites
  `tryN/processed.json` and `selected.json`. The original inspection file is
  neither copied nor hash-bound into `Processed` (`portrait_process.py:32`).
- Reproduction: process synthetic attempt1 with an uncropped-head observation
  set to false and a zero margin, obtaining `FAIL`; replace only the inspection
  with an in-canvas head and rerun the same raw/attempt. The same historical
  `processed.json` was overwritten with `PASS`. Its output contains no
  inspection source reference. The raw bytes remain unchanged, so the existing
  historical-raw overwrite test does not cover this case.
- The geometry summary also drops the supplied `hair_match`, `top_match`,
  `extra_accessory` and `identity_note` observations. These are not additional
  numeric geometry gates, but their original evidence should remain available.
- Required closure: preserve the original observation bytes and their hash,
  and reject conflicting writes or retain explicit immutable revisions with
  the old failed result. A pending observation may be completed without
  deleting any previous observed result.

### P1 — Same-round scoring can replace previous failures

- Location: `art/pipeline_polish2/blind_score.py:120`, `:121`;
  `art/pipeline_polish2/portrait_models.py:104`.
- Static evidence: the scorer unconditionally writes both `scores.json` and
  `blind_scores_roundN.json`; `write_evidence()` overwrites existing files.
  Response references hash the supplied file at scoring time, but the scorer
  does not preserve a copy or reject a different submission at the same path.
  A rerun with changed answers replaces the old result and its original hash.
- Required closure: seal submitted response bytes and preserve the first
  scored result, or use an append-only revision record that retains failures.
  Round0 and round1 filenames are separate, which prevents cross-round
  replacement but does not protect repeated runs within one round.

## Checks that passed

| Requirement | Verified evidence |
|---|---|
| Fixed eight looks | Current `FACE_VARIANTS.json` equals the first two lexical accepted looks in each adult-F/adult-M/child-F/child-M cohort. `blind_sample.py:25` checks eight unique looks and cohort counts. |
| Fixed 36 identity cases | Current list equals all six variants of owner-sample indices `0,1,2,4,6,7`, exactly 36 unique cases. Covered skins S1–S3, hairs H1–H4, adult/child/older/young, clean/full/short/none beard values, and all six named feature classes. |
| Two description directions | `blind_matching.py:96` sends B descriptions to A and A descriptions to B; `:51` rejects self matching. Synthetic description-transfer test passed. |
| Independent shuffles | Separate per-reviewer seeds (`blind_packets.py:128`); shuffled row and option orders (`:83`, `:88`, `:94`, `:99`); matching independently shuffles picture order and line order (`blind_matching.py:71`, `:73`). |
| Exact image scale | Independent patterned synthetic image comparison found all four 80×112 sprites equal to their exact 160×224 nearest-neighbour image, and the portrait equal to its native 128×128 pixels. Panel was 900×276 with no clipping. |
| No ordinary packet answer leaks | Existing real-file synthetic packet test verifies opaque names and empty PNG metadata. Production builders create new panels and expose instructions/images only; keys are under separate `private/`. Matching line text is reviewer-supplied. Actual reviewer confinement still requires execution evidence. |
| Every look at least 5/6 | `blind_score.py:89`–`:90` allows exactly one miss, rejects 4/6, and `:107` requires all eight looks. Existing tests confirm 46/48 aggregate cannot rescue a 4/6 look. Repeated image labels permit a true 5/6 result. |
| Every reviewer at least 32/36 | `blind_score.py:105` permits at most four misses; `:118` ANDs both reviewers' identity and distinctness results. Five misses are retained by the test. |
| Response completeness | Duplicate/missing question IDs and missing line IDs are rejected; typed literals reject invalid labels. Extra model fields are forbidden. |
| Raw preservation | Existing processing tests confirm byte-preserved raw, exact full-frame resize, pending structure without observations, changed-reference rejection and different-raw rejection within one attempt. |

## Processing geometry and evidence paths

`portrait_process.py:19` imports the unchanged ENV1/POLISH `geometry()`.
That instrument checks all face-box corners against the ellipse centred at
`(64,61)` with radii `(44,52)`, an observed uncropped head, and a minimum 2px
canvas margin (`art/pipeline_env1/portrait_finalize.py:95`). It uses actual
supplied boxes, not prompt bounds. Without inspection, status remains
`PENDING_OBSERVATION`. Inspection sheets put the native image above a 2×
coordinate-labelled image and ellipse (`portrait_inspect.py:30`–`:37`).
Sheet and inspection provenance is not yet sealed by the initial implementation.

Expected external paths under `/Users/rexxa/work/artpolish2-data/portraits/`:

- `generation/<look>/<variant>/request.json`: prompt and input references.
- `generation/<look>/<variant>/tryN/{raw.png,canonical.png,processed.json}`:
  attempt outputs; `selected.json` selects a current candidate.
- `candidates.json`: selected candidate set; `inspection/packets/`: observation
  sheets. The inspection JSON path is supplied by the caller, not standardized.
- `blind/roundN/private/{A,B}.json`: keys;
  `blind/roundN/packets/{A,B}/{describe,identity,match}/`: reviewer packets;
  `blind/roundN/scores.json`: score output.

Metadata-only inventory during review: **204 requests, 0 selected records,
0 processed records, no `candidates.json`, no production inspection JSON and no
production blind keys/scores**. Existing `pipeline_validation/` files are
synthetic tests and are excluded. Generation and review may progress after
this snapshot. No production geometry, distinctness or identity result is
claimed by this report.

## Commands and reproduction evidence

System Python ran the eight packet/scoring tests successfully. Adding the
processing module initially failed collection because system Python lacks
`skimage`. The complete suite then ran in an already-cached environment; no
dependencies were added:

```sh
/Users/rexxa/.cache/uv/archive-v0/m3Qiivy-EEbMpbyIKj-mJ/bin/python -m pytest art/pipeline_polish2/test_blind_packets.py art/pipeline_polish2/test_blind_score.py art/pipeline_polish2/test_portrait_process.py -q
```

Result: **11 passed in 6.27s**. This demonstrates existing test behavior,
including the gaps reproduced above, rather than production acceptance.

Bounded synthetic reproduction results are retained externally in
`/Users/rexxa/work/artpolish2-data/portraits/protocol_review/`:

- `candidate_mutation_reproduction.json`
- `inspection_overwrite_reproduction.json`
- `exact2x_reproduction.json`

## Later verification

Pending executor fixes and an independent recheck. Original findings and
failed values above must remain intact when recording their disposition.

### Recheck 1 — 2026-10-24 KST, 18:21 UTC snapshot

**Disposition: CHANGES_REQUIRED; one remaining P1 in reviewer aggregation.**
No production portrait content was viewed. All 22 current protocol tests
passed in 9.21s using the same cached Python environment, adding
`art/pipeline_polish2/test_blind_round.py` to the command above.

| Initial finding | Recheck disposition and evidence |
|---|---|
| Candidate mutation between phases | **CLOSED for the original reproduction.** `blind_round.py:62` copies candidate/plan/image bytes into private immutable evidence; `:84` verifies original and frozen hashes. Both matching keys must share the manifest (`blind_matching.py:55`) and load it before rendering (`:58`). The mutation regression now rejects an updated source image plus internally valid replacement index; frozen-copy corruption also rejects. |
| Two NEW reviewers | **PARTIALLY CLOSED; remaining P1 below.** `blind_models.py:26` records canonical agent task, optional session UUID, isolation declaration and hash-bound spawn receipt. The roster rejects duplicate roles/tasks/sessions (`:48`); `blind_round.py:40` rejects round1 task/session reuse and requires round0 evidence. Tests confirm two new canonical tasks work without inventing UUIDs, reused canonical tasks or sessions reject, unknown response tasks reject, and a relabelled round rejects. Actual isolation and receipt authenticity still require the recorded launch evidence. |
| Structural-failure replacement | **CLOSED for the original reproduction.** `portrait_process.py:82` preserves original observation bytes; `:95` retains full observation plus its AssetRef; `:97` records distinct immutable revisions. The real-processing synthetic regression retains `processed.json` as FAIL and the changed observation as `processed.revision002.json` with PASS, preserving the original hair/top/accessory observations and inspection hash. `selected_revision.json` points to the current record. |
| Same-round failure replacement | **CLOSED for the original overwrite path.** `blind_score.py:104`–`:108` preserves key and response bytes; `:136`–`:138` records immutable score revisions plus a latest pointer. An additional independent end-to-end synthetic run scored A=31/36 as FAIL, changed A's source response to 36/36, rescored, and preserved the original score and original response bytes while writing `scores.revision002.json`. This preserves history; it does not authorize a revised response as a new blind trial. |

#### Remaining P1 — The aggregate scorer can count A twice

- Location: `art/pipeline_polish2/blind_score.py:124`–`:135`.
- The scorer checks each key against the roster, then checks requested round
  and shared manifest, but never requires the **scored pair** to contain A and
  B exactly once. A valid A key stored at `private/B.json` remains a valid A
  assignment when loaded independently.
- Independent reproduction: build a valid synthetic round with distinct A/B
  roster, build A's matching packet from B's descriptions, place that valid A
  key in both `private/A.json` and `private/B.json`, and invoke
  `main(A_identity, A_identity, A_match, A_match)`. Actual result:
  `passed=true`, reviewer labels `["A","A"]`, agent tasks
  `["/synthetic/A","/synthetic/A"]`.
- Required closure: enforce A and B exactly once in the scored pair and bind
  each input slot to its expected role; keep distinct actual agent/session
  evidence and the same frozen round. Add an end-to-end aggregate scorer
  regression rather than testing only individual `score_reviewer()` calls.
- Evidence: external `protocol_review/recheck_duplicate_reviewer_reproduction.json`.
  The separate score-history reproduction is
  `protocol_review/recheck_score_history.json` under the same portrait data root.

The root and executor were notified immediately. This recheck does not claim
production structure, blind success, round1 eligibility or owner acceptance.

### Recheck 2 — 2026-10-24 KST, 18:25 UTC snapshot

**Final bounded protocol disposition: ALL FOUR FINDINGS CLOSED.** The later
duplicate-A aggregation finding is also **CLOSED**. This disposition applies
to the reviewed pipeline and synthetic integrity reproductions, not to the
quality or acceptance of any production portraits.

`blind_score.py:132` now requires the scored slots to be exactly `(A, B)`;
`:135`–`:142` verifies the shared frozen manifest, compares each complete agent
record with its assigned roster entry, and requires distinct canonical tasks
and distinct supplied session IDs before publishing a result.

Independent rerun of the original exploit (valid distinct roster, valid A key
copied to B's key path, A's exact identity/matching response paths supplied in
both slots) produced:

```text
ValueError: Aggregate scoring requires exactly A and B in their assigned slots
scores.json published: false
```

The corresponding positive control used the actual four separate synthetic
response paths `A_identity.json`, `B_identity.json`, `A_matching.json` and
`B_matching.json`. It produced `passed=true`, roles `[A,B]`, canonical tasks
`[/synthetic/A,/synthetic/B]`, both session IDs `null`, identity counts
`[36,36]`, and every distinctness look at 6/6. This confirms the fix supports
hash-bound canonical agent/launch evidence when a separate UUID is unavailable.
The round1 disjointness checks verified in Recheck 1 remain unchanged.

Fresh full scoped verification: **24 passed in 6.11s** with the four test modules
listed in Recheck 1. Independent negative/positive results are retained at
`/Users/rexxa/work/artpolish2-data/portraits/protocol_review/final_aggregation_recheck.json`.
No code or production evidence was edited by this reviewer, and no production
portrait image was viewed. All initial failed values and intermediate findings
remain above. Actual reviewer launch isolation, production geometry, both
production blind gates and owner choice still require their own evidence.

### Source-appearance/request repair review — 2026-10-24 KST, 18:51 UTC

**Disposition for this subsequent change: CHANGES_REQUIRED for one stale-envelope
guard gap.** The five earlier blind/evidence findings remain closed; this is a
separate bounded preparation/provenance review. No generation, code changes,
staging or production portrait-content review was performed.

#### P1 — Preparation and processing accept a changed source-observation envelope

- Locations: `art/pipeline_polish2/portrait_appearance.py:79`–`:85` and
  `art/pipeline_polish2/portrait_process.py:60`–`:66`.
- `compile_appearance()` records hashes for the source-envelope files, but
  `appearance_for()` verifies only the original sprite reference. Processing
  verifies the compiled appearance manifest and original sprite, but does not
  validate the manifest's envelope references. Thus changing a source-review
  envelope after compilation can silently leave its old brief in a new request.
- Independent synthetic reproduction: compile a valid appearance manifest,
  replace the envelope's `appearance_brief` with a different synthetic
  observation, then run actual `prepare()` and `process()`. Preparation accepted
  one request with the old brief; the updated brief was absent. Processing
  published its synthetic candidate with `PENDING_OBSERVATION`, despite the
  envelope hash mismatch.
- Required closure: verify all linked source-envelope hashes when loading
  appearance evidence for preparation and again before processing. Recompiling
  a deliberately updated provisional manifest is legitimate; silently accepting
  a changed envelope behind an already compiled manifest is the missing guard.
- Evidence: external
  `protocol_review/changed_appearance_envelope_reproduction.json`.

The current workspace manifest has stale hashes for both source-envelope files,
but those files are intentionally still under revision and no new generation
request has been dispatched from this provisional repair. **This is not a
finding of production corruption.** All 34 current compiled appearance briefs
still equal their current envelope briefs. The root confirmed that final
compilation/request preparation waits for the source-review freeze. The stale
provisional metadata demonstrates why the API guard must exist before use.

#### Verified preservation and requested boundaries

| Check | Fresh result |
|---|---|
| Original 30 requests | **30/30 byte-identical** to their `try1_inputs/request.json` snapshots. |
| Original 30 raws | **30/30 byte-identical** between `try1.png` and preserved `try1/raw.png`. No image content was viewed. |
| Source coverage and original hashes | Current appearance manifest covers exactly all **34 accepted looks**; all 34 references point to each look's original `S/idle.png`, and all 34 hashes verify. Importer rejects a changed original source and requires exact plan coverage (`portrait_appearance.py:56`). |
| Exact third reference | Independent per-pixel comparison checked every synthetic native RGBA pixel against all 16 pixels in its 4×4 block. Dimensions, mode and alpha all match. No resize call was used to compute that expected comparison. |
| Ordered references | `build_request()` keeps native tinted sprite first, style anchor second, exact enlargement third (`portrait_prepare.py:73`). `Candidate.sprite` still uses reference0 (`portrait_process.py:99`). |
| Legacy two references | Existing legacy JSON parses as attempt1 with no fabricated appearance metadata; processing tests still accept the two-reference path with valid original hashes. |
| Separate immutable retry | Independent end-to-end preparation retained original request/raw bytes, emitted `try2.request.json` and a single `try2.queue.json` object, and accepted an identical repeat. Conflicting retry preparation rejected without replacing the first retry request. |
| Explicit attempt2 and paths | Retry request, queue and processed candidate all identify attempt2. Queue `request_path` points exactly to `try2.request.json`; current corrected native reference, previous request hash and original face-plan hash are bound. |
| Limits and mismatch rejection | Preparing a retry from an attempt2 request, conflicting retry contents, processing retry2 as attempt1, and constructing processing attempt3 all rejected in the independent check. Existing tests also reject processing a first-attempt request as attempt2. |
| Mutable compiled manifest/original sprite | Current processing tests reject either mutation before publishing a candidate. The remaining gap is the linked source-envelope hash described above. |

Fresh scoped suite: **32 passed in 6.18s**, adding
`art/pipeline_polish2/test_portrait_prepare.py` to the previous four modules.
The suite covers the requested preparation, preservation, retry and processing
paths as well as all previous blind-scoring regressions. No dependencies were
added.

Additional external evidence under
`/Users/rexxa/work/artpolish2-data/portraits/protocol_review/`:

- `source_appearance_original_preservation.json`: all 30 key/hash comparisons.
- `current_appearance_provenance.json`: exact coverage, 34 source checks and
  provisional envelope-hash differences.
- `source_appearance_scale_retry_checks.json`: independent pixel-block, retry,
  immutable-input and attempt-rejection results.

The root was notified of the missing guard. Production appearance quality,
geometry, blind results and owner choice remain outside this review's claims.

### Linked-envelope guard recheck — 2026-10-24 KST, 18:55 UTC

**Disposition: linked-envelope P1 CLOSED.** All earlier blind/evidence findings
remain closed. Original failure evidence and provisional-state caveats remain
above; the production appearance manifest was not recompiled or otherwise
changed by this review.

`portrait_appearance.py:79`–`:85` now verifies the compiled manifest and every
linked envelope reference. `appearance_for()` uses that shared guard before
request preparation (`:90`); processing uses the same guard before creating
an attempt directory or writing a candidate (`portrait_process.py:65`).

Independent reruns of the edited-brief reproduction, using separate synthetic
workspaces for each path, returned:

- **Preparation:** changed source-envelope brief rejected with `ValueError`;
  zero request files written.
- **Processing:** prepare a valid request, then change its envelope brief;
  rejected with `ValueError`, no candidate published and no attempt directory
  created.
- **Valid control:** unchanged manifest/envelope prepared and processed
  successfully with the source brief, three references,
  `structure_status=PENDING_OBSERVATION` and `owner_adopted=false`.

Fresh original-evidence recheck: all **30/30 requests and 30/30 raws** still
match both their preserved snapshots and the hashes recorded by the preceding
review. Scoped verification: **36 tests passed**, including four regressions
covering each linked envelope independently in preparation and processing.
No dependencies were added and no production candidate content was viewed.

Independent evidence under external `portraits/protocol_review/`:

- `linked_envelope_final_recheck.json`
- `linked_envelope_originals_recheck.json`

The intentionally provisional production manifest still requires compilation
after the source-review freeze. This review establishes the rejection guard
and valid synthetic path, not source-review readiness or portrait acceptance.

### Fourth-reference and optional retry review — 2026-10-24 KST

**Disposition: PASS for the bounded tooling review; no remaining blocker found.**
All earlier findings and their original failure evidence remain preserved above.
This review used source inspection, existing regression tests and temporary
synthetic fixtures. It did not inspect production portrait content or prepare,
generate, process or replace any production request or asset.

| Required boundary | Independently verified evidence |
|---|---|
| Native crop geometry | `portrait_appearance.py:109` computes the nonzero-alpha bbox, keeps its full width and takes exactly `ceil(3 * visible_height / 4)` rows from its top. Independent fixtures covered every visible height from 1 through 12, using a separately calculated expected row count. All 12 cases passed. |
| Exact RGBA replication | Every output pixel in those fixtures matched the corresponding native pixel indexed by integer division by 12. Dimensions, RGBA mode, partial alpha and RGB behind zero alpha were retained; native bytes were unchanged. |
| Bound provenance | `BustDetail` (`portrait_models.py:70`) records source/output hashes, visible/crop bboxes, literal scale12 and half-open coordinate rules. The request and tool queue bind the sidecar hash. `verify_bust_detail()` (`portrait_appearance.py:125`) verifies both files and recomputes their exact source-derived geometry and RGBA bytes. |
| Tamper rejection before publication | Fresh tests rejected a stale source, painted output with refreshed hashes, altered visible/crop bbox and altered scale. Preparation validates an existing detail before rebuilding even a missing third reference (`portrait_prepare.py:98`; regression `test_portrait_prepare.py:181`). Processing checks before attempt/raw/canonical/result writes (`portrait_process.py:72`); its regression preserves a prior processed FAIL and all its evidence bytes. |
| UNSTARTED preparation | Automatic detail inclusion checks `not started(raw_directory)` (`portrait_prepare.py:96`). The unstarted-only regression keeps the started request byte-identical and queues only the unstarted variant (`test_portrait_prepare.py:118`). Upgrading an unstarted request preserves all first-three references and their bytes (`:166`). |
| Existing two/three references | Both historical request shapes still parse without synthetic detail metadata. Independent integration fixtures exercised each legacy shape through default and explicitly opted-in attempt2 preparation, JSON parsing and actual synthetic processing. |
| Explicit eligible attempt2 | `prepare_retry(..., include_bust_detail=True)` adds the fourth reference; the default remains three (`portrait_prepare.py:166`). CLI help exposes the option. Both legacy shapes passed both paths; all resulting requests and processed candidates explicitly identify attempt2. The existing attempt ceiling and retry-from-attempt2 rejection remain covered. |
| Immutable previous attempts | All four independent retry cases preserved the original native, face plan, first request and first raw bytes. Changing the detail option after sealing a retry rejected, retaining both `try2.request.json` and `try2.queue.json` bytes. No existing retry was upgraded in place. |
| Opt-in retry tampering | For both legacy inputs, an independently forged attempt2 request with painted fourth-reference pixels and refreshed image/sidecar/request bindings rejected during processing. Previously processed attempt2 records remained unchanged. |
| Native blind identity source | Valid first-attempt and retry processing retained `Candidate.sprite == request.references[0]` (`portrait_process.py:106`). The additional detail does not replace the native sprite used by blind packets. |

The exact `prompt_for()` function source equals the pinned pre-patch version in
commit `6892fbec32047c577d447873a069ed0bb2623bd2`. Both function-source hashes are
`fef5768eedf67a7dbc08acd9f675413eaad61df4cb5e2770bd4d15c6c4d0914c`.
This directly preserves the root's beard, top15% blank margin and garment-height
instructions. The fourth-reference clause is appended separately in
`build_request()`. The Git comparison supersedes the earlier interim evidence
file's statement that a pre-patch snapshot had not yet been verified.

Fresh final verification: **51 passed in 5.11s**, covering
`test_blind_packets.py`, `test_blind_round.py`, `test_blind_score.py`,
`test_portrait_prepare.py`, `test_portrait_process.py` and
`test_portrait_detail.py`. CLI help exited successfully. No dependencies were
added by this review. Implementation-lane lint/typecheck reports are recorded
in `PORTRAIT_PROTOCOL.md`; this independent verdict rests on the source and
fresh executable checks above.

Independent evidence under
`/Users/rexxa/work/artpolish2-data/portraits/protocol_review/`:

- `fourth_reference_geometry_recheck.json`
- `fourth_reference_prompt_git_comparison.json`
- `fourth_reference_retry_compatibility.json`
- `fourth_reference_final_suite.txt`

The optional retry expansion was explicitly authorized by the root during this
review. This verdict establishes the preparation/processing safeguards and
synthetic compatibility only; production generation, portrait visual quality,
blind acceptance and owner adoption remain outside its claims.

### Eligible retry full-prompt override review — 2026-10-24 KST

**Disposition: PASS for the bounded override review; no remaining blocker found.**
The override changes the prompt of a newly created eligible attempt2 request.
It does not permit rewriting a sealed retry or increase the attempt allowance.
All earlier findings and original failure evidence remain preserved above.

`prepare_retry()` rejects explicit blank overrides before writes
(`portrait_prepare.py:171`) and preserves the exact nonblank string as the
request and queue prompt, with no generated prefix or suffix (`:214`). The
observed correction remains separately recorded. Fresh tests preserve the
previous default prompt and serialized request bytes when the override is
omitted or `None`.

Independent temporary integration checks verified:

- Both three- and four-reference paths retained Unicode, CRLF, leading spaces
  and trailing whitespace exactly. First request/raw, native/source, envelope
  and frozen-plan bytes stayed unchanged. Valid attempt2 processing retained
  `Candidate.sprite == references[0]`; identical repeats changed no files.
- A different override rejected before mutation for both reference modes with
  the old third reference present, deleted or corrupted: **6/6 cases**, with
  the entire temporary file inventory and all file hashes unchanged. Separate
  cases also rejected a changed override combined with a changed detail option.
  Existing retry checks precede `build_request()` (`:193`–`:213`).
- Empty and Unicode-whitespace-only overrides, changed frozen plan, changed
  original source, changed envelope, forged detail crop and orphaned retry queue
  all rejected with the entire temporary workspace unchanged. The orphaned
  queue guard runs before reference creation (`:190`).
- The actual Typer CLI was invoked with only its preparation root redirected
  to a temporary fixture. `--prompt-file` produced request/queue UTF-8 bytes
  exactly equal to the input file, including CRLF and whitespace. Supplying
  that option without `--retry-request` rejected without writes (`:226`–`:231`).

Pre/post function-source hashes are identical for `prompt_for()`,
`build_request()`, `bust_pixels()`, `verify_bust_detail()`,
`verified_appearance()` and `process()`. Thus the existing source, appearance,
frozen-reference derivation and processing guards were not replaced by the
override. The existing first-attempt prerequisite, same-plan check and maximum
attempt2 remain in force (`portrait_prepare.py:174`–`:187`).

Fresh six-module suite: **60 passed in 4.99s**. Independent evidence is external
under `/Users/rexxa/work/artpolish2-data/portraits/protocol_review/`:

- `retry_prompt_prepatch_guard_hashes.json`
- `retry_prompt_override_independent_recheck.json`
- `retry_prompt_override_final_suite.txt`

Implementation red evidence remains in `red_retry_prompt_override.txt` and
`red_retry_override_orphan_queue.txt`; this review does not replace those
records with the passing result. No production request, asset or portrait
content was inspected or changed by this review, and no generation was run.
This is tooling verification only, with no new attempts or acceptance claimed.

### Previous-portrait fifth-reference review — initial finding, 2026-10-24 KST

**Initial disposition: P1 OPEN — complete foreign provenance chain accepted.**
This finding is bounded to the new optional attempt2 fifth-reference path.
The existing six-module checks plus the new previous-portrait module passed
**71 tests in 6.79s**, but the independent substitution below still published
output. Prior closed findings remain closed; their evidence is unchanged.

- **Location:** `portrait_previous.py:31`–`:49`, called by
  `portrait_process.py:68`. The helper verifies the first record/request/raw
  chain internally, but receives no consuming retry identity, output directory
  or frozen-plan context against which to authenticate that chain.
- **Reproduction:** create two valid synthetic first attempts in separate
  temporary roots with different raw images. Prepare a five-reference target
  retry, retain its native/style/4x/detail references, appearance, face plan,
  identity and output directory, then replace only `previous_request`,
  `previous_portrait` and reference5 with the second first attempt's complete
  valid chain. Parsing succeeds. Actual `process(..., attempt=2)` also succeeds
  and publishes the target `try2/processed.json` and canonical image.
- **Observed hashes:** target first raw
  `acab5ce0e0322f5940bbd0165334b3fbd8327ef087f290b349217a9d5caf64bc`;
  foreign first raw
  `a56bd6a6ab87f5fef11bf20fc19accf9a6a5f83817a5073a8ff1efef29b123ec`.
  These are different bytes, not an alternate path to the same input.
- **Required boundary:** verify that the authenticated first attempt belongs
  to the current retry's identity, preserved generation directory and frozen
  face plan before preparation/processing writes. Validity of an unrelated
  chain alone must not authorize reference5 substitution. Preserve the
  existing allowance for legitimate current native tint references.
- **Evidence:** external
  `protocol_review/previous_portrait_foreign_chain_reproduction.json`.

Root and implementation lane were notified immediately. All reproduction
assets were synthetic and temporary; no production requests/assets were
prepared or changed, and no portrait content was reviewed.

### Fifth-reference consuming-context recheck — 2026-10-24 KST

**Disposition: foreign-chain P1 CLOSED; bounded fifth-reference review PASS.**
The accepted exploit and its original hashes remain above and in the original
external evidence. The final rerun rejects the same complete foreign-chain
substitution before creating `try2` or changing any file.

`verify_previous_for_retry()` (`portrait_previous.py:52`) now authenticates the
first history and matches its look, variant, generation directory, canonical
output, frozen face-plan hash and native reference0 hash to the consuming retry.
The first native file itself is also verified (`:48`). Both preparation and
processing call this consumer-aware guard; preparation checks the current
baked native and canonical destination before creating additional references.
The root explicitly tightened fifth-reference mode to require the unchanged
first native hash. Existing two/three/four-reference tint-repair behavior remains
available and passed its existing regressions.

Independent final temporary-fixture results:

- The original foreign-chain substitution, changed look, variant, generation
  directory, canonical output, plan and native reference all rejected. A foreign
  fifth image with refreshed binding and a replaced raw with refreshed raw/
  processed-record bindings also rejected. Each rejection retained the entire
  temporary file inventory and every file hash; no attempt2 output was created.
- Preparation rejected changed native/raw/request evidence, a missing first
  processed record, a changed first-record candidate, and a rehashed raw
  replacement before any reference/request/queue writes.
- Historical **2-, 3- and 4-reference** first requests each successfully prepared
  and processed an eligible five-reference retry. `use_previous_portrait=True`
  forced the verified fourth detail without a separate flag. Exact order is
  native, style, 4x enlargement, bust detail, preserved `try1/raw.png`.
- The fifth reference is bound through `PreviousPortrait` to immutable
  `try1/processed.json`, its original request and its raw hash; its raw hash also
  agrees with the verified tool source. An unusable mutable `selected.json`
  did not affect preparation. Request and queue reference ordering agreed.
- Identical repeats changed no files. Sealed retry mode/prompt conflicts
  rejected without changing files. Invalid five-reference attempt1/attempt3,
  missing detail/previous-request bindings and an inconsistent reference count
  all failed request validation. The maximum remains attempt2.
- Actual synthetic processing preserved first request/raw/processed-record,
  native and plan bytes and retained `Candidate.sprite == references[0]`.
  The CLI edit option also exercised the actual preparation path with a
  temporary root and rejected use outside retry preparation.

Legacy serialized request and queue bytes for all **six** combinations of
2/3/4 references and attempt1/2 exactly matched pre-patch commit
`9c782e39a72e9fc0b604021e0b7a969035357f6b`. Absent previous-portrait metadata is
omitted. Function-source hashes for the existing prompt, request construction,
crop and appearance guards are unchanged. The final seven-module suite passed
**81 tests in 7.98s**.

Independent evidence under external `portraits/protocol_review/`:

- `previous_portrait_foreign_chain_reproduction.json` — preserved initial FAIL.
- `previous_portrait_final_recheck.json` — rejection, valid controls and unchanged
  file inventories under the final fix.
- `previous_portrait_valid_compatibility_recheck.json` — ordering and CLI path.
- `previous_portrait_legacy_serialization.json` — pre-patch byte comparison.
- `previous_portrait_prepatch_guard_hashes.json`
- `previous_portrait_final_suite.txt`

The implementation's additional failing context regressions remain in
`red_previous_foreign_context.txt`; they are not erased by this closure.
No production preparation, asset changes, generation, extra attempts, portrait
content review or owner/blind acceptance was performed or claimed by this lane.
