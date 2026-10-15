# Pass17 gate formatter synthetic proof

This is a persisted rerun of the 38 previously transient synthetic checks. It does not establish a real gate verdict. All fixture HEADs, counts, times, script hashes and logs are fabricated. The fake gate script path is never executed.

- Formatter SHA256: `da035c56e5b052fe6d9a5d26ec95d3f14b53ae4303bd18d66d4413b88d0e1874`
- Execution UTC: 2026-10-15T11:59:11.049012Z → 2026-10-15T11:59:11.413996Z
- Outcome: 38/38 checks PASS.
- `validation.json` contains case outcomes, actual exceptions/CLI output, source hash and artifact hashes.
- `input_snapshots.json` preserves receipt, gate, sidecar and mutated-file bytes for every case.
- `fixture/` is restored to the valid input used for the successful CLI summary.
- `summary.json` and `summary.md` are actual formatter CLI outputs from case 35.
- `checksums.sha256` binds all proof files and the validation receipt, excluding itself.

The original temporary run required resolving the macOS /var alias to /private/var in its fixture path. The path guard was preserved; this rerun resolves the fixture root before construction.

Reproduce to a fresh external directory:

```sh
python3 /Users/rexxa/work/p17-data/summarize_pass17_gate_validation/validate_formatter.py /Users/rexxa/work/p17-data/formatter_validation_NEW
```
