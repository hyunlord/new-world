# Art Pass 6 decisions

- GOAL.md is the owner's existing document and remains byte-for-byte unchanged. Its explicit branch/scope instructions override the generic lead/main reset workflow.
- Initial HEAD and remote WIP both equal ca47e703f7136a00cac83eb242fbec8dccc77d52. No local-only commits require backup. Existing .gitignore and .ignore edits are left untouched and excluded from staging.
- Merge exactly 4c35e29573bc28878de0cfed9ac67e0df0aa4c48. The sole conflict is rust/crates/sim-test/tests/harness_p14_delta_status_panel.rs; keep the art version (content assertions, no hash pin). This version contains no sha256_hex import, so no unused import remains.
- The goal is already a decision-complete execution plan. Execute its Parts directly; no additional planning approval or interview is appropriate under the explicit never-ask instruction.
- Use project-owned D3 assets and existing UI tokens. Optional generated god icons default to localized text; reserve the six-image budget for rejected carry props if needed.
