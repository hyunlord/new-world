#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 tools/v8/m6a_r1_stall.py --help
"""The dated M6a R1 stall exception, separate from raw strict acceptance."""
from __future__ import annotations

import argparse
from pathlib import Path
import subprocess
import sys
from typing import Literal, assert_never

from mux2_gate_inputs import EvidenceError, Json, require
from m6a_r1_stall_capture import (AUTHORITY, StallVerdict, capture,
                                   evaluate as evaluate, verify_capture)


def authorize(verdict: StallVerdict, other_steps_pass: bool) -> dict[str, Json]:
    """Only the final gate may establish the exception's all-other-steps condition."""
    require(other_steps_pass, 'other gate steps must all pass before R1 stall authorization')
    require(verdict.waiver_eligible, 'stall lies outside the R1 owner waiver')
    waivers: list[Json] = []
    if not verdict.strict_pass:
        waivers.append({'step': 41, 'authority': AUTHORITY, 'issue': '#515',
                        'all_other_steps_pass': True, 'evidence': verdict.record()})
    return {'strict_pass': verdict.strict_pass, 'waivers': waivers}


def verify(root: Path, receipt_path: Path) -> StallVerdict:
    """Public gate entry: authenticate all receipt bytes before recomputing the verdict."""
    return verify_capture(root, receipt_path)


class Arguments(argparse.Namespace):
    action: Literal['capture', 'verify'] = 'capture'
    root: Path = Path(__file__).resolve().parents[2]
    out: Path | None = None
    receipt: Path | None = None
    godot: Path = Path('/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot')


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    _ = parser.add_argument('action', choices=('capture', 'verify'))
    _ = parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[2])
    _ = parser.add_argument('--out', type=Path)
    _ = parser.add_argument('--receipt', type=Path)
    _ = parser.add_argument('--godot', type=Path,
                        default=Path('/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot'))
    args = parser.parse_args(namespace=Arguments())
    try:
        match args.action:
            case 'capture':
                if args.out is None:
                    raise EvidenceError('--out required')
                verdict = capture(args.root.resolve(), args.out, args.godot.resolve())
            case 'verify':
                if args.receipt is None:
                    raise EvidenceError('--receipt required')
                verdict = verify(args.root.resolve(), args.receipt)
            case _:
                assert_never(args.action)
        require(verdict.waiver_eligible, 'stall lies outside the R1 owner waiver')
        print(f'M6A_R1_STALL strict_pass={verdict.strict_pass} '
              + f'waiver_eligible={verdict.waiver_eligible} global_authorization=PENDING')
        return 0
    except (EvidenceError, OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f'M6A_R1_STALL_FAIL {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
