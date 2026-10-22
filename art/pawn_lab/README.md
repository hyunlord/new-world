# Pawn evidence lab

The lab reads the PNG layers covered by the structural checker's current hashes.
It never redraws a pawn or repairs output pixels. `compose_layers` is the shared
consumer in `pipeline4/pawn_gen.py`; its age registration and motion match the
checker. Failed structural inputs remain visibly `DIAGNOSTIC_REJECTED`.

```sh
python3 -m art.pawn_lab.build ROUTE_P/manifest.json CHECKS.json /Users/rexxa/work/arttrial4-data/lab
python3 -m art.pawn_lab.boards ROUTE_P /Users/rexxa/work/arttrial4-data/lab
CHROMIUM_EXECUTABLE="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" python3 -m art.pawn_lab.browser /Users/rexxa/work/arttrial4-data/lab 3
python3 -m art.pawn_lab.runtime /Users/rexxa/work/arttrial4-data/lab 60
```

Each of twelve balanced people has one activity for blind scoring. Its GIF shows
S/E columns and meadow/forest rows at original 1× or nearest 2×. The two walk and
two work frames alternate; idle/carry/sleep remain the correct static states.
The browser recorder captures native GIF playback at a target 8 Hz, retaining
actual timestamps and hashes. Static specimens correctly report static playback.
Only `answer_key.private.json` exposes expected attributes; the web page exposes
numbers and scale. Browser evidence is separate from Godot FPS.

The seven-frame Godot loop includes idle, both walk frames, both work frames,
carry and sleep. All twenty-four person/facing sequences use exactly the exported
composites. Crowd mode draws 200 pawns at native 1× for 60 seconds, records the
window renderer, every frame delta, bounds coverage and hashes. Pawn overlap is
possible in crowd mode; coverage does not imply readability. No game bridge is
loaded because `art/proto` is a standalone project.

Grounds use existing `art/style05/ground_v2/{meadow,forest_floor}.png`: these are
the calm tiles. `art/style05/env/ground/` contains the older detailed tiles.
Provenance records the exact ground hashes. The A/B/P board uses the historical
six A/B source-layer selections recomposed on that same meadow; rejected A/B
assets remain labelled. No background extraction from a flattened old board.

Generated PNGs/GIFs/runtime logs remain outside the repo except five requested
boards, each checked against the 1.5 MB ceiling. Six transparent numbered stills
are exported separately for style judging. A file's structural status and the
live viewing evidence never imply reviewer acceptance.

Runtime lesson from this pass: on macOS an occluded window can keep processing
while `frame_post_draw` stops. The lab follows the existing CharacterLab pattern:
foreground plus always-on-top, monotonic wall-clock completion from `_process`,
and a separate rendered-frame counter. It records `window_can_draw`, focus and
hidden process frames every second. Process callbacks never substitute for
rendered frames. Two failed initial harness runs are preserved with their logs;
the successful 60-second run is a distinct evidence directory.
