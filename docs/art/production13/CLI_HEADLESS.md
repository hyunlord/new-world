# CLI-Anything windowless execution

The user requested CLI-Anything after explicitly prohibiting Godot windows. No editor/game window is permitted. CLI-Anything Godot 1.0.0 is installed via `uv tool install /Users/rexxa/github/CLI-Anything/godot/agent-harness` from source commit `34f519533bc175d2fe287ab8316b0dd99bb9cc43`. The initial `cli-hub install godot` failed because the hub uv environment has no pip; that environment was not modified.

The inspected backend `run_godot` adds `--headless`, and script validate/run explicitly request headless=True. The installed executable is `/Users/rexxa/.local/bin/cli-anything-godot`; GODOT_BIN points to the existing Godot.app/Contents/MacOS/Godot binary. This runs the engine without a display, not a GUI application. Do not use a wrapper which can fall back to windowed execution.

Verified via installed CLI:
- `--json -p CHECKOUT project info`: project metadata returned.
- `--json -p CHECKOUT script validate scripts/ui/agent_renderer.gd`: valid=true, errors empty.
- `--json -p CHECKOUT script run scripts/test/art/test_write_skip_p13.gd`: returncode0, explicit `WRITE_SKIP_P13 failures=0`, stderr empty.
- No Godot process remained after completion.

Raw JSON lives under `/Users/rexxa/work/art-production13-evidence/cli-headless`. These tests establish syntax and forced-write behavior only. Actual windowed frame-budget measurements, screenshots and visible-input acceptance remain unverified where interrupted or not yet run. Do not use dummy/headless rendering timings as substitutes for the required release display workload.

The CLI script runner adds `--quit`; for asynchronous tests require their explicit completion sentinel, never accept process exit0 alone. The write-skip test did emit its completion sentinel.
