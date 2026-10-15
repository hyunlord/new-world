# Pass 18 stage c performance: PASS

| Check | Value | Limit | Pass |
|---|---:|---:|---|
| mean_relative | -0.011054400258409453 | <= 0.15 | True |
| mean_absolute | 0.3682840579710145 | <= 4.0 | True |
| p999_relative | -0.016036655211912942 | <= 0.35 | True |
| slope | 0.9730574105873652 | < 1.7 | True |

AgentDecisionSystem: inherited log-log OLS of observed mean alive at 06:00 versus mean microseconds per call; historical slope 1.06.

## repeats

```json
{
  "old": [
    {
      "file": "/Users/rexxa/work/p18-data/perf/official/c/a/r1/stress_stress_v22_s15100_n500_side64_fed.json",
      "sha256": "864d06dc82871be31773174ea50770290a96b838d10bbcb573d3ca069622dc84",
      "mean_ms": 0.3668014492753623,
      "p999_ms": 0.868,
      "max_ms": 0.883,
      "alive": 500.0,
      "decision_us": 239.43052101449277,
      "decision_share": 0.6527523854867578,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/c/a/r2/stress_stress_v22_s15100_n500_side64_fed.json",
      "sha256": "ff243f63913faca7c6d2fd49d8a8c0067b7f5078501099db6fcef75ebb6d2839",
      "mean_ms": 0.37240072463768115,
      "p999_ms": 1.013,
      "max_ms": 1.346,
      "alive": 500.0,
      "decision_us": 245.2471920289855,
      "decision_share": 0.6585572363415597,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/c/a/r3/stress_stress_v22_s15100_n500_side64_fed.json",
      "sha256": "984efe143c602bb8af8415413fb2388684d144f908d13ebe92c8a18ff25a1137",
      "mean_ms": 0.3754311594202899,
      "p999_ms": 0.873,
      "max_ms": 0.891,
      "alive": 500.0,
      "decision_us": 245.80050942028987,
      "decision_share": 0.6547152607147337,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/c/a/r1/stress_stress_v23_s15100_n500_side64_fed.json",
      "sha256": "cfbaeea681054823b7f8b6556be935db27813f6743a571a54beea25b92ef1537",
      "mean_ms": 0.36445,
      "p999_ms": 0.842,
      "max_ms": 0.892,
      "alive": 500.0,
      "decision_us": 238.06890434782608,
      "decision_share": 0.6532278895536454,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/c/a/r2/stress_stress_v23_s15100_n500_side64_fed.json",
      "sha256": "c315b071f2a024215d6e3dff4853f5f842cc0481bcc7d72885d6508f17f0c290",
      "mean_ms": 0.38314927536231885,
      "p999_ms": 1.565,
      "max_ms": 2.112,
      "alive": 500.0,
      "decision_us": 253.2180304347826,
      "decision_share": 0.6608861003203806,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/c/a/r3/stress_stress_v23_s15100_n500_side64_fed.json",
      "sha256": "a85a7c496031947217ed8c2c11bb795420d1252a525e9b78fb1eec4b8ad95752",
      "mean_ms": 0.3682840579710145,
      "p999_ms": 0.859,
      "max_ms": 0.881,
      "alive": 500.0,
      "decision_us": 240.70736594202899,
      "decision_share": 0.6535915979316532,
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
    "mean_ms": 0.37240072463768115,
    "p999_ms": 0.873
  },
  "new": {
    "mean_ms": 0.3682840579710145,
    "p999_ms": 0.859
  }
}
```

## slopes

```json
{
  "old": {
    "b": 0.9808891353018943,
    "a": -0.4829213986930103,
    "n": 3,
    "se": 0.05320084984735262,
    "ci": [
      0.30490824500294555,
      1.6568700256008428
    ],
    "r2": 0.9970669355502715
  },
  "new": {
    "b": 0.9730574105873652,
    "a": -0.4360154969846741,
    "n": 3,
    "se": 0.04650765487376333,
    "ci": [
      0.3821216259619771,
      1.5639931952127533
    ],
    "r2": 0.9977208076289152
  },
  "reference": 1.06,
  "old_points": [
    {
      "file": "/Users/rexxa/work/p18-data/perf/official/c/b_game_v22/stress_stress_v22_s15100_n100_side64_fed.json",
      "sha256": "e1948b289a97cda9a43288fc14d40e45912331bbb3f90a3cc83781519e41f560",
      "mean_ms": 0.09246773049645389,
      "p999_ms": 0.217,
      "max_ms": 0.248,
      "alive": 100.0,
      "decision_us": 57.715766312056736,
      "decision_share": 0.624171978723649,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/c/b_game_v22/stress_stress_v22_s15100_n500_side64_fed.json",
      "sha256": "53dab0b6a0d23d931fd185e3330758630e36d311257ca57a9a897a63008904a9",
      "mean_ms": 0.3889301418439716,
      "p999_ms": 0.86,
      "max_ms": 0.938,
      "alive": 500.0,
      "decision_us": 255.23764574468086,
      "decision_share": 0.6562557597993405,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/c/b_game_v22/stress_stress_v22_s15100_n1000_side64_fed.json",
      "sha256": "c0633d0836ac60337cbde6c72c1a696eba2fd8a5525a259748106206de560da7",
      "mean_ms": 0.8337390070921986,
      "p999_ms": 1.805,
      "max_ms": 1.83,
      "alive": 1000.0,
      "decision_us": 568.0761258865248,
      "decision_share": 0.6813596593828366,
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
      "file": "/Users/rexxa/work/p18-data/perf/official/c/b_game_v23/stress_stress_v23_s15100_n100_side64_fed.json",
      "sha256": "bd04b44b53517e18d53a0830668b2a2e9fd7bc241b3988c1b00b8e525f517f0b",
      "mean_ms": 0.09338156028368795,
      "p999_ms": 0.227,
      "max_ms": 1.39,
      "alive": 100.0,
      "decision_us": 58.18870815602837,
      "decision_share": 0.6231284632560683,
      "fingerprint": "dcab41714144fd00",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    },
    {
      "file": "/Users/rexxa/work/p18-data/perf/official/c/b_game_v23/stress_stress_v23_s15100_n500_side64_fed.json",
      "sha256": "0fbbe24cde4e4cc040ef50fe56d13a3abef07a8e9fcdaa1cfa783d7aaaf941ac",
      "mean_ms": 0.39122943262411347,
      "p999_ms": 0.866,
      "max_ms": 0.958,
      "alive": 500.0,
      "decision_us": 257.06469326241137,
      "decision_share": 0.6570689008191127,
      "fingerprint": "def8aaf45aa13987",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    },
    {
      "file": "/Users/rexxa/work/p18-data/perf/official/c/b_game_v23/stress_stress_v23_s15100_n1000_side64_fed.json",
      "sha256": "627f93a6a167b846460a1f9623b794dc5f21697798b51d127b0571f49b7d87ad",
      "mean_ms": 0.8269251773049645,
      "p999_ms": 1.762,
      "max_ms": 1.832,
      "alive": 1000.0,
      "decision_us": 560.5073290780142,
      "decision_share": 0.6778210948961139,
      "fingerprint": "db0a6ee406e8b5a9",
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
    "file": "/Users/rexxa/work/p18-data/perf/official/c/quick_base/quick_stress_v19_s15100_n500_side64_fed.json",
    "sha256": "eecebfe8119581d6a850f7ca9e587131189ea622d9d9a2fcdfc61c445851714c",
    "mean_ms": 0.30954202898550726,
    "p999_ms": 0.8,
    "max_ms": 1.021,
    "alive": 500.0,
    "decision_us": 190.71708405797102,
    "decision_share": 0.6161266199715334,
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
    "file": "/Users/rexxa/work/p18-data/perf/official/c/quick_final/quick_stress_v19_s15100_n500_side64_fed.json",
    "sha256": "b479f8c03d784622a7bc9f3ba77abd679a74c16922b5f193657cb498a1c47fba",
    "mean_ms": 0.30743405797101453,
    "p999_ms": 0.847,
    "max_ms": 1.029,
    "alive": 500.0,
    "decision_us": 187.96376449275363,
    "decision_share": 0.6113953858374248,
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
  "stage": "c",
  "status": 0,
  "builds": {
    "final": {
      "path": "/Users/rexxa/work/p18-data/bin_p3f/worldsim_perf",
      "commit": "57c47981c675c4e70149f2772c21698195a24de7",
      "sha256": "d6a5ef59ebcd81f725fb429559c0c919de8ce14070d4608831d6dbf5e679a295"
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
    "sha256": "726a7d3be8ce57240caf1d9d86137b73db9f58384e75b24d5f3e804175e56cdd"
  },
  "runlog": {
    "file": "RUNLOG.txt",
    "sha256": "3009c02ea3c91b6a11bc67975066bd34f5e12137be46e803e41b07756ae8c84f"
  },
  "indexes": {
    "quick_final/runs.json": "78ba13561ef972347778b0bf8a39b26eb7a48e8a468a897d717777fb9d3c8b79",
    "b_game_v22/runs.json": "08461d090d590998905c44652ddd6ca450c171192cb6e216348890b075ddfee0",
    "b_game_v23/runs.json": "d45181bb4023e4fd1e84b81c8903a3a2f7607f458d2ec81b8fc25c33594e00a3",
    "quick_base/runs.json": "227ae1dd2869a36e0fccb70bd718ef6ed9951bff9c760c9925d797d5262e9d26",
    "a/r1/runs.json": "29fded1c6ca9cb2336919ec448250a01bfb485dadfb5f0697e28cf9029ec958d",
    "a/r3/runs.json": "fe288f9f65cbc394b659d20a9d23e1f42684483eb59da5f3ee048432b883642e",
    "a/r2/runs.json": "5e40e1d694f62664b96b8eb9ff3465cbcd1c1ef6aa8552da0c8edc1311a587b4"
  },
  "manifest_sha256": "bc3fcd118f749c29aacf7adf46cde0c1097ee08e87258ede0eb09bed28d3759a",
  "quiet_log_sha256": "020cce5e90f6bd775c7bb8155149be3faee6174f1f5a7433bd5fe7b53079b731",
  "input_directory": "/Users/rexxa/work/p18-data/perf/official/c",
  "command_records": [
    {
      "index": 0,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p3f/worldsim_perf",
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
        "/Users/rexxa/work/p18-data/perf/official/c/a/r1"
      ],
      "started": 1792089523.428093,
      "binary_sha256_before": "d6a5ef59ebcd81f725fb429559c0c919de8ce14070d4608831d6dbf5e679a295",
      "finished": 1792089524.3704379,
      "exit_code": 0,
      "binary_sha256_after": "d6a5ef59ebcd81f725fb429559c0c919de8ce14070d4608831d6dbf5e679a295",
      "outputs": {
        "a/r1/stress_stress_v22_s15100_n500_side64_fed.json": "864d06dc82871be31773174ea50770290a96b838d10bbcb573d3ca069622dc84"
      }
    },
    {
      "index": 1,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p3f/worldsim_perf",
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
        "game_v23",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/c/a/r1"
      ],
      "started": 1792089524.44748,
      "binary_sha256_before": "d6a5ef59ebcd81f725fb429559c0c919de8ce14070d4608831d6dbf5e679a295",
      "finished": 1792089525.056473,
      "exit_code": 0,
      "binary_sha256_after": "d6a5ef59ebcd81f725fb429559c0c919de8ce14070d4608831d6dbf5e679a295",
      "outputs": {
        "a/r1/stress_stress_v23_s15100_n500_side64_fed.json": "cfbaeea681054823b7f8b6556be935db27813f6743a571a54beea25b92ef1537"
      }
    },
    {
      "index": 2,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p3f/worldsim_perf",
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
        "/Users/rexxa/work/p18-data/perf/official/c/a/r2"
      ],
      "started": 1792089525.134712,
      "binary_sha256_before": "d6a5ef59ebcd81f725fb429559c0c919de8ce14070d4608831d6dbf5e679a295",
      "finished": 1792089525.757426,
      "exit_code": 0,
      "binary_sha256_after": "d6a5ef59ebcd81f725fb429559c0c919de8ce14070d4608831d6dbf5e679a295",
      "outputs": {
        "a/r2/stress_stress_v22_s15100_n500_side64_fed.json": "ff243f63913faca7c6d2fd49d8a8c0067b7f5078501099db6fcef75ebb6d2839"
      }
    },
    {
      "index": 3,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p3f/worldsim_perf",
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
        "game_v23",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/c/a/r2"
      ],
      "started": 1792089525.836148,
      "binary_sha256_before": "d6a5ef59ebcd81f725fb429559c0c919de8ce14070d4608831d6dbf5e679a295",
      "finished": 1792089526.474426,
      "exit_code": 0,
      "binary_sha256_after": "d6a5ef59ebcd81f725fb429559c0c919de8ce14070d4608831d6dbf5e679a295",
      "outputs": {
        "a/r2/stress_stress_v23_s15100_n500_side64_fed.json": "c315b071f2a024215d6e3dff4853f5f842cc0481bcc7d72885d6508f17f0c290"
      }
    },
    {
      "index": 4,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p3f/worldsim_perf",
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
        "/Users/rexxa/work/p18-data/perf/official/c/a/r3"
      ],
      "started": 1792089526.5531478,
      "binary_sha256_before": "d6a5ef59ebcd81f725fb429559c0c919de8ce14070d4608831d6dbf5e679a295",
      "finished": 1792089527.1770802,
      "exit_code": 0,
      "binary_sha256_after": "d6a5ef59ebcd81f725fb429559c0c919de8ce14070d4608831d6dbf5e679a295",
      "outputs": {
        "a/r3/stress_stress_v22_s15100_n500_side64_fed.json": "984efe143c602bb8af8415413fb2388684d144f908d13ebe92c8a18ff25a1137"
      }
    },
    {
      "index": 5,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p3f/worldsim_perf",
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
        "game_v23",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/c/a/r3"
      ],
      "started": 1792089527.247603,
      "binary_sha256_before": "d6a5ef59ebcd81f725fb429559c0c919de8ce14070d4608831d6dbf5e679a295",
      "finished": 1792089527.863352,
      "exit_code": 0,
      "binary_sha256_after": "d6a5ef59ebcd81f725fb429559c0c919de8ce14070d4608831d6dbf5e679a295",
      "outputs": {
        "a/r3/stress_stress_v23_s15100_n500_side64_fed.json": "a85a7c496031947217ed8c2c11bb795420d1252a525e9b78fb1eec4b8ad95752"
      }
    },
    {
      "index": 6,
      "build": "baseline",
      "argv": [
        "/Users/rexxa/work/new-world-p18-base/rust/target/release/worldsim_perf",
        "quick",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/c/quick_base"
      ],
      "started": 1792089527.93908,
      "binary_sha256_before": "216cd0b6c40c246cacf1e1fa7a0e4b2fdcf14672eed38431b8cad7032a85ff25",
      "finished": 1792089528.471933,
      "exit_code": 0,
      "binary_sha256_after": "216cd0b6c40c246cacf1e1fa7a0e4b2fdcf14672eed38431b8cad7032a85ff25",
      "outputs": {
        "quick_base/quick_stress_v19_s15100_n500_side64_fed.json": "eecebfe8119581d6a850f7ca9e587131189ea622d9d9a2fcdfc61c445851714c"
      }
    },
    {
      "index": 7,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p3f/worldsim_perf",
        "quick",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/c/quick_final"
      ],
      "started": 1792089528.54942,
      "binary_sha256_before": "d6a5ef59ebcd81f725fb429559c0c919de8ce14070d4608831d6dbf5e679a295",
      "finished": 1792089529.08148,
      "exit_code": 0,
      "binary_sha256_after": "d6a5ef59ebcd81f725fb429559c0c919de8ce14070d4608831d6dbf5e679a295",
      "outputs": {
        "quick_final/quick_stress_v19_s15100_n500_side64_fed.json": "b479f8c03d784622a7bc9f3ba77abd679a74c16922b5f193657cb498a1c47fba"
      }
    },
    {
      "index": 8,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p3f/worldsim_perf",
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
        "game_v23",
        "--out",
        "/Users/rexxa/work/p18-data/perf/official/c/b_game_v23"
      ],
      "started": 1792089529.158154,
      "binary_sha256_before": "d6a5ef59ebcd81f725fb429559c0c919de8ce14070d4608831d6dbf5e679a295",
      "finished": 1792089533.034317,
      "exit_code": 0,
      "binary_sha256_after": "d6a5ef59ebcd81f725fb429559c0c919de8ce14070d4608831d6dbf5e679a295",
      "outputs": {
        "b_game_v23/stress_stress_v23_s15100_n100_side64_fed.json": "bd04b44b53517e18d53a0830668b2a2e9fd7bc241b3988c1b00b8e525f517f0b",
        "b_game_v23/stress_stress_v23_s15100_n500_side64_fed.json": "0fbbe24cde4e4cc040ef50fe56d13a3abef07a8e9fcdaa1cfa783d7aaaf941ac",
        "b_game_v23/stress_stress_v23_s15100_n1000_side64_fed.json": "627f93a6a167b846460a1f9623b794dc5f21697798b51d127b0571f49b7d87ad"
      }
    },
    {
      "index": 9,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p18-data/bin_p3f/worldsim_perf",
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
        "/Users/rexxa/work/p18-data/perf/official/c/b_game_v22"
      ],
      "started": 1792089533.1132228,
      "binary_sha256_before": "d6a5ef59ebcd81f725fb429559c0c919de8ce14070d4608831d6dbf5e679a295",
      "finished": 1792089537.001318,
      "exit_code": 0,
      "binary_sha256_after": "d6a5ef59ebcd81f725fb429559c0c919de8ce14070d4608831d6dbf5e679a295",
      "outputs": {
        "b_game_v22/stress_stress_v22_s15100_n100_side64_fed.json": "e1948b289a97cda9a43288fc14d40e45912331bbb3f90a3cc83781519e41f560",
        "b_game_v22/stress_stress_v22_s15100_n500_side64_fed.json": "53dab0b6a0d23d931fd185e3330758630e36d311257ca57a9a897a63008904a9",
        "b_game_v22/stress_stress_v22_s15100_n1000_side64_fed.json": "c0633d0836ac60337cbde6c72c1a696eba2fd8a5525a259748106206de560da7"
      }
    }
  ]
}
```
