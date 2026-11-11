# B4 UI art candidate pack

This is an art-only production pack for ART-BATCH round 2. The full coverage contract is `catalogue.json`; actual source identifiers are in `runtime_inventory.json`, read from game commit `4a44336e7782f2b27615d942b5cd54dbfee89406`. `STYLE.md` fixes the medium and review criteria; `docs/decisions/0016-b4-ui-art-candidates.md` records the decision.

Images live under `art/style05/ui_b4/`, final comparison sheets under `art/boards/ui_b4/`. Raw generations, exact prompts, rejected alternatives and audit logs live under `/Users/rexxa/work/art-batch/B4/`. The Korean handoff report is `REPORT.md` in that external directory.

A4 export and sheet tooling is reused unchanged; its displayed A4 header describes tool provenance, while filenames and this pack identify B4 subjects. General vision review is distinct from the historical model judge. Candidate selection is not runtime adoption or owner approval.

## Integration boundary

`art_import.draft.json` and `bindings.draft.json` are additive drafts, never replacements for full game configuration. C2/C3 must preserve existing configuration, inspect newly landed runtime IDs against the pin, import every explicit file at the immutable source commit, wire new UI consumers and metadata, and run importer tests plus real game captures. The current game contains no consumer for the proposed decision illustrations. A 64px UI center anchor is not the world feet anchor.

The actual runtime has 25 job kinds,19 alerts,16 authored cards (one test-only),17 mood modifiers,10 steward rules,6 building categories and5 room legend roles. Metal work variants remain recipes/work-type distinctions under existing jobs. `first_kid` is livestock birth; `human_birth` is separate. No rule6 or invented smelt/cast job number is introduced. Source and proposal rows are marked separately.

Default/hover/selected/disabled and severity presentation are draft recipes. Labels stay in the UI for localization; artwork contains no text. Two retained need critical variants and mood silhouettes are authored separately; other urgency states use localized UI labels. The none sentinels have no raster. Seasonal/damage/count-bucket world states do not apply to this UI family.
