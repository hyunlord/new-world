# Bound Pass17 gate runner readiness

**READY_FIXTURES_PASS_ACTUAL_GATE_NOT_RUN**. No actual gate, Cargo, Godot, probe or evaluator was launched.

Helper: `/Users/rexxa/work/p17-data/run_bound_gate.py`

SHA256: `f1158c2943f291d44a1ac3f01a514245bc4793fee60f6a0cb609024e925f436c`

[Detailed readiness](run_bound_gate_readiness.json), SHA256 `cc4155e78cd0c51557f55a4a61c662a0e4462d087dbd34a4b747b1cbef409e7c`.

The helper runs exactly one invocation of the unchanged `tools/revitalize/gate_pass17.sh`. Root launches C and R separately; this helper never commits, stages, publishes, acquires a quiet hold, or starts another workload. The gate retains its internal foreign-lock checks.

## Interfaces

```text
python3 run_bound_gate.py EXPECTED_FULL_SHA OUTPUT_DIR RECEIPT_JSON [REPO_DIR]
```

The optional repository defaults to `/Users/rexxa/work/new-world-pass4`. The SHA must contain exactly 40 lowercase hexadecimal digits. Repository output is restricted to `docs/revitalize/verification/`; the receipt must be outside both the repository and the output directory. Output must not exist. Both the receipt and its `.sha256` sidecar must not exist. Early invalid arguments/collisions return 2; a preserved failed attempt returns 1; verified terminal PASS returns 0.

Receipt and sidecar are exclusively reserved as empty files during the run, then populated after termination. Consumers must require valid terminal JSON and the completed sidecar. The wrapper never treats an empty reservation as completion.

## Canonical C gate — launch only when authorized and committed

```bash
C=<full_committed_C_sha>
python3 /Users/rexxa/work/p17-data/run_bound_gate.py "$C" \
  /Users/rexxa/work/new-world-pass4/docs/revitalize/verification/p17_gate \
  "/Users/rexxa/work/p17-data/p17-gate-${C}.receipt.json"
```

After a green C gate, root explicitly commits the intended report/evidence-only changes as R. Existing output or receipt paths require a new preserved destination; the helper never erases a previous attempt.

## Exact publication R gate — separate later invocation

```bash
R=<full_committed_R_sha>
python3 /Users/rexxa/work/p17-data/run_bound_gate.py "$R" \
  "/Users/rexxa/work/p17-data/p17-final-head-gate-${R}" \
  "/Users/rexxa/work/p17-data/p17-final-head-gate-${R}.receipt.json"
```

No later commit/amend may be labeled as the tested R. The external receipt binds before/after HEAD, source snapshots, gate script SHA before/after, wrapper and gate invocation UTC/epoch boundaries, gate process exit, signal state and every final output-file hash. Its `.json.sha256` sidecar hashes those output files plus the finalized receipt, without hashing itself. The committed C report does not contain its own future R hash.

## Guard and validation contract

- Before and after: exact expected HEAD, no staged/unstaged tracked `rust`/`tools` changes, no untracked files in those trees, and no ignored source/test files of the checked extensions outside generated `rust/target` and `__pycache__`. The configured source suffixes are retained in the helper. Intentional root `.gitignore`/`.ignore` and historical docs evidence are outside these source guards.
- Before: actual gate bytes equal the expected commit's gate blob. After: gate SHA is unchanged.
- Terminal evidence: exactly the 15 expected step exits in original order, all zero; matching gate HEAD; one `smoke_errors none`; empty `smoke_errors.log`; all 15 step logs; one final `finished ... status 0`; no `wait_failed`; process exit zero.
- Any failure preserves generated files and a FAIL receipt. SIGINT/SIGTERM terminates only the process group created by this invocation, with SIGKILL escalation after five seconds if necessary.
- `perf_quick` remains untimed validation; the receipt explicitly sets `perf_quick_is_timing_evidence=false`.

## Completed lightweight validation

27 checks passed using plain disposable directories, fake Git and a harmless gate substitute; no Git checkout was created. Covered valid terminal logs, missing/duplicate/nonzero steps, wrong HEAD, missing/duplicate/failing terminal markers, smoke errors, missing step log, all preflight source guards, generated-cache allowance, existing output/receipt preservation, nonzero gate exit, HEAD/source/script changes during the fake gate, complete sidecar hashes, timestamp ordering, and interrupted owned-child cleanup with an unrelated process surviving.

AST syntax parsing passed. The programming static checker reported `no violations in 1 file(s)`. The helper has 216 nonblank/noncomment lines and uses no additional dependencies. Actual full-gate runtime remains untested until root launches the authorized C/R runs.

A read-only invocation of the source snapshot guard also passed in the actual checkout at `32e2a009966d6d429a62282897241510400dbc45`. It did not invoke the gate.
