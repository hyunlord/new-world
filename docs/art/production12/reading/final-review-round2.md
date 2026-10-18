# Art Pass 12 independent final review — round 2

Review target: `c08076dd67ee0a54470f16fc6b993c3172752320`. Read-only source/evidence review; no source changes, builds, Godot runs, or child agents. GOAL sections3–7 and prior reader answers were not read. Final fresh-reader ledger and terminal full-gate results were subsequently inspected; closeout below is final for review round2.

## Final verdict

**ACCEPT the inspected finished implementation and honest evidence classification; REVISE / UNFINISHED against the complete requested acceptance bar.** No new blocking implementation defect was found in the inspected scope. The outstanding performance/readability criteria below are acceptance failures or missing evidence, not invented code defects. Frozen source must stay frozen.

## Findings and remaining acceptance gaps

- **High — requested bar unverified, not a measured regression:** `docs/art/production12/perf/after.md`, `after-admission-index.json`, and E/p4-matrix plus E/p4-sections-admitted-* preserve 21 failed admission holds. I directly inspected all15 matrix and6 section validity JSONs: every record has valid=false and exit75; matrix snapshots show foreign Python39003 above50% at both ends. No Godot measurement was admitted. H1–6 remain NOT_VERIFIED(contended), numerical before/after values NOT_MEASURED. Outer quiet-lock contended=false is not admission validity and must never be used as a pass. Section command errors before admission do not manufacture extra valid sessions.
- **High — requested readability bar failed:** H8 remains3/10 diagnostic answers plus4missing, not a10-item substitute for14. `activity/protocol.md`, E/p2/activity-attempt1/blind-diagnostic/protocol-status.json and terminal-village-disposition.json retain the shortage. Village reading is NOT_RUN:2eligible pairs of6. No lowered threshold or invented pairs were found. I deliberately did not inspect prior answers.
- **High — H9 requested readability bar FAIL:** final fresh attempt2 ledger records12 incorrect name assertions and7 unsupported construction claims. Full technical gate passed102/102; it cannot substitute for reading acceptance.
- **Low — historical commit-format deviation:** `DECISIONS.md` explicitly retains the missing p3 prefix on33b8f7c7/c46e66bd. This does not alter runtime behavior, but it is a real process deviation.

## Verified implementation and provenance boundaries

- `bf85dac60579dd4a9135dd9f6f92a3139c8d1f6e` has parents baseline `b83dc5aef50505217c9fc3cdf33dee87b4175a46` and simulation `6e1446b4d6af41e13e2b221b17609751510f046f`. The art-owned diff was reviewed after that merge, not misclassified as simulation changes. Its paths remain in scripts, tools/art_pipeline, docs/art, localization and the specifically allowed Rust kernel/FFI/fixture/static repin. No gdextension change appears. Existing .gitignore/.ignore remain local.
- c46e66bd→c08076dd changes only DECISIONS/TASKS documentation, so corrected native capture source and frozen product source agree. `after-build-provenance.json` identifies the actual release build/import and equal release/loaded debug-path SHA256 `1e6f76187e9e0d87e87426fe2b0aa510c3c797efd61b9cb14fed383531d2bf91`. Runners enforce profile, checkout/head, library and collector hashes per session; this establishes identity, not measured performance.
- `RELATIONS_CONTRACT.md`, SimReads, porter presentation and chronicle code agree on getter types and identity domains. Porter rows join by entity bits; relation/porter getters are tick-cached with reset invalidation and missing-getter fallback. Carry uses actual inventory, destination depends on outbound/return stage. Chronicle18 uses receiver settlement and sender settlement2;17 uses sender/receiver;19 handles viewed-other reversal;20/21 omit person prefixes. Grouping requires consecutive18 rows with identical tick/settlement/settlement2/cause; unknown>=22 are skipped/debug-counted.
- `art_kernels.rs` retains f32 vector/rectangle operations before widening, half-away rounding, signed-zero bit checks and ordered comparisons. Its real fixture test requires>=4000 records,>=2000 unique observed provenance calls, both overlap modes,>=200 calls in each of five worlds × two conditions, and exact output IDs/bits. Independently hashed the57,406,240-byte fixture: `f69fb677667ad16849a0590a5bfeecf4d3171e2f69f356d7b1f866883548fd58`. Recorded identity passes remain separate from speed. Kernel stays default-OFF; no accepted optimization is claimed. Candidate(b) is correctly inventory-only/NOT_IMPLEMENTED.
- The static harness repin adds only the exact P12 capture helper path and preserves safe-counter dispatch checks; `docs/art/TEST_LEDGER.md` records its previous failure and rationale. Localization adds exactly15 keys per EN/KO compiled strings with zero existing string changes.
- Person display word joiners are removed by plain(), including copy handling. Font evidence records zero pixel change for joiners and distinct ti/di glyphs. The windowed real-input attempt3 uses Input.parse_input_event, cycles25→56→55→25, follows dead parent and chronicle links, scrolls to the last line, closes records, and reports no visible Hangul after English switching. This is application input-route coverage, not an unassisted human OS-device test.
- Blind tooling freezes exclusions before image extraction, preserves native crop pixels, and requires frozen activity answers before village preparation; terminal village disposition contains the activity-answer hash. Geometry/exclusion and terminal shortage records are retained. Reader independence and answers-before-key are protocol/provenance claims rather than pixel checks; pending fresh reader remains separate.

## Direct visual inspection

Viewed actual E/p3/live-attempt2 PNGs: multi/porter_life_en_page_000.png, porter_life_ko_page_000.png, porter_road_z1.png, porter_road_z4.png; estranged/estranged_villages_z1.png; crowd1/crowd1_en.png and crowd3/crowd3_ko.png.

The former generic porter target blocker is **cleared**: English explicitly reads Target: Village1 and Korean 목표:1번 마을, matching each displayed carrying destination. Both full panels are readable in the inspected samples. Porter pack/carry glyphs and selection ring are visibly present on road captures; estranged view visibly contains a red dashed link and Korean label. Crowd samples show icons/body overlap in the scene; I do not infer blind activity recognition from their presence. These seven images do not substitute for exhaustive140-image visual review or H9 reader scores. E/p3/attempt2-audit reports140PNG provenance with no issues, which is provenance/text-coverage evidence, not universal visual legibility.

## Closeout

Final reader, replay audit, and technical gate have been received and inspected. H1–6 remain NOT_VERIFIED(contended), H8/H9 FAIL, village NOT_RUN, and candidate optimization unaccepted/default-OFF. Preserve this outcome as UNFINISHED against the full requested bar; no further implementation changes are authorized by this review.

## Fresh-reader and replay update

After leader notification that all140/140 answers were frozen (SHA256 `36c0bdfe5f3e3f646050b28b61e243fbb2c8ab6774996831b21575ad5de8ea04`), I read the fresh narrative. It reports no raw localization keys, malformed joiners, or internally split names. It explicitly reports clipped preceding/following rows at intermediate scroll boundaries and uncertainty identifying the exact selected crowd sprite/hand action. These remain visual limitations; full scroll-range coverage does not repair them.

Leader relayed independent scoring preliminary H9 **FAIL**:12 asserted name errors (Oemu→Demu on9images; Hahima→Hanima, 하히마→하니마, 소기루→소가루 once each). Seven construction=0 claims lack the construction getter in raw reference and remain unsupported, not confirmed false or PASS. Crowd activity3/3 and corrected porter target Village1 are successful bounded results. **High acceptance failure:** final readability is UNFINISHED; do not retune or run an unauthorized third attempt. Final per-image ledger E/p3/reading-score-attempt2.{json,md} was subsequently read: all140 observations are accounted for, with exactly12 errors across12 images and7 unsupported claims across7 images. The frozen answer SHA matches. This confirms H9 FAIL; crowd3/3 is a separate bounded pass.

I inspected resource-replay-audit.json. It preserves original false comparison flags, compares numeric values by exact decimal normalization without tolerance, keeps booleans typed separately, verifies equal source manifests and bridge, and records matching clocks/tallies/world-info. This supports separate berry corroboration only; it does not retroactively add resources to original capture-time evidence.

The final gate completed on docs-only followup `9eba38ffae8095b5f108722cd120a1a71e87aa59`. Its diff from c08076dd contains only production12 documentation/evidence, so frozen product source remains unchanged. Terminal evidence is now verified below.

Intermediate-scroll fragments are preserved as a visual limitation, not conflated with the independently tested top/bottom extreme-layout requirement. The requested-bar failures are H8/H9; no unsupported construction assertion is promoted to a factual mismatch.

## Final technical evidence and decision

`docs/art/production12/gates/final/terminal.json` records exit0 at2026-10-18T12:07:50.901171Z, head9eba38ff and frozen productc08076dd. I independently counted102 TSV rows, verified all102 exit codes are0, and verified results SHA256 `dee4f57e04f69b096d9d3dae43a9ee194b615da0fe0f585f84c6f1fefc82d356`. Terminal failed/script-parse-error arrays are empty. This closes the pending technical gate, including workspace tests, Clippy, strict, bridge, smoke, Python and individual Godot checks. No new runtime was launched by this reviewer.

**Final: ACCEPT finished code/evidence reporting as bounded above; REVISE / UNFINISHED for full Art Pass12 acceptance.** No new blocking implementation defect was found; requested performance evidence remains unavailable and reader acceptance fails. Name legibility, exact crowd identity uncertainty, intermediate scroll fragments, missing construction reference, and absent live18/21/broader cause coverage remain explicit risks. Do not relabel them as passed because102 technical checks passed. Review round2 is complete; no third review/retune/capture is implied.
