# Pass 12 Part 5 — exact commands

Recorder `p12bin/wm_frozen` (frozen `8142cc3a`), seeds 24000–24111 (never used before Part 5; train 24000–24095, test 24096–24111).

```bash
bash docs/revitalize/probe/p12-5_pilot/p5_pilot.sh 8142cc3a66b459a658258daee8c4b1eb6d393f0b      # 13:53–14:24 KST
cd tools/revitalize
python3 diversity_p11.py check-v1 --root /Users/rexxa/work/worldsim-wm-data --out ../../docs/revitalize/probe/p12-5_pilot/check_v1   # REPRODUCED
D=/Users/rexxa/work/worldsim-wm-data/p12/pilot; O=../../docs/revitalize/probe/p12-5_pilot
python3 diversity_p11.py pilot --root $D --scenes game_v16,game_v17,game_v18_nofission,game_v18 --train 24000..24096 --test 24096..24112 --out $O/p11_metrics
python3 diversity_p12.py score --root $D --scenes game_v16,game_v17,game_v18_nofission,game_v18 --seeds 24000..24112 --test 24096..24112 \
  --bootstrap game_v17:game_v18 game_v17:game_v18_nofission game_v18_nofission:game_v18 --split-subsets game_v18 --out $O/ubs
python3 pilot_p12.py --metrics $O/p11_metrics/diversity.json --ubs $O/ubs/ubs.json --out $O
```
