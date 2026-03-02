#!/usr/bin/env python3
"""Check Rust shadow cutover readiness from runtime shadow report JSON.

Usage:
  python3 tools/rust_shadow_cutover_check.py --report /abs/path/latest.json
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--report",
        required=True,
        help="Path to rust shadow report JSON (e.g. user://reports/rust_shadow/latest.json resolved path)",
    )
    args = parser.parse_args()

    report_path = Path(args.report).expanduser().resolve()
    if not report_path.exists():
        print(f"[shadow-cutover] report not found: {report_path}")
        return 2

    try:
        payload = json.loads(report_path.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001
        print(f"[shadow-cutover] failed to parse report: {exc}")
        return 2

    approved = bool(payload.get("approved_for_cutover", False))
    frames = int(payload.get("frames", 0))
    mismatch_frames = int(payload.get("mismatch_frames", 0))
    mismatch_ratio = float(payload.get("mismatch_ratio", 1.0))
    max_tick_delta = int(payload.get("max_tick_delta", -1))
    max_event_delta = int(payload.get("max_event_delta", -1))
    allowed_max_tick_delta = int(payload.get("allowed_max_tick_delta", 0))
    allowed_max_event_delta = int(payload.get("allowed_max_event_delta", 0))
    allowed_mismatch_ratio = float(payload.get("allowed_mismatch_ratio", 0.0))

    print(f"[shadow-cutover] approved_for_cutover={approved}")
    print(f"[shadow-cutover] frames={frames} mismatch_frames={mismatch_frames} mismatch_ratio={mismatch_ratio:.6f}")
    print(
        "[shadow-cutover] max_tick_delta=%d (allowed=%d) max_event_delta=%d (allowed=%d) allowed_mismatch_ratio=%.6f"
        % (
            max_tick_delta,
            allowed_max_tick_delta,
            max_event_delta,
            allowed_max_event_delta,
            allowed_mismatch_ratio,
        )
    )

    return 0 if approved else 1


if __name__ == "__main__":
    raise SystemExit(main())
