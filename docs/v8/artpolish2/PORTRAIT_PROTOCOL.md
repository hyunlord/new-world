# ART-POLISH2 portrait production and blind-test protocol

Status: protocol frozen before portrait generation; no score or acceptance is claimed.

## Source count and execution plan

The authoritative `art/character/route_i/MANIFEST.json` contains 16 accepted
adult looks and 18 child looks, plus two missing adult looks with named bun
fallbacks. The goal's introductory 204-face total agrees with this source;
§3.3's “17 adult” does not. Generate six variants for each of the 34 actual
looks (204 distinct portraits). The two missing looks reference their
fallback's six variants, giving 216 logical slots without inventing assets.
The goal text remains unchanged; the leader records this discrepancy in D3.

1. Read the frozen face plan and prepare one request per actual look/variant,
   hashing the tinted S/idle reference, style anchor and plan.
2. Record each image call in the shared generation ledger before executing it.
   Preserve raw output externally. Each variant permits at most two attempts.
3. Normalize only by full-frame scaling to 128×128, preserving raw provenance.
   Run the unchanged POLISH geometry instrument on observed boxes; numeric
   and model style scores are report-only. Never infer observations from the
   prompt or intended result.
4. Build the anonymous blind packets below only from completed candidates.
5. Collect descriptions and answers, score every look and each reviewer
   independently, and retain all failures. One failed-variant regeneration
   round is permitted; its round uses two new reviewers.
6. Hand the 8×6 owner selection to the board compositor, retaining owner
   choice pending independently of technical and blind-test results.

## Fixed sample rule

The authoritative frozen selections are `owner_sample` and `identity_sample`
in `art/character/FACE_VARIANTS.json`. Sort actual look IDs lexically inside
each of the four age/sex cohorts. Distinctness uses the first two looks of each cohort: four adult and
four child looks, two of each sex at each age. These same eight looks feed
`faces_owner.png`. Identity uses owner-sample indices 0, 1, 2, 4, 6, 7;
each contributes all six variants, for exactly 36 cases. This covers all
skin, hair, age, feature and male-beard axes of the frozen six-variant plan.
Selection is independent of image quality and reviewer results.

## Distinctness, two directions

Each reviewer receives an independently shuffled six-portrait row for all
eight looks. The only visible names are opaque question IDs and A–F labels.
A and B each write one visually specific line per displayed portrait.
Neither receives variant descriptions, source names or answer metadata.

After both description files are sealed, a separate matching packet for A
contains B's six lines and a newly shuffled portrait row; B receives A's.
Both the line order and portrait order are independently shuffled. Matching
answers independently map all line IDs 1–6 to image labels A–F. Repeated image
labels are allowed: requiring a bijection would make exactly 5/6 impossible
and silently strengthen the specified bar to 6/6. Score
each look separately in both directions: every one must reach at least
5/6. Total or mean accuracy cannot rescue a failed look.

## Identity with tints

Each question shows its 128px portrait and four idle sprites at exactly 2×
nearest-neighbour scale: the target, two other variants of the same look,
and one variant of another look in the same age/sex cohort. Options and
question order are shuffled. The same-look alternatives are the next and
third-next variant modulo six. The other look is the next lexical cohort
look and uses the target variant index. Each reviewer answers all 36
questions without seeing any descriptions or keys. Each must reach 32/36.

## Evidence isolation and integrity

Raw inputs, packets, keys and original reviewer outputs live under
`/Users/rexxa/work/artpolish2-data/portraits/`. Reviewer packet directories
contain images and instructions only. Keys are kept in a separate `private/`
directory, never in packet filenames, PNG metadata, instructions or sibling
manifests exposed to a reviewer. Reviewers are instructed to inspect only
their assigned packet directory. Fresh reviewers have no production context.
SHA-256 bindings cover every candidate, reference, packet and response.
Before either reviewer receives a packet, `blind_round.freeze_round` seals
the source candidate index, a separate copy of that index, the face plan,
and content-addressed copies of every portrait and tinted sprite. Both keys
bind the same immutable round manifest. Description, matching and scoring
load these frozen candidates and reject changes to the original index,
original source images, plan or frozen copies. Do not refresh the global
candidate index during an active round; generation may continue elsewhere.

The roster records the actual canonical agent task, an optional separately
available session ID, the isolated-context launch declaration, and a
hash-bound actual spawn receipt. A/B must be different agents. Round1
must cite round0's immutable manifest and use two tasks disjoint from its
reviewers; any supplied session IDs must also be disjoint. A canonical
task and actual receipt suffice when the native tool returns no UUID;
never fabricate a session ID. Each description/answer carries the assigned
task/session stamp, which the evaluator checks. The receipt and declaration
are explicit orchestrator evidence, not identity inferred from portrait
content. No roster or key exposes answer information inside a packet.
Scores reject missing/duplicate/extra IDs and invalid option labels.

The committed summaries retain the protocol, evidence hashes, per-look and
per-reviewer counts, failed variant identities for the bounded repair, and
the explicit status `OWNER_CHOICE_PENDING`.

## Commands and current pipeline validation

`python3 -m art.pipeline_polish2.portrait_prepare --available-only` emits an
explicitly partial `requests.json` and the image-tool-ready `queue.json` in
the external portrait directory. Owner-sample looks come first. Omit the
flag to require all 204 tinted references. A request whose generation has
begun cannot be replaced with a differently hashed reference.

`python3 -m art.pipeline_polish2.portrait_process REQUEST RAW --attempt 1`
preserves the raw bytes, writes the 128px candidate, and records the existing
ENV1 numeric judge as report-only. Add `--inspection FILE` to consume actual
POLISH `Inspection` observations; without it structure remains
`PENDING_OBSERVATION`. Full observed `Inspection` fields and their immutable
input bytes are retained, including hair/top/accessory findings. A changed
observation for the same raw attempt creates `processed.revision002.json`
(then003, etc.); the first `processed.json` and earlier FAIL measurements
remain intact. `selected_revision.json` points to the selected immutable
revision. Because the existing judge needs SciPy/scikit-image,
use `uv run --with pillow --with pydantic --with typer --with numpy --with
scipy --with scikit-image python -m ...` where the system environment lacks
those existing dependencies.

`portrait_process.collect()` seals `candidates.json` after verifying selected
portrait and tinted-reference hashes. `python3 -m
art.pipeline_polish2.portrait_inspect` then produces native128 and2x
coordinate-labelled sheets for independent observations, without estimating
any box in code.

The blind sequence is:

```sh
python3 -m art.pipeline_polish2.blind_packets ROSTER_JSON --round-number 0
python3 -m art.pipeline_polish2.blind_matching A_DESCRIPTIONS B_DESCRIPTIONS
python3 -m art.pipeline_polish2.blind_score A_IDENTITY B_IDENTITY A_MATCH B_MATCH
```

Each reviewer reads their directory's `INSTRUCTIONS.txt` for the strict JSON
response shape, including `agent_task` and optional `session_id` provenance.
The roster file has `round` and exactly two `assignments`, each containing
`reviewer` (A/B) and `agent`: `agent_task`, `session_id` (null if unavailable),
`isolated_context: true`, and `spawn_receipt: {path, sha256}`. Root supplies
these from actual isolated launches before generating the packets.

The scorer seals original responses and keys, and writes raw-bound per-reviewer/per-look results to
`docs/v8/artpolish2/blind_scores_round0.json`. Round1 uses the same commands
with `--round-number 1` and two new reviewers; it never overwrites round0.
Different rechecks within a round create numbered `.revision002.json`
results alongside the immutable first result. `latest_score_revision.json`
points to the selected external result, while all earlier failures remain
available. Score keys must match both the requested round and its manifest.

Pipeline-only verification on 2026-10-24: 11 tests passed, including actual
temporary-file packet rendering, description transfer with independent
shuffles, true5/6 acceptance,4/6 failure, malformed response rejection, raw
byte preservation, reference-change rejection, and historical-attempt
overwrite rejection. Ruff passed; basedpyright (system Python3.14 path) had
0errors/0warnings; the programming no-excuse checker found no violations in
the 12 files. Rendered synthetic fixture panels were inspected: all128px
portraits,2x80×112 sprites and opaque labels fit their panels. These are
pipeline checks only, not evidence of portrait quality or blind-test success.

Protocol repair validation: the pre-fix regressions reproduced both an
accepted candidate replacement after descriptions and acceptance of a
round1 key without actual reviewer evidence. Their red output is retained at
`/Users/rexxa/work/artpolish2-data/portraits/protocol_review/red_pipeline_fixes.txt`.
The repaired suite has22 passing tests, adding changed candidate rejection,
frozen-copy corruption rejection, exact round binding, reused canonical-task
and session rejection, unavailable-UUID handling, response-agent mismatch,
and immutable score/observation revision checks. Thresholds are unchanged.

Aggregate-scoring repair: a further independent reproduction copied A's
completed key and answers into both CLI slots and produced a misleading
`[A,A]` PASS. The actual CLI regression first reproduced that failure, then
passed after enforcing exact ordered A/B roles plus each scored agent's full
record against its frozen roster assignment and distinct task/session IDs.
No score is published for the duplicate case. The original reproduction and
red output remain under external `portraits/protocol_review/`; previous
score artifacts and the specified thresholds remain unchanged.

Final repair verification: all24 portrait/blind tests passed, including the
duplicate-slot rejection and a positive aggregate using both distinct frozen
assignments. Ruff passed across the16 owned Python files; basedpyright
reported0errors/0warnings, and the programming audit found no violations.
The retained verification record is external
`portraits/protocol_review/green_duplicate_reviewer_fix.txt`.

Source-appearance repair preserves the original30 first-attempt requests and
raws. `python3 -m art.pipeline_polish2.portrait_appearance` imports the literal
`appearance_brief` entries from `OWNER_SOURCE_ENVELOPES.json` and
`IDLE_SOURCE_ENVELOPES.json`, verifies each original S/idle hash, and requires
exact coverage of the34 accepted looks. The resulting
`art/character/source_appearance.json` records source and envelope hashes.
New prompts include those observations verbatim; filenames are identifiers,
not descriptions of clothing or visible hair length. The frozen variant's
age, beard and colours still control those identity axes.

New requests contain three ordered references: native tinted S/idle, the
portrait style anchor, then an exact4× nearest-neighbor expansion of the
same native sprite. The enlargement preserves RGBA pixels and alpha without
painting, filtering or cropping. Its immutable filename includes the native
source hash. `Candidate.sprite` remains reference0. Older two-reference
requests still parse with `attempt:1` as the default.

After corrected sprite references are frozen, prepare the unstarted queue:

```sh
python3 -m art.pipeline_polish2.portrait_prepare --unstarted-only
```

This skips completed/started directories and reports a partial queue against
the unchanged expected total204, including `skipped_started_count`. It does
not authorize a generation call or consume the root-owned call budget.

For an explicitly selected retry, use:

```sh
python3 -m art.pipeline_polish2.portrait_prepare --retry-request ORIGINAL_REQUEST_JSON --correction 'Concrete observed correction'
python3 -m art.pipeline_polish2.portrait_process TRY2_REQUEST_JSON RAW_IMAGE --attempt 2
```

The Python API is `prepare_retry(original_request_path, correction, root)`.
It requires preserved first-attempt raw evidence and the same frozen face-plan
hash, then writes separate immutable `try2.request.json` and `try2.queue.json`
files. The latter is one ToolRequest object with `attempt:2`, `request_path`
and three referenced image paths. Repeating identical preparation is safe;
conflicting retry requests and third attempts reject. Processing verifies the
request's attempt, appearance-file hash, original-source hash, native/expanded
references and previous-request hash before publishing. The first request,
try1 inputs, outputs and failed observations remain unchanged.

Source-reference repair verification on2026-10-24:32 tests passed, including
literal observed-text propagation, exact4× RGBA pixel replication, stale
original/appearance rejection, separate retry preservation, retry-limit and
attempt checks, legacy two-reference parsing, and all prior blind gates.
Ruff and the programming audit passed18 owned files; basedpyright reported
0errors/0warnings. All30 production request files still matched their original
`try1_inputs/request.json` snapshots byte-for-byte, and all30 `try1.png` raws
matched preserved `try1/raw.png` bytes. Logs remain in external
`portraits/protocol_review/{red_source_appearance_prepare,red_source_appearance_process,green_source_appearance}.txt`.
This is tooling evidence, not validation of regenerated portrait appearance.
The appearance importer must run after both source-envelope reviewers mark
their reports READY, before new production requests are prepared.

Linked-envelope guard followup: independent review found that the compiled
appearance manifest's envelope hashes were recorded but not checked when
used. Four red regressions changed either of two linked envelope files after
compilation and confirmed that preparation and processing incorrectly accepted
them. Both paths now use the same `verified_appearance()` loader: it verifies
the compiled manifest and every linked envelope hash before any output write.
Recompilation after an intentional source-review update remains required.
No final production compilation was run while the v4 source review was pending.

The full scoped suite now passes36 tests; Ruff, basedpyright(0errors/0warnings)
and the18-file programming audit pass. The30 original requests and raws remain
byte-identical to their preserved copies. Red/green logs:
`portraits/protocol_review/red_linked_envelope_guard.txt` and
`portraits/protocol_review/green_linked_envelope_guard.txt` outside the repo.

Fourth-reference extension, after observed garment-height failures: newly
prepared unstarted requests now append a source-derived bust detail. The
native tinted S/idle, style anchor and exact4× reference remain references1–3.
Reference4 is derived from the **native tinted RGBA source**, using its nonzero
alpha bounding box, the full width of that box and the upper
`ceil(visible_height * 3 / 4)` rows, followed by exact12× nearest-neighbor
replication. Every retained RGBA value, including RGB behind zero alpha, is
unchanged. No painting, mask edits or source annotation changes are involved.

Immutable external reference files use
`<native_sha256>.upper3of4.12x.{png,json}`. The JSON records native source and
output file hashes, visible bbox, crop bbox, scale12 and explicit half-open
coordinates (left/top inclusive, right/bottom exclusive). `Request.bust_detail`
and `ToolRequest.bust_detail` hash-bind that sidecar. Four-reference requests
require it; historical two/three-reference requests remain valid without it.
Preparation checks existing detail provenance before writing other references;
processing checks it before raw/canonical/result writes. Both recompute the
native-alpha crop and exact RGBA pixels, rejecting painted outputs or changed
crop/scale metadata even if their hashes have been refreshed. `Candidate.sprite`
continues to reference the native first image, with blind packets unchanged.

The existing beard, top15% blank margin and garment-height prompt instructions
remain in `prompt_for()`. A separate fourth-reference clause is appended by
`build_request()` only when the detail exists, directing the generator to
retain the observed clavicle-level edge, trim direction and shoulder coverage.
The detail does not replace the portrait composition or create a new design.

An explicitly authorized later retry may opt into the same additional evidence:

```sh
python3 -m art.pipeline_polish2.portrait_prepare --retry-request ORIGINAL_REQUEST_JSON --correction 'Concrete observed correction' --include-bust-detail
```

Python: `prepare_retry(path, correction, root, include_bust_detail=True)`.
The default remains three references. Opt-in creates only separate
`try2.request.json` / `try2.queue.json`; original requests/raws remain intact,
and differing rewrites of an already sealed retry still reject. The maximum
attempt2 and root-owned image-call budget are unchanged. No actual production
request preparation or generation was performed by this tooling extension.

Fourth-reference tooling verification:51 tests passed. New tests compare every
synthetic output pixel with independently indexed native pixels; reject stale
sources, rehashed painted output and changed bbox/scale metadata; verify rejection
before reference/output writes; preserve failed processing history; exercise
legacy2/3 references and explicit retry opt-in. Ruff and the19-file programming
audit passed; basedpyright reported0errors/0warnings. External logs retain the
red cases and final green record under `portraits/protocol_review/` as
`red_bust_detail.txt`, `red_bust_detail_prepare.txt`,
`red_bust_detail_retry_optin.txt` and `green_bust_detail.txt`.

Actual final-image observation found that some male variants copied a full
source beard despite the frozen clean-shaven brief. For unstarted male requests
only, the prompt now opens with the explicit full/short/clean requirement and
describes a planned identity variant instead of the exact source person.
Clean means a bare chin, jaw and upper lip without stubble; short means closely
cropped facial hair without a bulky hanging beard. The six-variant plan, source
references, female/child prompts and all started requests remain unchanged.
Observed failures on exhausted second attempts are retained; this clarification
does not authorize a third attempt or imply the model followed the new prompt.
Verification: 138 prepared female/child prompts were byte-identical; all36
portrait/blind tests, Ruff and strict typing(0errors/0warnings) passed.

The first49 final candidates include three exhausted second attempts whose
highest hair tips have a conservatively observed1px top margin. The unchanged
structure threshold remains2px. Unstarted requests now ask for a visibly empty
top15% parchment band and head bounds y20..104 to provide a wider generation
margin. Started prompts and images are not reframed or patched. This is an
input clarification, not a changed evaluation rule or an extra attempt.
The full36 protocol tests, Ruff and strict typing again pass after this edit;
actual framing adherence remains subject to observed-image review.

Actual later outputs also lowered some children's diagonal wraps and changed a
straight adult bandeau trim into a diagonal fold. Unstarted prompts now explicitly
preserve the source garment edge's height relative to the shoulders/clavicles,
its upper-trim direction and its chest coverage. This adds no look-name inference
and changes neither the source-appearance manifest nor the frozen face plan.
Existing failed first attempts remain candidates for the single coordinated
repair phase within the original attempt and call limits.

Eligible second attempts may now provide an explicit complete prompt at first
creation via `prepare_retry(..., prompt_override=full_text)`. This text becomes
the exact immutable `Request.prompt` and queue prompt, with no generated prefix
or suffix. `correction` separately records the actual observed repair reason.
Omitting the override or passing `None` retains the previous default prompt and
serialized request bytes. Blank/whitespace-only overrides reject.

CLI usage accepts the full UTF-8 text, preserving line endings and whitespace:

```sh
python3 -m art.pipeline_polish2.portrait_prepare --retry-request ORIGINAL_REQUEST_JSON --correction 'Actual observed failure' --prompt-file COMPLETE_PROMPT.txt --include-bust-detail
```

`--prompt-file` is valid only for retry preparation. Optional fourth-reference
derivation, native/source/appearance hashes and the same frozen face plan are
still verified. Existing second requests/queues are checked before reference
creation or recovery: different prompt/correction/detail selection, changed or
missing references, inconsistent queue evidence, and an orphaned queue all
reject without rewriting history. An identical repeat remains safe. This adds
no generation calls, attempt allowance or source-plan changes.

Full-prompt override verification:60 tests passed; Ruff, strict typing
(0errors/0warnings) and the19-file programming audit passed. Regressions cover
exact Unicode/CRLF/spacing, both reference modes, blank rejection, default-byte
compatibility, and complete temporary-workspace preservation on conflicts.
External records: `portraits/protocol_review/red_retry_prompt_override.txt`,
`red_retry_override_orphan_queue.txt`, and `green_retry_prompt_override.txt`.
No production request preparation, generation or portrait editing was performed
by this tooling followup.
