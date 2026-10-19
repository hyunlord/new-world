# Life capture evidence: captures complete; fact-reading verdict separate

The required Life capture families have retained successful outputs. This report establishes capture/provenance and crowd geometry only; it does **not** pass the fresh-reader non-name fact bar or crowd activity-reading bar. Fresh-reader scoring was not opened or conducted by this reporting agent.

| Case | Seed / tick | Successful PNGs | Attempts / status |
|---|---|---:|---|
| living/dead-parent and chronicle |20261010 /85080|41|1, capture complete|
| departures and porter Life |19400 /121080|51|1, capture complete|
| quarreller/estrangement/chronicle |19414 /51870|36|3, capture complete after two retained failures|
| crowd1 EN/KO |20261010 /41880|2|1, geometry pass|
| crowd2 EN/KO |20261018 /85080|2|1, geometry pass|
| crowd3 EN/KO |20261010 /42180|2|1, geometry pass|

The Life multi world19400 is inherited fact-reading evidence, not the seed19603 H10 porter requirement. Actual reconciliation captures are separately retained in `../h10/`; its failed road requirement is not repaired by successful Life captures.

## Estranged retry history

- Attempt1 at34680 failed `validated estrangement/quarrel missing at fixed tick`. Historical game_v26 seed19414 already has quarrels, but first kind20 is stamped51840. The older fixed instant was stale for the current scenario.
- Attempt2 at51840 failed the same assertion. This was a tooling boundary error: advancing N ticks executes ticks0..N-1, so the event stamped51840 needs at least51841. `sim-engine/src/lib.rs` assigns the current execution tick before systems and increments only after them. The earlier exact-event recommendation was incorrect and is preserved as failed evidence.
- Attempt3 at51870 succeeded with36 PNGs and complete raw references. First historical kind19 subject23 was alive at this instant; live capture validates current-build history. No fourth attempt was run. Attempt1/2 failure JSONs lacked raw references; that evidence limitation is retained, not reconstructed as measured data. The third-version failure writer includes raw references if a future failure occurs.

## Crowd geometry

All six existing EN/KO proof JSONs were inspected and copied here:

| Scene | Within10tiles | Body covered | Icon covered | Both-language geometry |
|---|---:|---:|---:|---|
| crowd1 |25|0|0|true|
| crowd2 |40|0|0|true|
| crowd3 |25|0|0|true|

Each has `zero_coverage_crowd_proven:true` and `activity_reading:NOT_VERIFIED`. Geometry is not an activity answer or fact-reading score. Selection uses actual raw entity identities and actual draw order; no rescaling or sprite relocation is implied.

## Provenance and limitations

All successful references explicitly record debug profile and bridge SHA256 `ab9be4eda437835c79d0ff7d96f3a7addfd73a52c677bb4a241120e0069267c2`. Live/multi are HEAD8ca6b93018b0ec0b795f8312c549197a97f851da; crowds are3bee1316acdbcb891bda0be1385ba959bbb1e7f1; estranged3 is75c2ea6a2cdfb085a4a4a90ceaf9fe257cda16a3. These are captures across retained source freezes, not one uniform final HEAD or release performance measurement.

`raw-manifest.json` hashes300 external artifacts beneath `/Users/rexxa/work/art-production13b-evidence`: Life/crowd original PNGs, sidecars, references, failure JSONs, launch logs and registration/result records. Reader bundles were excluded from this inventory and no reader answers/keys were inspected. Large originals remain external, explicitly identified by absolute path, byte size and SHA256. Small launch logs and six crowd proofs are copied here. No existing failed capture was deleted or overwritten.

Pending outside this report: fresh-reader non-name statement verification; names/confusions separately reported; unverifiable claims unscored; selected-person crowd activities read from images. Do not infer any of these from `complete:true`, screenshots existing, or geometric proof.
