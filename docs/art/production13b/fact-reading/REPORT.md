# Frozen fact-reading aggregate

**Fact bar: PASS within the scored, independently verifiable non-name scope.** All scored non-name facts were correct; zero verified incorrect claims. Screen-only matches, abstentions and inferences are excluded. Image-based activity recognition remains **NOT_ESTABLISHED**. This does not establish the complete H9 gate.

The four ledgers cover **161 images** (119 + 6 + 36), with answers frozen before reference access. This report aggregates their existing visual/data adjudications; it is not another blind reading. The rule is production13b GOAL §5.5 and production13 GOAL §7.2: every verifiable non-name statement correct, names reported separately, unverifiable claims reported but not scored.

## Non-name evidence

| Slice | Images | Independently verified correct | Verified wrong | Screen-only, not independently verified | Excluded material |
|---|---:|---:|---:|---:|---|
| [Part1a](part1a-score.md) | 65 | 665 | 0 | 127 | 8 abstentions; 2 inferences |
| [Part1b](part1b-score.md) | 54 | 595 | 0 | 4 | 54 qualitative annotations and 16 viewport observations outside ledger |
| [Part2](part2-score.md) | 6 | 117 | 0 | 14 | 6 picture-inference entries; artifact observations outside ledger |
| [Part3](part3-score.md) | 36 | 624 | 0 | 68 | 7 picture-inference entries; artifact observations outside ledger |

**No pooled claim denominator or success percentage is reported.** Part1a/1b bundle events; part1a counts relationship clauses and includes artifact/visual observations in its ledger, while part1b puts qualitative/viewport annotations outside it. Parts2/3 use atomic scalar fields, with day and event kind separately counted. Repeated screens and EN/KO readings are repeated occurrences, not independent samples. The table preserves those units.

Construction and berry counts lack independent raw fields. Other screen-only assertions include visible glyph/color context, printed-value absence, and clipped observations. Offscreen material and motion/action inferences receive no fact credit. Daytime is corroborated in the part1a adjudication but conservatively screen-only in parts2/3; that methodological difference is retained rather than silently normalized. The six part2 images represent three subjects, and their textual activity labels do not demonstrate picture-only activity recognition.

## Names

| Slice | Written occurrences | Exact correct as scored | Correct partial/uncertain | Incorrect | Uncertain alternatives |
|---|---:|---:|---:|---:|---:|
| Part1a | 424 | 411 | 2 | 11 | 0 |
| Part1b | 617 | 586 | 0 | 29 | 2 |
| Part2 | 6 | 6 | 0 | 0 | 0 |
| Part3 | 336 | 320 | 0 | 16 | 0 |
| Total occurrences | **1383** | **1323** | **2** | **56** | **2** |

These are spelling occurrences, not unique people or independent readers. Part1a’s two partial uncertain readings are Simowa (image0010) and Nuyoe (image0017). Part1b’s uncertain alternatives are Varihi/Warihi (image101); two partial-visibility readings, Muroba (75-N8) and 누후 (113-N1), are already included in its correct count. Part1a used PersonNames validation only after freezing. Other slices align visible spelling with ID-linked rendered text; raw reference data does not independently supply generated name strings. Relationship facts remain in non-name scoring.

Consolidated confusion pairs, **expected → read** (56 erroneous occurrences):

| Expected | Read | Occurrences |
|---|---|---:|
| Awado | Avado | 1 |
| Baga | Baya | 2 |
| Bubono | Budoho | 1 |
| Buyobi | Buyohi | 1 |
| Dakoha | Dakona | 1 |
| Dedo | Oedo | 4 |
| Dehona | Dehoha | 2 |
| Dehona | Oehona | 2 |
| Demu | Oemu | 8 |
| Denoba | Oenoba | 4 |
| Dotibi | Ootibi | 3 |
| Etuba | Etuha | 1 |
| Iano | Iaho | 1 |
| Inayo | Irayo | 5 |
| Nieba | Neba | 2 |
| Nitiso | Ntiso | 3 |
| Robonu | Roboru | 2 |
| Sogaru | Sogeru | 2 |
| Tanuka | Taruka | 3 |
| Tugami | Tugemi | 1 |
| 부보노 | 부도호 | 1 |
| 부요비 | 부요히 | 1 |
| 소가루 | 소거루 | 2 |
| 이아노 | 이아호 | 2 |
| 투가미 | 투거미 | 1 |

Per-occurrence image IDs and slice provenance are retained in [summary.json](summary.json) and the linked claim-level JSON ledgers.

## Frozen reader provenance

| Batch | Images | Freeze time (UTC) | Answer SHA-256 |
|---|---:|---|---|
| Part1 | 119 | 2026-10-19T13:10:05.688040+00:00 | `e6e2f47ceba07220fb6b11375dd6a221b7ba090dd65d21bbb79b7b6ea5171311` |
| Part2 | 6 | 2026-10-19T13:11:11.128064+00:00 | `a1615ec056fbfbc8327b845a45ee862200c8e610439b62bfbde7727280ce6ec8` |
| Part3 | 36 | 2026-10-19T13:14:16.721643+00:00 | `486b6a4700ab4adc51244d7634348a41238a1b80f68d1d3e797f56e88602319f` |

All three freeze records state `reader_reference_access: false`. Aggregation rechecked all three answer hashes against their freezes and all **161 source PNG hashes** against private maps. Claim-status counts were recomputed from the four ledgers; all **56** erroneous name occurrences were regrouped from individual records. Score-document, map and freeze hashes are recorded in summary.json. Frozen answers and score ledgers were not changed.

This report changes only REPORT.md and summary.json. No Godot, source changes, builds, or commits were used. Glyph geometry, real-input acceptance, and overall H9 remain separate evidence.
