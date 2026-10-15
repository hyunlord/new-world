# Live activity census evidence

Census integrity checks **PASS** for both completed headless runs. This records simulation holders, not rendered poses or visual acceptance.

| Framing | Seed | Tick | Population | Settlements | Holders / state 4 | Arrived | Approaching |
|---|---:|---:|---:|---:|---:|---:|---:|
| Day 30 15:00 | 20261010 | 42180 | 28 | 1 | 14 / 14 | 11 | 3 |
| Day 60 10:00 | 20261018 | 85080 | 92 | 2 | 54 / 54 | 28 | 26 |

| Kind | Day30 15:00 holders (arrived/approaching) | Day60 10:00 holders (arrived/approaching) |
|---|---:|---:|
| TendFire | 0 (0/0) | 11 (8/3) |
| Craft | 1 (1/0) | 2 (1/1) |
| Repair | 1 (1/0) | 10 (5/5) |
| HearthCircle | 7 (5/2) | 13 (7/6) |
| Explore | 1 (0/1) | 8 (0/8) |
| Play | 2 (2/0) | 1 (0/1) |
| SitByFire | 2 (2/0) | 9 (7/2) |

Both runs exited0, reported `complete=true`, had no probe failures, and had zero unknown state4 kinds. All activity entity IDs join the listed state4 agents in these two outputs. Each run includes five successful family/detail/temperament getter samples. Neither census has a lit hearth at its daytime tick.

The [existing seed selection](seed-selection.md) selected 20261018 by day60 endpoint population at tick86400. This census confirms population92 at calendar day60 10:00, tick85080; it does not rerank seeds. Earlier day30 10:00 census remains in [baseline evidence](../baseline/live-day30-1000.json).

[Raw evidence archive](census-evidence.zip): SHA256 `2315545e08bd77aa221f5608e5319a4c2f2cdecb0f16c350d77fe065b2e91665` (13345 bytes). [Machine-readable index](census-index.json) records every member hash, verified checks and full counts. The archive contains all ten original files plus the unchanged original probe and exact two-line calendar/label diff.

Provenance: the original probe and external probe match `source.json`; launch records match seeds and requested clock times. The launch record does not bind HEAD or GDExtension binary hashes, so this packaging makes no retroactive binary/source claim.

Serialization limitation: raw `PackedByteArray` kinds/fire flags are bracketed strings. The index parses these strings for counting and preserves raw bytes. Future capture scaffolding needs explicit packed-array normalization before its array-based validator can accept captures.

**Pending:** six actual Z4/Z1 captures, final renderer pose/visibility sidecars, independent zero-mismatch check, and fourteen real blind-review crops with recorded answers. Day30 15:00 contains no TendFire holder; day60 has only one Play holder at this tick. Holder availability is not crop visibility or proof of two usable crops per kind. No H7 or visual PASS is established.
