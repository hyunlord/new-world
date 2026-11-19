# Slice 1 renderer experiment

The isolated Slice 1 entry point runs the normal 3D scene against a saved world. The crop is 25×24 tiles at (72, 90). The people layer displays the six inhabitants inside the crop, and the surrounding world stays visible as scenery. Both capture arms use the same save and camera framing. The ultra arm replaces existing vegetation inside the crop, swaps the crowd to the existing Tripo VAT, and enables the PBR and Forward+ settings. The standard main scene is still the regular play entry point.

## Run locally

The save and framing are in `/Users/rexxa/work/art-img-data/slice1/fixture/`. Raw textures stay under `slice1/assets/`. Set `WORLDSIM_SLICE1_ASSETS` if the raw assets are elsewhere. Build the bridge and import the project before using the debug entry point.

```bash
source tools/v8/godot_bin.sh
SLICE=/Users/rexxa/work/art-img-data/slice1
WORLDSIM_TEST_SETTINGS_DIR="$SLICE/settings" \
  bash tools/v8/run_bounded.sh 300 "$GODOT_BIN" --path . \
  --script res://tools/v8/slice1.gd -- --quality=ultra \
  --save="$SLICE/fixture/village-seed0-day120.wsave" \
  --crop=72,90 --crop-size=25,24 --focus=86,99.8 --close=93.8,101.1
```

The GPU script expects an inherited `mux2_gate_lock.py` lock and a fresh output directory. It writes the six day/dusk pairs and two supplementary daytime pairs at 1080p, a contact sheet, raw FPS samples, machine-load admissions and paired per-setting costs. The crop benchmarks hold the simulation clock at day 120, and the 500-agent benchmark runs the bridge stress world at 1×. All runs use the same Godot executable and bridge build.

```bash
until /opt/homebrew/bin/python3 tools/v8/mux2_gate_lock.py -- \
  /opt/homebrew/bin/python3 tools/v8/slice1/run_gpu.py \
  --fixture "$SLICE/fixture/framing.json" --output "$SLICE/reproduction"; do sleep 1200; done
```

The external `LICENSES.md` lists the Poly Haven raw files and derived vegetation inputs. Material arrays use 1024-pixel layers with mipmaps. The vegetation exporter converts publisher DWAA EXRs to PNG for Godot's importer and retains mesh LOD generation. The Tripo character VAT and atlas files come from local `v8/tripo1` at `872dd4917`.

## Style 2 anchor

Append `-- --style=s2` to a regular Godot launch to enable the anchor across the game. With the Slice 1 script, pass `--style=s2` alongside the framing arguments. Use `--mode=capture --anchor-shots=1` with `--forest`, `--roof` and a fresh external `--out` path for the four daylight views. `--fixed-fps 48` before the Godot argument separator and `--mode=walk` after it record the three-second diagnostic walk. Both commands require the shared GPU lock.

The optional `--style s2` parameter on the step 39 receipt command selects the anchor for the existing eight-scene visibility test. The style variant uses the packaged texture strips and derived CC0 foliage in `assets/3d/anchor2/`. See [ADR 0072](../../decisions/0072-style-2-anchor.md).

The fir branch albedo and normal, and broadleaf branch normal are stored as 8-bit RGB PNG after export for the repository file-size cap. Convert the three copied textures with Pillow using `image.convert("RGB").save(path, optimize=True)`. The foliage cutout maps remain separate.
