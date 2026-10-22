# ART-TRIAL2 decisions

## D1 — Isolated art branch and restricted paths
Context: repository defaults name v8/wip; this pass explicitly requires v8/art-trial2 and only art/** plus docs/v8/arttrial2/**.
Choice: follow the pass-specific branch and file scope, no game changes or merge; preserve pre-existing .gitignore/.ignore/docs/art work.
Reason: the specific experiment contract overrides general end-of-pass edits to global docs.
Evidence: initial HEAD a540686647ed37a2f65d4b5b6c2a5d22960fa9f4 on v8/art-trial2.

## D2 — Accepted original selection
Context: initial S5 props and S7 camp images were superseded.
Choice: person S1_2; hut S4_1; props S5_2; ground S6_1; scene S7_3; portrait S8_1. Copy exact original bytes and preserve hashes.
Reason: lock accepted outputs, not rejected/superseded attempts. Anchor original-resolution allowance is used for these six files only (about 24 MB total).
Evidence: art/style05/anchors/manifest.json and docs/v8/artconcept/ORIGINALS.json; six SHA256 checks succeeded.

## D3 — Explicit plan and autonomous execution
Context: the complete seven-part brief already authorizes implementation and forbids owner questions.
Choice: use TRIAL2_PROGRESS.md as the execution ledger, direct work plus bounded native subagents; no additional approval handoff or out-of-scope planning files.
Reason: preserve scope and execute the supplied decision-complete contract.
Evidence: TRIAL2_GOAL.md rules and Part 0–6.

## D4 — Preserve model-judge identity
Context: generation tool returns images; a fresh-context multimodal Codex agent can return text scores, but is not proven to be the same image-generation model.
Choice: record its actual identity and scores, retain literal image-model-judge compliance as NOT_VERIFIED unless a matching surface is established.
Reason: silently equating tools would overstate compliance. Numeric checks and style judgments remain separately visible.
Evidence: available image_gen.imagegen tool schema versus native reviewer agent interface.

## D5 — Historical trial reference recovery
Context: docs/v8/arttrial/TRIAL_REPORT.md and COST.md are absent from the concept base tree.
Choice: read both via git show v8/art-trial:<path>, pinned at c3dafb07614ef58944d5dee4bfa454bd141a4854; do not copy historical files into this branch.
Reason: retrieve the requested historical evidence without importing unrelated work.
Evidence: historical cost reports 1019 calls, 1008 returned originals, and persistent person-layer/readability failures.

## D6 — Actual image-model scorecard surface
Context: D4 correctly separated Codex vision from the image-generation model. An actual tool probe subsequently returned four numeric scores and a sentence as a raster scorecard.
Choice: invoke image_gen separately for each candidate with two locked anchors and the candidate; request an undecorated evaluation card, preserve the card and transcribe its output verbatim. Do not supply desired scores. Keep the independent Codex review as separate corroboration.
Reason: exercise the requested image model directly instead of silently substituting reviewer identity. Each invocation receives only the neutral judging prompt and specified images; model internal isolation is not externally inspectable.
Evidence: negative-photo card exec-86d21acd-9a57-4cbc-adfa-cd0fd1c0329f.png scores medium2,line3,light4,palette4,FAIL. Raster scorecards are model outputs, not numeric measurements; include their generation calls in review cost, separate from asset tries.

## D7 — Freeze numeric calibration before production outcomes
Context: the six anchors all pass the initial LOO-calibrated thresholds; initial environment candidates often fail Lab distribution/saturation.
Choice: retain the thresholds, all failed candidate records and original try caps. Do not recolour/quantize candidates or retune thresholds to improve acceptance rates.
Reason: this rehearsal measures sustainability under a fixed style test; a green rewritten test would invalidate C2.
Evidence: calibration.json; external ground/plants/buildings numeric verdicts.

## D8 — One bounded relative-reference calibration revision
Context: V1 image model rejects one locked anchor and accepts the woodcut control; numeric tests still reject all controls. A textual ideal may diverge from the actual anchors.
Choice: retain V1 results and conduct one V2 round comparing the candidate only to the actual attached anchors using a fixed relative-match rubric. Supply no expected outcomes, previous scores or candidate classification. Keep numeric thresholds unchanged.
Reason: test prompt ambiguity transparently without tuning scores to obtain acceptance.
Evidence: external image-judge-calibration-v2/PROTOCOL.md and raw scorecards. No production certification before calibration passes.

## D9 — Freeze failed first-try rate and retain bounded retries
Context: only 1/38 first environment candidates passes the numeric test, so the preregistered 60% first-try C2 bar cannot be recovered by later retries.
Choice: retain the first-try denominator and proceed with limited second attempts for ground, plants and buildings to measure recoverability. Never relabel a retry as a first attempt. Ground receives the specified mirror-blend only after two seam failures.
Reason: the trial must distinguish an unsuccessful production method from an incomplete test. A later useful candidate does not erase C2 failure.
Evidence: first-generation ledgers and frozen calibration; second-attempt ledgers remain separate.

## D10 — Apply the combined calibration literally, retain weak model discrimination
Context: V2 model scores pass all nine calibration inputs, including all three negatives; frozen numeric tests reject those negatives.
Choice: record combined calibration PASS6/6 anchors and3/3 negative rejections, alongside model-only rejection0/3. The preregistered rule is numeric AND model, not independent perfection of each signal. Accept only candidates with both signals and their content-specific checks passing. Two mirrored ground candidates meet that rule with four5s.
Reason: neither invent an extra gate nor hide weak model selectivity. Ground12/2=6.0 attempts per accepted candidate exceeds the family stop bar, so stop production. All remaining zero-yield families are stopped; no retries beyond two were used.
Evidence: IMAGE_MODEL_CALIBRATION_V2.json; art/style05/env/ground sidecars; no altered numeric thresholds.
