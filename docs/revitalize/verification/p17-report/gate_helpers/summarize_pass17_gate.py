#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 summarize_pass17_gate.py WRAPPER_RECEIPT NEW_JSON NEW_MARKDOWN
from __future__ import annotations

from datetime import datetime, timedelta, timezone
import hashlib
import json
import math
from pathlib import Path
import re
import sys
from typing import Final, TypeAlias

Json: TypeAlias = None | bool | int | float | str | list["Json"] | dict[str, "Json"]
Document: TypeAlias = dict[str, Json]
STEPS: Final = ("prepare_bridge", "prepare_import", "test_debug", "test_release", "clippy", "clippy_all_targets", "forbidden_math", "bridge_build", "godot_import", "strict_check", "smoke", "python_unittest", "python_worldmodel", "perf_quick", "allowed_paths")
QUICK_NOTICE: Final = "perf_quick은 비계측 유효성 확인이며 R5/F8 성능 판정 근거가 아니다."


class GateSummaryError(RuntimeError):
    pass


def require(condition: bool, reason: str) -> None:
    if not condition:
        raise GateSummaryError(reason)


def mapping(value: Json) -> Document:
    require(type(value) is dict, "expected JSON object")
    return value


def array(value: Json) -> list[Json]:
    require(type(value) is list, "expected JSON array")
    return value


def string(value: Json) -> str:
    require(type(value) is str and bool(value), "expected nonempty string")
    return value


def number(value: Json) -> float:
    require(type(value) in (int, float), "expected numeric timestamp")
    converted = float(value)
    require(math.isfinite(converted), "nonfinite number")
    return converted


def unique_object(pairs: list[tuple[str, Json]]) -> Document:
    value = dict(pairs)
    require(len(value) == len(pairs), "duplicate JSON key")
    return value


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_regular(path: Path) -> bytes:
    require(path.is_file() and not path.is_symlink(), f"expected regular file: {path}")
    return path.read_bytes()


def interval(receipt: Document, prefix: str) -> Document:
    start, end = (number(receipt[prefix + name + "_epoch"]) for name in ("started", "finished"))
    require(end >= start, "reversed timestamp interval")
    for name, epoch in (("started", start), ("finished", end)):
        stamp = string(receipt[prefix + name + "_utc"])
        require(stamp.endswith("Z"), "timestamp is not explicit UTC")
        require(abs(datetime.fromisoformat(stamp).timestamp() - epoch) <= 0.00001, "UTC/epoch timestamp mismatch")
    return {"started_utc": receipt[prefix + "started_utc"], "finished_utc": receipt[prefix + "finished_utc"], "started_epoch": start, "finished_epoch": end, "elapsed_seconds": end - start}


def one_line(lines: list[str], expression: str) -> re.Match[str]:
    matches = [match for line in lines if (match := re.fullmatch(expression, line))]
    require(len(matches) == 1, f"expected exactly one row matching {expression}")
    return matches[0]


def log_utc(stamp: str) -> datetime:
    local, zone = stamp.rsplit(" ", 1)
    offsets = {"UTC": 0, "GMT": 0, "KST": 9}
    require(zone in offsets, f"unsupported explicit gate timezone: {zone}")
    return datetime.strptime(local, "%Y-%m-%d %H:%M:%S").replace(tzinfo=timezone(timedelta(hours=offsets[zone]))).astimezone(timezone.utc)


def parse_gate(lines: list[str], expected: str) -> Document:
    require([line for line in lines if line.startswith("head ")] == ["head " + expected], "gate HEAD mismatch")
    rows = [match.groups() for line in lines if (match := re.fullmatch(r"(\w+) exit (-?\d+)", line))]
    require(all(re.fullmatch(r"\w+ exit -?\d+", line) for line in lines if " exit " in line), "malformed step exit row")
    require(rows == [(name, "0") for name in STEPS], "expected exactly15 ordered zero step exits")
    require([line for line in lines if line.startswith("smoke_errors ")] == ["smoke_errors none"], "invalid smoke marker")
    require(not any("wait_failed" in line for line in lines), "failed wait guard")
    started = one_line(lines, r"started (.+)")
    finished = one_line(lines, r"finished (.+) status 0")
    require([line for line in lines if line.startswith("finished ")] == [lines[-1]] and lines[-1] == finished[0], "invalid terminal finished marker")
    begin, end = log_utc(started[1]), log_utc(finished[1])
    require(end >= begin, "reversed gate log times")
    tests: Document = {}
    for name in ("test_debug", "test_release"):
        require(sum(line.startswith(name + " passed ") for line in lines) == 1, f"{name}: duplicate totals")
        match = one_line(lines, name + r" passed (\d+) failed (\d+) ignored (\d+)")
        passed, failed, ignored = (int(value) for value in match.groups())
        require(passed > 0 and failed == 0, f"{name}: invalid totals")
        tests[name] = {"passed": passed, "failed": failed, "ignored": ignored}
    for name in ("python_unittest", "python_worldmodel"):
        require(sum(line.startswith(name + " ran ") for line in lines) == 1, f"{name}: duplicate totals")
        match = one_line(lines, name + r" ran (\d+) skipped (\d+) result OK(?: \(skipped=(\d+)\))?")
        ran, skipped = int(match[1]), int(match[2])
        require(ran > 0 and 0 <= skipped <= ran and (int(match[3]) if match[3] else 0) == skipped, f"{name}: inconsistent Python skips")
        tests[name] = {"ran": ran, "skipped": skipped, "executed": ran - skipped, "result": "OK"}
    require(sum(line.startswith("quick ") for line in lines) == 1, "duplicate quick sample")
    quick = one_line(lines, r"quick mean_ms (\S+) p999_ms (\S+) max_ms (\S+)")
    quick_values = [number(float(value)) for value in quick.groups()]
    require(all(value >= 0 for value in quick_values), "invalid quick sample")
    return {"head": expected, "steps": [{"name": name, "exit_code": 0} for name in STEPS], "tests": tests, "smoke_errors": "none", "log_interval": {"started_utc": begin.isoformat().replace("+00:00", "Z"), "finished_utc": end.isoformat().replace("+00:00", "Z"), "elapsed_seconds": (end - begin).total_seconds(), "precision": "whole seconds from gate.txt"}, "quick": {"mean_ms": quick_values[0], "p999_ms": quick_values[1], "max_ms": quick_values[2], "is_timing_evidence": False, "notice": QUICK_NOTICE}}


def summarize(path: Path) -> Document:
    data = read_regular(path)
    receipt = mapping(json.loads(data, object_pairs_hook=unique_object))
    require(type(receipt.get("schema_version")) is int and receipt.get("schema_version") == 1 and receipt.get("status") == "PASS" and receipt.get("passed") is True, "receipt is incomplete or not terminal PASS")
    require(receipt["errors"] == [] and type(receipt["gate_exit_code"]) is int and receipt["gate_exit_code"] == 0 and receipt["wrapper_signal"] is None and receipt["gate_launched"] is True, "invalid receipt completion")
    require(receipt["perf_quick_is_timing_evidence"] is False and receipt["expected_steps"] == list(STEPS), "receipt gate contract mismatch")
    expected = string(receipt["expected_head"])
    require(re.fullmatch(r"[0-9a-f]{40}", expected) is not None and Path(string(receipt["receipt"])).resolve() == path, "invalid receipt head/path")
    for key in ("before", "after"):
        snapshot = mapping(receipt[key])
        require(snapshot["head"] == expected and all(snapshot[name] == [] for name in ("staged", "unstaged", "untracked", "ignored_source")), "source snapshot is dirty or bound to another HEAD")
    require(receipt["gate_script_before"] == receipt["gate_script_after"], "gate script changed")
    script = mapping(receipt["gate_script_before"])
    require(re.fullmatch(r"[0-9a-f]{64}", string(script["sha256"])) is not None and type(script["bytes"]) is int and script["bytes"] > 0, "invalid gate script provenance")
    output = Path(string(receipt["output_directory"])).resolve()
    require(receipt["command"] == ["bash", string(script["path"]), str(output)], "gate command differs from script/output provenance")
    wrapper, invocation = interval(receipt, ""), interval(receipt, "gate_")
    require(wrapper["started_epoch"] <= invocation["started_epoch"] <= invocation["finished_epoch"] <= wrapper["finished_epoch"], "gate invocation lies outside wrapper interval")
    require(abs(number(receipt["elapsed_seconds"]) - wrapper["elapsed_seconds"]) <= 0.00001, "wrapper elapsed mismatch")
    artifacts: dict[str, str] = {}
    for raw in array(receipt["artifacts"]):
        item = mapping(raw); artifact = Path(string(item["path"]))
        require(artifact.is_absolute() and artifact.is_relative_to(output) and str(artifact) not in artifacts, "duplicate/outside artifact path")
        content = read_regular(artifact); digest = sha(content)
        require(type(item["bytes"]) is int and len(content) == item["bytes"] and digest == item["sha256"], f"artifact hash/size mismatch: {artifact}")
        artifacts[str(artifact)] = digest
    actual = {str(item) for item in output.rglob("*") if item.is_file() or item.is_symlink()}
    require(actual == set(artifacts), "artifact inventory changed")
    required = {str(output / (name + ".log")) for name in STEPS} | {str(output / "gate.txt"), str(output / "smoke_errors.log")}
    require(required <= set(artifacts) and read_regular(output / "smoke_errors.log") == b"", "missing logs or nonempty smoke errors")
    manifest_path = Path(string(receipt["sha_manifest"])); manifest = read_regular(manifest_path)
    recorded: dict[str, str] = {}
    for line in manifest.decode().splitlines():
        match = re.fullmatch(r"([0-9a-f]{64})  (.+)", line)
        require(match is not None and match[2] not in recorded, "malformed/duplicate SHA manifest entry")
        recorded[match[2]] = match[1]
    require(recorded == artifacts | {str(path): sha(data)}, "SHA manifest differs from finalized receipt/artifacts")
    gate = read_regular(output / "gate.txt")
    parsed = parse_gate(gate.decode().splitlines(), expected)
    log = mapping(parsed["log_interval"])
    require(invocation["started_epoch"] - 1 <= datetime.fromisoformat(log["started_utc"]).timestamp() <= datetime.fromisoformat(log["finished_utc"]).timestamp() <= invocation["finished_epoch"], "gate log times lie outside invocation")
    require(read_regular(path) == data and read_regular(manifest_path) == manifest and sha(read_regular(output / "gate.txt")) == artifacts[str(output / "gate.txt")], "evidence changed during read")
    return {"schema_version": 1, "status": "PASS", **parsed, "wrapper_interval": wrapper, "invocation_interval": invocation, "verified_artifact_count": len(artifacts), "gate_script": receipt["gate_script_before"], "provenance": {"receipt_path": str(path), "receipt_sha256": sha(data), "manifest_path": str(manifest_path), "manifest_sha256": sha(manifest), "gate_path": str(output / "gate.txt"), "gate_sha256": sha(gate), "repo": receipt["repo"], "output_directory": str(output), "artifact_sha256": artifacts}}


def markdown(summary: Document, json_output: Path) -> str:
    invocation, wrapper, log = (mapping(summary[key]) for key in ("invocation_interval", "wrapper_interval", "log_interval"))
    lines = ["## 게이트 결과", "", f'**PASS — 15/15 단계 종료 코드 0**, smoke 오류 없음. 검증 HEAD: `{summary["head"]}`.', "", f'- 게이트 호출 UTC: {invocation["started_utc"]} → {invocation["finished_utc"]} ({invocation["elapsed_seconds"]:.6f}초).', f'- 래퍼 전체 UTC: {wrapper["started_utc"]} → {wrapper["finished_utc"]} ({wrapper["elapsed_seconds"]:.6f}초).', f'- gate.txt 구간 UTC: {log["started_utc"]} → {log["finished_utc"]} ({log["elapsed_seconds"]:.0f}초, 원본 초 단위).', "", "| 검증 | 통과 / 실행 | 실패 | 무시 / 건너뜀 |", "|---|---:|---:|---:|"]
    tests = mapping(summary["tests"])
    for name in ("test_debug", "test_release"):
        row = mapping(tests[name]); lines.append(f'| {name} | {row["passed"]} | {row["failed"]} | {row["ignored"]} |')
    for name in ("python_unittest", "python_worldmodel"):
        row = mapping(tests[name]); lines.append(f'| {name} | {row["ran"]}건 중 {row["executed"]} 실행 | 0 | {row["skipped"]} |')
    provenance = mapping(summary["provenance"])
    lines += ["", QUICK_NOTICE, "", f'영수증·SHA 목록과 {summary["verified_artifact_count"]}개 산출물 해시를 확인했다. [구조화 결과]({json_output}), JSON SHA256 `{sha(json_output.read_bytes())}`.', f'원본 영수증: `{provenance["receipt_path"]}`, SHA256 `{provenance["receipt_sha256"]}`.']
    return "\n".join(lines) + "\n"


def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print("usage: summarize_pass17_gate.py WRAPPER_RECEIPT NEW_JSON NEW_MARKDOWN", file=sys.stderr)
        return 2
    receipt, json_output, md_output = (Path(value).absolute() for value in argv)
    try:
        require(not receipt.is_symlink(), "symlink receipt is not accepted")
        summary = summarize(receipt.resolve())
        provenance = mapping(summary["provenance"])
        outputs = (json_output, md_output)
        require(len({p.resolve() for p in outputs}) == 2 and not any(p.exists() or p.is_symlink() for p in outputs), "output collision; preserve existing files")
        require(all(not p.resolve().is_relative_to(Path(string(provenance[root])).resolve()) for p in outputs for root in ("repo", "output_directory")), "summary outputs must remain outside repository/gate artifacts")
        summary["formatter"] = {"path": str(Path(__file__).resolve()), "sha256": sha(Path(__file__).read_bytes())}
        with json_output.open("x") as output:
            json.dump(summary, output, ensure_ascii=False, indent=2, allow_nan=False); output.write("\n")
        with md_output.open("x") as output:
            output.write(markdown(summary, json_output))
        print(f"PASS: {json_output}; {md_output}")
        return 0
    except (GateSummaryError, OSError, ValueError, KeyError, TypeError, OverflowError) as error:
        print(f"REFUSED: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
