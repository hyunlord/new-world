# Pass 13 Part 0.4 — commands

Recorder `p13bin/wm_p0` (= Pass 12's `wm_final`, base `2aeb22c9`), `WORLDSIM_GIT_SHA=2aeb22c97617dc5b3a51896ffe7a8c9016647e7f`:

    for scene in game_v16 game_v17 game_v18; do
      wm_p0 --scene $scene --seeds 901000..901032 --days 120 --every 60 --schedule-dir p13/empty_schedules \
        --include-factual --fork-tick 1 --replicates 4 --workers 4 --out p13/paths_ref/$scene
    done
    wm_p0 --scene game_v18 --seeds 901000..901002 --days 120 --every 60 --workers 4 --out p13/paths_ref_unforked/game_v18

Table:

    python3 tools/revitalize/paths_p13.py --scene game_v16=p13/paths_ref/game_v16 --scene game_v17=p13/paths_ref/game_v17 \
      --scene game_v18=p13/paths_ref/game_v18 --diff game_v18:game_v17 --diff game_v18:game_v16 --diff game_v17:game_v16 \
      --check-unforked p13/paths_ref/game_v18 p13/paths_ref_unforked/game_v18 --seeds 901000,901001 --out docs/revitalize/probe/p13-0_paths

UBS (Pass 12's tool unchanged, on replicate 0 through a symlink view `p13/ubs_view/paths_ref/<scene>/<scene>_s<seed>_none`):

    python3 tools/revitalize/diversity_p12.py score --root p13/ubs_view/paths_ref --scenes game_v16,game_v17,game_v18 \
      --seeds 901000..901032 --out docs/revitalize/probe/p13-0_paths/ubs_r0
