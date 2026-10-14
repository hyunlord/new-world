# Final independent image packet protocol

Preparation only. No observer has been launched and no final frame has been graded by this document. Earlier-pass scripts and evidence remain unchanged.

## Freeze, then observe

1. Wait for the final raw manifest and source PNGs; calculate every screenshot SHA256. Store exact selected sources, selection rule and frame count in `review/packet_sources.json`.
2. Create packets from those pixels only. Use opaque filenames `V01.png`, `W01.png`, `C01.png`, etc. Do not put seed, day, cold/warm, cast type, expected direction, A-code or desired answer in observer-facing names.
3. Write answer files and `.sha256` companions **before** launching fresh `fork_turns="none"` observers. Record `created_ns`. Keep answer files outside observer-facing folders. Instructions explicitly prohibit repository source, GOAL, metadata/sidecars and answer-file access.
4. Two distinct fresh observer sessions answer independently. Give each the same image bytes and neutral questions. Save original JSON, then hash it before grading. Never edit answers to fix spelling/interpretation; any normalization is a separate recorded transform. Uncertain/unknown remains uncertain/unknown.
5. Grade only after both responses are saved. Record answer hashes, observer identities and source hashes. Maximum two review rounds globally; retain the first round verbatim if screens change.

## Artifact map and frame counts

| Criterion | Observer pixels | Required count / unchanged threshold | Consumed artifact |
|---|---|---|---|
| A1–A10,A12–A20,A26 | No new blind answers; pixel/FFI/gate/provenance measurements | Existing formulas unchanged | Raw capture manifests, gate, A20 current-head audit |
| A11 activity | Actual renderer calibration lineup, numbered complete-cue crops at 4x and2x | Exactly24 at each zoom, exactly3 carry, at least3 eat, all9 activity categories; each observer4x>=85%,2x>=70%,2xcarry>=2/3 | `blind_results.json`, plus `blind_identity_final.json` |
| A21 shelter/frost/breath/ash | Natural winter/night frames, identity-only object overlays plus unaltered source and actual >=2.4s sampled cycle contacts | Every qualifying natural winter-night frame, no cherry-picked subset | `visual_observations.json` |
| A22 morning cue | Every measured cold-morning frame plus valid measured warm control, blinded together | Observed agent IDs must match independently tracked90-minute eligibility interval; warm control shows none | `visual_observations.json` |
| A23 HUD | Every natural gameplay frame, including strip/evening frames; cropped HUD contacts keep text pixels1x | Every natural frame's cold count and winter label independently transcribed | `visual_observations.json` |
| A24 effects | Every live-seed ART_GOD effect frame, actual timed sequence where needed | Independent world-effect observation required; panel text alone is insufficient | `visual_observations.json`; report intervention scope explicitly |
| A25 cold readability | Natural cold/night frames with cold villagers and extinguished fire | At least2 independently observed cold cues per qualifying frame | `visual_observations.json` |
| A27 village recognition | Four **unselected FV** society frames, chosen before observation from remapped society milestones | Each observer>=3/4 exact count+parent mapping | `blind_society/results.json` |
| A28 travelling-group recognition | Two first-departure FD frames (+30min,+2h) | Each observer2/2 activity choice correct | `blind_society/results.json` |
| A29 targeted rain | Initial daughter-rain toast plus actual same-frame original/daughter night, and associated society rain frames | Actual daughter named; both hearths independently observed; daughter only rain | `visual_observations.json` |
| A18b countability | Exactly12 crowded frames, chosen by predeclared A18 count ranking / strata, opaque names, original pixels | Two yes/roughly/no responses per frame; OBSERVED companion only, never replaces A18 | `qa_evidence.json` field `blind_crowds` plus frozen raw responses |
| A32 direction | Every FD frame containing a raw walking group; additional second-departure FD included if captured | Both observers correct among8 directions on **every** FD frame | `qa_evidence.json` field `blind_way` plus frozen key/responses |
| Part8 review | Departure feed, arrow, pre-split meter, morning crowd, paired worlds | Independent ACCEPT / NEEDS WORK with reasons, max2 rounds | `review/final_round1.json` (round2 only if necessary) |

A27 still needs four unselected FV frames even if the minimum Part5 list mentions fewer; preserve the inherited four-frame coverage using measured milestones. Do not call `blind_p7.py create` unchanged: its dates are fixed to v17. Its `grade` and `check_blind` accept explicitly supplied current paths and preserve unchanged thresholds.

## A11 exact workflow and schemas

Existing pure/path-parameterized tools can operate on current evidence without writing prior-pass paths:

```sh
python3 tools/art_pipeline/make_blind_grid_p6.py create --raw RAW_ART --output docs/art/production8/blind --key docs/art/production8/blind/frozen_answers.json --repo .
# Immediately write frozen_answers.sha256, before launching observers.
python3 tools/art_pipeline/make_blind_grid_p6.py grade --labels OBSERVER_JSON --key docs/art/production8/blind/frozen_answers.json --output OBSERVER_GRADED_JSON
python3 tools/art_pipeline/blind_p6.py GRADED_A GRADED_B docs/art/production8/blind_results.json
python3 tools/art_pipeline/verify_blind_identity_p7.py RAW_ART docs/art/production8/blind docs/art/production8/blind/frozen_answers.json docs/art/production8/blind_identity_final.json
```

Observer JSON: `{"observer_id":"unique","4x":{"1":"walk",...},"2x":{"1":"walk",...}}`.

Graded row fields consumed by A11: `observer_id`, `count:24`, `absent_activities:[]`, `observer_labels_sha256`, `accuracy_4x`, `accuracy_2x`, `confusion:{"4x":{truth:{answer:count}},"2x":...}`. `blind_results.json` contains `{"observers":[gradedA,gradedB]}`. Both original observer label files and their SHA companions must remain retained.

Neutral prompt: “Label each numbered figure at both shown scales using walk, carry, gather, build, eat, drink, sit, talk, child, or unsure. Use only the images. Save the JSON labels; do not inspect answer files, metadata, repository code or design documents.”

**Final identity guard implemented:** `measure_p8.py` uses `blind_identity_p8.final_identity` and `guard_activity`. Both exact final lineup PNG hashes must match a passing `blind_identity_final.json`; missing proof or changed bytes forces A11 FAIL without changing observer answers. Run the identity verifier after final captures; a stored grading result alone is insufficient.

## A27/A28 frozen key and results

`blind_society/frozen_answers.json`:

```json
{"created_ns":0,"A27":{"1":{"count":2,"parents":{"2":1}},"2":{},"3":{},"4":{}},"A28":{"1":"moving away to settle","2":"moving away to settle"},"sources":{"A27_1":{"source":"absolute raw PNG","image":"opaque packet copy","sha256":"..."}}}
```

Values above illustrate schema only; freeze actual answers from current raw getter data. Society frame count must use the inherited implementation's exact snapshot semantics and record alive/history distinction, not silently change it. Source entries required for all six images. Observer JSON: `{"observer_id":"unique","A27":{"1":{"count":2,"parents":{"2":1}},...},"A28":{"1":"choice","2":"choice"}}`.

Neutral prompt: “For each V image, how many villages can you distinguish and which village appears to have originated from which? Give count and daughter-number→parent-number mapping; use unknown if unclear. For each W image, what are these people doing? Choose working, wandering, moving away to settle, fleeing, or unsure. Use only pixels.”

Run existing `blind_p7.py grade KEY OBSERVER_A OBSERVER_B docs/art/production8/blind_society/results.json`; this writes only supplied output and observer hash companions. The result includes `key`, `key_sha256`, `sources`, and two observer rows with `labels`, `labels_sha256`, `correct:{A27:n,A28:n}`, `total:{A27:4,A28:2}`. `check_blind` rechecks hashes and scores against the frozen key.

## Map observed pixels to IDs without revealing truth

Render neutral identity annotations for every visible agent/building, not only those expected to have a cue. Label `A<id>` / `B<id>` outside the object, with a1px bounding outline; keep the unaltered original beside the annotated image. These labels disclose identity only, not cold state/activity/eligibility. Preserve mapping `{label,id,kind,source_sha256}` outside the response; reviewers return the visible labels they identify. Never prehighlight only cold/frosted/target objects.

For a tiny cue that annotations would cover, use numbered native-scale crops or an actual sampled cycle contact. Include the complete original cue rectangle with context; no generated pixels, enhancement or selective recolouring. The observation remains bound to the original source PNG hash and the contact's separate hash.

## visual_observations.json

Top-level shape: `{"sources":[{"path":"original observer JSON","sha256":"..."}],"frames":{"actual_source_stem":{...}}}`. Every frame row has `sha256` of the **unmodified raw PNG**, `observer_id` or `observer_ids`. Merge only the reviewer fields actually observed; do not fill missing answers with expected values.

Fields consumed by current P8 tools:

- `hud_text`: literal transcription; `severity_visible`: boolean.
- `frosted_shelter_ids`, `ash_hearth_ids`, `lit_hearth_ids`: integer object IDs obtained by neutral-label mapping.
- `doorway_cycle_visible`: actual cycle observation, not inferred from a static frame.
- `chill_agent_ids`: IDs whose visible morning cue the observer identifies.
- `visible_cues`: observed vocabulary from frost, doorway_breath, cold_ash, tint, hud_count; checker requires>=2 for A25.
- `intervention_effects`: observed kind vocabulary (gift_food, spoil_food, gift_wood, rot_wood, rain, cold_snap, windfall_dust, windfall_dust_ended, windfall_drop, windfall_drop_ended, panel as applicable).
- `world_effect_visibly_identified`: false when the observer sees only panel confirmation.
- `toast_text`: literal applied-toast transcription; `rain_settlement_ids`: mapped village IDs the observer identifies as receiving rain.

Neutral prompt: “Transcribe visible HUD/toast text. Describe visible markings, outlines, breath/smoke/particles and fire states, naming the neutral object labels where possible. For each actual time sequence, say which changes you can see and when. State absent or uncertain when the pixels do not establish a cue. Do not infer from the filename, requested feature or sidecar.” The grading coordinator maps the original free-form descriptions to the finite consumed vocabulary in a separate retained transform; retain original prose to audit that mapping.

Explicit P8 timing rule: `observation_rules_p8.intervention_visible` expects dust through elapsed<1.3s (full hold0..1.0s, fade1.0..1.3s) and the unchanged pile drop through elapsed<0.8s. Categorical presence does not by itself prove alpha intensity; retain actual render metadata and the visual sequence for the hold/fade claim. The P7 timing rule and its0.6s diagnostic remain untouched. P8 observed checks import only the new timing rule; cold eligibility remains inherited.

## A18b/A32 raw and normalized schemas

Before observing, freeze `review/blind_new/frozen_answers.json` and hash. For A18b freeze source selection, not a manufactured correct countability answer. For A32 derive eight directions from source-frame centre→raw-site direction, quantized to N/NE/E/SE/S/SW/W/NW (screen y grows downward); freeze angle/vector, raw-site evidence and direction before observers run.

Observer JSON: `{"observer_id":"unique","countability":{"C01":"yes|roughly|no",...},"direction":{"D01":"N|NE|E|SE|S|SW|W|NW|unsure",...}}`.

Neutral prompts: “For each C image, can you count the people here? Answer yes, roughly, or no.” “For each D image, where is the travelling group going? Point in one of N, NE, E, SE, S, SW, W, NW; unsure is allowed.”

`qa_evidence.json` normalized fields consumed now:

```json
{"blind_crowds":[{"source":"raw PNG","sha256":"...","answers":["yes","roughly"]}],"blind_way":[{"source":"raw FD PNG","sha256":"...","expected":"SW","answers":["SW","SW"]}]}
```

Exactly12 crowd rows with two valid answers each; all actual FD walking frames must appear exactlyonce in blind_way. Keep observer IDs/label files/key hashes in the raw response/index artifacts, because the compact runner input does not independently encode those fields. New-criterion evaluator checks hashes and answers but the freeze/independence chain must also be audited from retained files.

## A28 geometry integrity confirmation

`capture_framings.gd` now sets `marker_in_view` true for an on-screen stake **or** a visible edge arrow. This does not by itself relax the inherited A28 result: `ingame_qa_founding_p8.check_group` still separately requires `site_in_view`, exactly as P7. Its bytes match the P7 checker. An off-screen site therefore fails A28 while a correct edge arrow can pass A32. The changed metadata alone must not be cited as an A28 threshold change.

## One-command packet preparation (implemented)

```sh
python3 tools/art_pipeline/prepare_blind_p8.py RAW_ART RAW_LONG RAW_SOCIETY RAW_GOD
```

Run only after final manifests are complete. Default output is `docs/art/production8/review`; `P8_REVIEW_OUTPUT` can explicitly select a separately retained round directory. This never launches observers or supplies answer values to responses. It refuses a pre-existing frozen key directory/packet manifest. Preflight requires4unselectedFV,2walkingFD,12natural frames before creating any key. A multiple-walking-group FD needs explicit group-question disambiguation and causes a preparation error instead of silently choosing one answer.

Outputs:

- `review/packets/activity/blind_{4,2}x.png`; private `review/keys/activity.json/.sha256`.
- `review/packets/society/V01..04.png,W01..02.png`; private `blind_society/frozen_answers.json/.sha256` at the review directory's parent.
- `review/packets/direction/Dxx.png`; private `review/keys/direction.json/.sha256` (all walkingFD).
- `review/packets/countability/C01..12.png`; private `review/keys/countability.json/.sha256` with unchanged A18 ranking scores.
- `review/packets/objects/batchNNN/Oxxx.png`, identity-only `_ids.png`, applicable `_sequence.png`; batches contain8source frames. Original time/day filenames in cycle-helper captions are removed from the observer-facing copy, and original cycle contact remains coordinator-only.
- `review/packets/hud/Hxxx.png`: four natural HUD crops per contact, including all strip/evening frames, pixels1x.
- `review/packet_sources.json/.sha256`: source directories/counts plus every observer PNG hash.
- `review/packets/INSTRUCTIONS.txt` and `RESPONSE_SCHEMA.json`: neutral questions and response field descriptions, no answers.

A27 four overviews are selected evenly through the captured chronology, including the final overview; selection is frozen before observation. A18b selects the top12 by unchanged half-occluded count, then visible population, then path. All ranks/selection facts stay in private keys.

The helper is prepared but has not run on final captures. It may create substantial native-resolution packet data; include it in the30MB production8 budget and retain large review packets externally if necessary through an explicitly selected output directory and stable path bindings. Do not downscale evidence to conceal crowd/arrow defects.
