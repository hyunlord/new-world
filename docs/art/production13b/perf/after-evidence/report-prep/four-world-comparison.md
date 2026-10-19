# Part4 terminal — 4개 승인 world 비교

단위 ms. 세션 간 기술적 비교이며 R로 보정하지 않았다. W500 running 역사값은 population loss로 제외되어 비율을 만들지 않는다.

| World / condition | before median | after median | after/before |
|---|---:|---:|---:|
| W0 frozen_Z4_0.0x | 2.823500 | 3.188000 | 1.129095 |
| W0 running_Z4_2.0x | 5.303500 | 5.262000 | 0.992175 |
| W1 frozen_Z4_0.0x | 6.297000 | 6.106000 | 0.969668 |
| W1 running_Z4_2.0x | 12.555000 | 12.050000 | 0.959777 |
| W200 frozen_Z4_0.0x | 16.257500 | 11.280500 | 0.693864 |
| W200 running_Z4_2.0x | 26.145000 | 17.039500 | 0.651731 |
| W500 frozen_Z4_0.0x | 28.403500 | 23.324500 | 0.821184 |
| W500 running_Z4_2.0x | EXCLUDED population loss | 41.240500 | N/A |

H1: 3.188000 ms / 기준 6 ms → PASS. H5: 23.324500 ms / 기준 16.67 ms → FAIL.

W-multi, sections 3개, ablation은 NOT_VERIFIED. Calibration 15개와 measurement 28회/56 endpoint와 정책 거부 1회를 comparison.md/json에 보존.
