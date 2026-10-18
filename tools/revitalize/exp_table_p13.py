#!/usr/bin/env python3
"""Pass 13 (PASS13_GOAL.md 7.1): the fixed-point table of the exponential quantile.

E(u) = -ln(1 - u) in thousandths at u = i / 1024 for i = 0..1023, and the top entry (i = 1024) capped at
E(1 - 1/2048) = ln(2048); 1,025 entries, rounded to the nearest integer. The simulation never evaluates a
logarithm: `sim_core::fission_threshold::EXP_TABLE` is this output pasted as a `const` array and read with
integer linear interpolation. Run with `--check` to compare the committed Rust table with a fresh computation.

usage: exp_table_p13.py [--rust] [--check rust/crates/sim-core/src/fission_threshold.rs]
"""
from __future__ import annotations

import argparse
import math
import re
import sys
from pathlib import Path

N = 1024


def table() -> list[int]:
    out = [round(-math.log(1.0 - i / N) * 1000.0) for i in range(N)]
    out.append(round(math.log(2.0 * N) * 1000.0))
    return out


def rust(values: list[int]) -> str:
    rows = [", ".join(str(v) for v in values[i:i + 16]) for i in range(0, len(values), 16)]
    return "pub const EXP_TABLE: [u32; 1025] = [\n" + "".join(f"    {r},\n" for r in rows) + "];\n"


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--rust", action="store_true", help="print the Rust const array")
    ap.add_argument("--check", type=Path, help="compare with the table in this Rust file")
    args = ap.parse_args(argv)
    values = table()
    if args.check:
        text = args.check.read_text()
        body = text[text.index("pub const EXP_TABLE"):]
        body = body[body.index("[", body.index("=")) + 1:body.index("];")]
        committed = [int(x) for x in re.findall(r"\d+", body)]
        ok = committed == values
        print("EXP_TABLE matches" if ok else "EXP_TABLE differs")
        return 0 if ok else 1
    if args.rust:
        sys.stdout.write(rust(values))
    else:
        print(f"entries {len(values)} first {values[:4]} last {values[-3:]} mean {sum(values[:-1]) / N:.2f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
