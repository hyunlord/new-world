# Pass 18 stage a performance: PASS

| Check | Value | Limit | Pass |
|---|---:|---:|---|
| mean_relative | 0.13852540281126433 | <= 0.15 | True |
| mean_absolute | 0.36032101449275367 | <= 4.0 | True |
| p999_relative | 0.342668863261944 | <= 0.35 | True |
| slope | 1.0025958646131128 | < 1.7 | True |

AgentDecisionSystem: inherited log-log OLS of observed mean alive at 06:00 versus mean microseconds per call; historical slope 1.06.

## repeats

```json
{
  "old": [
    {
      "file": "/Users/rexxa/work/p18-data/perf/official/a/a/r1/stress_stress_v20_s15100_n500_side64_fed.json",
      "sha256": "94701cd81d85bfbc21b5c122b802039626fc274a8488f85ddb4e40c0da2c921e",
      "mean_ms": 0.31735797101449276,
      "p999_ms": 1.272,
      "max_ms": 1.442,
      "alive": 500.0,
      "decision_us": 196.39706739130435,
      "decision_share": 0.6188502742297136,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/a/a/r2/stress_stress_v20_s15100_n500_side64_fed.json",
      "sha256": "d3d27612262e204e299d0593d881dc213f34db51589d552cdffbc661233706e2",
      "mean_ms": 0.31176666666666664,
      "p999_ms": 0.607,
      "max_ms": 0.797,
      "alive": 500.0,
      "decision_us": 192.46624420289857,
      "decision_share": 0.6173406742314719,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/a/a/r3/stress_stress_v20_s15100_n500_side64_fed.json",
      "sha256": "ec7e80241e9f8f0262f400d191ce89ebb7d29d6f431d783fe31eecf910c04f2b",
      "mean_ms": 0.31648043478260873,
      "p999_ms": 0.569,
      "max_ms": 0.8,
      "alive": 500.0,
      "decision_us": 195.69020289855072,
      "decision_share": 0.6183327036724114,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/a/a/r1/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "e9a958270e5719d362dfbe03404eacb711c3b84ea9f34559d657f73b37ec0f94",
      "mean_ms": 0.35827681159420294,
      "p999_ms": 0.815,
      "max_ms": 0.817,
      "alive": 500.0,
      "decision_us": 233.99868333333333,
      "decision_share": 0.653122601745068,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/a/a/r2/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "0ffc3f75f23473b48ef5d76d73d13c9f135569fb07b5271892a4372a2bb38d79",
      "mean_ms": 0.3630398550724637,
      "p999_ms": 0.751,
      "max_ms": 0.842,
      "alive": 500.0,
      "decision_us": 237.89914130434784,
      "decision_share": 0.6552975878002775,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/a/a/r3/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "629ec9c5b2d33bea285e19a02b87268b9d3751b05bc09ef38e689cd57106c988",
      "mean_ms": 0.36032101449275367,
      "p999_ms": 0.825,
      "max_ms": 0.826,
      "alive": 500.0,
      "decision_us": 235.07763333333332,
      "decision_share": 0.6524116659259155,
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
    "mean_ms": 0.31648043478260873,
    "p999_ms": 0.607
  },
  "new": {
    "mean_ms": 0.36032101449275367,
    "p999_ms": 0.815
  }
}
```

## slopes

```json
{
  "old": {
    "b": 1.0628983880997989,
    "a": -1.2534818908240215,
    "n": 3,
    "se": 0.057698484464157415,
    "ci": [
      0.32976963153122196,
      1.7960271446683758
    ],
    "r2": 0.9970618941789249
  },
  "new": {
    "b": 1.0025958646131128,
    "a": -0.6302217115245803,
    "n": 3,
    "se": 0.05589410174863989,
    "ci": [
      0.2923939642503164,
      1.7127977649759092
    ],
    "r2": 0.9969016358916942
  },
  "reference": 1.06,
  "old_points": [
    {
      "file": "/Users/rexxa/work/p18-data/perf/official/a/b_game_v20/stress_stress_v20_s15100_n100_side64_fed.json",
      "sha256": "75daf7af9369a1199c0bffa588087143ebf46854151248e459f633c3b0d990f7",
      "mean_ms": 0.06863546099290779,
      "p999_ms": 0.137,
      "max_ms": 0.18,
      "alive": 100.0,
      "decision_us": 39.03412872340425,
      "decision_share": 0.5687166394560634,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/a/b_game_v20/stress_stress_v20_s15100_n500_side64_fed.json",
      "sha256": "fe459426efa86591d6fb380a67605c09f0d8d809a5c0b6e37c79ad757311afd7",
      "mean_ms": 0.3194691489361702,
      "p999_ms": 0.594,
      "max_ms": 0.829,
      "alive": 500.0,
      "decision_us": 195.451209929078,
      "decision_share": 0.6117999518261122,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/a/b_game_v20/stress_stress_v20_s15100_n1000_side64_fed.json",
      "sha256": "af1772a0b2b6f43d64088490c92320b5abac251f07c5da214f04bc0d2579546a",
      "mean_ms": 0.7130148936170212,
      "p999_ms": 1.333,
      "max_ms": 1.7,
      "alive": 1000.0,
      "decision_us": 465.15655921985814,
      "decision_share": 0.6523798638485464,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/a/b_game_v21/stress_stress_v21_s15100_n100_side64_fed.json",
      "sha256": "0976cd294eb88b01569c1a15fe0a532a829ed72546f8ed803ab3f74d7c92a64b",
      "mean_ms": 0.08877482269503546,
      "p999_ms": 0.202,
      "max_ms": 0.211,
      "alive": 100.0,
      "decision_us": 55.1066865248227,
      "decision_share": 0.6207467934250734,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/a/b_game_v21/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "3a7ea361977e13424541e377ac1952379e9e91898908d97b868de2b4db5ed7c0",
      "mean_ms": 0.38387624113475177,
      "p999_ms": 0.841,
      "max_ms": 0.86,
      "alive": 500.0,
      "decision_us": 251.1908865248227,
      "decision_share": 0.6543538245094136,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/a/b_game_v21/stress_stress_v21_s15100_n1000_side64_fed.json",
      "sha256": "bc8b7218ea642e95641f5a41f3ac232829b5deeaf4e6f8bf67c7b35533a7dd05",
      "mean_ms": 0.8369960992907802,
      "p999_ms": 1.831,
      "max_ms": 5.493,
      "alive": 1000.0,
      "decision_us": 571.0075833333334,
      "decision_share": 0.6822105668319968,
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
    "file": "/Users/rexxa/work/p18-data/perf/official/a/quick_base/quick_stress_v19_s15100_n500_side64_fed.json",
    "sha256": "bd00d76fda2f444b87c341b1d0b509ac543f32dca06a979a579694e84ec77c2c",
    "mean_ms": 0.30366811594202897,
    "p999_ms": 0.607,
    "max_ms": 0.801,
    "alive": 500.0,
    "decision_us": 186.29565289855074,
    "decision_share": 0.6134844032625244,
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
    "file": "/Users/rexxa/work/p18-data/perf/official/a/quick_final/quick_stress_v19_s15100_n500_side64_fed.json",
    "sha256": "ba7cc54770b7d660f3083ae8112230db54b1fc67f4e9224b9e78040564193ada",
    "mean_ms": 0.3048942028985507,
    "p999_ms": 0.601,
    "max_ms": 0.808,
    "alive": 500.0,
    "decision_us": 186.49486666666667,
    "decision_share": 0.6116707529815522,
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
    "sha256": "e244fac94aa131bdc124a0475c4946bb163d7db461e2edb1d03c4292ceb680d2"
  },
  "runlog": {
    "file": "RUNLOG.txt",
    "sha256": "96e37a45f79208c7fffd2e30023fe8c09813a144005155c31e3c8902c7a459ac"
  },
  "indexes": {
    "quick_final/runs.json": "8fee1427cb708c7f9526dca61447d6cfe3e1cd4ffa1fd216d26c5e594f8d73b5",
    "b_game_v21/runs.json": "1504ef31d5d43410dd1e1b55dc223c9faab7e465e6965c822edc08647897421f",
    "b_game_v20/runs.json": "85c862114ed94ee5b656f9a4b75d415fca2827ac66173914a4f27716e42417e9",
    "quick_base/runs.json": "8b8bce8a4dc7c9d9fc0409a42a5a938fc10652cbacf63e5125ebb6e4b517d791",
    "a/r1/runs.json": "3eac2fab09b467cefcf314b60d131a226d5d40d401018daf855536cda9559e85",
    "a/r3/runs.json": "2fd58f90d3badf5205059b477c74321f8592e23ba3a6d2cda007174468e9d4d2",
    "a/r2/runs.json": "5e7c6191574597513ea915ae467de6bfd9a28b8c7e2462f9b52cdbb84db10f47"
  },
  "manifest_sha256": "2f1bf1e2f32a90e80122bd9736f4d5be6809d4edc9bcf34c12c967a193b467d5",
  "quiet_log_sha256": "020cce5e90f6bd775c7bb8155149be3faee6174f1f5a7433bd5fe7b53079b731",
  "input_directory": "/Users/rexxa/work/p18-data/perf/official/a",
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
        "game_v20",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/a/a/r1"
      ],
      "started": 1792089497.670633,
      "binary_sha256_before": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "finished": 1792089498.211818,
      "exit_code": 0,
      "binary_sha256_after": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "outputs": {
        "a/r1/stress_stress_v20_s15100_n500_side64_fed.json": "94701cd81d85bfbc21b5c122b802039626fc274a8488f85ddb4e40c0da2c921e"
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
        "game_v21",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/a/a/r1"
      ],
      "started": 1792089498.280501,
      "binary_sha256_before": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "finished": 1792089498.879889,
      "exit_code": 0,
      "binary_sha256_after": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "outputs": {
        "a/r1/stress_stress_v21_s15100_n500_side64_fed.json": "e9a958270e5719d362dfbe03404eacb711c3b84ea9f34559d657f73b37ec0f94"
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
        "game_v20",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/a/a/r2"
      ],
      "started": 1792089498.952173,
      "binary_sha256_before": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "finished": 1792089499.486261,
      "exit_code": 0,
      "binary_sha256_after": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "outputs": {
        "a/r2/stress_stress_v20_s15100_n500_side64_fed.json": "d3d27612262e204e299d0593d881dc213f34db51589d552cdffbc661233706e2"
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
        "game_v21",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/a/a/r2"
      ],
      "started": 1792089499.559957,
      "binary_sha256_before": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "finished": 1792089500.166304,
      "exit_code": 0,
      "binary_sha256_after": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "outputs": {
        "a/r2/stress_stress_v21_s15100_n500_side64_fed.json": "0ffc3f75f23473b48ef5d76d73d13c9f135569fb07b5271892a4372a2bb38d79"
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
        "game_v20",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/a/a/r3"
      ],
      "started": 1792089500.23927,
      "binary_sha256_before": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "finished": 1792089500.777478,
      "exit_code": 0,
      "binary_sha256_after": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "outputs": {
        "a/r3/stress_stress_v20_s15100_n500_side64_fed.json": "ec7e80241e9f8f0262f400d191ce89ebb7d29d6f431d783fe31eecf910c04f2b"
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
        "game_v21",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/a/a/r3"
      ],
      "started": 1792089500.844913,
      "binary_sha256_before": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "finished": 1792089501.445855,
      "exit_code": 0,
      "binary_sha256_after": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "outputs": {
        "a/r3/stress_stress_v21_s15100_n500_side64_fed.json": "629ec9c5b2d33bea285e19a02b87268b9d3751b05bc09ef38e689cd57106c988"
      }
    },
    {
      "index": 6,
      "build": "baseline",
      "argv": [
        "/Users/rexxa/work/new-world-p18-base/rust/target/release/worldsim_perf",
        "quick",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/a/quick_base"
      ],
      "started": 1792089501.51731,
      "binary_sha256_before": "216cd0b6c40c246cacf1e1fa7a0e4b2fdcf14672eed38431b8cad7032a85ff25",
      "finished": 1792089502.038829,
      "exit_code": 0,
      "binary_sha256_after": "216cd0b6c40c246cacf1e1fa7a0e4b2fdcf14672eed38431b8cad7032a85ff25",
      "outputs": {
        "quick_base/quick_stress_v19_s15100_n500_side64_fed.json": "bd00d76fda2f444b87c341b1d0b509ac543f32dca06a979a579694e84ec77c2c"
      }
    },
    {
      "index": 7,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p2f/plain/worldsim_perf",
        "quick",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/a/quick_final"
      ],
      "started": 1792089502.112385,
      "binary_sha256_before": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "finished": 1792089502.633421,
      "exit_code": 0,
      "binary_sha256_after": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "outputs": {
        "quick_final/quick_stress_v19_s15100_n500_side64_fed.json": "ba7cc54770b7d660f3083ae8112230db54b1fc67f4e9224b9e78040564193ada"
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
        "game_v21",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/a/b_game_v21"
      ],
      "started": 1792089502.7056131,
      "binary_sha256_before": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "finished": 1792089506.577136,
      "exit_code": 0,
      "binary_sha256_after": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "outputs": {
        "b_game_v21/stress_stress_v21_s15100_n100_side64_fed.json": "0976cd294eb88b01569c1a15fe0a532a829ed72546f8ed803ab3f74d7c92a64b",
        "b_game_v21/stress_stress_v21_s15100_n500_side64_fed.json": "3a7ea361977e13424541e377ac1952379e9e91898908d97b868de2b4db5ed7c0",
        "b_game_v21/stress_stress_v21_s15100_n1000_side64_fed.json": "bc8b7218ea642e95641f5a41f3ac232829b5deeaf4e6f8bf67c7b35533a7dd05"
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
        "game_v20",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/a/b_game_v20"
      ],
      "started": 1792089506.651148,
      "binary_sha256_before": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "finished": 1792089509.92555,
      "exit_code": 0,
      "binary_sha256_after": "d4a4834fc9f9529b1182deabd632ffa4e6af5e927035ecae3f28a062a29362a2",
      "outputs": {
        "b_game_v20/stress_stress_v20_s15100_n100_side64_fed.json": "75daf7af9369a1199c0bffa588087143ebf46854151248e459f633c3b0d990f7",
        "b_game_v20/stress_stress_v20_s15100_n500_side64_fed.json": "fe459426efa86591d6fb380a67605c09f0d8d809a5c0b6e37c79ad757311afd7",
        "b_game_v20/stress_stress_v20_s15100_n1000_side64_fed.json": "af1772a0b2b6f43d64088490c92320b5abac251f07c5da214f04bc0d2579546a"
      }
    }
  ]
}
```
