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

## Rendering boundary verified against Godot 4.6

The installed engine help lists only `dummy` for the headless display. The [4.6 headless display implementation](https://github.com/godotengine/godot/blob/4.6-stable/servers/display/display_server_headless.h#L43) advertises that driver and selects RasterizerDummy. Standard headless execution cannot produce a real Metal framebuffer; a SubViewport or CLI wrapper does not change this. Hidden/minimized/offscreen native windows are not an authorized workaround.

The [TextServer API](https://docs.godotengine.org/en/4.6/classes/class_textserver.html) offers CPU glyph rasterization and atlas retrieval. Godot 4.6 [renders glyphs through FreeType](https://github.com/godotengine/godot/blob/4.6-stable/modules/text_server_adv/text_server_adv.cpp#L1362) and [returns the stored CPU Image](https://github.com/godotengine/godot/blob/4.6-stable/modules/text_server_adv/text_server_adv.cpp#L3129). This is a candidate route for Part3 mechanical glyph masks, pending runtime verification with the actual Life fonts/settings. It cannot prove final UI layout or framebuffer compositing.
