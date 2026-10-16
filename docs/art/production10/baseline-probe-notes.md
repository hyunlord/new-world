# Part 0 baseline probe

`probe_baseline_p10.gd` instantiates only `WorldSimNode`, disables automatic processing, resets the world and advances exact tick counts. It does not launch a scene, render, intervene, or invent a world state. It records complete world-info, clock, settlement/fire, building and activity getter outputs. The source configuration must still be confirmed against `world_node.rs`; the script does not claim a scene version from a label.

Run only after the merged bridge is built and the sharing protocol permits Godot:

```sh
P10_BASELINE_OUT="$CAPTURE_EVIDENCE/baseline-live.json" "$GODOT_BIN" --headless --path . --script scripts/test/art/probe_baseline_p10.gd -- --mode=live
P10_BASELINE_OUT="$CAPTURE_EVIDENCE/baseline-framings.json" "$GODOT_BIN" --headless --path . --script scripts/test/art/probe_baseline_p10.gd -- --mode=framings
```

The live mode defaults to seed 20261010 at display day 30, 10:00, calculated from the actual ticks-per-day and clock offset. It joins activity entity bits to agent snapshots, counts only `state_tag == 4` by all seven kinds, reports missing activity matches explicitly, and records five found family getter results together with detail and temperament. Fewer than five found families remains visible as a shorter list. Five families means five agent family records, not five distinct households.

Framings mode records the original P1–P6 states, including stress P5. Use `--case=P4` to isolate the long run. Outputs are checkpointed after each framing. A census is not pixel consistency evidence; P5 must still be captured and compared with Pass 9 final B.

For replacement discovery, prefer one bounded seed per launch:

```sh
P10_BASELINE_OUT="$CAPTURE_EVIDENCE/search-20261011.json" "$GODOT_BIN" --headless --path . --script scripts/test/art/probe_baseline_p10.gd -- --mode=search --seed=20261011 --purpose=villages --max-tick=172800
```

`--purpose=night` searches for a lit night (20:00–06:00) at the first settlement, matching the framing camera anchor (the getter sorts settlement IDs). `--last-seed=20261042` optionally extends through the allowed seed range. Seed 20261010 is accepted for searching the original live world; alternatives are bounded to 20261011–20261042. Search stops at the first matching tick of the first matching seed. Default stride 1 scans every tick from zero, proving the first qualifying tick. Optional `--start-tick=N --stride=N` supports cheap exploratory sampling and explicitly sets `first_tick_proven=false`; these exploratory results must not be called first-tick results. No match is reported as `search_exhausted`, not a successful replacement.

Use `run_bounded.sh` around launches as required by GOAL. Script creation was checked with `git diff --check`; runtime/parse validation is delegated to the leader after its active gate. No Godot, cargo, or runtime process was started by this implementation lane.
