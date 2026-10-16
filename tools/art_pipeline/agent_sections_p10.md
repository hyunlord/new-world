# P10 diagnostic section session

Preparation only: the harness/runner have offline Python verification, not a Godot launch. Parent owns the Part 0 completion boundary before this command may run.

From the art checkout, after the leader sets `P10_DEADLINE_EPOCH` to the original task's 12-hour deadline:

```sh
python3 tools/art_pipeline/run_agent_sections_p10.py \
  /Users/rexxa/work/new-world-art-prod \
  /Users/rexxa/work/art-production10-evidence/sections-before-attempt1
```

The output directory must be new. The runner creates the diagnostic renderer and SHA-bound manifest there, then invokes the existing quiet-lock wrapper (which nests the bounded window lock) with a 3,600-second hold ceiling. An outer process-group deadline also limits lock waiting to the original task deadline. No fixed FPS, headless mode, simulation checkout, production source mutation, implicit retry or build occurs. Failed attempts and terminal exit codes remain in their directories. Retry only with a new attempt directory within the brief's three-strike rule.

The harness installs the generated script on the detached AgentRenderer before main enters the tree. It uses stress seed15100/population500/tick1440, the matrix's lowest-ID settlement camera formula, 1920x1080/Z4, 30 discarded frames then 300 counted frames. It requires closed inspector, influence off, layers enabled, renderer/world callbacks enabled, and fixed tick across every sample. World speed has no public getter: `speed_setting=0` records the actual setter command, explicitly distinguished from an observed native getter; unchanged ticks and enabled callbacks corroborate the frozen condition.

`frames.json` contains all samples, 25 exclusive spans, whole-callback times, frame alignment, installation/build provenance, UI conditions and restoration evidence. `launch.json` records source/build hashes before/after and command/deadline; `session.log` and `session.exit` preserve terminal outcome. `summary.json` independently checks those results and reports median, nearest-rank p90/p99 and maximum for each section in microseconds. FoundingGroups remains explicitly `unavailable_separate_callback`, not attributed to the parent callback.

The generated script currently loads from its absolute evidence path via the existing loader. Godot acceptance of that path and full scene-script replacement must be verified during the leader's first allowed parse/runtime session; Python checks cannot establish it. Source markers reject unreviewed changed renderer structure. Diagnostic results include timer overhead and cannot replace uninstrumented H1–H4 measurements.

Offline checks:

```sh
python3 -m unittest discover -s tools/art_pipeline -p 'test_*p10.py'
python3 -m py_compile tools/art_pipeline/run_agent_sections_p10.py tools/art_pipeline/test_agent_sections_session_p10.py
```
