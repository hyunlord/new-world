# Pass 17 stage b performance: PASS

| Check | Value | Limit | Pass |
|---|---:|---:|---|
| mean_relative | 0.01202441891564071 | <= 0.15 | True |
| mean_absolute | 0.4716239130434782 | <= 4.0 | True |
| p999_relative | 0.020448289421942586 | <= 0.35 | True |
| slope | 0.9881343412222895 | < 1.7 | True |

AgentDecisionSystem: inherited log-log OLS of observed mean alive at 06:00 versus mean microseconds per call; historical slope 1.06.

## repeats

```json
{
  "old": [
    {
      "file": "/Users/rexxa/work/p17-data/b-perf-final/a/r1/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "0d09c30b7ce0f3f16f904f880b8278b976592b7cbec89ab0eddcee8dbe127fc8",
      "mean_ms": 0.4706,
      "p999_ms": 2.583,
      "max_ms": 2.834,
      "alive": 500.0,
      "decision_us": 344.1830746376812,
      "decision_share": 0.731370749336339,
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
      "file": "/Users/rexxa/work/p17-data/b-perf-final/a/r2/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "6075e93117b297a4212fde93ad8691df21e8fa2dc91f1ead1ec6b6f2d5ce923b",
      "mean_ms": 0.466018115942029,
      "p999_ms": 2.539,
      "max_ms": 3.071,
      "alive": 500.0,
      "decision_us": 341.061734057971,
      "decision_share": 0.7318636816694007,
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
      "file": "/Users/rexxa/work/p17-data/b-perf-final/a/r3/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "6f2a8df10e2859db0ba1c1fb7a1de3eb56860f19b2c2006320da972052f00bc0",
      "mean_ms": 0.4660202898550725,
      "p999_ms": 2.543,
      "max_ms": 3.024,
      "alive": 500.0,
      "decision_us": 340.9403079710145,
      "decision_share": 0.7315997079806191,
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
      "file": "/Users/rexxa/work/p17-data/b-perf-final/a/r1/stress_stress_v22_s15100_n500_side64_fed.json",
      "sha256": "8e261b07d9573e17817bf527df9ce5790e717fdaf3428b713760f186e7d150a9",
      "mean_ms": 0.47577971014492754,
      "p999_ms": 2.595,
      "max_ms": 3.137,
      "alive": 500.0,
      "decision_us": 347.4572173913043,
      "decision_share": 0.7302901111219418,
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
      "file": "/Users/rexxa/work/p17-data/b-perf-final/a/r2/stress_stress_v22_s15100_n500_side64_fed.json",
      "sha256": "5287416a758f9ed2ecdee43f9afd7fb919cb0f994a1a028167c7cda4306f43b1",
      "mean_ms": 0.47132246376811593,
      "p999_ms": 2.624,
      "max_ms": 3.07,
      "alive": 500.0,
      "decision_us": 344.10375217391305,
      "decision_share": 0.730081374485913,
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
      "file": "/Users/rexxa/work/p17-data/b-perf-final/a/r3/stress_stress_v22_s15100_n500_side64_fed.json",
      "sha256": "74aeec136018dcf4c4d96a4d993e9b5f19393bd337d266bc3c86ac947c96ea87",
      "mean_ms": 0.4716239130434782,
      "p999_ms": 2.56,
      "max_ms": 3.076,
      "alive": 500.0,
      "decision_us": 344.49191231884055,
      "decision_share": 0.7304377551506436,
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
    "mean_ms": 0.4660202898550725,
    "p999_ms": 2.543
  },
  "new": {
    "mean_ms": 0.4716239130434782,
    "p999_ms": 2.595
  }
}
```

## slopes

```json
{
  "old": {
    "b": 0.9885316536105887,
    "a": -0.19856894975199513,
    "n": 3,
    "se": 0.047107594848563986,
    "ci": [
      0.389972908835968,
      1.5870903983852094
    ],
    "r2": 0.997734231292559
  },
  "new": {
    "b": 0.9881343412222895,
    "a": -0.17413654248912458,
    "n": 3,
    "se": 0.04969259234051691,
    "ci": [
      0.356730089072417,
      1.619538593372162
    ],
    "r2": 0.9974773652695818
  },
  "reference": 1.06,
  "old_points": [
    {
      "file": "/Users/rexxa/work/p17-data/b-perf-final/b_game_v21/stress_stress_v21_s15100_n100_side64_fed.json",
      "sha256": "709e9d2d3dc93c85a32c9d6d8ce6d0a9f68939825750e0c12dbf5fd52f7bca35",
      "mean_ms": 0.11312765957446809,
      "p999_ms": 0.265,
      "max_ms": 0.33,
      "alive": 100.0,
      "decision_us": 79.25254113475178,
      "decision_share": 0.7005584790922199,
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
      "file": "/Users/rexxa/work/p17-data/b-perf-final/b_game_v21/stress_stress_v21_s15100_n500_side64_fed.json",
      "sha256": "74f12df1bb3780d2724776460bc3156832077a05831345f3b5650b58d13dd90b",
      "mean_ms": 0.49170709219858155,
      "p999_ms": 2.714,
      "max_ms": 3.067,
      "alive": 500.0,
      "decision_us": 358.57689290780144,
      "decision_share": 0.7292489748408714,
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
      "file": "/Users/rexxa/work/p17-data/b-perf-final/b_game_v21/stress_stress_v21_s15100_n1000_side64_fed.json",
      "sha256": "701d0ede899ffaf0a115225d22134d4e6fe294dc8e896434b940ef8d56fe9600",
      "mean_ms": 1.0567570921985816,
      "p999_ms": 5.479,
      "max_ms": 6.308,
      "alive": 1000.0,
      "decision_us": 791.3486322695036,
      "decision_share": 0.7488462941120214,
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
      "file": "/Users/rexxa/work/p17-data/b-perf-final/b_game_v22/stress_stress_v22_s15100_n100_side64_fed.json",
      "sha256": "5f29476907a3fce6495276a60a54631ad3febb05e7aa5c5d33f26cfea5409b79",
      "mean_ms": 0.11577695035460993,
      "p999_ms": 0.26,
      "max_ms": 0.343,
      "alive": 100.0,
      "decision_us": 81.14816489361702,
      "decision_share": 0.70090086709894,
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
      "file": "/Users/rexxa/work/p17-data/b-perf-final/b_game_v22/stress_stress_v22_s15100_n500_side64_fed.json",
      "sha256": "f383fbf91ca3cdf9bb15422dc2c3932ec456664a52da493545636ec0c5d3d019",
      "mean_ms": 0.49998156028368795,
      "p999_ms": 2.734,
      "max_ms": 3.162,
      "alive": 500.0,
      "decision_us": 365.28208758865253,
      "decision_share": 0.7305911189632526,
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
      "file": "/Users/rexxa/work/p17-data/b-perf-final/b_game_v22/stress_stress_v22_s15100_n1000_side64_fed.json",
      "sha256": "5acfd9c264603adbb124edb99aba44c01c8495c2d1dcaf0b434f9cdb40023cec",
      "mean_ms": 1.0780354609929077,
      "p999_ms": 5.566,
      "max_ms": 6.484,
      "alive": 1000.0,
      "decision_us": 810.643595035461,
      "decision_share": 0.7519637566363822,
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
    "file": "/Users/rexxa/work/p17-data/b-perf-final/quick_base/quick_stress_v19_s15100_n500_side64_fed.json",
    "sha256": "e588dae8b2bcd84859625d91feb08c3981f1aa701387d2853521381d39461810",
    "mean_ms": 0.3039905797101449,
    "p999_ms": 0.615,
    "max_ms": 0.811,
    "alive": 500.0,
    "decision_us": 184.16406594202897,
    "decision_share": 0.6058216215700811,
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
    "file": "/Users/rexxa/work/p17-data/b-perf-final/quick_final/quick_stress_v19_s15100_n500_side64_fed.json",
    "sha256": "28c0177d7395ffcf63c0ca01c37b98af4f6dbdc7545f9be16bddc06d761fc34d",
    "mean_ms": 0.3055442028985507,
    "p999_ms": 0.588,
    "max_ms": 0.832,
    "alive": 500.0,
    "decision_us": 186.8194956521739,
    "decision_share": 0.6114319757334857,
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
      "path": "/Users/rexxa/work/p17-data/b-final-bin/worldsim_perf",
      "commit": "47b7244052fcf616bf2efbe7e3f3f9e11aa6fc21",
      "sha256": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f"
    },
    "baseline": {
      "path": "/Users/rexxa/work/p17-data/stage-a-bin/worldsim_perf",
      "commit": "50a73474e06d3a9abbb535729e9c7e1e134954b7",
      "sha256": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38"
    }
  },
  "quiet": {
    "owner": "pass17-sim (Codex, new-world-pass4)",
    "pid": "38429",
    "purpose": "Stage B final performance",
    "started": "2026-10-15T10:43:58Z",
    "log": "/Users/rexxa/work/p17-data/b-perf-final/quiet.log",
    "prefix_bytes": 158,
    "prefix_sha256": "97e484d7f68f1808254ba4b3f48f636ba6535ae437281b117565c44d498668cc",
    "contended": false
  },
  "commands": {
    "file": "commands.jsonl",
    "sha256": "d7ff1bcfa3892f83e1cbecf2d483632456a8cb1ed3a1d25fe2a788f61efcdbde"
  },
  "runlog": {
    "file": "RUNLOG.txt",
    "sha256": "636ced85f0dc80259f3e9744bc0a38c19790c4ce4b37226f92f6862c9f6fb15c"
  },
  "indexes": {
    "quick_final/runs.json": "1ec46415d46a8fdea4c3f7d876eb2d119bd0539f25f2b49caf9ad9039941356c",
    "b_game_v21/runs.json": "c4800fd03cc51cf56f461af4feaa9571bbdbb8b295b5e383132883d81376c6a2",
    "b_game_v22/runs.json": "f4438bb15df7a7b64f28fbf91357aa4406e6530a3fcddd000bc1fc479888ce93",
    "quick_base/runs.json": "49bc21e7f9156dd592508a7f07fc042fb25f8c3eb491d2d671d05da9a424fd6f",
    "a/r1/runs.json": "264551da332ad018ee39d83849d758cc558c8a33ef1405afc2932383cb2bc07c",
    "a/r3/runs.json": "544b0a1e95b8aa9d488363bebb4bd590c50001bd291dd17e5bee6049cb3c883a",
    "a/r2/runs.json": "8c5ae0f452097b48760707228a8e729c7afa7cbb2cbcb4573fb83d85ce1db347"
  },
  "manifest_sha256": "8e8193193e62f480355ad2253fff450512e007a176668e8c5f31f3bbfd6c0640",
  "quiet_log_sha256": "9f4118c9a3702485bfc5bfd9cf9c3a7ec05e6585b2f176c7dd22bdc026c0003f",
  "input_directory": "/Users/rexxa/work/p17-data/b-perf-final",
  "command_records": [
    {
      "index": 0,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p17-data/b-final-bin/worldsim_perf",
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
        "/Users/rexxa/work/p17-data/b-perf-final/a/r1"
      ],
      "started": 1792061038.784953,
      "binary_sha256_before": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
      "finished": 1792061039.888675,
      "exit_code": 0,
      "binary_sha256_after": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
      "outputs": {
        "a/r1/stress_stress_v21_s15100_n500_side64_fed.json": "0d09c30b7ce0f3f16f904f880b8278b976592b7cbec89ab0eddcee8dbe127fc8"
      }
    },
    {
      "index": 1,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p17-data/b-final-bin/worldsim_perf",
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
        "/Users/rexxa/work/p17-data/b-perf-final/a/r1"
      ],
      "started": 1792061039.951224,
      "binary_sha256_before": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
      "finished": 1792061040.721879,
      "exit_code": 0,
      "binary_sha256_after": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
      "outputs": {
        "a/r1/stress_stress_v22_s15100_n500_side64_fed.json": "8e261b07d9573e17817bf527df9ce5790e717fdaf3428b713760f186e7d150a9"
      }
    },
    {
      "index": 2,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p17-data/b-final-bin/worldsim_perf",
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
        "/Users/rexxa/work/p17-data/b-perf-final/a/r2"
      ],
      "started": 1792061040.78246,
      "binary_sha256_before": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
      "finished": 1792061041.5382771,
      "exit_code": 0,
      "binary_sha256_after": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
      "outputs": {
        "a/r2/stress_stress_v21_s15100_n500_side64_fed.json": "6075e93117b297a4212fde93ad8691df21e8fa2dc91f1ead1ec6b6f2d5ce923b"
      }
    },
    {
      "index": 3,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p17-data/b-final-bin/worldsim_perf",
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
        "/Users/rexxa/work/p17-data/b-perf-final/a/r2"
      ],
      "started": 1792061041.600702,
      "binary_sha256_before": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
      "finished": 1792061042.364175,
      "exit_code": 0,
      "binary_sha256_after": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
      "outputs": {
        "a/r2/stress_stress_v22_s15100_n500_side64_fed.json": "5287416a758f9ed2ecdee43f9afd7fb919cb0f994a1a028167c7cda4306f43b1"
      }
    },
    {
      "index": 4,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p17-data/b-final-bin/worldsim_perf",
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
        "/Users/rexxa/work/p17-data/b-perf-final/a/r3"
      ],
      "started": 1792061042.426471,
      "binary_sha256_before": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
      "finished": 1792061043.1825862,
      "exit_code": 0,
      "binary_sha256_after": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
      "outputs": {
        "a/r3/stress_stress_v21_s15100_n500_side64_fed.json": "6f2a8df10e2859db0ba1c1fb7a1de3eb56860f19b2c2006320da972052f00bc0"
      }
    },
    {
      "index": 5,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p17-data/b-final-bin/worldsim_perf",
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
        "/Users/rexxa/work/p17-data/b-perf-final/a/r3"
      ],
      "started": 1792061043.243872,
      "binary_sha256_before": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
      "finished": 1792061044.008727,
      "exit_code": 0,
      "binary_sha256_after": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
      "outputs": {
        "a/r3/stress_stress_v22_s15100_n500_side64_fed.json": "74aeec136018dcf4c4d96a4d993e9b5f19393bd337d266bc3c86ac947c96ea87"
      }
    },
    {
      "index": 6,
      "build": "baseline",
      "argv": [
        "/Users/rexxa/work/p17-data/stage-a-bin/worldsim_perf",
        "quick",
        "--out",
        "/Users/rexxa/work/p17-data/b-perf-final/quick_base"
      ],
      "started": 1792061044.071317,
      "binary_sha256_before": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
      "finished": 1792061044.84062,
      "exit_code": 0,
      "binary_sha256_after": "2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38",
      "outputs": {
        "quick_base/quick_stress_v19_s15100_n500_side64_fed.json": "e588dae8b2bcd84859625d91feb08c3981f1aa701387d2853521381d39461810"
      }
    },
    {
      "index": 7,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p17-data/b-final-bin/worldsim_perf",
        "quick",
        "--out",
        "/Users/rexxa/work/p17-data/b-perf-final/quick_final"
      ],
      "started": 1792061044.903966,
      "binary_sha256_before": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
      "finished": 1792061045.423543,
      "exit_code": 0,
      "binary_sha256_after": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
      "outputs": {
        "quick_final/quick_stress_v19_s15100_n500_side64_fed.json": "28c0177d7395ffcf63c0ca01c37b98af4f6dbdc7545f9be16bddc06d761fc34d"
      }
    },
    {
      "index": 8,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p17-data/b-final-bin/worldsim_perf",
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
        "/Users/rexxa/work/p17-data/b-perf-final/b_game_v22"
      ],
      "started": 1792061045.485229,
      "binary_sha256_before": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
      "finished": 1792061050.47699,
      "exit_code": 0,
      "binary_sha256_after": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
      "outputs": {
        "b_game_v22/stress_stress_v22_s15100_n100_side64_fed.json": "5f29476907a3fce6495276a60a54631ad3febb05e7aa5c5d33f26cfea5409b79",
        "b_game_v22/stress_stress_v22_s15100_n500_side64_fed.json": "f383fbf91ca3cdf9bb15422dc2c3932ec456664a52da493545636ec0c5d3d019",
        "b_game_v22/stress_stress_v22_s15100_n1000_side64_fed.json": "5acfd9c264603adbb124edb99aba44c01c8495c2d1dcaf0b434f9cdb40023cec"
      }
    },
    {
      "index": 9,
      "build": "final",
      "argv": [
        "/Users/rexxa/work/p17-data/b-final-bin/worldsim_perf",
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
        "/Users/rexxa/work/p17-data/b-perf-final/b_game_v21"
      ],
      "started": 1792061050.538188,
      "binary_sha256_before": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
      "finished": 1792061055.4371798,
      "exit_code": 0,
      "binary_sha256_after": "89b2fb39321423c5db5990823e187859a3e3e8b0f12b6ca62f214cd03106813f",
      "outputs": {
        "b_game_v21/stress_stress_v21_s15100_n100_side64_fed.json": "709e9d2d3dc93c85a32c9d6d8ce6d0a9f68939825750e0c12dbf5fd52f7bca35",
        "b_game_v21/stress_stress_v21_s15100_n500_side64_fed.json": "74f12df1bb3780d2724776460bc3156832077a05831345f3b5650b58d13dd90b",
        "b_game_v21/stress_stress_v21_s15100_n1000_side64_fed.json": "701d0ede899ffaf0a115225d22134d4e6fe294dc8e896434b940ef8d56fe9600"
      }
    }
  ]
}
```
