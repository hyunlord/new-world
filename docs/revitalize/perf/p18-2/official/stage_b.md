# Pass 18 stage b performance: PASS

| Check | Value | Limit | Pass |
|---|---:|---:|---|
| mean_relative | -0.0021123132405978704 | <= 0.15 | True |
| mean_absolute | 0.36492318840579707 | <= 4.0 | True |
| p999_relative | 0.023837902264600714 | <= 0.35 | True |
| slope | 0.9966264729477696 | < 1.7 | True |

AgentDecisionSystem: inherited log-log OLS of observed mean alive at 06:00 versus mean microseconds per call; historical slope 1.06.

## repeats

```json
{
  "old": [
    {
      "file": "/Users/rexxa/work/p18-data/perf/official/b/a/r1/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "c08356a7a44c7e02ed195c32b1eb2368f36019304b9e16d492f3c221d38c4289",
      "mean_ms": 0.36819927536231883,
      "p999_ms": 0.887,
      "max_ms": 0.928,
      "alive": 500.0,
      "decision_us": 243.3921231884058,
      "decision_share": 0.6610336833197209,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/b/a/r2/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "6ddad52895e5067965bab6e3003d736495ff6c5e1bc67999068e93fc2942da2f",
      "mean_ms": 0.3656956521739131,
      "p999_ms": 0.832,
      "max_ms": 0.839,
      "alive": 500.0,
      "decision_us": 239.64442753623186,
      "decision_share": 0.655311120358261,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/b/a/r3/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "26f7b64ef842be7f836c33bf813ad617df12a00ebaed5566d3582c5f4eacf61c",
      "mean_ms": 0.36021449275362316,
      "p999_ms": 0.839,
      "max_ms": 0.848,
      "alive": 500.0,
      "decision_us": 236.04248913043477,
      "decision_share": 0.6552831545616943,
      "fingerprint": "14458e1fa711de0c",
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
      "file": "/Users/rexxa/work/p18-data/perf/official/b/a/r1/stress_stress_v22_s15100_n500_side64_fed.json",
      "sha256": "b92ae51840f2e84d243c32ebd0db0d8595242f45633c5eb13a75b7fa34d2e20b",
      "mean_ms": 0.3717949275362319,
      "p999_ms": 0.812,
      "max_ms": 0.851,
      "alive": 500.0,
      "decision_us": 243.04426449275363,
      "decision_share": 0.6537051651116694,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/b/a/r2/stress_stress_v22_s15100_n500_side64_fed.json",
      "sha256": "e40eb1151d5c19d24f9ea0ccfc8ac012377e6855d28013b0a01225a4266b29d5",
      "mean_ms": 0.36492318840579707,
      "p999_ms": 0.875,
      "max_ms": 0.881,
      "alive": 500.0,
      "decision_us": 238.8142847826087,
      "decision_share": 0.6544234303824112,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/b/a/r3/stress_stress_v22_s15100_n500_side64_fed.json",
      "sha256": "faf579c408d1d16e47cbcd435152faf0edf52874411ced1428fe63f28840f5b7",
      "mean_ms": 0.36329347826086955,
      "p999_ms": 0.859,
      "max_ms": 0.864,
      "alive": 500.0,
      "decision_us": 237.35547971014492,
      "decision_share": 0.6533436296362785,
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
    "mean_ms": 0.3656956521739131,
    "p999_ms": 0.839
  },
  "new": {
    "mean_ms": 0.36492318840579707,
    "p999_ms": 0.859
  }
}
```

## slopes

```json
{
  "old": {
    "b": 1.009137807842543,
    "a": -0.6636449696273399,
    "n": 3,
    "se": 0.05356335290375397,
    "ci": [
      0.32855087949146966,
      1.6897247361936163
    ],
    "r2": 0.9971906054685874
  },
  "new": {
    "b": 0.9966264729477696,
    "a": -0.5759294504849155,
    "n": 3,
    "se": 0.051849393752231854,
    "ci": [
      0.33781746048537653,
      1.6554354854101627
    ],
    "r2": 0.9973007155162192
  },
  "reference": 1.06,
  "old_points": [
    {
      "file": "/Users/rexxa/work/p18-data/perf/official/b/b_game_v21/stress_stress_v21_s15100_n100_side64_fed.json",
      "sha256": "28f0bf96ba6286c473db830eab0647dc9d998e3df2f2aa985b303d3f8db3a188",
      "mean_ms": 0.0882418439716312,
      "p999_ms": 0.192,
      "max_ms": 0.211,
      "alive": 100.0,
      "decision_us": 54.8741195035461,
      "decision_share": 0.6218605259562292,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/b/b_game_v21/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "31bc77fdf646b41cc398a5b1f26cad03763ed85eb47c3bbd23fc355a3bdba6b1",
      "mean_ms": 0.3868329787234043,
      "p999_ms": 0.9,
      "max_ms": 0.927,
      "alive": 500.0,
      "decision_us": 253.79935212765957,
      "decision_share": 0.6560954367573009,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/b/b_game_v21/stress_stress_v21_s15100_n1000_side64_fed.json",
      "sha256": "06737adaf3b75c2d71ec1307752a463e212babcdb37f982e6eb575f6b8f2209e",
      "mean_ms": 0.8414631205673759,
      "p999_ms": 1.75,
      "max_ms": 1.836,
      "alive": 1000.0,
      "decision_us": 576.5163046099291,
      "decision_share": 0.6851355579567168,
      "fingerprint": "fbf486f2e08c29ff",
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
      "file": "/Users/rexxa/work/p18-data/perf/official/b/b_game_v22/stress_stress_v22_s15100_n100_side64_fed.json",
      "sha256": "f2f89e5439043715bdf74fde84c7a679fdefdb6487ca4816ef86f1933379bda4",
      "mean_ms": 0.09064007092198581,
      "p999_ms": 0.211,
      "max_ms": 0.229,
      "alive": 100.0,
      "decision_us": 56.51209078014185,
      "decision_share": 0.6234780070812387,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/b/b_game_v22/stress_stress_v22_s15100_n500_side64_fed.json",
      "sha256": "02c08fd29beb69ffc821565d1b5b9c24454dd75c1258848e38cb928d7a212388",
      "mean_ms": 0.3927117021276596,
      "p999_ms": 0.905,
      "max_ms": 0.946,
      "alive": 500.0,
      "decision_us": 256.9252053191489,
      "decision_share": 0.6542336373659416,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/b/b_game_v22/stress_stress_v22_s15100_n1000_side64_fed.json",
      "sha256": "360b7299a581effd3eb970de2a810564d0c064fa8ba58d0fa90ff8092b656314",
      "mean_ms": 0.8461418439716313,
      "p999_ms": 2.011,
      "max_ms": 5.562,
      "alive": 1000.0,
      "decision_us": 576.3420226950354,
      "decision_share": 0.6811411429433557,
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
    "file": "/Users/rexxa/work/p18-data/perf/official/b/quick_base/quick_stress_v19_s15100_n500_side64_fed.json",
    "sha256": "0eb87e3b2dae9a6f2df4cca10387efec4569cce3368657cec5c08b82f2c77ba6",
    "mean_ms": 0.3064963768115942,
    "p999_ms": 0.576,
    "max_ms": 0.809,
    "alive": 500.0,
    "decision_us": 187.39393260869565,
    "decision_share": 0.6114066814038986,
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
    "file": "/Users/rexxa/work/p18-data/perf/official/b/quick_final/quick_stress_v19_s15100_n500_side64_fed.json",
    "sha256": "ff3c92165bb9ab78e82e170c8a8ac8a173ccfadee0d162d0e6ca330eff724816",
    "mean_ms": 0.30432101449275367,
    "p999_ms": 0.57,
    "max_ms": 0.76,
    "alive": 500.0,
    "decision_us": 185.48163985507244,
    "decision_share": 0.6094933672728312,
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
  "stage": "b",
  "status": 0,
  "builds": {
    "final": {
      "path": "/Users/rexxa/work/p18-data/bin_p2f/plain/worldsim_perf",
      "commit": "d3fc22f792513fc22145cf5d4bfd698a79f0d196",
      "sha256": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2"
    },
    "baseline": {
      "path": "/Users/rexxa/work/new-world-p18-base/rust/target/release/worldsim_perf",
      "commit": "8cbb697c97aac800935813d2c79c34195ebef6a3",
      "sha256": "216cd0b6c40c246cacf1e1fa7a0e4b2fdcf14672eed38431b8cad7032a85ff25"
    }
  },
  "quiet": {
    "owner": "pass18-sim (Claude Code, new-world-pass4)",
    "pid": "41069",
    "purpose": "p18 official: final attribution, R5 (a), F8 (b), G6 (c)",
    "started": "2026-10-15T18:38:06Z",
    "log": "/Users/rexxa/work/p18-data/perf/quiet.log",
    "prefix_bytes": 3651,
    "prefix_sha256": "3a711d87ed3e7693e34dbf0da524d58dbb9beb71a45128976763dd9f2133bb66",
    "contended": false
  },
  "commands": {
    "file": "commands.jsonl",
    "sha256": "df26920d015fb77e1c28d6ba2fc067aff1781b43d467181a7763379feb2a5447"
  },
  "runlog": {
    "file": "RUNLOG.txt",
    "sha256": "9e5a483762ed21b7b1cc67ed297dd7f2d1d856b68ce8e73fa8d14bd264484bdd"
  },
  "indexes": {
    "quick_final/runs.json": "c461aecc6a3b8134ec146d2639eaec14af7cf20486eee9cd3349843f44999dda",
    "b_game_v21/runs.json": "565316c996369dbe26df1eea26327ea4d0cd67a1a9d474fd4c3631008976db15",
    "b_game_v22/runs.json": "ae3e530024253f45764df516d58c870d5e96317b7bbe8f61bd466b520bace8e8",
    "quick_base/runs.json": "30db686f9dd161b8e2ebeea64aef71fd4aecf00458e5362fded7e54258319bdd",
    "a/r1/runs.json": "18ae1807db8f803e6da2d20fec774487e00501a60b684ac41385aeac6d4a0b72",
    "a/r3/runs.json": "588872f5c8b950a0e6536fdd72829d619c423b1eb11ddf0865a9858906c5abe9",
    "a/r2/runs.json": "a31cb10fc761c6ba5a659031cf0f79e5316da0104a3757b90f89a7b8c876b3ff"
  },
  "manifest_sha256": "feee9e7ba4a1ab398a8051de0cebb4079f29e6055ecff61ca4805fdf47f9c2cc",
  "quiet_log_sha256": "020cce5e90f6bd775c7bb8155149be3faee6174f1f5a7433bd5fe7b53079b731",
  "input_directory": "/Users/rexxa/work/p18-data/perf/official/b",
  "command_records": [
    {
      "index": 0,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p2f/plain/worldsim_perf",
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
        "/Users/rexxa/work/p18-data/perf/official/b/a/r1"
      ],
      "started": 1792089510.079031,
      "binary_sha256_before": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "finished": 1792089510.69407,
      "exit_code": 0,
      "binary_sha256_after": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "outputs": {
        "a/r1/stress_stress_v21_s15100_n500_side64_fed.json": "c08356a7a44c7e02ed195c32b1eb2368f36019304b9e16d492f3c221d38c4289"
      }
    },
    {
      "index": 1,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p2f/plain/worldsim_perf",
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
        "game_v22",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/b/a/r1"
      ],
      "started": 1792089510.76488,
      "binary_sha256_before": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "finished": 1792089511.381645,
      "exit_code": 0,
      "binary_sha256_after": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "outputs": {
        "a/r1/stress_stress_v22_s15100_n500_side64_fed.json": "b92ae51840f2e84d243c32ebd0db0d8595242f45633c5eb13a75b7fa34d2e20b"
      }
    },
    {
      "index": 2,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p2f/plain/worldsim_perf",
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
        "/Users/rexxa/work/p18-data/perf/official/b/a/r2"
      ],
      "started": 1792089511.4543529,
      "binary_sha256_before": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "finished": 1792089512.0771651,
      "exit_code": 0,
      "binary_sha256_after": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "outputs": {
        "a/r2/stress_stress_v21_s15100_n500_side64_fed.json": "6ddad52895e5067965bab6e3003d736495ff6c5e1bc67999068e93fc2942da2f"
      }
    },
    {
      "index": 3,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p2f/plain/worldsim_perf",
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
        "game_v22",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/b/a/r2"
      ],
      "started": 1792089512.149874,
      "binary_sha256_before": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "finished": 1792089512.7593632,
      "exit_code": 0,
      "binary_sha256_after": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "outputs": {
        "a/r2/stress_stress_v22_s15100_n500_side64_fed.json": "e40eb1151d5c19d24f9ea0ccfc8ac012377e6855d28013b0a01225a4266b29d5"
      }
    },
    {
      "index": 4,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p2f/plain/worldsim_perf",
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
        "/Users/rexxa/work/p18-data/perf/official/b/a/r3"
      ],
      "started": 1792089512.833612,
      "binary_sha256_before": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "finished": 1792089513.4347892,
      "exit_code": 0,
      "binary_sha256_after": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "outputs": {
        "a/r3/stress_stress_v21_s15100_n500_side64_fed.json": "26f7b64ef842be7f836c33bf813ad617df12a00ebaed5566d3582c5f4eacf61c"
      }
    },
    {
      "index": 5,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p2f/plain/worldsim_perf",
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
        "game_v22",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/b/a/r3"
      ],
      "started": 1792089513.507591,
      "binary_sha256_before": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "finished": 1792089514.117216,
      "exit_code": 0,
      "binary_sha256_after": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "outputs": {
        "a/r3/stress_stress_v22_s15100_n500_side64_fed.json": "faf579c408d1d16e47cbcd435152faf0edf52874411ced1428fe63f28840f5b7"
      }
    },
    {
      "index": 6,
      "build": "baseline",
      "argv": [
        "/Users/rexxa/work/new-world-p18-base/rust/target/release/worldsim_perf",
        "quick",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/b/quick_base"
      ],
      "started": 1792089514.189552,
      "binary_sha256_before": "216cd0b6c40c246cacf1e1fa7a0e4b2fdcf14672eed38431b8cad7032a85ff25",
      "finished": 1792089514.714897,
      "exit_code": 0,
      "binary_sha256_after": "216cd0b6c40c246cacf1e1fa7a0e4b2fdcf14672eed38431b8cad7032a85ff25",
      "outputs": {
        "quick_base/quick_stress_v19_s15100_n500_side64_fed.json": "0eb87e3b2dae9a6f2df4cca10387efec4569cce3368657cec5c08b82f2c77ba6"
      }
    },
    {
      "index": 7,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p2f/plain/worldsim_perf",
        "quick",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/b/quick_final"
      ],
      "started": 1792089514.7873878,
      "binary_sha256_before": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "finished": 1792089515.310954,
      "exit_code": 0,
      "binary_sha256_after": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "outputs": {
        "quick_final/quick_stress_v19_s15100_n500_side64_fed.json": "ff3c92165bb9ab78e82e170c8a8ac8a173ccfadee0d162d0e6ca330eff724816"
      }
    },
    {
      "index": 8,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p2f/plain/worldsim_perf",
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
        "game_v22",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/b/b_game_v22"
      ],
      "started": 1792089515.386415,
      "binary_sha256_before": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "finished": 1792089519.3115861,
      "exit_code": 0,
      "binary_sha256_after": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "outputs": {
        "b_game_v22/stress_stress_v22_s15100_n100_side64_fed.json": "f2f89e5439043715bdf74fde84c7a679fdefdb6487ca4816ef86f1933379bda4",
        "b_game_v22/stress_stress_v22_s15100_n500_side64_fed.json": "02c08fd29beb69ffc821565d1b5b9c24454dd75c1258848e38cb928d7a212388",
        "b_game_v22/stress_stress_v22_s15100_n1000_side64_fed.json": "360b7299a581effd3eb970de2a810564d0c064fa8ba58d0fa90ff8092b656314"
      }
    },
    {
      "index": 9,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p2f/plain/worldsim_perf",
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
        "/Users/rexxa/work/p18-data/perf/official/b/b_game_v21"
      ],
      "started": 1792089519.387546,
      "binary_sha256_before": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "finished": 1792089523.2767181,
      "exit_code": 0,
      "binary_sha256_after": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "outputs": {
        "b_game_v21/stress_stress_v21_s15100_n100_side64_fed.json": "28f0bf96ba6286c473db830eab0647dc9d998e3df2f2aa985b303d3f8db3a188",
        "b_game_v21/stress_stress_v21_s15100_n500_side64_fed.json": "31bc77fdf646b41cc398a5b1f26cad03763ed85eb47c3bbd23fc355a3bdba6b1",
        "b_game_v21/stress_stress_v21_s15100_n1000_side64_fed.json": "06737adaf3b75c2d71ec1307752a463e212babcdb37f982e6eb575f6b8f2209e"
      }
    }
  ]
}
```
