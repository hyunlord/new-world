# External engines

The character and motion tooling now lives in two engine repositories (D-REPO1, 2026-11-24):

| engine | what it does | replaces |
|---|---|---|
| [motion-engine](https://github.com/hyunlord/motion-engine) | humanoid retarget, physics tracking (MJX), video to motion, BVH/glTF/VAT export | PHYS1/PHYS2 and VID2MO tooling (DGX `~/worldsim-3d/phys2`, `vid2mo`) |
| [human-forge](https://github.com/hyunlord/human-forge) | slider bodies on CC0 MakeHuman data, rig + skin, deformation audit, cloth, hair, LOD, VAT, Blender export | CHAR3 tools (`v8/char3` branch), LOOK2 tools, and the mocap pipeline formerly in `tools/3d/blender/mocap_*` |

- `engines.toml` pins each engine by tag and commit. The game never vendors engine code.
- `fetch_engines.sh` checks the pinned commits out under `../3d-data/engines` with a Python 3.12 venv.
- `generate_people.sh` / `generate_motion.sh` run the engines with `profiles/worldsim` and write assets plus a
  `.provenance.json` per file (`provenance.py`: engine, tag, commit, inputs with sha256, command, licence, time).
- Generation runs on the Mac (Blender under the mux2 GPU lock) or the DGX, never at game runtime. Assets already in
  `assets/3d` stay as they are; regenerate them through these scripts when an engine pin changes.
- The WorldSim mocap pipeline (`mocap_*.py`, its RON specs and unit tests) moved to
  human-forge `profiles/worldsim/blender/mocap`. Its bake/pose/gait modules still import this folder's content modules
  (`common`, `chars_b`, `s2clips*`, `s2kit`): run them with `PYTHONPATH=<this repo>/tools/3d/blender`.
