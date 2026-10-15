# Pass 18 stage a performance: FAIL

| Check | Value | Limit | Pass |
|---|---:|---:|---|
| mean_relative | 0.15028972346045527 | <= 0.15 | False |
| mean_absolute | 0.36164275362318843 | <= 4.0 | True |
| p999_relative | 0.38023450586264657 | <= 0.35 | False |
| slope | 1.0135511021027919 | < 1.7 | True |

AgentDecisionSystem: inherited log-log OLS of observed mean alive at 06:00 versus mean microseconds per call; historical slope 1.06.

## repeats

```json
{
  "old": [
    {
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/a/a/r1/stress_stress_v20_s15100_n500_side64_fed.json",
      "sha256": "1430f5d6d9f2b35aec8401b2bdb5158e82d431a2ee57088fd26441f01d77f87d",
      "mean_ms": 0.31685942028985503,
      "p999_ms": 0.939,
      "max_ms": 1.103,
      "alive": 500.0,
      "decision_us": 197.30151739130434,
      "decision_share": 0.6226784017051405,
      "fingerprint": "0fa49639a76b0281",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    },
    {
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/a/a/r2/stress_stress_v20_s15100_n500_side64_fed.json",
      "sha256": "d3f04a602735b499ef15c9abb09fde40aceead77f2e8a4f460b58398cbd93dd6",
      "mean_ms": 0.3143927536231884,
      "p999_ms": 0.597,
      "max_ms": 0.809,
      "alive": 500.0,
      "decision_us": 194.35012101449277,
      "decision_share": 0.6181762104079177,
      "fingerprint": "0fa49639a76b0281",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    },
    {
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/a/a/r3/stress_stress_v20_s15100_n500_side64_fed.json",
      "sha256": "a2b9063e30a467c2486a53ebe55577a90ed3164e038a91117676310b9d097274",
      "mean_ms": 0.3122855072463768,
      "p999_ms": 0.533,
      "max_ms": 0.813,
      "alive": 500.0,
      "decision_us": 192.24411376811594,
      "decision_share": 0.6156037001628945,
      "fingerprint": "0fa49639a76b0281",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    }
  ],
  "new": [
    {
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/a/a/r1/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "2c074403b79960dfd059c74cecd742efdfe5df2fb7198a06a977b8a7fcdf7560",
      "mean_ms": 0.3593572463768116,
      "p999_ms": 0.83,
      "max_ms": 0.841,
      "alive": 500.0,
      "decision_us": 234.09292971014494,
      "decision_share": 0.6514212029126076,
      "fingerprint": "14458e1fa711de0c",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    },
    {
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/a/a/r2/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "8d46fdd05bfa5b83f24626a07caf9820c58e0c6214c532a8342099d466743157",
      "mean_ms": 0.3621072463768116,
      "p999_ms": 0.824,
      "max_ms": 0.847,
      "alive": 500.0,
      "decision_us": 235.70880869565218,
      "decision_share": 0.6509364588919929,
      "fingerprint": "14458e1fa711de0c",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    },
    {
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/a/a/r3/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "388b74eb64ac9a32922b790b185d77877dc335ff9c46768446255fb583bbc795",
      "mean_ms": 0.36164275362318843,
      "p999_ms": 0.799,
      "max_ms": 0.822,
      "alive": 500.0,
      "decision_us": 236.3862275362319,
      "decision_share": 0.6536456908591431,
      "fingerprint": "14458e1fa711de0c",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    }
  ]
}
```

## medians

```json
{
  "old": {
    "mean_ms": 0.3143927536231884,
    "p999_ms": 0.597
  },
  "new": {
    "mean_ms": 0.36164275362318843,
    "p999_ms": 0.824
  }
}
```

## slopes

```json
{
  "old": {
    "b": 1.0490806776282702,
    "a": -1.1644509972492356,
    "n": 3,
    "se": 0.04783747211920202,
    "ci": [
      0.44124796282064227,
      1.656913392435898
    ],
    "r2": 0.9979250072964574
  },
  "new": {
    "b": 1.0135511021027919,
    "a": -0.6994032862343857,
    "n": 3,
    "se": 0.04667226325638848,
    "ci": [
      0.4205237696664781,
      1.6065784345391056
    ],
    "r2": 0.9978840446405544
  },
  "reference": 1.06,
  "old_points": [
    {
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/a/b_game_v20/stress_stress_v20_s15100_n100_side64_fed.json",
      "sha256": "df0031d5b3d03a559b0af376bfbbe0db726099d4354cb06f3319435f0777cf73",
      "mean_ms": 0.06983687943262412,
      "p999_ms": 0.155,
      "max_ms": 0.182,
      "alive": 100.0,
      "decision_us": 39.880523758865245,
      "decision_share": 0.5710524880674317,
      "fingerprint": "e44f8265b2219b8e",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    },
    {
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/a/b_game_v20/stress_stress_v20_s15100_n500_side64_fed.json",
      "sha256": "6a51b45507ff55e44e81466c93a8152b043f0332c3ed03b9791480e1339b248f",
      "mean_ms": 0.32404007092198583,
      "p999_ms": 0.589,
      "max_ms": 0.833,
      "alive": 500.0,
      "decision_us": 198.65690319148936,
      "decision_share": 0.6130627691391815,
      "fingerprint": "10746baa35b729ed",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    },
    {
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/a/b_game_v20/stress_stress_v20_s15100_n1000_side64_fed.json",
      "sha256": "dcc1c4cb71a718055884bfe08c3a6c04886bdb0cd44b9da977b92fab72f711ac",
      "mean_ms": 0.7077861702127659,
      "p999_ms": 1.366,
      "max_ms": 1.701,
      "alive": 1000.0,
      "decision_us": 457.96493546099293,
      "decision_share": 0.6470385474236169,
      "fingerprint": "603177818e8fb210",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    }
  ],
  "new_points": [
    {
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/a/b_game_v21/stress_stress_v21_s15100_n100_side64_fed.json",
      "sha256": "38d9e4d59ebd62633758bdb78f4f41a85ef7f1f4a6ada63e33b6dc95986abe91",
      "mean_ms": 0.08726702127659575,
      "p999_ms": 0.208,
      "max_ms": 0.218,
      "alive": 100.0,
      "decision_us": 53.88487056737589,
      "decision_share": 0.6174711795947061,
      "fingerprint": "bea6a0b9ada4edc7",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    },
    {
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/a/b_game_v21/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "d6752920eb413907519a8ca6aeee055ea1550321690fab8f49da7e47d40b881d",
      "mean_ms": 0.3887514184397163,
      "p999_ms": 1.159,
      "max_ms": 1.841,
      "alive": 500.0,
      "decision_us": 254.00998617021276,
      "decision_share": 0.6533995096138848,
      "fingerprint": "7ce50f96939b36b6",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    },
    {
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/a/b_game_v21/stress_stress_v21_s15100_n1000_side64_fed.json",
      "sha256": "3bb83278ada2d4e03e3628d93b462b8c16477edf9dc89a32e5116a8f9b1656ff",
      "mean_ms": 0.8319294326241135,
      "p999_ms": 1.786,
      "max_ms": 1.792,
      "alive": 1000.0,
      "decision_us": 569.8242592198582,
      "decision_share": 0.6849430214561467,
      "fingerprint": "fbf486f2e08c29ff",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    }
  ]
}
```

## quick

```json
{
  "base": {
    "file": "/Users/rexxa/work/p18-data/perf/p18-2b/a/quick_base/quick_stress_v19_s15100_n500_side64_fed.json",
    "sha256": "5cf14b0be258416207f37cf1250c0d7fd2f62d8a6e7ec23d51fc9fedd7d08d16",
    "mean_ms": 0.3029927536231884,
    "p999_ms": 0.568,
    "max_ms": 0.75,
    "alive": 500.0,
    "decision_us": 185.21637173913044,
    "decision_share": 0.6112897735154139,
    "fingerprint": "54ac4928a770f238",
    "machine": {
      "cpu": "Apple M4 Max",
      "memsize": "38654705664",
      "ncpu": "14",
      "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
      "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
    }
  },
  "final": {
    "file": "/Users/rexxa/work/p18-data/perf/p18-2b/a/quick_final/quick_stress_v19_s15100_n500_side64_fed.json",
    "sha256": "b380afe666501c14069d1bf2a6bedaa81501518e829c1fd30885a0018fc6cce6",
    "mean_ms": 0.3026152173913043,
    "p999_ms": 0.544,
    "max_ms": 0.813,
    "alive": 500.0,
    "decision_us": 184.17779710144927,
    "decision_share": 0.6086204080850748,
    "fingerprint": "54ac4928a770f238",
    "machine": {
      "cpu": "Apple M4 Max",
      "memsize": "38654705664",
      "ncpu": "14",
      "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
      "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
    }
  }
}
```

## provenance

```json
{
  "schema_version": 1,
  "stage": "a",
  "status": 0,
  "builds": {
    "final": {
      "path": "/Users/rexxa/work/p18-data/bin_p2b/plain/worldsim_perf",
      "commit": "a53803afd49702d4fae42146ba4ab4ad9b47eb38",
      "sha256": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d"
    },
    "baseline": {
      "path": "/Users/rexxa/work/new-world-p18-base/rust/target/release/worldsim_perf",
      "commit": "8cbb697c97aac800935813d2c79c34195ebef6a3",
      "sha256": "216cd0b6c40c246cacf1e1fa7a0e4b2fdcf14672eed38431b8cad7032a85ff25"
    }
  },
  "quiet": {
    "owner": "pass18-sim (Claude Code, new-world-pass4)",
    "pid": "8609",
    "purpose": "p18 part 2 candidate a53803af: attribution and R5/F8",
    "started": "2026-10-15T17:08:37Z",
    "log": "/Users/rexxa/work/p18-data/perf/quiet.log",
    "prefix_bytes": 1815,
    "prefix_sha256": "86eb55746ea28b3a6ecea38697ee57fcd4cc9d8209e0b74fef9eea188eb2afdb",
    "contended": false
  },
  "commands": {
    "file": "commands.jsonl",
    "sha256": "7c2507d648afe6f8ae940333477dd25195de38d240fdfaec50cda9d179d9c012"
  },
  "runlog": {
    "file": "RUNLOG.txt",
    "sha256": "6ca24f2699fe9365050fd3f134e1d8c9e0a65c9e350e2675993c70b404bebf12"
  },
  "indexes": {
    "quick_final/runs.json": "cf845b51c275db039129099757f7daa79cdbebe1ad9dccf51dd74b239d7e6596",
    "b_game_v21/runs.json": "66ff7760e61e89bc83c2bf109f5601be97f8f35009e7af887fe50205ad286127",
    "b_game_v20/runs.json": "677fa3c9b609c11ce93608c6ddebc87b1ed42f57f8cda6fc5f6fcd7b7d8313fb",
    "quick_base/runs.json": "bdad9aff157e3d3e6624642efa7fdeace54981f0cff7f6df925743dc1a190f13",
    "a/r1/runs.json": "13cb995e196491a80efe17242b61ed371edd8a186e6d38850f198beabfff5d61",
    "a/r3/runs.json": "2c97e87db0aa65387ac39f93d5c9c278b48782db8424befe103a71b50117c38e",
    "a/r2/runs.json": "60dc60f5c896a75db3c3182c7355728b6a64b7f6ab3a610a851b6987a6314974"
  },
  "manifest_sha256": "11c9d91a46c9cd2ad1587636efd4cc191d9c3ad28c8871a45b9f51e2930179ac",
  "quiet_log_sha256": "8907fbd468171a65fd5e0d3f44df87832bc811b4c5c894f92d5cd943d90e26e2",
  "input_directory": "/Users/rexxa/work/p18-data/perf/p18-2b/a",
  "command_records": [
    {
      "index": 0,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p2b/plain/worldsim_perf",
        "stress",
        "--seeds",
        "15100..15101",
        "--agents",
        "500",
        "--side",
        "64",
        "--days",
        "1",
        "--feed",
        "fed",
        "--rules",
        "game_v20",
        "--out",
        "/Users/rexxa/work/p18-data/perf/p18-2b/a/a/r1"
      ],
      "started": 1792084128.794848,
      "binary_sha256_before": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "finished": 1792084129.340659,
      "exit_code": 0,
      "binary_sha256_after": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "outputs": {
        "a/r1/stress_stress_v20_s15100_n500_side64_fed.json": "1430f5d6d9f2b35aec8401b2bdb5158e82d431a2ee57088fd26441f01d77f87d"
      }
    },
    {
      "index": 1,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p2b/plain/worldsim_perf",
        "stress",
        "--seeds",
        "15100..15101",
        "--agents",
        "500",
        "--side",
        "64",
        "--days",
        "1",
        "--feed",
        "fed",
        "--rules",
        "game_v21",
        "--out",
        "/Users/rexxa/work/p18-data/perf/p18-2b/a/a/r1"
      ],
      "started": 1792084129.416354,
      "binary_sha256_before": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "finished": 1792084130.021079,
      "exit_code": 0,
      "binary_sha256_after": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "outputs": {
        "a/r1/stress_stress_v21_s15100_n500_side64_fed.json": "2c074403b79960dfd059c74cecd742efdfe5df2fb7198a06a977b8a7fcdf7560"
      }
    },
    {
      "index": 2,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p2b/plain/worldsim_perf",
        "stress",
        "--seeds",
        "15100..15101",
        "--agents",
        "500",
        "--side",
        "64",
        "--days",
        "1",
        "--feed",
        "fed",
        "--rules",
        "game_v20",
        "--out",
        "/Users/rexxa/work/p18-data/perf/p18-2b/a/a/r2"
      ],
      "started": 1792084130.0954628,
      "binary_sha256_before": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "finished": 1792084130.629571,
      "exit_code": 0,
      "binary_sha256_after": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "outputs": {
        "a/r2/stress_stress_v20_s15100_n500_side64_fed.json": "d3f04a602735b499ef15c9abb09fde40aceead77f2e8a4f460b58398cbd93dd6"
      }
    },
    {
      "index": 3,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p2b/plain/worldsim_perf",
        "stress",
        "--seeds",
        "15100..15101",
        "--agents",
        "500",
        "--side",
        "64",
        "--days",
        "1",
        "--feed",
        "fed",
        "--rules",
        "game_v21",
        "--out",
        "/Users/rexxa/work/p18-data/perf/p18-2b/a/a/r2"
      ],
      "started": 1792084130.699021,
      "binary_sha256_before": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "finished": 1792084131.301091,
      "exit_code": 0,
      "binary_sha256_after": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "outputs": {
        "a/r2/stress_stress_v21_s15100_n500_side64_fed.json": "8d46fdd05bfa5b83f24626a07caf9820c58e0c6214c532a8342099d466743157"
      }
    },
    {
      "index": 4,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p2b/plain/worldsim_perf",
        "stress",
        "--seeds",
        "15100..15101",
        "--agents",
        "500",
        "--side",
        "64",
        "--days",
        "1",
        "--feed",
        "fed",
        "--rules",
        "game_v20",
        "--out",
        "/Users/rexxa/work/p18-data/perf/p18-2b/a/a/r3"
      ],
      "started": 1792084131.3724759,
      "binary_sha256_before": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "finished": 1792084131.905854,
      "exit_code": 0,
      "binary_sha256_after": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "outputs": {
        "a/r3/stress_stress_v20_s15100_n500_side64_fed.json": "a2b9063e30a467c2486a53ebe55577a90ed3164e038a91117676310b9d097274"
      }
    },
    {
      "index": 5,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p2b/plain/worldsim_perf",
        "stress",
        "--seeds",
        "15100..15101",
        "--agents",
        "500",
        "--side",
        "64",
        "--days",
        "1",
        "--feed",
        "fed",
        "--rules",
        "game_v21",
        "--out",
        "/Users/rexxa/work/p18-data/perf/p18-2b/a/a/r3"
      ],
      "started": 1792084131.978571,
      "binary_sha256_before": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "finished": 1792084132.5819771,
      "exit_code": 0,
      "binary_sha256_after": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "outputs": {
        "a/r3/stress_stress_v21_s15100_n500_side64_fed.json": "388b74eb64ac9a32922b790b185d77877dc335ff9c46768446255fb583bbc795"
      }
    },
    {
      "index": 6,
      "build": "baseline",
      "argv": [
        "/Users/rexxa/work/new-world-p18-base/rust/target/release/worldsim_perf",
        "quick",
        "--out",
        "/Users/rexxa/work/p18-data/perf/p18-2b/a/quick_base"
      ],
      "started": 1792084132.652899,
      "binary_sha256_before": "216cd0b6c40c246cacf1e1fa7a0e4b2fdcf14672eed38431b8cad7032a85ff25",
      "finished": 1792084133.1764648,
      "exit_code": 0,
      "binary_sha256_after": "216cd0b6c40c246cacf1e1fa7a0e4b2fdcf14672eed38431b8cad7032a85ff25",
      "outputs": {
        "quick_base/quick_stress_v19_s15100_n500_side64_fed.json": "5cf14b0be258416207f37cf1250c0d7fd2f62d8a6e7ec23d51fc9fedd7d08d16"
      }
    },
    {
      "index": 7,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p2b/plain/worldsim_perf",
        "quick",
        "--out",
        "/Users/rexxa/work/p18-data/perf/p18-2b/a/quick_final"
      ],
      "started": 1792084133.245607,
      "binary_sha256_before": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "finished": 1792084133.767724,
      "exit_code": 0,
      "binary_sha256_after": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "outputs": {
        "quick_final/quick_stress_v19_s15100_n500_side64_fed.json": "b380afe666501c14069d1bf2a6bedaa81501518e829c1fd30885a0018fc6cce6"
      }
    },
    {
      "index": 8,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p2b/plain/worldsim_perf",
        "stress",
        "--seeds",
        "15100..15101",
        "--agents",
        "100,500,1000",
        "--side",
        "64",
        "--days",
        "2",
        "--feed",
        "fed",
        "--rules",
        "game_v21",
        "--out",
        "/Users/rexxa/work/p18-data/perf/p18-2b/a/b_game_v21"
      ],
      "started": 1792084133.843543,
      "binary_sha256_before": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "finished": 1792084137.7114618,
      "exit_code": 0,
      "binary_sha256_after": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "outputs": {
        "b_game_v21/stress_stress_v21_s15100_n100_side64_fed.json": "38d9e4d59ebd62633758bdb78f4f41a85ef7f1f4a6ada63e33b6dc95986abe91",
        "b_game_v21/stress_stress_v21_s15100_n500_side64_fed.json": "d6752920eb413907519a8ca6aeee055ea1550321690fab8f49da7e47d40b881d",
        "b_game_v21/stress_stress_v21_s15100_n1000_side64_fed.json": "3bb83278ada2d4e03e3628d93b462b8c16477edf9dc89a32e5116a8f9b1656ff"
      }
    },
    {
      "index": 9,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p2b/plain/worldsim_perf",
        "stress",
        "--seeds",
        "15100..15101",
        "--agents",
        "100,500,1000",
        "--side",
        "64",
        "--days",
        "2",
        "--feed",
        "fed",
        "--rules",
        "game_v20",
        "--out",
        "/Users/rexxa/work/p18-data/perf/p18-2b/a/b_game_v20"
      ],
      "started": 1792084137.784422,
      "binary_sha256_before": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "finished": 1792084141.0626838,
      "exit_code": 0,
      "binary_sha256_after": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "outputs": {
        "b_game_v20/stress_stress_v20_s15100_n100_side64_fed.json": "df0031d5b3d03a559b0af376bfbbe0db726099d4354cb06f3319435f0777cf73",
        "b_game_v20/stress_stress_v20_s15100_n500_side64_fed.json": "6a51b45507ff55e44e81466c93a8152b043f0332c3ed03b9791480e1339b248f",
        "b_game_v20/stress_stress_v20_s15100_n1000_side64_fed.json": "dcc1c4cb71a718055884bfe08c3a6c04886bdb0cd44b9da977b92fab72f711ac"
      }
    }
  ]
}
```
