# Part 4 implementation checks

These are focused checks of the Stage B implementation above the committed Stage A verdict `44aebafa`. They do not replace frozen cohort identity, tuning, performance, held-out evaluation or the final full gate.

| Check | Result |
|---|---|
| sim-core library tests | 320 passed |
| sim-systems library tests | 142 passed |
| sim-bridge library tests | 33 passed, including five family collector tests and live/stress selection |
| sim-test library tests | 101 passed, including seven family observer tests |
| Family integration harnesses | 14 passed: matching/registration/founders 6, beds 2, births 3, fission 3 |
| Python metric evaluator tests | 42 passed: existing Stage A 25, Stage B 17 |
| Workspace all-target Clippy | exit 0, warnings denied |
| Actual Godot live-v22 API smoke | 13,563 checks; 286 activity joins; mutual pair; newborn-parent-child links |
| Static public shapes | Only the two permitted existing-public-struct rule fields added |
| Parent-slot consumers | Maternal lineage is intentional; no incompatible external reader found |

The core/system/bridge/probe commands use `CARGO_BUILD_JOBS=6`, with test threads 6. Full command shapes: `cargo test --manifest-path rust/Cargo.toml -p <crate> --lib -- --test-threads=6`; the four `harness_revitalize_pass17_family*` integration targets are selected together. Clippy uses `--workspace --all-targets -- -D warnings`. Python uses `python3 -m unittest discover -s tools/revitalize -p 'test_evaluate_pass17*.py'`.

Godot was built with `cargo build -p sim-bridge`, imported headlessly, then ran `tools/revitalize/bridge_smoke_pass17.gd -- --routine --family`. The smoke records exact raw entity bits and AgentIds. Its first newborn is a truthful early fallback; real-couple birth selection is separately tested. API execution is not new-art visual acceptance.

Two initial Clippy attempts failed only on required public API documentation (three then one missing comments). Both literal failure logs are preserved, followed by successful reruns. No warning suppressions were added. LSP remains unavailable from the earlier closed MCP transport; `basedpyright`, `pyright`, and `ruff` executables are not installed. No dependency was installed to alter the environment.

Small-seed release trace on/off and exact-day60 family checks follow the implementation commit before Part 4 closure. No Stage B tuning or evaluation seed has run at this point.
