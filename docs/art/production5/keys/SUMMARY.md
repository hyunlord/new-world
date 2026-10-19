# Part 4 generation and checkpoint

Two of eight permitted image calls were used. Both returned1254×1254 PNGs, stored outside git under `/Users/rexxa/work/art-production5-raw/`; `image_calls.json` records file hashes and generator provenance.

| Sheet | Attempt | Visual approval | Key-clean | Result |
|---|---:|---:|---:|---|
| V_a6 | 1 |12/16|8/16|STOP key generation: fewer than12 clean cells|
| V_c4 | 1 |10/16|not applicable|Child exception allowed; no retry qualifies|

The fresh reviewers directly opened all four boards for each sheet. Candidate verdicts and all rejected cells remain in `processed/`. The first checkpoint is frozen in `checkpoint_first_key.json`. Adult_b's reference idle did not receive an installed approved new mask, so its prepared prompts use the documented natural fallback; no adult_b generation was made after the key stop.

The V_a6 initial conversion carried obsolete `conversion_failure` state despite successful re-fitting. A separate `candidates_reguarded.json` records corrected guards, retaining the original ledger and identical112image hashes; this correction does not change key counts, pixels or visual verdicts. The child conversion uses the corrected code and has no conversion failures.

The complete-set search in `keys/harvest_validated/search.json` found zero eligible new groups. Two V_a6 cells passed all frame guards; six V_c4 cells passed frame guards, but approved complete physical animations could not be formed. No generated art was installed, and no guard or review threshold was relaxed. Existing Part1/3 gains remain installed. Retrying is limited to sheets with fewer than half their cells approved; neither qualifies.

The independent pipeline review exposed approval-binding weaknesses. Part4 hardening preserves key/conversion hard errors, binds normalized candidates to original approved pixels, binds ordered animation candidates and board/GIF hashes, and compares actual final sprite/native/mask content against immutable ArtPass4 Git blobs. Mutation regressions and final A20 evidence are recorded in the harvest audit.

The child reviewer used numeric candidate_id=1 with correct unique sprite IDs. A separate reviews_normalized.json maps each exact sprite ID and attempt to the ledger ID without changing any verdict. The original reviews and first unbound search remain preserved; harvest_validated is the authoritative search.
