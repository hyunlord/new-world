# Reference-only validation

Status: PASS. Candidate identity / H5: **NOT_MEASURED**.

Offline validation of actual A/A′ clocks and RGBA, plus running-A structure and recorded provenance. No runtime or source changes.

| Framing | Clock exact | RGBA pixels / max | Manual / automatic draws | Result |
|---|---|---|---|---|
| P1 | True | 0 / 0 | 131 / 35 | PASS |
| P2 | True | 0 / 0 | 131 / 1 | PASS |
| P3 | True | 0 / 0 | 122 / 120 | PASS |
| P4 | True | 0 / 0 | 131 / 120 | PASS |
| P5 | True | 0 / 0 | 131 / 1 | PASS |
| P6 | True | 0 / 0 | 131 / 120 | PASS |

| Running reference | Frames | Tick start → end | Population start → end | Structure/provenance |
|---|---|---|---|---|
| W0 | 600 | 40560 → 40860 | 29 → 29 | PASS |
| W200 | 600 | 1440 → 1740 | 200 → 200 | PASS |
| W500_FROZEN | 240 | 1440 → 1440 | 500 → 500 | PASS |
| W500_2X | 240 | 1440 → 1680 | 500 → 500 | PASS |

Running validation uses the existing comparator with the same reference as both inputs solely to execute its frame/property/schedule/checksum checks. Equality to itself is not candidate identity. Before/inside/after source and debug hashes were separately checked, as were collector/build hashes, terminal launch rows and monotonic render counts. Per-framing A/A′ complete observed clocks were independently validated and compared exactly; RGBA has no mask or tolerance.

The six-framing supplemental base-provenance record was captured during A, not a per-launch before/after audit. Preserve that provenance limitation; running-A has full per-launch snapshots. All results and file hashes are in reference-validation.json.
