# Pass 11 Part 6.2 — exact commands

```bash
# recording (wm_frozen, WORLDSIM_GIT_SHA = c5b200d7…; the macro schedule directory holds only the
# rot_wood and gift_wood files of p11/schedules/main)
bash docs/revitalize/probe/p11-6_macro/p11_macro.sh c5b200d7ee618b465d6aa0fdf054fb82d820eed6
# metrics
python3 tools/revitalize/macro_sensitivity.py --root /Users/rexxa/work/worldsim-wm-data/p11/macro --scene game_v17 \
  --seeds 12000..12032 --variants rot_wood,gift_wood --replicates 2 --out docs/revitalize/probe/p11-6_macro
```
