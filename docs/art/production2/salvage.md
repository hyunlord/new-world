# Art Pass 2 salvage

World scale: adult 28, child 21. Round 1 fitting-damage trigger: 0/128 villagers; escape not activated.

**143 generated cells + 4 explicitly derived cells pass visual and technical checks; 109 original slots remain empty.** A visual ok alone is not installation approval.

| Sheet | Raw accepted | Derived | Empty | Remaining reasons |
|---|---:|---:|---:|---|
| F1 | 8 | 0 | 8 | no coherent guard-passing selected candidate: 8 |
| F2 | 9 | 0 | 7 | detail_destroyed_by_fit: 1; no coherent guard-passing selected candidate: 6 |
| N1 | 15 | 0 | 1 | artefact: 1 |
| N2 | 14 | 0 | 2 | no coherent guard-passing selected candidate: 2 |
| N3 | 10 | 0 | 6 | no coherent guard-passing selected candidate: 5; artefact: 1 |
| S1 | 16 | 0 | 0 |  |
| S2 | 13 | 0 | 3 | no coherent guard-passing selected candidate: 3 |
| U1 | 11 | 0 | 5 | no coherent guard-passing selected candidate: 5 |
| V_a1 | 3 | 0 | 13 | artefact: 9; pending_cycle_completion: 4 |
| V_a2 | 1 | 1 | 14 | artefact: 6; pending_cycle_completion: 5; broken_anatomy: 1; no coherent guard-passing selected candidate: 2 |
| V_a3 | 5 | 1 | 10 | artefact: 5; pending_cycle_completion: 1; region_map_wrong: 3; no coherent guard-passing selected candidate: 1 |
| V_b1 | 10 | 2 | 4 | no coherent guard-passing selected candidate: 1; region_map_wrong: 3 |
| V_b2 | 4 | 0 | 12 | no coherent guard-passing selected candidate: 11; artefact: 1 |
| V_b3 | 14 | 0 | 2 | region_map_wrong: 1; no coherent guard-passing selected candidate: 1 |
| V_c1 | 4 | 0 | 12 | no coherent guard-passing selected candidate: 6; artefact: 3; region_map_wrong: 2; pending_cycle_completion: 1 |
| V_c2 | 6 | 0 | 10 | no coherent guard-passing selected candidate: 3; pending_cycle_completion: 3; artefact: 2; region_map_wrong: 2 |

[Accepted contact sheet](accepted_contact.png)

## Evidence

- [Original candidate ledger](candidates.json) and [round 1 authoritative verdicts](review/verdicts.json).
- [Round 2 candidate ledger](candidates_round2.json), [authoritative verdicts](review/round2/verdicts.json), and [adjudication](adjudication_round2.json).
- [Final explicit slot decisions](final_selection.json). Original catalogs and Production1 documents are unchanged.
- Two visual rounds only. Region masks were corrected once; eleven second-round region objections and all other remaining rejections are retained.
- A static review found stale grid/factor provenance on derived frames. The fix preserves source metadata and warnings; all 30 derived round2 records independently checked. Original round1 evidence remains immutable.
- Python suite: 97 tests passed. These checks establish asset-pipeline behavior, not in-game activity readability or visual acceptance of the final installed world.

## Review provenance

- Final candidate ledger SHA256: `b9b265abde5084727b02c016f7b616a27209d0796e7c3ab58f4c2fb6bdf36854`.
- Final verdict SHA256: `f59af54c4d0531b00b305ba4a040239497451985f416356d4e62fa890b3be830`.
- The initial object-review transcription correction is retained as `review/verdicts_objects_round1_unvalidated.json`.
- No new image generation in Parts 0–2; budget remains 18 calls.
- `derived` never becomes a claim that pixels were newly generated or independently sourced; lineage stays explicit.
