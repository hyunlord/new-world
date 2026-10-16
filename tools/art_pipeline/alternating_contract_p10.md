# Part 1 alternating 150-frame experiment

Run from the primary art checkout after installing the candidate switch:

```sh
python3 tools/art_pipeline/run_alternating_p10.py /Users/rexxa/work/new-world-art-prod wood-icon-cache /Users/rexxa/work/art-production10-evidence/pairs/wood-icon-cache-1
```

Default world: W500, frozen Z4, same seed/tick/hearth framing as `perf_frames_p10.gd`. An optional fourth argument selects W0/W1/W200. The runner refuses existing evidence directories. It delegates to `with_quiet_lock_p10.sh`, so one invocation means one launch/world/hold, at most 60 minutes with the shared cooldown and contention protocol. Do not invoke it inside another quiet hold.

## Required production integration, intentionally not implemented by this preparation

`Main/AgentRenderer` must temporarily expose:

```gdscript
func set_perf_candidate_p10(change_id: String, enabled: bool) -> bool:
    # Return false for an unknown change; true only after activating the requested implementation.

func get_perf_candidate_p10() -> Dictionary:
    # Return exactly {"change_id": <id>, "candidate_enabled": <bool>}.
```

`false` executes the **immediately preceding accepted implementation**. `true` executes that implementation plus **one named candidate change**. Neither branch may be a placebo. Both must do identical visible work. If the change lives in a helper, the renderer switch must route that helper too. Invalidate any implementation-specific derived caches when switching, without resetting animation, pose, stationary/idle timers, sprite positions, world state or other visual clocks. Retain previous accepted optimizations in both branches. Review the actual branch bodies before trusting timing evidence; the harness can verify switch readback, not prove that both implementations are real.

Candidate identity captures and running-equality checks run separately with candidate=true. The identity captures must not be launched inside the timing hold. Only after the named candidate passes identity/running gates and its performance evidence is retained may its temporary control branch/toggle be removed. Do not remove earlier accepted semantics to make a timing result look better.

## Measurement and evidence

Order is control, candidate, control, candidate, control, candidate. Each block discards 30 frames then records 150 frames. Switching is outside the CPU bracket. Warmup does not reset pose clocks. Every sample uses the reused P9 paired process stamps plus measured render CPU: primary = process_ms + render_cpu_ms. Setup CPU, GPU and pacing remain separate. Vsync is disabled, FPS uncapped, camera processing disabled, viewport 1920x1080, world frozen at Z4 by simulation speed zero, retaining the WorldSim process callback as the matrix does. Camera smoothing is reset and canvas scrolling forced after positioning. Society layers remain enabled; the inspector is closed and influence channel is -1, checked on every sample. The reused Runtime snapshot field `simulation_frozen` specifically means process-mode disabled, so it is false here; invariant tick values and simulation_speed=0 establish the frozen condition. Runtime state is restored before exit.

`launch.json` records source SHA256 for GDScript/scenes/shaders/project.godot, HEAD and the debug dylib SHA256 both before and after the hold. `frames.json` records the loaded build hash, raw ordered blocks, readback states, restoration and errors. `session.exit` must exist and equal zero; the log must contain no ERROR. Validation rejects incomplete blocks, changed source/build/world, unpaired/nonincreasing frames, missing rendering measurements, wrong primary metric or missing terminal evidence. `summary.json` reports individual blocks, pooled control/candidate distributions and speedup, explicitly leaving pixel identity NOT_VERIFIED. A faster pair is not acceptance by itself.

If integration is absent, the runner rejects before acquiring a hold or starting Godot. This preparation has not been runtime-validated; execute it after Part 0 and the actual candidate implementation exist.
