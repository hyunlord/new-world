# Pass 17 Stage A round-1 observer-fix counter audit

Status: PASS for the bounded data comparison below. This is an audit of existing tuning artifacts, not the Part 8 independent review or a new simulation run.

## Scope and provenance

- Original root: `/Users/rexxa/work/p17-data/a-tuning/r1/game_v21`.
- Corrected root: `/Users/rexxa/work/p17-data/a-tuning/r1-observerfix/game_v21`.
- All 16 seeds `17100..17115`; each hourly file contains 1,440 samples, covering the recorded 60-day tuning run.
- Original producer: `753b51942c20489538faac59c15c5ac45ab48075` ([record](old_r1_source_commit.txt)). Corrected producer: `50a73474e06d3a9abbb535729e9c7e1e134954b7` ([record](source_commit.txt)).
- Read all `_people.jsonl`, `_agents.jsonl`, `_hours.jsonl`, `_routine.jsonl`, and `_trace_meta.json` files in both roots: 80 files per root. No source data was written, no build or probe was run, and no absent file, person, field, or counter was replaced with zero.
- Original input SHA-256 values are embedded below. All 80 corrected input hashes were recomputed and matched their `./game_v21/...` entries in [raw_sha256.txt](raw_sha256.txt), whose SHA-256 is `e4db5da5cf311e1e7e6f4206881aa67a511c8038f71b93614847a3546074a79f`.

## Exhaustive comparison

JSONL person rows were indexed by the explicit `id`, rejecting duplicate IDs. Original and corrected ID sets and routine leaf-key sets matched for every seed and table. Every non-routine row object was compared by sorted canonical JSON, retaining numeric representations. Routine arrays were compared at every index and scalar counters exactly; hourly rows were compared in order and file hashes were compared separately.

- **1125 lifetime people** and **1075 adult-agent rows**: identical person sets and identical non-routine fields.
- **All 16 adult `_agents.jsonl` files are byte-identical.** Therefore every adult counter, including `adult_ticks`, `adult_days`, starts, activity minutes, completed activities, completed minutes, no-place counters, and other non-routine observations, is unchanged.
- Adult ticks total **37,549,620 → 37,549,620**; adult days total **26,076.125 → 26,076.125**.
- **All 16 hourly files are byte-identical:** 23,040 ordered rows, including all life-stage/state counts and food/wood samples.
- Every per-person and per-adult `activity_minutes[kind]` value is identical. Lifetime total **6,193,739 → 6,193,739**; adult total **3,180,956 → 3,180,956**.
- Exactly **26 changed scalar cells in 15 lifetime people rows**, across eight seeds. The other 1,110 lifetime rows have identical routine counters; eight entire people files are byte-identical.
- The only aggregate routine changes are **+11 completed activities, +682 completed minutes, +3 HearthCircle starts, and +1 SitByFire start**. All adult routine changes are exactly zero.
- The `pass17` metadata objects are identical for all 16 seeds, including explicitly present `observation_misses: 0` in both runs. The original already reported zero misses, so that counter alone did not reveal the lost completion/start observations.

| Seed | People | Adult agents | Hourly rows | IDs / non-routine / activity minutes | Adult file / hourly file |
|---|---:|---:|---:|---|---|
| 17100 | 66 | 63 | 1440 | identical | byte-identical |
| 17101 | 100 | 92 | 1440 | identical | byte-identical |
| 17102 | 58 | 54 | 1440 | identical | byte-identical |
| 17103 | 52 | 50 | 1440 | identical | byte-identical |
| 17104 | 63 | 60 | 1440 | identical | byte-identical |
| 17105 | 75 | 72 | 1440 | identical | byte-identical |
| 17106 | 84 | 83 | 1440 | identical | byte-identical |
| 17107 | 54 | 53 | 1440 | identical | byte-identical |
| 17108 | 53 | 53 | 1440 | identical | byte-identical |
| 17109 | 109 | 104 | 1440 | identical | byte-identical |
| 17110 | 43 | 43 | 1440 | identical | byte-identical |
| 17111 | 120 | 115 | 1440 | identical | byte-identical |
| 17112 | 60 | 55 | 1440 | identical | byte-identical |
| 17113 | 64 | 59 | 1440 | identical | byte-identical |
| 17114 | 51 | 51 | 1440 | identical | byte-identical |
| 17115 | 73 | 68 | 1440 | identical | byte-identical |

## All routine counters

Array order is TendFire, Craft, Repair, HearthCircle, Explore, Play, SitByFire. The table enumerates every array element; unchanged columns show the value in both runs.

| Kind | Lifetime starts, original → corrected | Adult starts, both | Lifetime activity minutes, both | Adult activity minutes, both |
|---|---:|---:|---:|---:|
| TendFire | 25285 → 25285 | 25285 | 793581 | 793246 |
| Craft | 20005 → 20005 | 20005 | 607304 | 606502 |
| Repair | 18854 → 18854 | 18854 | 618523 | 617841 |
| HearthCircle | 46159 → 46162 | 28572 | 1859881 | 883159 |
| Explore | 16545 → 16545 | 16545 | 275472 | 274860 |
| Play | 14884 → 14884 | 4 | 1042307 | 5348 |
| SitByFire | 18052 → 18053 | 0 | 996671 | 0 |

| Scalar counter | Lifetime original → corrected | Adult original → corrected |
|---|---:|---:|
| `completed_activities` | 65516 → 65527 | 32018 → 32018 |
| `completed_minutes` | 4916784 → 4917466 | 2306072 → 2306072 |
| `no_place_redraws` | 3170 → 3170 | 3157 → 3157 |
| `no_place_failures` | 0 → 0 | 0 → 0 |

## Every changed person counter

All entries below belong to `_people.jsonl`. No `_agents.jsonl` counter changed. Array indices 3 and 6 mean HearthCircle and SitByFire respectively.

| Seed | Person ID | Routine counter | Original | Corrected | Delta |
|---|---:|---|---:|---:|---:|
| 17100 | 2 | `activities_started[3]` | 94 | 95 | +1 |
| 17101 | 15 | `completed_activities` | 110 | 111 | +1 |
| 17101 | 15 | `completed_minutes` | 7546 | 7622 | +76 |
| 17101 | 24 | `completed_activities` | 59 | 60 | +1 |
| 17101 | 24 | `completed_minutes` | 4595 | 4613 | +18 |
| 17101 | 27 | `completed_activities` | 43 | 44 | +1 |
| 17101 | 27 | `completed_minutes` | 3848 | 3960 | +112 |
| 17105 | 12 | `completed_activities` | 78 | 79 | +1 |
| 17105 | 12 | `completed_minutes` | 4794 | 4858 | +64 |
| 17106 | 15 | `completed_activities` | 63 | 64 | +1 |
| 17106 | 15 | `completed_minutes` | 4783 | 4847 | +64 |
| 17110 | 6 | `completed_activities` | 41 | 42 | +1 |
| 17110 | 6 | `completed_minutes` | 2337 | 2397 | +60 |
| 17112 | 0 | `completed_activities` | 69 | 70 | +1 |
| 17112 | 0 | `completed_minutes` | 3365 | 3389 | +24 |
| 17112 | 1 | `activities_started[3]` | 94 | 95 | +1 |
| 17112 | 24 | `completed_activities` | 52 | 53 | +1 |
| 17112 | 24 | `completed_minutes` | 4845 | 4949 | +104 |
| 17113 | 18 | `completed_activities` | 29 | 30 | +1 |
| 17113 | 18 | `completed_minutes` | 2835 | 2859 | +24 |
| 17115 | 10 | `completed_activities` | 63 | 64 | +1 |
| 17115 | 10 | `completed_minutes` | 5168 | 5232 | +64 |
| 17115 | 19 | `activities_started[6]` | 40 | 41 | +1 |
| 17115 | 23 | `activities_started[3]` | 51 | 52 | +1 |
| 17115 | 25 | `completed_activities` | 53 | 54 | +1 |
| 17115 | 25 | `completed_minutes` | 5127 | 5199 | +72 |

## Raw evidence for the 11 recovered completions

Each row below joins exactly one changed lifetime `completed_activities` counter (+1) and `completed_minutes` counter (+the listed minutes) to the same seed/person/`chosen_tick` episode. Every corrected ending explicitly has `event: ended`, `completed: true`, `source: observed`, and `observation_boundary: pre_lethal`; the adjacent `end_observation` also explicitly records `completed: true` and `life_stage: elder`. All 11 have `adult_consuming_minutes: 0`.

The original ending for each identical episode is `completed: false`, `source: agent_disappeared`, with the **same** consuming minutes. The original raw consuming interval sums independently equal the listed minutes in every case. Thus +682 completed minutes reclassifies already recorded activity minutes; it does not add consumption at the death tick.

Line references use `game_v21_s<seed>_routine.jsonl` under the original (O) or corrected (C) root above. `C ended/observation` gives both independently present raw rows. Completion is established by the explicit outcome, not inferred from `until_tick`; seven of these endings occur before their stored `until_tick`.

| Seed | ID | Kind | Chosen tick | Arrival tick | End tick | Minutes | O interval / ended lines | C ended / observation lines |
|---|---:|---|---:|---:|---:|---:|---|---|
| 17101 | 15 | SitByFire | 70530 | 70544 | 70620 | 76 | 34067 / 34068 | 34070 / 34071 |
| 17101 | 24 | HearthCircle | 76901 | 76902 | 76920 | 18 | 40137 / 40138 | 40042 / 40043 |
| 17101 | 27 | SitByFire | 55069 | 55088 | 55200 | 112 | 25579 / 25580 | 25581 / 25582 |
| 17105 | 12 | SitByFire | 43485 | 43496 | 43560 | 64 | 25170 / 25171 | 25168 / 25169 |
| 17106 | 15 | HearthCircle | 48022 | 48056 | 48120 | 64 | 20751 / 20752 | 20685 / 20686 |
| 17110 | 6 | HearthCircle | 24696 | 24720 | 24780 | 60 | 5748 / 5749 | 5749 / 5750 |
| 17112 | 0 | SitByFire | 35130 | 35136 | 35160 | 24 | 12901 / 12902 | 12862 / 12863 |
| 17112 | 24 | SitByFire | 35050 | 35056 | 35160 | 104 | 12903 / 12904 | 12865 / 12866 |
| 17113 | 18 | SitByFire | 35106 | 35136 | 35160 | 24 | 15562 / 15563 | 15500 / 15501 |
| 17115 | 10 | SitByFire | 53803 | 53816 | 53880 | 64 | 27243 / 27244 | 27199 / 27200 |
| 17115 | 25 | SitByFire | 43725 | 43728 | 43800 | 72 | 22239 / 22240 | 22167 / 22168 |

The coordinator's seed 17101 / person 27 example is reproduced: `[55088,55200)` is exactly **112** consuming minutes in both runs. Original line 25580 labels the episode incomplete; corrected lines 25581–25582 contain the recovered true completion. Its lifetime totals change from 43 to 44 completed activities and 3,848 to 3,960 completed minutes; adult counters stay unchanged.

## Raw evidence for the four recovered starts

Every recovered start is an explicit `pre_lethal`, `life_stage: elder`, `redrawn: false` row in Seeking(Activity), with null arrival/end ticks. Each is followed by an explicit same-tick `agent_disappeared` ending with `completed: false`, `consuming_minutes: 0`, and `adult_consuming_minutes: 0`. The original raw trace contains no start or episode for those exact seed/person/chosen-tick keys; the zero consumption values below come from the corrected raw rows, not missing-data substitution.

| Seed | ID | Kind | Tick | C start line | C disappeared-ending line | Lifetime start delta |
|---|---:|---|---:|---:|---:|---:|
| 17100 | 2 | HearthCircle | 72240 | 33719 | 33720 | +1 |
| 17112 | 1 | HearthCircle | 60420 | 22276 | 22277 | +1 |
| 17115 | 19 | SitByFire | 43620 | 21979 | 21980 | +1 |
| 17115 | 23 | HearthCircle | 46560 | 23453 | 23454 | +1 |

The corrected raw traces contain 281 rows labeled `pre_lethal`: 146 `ended` (11 true, 135 false), 114 `consuming_interval`, 13 `end_observation` (11 true, two false), four `started`, and four `arrived`. These are boundary-labeled rows, not 281 new activities. Only the 11 true completions and four starts produce routine counter deltas. Raw routine files are not byte-identical because recovered transitions and boundary labels are present.

## Original input SHA-256 manifest

Every filename below is relative to the original root declared above. The corrected counterparts are hash-verified against the linked canonical manifest. These 80 original records plus 80 verified corrected records bind every input read for this audit.

```text
4fe2ca342cdf1cf70f144b46aa46f8f9bb94b2731f58b17c1323f37a4ba7ba20  game_v21_s17100_agents.jsonl
189d9efd161362122e8593ed62be3660cd04c4339e0b49bdb55cbad1d2bb89db  game_v21_s17100_hours.jsonl
0f0f794b233d76d4dbfca83cd6cf922466092b469515bbd2263362247bd752f4  game_v21_s17100_people.jsonl
e92858135e07e47346dfdc2fab0ada56e36b2c6e97967381b2473894d72a0405  game_v21_s17100_routine.jsonl
5f885d9f4ed8cef856b7d301e79821e2010c1fb429eb44ab07d15f4f60030d07  game_v21_s17100_trace_meta.json
5a6da90a9a624acd2101aed258e35402a849d24f031c39dcc4e165bab17ab434  game_v21_s17101_agents.jsonl
00c382e2b413235f7dde4eb4b3ebc6730817aad63591e5cae32e52a8e41385aa  game_v21_s17101_hours.jsonl
1f55c8a7517383418992231dfa6080053ed8a0ae32475b96c76e9c1540596daf  game_v21_s17101_people.jsonl
e9ed81dbca3e5d909bd5dd46a9d8ddaf85a53ee688ec9077d09470c492ceb66a  game_v21_s17101_routine.jsonl
e7e169392a40fddc352fb00a9db56a04b8562d9b05235fac4bb467b77996795e  game_v21_s17101_trace_meta.json
6efa199046eecb4f65df6b9f15d4c3e301c2aafb970656ececd1d35b86be6fda  game_v21_s17102_agents.jsonl
3f34cfdccab4b5f53224cd71849f221c3e9633b4ee51cd2ddd060262111ca754  game_v21_s17102_hours.jsonl
cbdf80964d43b461ad94a04141d2c25068622a035c9bb5a19076e81355f0c61d  game_v21_s17102_people.jsonl
556d240cadca9d7daf0379882bdd0268e475e3d20ccbc132a3eb33bee7e8d2a9  game_v21_s17102_routine.jsonl
af8a6b9ba5039c2c078143286570b1eb73271daa50630e8b56db4b9ae82ce399  game_v21_s17102_trace_meta.json
4b405afcadfa05bf1ecc308eb0ee7466b4e8e07e2870fa359314a4ba6e9059a1  game_v21_s17103_agents.jsonl
b85be3ad8447e2017015940952aa077f48d3b06be439e5cfb2e64ee7748be07b  game_v21_s17103_hours.jsonl
5833c4b4d12856589c910b311a0c8c7beb976702915ebc35effdad02a134ccce  game_v21_s17103_people.jsonl
4c74078f232d7857e20b342aa1a99d510b43859c07e12812189fe14cadcf62ac  game_v21_s17103_routine.jsonl
28cd708721e2a9263bbb4eed893fbfe4682d70f69b49f670ce14be0e916814da  game_v21_s17103_trace_meta.json
a3941f98a1dd4bdad107740a35e48783c59a9a2ef4017fbe73cb6ecdea5a3d70  game_v21_s17104_agents.jsonl
a37116d51a31c305ac1f221a9c9a0f1de96249baa39412f4191821e1a1391c24  game_v21_s17104_hours.jsonl
e3b6cb98bfb61ccc0573d1468f9a6662ddf0264852f95de9def3fcb91953aa50  game_v21_s17104_people.jsonl
ca093067a1748f35edd2e3c10cdc1885b301269d357c69c3b73325f7fcebb976  game_v21_s17104_routine.jsonl
a825eea701f5372749d01586283bd3970a18a85d0ac7eabba9985de5399079ba  game_v21_s17104_trace_meta.json
9f76f92e5719c7e8ed330f60323b2a553515e0a1732e05db08246ab2d3b4b66a  game_v21_s17105_agents.jsonl
661f764b22e273caa4582976cd883547a0680cada3e4189fcca091f700e62794  game_v21_s17105_hours.jsonl
4c8b9b6e28b83963d27ea2e27dc87b30b9f48cf3fdda010b1218705d1f7b7ba5  game_v21_s17105_people.jsonl
e3a990bcdf5ebb96f0d2f4b6b611b5bc5e3da18fd53c59309554668d3d3ea930  game_v21_s17105_routine.jsonl
e123cf273d41dbd4941ecbf2a7c2dc27d8fe947272030d52a7b65458d035e4bc  game_v21_s17105_trace_meta.json
2819a981b365fda57b380c9e7b09561625f163099ae68092fba41ddf3d69f1a3  game_v21_s17106_agents.jsonl
4fd82b10394843697ec43963806f0e93a8bef105b9753dc138c9ff4feeb764cf  game_v21_s17106_hours.jsonl
a7954418a7173cdf2ab477209dd30f0e4b3cefc64cb3c7cf510a7730d9cf39d0  game_v21_s17106_people.jsonl
776f2e32211ac8c5871eb3eaa5f9e8ca3c549e5dafc251875ac71d48ca954711  game_v21_s17106_routine.jsonl
3f7bef1037c0955f3a00691ba0c9c21eb1dcd23ec741d220f8d9b3032605a63d  game_v21_s17106_trace_meta.json
3e93f51442898a54e434216bb06bc497d89c96ec570edbf992d79c68e09f2288  game_v21_s17107_agents.jsonl
ca680d8dd81617afedab1e32c6f8e90317cce6ca95d0165e30f76db869bb3737  game_v21_s17107_hours.jsonl
2d6b9f4987b63674346649fa0579598ffa9b5e7ef2f306c60a95eebdfd8f1154  game_v21_s17107_people.jsonl
271158c2f1c1b4199401eee173d9f85f367f0ae6fdbbd47c26d2273931bc9822  game_v21_s17107_routine.jsonl
e389d822e29b918091e7442191b180d070fdd3f6cbb148ed37856080ce4c7157  game_v21_s17107_trace_meta.json
43cff6535606ba8bcd84eb4f124002a857a7730978cd25815fe5d0d179d8fa55  game_v21_s17108_agents.jsonl
66e76b9021ffcba35ccd2aa8eb12d8bef3d6b63fc8f14e0beaf367c622bc85b7  game_v21_s17108_hours.jsonl
f9711388a8105930f622187d2fb1b1bf8f849cd7ec7fa9dddefd04f08313c196  game_v21_s17108_people.jsonl
10b3c6cdc18bb1db9828c21ce227e1d4a5f3f631117cdf57de6fbff413dc601e  game_v21_s17108_routine.jsonl
6923f01b1adc0f03412e41f941ef19c0232f9cbebda99a076efd53f323b4e214  game_v21_s17108_trace_meta.json
6108630bf168bb2f3aa49d83bb84c7bbbbd6df95c99383a0d5c3dcf5a2e3cabf  game_v21_s17109_agents.jsonl
b08b97c3112c01968ec5335b9fcdb8d3fe02a98f31ae32066fe0af2154eacde2  game_v21_s17109_hours.jsonl
7af413fe7efbf05f2f94a44fa6334c061ad2c95e7aa1a97f080adb97d7e45d87  game_v21_s17109_people.jsonl
09e83b3a5bcd07304e7cffe635155facf1d1c41cd14790ca03482bfc4c6a09b8  game_v21_s17109_routine.jsonl
cc28de0485c9289cfb941f2c327694f4b304e6ec77a391bc046785b8bacc7097  game_v21_s17109_trace_meta.json
0789f1ffc281536456a3a73a8b40e4f0533c291073bd6f50439092015b78fba8  game_v21_s17110_agents.jsonl
74f46cb1c805fc9a453bd50518de66fa6719d3fb8e0adc0b56fae48772e0f451  game_v21_s17110_hours.jsonl
1e0a2a8541b093d82b38e59d6b376d85c2b6174a39779be3f35ff78f042a3f49  game_v21_s17110_people.jsonl
9cb75f21eed5a4896e7c367fdbf03c00f2ccacbcdf3e961a6efa470b1b991590  game_v21_s17110_routine.jsonl
a531668ce832194d0cbb5680ce2ea52d77e04221fe36681bb51da9622647bfa7  game_v21_s17110_trace_meta.json
4778d244f354f41c51c8653b45d8b7e3f7ed469076b39a767199f3ae7a1e716e  game_v21_s17111_agents.jsonl
baa9863e851bdb6b9f14f1f2903df3c91fbfbb21bb9d24d75cf9ddc2ddc4c1c7  game_v21_s17111_hours.jsonl
4b001e243b13478817b3cc350db95d67ba0b95e7c702f0749361c1ef6a9e9079  game_v21_s17111_people.jsonl
d2fd03fa4361c4b86979f41bd12c3ea312a21219bd08cff894af3f675cb3ee35  game_v21_s17111_routine.jsonl
75d36277ff02426f0e700a23e29b979731c68b21dfe96048ce3fcdcd5562c8aa  game_v21_s17111_trace_meta.json
a8aec82efdb2dc6b4c255b96ab40e065f089455bbff39140c9849bc9d4953513  game_v21_s17112_agents.jsonl
b5652cd1398bfb00e4155c6eb291f3b06068c382acf564cc085c48b8f1e29d67  game_v21_s17112_hours.jsonl
9971f13d05b7202509f15386a4a2bbd326e843f4b7610f46d2dae269a8e9c9ef  game_v21_s17112_people.jsonl
6a4f2e8333ee4d414900b35ad92e15f0b22201386122f6bd2ef032ad39acc126  game_v21_s17112_routine.jsonl
f9f96a91e8956ed338c20fb26e41f067f9bfcc92a6d444fc0cdbb9f1027e349a  game_v21_s17112_trace_meta.json
cd81bda2556b274d0c655dba3bc793460f6a36861cf1140ba27341fed83fd4fe  game_v21_s17113_agents.jsonl
558d6b8cd1aaa8b88dcc668a0c8f87d9c64000af7091eeee57f8073587b7a7d6  game_v21_s17113_hours.jsonl
b1e232b11324db673754f0b10dee15689caea422835da9931526e84b245e57d4  game_v21_s17113_people.jsonl
1a956c7dbbd5330bd5ed4b9ea3840471421c155c6ac9e6a05a712fcee8c3fc23  game_v21_s17113_routine.jsonl
3f1ba4b703251f3135f32c09ae6eeb4a798026b23100f754a00bc276e3d5ec8d  game_v21_s17113_trace_meta.json
249a5a5138eda8d29481d48a3f219e187cae935a74324fd4556eda4502cd95c0  game_v21_s17114_agents.jsonl
2bd5d2c3cd61d1bc4060935763410db20bab6b582d922f6b01cd891a7c746eb8  game_v21_s17114_hours.jsonl
0f9f323a155c8ebcff80f41dcbbd472db7ebb76e3e6e1a603ea453674fff6bac  game_v21_s17114_people.jsonl
b6b990287b4de29180e6b6056a36e42ec4ffd3eb3dab2f4ede83d6d76d75f85a  game_v21_s17114_routine.jsonl
3c7c6641129f323f91ceaae97a524d9f96186df1c045b3271123a3b3ac985cb1  game_v21_s17114_trace_meta.json
d127958b649fb8a83d53ddfa3417301ea7d278f0daf3bb67aa9fda29e507817e  game_v21_s17115_agents.jsonl
7505cf3eab17e9a3539ab03c19890667fbdf760ceb6b428329d3c55bfa1636e8  game_v21_s17115_hours.jsonl
e16ae7f3b186ba3f2cf3ecf2d76f2d32c8be3effdc3b972900c244451ed74efa  game_v21_s17115_people.jsonl
3d3ed8df1958f22778f2ffe555994b2ff70bf28746d5a87d6971c9b8e6bbc40d  game_v21_s17115_routine.jsonl
38ab4e2be0538ae29a3b3302a6ec931391519e5155354ca73e3f0b5ec20ee966  game_v21_s17115_trace_meta.json
```

Audit procedure: read-only Python standard-library JSON parsing and SHA-256 hashing; duplicate-ID, key-set, canonical non-routine, per-leaf routine, ordered-hour, and raw episode comparisons. No simulation inference was used to fill an absent observation. No builds, tests, probes, source edits, or tuning changes were performed.
