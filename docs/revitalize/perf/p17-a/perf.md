# Pass 17 stage a performance: FAIL

| Check | Value | Limit | Pass |
|---|---:|---:|---|
| mean_relative | 0.33040145588540026 | <= 0.15 | False |
| mean_absolute | 0.4698833333333333 | <= 4.0 | True |
| p999_relative | 3.3986371379897786 | <= 0.35 | False |
| slope | 1.0013160279134086 | < 1.7 | True |

AgentDecisionSystem: inherited log-log OLS of observed mean alive at 06:00 versus mean microseconds per call; historical slope 1.06.

## repeats

```json
{
  "old": [
    {
      "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r1/stress_stress_v20_s15100_n500_side64_fed.json",
      "sha256": "a96118800d530c26dce375d9a90bc92e5215593a845920d695bb0f9aecfeb8ca",
      "mean_ms": 0.36824855072463764,
      "p999_ms": 1.267,
      "max_ms": 2.607,
      "alive": 500.0,
      "decision_us": 244.37835942028985,
      "decision_share": 0.6636234112514586,
      "fingerprint": "0fa49639a76b0281",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    },
    {
      "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r2/stress_stress_v20_s15100_n500_side64_fed.json",
      "sha256": "d5684802df474541941e7313f99bc31b36b389781553854e4fa6de78484bbfe9",
      "mean_ms": 0.35226231884057974,
      "p999_ms": 0.567,
      "max_ms": 0.834,
      "alive": 500.0,
      "decision_us": 232.32263913043477,
      "decision_share": 0.6595160103842246,
      "fingerprint": "0fa49639a76b0281",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    },
    {
      "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r3/stress_stress_v20_s15100_n500_side64_fed.json",
      "sha256": "2c430f7fb57adc384acd8b4820cdd92fd23f16b6ac99a90061b286f4179a0fc7",
      "mean_ms": 0.3531891304347826,
      "p999_ms": 0.587,
      "max_ms": 0.849,
      "alive": 500.0,
      "decision_us": 233.18183115942028,
      "decision_share": 0.6602180278661718,
      "fingerprint": "0fa49639a76b0281",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    }
  ],
  "new": [
    {
      "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r1/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "f0add76cb2142d4601c7d51acfb8f1aee3b77da2c5fe05fbd44500bc0b838f1c",
      "mean_ms": 0.4701536231884058,
      "p999_ms": 2.646,
      "max_ms": 3.114,
      "alive": 500.0,
      "decision_us": 342.7775869565217,
      "decision_share": 0.7290757106835262,
      "fingerprint": "14458e1fa711de0c",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    },
    {
      "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r2/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "5ca518ccdec62c5f1e32527b4c44e09c5934a2cda1b7e0003448a88bafc38d70",
      "mean_ms": 0.46749855072463764,
      "p999_ms": 2.53,
      "max_ms": 3.028,
      "alive": 500.0,
      "decision_us": 340.66000942028984,
      "decision_share": 0.7286867710974846,
      "fingerprint": "14458e1fa711de0c",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    },
    {
      "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r3/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "5fe914ece0fd2f11cfaf3e99f75d58f5f519a7815d3e008cbff8146f0a393af3",
      "mean_ms": 0.4698833333333333,
      "p999_ms": 2.582,
      "max_ms": 3.207,
      "alive": 500.0,
      "decision_us": 342.6569405797101,
      "decision_share": 0.729238336990835,
      "fingerprint": "14458e1fa711de0c",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    }
  ]
}
```

## medians

```json
{
  "old": {
    "mean_ms": 0.3531891304347826,
    "p999_ms": 0.587
  },
  "new": {
    "mean_ms": 0.4698833333333333,
    "p999_ms": 2.582
  }
}
```

## slopes

```json
{
  "old": {
    "b": 1.0516753956326028,
    "a": -1.1488279535971158,
    "n": 3,
    "se": 0.04053454003967519,
    "ci": [
      0.5366352310018206,
      1.5667155602633849
    ],
    "r2": 0.9985166542748841
  },
  "new": {
    "b": 1.0013160279134086,
    "a": -0.3899074213967184,
    "n": 3,
    "se": 0.053493246070234104,
    "ci": [
      0.3216198913424435,
      1.6810121644843736
    ],
    "r2": 0.9971541116780275
  },
  "reference": 1.06,
  "old_points": [
    {
      "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/b_game_v20/stress_stress_v20_s15100_n100_side64_fed.json",
      "sha256": "8d7d25df03bb4a92fd0a303a4e70e25a6edc37c96176f7cd6e1e64477f90f916",
      "mean_ms": 0.07218014184397163,
      "p999_ms": 0.172,
      "max_ms": 0.191,
      "alive": 100.0,
      "decision_us": 40.87577907801419,
      "decision_share": 0.5663022825083027,
      "fingerprint": "e44f8265b2219b8e",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    },
    {
      "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/b_game_v20/stress_stress_v20_s15100_n500_side64_fed.json",
      "sha256": "6a750daa10dc741441ae81e3a9891697929095538260f65a4da1202b3e66ea19",
      "mean_ms": 0.3370964539007092,
      "p999_ms": 0.656,
      "max_ms": 0.987,
      "alive": 500.0,
      "decision_us": 207.06565886524822,
      "decision_share": 0.6142623467829146,
      "fingerprint": "10746baa35b729ed",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    },
    {
      "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/b_game_v20/stress_stress_v20_s15100_n1000_side64_fed.json",
      "sha256": "4d84647b7594dfc032d2da8834f5f38db27e595e2183cfa6dd347f4e877cc347",
      "mean_ms": 0.7230078014184397,
      "p999_ms": 1.369,
      "max_ms": 1.75,
      "alive": 1000.0,
      "decision_us": 470.3858843971631,
      "decision_share": 0.6505958628307082,
      "fingerprint": "603177818e8fb210",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    }
  ],
  "new_points": [
    {
      "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/b_game_v21/stress_stress_v21_s15100_n100_side64_fed.json",
      "sha256": "e325de81268b32f44ea53f2c0aa8bc68b8bd03d3546b21f19d8e3cdcfcd9326d",
      "mean_ms": 0.1033372340425532,
      "p999_ms": 0.258,
      "max_ms": 0.323,
      "alive": 100.0,
      "decision_us": 69.59764219858155,
      "decision_share": 0.6735001458421268,
      "fingerprint": "bea6a0b9ada4edc7",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    },
    {
      "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/b_game_v21/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "4d20c5c4bf3e9eda927b002d601c62f9d5738c3984c86969e3046d9178ab137b",
      "mean_ms": 0.45132801418439716,
      "p999_ms": 2.737,
      "max_ms": 3.079,
      "alive": 500.0,
      "decision_us": 317.9090875886525,
      "decision_share": 0.704385895839308,
      "fingerprint": "7ce50f96939b36b6",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    },
    {
      "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/b_game_v21/stress_stress_v21_s15100_n1000_side64_fed.json",
      "sha256": "5ad1d301c1aa8bdb440e5b0d4fb00b7dcb5ccabdf61955eb32d759648f009c93",
      "mean_ms": 0.9876166666666667,
      "p999_ms": 5.574,
      "max_ms": 6.175,
      "alive": 1000.0,
      "decision_us": 718.1260308510638,
      "decision_share": 0.7271303280804602,
      "fingerprint": "fbf486f2e08c29ff",
      "machine": {
        "cpu": "Apple M4 Max",
        "memsize": "38654705664",
        "ncpu": "14",
        "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
        "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
      }
    }
  ]
}
```

## quick

```json
{
  "base": {
    "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/quick_base/quick_stress_v19_s15100_n500_side64_fed.json",
    "sha256": "ca79b3e496236651e472ed4972ec4c561d5dcd6794fca3bcdb2f94053a410a75",
    "mean_ms": 0.3047492753623188,
    "p999_ms": 0.597,
    "max_ms": 0.804,
    "alive": 500.0,
    "decision_us": 184.99849347826088,
    "decision_share": 0.6070514630701408,
    "fingerprint": "54ac4928a770f238",
    "machine": {
      "cpu": "Apple M4 Max",
      "memsize": "38654705664",
      "ncpu": "14",
      "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
      "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
    }
  },
  "final": {
    "file": "/Users/rexxa/work/p17-data/a-perf-observerfix/quick_final/quick_stress_v19_s15100_n500_side64_fed.json",
    "sha256": "39c0ac7d4607224d43999918e0399cf42f27deb6610170f42ce7f6e5a280ee04",
    "mean_ms": 0.30321884057971016,
    "p999_ms": 0.609,
    "max_ms": 0.816,
    "alive": 500.0,
    "decision_us": 184.470518115942,
    "decision_share": 0.6083741952289684,
    "fingerprint": "54ac4928a770f238",
    "machine": {
      "cpu": "Apple M4 Max",
      "memsize": "38654705664",
      "ncpu": "14",
      "rustc": "rustc 1.93.1 (01f6ddf75 2026-02-11)",
      "uname": "Darwin 10.nate.com 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64"
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
      "path": "/Users/rexxa/work/p17-data/a-observerfix-bin/worldsim_perf",
      "commit": "50a73474e06d3a9abbb535729e9c7e1e134954b7",
      "sha256": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38"
    },
    "baseline": {
      "path": "/Users/rexxa/work/p17-data/base-bin/worldsim_perf",
      "commit": "adfdecfece79ab019964729319389fd602685fab",
      "sha256": "774ec9c383a2580770e6d8b487928f5cbd0df7f85eb4517aa4bc3bec93951ded"
    }
  },
  "quiet": {
    "owner": "pass17-sim (Codex, new-world-pass4)",
    "pid": "24639",
    "purpose": "Stage A corrected-build performance",
    "started": "2026-10-15T08:20:08Z",
    "log": "/Users/rexxa/work/p17-data/a-perf-observerfix/quiet.log",
    "prefix_bytes": 178,
    "prefix_sha256": "74be75ba1be87ded142d6dd7920997e90a59517b636e8145e8ea240dca45b874",
    "contended": false
  },
  "commands": {
    "file": "commands.jsonl",
    "sha256": "2ee5b361c678b83237518fdadf8ac4da20494b97879e46132379f7adcc5a292f"
  },
  "runlog": {
    "file": "RUNLOG.txt",
    "sha256": "3c03c15fba1b03bf66b14b7ac0745947ba556132a82828018a0a5ab842375991"
  },
  "indexes": {
    "quick_final/runs.json": "1f1c5433072e6935567c92f64ed9e2a39a90242a5ef710cbdcfd529f7d43a493",
    "b_game_v21/runs.json": "c4864a3d924e60836e322d4d05c6579133b8ef644793b7a09b8640b6e59d17ad",
    "b_game_v20/runs.json": "f00b469612661cc5f79db3d66f4e14f6f4d42a8342576224fd58f138f0e728be",
    "quick_base/runs.json": "8f91108365cfe0a4f8f5b2b89be53333ed64833120df6a905ade959c0a75a8f7",
    "a/r1/runs.json": "f14ca8a99273a0d5a04558b27e1cc211eec5f41e76bf472ad659ee60dc178adc",
    "a/r3/runs.json": "29373f16c285309c9e2b5962c82ae3032b197d8e528898083282fd70e2fd61fb",
    "a/r2/runs.json": "41d3f20c15c0b64d44fb95260df887bf0851c3aab2bf033d90c757b6df4a1476"
  },
  "manifest_sha256": "8eb105e3c79faa9ce22c2a945b16f31bc49ed7401633ce051041dd18c5a82388",
  "quiet_log_sha256": "bffb2e993061681b4749e4508a214f9bff6ae30e670e28591e7de19c9facd8f9",
  "input_directory": "/Users/rexxa/work/p17-data/a-perf-observerfix",
  "command_records": [
    {
      "index": 0,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p17-data/a-observerfix-bin/worldsim_perf",
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
        "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r1"
      ],
      "started": 1792052408.3640478,
      "binary_sha256_before": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
      "finished": 1792052409.906317,
      "exit_code": 0,
      "binary_sha256_after": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
      "outputs": {
        "a/r1/stress_stress_v20_s15100_n500_side64_fed.json": "a96118800d530c26dce375d9a90bc92e5215593a845920d695bb0f9aecfeb8ca"
      }
    },
    {
      "index": 1,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p17-data/a-observerfix-bin/worldsim_perf",
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
        "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r1"
      ],
      "started": 1792052409.9684951,
      "binary_sha256_before": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
      "finished": 1792052410.7348511,
      "exit_code": 0,
      "binary_sha256_after": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
      "outputs": {
        "a/r1/stress_stress_v21_s15100_n500_side64_fed.json": "f0add76cb2142d4601c7d51acfb8f1aee3b77da2c5fe05fbd44500bc0b838f1c"
      }
    },
    {
      "index": 2,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p17-data/a-observerfix-bin/worldsim_perf",
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
        "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r2"
      ],
      "started": 1792052410.796533,
      "binary_sha256_before": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
      "finished": 1792052411.38445,
      "exit_code": 0,
      "binary_sha256_after": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
      "outputs": {
        "a/r2/stress_stress_v20_s15100_n500_side64_fed.json": "d5684802df474541941e7313f99bc31b36b389781553854e4fa6de78484bbfe9"
      }
    },
    {
      "index": 3,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p17-data/a-observerfix-bin/worldsim_perf",
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
        "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r2"
      ],
      "started": 1792052411.445913,
      "binary_sha256_before": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
      "finished": 1792052412.2049751,
      "exit_code": 0,
      "binary_sha256_after": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
      "outputs": {
        "a/r2/stress_stress_v21_s15100_n500_side64_fed.json": "5ca518ccdec62c5f1e32527b4c44e09c5934a2cda1b7e0003448a88bafc38d70"
      }
    },
    {
      "index": 4,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p17-data/a-observerfix-bin/worldsim_perf",
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
        "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r3"
      ],
      "started": 1792052412.267819,
      "binary_sha256_before": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
      "finished": 1792052412.8559191,
      "exit_code": 0,
      "binary_sha256_after": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
      "outputs": {
        "a/r3/stress_stress_v20_s15100_n500_side64_fed.json": "2c430f7fb57adc384acd8b4820cdd92fd23f16b6ac99a90061b286f4179a0fc7"
      }
    },
    {
      "index": 5,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p17-data/a-observerfix-bin/worldsim_perf",
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
        "/Users/rexxa/work/p17-data/a-perf-observerfix/a/r3"
      ],
      "started": 1792052412.9158552,
      "binary_sha256_before": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
      "finished": 1792052413.677444,
      "exit_code": 0,
      "binary_sha256_after": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
      "outputs": {
        "a/r3/stress_stress_v21_s15100_n500_side64_fed.json": "5fe914ece0fd2f11cfaf3e99f75d58f5f519a7815d3e008cbff8146f0a393af3"
      }
    },
    {
      "index": 6,
      "build": "baseline",
      "argv": [
        "/Users/rexxa/work/p17-data/base-bin/worldsim_perf",
        "quick",
        "--out",
        "/Users/rexxa/work/p17-data/a-perf-observerfix/quick_base"
      ],
      "started": 1792052413.738869,
      "binary_sha256_before": "774ec9c383a2580770e6d8b487928f5cbd0df7f85eb4517aa4bc3bec93951ded",
      "finished": 1792052414.536443,
      "exit_code": 0,
      "binary_sha256_after": "774ec9c383a2580770e6d8b487928f5cbd0df7f85eb4517aa4bc3bec93951ded",
      "outputs": {
        "quick_base/quick_stress_v19_s15100_n500_side64_fed.json": "ca79b3e496236651e472ed4972ec4c561d5dcd6794fca3bcdb2f94053a410a75"
      }
    },
    {
      "index": 7,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p17-data/a-observerfix-bin/worldsim_perf",
        "quick",
        "--out",
        "/Users/rexxa/work/p17-data/a-perf-observerfix/quick_final"
      ],
      "started": 1792052414.597124,
      "binary_sha256_before": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
      "finished": 1792052415.112693,
      "exit_code": 0,
      "binary_sha256_after": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
      "outputs": {
        "quick_final/quick_stress_v19_s15100_n500_side64_fed.json": "39c0ac7d4607224d43999918e0399cf42f27deb6610170f42ce7f6e5a280ee04"
      }
    },
    {
      "index": 8,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p17-data/a-observerfix-bin/worldsim_perf",
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
        "/Users/rexxa/work/p17-data/a-perf-observerfix/b_game_v21"
      ],
      "started": 1792052415.173277,
      "binary_sha256_before": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
      "finished": 1792052419.733844,
      "exit_code": 0,
      "binary_sha256_after": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
      "outputs": {
        "b_game_v21/stress_stress_v21_s15100_n100_side64_fed.json": "e325de81268b32f44ea53f2c0aa8bc68b8bd03d3546b21f19d8e3cdcfcd9326d",
        "b_game_v21/stress_stress_v21_s15100_n500_side64_fed.json": "4d20c5c4bf3e9eda927b002d601c62f9d5738c3984c86969e3046d9178ab137b",
        "b_game_v21/stress_stress_v21_s15100_n1000_side64_fed.json": "5ad1d301c1aa8bdb440e5b0d4fb00b7dcb5ccabdf61955eb32d759648f009c93"
      }
    },
    {
      "index": 9,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p17-data/a-observerfix-bin/worldsim_perf",
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
        "/Users/rexxa/work/p17-data/a-perf-observerfix/b_game_v20"
      ],
      "started": 1792052419.796577,
      "binary_sha256_before": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
      "finished": 1792052423.164577,
      "exit_code": 0,
      "binary_sha256_after": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
      "outputs": {
        "b_game_v20/stress_stress_v20_s15100_n100_side64_fed.json": "8d7d25df03bb4a92fd0a303a4e70e25a6edc37c96176f7cd6e1e64477f90f916",
        "b_game_v20/stress_stress_v20_s15100_n500_side64_fed.json": "6a750daa10dc741441ae81e3a9891697929095538260f65a4da1202b3e66ea19",
        "b_game_v20/stress_stress_v20_s15100_n1000_side64_fed.json": "4d84647b7594dfc032d2da8834f5f38db27e595e2183cfa6dd347f4e877cc347"
      }
    }
  ]
}
```
