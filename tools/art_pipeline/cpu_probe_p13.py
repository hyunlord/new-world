#!/usr/bin/env python3
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///
# How to run: python3 cpu_probe_p13.py [calibrate OUTPUT_JSON]
"""Fixed-work interference probe; calibration never changes the kernel workload."""
from __future__ import annotations

import json
import os
import platform
import statistics
import subprocess
import sys
import time
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final

SWEEPS: Final = 64
WORKING_SET_BYTES: Final = 256 * 1024
KERNEL_ID: Final = "p13-int32-byte-sweep-v1-64"


@dataclass(frozen=True, slots=True)
class Probe:
    epoch: float
    wall_seconds: float
    load_1m: float
    ps: str
    checksum: int


def kernel(sweeps: int = SWEEPS) -> int:
    """Sweep every byte of a 256-KiB buffer using deterministic 32-bit mixing."""
    working = bytearray(WORKING_SET_BYTES)
    state = 0x13C0FFEE
    for _ in range(sweeps):
        for index in range(WORKING_SET_BYTES):
            state = ((state ^ (state >> 13)) * 1664525 + 1013904223) & 0xFFFFFFFF
            working[index] = (working[index] + state) & 255
    return state ^ sum(working)


def snapshot() -> Probe:
    """Record process evidence without treating other processes as disqualifying."""
    ps = subprocess.check_output(["ps", "-Ao", "pid,pcpu,comm"], text=True)
    epoch = time.time()
    start = time.perf_counter()
    checksum = kernel()
    return Probe(epoch, time.perf_counter() - start, os.getloadavg()[0], ps, checksum)


def reference_seconds(seconds: list[float]) -> float:
    """The reference is the median of exactly the five fastest of fifteen runs."""
    if len(seconds) != 15 or any(not 0 < item < float("inf") for item in seconds):
        raise ValueError("reference requires fifteen finite positive durations")
    return statistics.median(sorted(seconds)[:5])


def write_json(path: Path, value: dict) -> None:
    """Atomically publish durable evidence, including partial calibration."""
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    with temporary.open("w") as handle:
        json.dump(value, handle, indent=2)
        handle.write("\n")
        handle.flush()
        os.fsync(handle.fileno())
    temporary.replace(path)


def calibrate(path: Path) -> None:
    """Preserve every run and wait twenty seconds after each finished probe."""
    if path.exists():
        raise FileExistsError(path)
    runs: list[Probe] = []
    for index in range(15):
        if index:
            time.sleep(20)
        runs.append(snapshot())
        record = {"kernel_id": KERNEL_ID, "sweeps": SWEEPS,
                  "working_set_bytes": WORKING_SET_BYTES, "python": sys.version,
                  "platform": platform.platform(), "runs": [asdict(run) for run in runs],
                  "complete": len(runs) == 15, "interval_seconds": 20}
        if len(runs) == 15:
            record["reference_seconds"] = reference_seconds([run.wall_seconds for run in runs])
        write_json(path, record)
        print(f"probe {index + 1}/15: {runs[-1].wall_seconds:.6f}s", flush=True)


def main() -> None:
    if len(sys.argv) == 3 and sys.argv[1] == "calibrate":
        calibrate(Path(sys.argv[2]))
    elif len(sys.argv) == 1:
        print(json.dumps(asdict(snapshot()), indent=2))
    else:
        raise SystemExit("usage: cpu_probe_p13.py [calibrate OUTPUT_JSON]")


if __name__ == "__main__":
    main()
