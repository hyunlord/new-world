# Stage B 완료된 동일성 항목 부분 검증

상태: **COMPLETED_SCOPE_PASS**. 생성 UTC: 2026-10-15T10:31:57.502394Z. 생산자 RUNLOG: `47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21`.

검증 범위는 완료된 I1·I3·I4·I2에 한정한다. I5 파일은 읽거나 판정하지 않았고, 최종 F7 및 핀 하네스 판정도 하지 않았다. 새로운 빌드·프로브·평가 실행이나 독립 최종 리뷰 라운드는 없다.

[전체 JSON](b_identity_partial_audit.json), SHA256 `6e5d2d3497b13426e012f9965c7513e5b1e4811ec90a260a48c0d24df285f023`. 입력 60개 파일은 검증 종료 시 다시 해시하여 모두 동일함을 확인했다.

## I1·I3·I4

기존 `compare_identity_pass17.py`의 엄격 비교를 사용했다. 재귀적으로 제외한 키는 `timing`, `wall_seconds`, `ms_per_agent_tick`뿐이다. 나머지 키 집합, 빈 컨테이너, 배열 순서/길이, 값과 타입 및 f64 표현을 비교했다.

| 항목 | 장면 | 시작 UTC | 종료 UTC | 세계·일수 | 비시간 차이 | 로그 마지막 줄 |
|---|---|---|---|---|---:|---|
| I1 | game_v19 | 2026-10-15T10:08:31Z | 2026-10-15T10:15:27Z | seeds 0–31, 32 × 120일 | 0 | `DETERMINISM PASS` |
| I3 | game_v16 | 2026-10-15T10:15:27Z | 2026-10-15T10:19:59Z | seeds 0–31, 32 × 120일 | 0 | `DETERMINISM PASS` |
| I4 | game_v20 | 2026-10-15T10:19:59Z | 2026-10-15T10:27:22Z | seeds 0–31, 32 × 120일 | 0 | `DETERMINISM PASS` |

세 요약 모두 32개의 중복 없는 seed 0–31, day key 1–120, 각 seed의 120일 생존/정착 일별 자료, `determinism_checked=true`를 만족한다. 각 로그는 32개 seed를 정확히 한 번씩 기록하고 `DETERMINISM PASS`로 끝난다. 저장소 요약은 해당 완료 원본과 바이트 해시가 같다.

## I2

12000·12001의 120일, every=60 기록이다. 두 에피소드는 각각 2,881행이며 로그는 `DONE episodes=2 wall_seconds=13.6`으로 끝난다. 기준/후보의 20개 명세 항목(18 NPY + 2 interventions.json)은 경로와 해시가 모두 같고, 양쪽 실제 파일 40개를 다시 해시하여 모두 일치했다. 메타데이터의 18개 배열 해시도 일치한다.

| 상대 경로 | 기준 명세 = 후보 명세 = 기준 실제 = 후보 실제 SHA256 |
|---|---|
| `./game_v19_s12000_none/building_keys.npy` | `55088d5ef75c36b9c0abcd64da206864e1b294b7e09bd11bc6e8e23ba8ba359c` |
| `./game_v19_s12000_none/building_present.npy` | `5a6bfd7bd98def7f912d8d0fa87370d73be6d2c2ea98129be98913472356ab7d` |
| `./game_v19_s12000_none/elevation.npy` | `0e19088555273a252dd22867323c857517897e50a62f268dfd8320b5a7d1633b` |
| `./game_v19_s12000_none/global.npy` | `7a46a6bd804c9d352c5b507e30832777fcd3c4e8f5a0d9bf5ee461b3b8e23135` |
| `./game_v19_s12000_none/interventions.json` | `37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
| `./game_v19_s12000_none/latent.npy` | `e303dc138cb77922f72234308388f73c433e3dc510566fae22fd1fdaf68d81c9` |
| `./game_v19_s12000_none/row_digest.npy` | `380dc00fdf29b6d13e8f1abde6f1fb47995d4c8a0e8587bbb3de8f7e21356bc0` |
| `./game_v19_s12000_none/settlement_ids.npy` | `16ce3579de63bc66efaea992be8b419b07b98cf1f558941f2c5f4b0abd50fe78` |
| `./game_v19_s12000_none/settlements.npy` | `4919fa276119ee6fc1ff12d6c134d174e4eb5dd5700c65f72a2434acc61871d1` |
| `./game_v19_s12000_none/terrain.npy` | `20665223d630f9888fae8624c427236e3e266c27e244e543d7b7a31c22bc4192` |
| `./game_v19_s12001_none/building_keys.npy` | `61818bd631afcda2f436d57ac63952ae673855f3100f106ad12801f2050cc31e` |
| `./game_v19_s12001_none/building_present.npy` | `8a9980093343df528ace56ffbc7f022f5bed75fc1f43ea7dd8d2eeb1b16e3d1f` |
| `./game_v19_s12001_none/elevation.npy` | `b568e26c0fa404ee7316e0d34b76542d9b0f836ab6ca55e23fa7dab598e0f5a2` |
| `./game_v19_s12001_none/global.npy` | `4ec2ccd80878e8c541cc58993c409e4707ad49526499113df48d0bda74fb2e44` |
| `./game_v19_s12001_none/interventions.json` | `37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
| `./game_v19_s12001_none/latent.npy` | `54b34a56fb704688d07174450483eb0aa6ec1d5e755a7c872ed8b306cf037fe9` |
| `./game_v19_s12001_none/row_digest.npy` | `99e138c98470b0e07b98197e338262cc64f4f485aa6899fa59ebe2a5a82642fe` |
| `./game_v19_s12001_none/settlement_ids.npy` | `16ce3579de63bc66efaea992be8b419b07b98cf1f558941f2c5f4b0abd50fe78` |
| `./game_v19_s12001_none/settlements.npy` | `0e5c57418e7dde0c05c7ff51cbcc413f3fe23d7ee30c02f9c1d35f6829a1a9b6` |
| `./game_v19_s12001_none/terrain.npy` | `cfd3878fecdcd4f90fc332b0b6bbde9da36533c45fff9c5e27036535557c6509` |

## 입력 파일 해시

| 경로 | 바이트 | SHA256 |
|---|---:|---|
| `/Users/rexxa/work/new-world-pass4/docs/revitalize/probe/p17-0_base/i1_v19/summary.json` | 3950085 | `09a66a9b3bda8cb410eff73a7641237e7c85b2c8947f06f926759dc440843984` |
| `/Users/rexxa/work/new-world-pass4/docs/revitalize/probe/p17-0_base/i2_sha256.txt` | 2092 | `1c8a7f8261a6f3093170391e110f79dbc0c3d111e6a399bc1a877cb8a363db84` |
| `/Users/rexxa/work/new-world-pass4/docs/revitalize/probe/p17-0_base/i3_v16/summary.json` | 2935134 | `efed1ea8d506a023d2d6cc5c3e8d5832f3dfb48e1432a7ac699965dbe97ef6e1` |
| `/Users/rexxa/work/new-world-pass4/docs/revitalize/probe/p17-0_base/i4_v20/summary.json` | 3961434 | `95102ee69f4cde2b4d09cf2be71a3ba1f3d3fd4d554a91eae8773b6093cb983c` |
| `/Users/rexxa/work/new-world-pass4/docs/revitalize/probe/p17-5_b_identity/i1_v19/summary.json` | 3950079 | `1168bd5b778d94818373b624ffd2ab6334b93eaadbbacf0ee0d9389a810da1a8` |
| `/Users/rexxa/work/new-world-pass4/docs/revitalize/probe/p17-5_b_identity/i2_sha256.txt` | 2092 | `1c8a7f8261a6f3093170391e110f79dbc0c3d111e6a399bc1a877cb8a363db84` |
| `/Users/rexxa/work/new-world-pass4/docs/revitalize/probe/p17-5_b_identity/i3_v16/summary.json` | 2935160 | `ba9f25faeab1cd33bf14366d13cb28ffcb32ac449a15dc857d21010a7503e368` |
| `/Users/rexxa/work/new-world-pass4/docs/revitalize/probe/p17-5_b_identity/i4_v20/summary.json` | 3961422 | `c4321ed4d14f2a09d1f97c76b40f175d5bc42c7b7a4dd6507e28d62bbcc9b540` |
| `/Users/rexxa/work/new-world-pass4/tools/revitalize/compare_identity_pass17.py` | 3509 | `f8d5dae0f8e01591c9c91b51dcb82ab38405d1783836e2aed31b560ac640867d` |
| `/Users/rexxa/work/p17-data/b-final-identity/i1_v19.log` | 1127 | `8ab4cff5b862bcafeaf75aff94027ca8f2e6edc86147c1687b0fff27f33d731f` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2.log` | 150 | `99638141d0dbbf46f3413d6ae4180f020ca5746738af0e550e928ab582be6cdb` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12000_none/building_keys.npy` | 512 | `55088d5ef75c36b9c0abcd64da206864e1b294b7e09bd11bc6e8e23ba8ba359c` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12000_none/building_present.npy` | 34700 | `5a6bfd7bd98def7f912d8d0fa87370d73be6d2c2ea98129be98913472356ab7d` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12000_none/elevation.npy` | 4224 | `0e19088555273a252dd22867323c857517897e50a62f268dfd8320b5a7d1633b` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12000_none/global.npy` | 449564 | `7a46a6bd804c9d352c5b507e30832777fcd3c4e8f5a0d9bf5ee461b3b8e23135` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12000_none/interventions.json` | 3 | `37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12000_none/latent.npy` | 80796 | `e303dc138cb77922f72234308388f73c433e3dc510566fae22fd1fdaf68d81c9` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12000_none/meta.json` | 17580 | `696eaa2890e94dd1bbe917cec9d60e23951ef8ce01eb16ee845fc31925c89b00` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12000_none/row_digest.npy` | 23176 | `380dc00fdf29b6d13e8f1abde6f1fb47995d4c8a0e8587bbb3de8f7e21356bc0` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12000_none/settlement_ids.npy` | 144 | `16ce3579de63bc66efaea992be8b419b07b98cf1f558941f2c5f4b0abd50fe78` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12000_none/settlements.npy` | 322800 | `4919fa276119ee6fc1ff12d6c134d174e4eb5dd5700c65f72a2434acc61871d1` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12000_none/terrain.npy` | 4224 | `20665223d630f9888fae8624c427236e3e266c27e244e543d7b7a31c22bc4192` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12001_none/building_keys.npy` | 480 | `61818bd631afcda2f436d57ac63952ae673855f3100f106ad12801f2050cc31e` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12001_none/building_present.npy` | 31819 | `8a9980093343df528ace56ffbc7f022f5bed75fc1f43ea7dd8d2eeb1b16e3d1f` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12001_none/elevation.npy` | 4224 | `b568e26c0fa404ee7316e0d34b76542d9b0f836ab6ca55e23fa7dab598e0f5a2` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12001_none/global.npy` | 449564 | `4ec2ccd80878e8c541cc58993c409e4707ad49526499113df48d0bda74fb2e44` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12001_none/interventions.json` | 3 | `37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12001_none/latent.npy` | 80796 | `54b34a56fb704688d07174450483eb0aa6ec1d5e755a7c872ed8b306cf037fe9` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12001_none/meta.json` | 17581 | `f42815bbd078c8eaba078fb39b62b5cc81685dc87cd89b27432e7b744e252411` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12001_none/row_digest.npy` | 23176 | `99e138c98470b0e07b98197e338262cc64f4f485aa6899fa59ebe2a5a82642fe` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12001_none/settlement_ids.npy` | 144 | `16ce3579de63bc66efaea992be8b419b07b98cf1f558941f2c5f4b0abd50fe78` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12001_none/settlements.npy` | 322800 | `0e5c57418e7dde0c05c7ff51cbcc413f3fe23d7ee30c02f9c1d35f6829a1a9b6` |
| `/Users/rexxa/work/p17-data/b-final-identity/i2/game_v19_s12001_none/terrain.npy` | 4224 | `cfd3878fecdcd4f90fc332b0b6bbde9da36533c45fff9c5e27036535557c6509` |
| `/Users/rexxa/work/p17-data/b-final-identity/i3_v16.log` | 1127 | `8795a57f57b5deb79d102a91f35543da7d618e2c6296ee2fea17ff91f2720b70` |
| `/Users/rexxa/work/p17-data/b-final-identity/i4_v20.log` | 1127 | `9ff7da2399eea51c79e717fafc3b39785755de594120bde6ef378a53da593cf9` |
| `/Users/rexxa/work/p17-data/b-final-identity/probe/i1_v19/summary.json` | 3950079 | `1168bd5b778d94818373b624ffd2ab6334b93eaadbbacf0ee0d9389a810da1a8` |
| `/Users/rexxa/work/p17-data/b-final-identity/probe/i3_v16/summary.json` | 2935160 | `ba9f25faeab1cd33bf14366d13cb28ffcb32ac449a15dc857d21010a7503e368` |
| `/Users/rexxa/work/p17-data/b-final-identity/probe/i4_v20/summary.json` | 3961422 | `c4321ed4d14f2a09d1f97c76b40f175d5bc42c7b7a4dd6507e28d62bbcc9b540` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12000_none/building_keys.npy` | 512 | `55088d5ef75c36b9c0abcd64da206864e1b294b7e09bd11bc6e8e23ba8ba359c` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12000_none/building_present.npy` | 34700 | `5a6bfd7bd98def7f912d8d0fa87370d73be6d2c2ea98129be98913472356ab7d` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12000_none/elevation.npy` | 4224 | `0e19088555273a252dd22867323c857517897e50a62f268dfd8320b5a7d1633b` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12000_none/global.npy` | 449564 | `7a46a6bd804c9d352c5b507e30832777fcd3c4e8f5a0d9bf5ee461b3b8e23135` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12000_none/interventions.json` | 3 | `37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12000_none/latent.npy` | 80796 | `e303dc138cb77922f72234308388f73c433e3dc510566fae22fd1fdaf68d81c9` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12000_none/meta.json` | 17580 | `b81d07a0ab352126fceddebea1e147b6438febac0f8c43769048b4137635ef89` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12000_none/row_digest.npy` | 23176 | `380dc00fdf29b6d13e8f1abde6f1fb47995d4c8a0e8587bbb3de8f7e21356bc0` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12000_none/settlement_ids.npy` | 144 | `16ce3579de63bc66efaea992be8b419b07b98cf1f558941f2c5f4b0abd50fe78` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12000_none/settlements.npy` | 322800 | `4919fa276119ee6fc1ff12d6c134d174e4eb5dd5700c65f72a2434acc61871d1` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12000_none/terrain.npy` | 4224 | `20665223d630f9888fae8624c427236e3e266c27e244e543d7b7a31c22bc4192` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12001_none/building_keys.npy` | 480 | `61818bd631afcda2f436d57ac63952ae673855f3100f106ad12801f2050cc31e` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12001_none/building_present.npy` | 31819 | `8a9980093343df528ace56ffbc7f022f5bed75fc1f43ea7dd8d2eeb1b16e3d1f` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12001_none/elevation.npy` | 4224 | `b568e26c0fa404ee7316e0d34b76542d9b0f836ab6ca55e23fa7dab598e0f5a2` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12001_none/global.npy` | 449564 | `4ec2ccd80878e8c541cc58993c409e4707ad49526499113df48d0bda74fb2e44` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12001_none/interventions.json` | 3 | `37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12001_none/latent.npy` | 80796 | `54b34a56fb704688d07174450483eb0aa6ec1d5e755a7c872ed8b306cf037fe9` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12001_none/meta.json` | 17581 | `a8211a206e89b59263884de365a97a1bb3f3c700d4acdc917abbba5592da499a` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12001_none/row_digest.npy` | 23176 | `99e138c98470b0e07b98197e338262cc64f4f485aa6899fa59ebe2a5a82642fe` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12001_none/settlement_ids.npy` | 144 | `16ce3579de63bc66efaea992be8b419b07b98cf1f558941f2c5f4b0abd50fe78` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12001_none/settlements.npy` | 322800 | `0e5c57418e7dde0c05c7ff51cbcc413f3fe23d7ee30c02f9c1d35f6829a1a9b6` |
| `/Users/rexxa/work/p17-data/p17-0_base/i2/game_v19_s12001_none/terrain.npy` | 4224 | `cfd3878fecdcd4f90fc332b0b6bbde9da36533c45fff9c5e27036535557c6509` |

## 완료 시각 증거

공유 RUNLOG는 I2 종료까지의 첫 435바이트만 읽고 해시했다: `a15af0b585d7c04c42e0473767bb281f1ba3676f46636fe65a2b121f2110f43d`. 실행 중인 뒤쪽 내용은 이 영수증의 해시 대상이 아니다.

```text
2026-10-15T10:08:31.479818Z runner start producer=47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21 bin=/Users/rexxa/work/p17-data/b-final-bin
2026-10-15T10:08:31.559527Z identities start
2026-10-15T10:08:31Z i1_v19 start
2026-10-15T10:15:27Z i1_v19 end
2026-10-15T10:15:27Z i3_v16 start
2026-10-15T10:19:59Z i3_v16 end
2026-10-15T10:19:59Z i4_v20 start
2026-10-15T10:27:22Z i4_v20 end
2026-10-15T10:27:22Z I2 start
2026-10-15T10:27:36Z I2 end
```
