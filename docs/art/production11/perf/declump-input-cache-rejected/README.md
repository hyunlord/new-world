# Part 3 — declump-input-cache: REJECTED

**H5 candidate identity PASS; performance REJECTED; no candidate adopted.** Candidate commit: `1dc641b5cd161cf9683349f367194935e8213b3f`. The leader restores the pre-candidate renderer and seat implementation; the preserved source here documents the rejected experiment, not installed behavior.

P0 section timing selected declump first: median 6,676.5 µs, above pose (2,550 µs) and post-loop (2,112 µs), renderer total 24,923 µs. The experiment memoized the pure declump function's complete inputs on a renderer-owned instance. Static declump and downstream easing, metadata and walker updates remained live. Exact rows, targets, all rects, terrain cells, blocked cells and spatial mode determined reuse; input/output snapshots prevented aliasing. No tick-only cache or off-screen skip was introduced.

## Alternating release result

Three alternating pairs, 150 frames per block, W500 frozen Z4. All blocks have 150 samples; aggregated control/candidate each have 450.

| Pair | Control median ms | Candidate median ms |
|---|---:|---:|
| 1 | 39.1445 | 37.8680 |
| 2 | 63.8485 | 65.8330 |
| 3 | 64.7850 | 65.9935 |

| Aggregate primary CPU | Control ms | Candidate ms |
|---|---:|---:|
| Median | 62.8585000098 | 63.9470000000 |
| p90 | 67.5499999943 | 70.0980000224 |
| p99 | 83.7340000000 | 86.5870000391 |
| Maximum | 119.2989999723 | 133.3989999666 |

Median speed-up = **0.9829780913846646×**, so the candidate was slower and fails §7.1. The first pair's improvement does not override the complete three-pair result.

The session validator reports `valid: true`, session exit 0 and unchanged before/after provenance at the candidate commit. Release/debug bridge hashes match: `7752d9ce994da64b08887c961fdf006153fae526dcad06526e03aed84e65df7c`. This was a **contended hold** after 3,602 seconds waiting, 2026-10-16 15:20:34–15:21:51 UTC (77 seconds), load 111.32 before / 51.64 after. These machine conditions and block drift are retained as limitations; the candidate is rejected without claiming a quiet-host speed estimate.

`summary.json` contains `pixel_identity: NOT_VERIFIED` because the standalone pair validator sets that field without ingesting identity results. It is preserved unchanged. The actual completed identity comparisons below establish candidate H5 PASS independently.

## Identity and functional checks

- Six framings P1–P6: A/A′ and B/A have **zero changed pixels**, all RGBA hashes match. P5 is exact zero. Complete render-event sequence comparison passes.
- Running W0 and W200: 600 frames each, fixed 60 FPS, both PASS.
- W500 pan: frozen and 2×, 240 frames each, both PASS.
- Build/import, six-framing driver, four running cases, three comparison tools and paired session all exit 0. Pair launch source/build provenance is unchanged before/after.
- Attempt 2: strict GDScript plus six focused/regression tests all exit 0 with `errors: false` (seven checks).
- The initial missing-API RED and first fixture's `SCRIPT ERROR`/resource-leak output are retained. The first fixture error remains a failure despite its process exit 0; fixture cleanup was corrected before attempt 2.

No candidate hit-rate diagnostic was measured. Functional tests demonstrate exact-input hits and mutation invalidation; they do not establish production cache hit rate or acceptance.

## Evidence index

- [Raw evidence ZIP](raw-evidence.zip): 170 source files, 172 members including manifest and SHA256SUMS; 7,126,337 bytes. SHA256 `df49b17e75becac0e1251ebf73276375a17993821d19bde63de89326f66b701a`.
- [Full member manifest](manifest.json), [SHA256SUMS](SHA256SUMS), [archive validation](archive-validation.json). Every member was read back and SHA256-checked, ZIP CRC passed, and source bytes remained unchanged during packaging. Repository copies match external files exactly.
- Reference A/A′ and running-A remain in [the separate reference archive](../identity-reference/reference-raw-evidence.zip), SHA256 `fbdb1675f8b79e7817a5dfd34d5b3712e3fda4124f3d9f0c37be2598e0e9a2b2`; its SHA and CRC were rechecked. The candidate manifest binds this reference archive.

Inside the candidate ZIP, original evidence-relative paths are preserved:

| Path | Contents |
|---|---|
| `p3-candidate-tests/` | RED, failed fixture attempt, attempt2 seven checks, generated renderer and runner |
| `p3-runtime/candidate1-build/` | Release build/import logs, exit codes, bridge hashes |
| `p3-runtime/candidate1/` | B six captures, running/pan dumps, source snapshots and per-case provenance |
| `p3-runtime/candidate1-comparison/` | Clock, pixel and running comparisons; unchanged exit records and diff images |
| `p3-pair-declump-input-cache/` | Complete launch provenance, session output, frames and summary |
| `p3-runtime/base-instrumentation/` | Exact baseline instrumentation patch and seven source files |
| `p3-candidate-package/rejected-source/` | Full renderer, CampSeats and candidate test retrieved via `git show` at candidate SHA |
| `p3-candidate-package/candidate-commit.patch` | Full candidate commit patch |
| `p3-candidate-package/holds-terminal-snapshot.md` | Immutable holds snapshot taken after paired-session termination |

External final archive: `/Users/rexxa/work/art-production11-evidence/p3-candidate-package/raw-evidence-v2.zip`. Earlier external `raw-evidence.zip` is retained as a superseded packaging draft: its manifest described the standalone identity field imprecisely as an unsupplied optional argument. V2 corrects that explanatory sentence only; the original experiment artifacts are unchanged.
