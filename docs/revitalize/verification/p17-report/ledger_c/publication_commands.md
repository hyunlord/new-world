# Pending publication commands and receipt boundary

Canonical C is `21d2b478a81d55ac138f0a60b830e06719ec0af3`. Its gate was launched by the coordinator at 2026-10-15T11:33:38Z and is already running. Do not launch another C gate. This preparation makes no terminal gate claim.

Aggregate RUNLOG is `docs/revitalize/DECISIONS.md:872`, with subsequent Part tables. Workload-specific RUNLOG files are supporting evidence, not a separate aggregate file.

After C terminates PASS, check its finalized external receipt and SHA sidecar, then explicitly commit the intended report/doc/evidence files as R. Keep this25-commit C ledger together under a report-linked directory, for example `docs/revitalize/verification/p17_commit_ledger/`; preserve relative links. The report names tested C. It contains no self SHA, future R, final R-gate time, or future publication completion time.

Only after R is committed, run these commands sequentially (prepared, not executed):

```bash
P17_CODE_SHA=21d2b478a81d55ac138f0a60b830e06719ec0af3
P17_PUBLISH_SHA=$(git rev-parse HEAD)
git diff --exit-code "$P17_CODE_SHA" "$P17_PUBLISH_SHA" -- rust tools
git diff --name-only "$P17_CODE_SHA" "$P17_PUBLISH_SHA"
python3 /Users/rexxa/work/p17-data/run_bound_gate.py "$P17_PUBLISH_SHA" \
  "/Users/rexxa/work/p17-data/p17-final-head-gate-${P17_PUBLISH_SHA}" \
  "/Users/rexxa/work/p17-data/p17-final-head-gate-${P17_PUBLISH_SHA}.receipt.json"
shasum -a 256 -c "/Users/rexxa/work/p17-data/p17-final-head-gate-${P17_PUBLISH_SHA}.receipt.json.sha256"
```

Require the complete C→R path list to contain only intended documentation/evidence. The wrapper requires pristine tracked/untracked/ignored discoverable rust/tools, fresh output/receipt paths and exact HEAD before/after. It validates all15ordered steps, terminalstatus0 and no smoke errors. Preserve failures; never reuse output paths. Do not commit/amend after the final R gate. If source changes or a new commit is needed, bind a new exact head and new gate; do not label it R from an older receipt.

After a terminal PASS on R, refresh an external ledger including R, fill `final_publication_pass17.json`, then publish with the existing authorization. Publication commands are also preparation only:

```bash
git remote -v
git branch --show-current
git fetch origin
git merge-base --is-ancestor origin/codex/revitalize-2026-10-wip "$P17_PUBLISH_SHA"
git merge-base --is-ancestor origin/codex/revitalize-2026-10 "$P17_PUBLISH_SHA"
git push --atomic origin \
  "${P17_PUBLISH_SHA}:refs/heads/codex/revitalize-2026-10-wip" \
  "${P17_PUBLISH_SHA}:refs/heads/codex/revitalize-2026-10"
git ls-remote origin refs/heads/codex/revitalize-2026-10 refs/heads/codex/revitalize-2026-10-wip
```

Require both returned remote SHAs to equal the gated R; record actual UTC/epoch and output hash in the external receipt. Update PR510 through an exact body file, then read the final PR state/link. Publish report text with a separate appendix naming R, both gate receipts, final ledger SHA and actual completion times. These completion facts live externally and cannot cause another report commit.

Receipt hash order: finalize all gate logs → write stable gate receipt → hash logs and receipt in sidecar (sidecar does not hash itself). After publication/PR verification, finalize `final_publication_pass17.json` → compute its SHA for a separate sidecar and final message/appendix, never as a self field. The PR body can link the stable receipt path but must not contain its SHA when the receipt hashes that PR body. The receipt records only the appendix path and deliberately does not hash the appendix. Its linked C ledger remains immutable; later R ledger is external. Current pending values remain null.

The operational helper is unchanged from its27fixture readiness proof: SHA256 `f1158c2943f291d44a1ac3f01a514245bc4793fee60f6a0cb609024e925f436c`. Gate script SHA256 `81f30f2e0b70c53ea745628886c1379273810493f200d573321530ae30ba5b65`. A read-only check at51e found clean source guards and the gate matching its committed blob;51e→C has no rust/tools difference. These checks are not actual gate results.
