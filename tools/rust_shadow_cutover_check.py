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
    parser.add_argument(
        "--required-min-frames",
        type=int,
        default=-1,
        help="Override required minimum shadow frames (default: use report value)",
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
    min_frames_report = int(payload.get("min_frames_for_cutover", 0))
    required_min_frames = (
        int(args.required_min_frames)
        if int(args.required_min_frames) >= 0
        else min_frames_report
    )
    mismatch_frames = int(payload.get("mismatch_frames", 0))
    mismatch_ratio = float(payload.get("mismatch_ratio", 1.0))
    max_tick_delta = int(payload.get("max_tick_delta", -1))
    max_work_delta = int(payload.get("max_work_delta", payload.get("max_event_delta", -1)))
    allowed_max_tick_delta = int(payload.get("allowed_max_tick_delta", 0))
    allowed_max_work_delta = int(
        payload.get("allowed_max_work_delta", payload.get("allowed_max_event_delta", 0))
    )
    allowed_mismatch_ratio = float(payload.get("allowed_mismatch_ratio", 0.0))
    frames_ready_for_cutover = bool(payload.get("frames_ready_for_cutover", False))

    frame_gate_ok = frames >= required_min_frames
    tick_gate_ok = max_tick_delta <= allowed_max_tick_delta
    work_gate_ok = max_work_delta <= allowed_max_work_delta
    mismatch_gate_ok = mismatch_ratio <= allowed_mismatch_ratio
    gates_ok = frame_gate_ok and tick_gate_ok and work_gate_ok and mismatch_gate_ok

    print(f"[shadow-cutover] approved_for_cutover={approved}")
    print(
        "[shadow-cutover] frame_gate="
        f"{frame_gate_ok} frames={frames} required_min_frames={required_min_frames} "
        f"report_min_frames={min_frames_report} report_frames_ready={frames_ready_for_cutover}"
    )
    print(f"[shadow-cutover] frames={frames} mismatch_frames={mismatch_frames} mismatch_ratio={mismatch_ratio:.6f}")
    print(
        "[shadow-cutover] max_tick_delta=%d (allowed=%d) max_work_delta=%d (allowed=%d) allowed_mismatch_ratio=%.6f"
        % (
            max_tick_delta,
            allowed_max_tick_delta,
            max_work_delta,
            allowed_max_work_delta,
            allowed_mismatch_ratio,
        )
    )
    print(
        "[shadow-cutover] gates: tick=%s work=%s mismatch=%s all=%s"
        % (str(tick_gate_ok), str(work_gate_ok), str(mismatch_gate_ok), str(gates_ok))
    )
    remaining_frames = max(0, required_min_frames - frames)
    print(f"[shadow-cutover] remaining_frames_for_gate={remaining_frames}")

    return 0 if approved and gates_ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
