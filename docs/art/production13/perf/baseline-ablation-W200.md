# W200 running 2× ablation ranking

Source: `/Users/rexxa/work/art-production13-evidence/baseline/ablation-W200-attempt2.json`; actual alive: 200; release, running 2×, fresh start tick 1440 per block, Z4.

Cost = median of 300 samples from neighbouring all-on blocks minus median of 150 node-off samples. Negative costs are retained.

Diagnostic controls retain measured differences but are not removable costs. SimReads process-disable does not disable synchronous caching/getters. Floor disables simulation as well as presentation.

| Rank | Node/group | Cost ms | Off median | Neighbour median | Layer only | Role |
|---:|---|---:|---:|---:|---|---|
| 1 | AgentRenderer | 21.85050000756979 | 3.6469999988675097 | 25.4975000064373 | False | presentation |
| 2 | WorldSim | 9.78550000101325 | 15.8999999983311 | 25.68549999934435 | False | simulation_disabled_control |
| 3 | ActivityTrailRenderer | 0.861500004112699 | 25.10049999660255 | 25.96200000071525 | False | presentation |
| 4 | BuildingRenderer | 0.8604999949336474 | 25.055000003337852 | 25.9154999982715 | False | presentation |
| 5 | SeekVizRenderer | 0.6250000019669528 | 25.394000001370898 | 26.01900000333785 | False | presentation |
| 6 | NatureRenderer | 0.6145000138878984 | 24.869499992907052 | 25.48400000679495 | False | presentation |
| 7 | SettlementOverviewRenderer | 0.5550000003576017 | 25.1670000023246 | 25.7220000026822 | False | presentation |
| 8 | SettlementLabels | 0.5544999959469017 | 24.881999999999998 | 25.4364999959469 | False | presentation |
| 9 | NeedBarRenderer | 0.39999999564884803 | 25.1790000039935 | 25.57899999964235 | False | presentation |
| 10 | ZoomLodController | 0.32250000101325327 | 25.367500000953697 | 25.69000000196695 | False | presentation |
| 11 | Camera2D | 0.24600000137090205 | 25.50699999898675 | 25.753000000357652 | False | already_frozen_camera_control |
| 12 | DaylightController | 0.2220000020265971 | 25.305000000298 | 25.5270000023246 | False | presentation |
| 13 | WorldRenderer | 0.20600000238415106 | 25.4405 | 25.64650000238415 | False | presentation |
| 14 | DeathVizRenderer | 0.20500000166889976 | 25.334500001311298 | 25.539500002980198 | False | presentation |
| 15 | TerrainRenderer | 0.17699999612574757 | 25.60250000220535 | 25.779499998331097 | False | presentation |
| 16 | UI | 0.11200000149009881 | 25.6729999983907 | 25.7849999998808 | False | presentation |
| 17 | GodHandEffects | 0.10349999630454931 | 25.2695000026822 | 25.37299999898675 | False | presentation |
| 18 | UI/GodHandPanel | 0.07049999630449832 | 25.3160000010133 | 25.3864999973178 | False | presentation |
| 19 | UI/ChroniclePanel | 0.06349999827150299 | 25.3309999976158 | 25.3944999958873 | False | presentation |
| 20 | UI/CausalPanel | 0.05600000685450013 | 25.5029999935031 | 25.5590000003576 | False | presentation |
| 21 | UI/HudTopbar | 0.029499995887302788 | 25.15999999833105 | 25.18949999421835 | False | presentation |
| 22 | WorldObjects | -0.07250000247364952 | 25.5124999994338 | 25.43999999696015 | True | presentation |
| 23 | WorldEffects | -0.2339999958872525 | 25.55150000065565 | 25.317500004768398 | False | presentation |
| 24 | SimReads | -0.29949999529119964 | 26.02699999833105 | 25.72750000303985 | False | synchronous_reader_process_control |
| 25 | Shadows | -0.32899999910589983 | 26.04149999815225 | 25.71249999904635 | True | presentation |
| 26 | UI/AgentInspectorPanel | -0.4095000067949002 | 25.6105000040531 | 25.2009999972582 | False | presentation |
| 27 | UI/HudStatusPanel | -0.5499999921322001 | 25.71149999690055 | 25.16150000476835 | False | presentation |

Top three measured rows: AgentRenderer, ActivityTrailRenderer, BuildingRenderer

Floor: {"count": 150, "median": 0.01850000029429795, "p90": 0.0290000008642674, "p99": 0.039000001847744, "maximum": 0.0580000017285347, "over_budget": 0}

