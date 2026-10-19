# Part4 비교 초안 — 승인된 측정만 반영

전체 관문: 진행 중. PENDING은 미측정·미승인 상태이며 통과가 아니다. 비승인 raw 값은 after 표에 사용하지 않는다.

역사 R=2.0089961669873446; 현재 R=2.762010041042231. 시간 보정 없음. 아래 after/before는 세션 간 기술적 비교이며 후보의 인과 효과 판정은 동일 세션 paired 결과를 별도로 사용한다.

출처 동등성: 전체 manifest 동일 주장은 하지 않는다. identity-source-equivalence.md에 따라 추적 runtime source, compiled imported-resource bytes, loaded bridge 동등성과 measurement-local snapshot 안정성만 구분한다.

## H1–H11

| 항목 | 조건/지표 | before ms | after ms | after/before | 기존 기준 ms | 상태 |
| --- | --- | --- | --- | --- | --- | --- |
| H1 | W0 median | 2.823500 | 3.188000 | 1.129095 | 6 | PASS |
| H2 | W1 p99 | 13.921000 | 13.415000 | 0.963652 | 16.670000 | PASS |
| H3 | W200 median | 26.145000 | 17.039500 | 0.651731 | 16.670000 | FAIL |
| H4 | W200 median | 16.257500 | 11.280500 | 0.693864 | 10 | FAIL |
| H5 | W500 median | 28.403500 | 23.324500 | 0.821184 | 16.670000 | FAIL |
| H6 | 비수치  | 별도 역사 기록 | PENDING | PENDING | 별도 증거 | PENDING |
| H7 | 비수치  | 별도 역사 기록 | PENDING | PENDING | 별도 증거 | PENDING |
| H8 | 비수치  | 별도 역사 기록 | PENDING | PENDING | 별도 증거 | PENDING |
| H9 | 비수치  | 별도 역사 기록 | PENDING | PENDING | 별도 증거 | PENDING |
| H10 | 비수치  | 별도 역사 기록 | PENDING | PENDING | 별도 증거 | PENDING |
| H11 | 비수치  | 별도 역사 기록 | PENDING | PENDING | 별도 증거 | PENDING |

H6–H11은 성능 숫자로 대체할 수 없는 보고/identity/가독성/입력 등 증거를 포함한다. 이 초안은 별도 증거를 추정하지 않는다.

## Matrix


### W0


frozen_Z4_0.0x

| metric | median | p90 | p99 | max | mean |
| --- | --- | --- | --- | --- | --- |
| primary_cpu_ms | 3.188000 | 3.556000 | 3.969000 | 4.633000 | 3.198502 |
| process_ms | 3.023000 | 3.360000 | 3.818000 | 4.436000 | 3.035372 |
| render_cpu_ms | 0.154000 | 0.207000 | 0.233000 | 0.245000 | 0.163130 |
| setup_cpu_ms | 0.006000 | 0.010000 | 0.012000 | 0.016000 | 0.006629 |
| pacing_ms | 8.057000 | 9.807000 | 13.591000 | 18.932000 | 7.070304 |
| ticks_run | 0.000000 | 0.000000 | 0.000000 | 0.000000 | 0.000000 |

frames=900; ticks=0.0; tick-carrying frames=0

running_Z4_2.0x

| metric | median | p90 | p99 | max | mean |
| --- | --- | --- | --- | --- | --- |
| primary_cpu_ms | 5.262000 | 5.599000 | 5.862000 | 6.100000 | 5.272384 |
| process_ms | 5.117500 | 5.447000 | 5.697000 | 5.951000 | 5.125716 |
| render_cpu_ms | 0.146000 | 0.160000 | 0.177000 | 0.189000 | 0.146669 |
| setup_cpu_ms | 0.009000 | 0.011000 | 0.013000 | 0.017000 | 0.008911 |
| pacing_ms | 16.655500 | 17.871000 | 18.911000 | 19.563000 | 16.665866 |
| ticks_run | 1.000000 | 1.000000 | 1.000000 | 1.000000 | 0.998889 |

frames=900; ticks=899.0; tick-carrying frames=899

| block | frames | ticks | tick frames | pop start/end | tick start/end | excluded |
| --- | --- | --- | --- | --- | --- | --- |
| frozen_Z4_0 | 300 | 0.000000 | 0 | 29 → 29 | 40560 → 40560 | False |
| frozen_Z4_1 | 300 | 0.000000 | 0 | 29 → 29 | 40560 → 40560 | False |
| frozen_Z4_2 | 300 | 0.000000 | 0 | 29 → 29 | 40560 → 40560 | False |
| running_2x_0 | 300 | 299.000000 | 299 | 29 → 29 | 40560 → 40896 | False |
| running_2x_1 | 300 | 300.000000 | 300 | 29 → 29 | 40560 → 40896 | False |
| running_2x_2 | 300 | 300.000000 | 300 | 29 → 29 | 40560 → 40896 | False |

### W1


frozen_Z4_0.0x

| metric | median | p90 | p99 | max | mean |
| --- | --- | --- | --- | --- | --- |
| primary_cpu_ms | 6.106000 | 6.615000 | 10.959000 | 11.640000 | 6.204270 |
| process_ms | 5.936000 | 6.439000 | 10.772000 | 11.429000 | 6.034781 |
| render_cpu_ms | 0.164000 | 0.197000 | 0.242000 | 0.283000 | 0.169489 |
| setup_cpu_ms | 0.012000 | 0.017000 | 0.024000 | 0.029000 | 0.012542 |
| pacing_ms | 8.048500 | 8.713000 | 12.786000 | 14.813000 | 8.085551 |
| ticks_run | 0.000000 | 0.000000 | 0.000000 | 0.000000 | 0.000000 |

frames=900; ticks=0.0; tick-carrying frames=0

running_Z4_2.0x

| metric | median | p90 | p99 | max | mean |
| --- | --- | --- | --- | --- | --- |
| primary_cpu_ms | 12.050000 | 12.767000 | 13.415000 | 14.723000 | 12.079683 |
| process_ms | 11.882000 | 12.595000 | 13.229000 | 14.511000 | 11.909053 |
| render_cpu_ms | 0.169000 | 0.190000 | 0.209000 | 0.221000 | 0.170630 |
| setup_cpu_ms | 0.015000 | 0.019000 | 0.023000 | 0.032000 | 0.015103 |
| pacing_ms | 16.674000 | 17.471000 | 18.111000 | 18.834000 | 16.666769 |
| ticks_run | 1.000000 | 1.000000 | 1.000000 | 1.000000 | 1.000000 |

frames=900; ticks=900.0; tick-carrying frames=900

| block | frames | ticks | tick frames | pop start/end | tick start/end | excluded |
| --- | --- | --- | --- | --- | --- | --- |
| frozen_Z4_0 | 300 | 0.000000 | 0 | 71 → 71 | 126994 → 126994 | False |
| frozen_Z4_1 | 300 | 0.000000 | 0 | 71 → 71 | 126994 → 126994 | False |
| frozen_Z4_2 | 300 | 0.000000 | 0 | 71 → 71 | 126994 → 126994 | False |
| running_2x_0 | 300 | 300.000000 | 300 | 71 → 71 | 126994 → 127330 | False |
| running_2x_1 | 300 | 300.000000 | 300 | 71 → 71 | 126994 → 127330 | False |
| running_2x_2 | 300 | 300.000000 | 300 | 71 → 71 | 126994 → 127330 | False |

### W200


frozen_Z4_0.0x

| metric | median | p90 | p99 | max | mean |
| --- | --- | --- | --- | --- | --- |
| primary_cpu_ms | 11.280500 | 12.325000 | 12.971000 | 13.342000 | 11.345338 |
| process_ms | 11.068000 | 12.093000 | 12.752000 | 13.139000 | 11.136637 |
| render_cpu_ms | 0.205000 | 0.235000 | 0.273000 | 0.296000 | 0.208701 |
| setup_cpu_ms | 0.029000 | 0.037000 | 0.050000 | 0.057000 | 0.030250 |
| pacing_ms | 12.894500 | 14.004000 | 14.648000 | 15.048000 | 12.956069 |
| ticks_run | 0.000000 | 0.000000 | 0.000000 | 0.000000 | 0.000000 |

frames=900; ticks=0.0; tick-carrying frames=0

running_Z4_2.0x

| metric | median | p90 | p99 | max | mean |
| --- | --- | --- | --- | --- | --- |
| primary_cpu_ms | 17.039500 | 18.210000 | 19.785000 | 60.354000 | 17.296784 |
| process_ms | 16.837000 | 17.993000 | 19.523000 | 60.171000 | 17.096258 |
| render_cpu_ms | 0.197000 | 0.220000 | 0.265000 | 0.295000 | 0.200527 |
| setup_cpu_ms | 0.030000 | 0.037000 | 0.045000 | 0.077000 | 0.030903 |
| pacing_ms | 18.600000 | 19.835000 | 21.453000 | 62.299000 | 18.872819 |
| ticks_run | 1.000000 | 2.000000 | 2.000000 | 3.000000 | 1.132222 |

frames=900; ticks=1019.0; tick-carrying frames=900

| block | frames | ticks | tick frames | pop start/end | tick start/end | excluded |
| --- | --- | --- | --- | --- | --- | --- |
| frozen_Z4_0 | 300 | 0.000000 | 0 | 200 → 200 | 1440 → 1440 | False |
| frozen_Z4_1 | 300 | 0.000000 | 0 | 200 → 200 | 1440 → 1440 | False |
| frozen_Z4_2 | 300 | 0.000000 | 0 | 200 → 200 | 1440 → 1440 | False |
| running_2x_0 | 300 | 346.000000 | 300 | 200 → 200 | 1440 → 1832 | False |
| running_2x_1 | 300 | 338.000000 | 300 | 200 → 200 | 1440 → 1822 | False |
| running_2x_2 | 300 | 335.000000 | 300 | 200 → 200 | 1440 → 1818 | False |

### W500


frozen_Z4_0.0x

| metric | median | p90 | p99 | max | mean |
| --- | --- | --- | --- | --- | --- |
| primary_cpu_ms | 23.324500 | 24.236000 | 25.085000 | 25.760000 | 23.373186 |
| process_ms | 23.049500 | 23.947000 | 24.830000 | 25.501000 | 23.100014 |
| render_cpu_ms | 0.271000 | 0.299000 | 0.334000 | 0.366000 | 0.273171 |
| setup_cpu_ms | 0.071000 | 0.089000 | 0.109000 | 0.120000 | 0.073010 |
| pacing_ms | 25.612500 | 26.617000 | 27.548000 | 28.155000 | 25.639857 |
| ticks_run | 0.000000 | 0.000000 | 0.000000 | 0.000000 | 0.000000 |

frames=900; ticks=0.0; tick-carrying frames=0

running_Z4_2.0x

| metric | median | p90 | p99 | max | mean |
| --- | --- | --- | --- | --- | --- |
| primary_cpu_ms | 41.240500 | 42.977000 | 44.771000 | 46.189000 | 41.287593 |
| process_ms | 40.950000 | 42.701000 | 44.471000 | 45.876000 | 41.003704 |
| render_cpu_ms | 0.282000 | 0.318000 | 0.353000 | 0.375000 | 0.283889 |
| setup_cpu_ms | 0.070000 | 0.090000 | 0.105000 | 0.112000 | 0.073343 |
| pacing_ms | 43.737000 | 45.497000 | 47.311000 | 48.880000 | 43.765398 |
| ticks_run | 3.000000 | 3.000000 | 3.000000 | 5.000000 | 2.632222 |

frames=900; ticks=2369.0; tick-carrying frames=900

| block | frames | ticks | tick frames | pop start/end | tick start/end | excluded |
| --- | --- | --- | --- | --- | --- | --- |
| frozen_Z4_0 | 300 | 0.000000 | 0 | 500 → 500 | 1440 → 1440 | False |
| frozen_Z4_1 | 300 | 0.000000 | 0 | 500 → 500 | 1440 → 1440 | False |
| frozen_Z4_2 | 300 | 0.000000 | 0 | 500 → 500 | 1440 → 1440 | False |
| running_2x_0 | 300 | 798.000000 | 300 | 500 → 500 | 1440 → 2324 | False |
| running_2x_1 | 300 | 791.000000 | 300 | 500 → 500 | 1440 → 2318 | False |
| running_2x_2 | 300 | 780.000000 | 300 | 500 → 500 | 1440 → 2304 | False |

### W-multi

PENDING

## Exclusive sections / getter means


### W200

PENDING

### W-multi

PENDING

### W500

PENDING

## W200 node ablation

PENDING

## Calibration — 15 runs

| run | epoch | seconds | load1m | checksum |
| --- | --- | --- | --- | --- |
| 1 | 1792411868.931956 | 2.872518 | 17.449707 | 4216682400 |
| 2 | 1792411891.880399 | 2.838411 | 18.868652 | 4216682400 |
| 3 | 1792411914.773515 | 3.034052 | 14.947266 | 4216682400 |
| 4 | 1792411937.866375 | 2.754094 | 16.224609 | 4216682400 |
| 5 | 1792411960.685162 | 2.907054 | 13.933105 | 4216682400 |
| 6 | 1792411983.647981 | 3.359424 | 11.797852 | 4216682400 |
| 7 | 1792412007.065152 | 2.787543 | 13.531738 | 4216682400 |
| 8 | 1792412029.910710 | 2.807728 | 11.766602 | 4216682400 |
| 9 | 1792412052.777726 | 2.769624 | 11.410156 | 4216682400 |
| 10 | 1792412075.610689 | 2.762029 | 12.346680 | 4216682400 |
| 11 | 1792412098.429874 | 2.762010 | 11.361328 | 4216682400 |
| 12 | 1792412121.250281 | 2.765163 | 9.387207 | 4216682400 |
| 13 | 1792412144.069709 | 2.790781 | 9.095215 | 4216682400 |
| 14 | 1792412166.914020 | 2.761452 | 7.787598 | 4216682400 |
| 15 | 1792412189.734707 | 3.019692 | 9.985352 | 4216682400 |

## 모든 measurement endpoint — 거부 포함

| attempt | endpoint | epoch | seconds | load1m | verdict | exit | raw |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ablation-W200-attempt1.json.validity.json | start | 1792424248.818003 | 2.042412 | 17.242188 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| ablation-W200-attempt1.json.validity.json | end | 1792424250.899751 | 2.043479 | 17.242188 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| ablation-W200-attempt2.json.validity.json | start | 1792426387.145471 | 2.545099 | 38.081055 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| ablation-W200-attempt2.json.validity.json | end | 1792426389.732281 | 2.150803 | 35.912109 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| ablation-W200-attempt3.json.validity.json | start | 1792428524.357377 | 1.993955 | 8.122559 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| ablation-W200-attempt3.json.validity.json | end | 1792428526.384531 | 1.996908 | 8.032227 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| after-W-multi-attempt1.json.validity.json | start | 1792424053.589557 | 1.996135 | 4.944336 | NOT_VERIFIED (contended or unsuccessful) | 0 | True |
| after-W-multi-attempt1.json.validity.json | end | 1792424225.865098 | 2.021823 | 17.363770 | NOT_VERIFIED (contended or unsuccessful) | 0 | True |
| after-W-multi-attempt2.json.validity.json | start | 1792426360.378506 | 2.370871 | 36.236328 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| after-W-multi-attempt2.json.validity.json | end | 1792426362.794157 | 2.536675 | 36.236328 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| after-W-multi-attempt3.json.validity.json | start | 1792428334.324141 | 2.077279 | 5.889160 | NOT_VERIFIED (contended or unsuccessful) | 0 | True |
| after-W-multi-attempt3.json.validity.json | end | 1792428502.194582 | 2.097242 | 8.118652 | NOT_VERIFIED (contended or unsuccessful) | 0 | True |
| after-W0-attempt1.json.validity.json | start | 1792423762.297963 | 2.078322 | 8.832520 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| after-W0-attempt1.json.validity.json | end | 1792423764.416926 | 1.998670 | 8.765625 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| after-W0-attempt2.json.validity.json | start | 1792426291.311260 | 1.998475 | 7.833008 | NOT_VERIFIED (contended or unsuccessful) | 0 | True |
| after-W0-attempt2.json.validity.json | end | 1792426346.963676 | 3.663989 | 30.971191 | NOT_VERIFIED (contended or unsuccessful) | 0 | True |
| after-W0-attempt3.json.validity.json | start | 1792428193.178432 | 2.020753 | 4.925293 | VALID | 0 | True |
| after-W0-attempt3.json.validity.json | end | 1792428249.988165 | 2.036457 | 4.492188 | VALID | 0 | True |
| after-W1-attempt1.json.validity.json | start | 1792423773.305216 | 2.004006 | 7.958984 | VALID | 0 | True |
| after-W1-attempt1.json.validity.json | end | 1792424008.888269 | 2.047848 | 4.751953 | VALID | 0 | True |
| after-W200-attempt1.json.validity.json | start | 1792424012.521998 | 2.116670 | 4.751953 | VALID | 0 | True |
| after-W200-attempt1.json.validity.json | end | 1792424049.916903 | 2.097419 | 4.678711 | VALID | 0 | True |
| after-W500-attempt1.json.validity.json | start | 1792423767.840144 | 1.975206 | 8.765625 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| after-W500-attempt1.json.validity.json | end | 1792423769.851563 | 2.005951 | 8.303711 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| after-W500-attempt2.json.validity.json | start | 1792426352.327185 | 2.613776 | 32.014160 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| after-W500-attempt2.json.validity.json | end | 1792426354.988595 | 3.265273 | 34.254883 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| after-W500-attempt3.json.validity.json | start | 1792428253.580991 | 2.076803 | 4.212402 | VALID | 0 | True |
| after-W500-attempt3.json.validity.json | end | 1792428330.402663 | 2.102182 | 6.314941 | VALID | 0 | True |
| sections-W-multi-attempt1/frames.json.validity.json | start | 1792424236.660984 | 2.029208 | 16.218262 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| sections-W-multi-attempt1/frames.json.validity.json | end | 1792424238.729481 | 2.009588 | 17.722168 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| sections-W-multi-attempt2/frames.json.validity.json | start | 1792426374.255716 | 2.142577 | 38.159668 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| sections-W-multi-attempt2/frames.json.validity.json | end | 1792426376.441707 | 2.179716 | 37.986328 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| sections-W-multi-attempt3/frames.json.validity.json | start | 1792428512.462151 | 2.005621 | 8.414062 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| sections-W-multi-attempt3/frames.json.validity.json | end | 1792428514.502516 | 2.029941 | 8.414062 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| sections-W200-attempt1/frames.json.validity.json | start | 1792424230.248166 | 2.196788 | 16.934082 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| sections-W200-attempt1/frames.json.validity.json | end | 1792424232.485960 | 2.179264 | 16.934082 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| sections-W200-attempt2/frames.json.validity.json | start | 1792426367.845583 | 2.240443 | 37.738770 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| sections-W200-attempt2/frames.json.validity.json | end | 1792426370.129100 | 2.137166 | 38.159668 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| sections-W200-attempt3/frames.json.validity.json | start | 1792428506.565616 | 1.986578 | 8.188965 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| sections-W200-attempt3/frames.json.validity.json | end | 1792428508.585305 | 2.004651 | 8.188965 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| sections-W500-attempt1/frames.json.validity.json | start | 1792424242.735175 | 2.021585 | 17.722168 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| sections-W500-attempt1/frames.json.validity.json | end | 1792424244.795231 | 2.034775 | 17.263672 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| sections-W500-attempt2/frames.json.validity.json | start | 1792426380.668743 | 2.191086 | 37.827148 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| sections-W500-attempt2/frames.json.validity.json | end | 1792426382.899449 | 2.203791 | 37.827148 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| sections-W500-attempt3/frames.json.validity.json | start | 1792428518.451936 | 2.002093 | 8.220703 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
| sections-W500-attempt3/frames.json.validity.json | end | 1792428520.491459 | 1.988937 | 8.220703 | NOT_VERIFIED (contended or unsuccessful) | 75 | False |
