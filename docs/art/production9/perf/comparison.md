# Art Pass 9 comparison

Speed-up = before / after. Missing evidence is NOT_VERIFIED; F6/F7 have no performance verdict.

| Rule | Status | Evidence |
|---|---|---|
| F1 | PASS | {"status": "PASS", "before": {"count": 900, "median": 10.2140000038147, "p90": 10.7369999935627, "p99": 11.9540000017881, "maximum": 12.7100000030994, "over_budget": 0}, "after": {"count": 900, "median": 3.9680000010132748, "p90": 4.361000005126, "p99": 4.76099999988079, "maximum": 5.02700000596046, "over_budget": 0}, "median_speedup": 2.5740927422395252, "median_limit_ms": 8, "p99_limit_ms": null} |
| F2 | FAIL | {"status": "FAIL", "before": {"count": 900, "median": 67.57949999976155, "p90": 68.8040000071526, "p99": 71.2170000027418, "maximum": 72.6900000100136, "over_budget": 900}, "after": {"count": 900, "median": 51.5325000092983, "p90": 52.2609999864101, "p99": 53.0210000066757, "maximum": 53.6500000026226, "over_budget": 900}, "median_speedup": 1.3113957209055993, "median_limit_ms": 12, "p99_limit_ms": 16.67} |
| F3 | FAIL | {"status": "FAIL", "before": {"count": 900, "median": 68.29799999809265, "p90": 186.902999999762, "p99": 552.813999993086, "maximum": 1707.665, "over_budget": 900}, "after": {"count": 900, "median": 36.13349999326465, "p90": 163.052999999046, "p99": 168.29400000453, "maximum": 173.741999999762, "over_budget": 900}, "median_speedup": 1.8901573335221753, "median_limit_ms": 16.67, "p99_limit_ms": null} |
| F4 | PASS | {"status": "PASS", "baseline_counts": "NOT_MEASURED", "inventory": {"confirmed": true, "no_scaling": true, "source": "docs/art/production9/FFI_INVENTORY.md; current static scanner and SimReads counted causal route; runtime closed-state context must independently validate", "causal_source": "sim_reads", "direct_calls": {"get_agent_detail": 0, "get_influence_overlay": 0, "get_tile_causal_events": 0}}, "total_calls": {"count": 900, "median": 11.0, "p90": 11.0, "p99": 11.0, "maximum": 11.0}, "limit_calls_per_frame": 20, "direct_residual_calls": 0.0} |
| F5 | PASS | {"status": "PASS", "framings": [{"framing": "P1", "control": {"pixels": 0, "max_channel": 0}, "candidate": {"pixels": 0, "max_channel": 0}, "bit_reproducible": true, "passed": true, "cause": "bit-reproducible", "rgba_sha256": ["47785c7a85cfc635fa7aca4efa93b4daa36ff7d78b2f25e431a0de18372c97f2", "47785c7a85cfc635fa7aca4efa93b4daa36ff7d78b2f25e431a0de18372c97f2", "47785c7a85cfc635fa7aca4efa93b4daa36ff7d78b2f25e431a0de18372c97f2"], "capture_frame": 130, "state_change_frame": 10}, {"framing": "P2", "control": {"pixels": 0, "max_channel": 0}, "candidate": {"pixels": 0, "max_channel": 0}, "bit_reproducible": true, "passed": true, "cause": "bit-reproducible", "rgba_sha256": ["39d798fdd859f1cf3bd231317df514997ff9e8b47cf5a2e0e6ce999c87f8549f", "39d798fdd859f1cf3bd231317df514997ff9e8b47cf5a2e0e6ce999c87f8549f", "39d798fdd859f1cf3bd231317df514997ff9e8b47cf5a2e0e6ce999c87f8549f"], "capture_frame": 130, "state_change_frame": 10}, {"framing": "P3", "control": {"pixels": 0, "max_channel": 0}, "candidate": {"pixels": 0, "max_channel": 0}, "bit_reproducible": true, "passed": true, "cause": "bit-reproducible", "rgba_sha256": ["113af7d1259d84b29984f958be8201d9a8dd738f0c1d4599b5505a205fabc800", "113af7d1259d84b29984f958be8201d9a8dd738f0c1d4599b5505a205fabc800", "113af7d1259d84b29984f958be8201d9a8dd738f0c1d4599b5505a205fabc800"], "capture_frame": 121, "state_change_frame": 1}, {"framing": "P4", "control": {"pixels": 0, "max_channel": 0}, "candidate": {"pixels": 0, "max_channel": 0}, "bit_reproducible": true, "passed": true, "cause": "bit-reproducible", "rgba_sha256": ["182146e5c1f435d4c30336f64d6d92cf71ec702b1dff05553b5afe49703b767c", "182146e5c1f435d4c30336f64d6d92cf71ec702b1dff05553b5afe49703b767c", "182146e5c1f435d4c30336f64d6d92cf71ec702b1dff05553b5afe49703b767c"], "capture_frame": 130, "state_change_frame": 10}, {"framing": "P5", "control": {"pixels": 0, "max_channel": 0}, "candidate": {"pixels": 0, "max_channel": 0}, "bit_reproducible": true, "passed": true, "cause": "bit-reproducible", "rgba_sha256": ["439dc8420d24e72b763fc0048cb24909b6328bceb15aa2dc1482882408958a38", "439dc8420d24e72b763fc0048cb24909b6328bceb15aa2dc1482882408958a38", "439dc8420d24e72b763fc0048cb24909b6328bceb15aa2dc1482882408958a38"], "capture_frame": 130, "state_change_frame": 10}, {"framing": "P6", "control": {"pixels": 0, "max_channel": 0}, "candidate": {"pixels": 0, "max_channel": 0}, "bit_reproducible": true, "passed": true, "cause": "bit-reproducible", "rgba_sha256": ["47785c7a85cfc635fa7aca4efa93b4daa36ff7d78b2f25e431a0de18372c97f2", "47785c7a85cfc635fa7aca4efa93b4daa36ff7d78b2f25e431a0de18372c97f2", "47785c7a85cfc635fa7aca4efa93b4daa36ff7d78b2f25e431a0de18372c97f2"], "capture_frame": 130, "state_change_frame": 10}]} |
| F6 | REPORT_ONLY | {"status": "REPORT_ONLY", "before": {"largest_requested_n_measured_meeting_rule": null, "actual_alive_by_requested_n": {"200": 200, "500": 500}, "unmeasured_n": [1000, 2000]}, "after": {"largest_requested_n_measured_meeting_rule": null, "actual_alive_by_requested_n": {"200": 200, "500": 500}, "unmeasured_n": [1000, 2000]}} |
| F7 | REPORT_ONLY | {"status": "REPORT_ONLY", "before": {"source": "docs/art/production9/perf/a30-before/frame_times_soc_layers.json", "layers": {"on": {"count": 300, "median": 17.767999986171723, "p90": 18.189999997615814, "p99": 20.058000013828277, "maximum": 20.36699999165535, "over_budget": 300}, "off": {"count": 300, "median": 15.96550000357628, "p90": 16.401999993085862, "p99": 18.359000000476836, "maximum": 18.686999993801116, "over_budget": 14}}}, "after": {"source": "docs/art/production9/perf/a30-after/frame_times_soc_layers.json", "layers": {"on": {"count": 300, "median": 10.807500000715255, "p90": 11.355999993562698, "p99": 11.866000007867813, "maximum": 12.428999990701675, "over_budget": 0}, "off": {"count": 300, "median": 9.759000007271766, "p90": 10.135999999046327, "p99": 10.63900001335144, "maximum": 11.247999986171722, "over_budget": 0}}}, "speedup": {"on": 1.6440434869299851, "off": 1.6359770459760055}} |

| After getter | Calls distribution | Microseconds distribution |
|---|---|---|
| get_agent_snapshot | {"count": 900, "median": 1.0, "p90": 1.0, "p99": 1.0, "maximum": 1.0} | {"count": 900, "median": 365.0, "p90": 384.0, "p99": 406.0, "maximum": 435.0} |
| get_building_snapshot | {"count": 900, "median": 1.0, "p90": 1.0, "p99": 1.0, "maximum": 1.0} | {"count": 900, "median": 13.0, "p90": 15.0, "p99": 17.0, "maximum": 30.0} |
| get_construction_snapshot | {"count": 900, "median": 1.0, "p90": 1.0, "p99": 1.0, "maximum": 1.0} | {"count": 900, "median": 8.0, "p90": 10.0, "p99": 12.0, "maximum": 19.0} |
| get_intervention_state | {"count": 900, "median": 1.0, "p90": 1.0, "p99": 1.0, "maximum": 1.0} | {"count": 900, "median": 10.0, "p90": 12.0, "p99": 13.0, "maximum": 25.0} |
| get_recent_deaths | {"count": 900, "median": 1.0, "p90": 1.0, "p99": 1.0, "maximum": 1.0} | {"count": 900, "median": 6.0, "p90": 7.0, "p99": 8.0, "maximum": 16.0} |
| get_resource_snapshot | {"count": 900, "median": 1.0, "p90": 1.0, "p99": 1.0, "maximum": 1.0} | {"count": 900, "median": 130.0, "p90": 139.0, "p99": 148.0, "maximum": 154.0} |
| get_settlement_snapshot | {"count": 900, "median": 1.0, "p90": 1.0, "p99": 1.0, "maximum": 1.0} | {"count": 900, "median": 401.0, "p90": 414.0, "p99": 425.0, "maximum": 440.0} |
| get_society_snapshot | {"count": 900, "median": 1.0, "p90": 1.0, "p99": 1.0, "maximum": 1.0} | {"count": 900, "median": 557.0, "p90": 575.0, "p99": 594.0, "maximum": 638.0} |
| get_tallies | {"count": 900, "median": 1.0, "p90": 1.0, "p99": 1.0, "maximum": 1.0} | {"count": 900, "median": 20.0, "p90": 26.0, "p99": 29.0, "maximum": 41.0} |
| get_time_snapshot | {"count": 900, "median": 1.0, "p90": 1.0, "p99": 1.0, "maximum": 1.0} | {"count": 900, "median": 5.0, "p90": 7.0, "p99": 8.0, "maximum": 14.0} |
| get_world_info | {"count": 900, "median": 1.0, "p90": 1.0, "p99": 1.0, "maximum": 1.0} | {"count": 900, "median": 4.0, "p90": 5.0, "p99": 6.0, "maximum": 15.0} |

| World / condition | Metric | Before median / p90 / p99 / max / >16.67 | After median / p90 / p99 / max / >16.67 | Median speed-up |
|---|---|---|---|---|
| W0/frozen/Z4/0x | primary_cpu_ms | 10.2140000038147 / 10.7369999935627 / 11.9540000017881 / 12.7100000030994 / 0 | 3.9680000010132748 / 4.361000005126 / 4.76099999988079 / 5.02700000596046 / 0 | 2.5740927422395252 |
| W0/frozen/Z4/0x | process_ms | 10.039 / 10.55 / 11.784 / 12.532 / 0 | 3.831 / 4.219 / 4.596 / 4.879 / 0 | 2.62046463064474 |
| W0/frozen/Z4/0x | render_cpu_ms | 0.172999992966652 / 0.186000004410744 / 0.216000005602837 / 0.307000011205673 / 0 | 0.130999997258186 / 0.153999999165535 / 0.199000000953674 / 0.226999998092651 / 0 | 1.3206106609734412 |
| W0/frozen/Z4/0x | setup_cpu_ms | 0.004 / 0.009 / 0.012 / 0.015 / 0 | 0.005 / 0.008 / 0.011 / 0.012 / 0 | 0.8 |
| W0/frozen/Z4/0x | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 0 | 0.0 / 0.0 / 0.0 / 0.0 / 0 | None |
| W0/frozen/Z4/0x | pacing_ms | 11.4835 / 12.139 / 13.257 / 14.059 / 0 | 5.29 / 8.333 / 13.931 / 16.867 / 0 | 2.1707939508506615 |
| W0/frozen/Z4/0x | ticks_run | 0.0 / 0.0 / 0.0 / 0.0 / 0 | 0.0 / 0.0 / 0.0 / 0.0 / 0 | None |
| W0/frozen/Z1/0x | primary_cpu_ms | 10.438499988913499 / 10.9469999949932 / 12.0810000092983 / 13.9599999990463 / 0 | 4.029499999821185 / 4.43299999749661 / 4.71900000309944 / 5.7380000026226 / 0 | 2.5905199129859096 |
| W0/frozen/Z1/0x | process_ms | 10.145 / 10.634 / 11.791 / 13.659 / 0 | 3.8055000000000003 / 4.165 / 4.457 / 5.472 / 0 | 2.665878333990277 |
| W0/frozen/Z1/0x | render_cpu_ms | 0.293999999761581 / 0.307000011205673 / 0.361999988555908 / 0.537999987602234 / 0 | 0.224999994039536 / 0.259999990463257 / 0.303999990224838 / 0.323000013828278 / 0 | 1.3066667002218704 |
| W0/frozen/Z1/0x | setup_cpu_ms | 0.004 / 0.009 / 0.013 / 0.016 / 0 | 0.005 / 0.008 / 0.01 / 0.018 / 0 | 0.8 |
| W0/frozen/Z1/0x | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 0 | 0.0 / 0.0 / 0.0 / 0.0 / 0 | None |
| W0/frozen/Z1/0x | pacing_ms | 11.629 / 12.272 / 13.29 / 24.433 / 0 | 5.325 / 8.469 / 14.613 / 20.228 / 0 | 2.183849765258216 |
| W0/frozen/Z1/0x | ticks_run | 0.0 / 0.0 / 0.0 / 0.0 / 0 | 0.0 / 0.0 / 0.0 / 0.0 / 0 | None |
| W0/running/Z4/2x | primary_cpu_ms | 18.14800000607965 / 44.6350000209808 / 86.1680000081062 / 243.258000022888 / 479 | 6.7149999961853 / 7.51499999785423 / 8.47500000393391 / 9.37299999988079 / 0 | 2.7026061081741295 |
| W0/running/Z4/2x | process_ms | 17.8855 / 44.136 / 85.414 / 242.482 / 479 | 6.573 / 7.369 / 8.328 / 9.226 / 0 | 2.721055834474365 |
| W0/running/Z4/2x | render_cpu_ms | 0.247499994933605 / 0.492000013589859 / 1.16100001335144 / 8.32600021362305 / 479 | 0.137999996542931 / 0.150999993085861 / 0.163000002503395 / 0.180999994277954 / 0 | 1.793478269085385 |
| W0/running/Z4/2x | setup_cpu_ms | 0.009 / 0.039 / 0.069 / 0.36 / 479 | 0.007 / 0.01 / 0.013 / 0.022 / 0 | 1.2857142857142856 |
| W0/running/Z4/2x | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 479 | 0.0 / 0.0 / 0.0 / 0.0 / 0 | None |
| W0/running/Z4/2x | pacing_ms | 22.5655 / 48.66 / 122.669 / 379.218 / 479 | 16.603 / 18.243 / 18.935 / 19.306 / 0 | 1.3591218454496174 |
| W0/running/Z4/2x | ticks_run | 1.0 / 3.0 / 5.0 / 5.0 / 479 | 1.0 / 1.0 / 1.0 / 1.0 / 0 | 1.0 |
| W0/running/Z4/2x/tick-only | primary_cpu_ms | 18.3350000021458 / 44.7330000009537 / 103.619000002861 / 243.258000022888 / 479 | 6.7149999961853 / 7.51499999785423 / 8.47500000393391 / 9.37299999988079 / 0 | 2.7304542088699426 |
| W0/running/Z4/2x/tick-only | process_ms | 18.106 / 44.284 / 96.329 / 242.482 / 479 | 6.573 / 7.369 / 8.328 / 9.226 / 0 | 2.754602160352959 |
| W0/running/Z4/2x/tick-only | render_cpu_ms | 0.25 / 0.493000000715256 / 1.41100001335144 / 8.32600021362305 / 479 | 0.137999996542931 / 0.150999993085861 / 0.163000002503395 / 0.180999994277954 / 0 | 1.8115942482812053 |
| W0/running/Z4/2x/tick-only | setup_cpu_ms | 0.01 / 0.039 / 0.072 / 0.36 / 479 | 0.007 / 0.01 / 0.013 / 0.022 / 0 | 1.4285714285714286 |
| W0/running/Z4/2x/tick-only | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 479 | 0.0 / 0.0 / 0.0 / 0.0 / 0 | None |
| W0/running/Z4/2x/tick-only | pacing_ms | 22.726 / 49.103 / 132.308 / 379.218 / 479 | 16.603 / 18.243 / 18.935 / 19.306 / 0 | 1.3687887731132926 |
| W0/running/Z4/2x/tick-only | ticks_run | 1.0 / 3.0 / 5.0 / 5.0 / 479 | 1.0 / 1.0 / 1.0 / 1.0 / 0 | 1.0 |
| W0/running/Z4/4x | primary_cpu_ms | 17.3530000015497 / 55.5430000076294 / 187.315999956131 / 1430.32100000691 / 529 | 8.39750000089407 / 9.23100000727177 / 10.0629999978542 / 10.4079999984503 / 0 | 2.0664483476870674 |
| W0/running/Z4/4x | process_ms | 17.139 / 54.844 / 186.22 / 1429.972 / 529 | 8.2515 / 9.073 / 9.917 / 10.252 / 0 | 2.07707689510998 |
| W0/running/Z4/4x | render_cpu_ms | 0.202999994158745 / 0.541000008583069 / 1.50999999046326 / 180.087005615234 / 529 | 0.141000002622604 / 0.157000005245209 / 0.174999997019768 / 0.190999999642372 / 0 | 1.4397162438506341 |
| W0/running/Z4/4x | setup_cpu_ms | 0.011 / 0.042 / 0.059 / 0.208 / 529 | 0.008 / 0.01 / 0.012 / 0.014 / 0 | 1.375 |
| W0/running/Z4/4x | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 529 | 0.0 / 0.0 / 0.0 / 0.0 / 0 | None |
| W0/running/Z4/4x | pacing_ms | 18.7715 / 62.101 / 216.571 / 1894.082 / 529 | 16.6145 / 18.066 / 18.936 / 21.093 / 0 | 1.129826356495832 |
| W0/running/Z4/4x | ticks_run | 2.0 / 5.0 / 5.0 / 5.0 / 529 | 2.0 / 2.0 / 2.0 / 2.0 / 0 | 1.0 |
| W0/running/Z4/4x/tick-only | primary_cpu_ms | 17.3530000015497 / 55.5430000076294 / 187.315999956131 / 1430.32100000691 / 529 | 8.39750000089407 / 9.23100000727177 / 10.0629999978542 / 10.4079999984503 / 0 | 2.0664483476870674 |
| W0/running/Z4/4x/tick-only | process_ms | 17.139 / 54.844 / 186.22 / 1429.972 / 529 | 8.2515 / 9.073 / 9.917 / 10.252 / 0 | 2.07707689510998 |
| W0/running/Z4/4x/tick-only | render_cpu_ms | 0.202999994158745 / 0.541000008583069 / 1.50999999046326 / 180.087005615234 / 529 | 0.141000002622604 / 0.157000005245209 / 0.174999997019768 / 0.190999999642372 / 0 | 1.4397162438506341 |
| W0/running/Z4/4x/tick-only | setup_cpu_ms | 0.011 / 0.042 / 0.059 / 0.208 / 529 | 0.008 / 0.01 / 0.012 / 0.014 / 0 | 1.375 |
| W0/running/Z4/4x/tick-only | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 529 | 0.0 / 0.0 / 0.0 / 0.0 / 0 | None |
| W0/running/Z4/4x/tick-only | pacing_ms | 18.7715 / 62.101 / 216.571 / 1894.082 / 529 | 16.6145 / 18.066 / 18.936 / 21.093 / 0 | 1.129826356495832 |
| W0/running/Z4/4x/tick-only | ticks_run | 2.0 / 5.0 / 5.0 / 5.0 / 529 | 2.0 / 2.0 / 2.0 / 2.0 / 0 | 1.0 |
| W1/frozen/Z4/0x | primary_cpu_ms | 15.9930000066757 / 16.3409999998808 / 18.2379999930859 / 19.0219999930859 / 36 | 9.055999999523166 / 9.53999999988079 / 9.95599999296665 / 10.5629999930859 / 0 | 1.766011484929085 |
| W1/frozen/Z4/0x | process_ms | 15.847 / 16.195 / 18.087 / 18.871 / 36 | 8.909500000000001 / 9.386 / 9.797 / 10.412 / 0 | 1.7786632246478475 |
| W1/frozen/Z4/0x | render_cpu_ms | 0.146999999880791 / 0.159999996423721 / 0.172000005841255 / 0.187000006437302 / 36 | 0.146999999880791 / 0.165999993681908 / 0.187000006437302 / 0.20100000500679 / 0 | 1.0 |
| W1/frozen/Z4/0x | setup_cpu_ms | 0.011 / 0.015 / 0.02 / 0.023 / 36 | 0.01 / 0.015 / 0.02 / 0.024 / 0 | 1.0999999999999999 |
| W1/frozen/Z4/0x | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 36 | 0.0 / 0.0 / 0.0 / 0.0 / 0 | None |
| W1/frozen/Z4/0x | pacing_ms | 17.303 / 17.749 / 19.576 / 20.538 / 36 | 10.351500000000001 / 10.924 / 11.418 / 12.035 / 0 | 1.6715451866879194 |
| W1/frozen/Z4/0x | ticks_run | 0.0 / 0.0 / 0.0 / 0.0 / 36 | 0.0 / 0.0 / 0.0 / 0.0 / 0 | None |
| W1/frozen/Z1/0x | primary_cpu_ms | 16.272500000238452 / 16.6590000140667 / 18.4510000066757 / 18.8079999876022 / 84 | 9.73600000143051 / 10.0159999873638 / 10.6309999916553 / 10.8069999902248 / 0 | 1.6713742807978158 |
| W1/frozen/Z1/0x | process_ms | 15.997499999999999 / 16.381 / 18.183 / 18.52 / 84 | 9.4345 / 9.705 / 10.341 / 10.503 / 0 | 1.6956383486141289 |
| W1/frozen/Z1/0x | render_cpu_ms | 0.272000014781952 / 0.287000000476837 / 0.301999986171722 / 0.319000005722046 / 84 | 0.293999999761581 / 0.319999992847443 / 0.342000007629395 / 0.363999992609024 / 0 | 0.925170119056225 |
| W1/frozen/Z1/0x | setup_cpu_ms | 0.011 / 0.015 / 0.019 / 0.023 / 84 | 0.016 / 0.02 / 0.023 / 0.032 / 0 | 0.6875 |
| W1/frozen/Z1/0x | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 84 | 0.0 / 0.0 / 0.0 / 0.0 / 0 | None |
| W1/frozen/Z1/0x | pacing_ms | 17.678 / 18.157 / 19.925 / 20.299 / 84 | 11.2305 / 11.596 / 12.238 / 12.88 / 0 | 1.5741062285739729 |
| W1/frozen/Z1/0x | ticks_run | 0.0 / 0.0 / 0.0 / 0.0 / 84 | 0.0 / 0.0 / 0.0 / 0.0 / 0 | None |
| W1/running/Z4/2x | primary_cpu_ms | 21.43000000154975 / 25.0919999991655 / 26.864999997139 / 64.4259999951124 / 717 | 13.1835000048876 / 16.9739999957085 / 19.5800000010729 / 21.687999994874 / 119 | 1.6255167439302822 |
| W1/running/Z4/2x | process_ms | 21.2785 / 24.927 / 26.702 / 64.274 / 717 | 13.026499999999999 / 16.808 / 19.403 / 21.492 / 119 | 1.6334779104133883 |
| W1/running/Z4/2x | render_cpu_ms | 0.153999999165535 / 0.171000003814697 / 0.187000006437302 / 0.199000000953674 / 717 | 0.162000000476837 / 0.184000000357628 / 0.200000002980232 / 0.219999998807907 / 119 | 0.9506172760015156 |
| W1/running/Z4/2x | setup_cpu_ms | 0.012 / 0.015 / 0.02 / 0.022 / 717 | 0.016 / 0.018 / 0.022 / 0.025 / 119 | 0.75 |
| W1/running/Z4/2x | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 717 | 0.0 / 0.0 / 0.0 / 0.0 / 119 | None |
| W1/running/Z4/2x | pacing_ms | 22.8155 / 26.521 / 28.227 / 66.039 / 717 | 17.205 / 18.477 / 21.051 / 23.271 / 119 | 1.3260970648067423 |
| W1/running/Z4/2x | ticks_run | 1.0 / 2.0 / 2.0 / 4.0 / 717 | 1.0 / 1.0 / 2.0 / 2.0 / 119 | 1.0 |
| W1/running/Z4/2x/tick-only | primary_cpu_ms | 21.43000000154975 / 25.0919999991655 / 26.864999997139 / 64.4259999951124 / 717 | 13.1835000048876 / 16.9739999957085 / 19.5800000010729 / 21.687999994874 / 119 | 1.6255167439302822 |
| W1/running/Z4/2x/tick-only | process_ms | 21.2785 / 24.927 / 26.702 / 64.274 / 717 | 13.026499999999999 / 16.808 / 19.403 / 21.492 / 119 | 1.6334779104133883 |
| W1/running/Z4/2x/tick-only | render_cpu_ms | 0.153999999165535 / 0.171000003814697 / 0.187000006437302 / 0.199000000953674 / 717 | 0.162000000476837 / 0.184000000357628 / 0.200000002980232 / 0.219999998807907 / 119 | 0.9506172760015156 |
| W1/running/Z4/2x/tick-only | setup_cpu_ms | 0.012 / 0.015 / 0.02 / 0.022 / 717 | 0.016 / 0.018 / 0.022 / 0.025 / 119 | 0.75 |
| W1/running/Z4/2x/tick-only | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 717 | 0.0 / 0.0 / 0.0 / 0.0 / 119 | None |
| W1/running/Z4/2x/tick-only | pacing_ms | 22.8155 / 26.521 / 28.227 / 66.039 / 717 | 17.205 / 18.477 / 21.051 / 23.271 / 119 | 1.3260970648067423 |
| W1/running/Z4/2x/tick-only | ticks_run | 1.0 / 2.0 / 2.0 / 4.0 / 717 | 1.0 / 1.0 / 2.0 / 2.0 / 119 | 1.0 |
| W1/running/Z4/4x | primary_cpu_ms | 31.571999997019752 / 34.547999997139 / 37.27400000453 / 38.3360000038147 / 900 | 21.18300000345705 / 23.0570000017881 / 24.5840000032187 / 57.2690000011921 / 602 | 1.4904404471447494 |
| W1/running/Z4/4x | process_ms | 31.3955 / 34.395 / 37.106 / 38.165 / 900 | 21.008499999999998 / 22.887 / 24.428 / 57.114 / 602 | 1.4944189256729419 |
| W1/running/Z4/4x | render_cpu_ms | 0.164000004529953 / 0.180999994277954 / 0.194000005722046 / 0.216000005602837 / 900 | 0.165999993681908 / 0.186000004410744 / 0.203999996185303 / 0.214000001549721 / 602 | 0.9879518721200231 |
| W1/running/Z4/4x | setup_cpu_ms | 0.013 / 0.015 / 0.019 / 0.023 / 900 | 0.014 / 0.017 / 0.02 / 0.023 / 602 | 0.9285714285714285 |
| W1/running/Z4/4x | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 900 | 0.0 / 0.0 / 0.0 / 0.0 / 602 | None |
| W1/running/Z4/4x | pacing_ms | 32.994 / 36.1 / 38.731 / 39.782 / 900 | 22.624499999999998 / 24.56 / 26.042 / 58.958 / 602 | 1.4583305708413448 |
| W1/running/Z4/4x | ticks_run | 4.0 / 4.0 / 5.0 / 5.0 / 900 | 3.0 / 3.0 / 3.0 / 5.0 / 602 | 1.3333333333333333 |
| W1/running/Z4/4x/tick-only | primary_cpu_ms | 31.571999997019752 / 34.547999997139 / 37.27400000453 / 38.3360000038147 / 900 | 21.18300000345705 / 23.0570000017881 / 24.5840000032187 / 57.2690000011921 / 602 | 1.4904404471447494 |
| W1/running/Z4/4x/tick-only | process_ms | 31.3955 / 34.395 / 37.106 / 38.165 / 900 | 21.008499999999998 / 22.887 / 24.428 / 57.114 / 602 | 1.4944189256729419 |
| W1/running/Z4/4x/tick-only | render_cpu_ms | 0.164000004529953 / 0.180999994277954 / 0.194000005722046 / 0.216000005602837 / 900 | 0.165999993681908 / 0.186000004410744 / 0.203999996185303 / 0.214000001549721 / 602 | 0.9879518721200231 |
| W1/running/Z4/4x/tick-only | setup_cpu_ms | 0.013 / 0.015 / 0.019 / 0.023 / 900 | 0.014 / 0.017 / 0.02 / 0.023 / 602 | 0.9285714285714285 |
| W1/running/Z4/4x/tick-only | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 900 | 0.0 / 0.0 / 0.0 / 0.0 / 602 | None |
| W1/running/Z4/4x/tick-only | pacing_ms | 32.994 / 36.1 / 38.731 / 39.782 / 900 | 22.624499999999998 / 24.56 / 26.042 / 58.958 / 602 | 1.4583305708413448 |
| W1/running/Z4/4x/tick-only | ticks_run | 4.0 / 4.0 / 5.0 / 5.0 / 900 | 3.0 / 3.0 / 3.0 / 5.0 / 602 | 1.3333333333333333 |
| W200/frozen/Z4/0x | primary_cpu_ms | 35.199000000655644 / 35.8080000008345 / 37.5219999941587 / 39.262000002265 / 900 | 29.6904999936819 / 31.0039999883175 / 32.8339999902248 / 33.498999994278 / 900 | 1.1855307255905405 |
| W200/frozen/Z4/0x | process_ms | 34.9945 / 35.587 / 37.319 / 39.055 / 900 | 29.3995 / 30.713 / 32.535 / 33.193 / 900 | 1.1903093590027043 |
| W200/frozen/Z4/0x | render_cpu_ms | 0.20100000500679 / 0.219999998807907 / 0.233999997377396 / 0.273000001907349 / 900 | 0.293000012636185 / 0.317999988794327 / 0.345999985933304 / 0.368999987840652 / 900 | 0.6860068134412319 |
| W200/frozen/Z4/0x | setup_cpu_ms | 0.029 / 0.036 / 0.045 / 0.053 / 900 | 0.034 / 0.04 / 0.049 / 0.068 / 900 | 0.8529411764705882 |
| W200/frozen/Z4/0x | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 900 | 0.0 / 0.0 / 0.0 / 0.0 / 900 | None |
| W200/frozen/Z4/0x | pacing_ms | 36.8365 / 37.494 / 39.231 / 41.038 / 900 | 30.6755 / 31.992 / 33.897 / 36.61 / 900 | 1.200844322015941 |
| W200/frozen/Z4/0x | ticks_run | 0.0 / 0.0 / 0.0 / 0.0 / 900 | 0.0 / 0.0 / 0.0 / 0.0 / 900 | None |
| W200/frozen/Z1/0x | primary_cpu_ms | 35.4369999936819 / 35.9570000109673 / 37.5660000109673 / 38.2499999980927 / 900 | 30.42099999773505 / 40.2430000114441 / 93.9499998550415 / 187.802000013113 / 900 | 1.1648860982978966 |
| W200/frozen/Z1/0x | process_ms | 35.085499999999996 / 35.606 / 37.215 / 37.898 / 900 | 30.038 / 39.838 / 89.44 / 187.347 / 900 | 1.1680371529396096 |
| W200/frozen/Z1/0x | render_cpu_ms | 0.349000006914139 / 0.368000000715256 / 0.391000002622604 / 0.416999995708466 / 900 | 0.381999999284744 / 0.50900000333786 / 1.2940000295639 / 4.8769998550415 / 900 | 0.9136125852555128 |
| W200/frozen/Z1/0x | setup_cpu_ms | 0.029 / 0.035 / 0.043 / 0.048 / 900 | 0.036 / 0.046 / 0.136 / 0.294 / 900 | 0.8055555555555557 |
| W200/frozen/Z1/0x | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 900 | 0.0 / 0.0 / 0.0 / 0.0 / 900 | None |
| W200/frozen/Z1/0x | pacing_ms | 36.971999999999994 / 37.571 / 39.067 / 40.042 / 900 | 32.156 / 42.47 / 100.354 / 191.227 / 900 | 1.1497698718746112 |
| W200/frozen/Z1/0x | ticks_run | 0.0 / 0.0 / 0.0 / 0.0 / 900 | 0.0 / 0.0 / 0.0 / 0.0 / 900 | None |
| W200/running/Z4/2x | primary_cpu_ms | 45.157499996304495 / 48.2760000070333 / 50.9439999946356 / 52.9199999933243 / 900 | 45.3485000038147 / 72.5940000100136 / 105.306999996662 / 1024.48900000572 / 900 | 0.995788173644241 |
| W200/running/Z4/2x | process_ms | 44.913 / 48.078 / 50.704 / 52.688 / 900 | 45.0895 / 72.35 / 104.691 / 1024.17 / 900 | 0.9960855631577196 |
| W200/running/Z4/2x | render_cpu_ms | 0.20100000500679 / 0.231000006198883 / 0.261000007390976 / 0.291000008583069 / 900 | 0.291000008583069 / 0.691999971866608 / 0.990000009536743 / 1.63699996471405 / 900 | 0.690721646317108 |
| W200/running/Z4/2x | setup_cpu_ms | 0.026 / 0.031 / 0.036 / 0.044 / 900 | 0.03 / 0.074 / 0.119 / 0.291 / 900 | 0.8666666666666667 |
| W200/running/Z4/2x | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 900 | 0.0 / 0.0 / 0.0 / 0.0 / 900 | None |
| W200/running/Z4/2x | pacing_ms | 46.920500000000004 / 50.005 / 52.767 / 54.704 / 900 | 47.025499999999994 / 74.659 / 107.846 / 1026.057 / 900 | 0.9977671688764609 |
| W200/running/Z4/2x | ticks_run | 3.0 / 3.0 / 3.0 / 5.0 / 900 | 3.0 / 4.0 / 5.0 / 5.0 / 900 | 1.0 |
| W200/running/Z4/2x/tick-only | primary_cpu_ms | 45.157499996304495 / 48.2760000070333 / 50.9439999946356 / 52.9199999933243 / 900 | 45.3485000038147 / 72.5940000100136 / 105.306999996662 / 1024.48900000572 / 900 | 0.995788173644241 |
| W200/running/Z4/2x/tick-only | process_ms | 44.913 / 48.078 / 50.704 / 52.688 / 900 | 45.0895 / 72.35 / 104.691 / 1024.17 / 900 | 0.9960855631577196 |
| W200/running/Z4/2x/tick-only | render_cpu_ms | 0.20100000500679 / 0.231000006198883 / 0.261000007390976 / 0.291000008583069 / 900 | 0.291000008583069 / 0.691999971866608 / 0.990000009536743 / 1.63699996471405 / 900 | 0.690721646317108 |
| W200/running/Z4/2x/tick-only | setup_cpu_ms | 0.026 / 0.031 / 0.036 / 0.044 / 900 | 0.03 / 0.074 / 0.119 / 0.291 / 900 | 0.8666666666666667 |
| W200/running/Z4/2x/tick-only | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 900 | 0.0 / 0.0 / 0.0 / 0.0 / 900 | None |
| W200/running/Z4/2x/tick-only | pacing_ms | 46.920500000000004 / 50.005 / 52.767 / 54.704 / 900 | 47.025499999999994 / 74.659 / 107.846 / 1026.057 / 900 | 0.9977671688764609 |
| W200/running/Z4/2x/tick-only | ticks_run | 3.0 / 3.0 / 3.0 / 5.0 / 900 | 3.0 / 4.0 / 5.0 / 5.0 / 900 | 1.0 |
| W200/running/Z4/4x | primary_cpu_ms | 32.4085000050068 / 39.71099999547 / 41.934999992609 / 43.0059999911785 / 900 | 31.471499997437 / 55.3720000219345 / 79.9230000047684 / 139.70099998188 / 889 | 1.029772969437304 |
| W200/running/Z4/4x | process_ms | 32.208 / 39.5 / 41.696 / 42.753 / 900 | 31.168 / 54.795 / 79.678 / 138.857 / 889 | 1.0333675564681724 |
| W200/running/Z4/4x | render_cpu_ms | 0.202000007033348 / 0.229000002145767 / 0.254999995231628 / 0.261999994516373 / 900 | 0.2654999941587445 / 0.648999989032745 / 1.09399998188019 / 2.66199994087219 / 889 | 0.7608286684653207 |
| W200/running/Z4/4x | setup_cpu_ms | 0.02 / 0.024 / 0.03 / 0.033 / 900 | 0.024 / 0.057 / 0.072 / 0.162 / 889 | 0.8333333333333334 |
| W200/running/Z4/4x | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 900 | 0.0 / 0.0 / 0.0 / 0.0 / 889 | None |
| W200/running/Z4/4x | pacing_ms | 34.021 / 41.518 / 43.951 / 44.621 / 900 | 33.4925 / 58.045 / 84.701 / 150.234 / 889 | 1.0157796521609317 |
| W200/running/Z4/4x | ticks_run | 4.0 / 5.0 / 5.0 / 5.0 / 900 | 4.0 / 5.0 / 5.0 / 5.0 / 889 | 1.0 |
| W200/running/Z4/4x/tick-only | primary_cpu_ms | 32.4085000050068 / 39.71099999547 / 41.934999992609 / 43.0059999911785 / 900 | 31.471499997437 / 55.3720000219345 / 79.9230000047684 / 139.70099998188 / 889 | 1.029772969437304 |
| W200/running/Z4/4x/tick-only | process_ms | 32.208 / 39.5 / 41.696 / 42.753 / 900 | 31.168 / 54.795 / 79.678 / 138.857 / 889 | 1.0333675564681724 |
| W200/running/Z4/4x/tick-only | render_cpu_ms | 0.202000007033348 / 0.229000002145767 / 0.254999995231628 / 0.261999994516373 / 900 | 0.2654999941587445 / 0.648999989032745 / 1.09399998188019 / 2.66199994087219 / 889 | 0.7608286684653207 |
| W200/running/Z4/4x/tick-only | setup_cpu_ms | 0.02 / 0.024 / 0.03 / 0.033 / 900 | 0.024 / 0.057 / 0.072 / 0.162 / 889 | 0.8333333333333334 |
| W200/running/Z4/4x/tick-only | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 900 | 0.0 / 0.0 / 0.0 / 0.0 / 889 | None |
| W200/running/Z4/4x/tick-only | pacing_ms | 34.021 / 41.518 / 43.951 / 44.621 / 900 | 33.4925 / 58.045 / 84.701 / 150.234 / 889 | 1.0157796521609317 |
| W200/running/Z4/4x/tick-only | ticks_run | 4.0 / 5.0 / 5.0 / 5.0 / 900 | 4.0 / 5.0 / 5.0 / 5.0 / 889 | 1.0 |
| W500/frozen/Z4/0x | primary_cpu_ms | 67.57949999976155 / 68.8040000071526 / 71.2170000027418 / 72.6900000100136 / 900 | 51.5325000092983 / 52.2609999864101 / 53.0210000066757 / 53.6500000026226 / 900 | 1.3113957209055993 |
| W500/frozen/Z4/0x | process_ms | 67.304 / 68.514 / 70.973 / 72.413 / 900 | 51.266000000000005 / 51.978 / 52.753 / 53.384 / 900 | 1.312838918581516 |
| W500/frozen/Z4/0x | render_cpu_ms | 0.270999997854233 / 0.300999999046326 / 0.326999992132187 / 0.374000012874603 / 900 | 0.264999985694885 / 0.296000003814697 / 0.319000005722046 / 0.349999994039536 / 900 | 1.0226415565405211 |
| W500/frozen/Z4/0x | setup_cpu_ms | 0.064 / 0.08 / 0.095 / 0.101 / 900 | 0.064 / 0.074 / 0.087 / 0.1 / 900 | 1.0 |
| W500/frozen/Z4/0x | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 900 | 0.0 / 0.0 / 0.0 / 0.0 / 900 | None |
| W500/frozen/Z4/0x | pacing_ms | 70.0275 / 71.355 / 73.68 / 75.149 / 900 | 53.9155 / 54.728 / 55.509 / 56.44 / 900 | 1.2988379964945147 |
| W500/frozen/Z4/0x | ticks_run | 0.0 / 0.0 / 0.0 / 0.0 / 900 | 0.0 / 0.0 / 0.0 / 0.0 / 900 | None |
| W500/frozen/Z1/0x | primary_cpu_ms | 69.03900001096724 / 71.6109999861717 / 79.8269999871254 / 106.146000006676 / 900 | 51.72150000023845 / 52.4780000050068 / 53.3020000185967 / 56.6609999995232 / 900 | 1.3348220761317624 |
| W500/frozen/Z1/0x | process_ms | 68.5305 / 71.068 / 79.314 / 105.503 / 900 | 51.2585 / 52.002 / 52.789 / 55.948 / 900 | 1.3369587483051593 |
| W500/frozen/Z1/0x | render_cpu_ms | 0.497999995946884 / 0.577000021934509 / 0.683000028133392 / 1.29499995708466 / 900 | 0.46399998664856 / 0.495999991893768 / 0.532999992370605 / 0.712999999523163 / 900 | 1.0732758842169452 |
| W500/frozen/Z1/0x | setup_cpu_ms | 0.07 / 0.086 / 0.121 / 0.284 / 900 | 0.066 / 0.074 / 0.09 / 0.125 / 900 | 1.0606060606060606 |
| W500/frozen/Z1/0x | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 900 | 0.0 / 0.0 / 0.0 / 0.0 / 900 | None |
| W500/frozen/Z1/0x | pacing_ms | 71.98400000000001 / 74.711 / 83.549 / 114.394 / 900 | 54.5315 / 55.304 / 56.176 / 60.271 / 900 | 1.3200443780200435 |
| W500/frozen/Z1/0x | ticks_run | 0.0 / 0.0 / 0.0 / 0.0 / 900 | 0.0 / 0.0 / 0.0 / 0.0 / 900 | None |
| W500/running/Z4/2x | primary_cpu_ms | 68.29799999809265 / 186.902999999762 / 552.813999993086 / 1707.665 / 900 | 36.13349999326465 / 163.052999999046 / 168.29400000453 / 173.741999999762 / 900 | 1.8901573335221753 |
| W500/running/Z4/2x | process_ms | 67.76650000000001 / 186.484 / 552.538 / 1707.29 / 900 | 35.878 / 162.752 / 168.005 / 173.448 / 900 | 1.88880372373042 |
| W500/running/Z4/2x | render_cpu_ms | 0.333000004291534 / 0.483999997377396 / 1.41400003433228 / 5.15000009536743 / 900 | 0.263000011444092 / 0.303999990224838 / 0.333000004291534 / 0.365000009536743 / 900 | 1.266159657039872 |
| W500/running/Z4/2x | setup_cpu_ms | 0.036 / 0.069 / 0.136 / 3.672 / 900 | 0.031 / 0.06 / 0.069 / 0.081 / 900 | 1.161290322580645 |
| W500/running/Z4/2x | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 900 | 0.0 / 0.0 / 0.0 / 0.0 / 900 | None |
| W500/running/Z4/2x | pacing_ms | 70.632 / 189.203 / 567.642 / 1714.69 / 900 | 37.971000000000004 / 165.852 / 171.343 / 176.931 / 900 | 1.860156435174212 |
| W500/running/Z4/2x | ticks_run | 4.0 / 5.0 / 5.0 / 5.0 / 900 | 2.0 / 5.0 / 5.0 / 5.0 / 900 | 2.0 |
| W500/running/Z4/2x/tick-only | primary_cpu_ms | 68.29799999809265 / 186.902999999762 / 552.813999993086 / 1707.665 / 900 | 36.13349999326465 / 163.052999999046 / 168.29400000453 / 173.741999999762 / 900 | 1.8901573335221753 |
| W500/running/Z4/2x/tick-only | process_ms | 67.76650000000001 / 186.484 / 552.538 / 1707.29 / 900 | 35.878 / 162.752 / 168.005 / 173.448 / 900 | 1.88880372373042 |
| W500/running/Z4/2x/tick-only | render_cpu_ms | 0.333000004291534 / 0.483999997377396 / 1.41400003433228 / 5.15000009536743 / 900 | 0.263000011444092 / 0.303999990224838 / 0.333000004291534 / 0.365000009536743 / 900 | 1.266159657039872 |
| W500/running/Z4/2x/tick-only | setup_cpu_ms | 0.036 / 0.069 / 0.136 / 3.672 / 900 | 0.031 / 0.06 / 0.069 / 0.081 / 900 | 1.161290322580645 |
| W500/running/Z4/2x/tick-only | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 900 | 0.0 / 0.0 / 0.0 / 0.0 / 900 | None |
| W500/running/Z4/2x/tick-only | pacing_ms | 70.632 / 189.203 / 567.642 / 1714.69 / 900 | 37.971000000000004 / 165.852 / 171.343 / 176.931 / 900 | 1.860156435174212 |
| W500/running/Z4/2x/tick-only | ticks_run | 4.0 / 5.0 / 5.0 / 5.0 / 900 | 2.0 / 5.0 / 5.0 / 5.0 / 900 | 2.0 |
| W500/running/Z4/4x | primary_cpu_ms | 39.135999985754495 / 63.9659999945164 / 97.0259999918938 / 232.554000012398 / 900 | 23.39700000292065 / 31.485000002265 / 34.4110000061989 / 36.0749999953508 / 792 | 1.6726930794917787 |
| W500/running/Z4/4x | process_ms | 38.843 / 63.583 / 96.405 / 232.217 / 900 | 23.186 / 31.278 / 34.18 / 35.842 / 792 | 1.6752781851117056 |
| W500/running/Z4/4x | render_cpu_ms | 0.25900000333786 / 0.639999985694885 / 0.922999978065491 / 2.43300008773804 / 900 | 0.200000002980232 / 0.234999999403954 / 0.266000002622604 / 0.284999996423721 / 792 | 1.2949999973922979 |
| W500/running/Z4/4x | setup_cpu_ms | 0.025 / 0.054 / 0.069 / 0.436 / 900 | 0.02 / 0.025 / 0.03 / 0.034 / 792 | 1.25 |
| W500/running/Z4/4x | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 900 | 0.0 / 0.0 / 0.0 / 0.0 / 792 | None |
| W500/running/Z4/4x | pacing_ms | 41.058 / 66.477 / 100.062 / 236.916 / 900 | 24.954 / 33.279 / 36.3 / 37.777 / 792 | 1.6453474392882903 |
| W500/running/Z4/4x | ticks_run | 5.0 / 5.0 / 5.0 / 5.0 / 900 | 3.0 / 4.0 / 5.0 / 5.0 / 792 | 1.6666666666666667 |
| W500/running/Z4/4x/tick-only | primary_cpu_ms | 39.135999985754495 / 63.9659999945164 / 97.0259999918938 / 232.554000012398 / 900 | 23.39700000292065 / 31.485000002265 / 34.4110000061989 / 36.0749999953508 / 792 | 1.6726930794917787 |
| W500/running/Z4/4x/tick-only | process_ms | 38.843 / 63.583 / 96.405 / 232.217 / 900 | 23.186 / 31.278 / 34.18 / 35.842 / 792 | 1.6752781851117056 |
| W500/running/Z4/4x/tick-only | render_cpu_ms | 0.25900000333786 / 0.639999985694885 / 0.922999978065491 / 2.43300008773804 / 900 | 0.200000002980232 / 0.234999999403954 / 0.266000002622604 / 0.284999996423721 / 792 | 1.2949999973922979 |
| W500/running/Z4/4x/tick-only | setup_cpu_ms | 0.025 / 0.054 / 0.069 / 0.436 / 900 | 0.02 / 0.025 / 0.03 / 0.034 / 792 | 1.25 |
| W500/running/Z4/4x/tick-only | gpu_ms | 0.0 / 0.0 / 0.0 / 0.0 / 900 | 0.0 / 0.0 / 0.0 / 0.0 / 792 | None |
| W500/running/Z4/4x/tick-only | pacing_ms | 41.058 / 66.477 / 100.062 / 236.916 / 900 | 24.954 / 33.279 / 36.3 / 37.777 / 792 | 1.6453474392882903 |
| W500/running/Z4/4x/tick-only | ticks_run | 5.0 / 5.0 / 5.0 / 5.0 / 900 | 3.0 / 4.0 / 5.0 / 5.0 / 792 | 1.6666666666666667 |
| W1000/frozen/Z4/0x | primary_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/frozen/Z4/0x | process_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/frozen/Z4/0x | render_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/frozen/Z4/0x | setup_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/frozen/Z4/0x | gpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/frozen/Z4/0x | pacing_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/frozen/Z4/0x | ticks_run | NOT_MEASURED | NOT_MEASURED | None |
| W1000/frozen/Z1/0x | primary_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/frozen/Z1/0x | process_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/frozen/Z1/0x | render_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/frozen/Z1/0x | setup_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/frozen/Z1/0x | gpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/frozen/Z1/0x | pacing_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/frozen/Z1/0x | ticks_run | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/2x | primary_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/2x | process_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/2x | render_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/2x | setup_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/2x | gpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/2x | pacing_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/2x | ticks_run | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/2x/tick-only | primary_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/2x/tick-only | process_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/2x/tick-only | render_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/2x/tick-only | setup_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/2x/tick-only | gpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/2x/tick-only | pacing_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/2x/tick-only | ticks_run | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/4x | primary_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/4x | process_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/4x | render_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/4x | setup_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/4x | gpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/4x | pacing_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/4x | ticks_run | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/4x/tick-only | primary_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/4x/tick-only | process_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/4x/tick-only | render_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/4x/tick-only | setup_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/4x/tick-only | gpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/4x/tick-only | pacing_ms | NOT_MEASURED | NOT_MEASURED | None |
| W1000/running/Z4/4x/tick-only | ticks_run | NOT_MEASURED | NOT_MEASURED | None |
| W2000/frozen/Z4/0x | primary_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/frozen/Z4/0x | process_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/frozen/Z4/0x | render_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/frozen/Z4/0x | setup_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/frozen/Z4/0x | gpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/frozen/Z4/0x | pacing_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/frozen/Z4/0x | ticks_run | NOT_MEASURED | NOT_MEASURED | None |
| W2000/frozen/Z1/0x | primary_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/frozen/Z1/0x | process_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/frozen/Z1/0x | render_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/frozen/Z1/0x | setup_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/frozen/Z1/0x | gpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/frozen/Z1/0x | pacing_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/frozen/Z1/0x | ticks_run | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/2x | primary_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/2x | process_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/2x | render_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/2x | setup_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/2x | gpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/2x | pacing_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/2x | ticks_run | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/2x/tick-only | primary_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/2x/tick-only | process_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/2x/tick-only | render_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/2x/tick-only | setup_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/2x/tick-only | gpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/2x/tick-only | pacing_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/2x/tick-only | ticks_run | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/4x | primary_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/4x | process_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/4x | render_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/4x | setup_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/4x | gpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/4x | pacing_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/4x | ticks_run | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/4x/tick-only | primary_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/4x/tick-only | process_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/4x/tick-only | render_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/4x/tick-only | setup_cpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/4x/tick-only | gpu_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/4x/tick-only | pacing_ms | NOT_MEASURED | NOT_MEASURED | None |
| W2000/running/Z4/4x/tick-only | ticks_run | NOT_MEASURED | NOT_MEASURED | None |

## Ablation before / after

| World | Node/group | Before ms | After ms |
|---|---|---:|---:|
| W0 | ActivityTrailRenderer | 0.20799999862910035 | 0.08550000065564989 |
| W0 | AgentRenderer | 5.70800001150374 | 2.153000000953675 |
| W0 | AgentRenderer/FoundingGroups | 0.20150000339744878 | 0.03200000241398504 |
| W0 | BuildingRenderer | 0.5054999911188958 | 0.07400000137091034 |
| W0 | BuildingRenderer/NightChillLayer | 0.1804999955296509 | -0.2739999972581848 |
| W0 | DaylightController | 0.013499995291251565 | 0.04350000137090504 |
| W0 | DeathVizRenderer | 0.104000003457001 | -0.019999998629095295 |
| W0 | GodHandEffects | 0.2855000003576098 | -0.022499994277954904 |
| W0 | NatureRenderer | 0.7100000000000115 | 0.3750000006556502 |
| W0 | NeedBarRenderer | 0.38250000810619866 | -0.04350000345707006 |
| W0 | SeekVizRenderer | 0.28099999290710187 | 0.15800000166893513 |
| W0 | SettlementLabels | 0.9290000016093387 | 0.27399999794364494 |
| W0 | SettlementOverviewRenderer | -0.195500004649098 | -0.012000001400704896 |
| W0 | Shadows | 0.00299999713895005 | 0.05949999946356055 |
| W0 | TerrainRenderer | -0.18100000500679947 | -0.0685000016093249 |
| W0 | UI/AgentInspectorPanel | -0.03499999529129916 | -0.0840000081062291 |
| W0 | UI/GodHandPanel | 0.11000000059604886 | 0.2505000043511396 |
| W0 | UI/HudStatusPanel | 0.46699999755622024 | 0.01599999457597523 |
| W0 | UI/HudTopbar | 0.9234999998807858 | 0.16000000196695519 |
| W0 | WorldEffects | -0.14749999380110168 | 0.12000000643730502 |
| W0 | WorldObjects | -0.11750000062580135 | 0.04300000047683472 |
| W0 | WorldRenderer | 0.06299999701974812 | -0.3350000098347654 |
| W0 | ZoomLodController | 0.09200000989440049 | 0.00500000101328002 |
| W500 | ActivityTrailRenderer | 0.9609999990462939 | 0.3024999864101474 |
| W500 | AgentRenderer | 56.49199999570849 | 46.3584999961853 |
| W500 | AgentRenderer/FoundingGroups | 1.2625000075102122 | 0.17849999523165394 |
| W500 | BuildingRenderer | 0.7164999974966406 | 0.06450000572205283 |
| W500 | BuildingRenderer/NightChillLayer | 0.17649998688699498 | -0.06050001418589801 |
| W500 | DaylightController | 0.5375000076294612 | 0.037999996781294954 |
| W500 | DeathVizRenderer | 0.015999996304500996 | -0.05999999183415383 |
| W500 | GodHandEffects | 0.2734999957084483 | 0.11349998235704817 |
| W500 | NatureRenderer | 1.9125000000000512 | 1.489500013768648 |
| W500 | NeedBarRenderer | 0.36199999737739574 | 0.37599999457599864 |
| W500 | SeekVizRenderer | 0.19299998676770258 | 0.051500008761891536 |
| W500 | SettlementLabels | 1.6849999951124346 | 0.6940000055431952 |
| W500 | SettlementOverviewRenderer | 0.08050000000001489 | 0.09199999856944885 |
| W500 | Shadows | 0.3029999998807682 | 0.06899999499324849 |
| W500 | TerrainRenderer | 0.6350000010728536 | 0.20550000345709662 |
| W500 | UI/AgentInspectorPanel | -0.0800000060796009 | 0.10600000584124558 |
| W500 | UI/GodHandPanel | 1.2375000029801981 | 0.09200000500674577 |
| W500 | UI/HudStatusPanel | 1.100499996662208 | 0.024999993562708767 |
| W500 | UI/HudTopbar | 1.7365000057220925 | 0.17950000476834305 |
| W500 | WorldEffects | 2.276999999403955 | 0.898000007212147 |
| W500 | WorldObjects | -0.27049999493360133 | 0.2854999989867508 |
| W500 | WorldRenderer | 0.02400000691410753 | 0.07900000774854732 |
| W500 | ZoomLodController | -0.00899999964234155 | 0.022000013113000705 |

## Build profiles

{"before": {"W0": "debug", "W500": "debug", "W1": "debug", "W200": "debug"}, "after": {"W0": "debug", "W500": "debug", "W1": "debug", "W200": "debug"}}
