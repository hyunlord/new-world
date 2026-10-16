# Activity blind scores — final two rounds

Full C protocol: **NOT_VERIFIED (12/14 eligible)**. Original bar remains >=10/14 and no kind0/2. Diagnostic failure is reported separately; no denominator scaling or third round.

| Round | C diagnostic | Full C | D | D >=5/6 |
|---|---:|---|---:|---|
| 1 | 0/12 | NOT_VERIFIED | 2/6 | FAIL |
| 2 | 1/12 | NOT_VERIFIED | 5/6 | PASS |

| Activity | Available / required | Round1 correct | Round2 correct |
|---|---:|---:|---:|
| TendFire | 1/2 | 0 | 0 |
| Craft | 2/2 | 0 | 0 |
| Repair | 2/2 | 0 | 0 |
| HearthCircle | 2/2 | 0 | 0 |
| Explore | 1/2 | 0 | 1 |
| Play | 2/2 | 0 | 0 |
| SitByFire | 2/2 | 0 | 0 |

Missing TendFire/Explore samples are absent, not fabricated wrong answers. Fully sampled kinds with0/2 independently show recognition failure. D round2 pass does not erase round1 failure or complete H7.

## Frozen answers and reveal provenance

- Round1: C SHA256 `fa16a54c131564787af6b821fe18cd1aec427a1b28a4fe9f947a8d3aa38140be`; D SHA256 `b434503ac24ea79cfd224d1f67cdf41e44f5dbbd8e531e41ae491b091f4ade07`; reveal `2026-10-16T12:25:32.068021+00:00`.
- Round2: C SHA256 `c8e84aae68c4d1c189a443c798351c44880ebfb501ce18e587fe11bcce975ae1`; D SHA256 `7b2e55e25d2c39a6ce088431b90a9bbf870209fc272b4eef59b55e48a0d8d596`; reveal `2026-10-16T12:30:27.998189+00:00`.

Both per-item score JSON files, frozen answer files, original numbered crops, keys and source sidecars are retained in raw-evidence.zip. Archive manifest maps every entry to its original absolute source path and SHA256. No new answers or crops were generated during packaging.
