# A1 animal batch decisions — 2026-11-10

Status: production decisions; 196 new frames accepted as art candidates and the
216-frame delivery verified. Owner approval and runtime import remain pending.
Authority: `/Users/rexxa/work/art-batch/BRIEF.md`, content bible
[#690](https://github.com/hyunlord/new-world/issues/690), C2 §3.5 and
[#695](https://github.com/hyunlord/new-world/issues/695).

## D1 — Bound completion by animal appearance and pose

The finite required scope is 17 sets and 184 pose-direction images, recorded in
`art/animal/batch_a1/catalog.json`. The explicit A1 enumeration supplies 14 sets;
the bible adds piglet (capture young), chicken and junglefowl. S/E stand, walk,
run, graze and dead are required throughout; seven wild-capable sets add attack.
Existing deer/wolf are references. Hives remain A2's responsibility. A knowledge
node naming falconry without a species does not add an invented bird family.

Alternative considered: generate only the brief's species names. Rejected because
the brief also requires the bible family and pig/chicken source content would be
left uncovered. Alternative considered: unlimited seasonal and breed permutations.
Rejected because no such finite list is specified. Winter reindeer is a labelled
optional extension after the required matrix.

## D2 — Preserve origin and era without inventing simulation metadata

Neolithic livestock are goat, sheep, pig, dog and harsh-climate reindeer. Cattle
and chicken are bronze; horse is iron. Donkey is named by the bronze cart node,
but has no climate or wild-source row; preserve those fields as unspecified.
Wild animals can exist before domestication; the bible's livestock-era column
does not establish a first-existence era for their wild sources.

The catalogue stores source labels and roles, not guessed `animals.ron` IDs.
Bare animals have no metal, dyed cloth, bells or harness. A separate wild/penned
marker belongs to the importer/renderer. Goat, sheep, reindeer and horse can reuse
one body appearance across those roles; aurochs/cattle and boar/pig are visibly
distinct. This makes ancestry readable without implying that art activates a
domestication mechanic.

## D3 — Freeze one ink language and explicit size targets

`art/animal/batch_a1/STYLE.md` fixes contour/interior ratio 2:1, upper-left light,
upper-left-to-lower-right hatching, material mark budgets and restrained two-tone
wash. Attach actual locked person and scene images plus the relevant deer/wolf
sheet; record actual attachments per call. Historical animal candidates are not
silently reclassified as owner-approved anchors.

Use the existing 96×80 / (48,72) upright and 96×64 / (48,56) dead convention.
The new species shoulder/dead-length targets are proposed art production values
in the catalogue, anchored to existing deer 40 px and wolf 28 px. They are not
biological measurements or balance parameters. Young round 0.6 adult size to
whole pixels. Change targets only with a recorded reason before normalization.
One measured S-stand shoulder scale covers all upright frames in a set; dead
uses its declared length. Do not make per-pose fit corrections to conceal a
generation's clipped or inconsistent anatomy.

## D4 — Keep pose names interoperable and meaning explicit

The brief requests `graze` for every set. Dog's pose depicts sniffing/foraging;
bird graze depicts pecking. These aliases remain explicit in catalogue metadata.
Run and attack must have distinguishable silhouettes. Dead is limp, horizontal,
non-graphic, and keeps the existing bottom margin. These are one key pose each,
not a claim of complete locomotion animation cycles.

## D5 — Separate production evidence, adoption and runtime import

The branch is an art delivery line only. Raw generations, prompts, logs and
rejects live under `/Users/rexxa/work/art-batch/A1/`; committed artifacts contain
assets, their provenance/metadata and curated contact sheets. Numeric and visual
results are recorded separately. Missing checks remain pending. No invented seed,
model identifier, approval, measured hue or lighting statistic is permitted.

An alpha component/anchor pass does not certify camera, anatomy or style. Review
every pose at native scale and 2× beside existing references and other species,
record visual-verdict JSON before retries, and reject weak work. Sprite seam
testing is not applicable; cutout edges and atlas gutters are checked instead.

Draft import/binding rows are handed to C2/C3 without applying them. Runtime
integration, Godot playback and gameplay verification remain with that import
pass. Push only `v8/art-batch-a1` and verify remote landing; do not move the main,
WIP, C2, or other art branches.

## D6 — Freeze generation records after hashing

Seven generator-record hashes became stale when later review notes were added to
the records. The verifier now checks the optional generator-record hash as well
as source, prompt and reference hashes. A regression demonstrated the former false
pass and now rejects changed evidence. Before assembly, root verified raw/prompt/
reference equality and reconciled the seven manifest hashes; PNGs did not change.
Keep future dispositions in separate ledgers instead of editing hashed records.
Evidence: `/Users/rexxa/work/art-batch/A1/records/generator-record-reconciliation.json`;
`art/animal/batch_a1/test_process.py`.
