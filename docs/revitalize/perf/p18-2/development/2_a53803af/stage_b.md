# Pass 18 stage b performance: PASS

| Check | Value | Limit | Pass |
|---|---:|---:|---|
| mean_relative | 0.023021516409869967 | <= 0.15 | True |
| mean_absolute | 0.3698949275362319 | <= 4.0 | True |
| p999_relative | 0.06896551724137931 | <= 0.35 | True |
| slope | 0.997941192378278 | < 1.7 | True |

AgentDecisionSystem: inherited log-log OLS of observed mean alive at 06:00 versus mean microseconds per call; historical slope 1.06.

## repeats

```json
{
  "old": [
    {
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/b/a/r1/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "62621a23a7dd5a6f0bbc1d159eaad26705ebba404d9f1f5aefa50d2820f2379e",
      "mean_ms": 0.36157101449275364,
      "p999_ms": 0.838,
      "max_ms": 0.846,
      "alive": 500.0,
      "decision_us": 235.22373550724637,
      "decision_share": 0.6505602663898286,
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
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/b/a/r2/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "2e9cee502ef5410aa30fa278c29c0cdedbf8c4620626ef0b0ff1169a66217f51",
      "mean_ms": 0.36031231884057974,
      "p999_ms": 0.772,
      "max_ms": 0.837,
      "alive": 500.0,
      "decision_us": 235.60077608695653,
      "decision_share": 0.6538793257057585,
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
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/b/a/r3/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "9e51bf946f2ed1adb1a31b05fa2b01f03fe652b753a537e7c1fba844b15513ba",
      "mean_ms": 0.36342681159420287,
      "p999_ms": 0.812,
      "max_ms": 0.831,
      "alive": 500.0,
      "decision_us": 237.9511652173913,
      "decision_share": 0.6547430118697024,
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
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/b/a/r1/stress_stress_v22_s15100_n500_side64_fed.json",
      "sha256": "5b696d722c8cc646312101450e83089180fe3ca59c271935edf135764bd1563f",
      "mean_ms": 0.3780449275362319,
      "p999_ms": 1.448,
      "max_ms": 2.082,
      "alive": 500.0,
      "decision_us": 249.23417971014493,
      "decision_share": 0.659271323475854,
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
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/b/a/r2/stress_stress_v22_s15100_n500_side64_fed.json",
      "sha256": "9504e49598cbda0702c977fb391ae4a306524f42b3f52f5554e77f2ab5f0863e",
      "mean_ms": 0.3632717391304348,
      "p999_ms": 0.868,
      "max_ms": 0.9,
      "alive": 500.0,
      "decision_us": 236.60887826086957,
      "decision_share": 0.6513275126417521,
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
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/b/a/r3/stress_stress_v22_s15100_n500_side64_fed.json",
      "sha256": "4ae82b4de1b76c59720a567cce4bf83c5bba77d5967292b1f4ddf8b0aaa058c1",
      "mean_ms": 0.3698949275362319,
      "p999_ms": 0.859,
      "max_ms": 1.315,
      "alive": 500.0,
      "decision_us": 242.68082246376812,
      "decision_share": 0.6560804282453889,
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
    "mean_ms": 0.36157101449275364,
    "p999_ms": 0.812
  },
  "new": {
    "mean_ms": 0.3698949275362319,
    "p999_ms": 0.868
  }
}
```

## slopes

```json
{
  "old": {
    "b": 1.003154441170095,
    "a": -0.6401627569776016,
    "n": 3,
    "se": 0.05257190034656501,
    "ci": [
      0.3351651119968685,
      1.6711437703433214
    ],
    "r2": 0.9972610719843795
  },
  "new": {
    "b": 0.997941192378278,
    "a": -0.5819791555839036,
    "n": 3,
    "se": 0.05581157819965434,
    "ci": [
      0.28878785312444777,
      1.7070945316321082
    ],
    "r2": 0.9968819545441717
  },
  "reference": 1.06,
  "old_points": [
    {
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/b/b_game_v21/stress_stress_v21_s15100_n100_side64_fed.json",
      "sha256": "2a8bc3ec21e246cd2fff33e038726af63388b984bcec9e38dc801ef9c10cc131",
      "mean_ms": 0.08820921985815604,
      "p999_ms": 0.195,
      "max_ms": 0.211,
      "alive": 100.0,
      "decision_us": 54.62943546099291,
      "decision_share": 0.6193166150753768,
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
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/b/b_game_v21/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "0e760fb5f60ae3f888e562b6bd20b64bb0fe12661a7c8b057b3a7001d996f61c",
      "mean_ms": 0.38311028368794325,
      "p999_ms": 0.846,
      "max_ms": 0.887,
      "alive": 500.0,
      "decision_us": 250.67567482269504,
      "decision_share": 0.654317269715681,
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
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/b/b_game_v21/stress_stress_v21_s15100_n1000_side64_fed.json",
      "sha256": "c3fa1e759c750f2a6b52ef9043d94de5bd42c0ec148fe156888a61997fa77bb3",
      "mean_ms": 0.8295007092198582,
      "p999_ms": 1.758,
      "max_ms": 1.855,
      "alive": 1000.0,
      "decision_us": 565.7956294326241,
      "decision_share": 0.6820917970820694,
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
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/b/b_game_v22/stress_stress_v22_s15100_n100_side64_fed.json",
      "sha256": "e68b6636100986844225d3ceb691c19fc03267b0fc49cd4aa54f8c310d30c0b6",
      "mean_ms": 0.09111702127659575,
      "p999_ms": 0.213,
      "max_ms": 0.221,
      "alive": 100.0,
      "decision_us": 56.60204042553192,
      "decision_share": 0.6212016112084063,
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
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/b/b_game_v22/stress_stress_v22_s15100_n500_side64_fed.json",
      "sha256": "aff42637c1ee730ef6d0789b4a1906fd0ab56f6fbc453eabf88f6618bb8e0982",
      "mean_ms": 0.3904801418439716,
      "p999_ms": 0.878,
      "max_ms": 0.918,
      "alive": 500.0,
      "decision_us": 256.1180453900709,
      "decision_share": 0.6559054301214907,
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
      "file": "/Users/rexxa/work/p18-data/perf/p18-2b/b/b_game_v22/stress_stress_v22_s15100_n1000_side64_fed.json",
      "sha256": "c6a8f29d4333f4429118d299a2575cab4d6c131fe1c094d12c86bfa6402489fa",
      "mean_ms": 0.8491390070921986,
      "p999_ms": 2.234,
      "max_ms": 6.197,
      "alive": 1000.0,
      "decision_us": 580.224459574468,
      "decision_share": 0.6833091575446468,
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
    "file": "/Users/rexxa/work/p18-data/perf/p18-2b/b/quick_base/quick_stress_v19_s15100_n500_side64_fed.json",
    "sha256": "a9cdd89bb67534997c707273f291114bcf3325db932b56eb61d79d704ff3733c",
    "mean_ms": 0.31058478260869565,
    "p999_ms": 0.679,
    "max_ms": 0.843,
    "alive": 500.0,
    "decision_us": 190.07194565217392,
    "decision_share": 0.6119808705877412,
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
    "file": "/Users/rexxa/work/p18-data/perf/p18-2b/b/quick_final/quick_stress_v19_s15100_n500_side64_fed.json",
    "sha256": "4f7f8d515975f7543312db68824655103674d5a608d7baf48009d8ce6cf1bff6",
    "mean_ms": 0.3115760869565217,
    "p999_ms": 0.698,
    "max_ms": 0.809,
    "alive": 500.0,
    "decision_us": 191.88920000000002,
    "decision_share": 0.6158662619919762,
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
    "sha256": "ae7f34632b8de33d3ac1cdd4b0a5e7cb1ad362ced6be02c62b2b55b96e913b49"
  },
  "runlog": {
    "file": "RUNLOG.txt",
    "sha256": "8c29a1006e82444b250ce24098116a200c783dfd2c176da7fb8c4724a0b1fac7"
  },
  "indexes": {
    "quick_final/runs.json": "6608d4baba65c865ca5788399393d732ab8a3fd0ed822aa3d95a1fc567a928e3",
    "b_game_v21/runs.json": "eb06a6c35a3e4190126682a7bed92ed25f3e71cba6fa858f226f2df71ee68a8c",
    "b_game_v22/runs.json": "f41f41eee0831ffe52a8d0d00c1f677968e3e5b120f99ae38d5ab741ef886a63",
    "quick_base/runs.json": "f56d6875dbcd95e0417bd31afd3208e6301832648f07039c1d85d86df0bc75ae",
    "a/r1/runs.json": "11d2a4cce47978b2479279364ea843245a0bccc5efd2253da63622d1b5460461",
    "a/r3/runs.json": "c4da49ea3f02ba16b343172cb893b8d33bee3a17fe955bd335b7774d18b859b5",
    "a/r2/runs.json": "cd4621df9d367c7438006b61711c17699f549a2eff97e6b27b04a5aa18867b4e"
  },
  "manifest_sha256": "c8638511721cbda623b92319954e3eb3fb47d9a572ede9cb3b439e9cca9e7686",
  "quiet_log_sha256": "8907fbd468171a65fd5e0d3f44df87832bc811b4c5c894f92d5cd943d90e26e2",
  "input_directory": "/Users/rexxa/work/p18-data/perf/p18-2b/b",
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
        "game_v21",
        "--out",
        "/Users/rexxa/work/p18-data/perf/p18-2b/b/a/r1"
      ],
      "started": 1792084141.21453,
      "binary_sha256_before": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "finished": 1792084141.8160799,
      "exit_code": 0,
      "binary_sha256_after": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "outputs": {
        "a/r1/stress_stress_v21_s15100_n500_side64_fed.json": "62621a23a7dd5a6f0bbc1d159eaad26705ebba404d9f1f5aefa50d2820f2379e"
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
        "game_v22",
        "--out",
        "/Users/rexxa/work/p18-data/perf/p18-2b/b/a/r1"
      ],
      "started": 1792084141.882671,
      "binary_sha256_before": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "finished": 1792084142.510529,
      "exit_code": 0,
      "binary_sha256_after": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "outputs": {
        "a/r1/stress_stress_v22_s15100_n500_side64_fed.json": "5b696d722c8cc646312101450e83089180fe3ca59c271935edf135764bd1563f"
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
        "game_v21",
        "--out",
        "/Users/rexxa/work/p18-data/perf/p18-2b/b/a/r2"
      ],
      "started": 1792084142.584078,
      "binary_sha256_before": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "finished": 1792084143.1879501,
      "exit_code": 0,
      "binary_sha256_after": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "outputs": {
        "a/r2/stress_stress_v21_s15100_n500_side64_fed.json": "2e9cee502ef5410aa30fa278c29c0cdedbf8c4620626ef0b0ff1169a66217f51"
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
        "game_v22",
        "--out",
        "/Users/rexxa/work/p18-data/perf/p18-2b/b/a/r2"
      ],
      "started": 1792084143.263218,
      "binary_sha256_before": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "finished": 1792084143.8712692,
      "exit_code": 0,
      "binary_sha256_after": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "outputs": {
        "a/r2/stress_stress_v22_s15100_n500_side64_fed.json": "9504e49598cbda0702c977fb391ae4a306524f42b3f52f5554e77f2ab5f0863e"
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
        "game_v21",
        "--out",
        "/Users/rexxa/work/p18-data/perf/p18-2b/b/a/r3"
      ],
      "started": 1792084143.9457011,
      "binary_sha256_before": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "finished": 1792084144.550665,
      "exit_code": 0,
      "binary_sha256_after": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "outputs": {
        "a/r3/stress_stress_v21_s15100_n500_side64_fed.json": "9e51bf946f2ed1adb1a31b05fa2b01f03fe652b753a537e7c1fba844b15513ba"
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
        "game_v22",
        "--out",
        "/Users/rexxa/work/p18-data/perf/p18-2b/b/a/r3"
      ],
      "started": 1792084144.621448,
      "binary_sha256_before": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "finished": 1792084145.2387788,
      "exit_code": 0,
      "binary_sha256_after": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "outputs": {
        "a/r3/stress_stress_v22_s15100_n500_side64_fed.json": "4ae82b4de1b76c59720a567cce4bf83c5bba77d5967292b1f4ddf8b0aaa058c1"
      }
    },
    {
      "index": 6,
      "build": "baseline",
      "argv": [
        "/Users/rexxa/work/new-world-p18-base/rust/target/release/worldsim_perf",
        "quick",
        "--out",
        "/Users/rexxa/work/p18-data/perf/p18-2b/b/quick_base"
      ],
      "started": 1792084145.3143702,
      "binary_sha256_before": "216cd0b6c40c246cacf1e1fa7a0e4b2fdcf14672eed38431b8cad7032a85ff25",
      "finished": 1792084145.8480132,
      "exit_code": 0,
      "binary_sha256_after": "216cd0b6c40c246cacf1e1fa7a0e4b2fdcf14672eed38431b8cad7032a85ff25",
      "outputs": {
        "quick_base/quick_stress_v19_s15100_n500_side64_fed.json": "a9cdd89bb67534997c707273f291114bcf3325db932b56eb61d79d704ff3733c"
      }
    },
    {
      "index": 7,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p2b/plain/worldsim_perf",
        "quick",
        "--out",
        "/Users/rexxa/work/p18-data/perf/p18-2b/b/quick_final"
      ],
      "started": 1792084145.919765,
      "binary_sha256_before": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "finished": 1792084146.4524002,
      "exit_code": 0,
      "binary_sha256_after": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "outputs": {
        "quick_final/quick_stress_v19_s15100_n500_side64_fed.json": "4f7f8d515975f7543312db68824655103674d5a608d7baf48009d8ce6cf1bff6"
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
        "game_v22",
        "--out",
        "/Users/rexxa/work/p18-data/perf/p18-2b/b/b_game_v22"
      ],
      "started": 1792084146.526902,
      "binary_sha256_before": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "finished": 1792084150.462611,
      "exit_code": 0,
      "binary_sha256_after": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "outputs": {
        "b_game_v22/stress_stress_v22_s15100_n100_side64_fed.json": "e68b6636100986844225d3ceb691c19fc03267b0fc49cd4aa54f8c310d30c0b6",
        "b_game_v22/stress_stress_v22_s15100_n500_side64_fed.json": "aff42637c1ee730ef6d0789b4a1906fd0ab56f6fbc453eabf88f6618bb8e0982",
        "b_game_v22/stress_stress_v22_s15100_n1000_side64_fed.json": "c6a8f29d4333f4429118d299a2575cab4d6c131fe1c094d12c86bfa6402489fa"
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
        "game_v21",
        "--out",
        "/Users/rexxa/work/p18-data/perf/p18-2b/b/b_game_v21"
      ],
      "started": 1792084150.5353441,
      "binary_sha256_before": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "finished": 1792084154.383208,
      "exit_code": 0,
      "binary_sha256_after": "2f58df4ff4ce5ebed986d1045d15d2992ae61d417b7fd3c41d8c23a73a47740d",
      "outputs": {
        "b_game_v21/stress_stress_v21_s15100_n100_side64_fed.json": "2a8bc3ec21e246cd2fff33e038726af63388b984bcec9e38dc801ef9c10cc131",
        "b_game_v21/stress_stress_v21_s15100_n500_side64_fed.json": "0e760fb5f60ae3f888e562b6bd20b64bb0fe12661a7c8b057b3a7001d996f61c",
        "b_game_v21/stress_stress_v21_s15100_n1000_side64_fed.json": "c3fa1e759c750f2a6b52ef9043d94de5bd42c0ec148fe156888a61997fa77bb3"
      }
    }
  ]
}
```
