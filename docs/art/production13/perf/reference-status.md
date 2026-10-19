# Reference provenance

The original tooling commit was `c1b1358e5d443f2eff11733b0ca8136df32a7d00`. Before any performance session, an audit corrected admission retry/wait accounting and ablation coverage/provenance in `6d493fa8ade6d368eca8e7efa7ab3f668eea1827`. This corrected commit is the definitive Part 0 tooling baseline. Both detached worktrees now point there. The correction changes no production simulation or presentation behavior.

## Preserved preliminary references

External root: `/Users/rexxa/work/art-production13-evidence/identity`.

- `six/A` and `six/Aprime`: all six launches exit0, no log errors, source/build provenance unchanged. `preliminary-control/comparison.json` reports zero differing RGBA pixels and zero maximum channel difference for every A/A′ framing. Its B input deliberately repeats A; that column is not candidate acceptance evidence.
- `running-reference`: W0, W200 and W-multi each600 frames; W500 frozen/2× pans each240. All launches exit0, no log errors, provenance unchanged. `running-reference-completeness.json` validates schedule, row counts and hashes by self-comparison only. It does not prove independent candidate identity.
- Census in `derivation/all.json` and `framing-derivation.json` remains valid: the correction leaves census and simulation source unchanged.

## Corrected references

`corrected-six/A`, `corrected-six/Aprime`, `corrected-control`, and `corrected-running-reference` were independently collected on the definitive tooling commit. Every launch exited0 without log errors or provenance changes. All6 A/A′ RGBA differences are zero. All3 running600-frame and both240-frame pan captures pass completeness validation. The B column and running self-comparison remain integrity checks, not optimization acceptance. Additional tooling-change-pixel-check and tooling-change-running-check compare the preliminary and corrected commits: all exact; production behavior unchanged.

## Builds and gate

Both actual corrected-commit bridge builds and imports exited0. Measure release and copied debug dylibs both SHA256 `b4d29cf526676adc3fa77f21454d2bee79c7865a08b32f398c975e7d8bf6a9f6`. Build/import logs and hashes are under external `builds/*-corrected*`.

Full Part0 gate `gates/p0/results.tsv`:116 checks, all exit0, summed check time1300s. Rust tests911s, Clippy98s, strict GDScript12s; both real-input suites pass. Because measurement-only corrections were made while later gate checks were running, supplementary fresh verification is retained separately:42 p13 Python tests, corrected collector `--check-only`, shell syntax and diff whitespace. This is a technical gate, not an H1–H10 acceptance result. First performance attempts were subsequently refused; see baseline-admission.md. No measured frame evidence exists yet.
