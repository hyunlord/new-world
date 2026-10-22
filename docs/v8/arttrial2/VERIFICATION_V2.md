# TRIAL2b final verification

Technical execution completed; art acceptance remains partial. This record does not convert C5/C7/G1 failures or carried C4 NOT_MET into passes.

| Check | Result | Evidence |
|---|---|---|
| Python suite |19tests +32subtests PASS,11.68s|external `trial2b/final-pytest.log`|
| Changed Python Ruff |PASS|explicit14file invocation; import-order issue fixed before final run|
| Production typing |0errors,331warnings; strictNOT_GREEN|`final-production-typecheck.log`, dependency venv selected explicitly|
| Original six anchors |6/6SHA unchanged|manifest_v1 original identities|
| Runtime parity |11captures match current inputs and own screenshot hash|VERIFICATION_V2.json;50or66input hashes per capture|
| Published environment |35PNG/raw/working/model/reference/calibration/acceptance joins verified|ENV_ACCEPTANCE_V2.json and sidecars; independent audit|
| Calibration |12/12LOO,3/3personcontrols reject|JUDGE_V2_CALIBRATION.json; correlated/control-content limits retained|
| File cap |all changed/new scoped files≤1,500,000bytes|final manifest audit|
| Art growth from explicitTRIAL2b base |25,253,080bytes≤40,000,000|base034c38ab art52,740,912bytes; current77,993,992|
| Real window200allwalk |60.002084s,5294draw,88.230FPS|RUNTIME_V2.md|
| Readability |two initial reviewers+one revision+two fresh reviewers; FAIL retained|READABILITY_V2_ROUND1/2|
| Scope |onlyart/** anddocs/v8/arttrial2/** committed|per-commit explicitpath lists|

The first broad typecheck used an isolated tool environment without Pillow/NumPy and reported27errors/709warnings; that invalid environment result remains external `final-typecheck.log`. The corrected production-only invocation uses `--pythonpath /Users/rexxa/work/arttrial2-data/.venv/bin/python`. Its0errors do not mean warnings disappeared or the test files were strictly typed. Ruff initially found one import-order issue in people_v2; corrected import now also works as a module. An initial staged whitespace check caught a trailing blank line in accept_types_v2; corrected before commit. No failed run was relabeled passing.

Actual Godot import and11final window runs are the standalone proto surface. This is not the full game gate, game determinism/identity, remoteCI, device acceptance or human art approval. RenderingServer draw count is not displayHz.

Independent reviews found and recorded: source-snapshot wording for floors, visible dark floor boundary, unsupported east/west female-child head combination, stale pending promotion text. Evidence prose was corrected; no frozen people pose or runtime asset was changed after round2. New quiet grounds remain combined-styleFAIL despite model6/6and tone/seam6/6.

Raw logs/captures remain external. `VERIFICATION_V2.json` is the compact provenance summary, not raw measurement data. The unrelated pre-existing `.gitignore`, `.ignore`, `docs/art/` changes are not staged.

Publication: Parts0–4 pushed individually with exactremoteHEAD checks. Part5finaldocument commit is verified immediately after push and linked in the final response; its own SHA is not embedded recursively in this file.
