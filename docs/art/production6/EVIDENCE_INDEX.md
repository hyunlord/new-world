# Art Pass 6 evidence locations

- Final original framebuffers and full JSON: `/Users/rexxa/work/art-production6-evidence/shots/{art,art_long,art_god,calibration}`. Four manifests bind to `44011f92347dc4176cc115195b1cc55a16ff93ae`.
- Before art edits: `/Users/rexxa/work/art-production6-evidence/baseline/shots`; repository summaries under `baseline/`.
- Rejected capture attempts: external `rejected/capture_attempt1` and `rejected/capture_attempt2`.
- First screen/blind round: external `round1/`; immutable original observer outcomes under `blind/round1/`.
- Frozen final blind answer key: external `blind_key_round2.json`; repository key hash under `blind/key.sha256`.
- Original measurement failures before schema correction: external `measurement_round1/`.
- Independent identity images and13-sample cycle contacts: external `review/identity/` and `review/cycles/`; source hashes under `review/board_sources.json` and `review/cycle_contacts.json`.
- Warm-control cycle GIFs moved byte-for-byte for budget: external `published_controls/`; receipts under `review/budget_relocations.json`. Natural cold cycles remain in `shots/`.
- Full gate attempts are retained separately under `gates/final_attempt1`, `gates/final_attempt2`, and `gates/final` (last attempt). Source compatibility audit under `gates/p9_static/`.
- Render/runtime source fingerprint: `review/source_fingerprint.json`; forbidden-path audit: `review/scope_audit.json`.

External evidence is local to this Mac and is not uploaded by the PR. The repository carries compressed entity snapshots, review boards, required comparisons/GIFs and hashes. A path/hash is not a claim that external raw files are available to remote reviewers.
