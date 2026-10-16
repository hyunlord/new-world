#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 tools/art_pipeline/summarize_ablation_p9.py SESSION OUTPUT_PREFIX [BEFORE_SESSION]
"""Validate a W500 ablation session and rank recomputed neighbour-minus-off costs."""
from __future__ import annotations

import json
import math
import sys
from pathlib import Path
from typing import TypeAlias

import summarize_perf_p9 as perf

Record: TypeAlias = dict[str, perf.Json]


def load(path: Path) -> Record:
    raw: perf.Json = json.loads(path.read_text())
    data = dict(perf.mapping(raw))
    if data.get("world") != "W500" or data.get("build_profile") != "debug":
        raise perf.EvidenceError("Ablation ranking requires W500 debug evidence")
    if data.get("valid") is not True or data.get("errors") != []:
        raise perf.EvidenceError("Ablation evidence must be valid with no errors")
    if data.get("ablation_only") is True:
        if data.get("phase") != "after":
            raise perf.EvidenceError("Ablation-only mode is defined only for after W500")
        blocks = [dict(perf.mapping(block)) for block in perf.sequence(data["blocks"])]
    else:
        perf.load_session(path)
        blocks = [dict(perf.mapping(block)) for block in perf.sequence(data["blocks"]) if str(perf.mapping(block)["condition"]).startswith("ablation")]
    validate(data, blocks)
    by_label = {str(block["label"]): block for block in blocks}
    ranked: list[perf.Json] = []
    for raw_row in perf.sequence(data["ablation"]):
        row = dict(perf.mapping(raw_row))
        off = perf.sequence(by_label[str(row["off_block"])]["samples"])
        neighbours = [sample for name in perf.sequence(row["neighbour_blocks"]) for sample in perf.sequence(by_label[str(name)]["samples"])]
        off_stats, on_stats = perf.summary(off, "primary_cpu_ms"), perf.summary(neighbours, "primary_cpu_ms")
        row["reported_cost_ms"] = row.get("cost_ms")
        row["cost_ms"] = perf.number(on_stats["median"]) - perf.number(off_stats["median"])
        row["off_statistics"] = dict(off_stats)
        row["neighbour_statistics"] = dict(on_stats)
        ranked.append(row)
    ranked.sort(key=lambda row: (-perf.number(perf.mapping(row)["cost_ms"]), str(perf.mapping(row)["node"])))
    return {"source": str(path), "world": "W500", "phase": data["phase"], "build_profile": "debug", "ablation_only": data.get("ablation_only", False), "actual_alive": perf.mapping(data["initial"]).get("population"), "ranked": ranked, "top_three": [perf.mapping(row)["node"] for row in ranked[:3]], "floor": dict(perf.summary(perf.sequence(by_label["ablation_floor"]["samples"]), "primary_cpu_ms"))}


def validate(data: Record, blocks: list[Record]) -> None:
    restoration = perf.mapping(data.get("restoration", {}))
    if any(restoration.get(key) is not True for key in ("vsync_restored", "layers_restored", "camera_process_restored", "probes_removed")) or data.get("ablation_visibility_restored") is not True:
        raise perf.EvidenceError("Ablation restoration unproven")
    rows = [perf.mapping(row) for row in perf.sequence(data.get("ablation", []))]
    nodes = [str(row["node"]) for row in rows]
    layers = {str(row["node"]).split("/")[-1] for row in rows if row.get("layer_only") is True}
    if not {"WorldObjects", "Shadows", "NightChillLayer"}.issubset(layers) or not any(row.get("layer_only") is False for row in rows):
        raise perf.EvidenceError("Mandatory node/shared-layer ablation missing")
    if len(nodes) != len(set(nodes)) or any(node in {"WorldSim", "Camera2D"} for node in nodes):
        raise perf.EvidenceError("Duplicate or prohibited ablation node")
    expected = ["ablation_all_on_0"]
    previous = expected[0]
    for row in rows:
        if row.get("layer_only") is not True and row.get("layer_only") is not False:
            raise perf.EvidenceError("Ablation row must identify node versus shared-layer mode")
        if any(not isinstance(child, str) for child in perf.sequence(row.get("descendants_disabled_as_group"))):
            raise perf.EvidenceError("Ablation group descendants must be explicit node paths")
        node = str(row["node"])
        off, after = "off_" + node, "on_after_" + node
        if row["off_block"] != off or row["neighbour_blocks"] != [previous, after]:
            raise perf.EvidenceError("Ablation rows must reference their adjacent all-on neighbours")
        expected.extend([off, after])
        previous = after
    expected.extend(["ablation_floor", "ablation_restored"])
    if [str(block["label"]) for block in blocks] != expected:
        raise perf.EvidenceError("Missing, duplicate or reordered ablation blocks/floor")
    initial = perf.mapping(data["initial"])
    tick = perf.number(initial["tick"])
    if tick != 1440:
        raise perf.EvidenceError("W500 ablation must be frozen at tick 1440")
    previous_frame = -1.0
    render_positive = False
    for block in blocks:
        label = str(block["label"])
        expected_condition = "ablation_off" if label.startswith("off_") else ("ablation_floor" if label == "ablation_floor" else "ablation_reference")
        if block["condition"] != expected_condition or (block["zoom"], block["speed"], block["max_fps"]) != (4, 0, 0):
            raise perf.EvidenceError("Ablation controls do not match frozen Z4")
        if block.get("vsync_disabled") is not True or block.get("camera_process_frozen") is not True or block.get("window_size") != [1920, 1080]:
            raise perf.EvidenceError("Ablation window/camera/vsync unproven")
        counted, discarded = perf.sequence(block["samples"]), perf.sequence(block["discarded"])
        if len(counted) != 150 or len(discarded) != 30:
            raise perf.EvidenceError("Ablation blocks require 150 counted and 30 discarded frames")
        for raw_sample in [*discarded, *counted]:
            sample = perf.mapping(raw_sample)
            if any(perf.number(sample[key]) < 0 for key in perf.METRICS):
                raise perf.EvidenceError("Invalid negative ablation measurement")
            if not math.isclose(perf.number(sample["primary_cpu_ms"]), perf.number(sample["process_ms"]) + perf.number(sample["render_cpu_ms"]), rel_tol=0, abs_tol=1e-9):
                raise perf.EvidenceError("Ablation primary does not equal process + render")
            render_positive |= perf.number(sample["render_cpu_ms"]) > 0
            frame = perf.number(sample["process_frame"])
            if sample.get("paired") is not True or frame <= previous_frame:
                raise perf.EvidenceError("Ablation frame stamps unpaired or non-increasing")
            previous_frame = frame
            state = perf.mapping(sample["state"])
            if sample["ticks_run"] != 0 or state["tick"] != tick or state["camera"] != initial["camera"] or state["zoom"] != [4, 4]:
                raise perf.EvidenceError("Ablation tick/camera/zoom changed")
            if data["phase"] == "after" or "ffi_calls" in sample:
                counts = perf.mapping(sample.get("ffi_calls"))
                if sample.get("ffi_counter_frame") != frame or any(perf.number(value) < 0 or not perf.number(value).is_integer() for value in counts.values()):
                    raise perf.EvidenceError("Ablation FFI counters invalid or misaligned")
            if data["phase"] == "after" or "direct_read_context" in sample:
                context = perf.mapping(sample.get("direct_read_context"))
                if context.get("inspector_visible") is not False or context.get("influence_channel") != -1:
                    raise perf.EvidenceError("Ablation inspector/overlay context is not closed")
    if not render_positive:
        raise perf.EvidenceError("Ablation render CPU unavailable")


def comparison(current: Record, before: Record) -> list[perf.Json]:
    a = {str(perf.mapping(row)["node"]): perf.mapping(row) for row in perf.sequence(before["ranked"])}
    b = {str(perf.mapping(row)["node"]): perf.mapping(row) for row in perf.sequence(current["ranked"])}
    output: list[perf.Json] = []
    for node in sorted(a.keys() | b.keys()):
        old, new = a.get(node), b.get(node)
        same_group = old is not None and new is not None and old.get("descendants_disabled_as_group") == new.get("descendants_disabled_as_group") and old["layer_only"] == new["layer_only"]
        output.append({"node": node, "before_cost_ms": old["cost_ms"] if old else None, "after_cost_ms": new["cost_ms"] if new else None, "status": "COMPARABLE" if same_group else "NOT_COMPARABLE", "saved_ms": perf.number(old["cost_ms"]) - perf.number(new["cost_ms"]) if same_group and old is not None and new is not None else None})
    return output


def markdown(result: Record) -> str:
    lines = ["# W500 ablation ranking", "", f"Source: `{result['source']}`; actual alive: {result['actual_alive']}; debug, frozen tick 1440, Z4.", "", "Cost = median of 300 samples from neighbouring all-on blocks minus median of 150 node-off samples. Negative costs are retained.", "", "| Rank | Node/group | Cost ms | Off median | Neighbour median | Layer only |", "|---:|---|---:|---:|---:|---|"]
    for rank, raw in enumerate(perf.sequence(result["ranked"]), 1):
        row = perf.mapping(raw)
        lines.append(f"| {rank} | {row['node']} | {row['cost_ms']} | {perf.mapping(row['off_statistics'])['median']} | {perf.mapping(row['neighbour_statistics'])['median']} | {row['layer_only']} |")
    lines.extend(["", "Top three measured rows: " + ", ".join(map(str, perf.sequence(result["top_three"]))), "", "Floor: " + json.dumps(result["floor"]), ""])
    if "comparison" in result:
        lines.extend(["| Node/group | Before cost ms | After cost ms | Saved ms | Status |", "|---|---:|---:|---:|---|"])
        for raw in perf.sequence(result["comparison"]):
            row = perf.mapping(raw)
            lines.append(f"| {row['node']} | {row['before_cost_ms']} | {row['after_cost_ms']} | {row['saved_ms']} | {row['status']} |")
    return "\n".join(lines) + "\n"


def main() -> None:
    if len(sys.argv) not in (3, 4):
        raise perf.EvidenceError("Usage: summarize_ablation_p9.py SESSION OUTPUT_PREFIX [BEFORE_SESSION]")
    result = load(Path(sys.argv[1]))
    if len(sys.argv) == 4:
        before = load(Path(sys.argv[3]))
        result["comparison"] = comparison(result, before)
        result["before_source"] = before["source"]
    prefix = Path(sys.argv[2])
    prefix.parent.mkdir(parents=True, exist_ok=True)
    prefix.with_suffix(".json").write_text(json.dumps(result, indent=2) + "\n")
    prefix.with_suffix(".md").write_text(markdown(result))


if __name__ == "__main__":
    main()
