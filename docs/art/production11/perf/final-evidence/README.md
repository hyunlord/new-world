# P11 final performance evidence

Frozen matrix commit: `5ee5e1ee98df24a16756360b8b125d582f68fdd0`. All four after worlds are validated against their raw sessions. W500 attempt1 was interrupted; attempt2 is accepted. Source-equivalence receipt and interruption evidence are retained in the ZIP.

After section timers are **NOT_MEASURED**: the queued section session reached the original admission cutoff at19:50UTC, wrapper exit124, driver exit1, invalid summary, no frames.json. Its summary, log, launch, exit, generated diagnostic source and manifest are preserved as rejected evidence. No absent timers are replaced with zeros. Baseline timers remain measured.

`performance-compact.md` has16 primary condition comparisons plus block populations, flags and node costs. `performance-full.md` includes all metrics. GPU zero is an unavailable counter placeholder, not a measured GPU duration. Optional W1000 is outside the required W0/W1/W200/W500 matrix. Art10 uses its actual historical baseline, with absent after results reported separately.

`raw-evidence.zip` preserves both full aggregate schemas, eight accepted raw sessions, baseline evidence ZIP, actual historical Art10 baseline/raw sources, build/source-equivalence receipts, terminal holds snapshot and failed section evidence. `manifest.json` lists original paths, sizes and SHA256 for55 explicit inputs; archive payloads were rehashed after copying. `index.json` records archive hash and external provenance locations. Absolute paths are original provenance, not portable checkout dependencies.

No large plain raw aggregates are committed here. Extract members identified by manifest when full data is needed.
