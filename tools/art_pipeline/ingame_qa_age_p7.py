"""A16 measurements from lossless gameplay captures and their observed sidecars."""
from __future__ import annotations

import json
from pathlib import Path
import re
from typing import Any

import numpy as np

from ingame_qa_pixels import Check, result
from ingame_qa_types import Capture, Entity, load_capture, placed

SEASONS = (("봄", "Spring"), ("여름", "Summer"), ("가을", "Autumn"), ("겨울", "Winter"))
DEATH_KEYS = ("deaths_starvation", "deaths_dehydration", "deaths_combat", "deaths_old_age", "deaths_exposure")


def _exclude(mask: np.ndarray, rect: tuple[float, ...], scale: float, margin: int = 0) -> None:
    x, y, w, h = (round(v / scale) for v in rect)
    mask[max(0, y-margin):min(mask.shape[0], y+h+margin), max(0, x-margin):min(mask.shape[1], x+w+margin)] = False


def _hud(capture: Capture, data: dict[str, Any], day: int) -> dict[str, Any]:
    clock = data.get("time_snapshot", {})
    season = int(clock.get("season", -1))
    season_day = int(clock.get("day_of_season", -1))
    labels = [label.text for label in capture.labels if "HudTopbar" in label.name]
    day_pattern = rf"(?:{day}일차|Day\s+{day}\b)"
    season_patterns = [rf"{re.escape(name)}\s+{season_day}(?:일째|\b)" for name in SEASONS[season]] if 0 <= season < 4 else []
    season_label = any(re.search(day_pattern, text) and any(re.search(pattern, text) for pattern in season_patterns) for text in labels)
    tallies = data.get("tallies", {})
    total = sum(int(tallies.get(key, 0)) for key in DEATH_KEYS)
    deaths = any(re.fullmatch(rf"(?:사망|Deaths?)\s+{total}", text, re.IGNORECASE) for text in labels)
    period = int(clock.get("days_per_season", 0))
    consistent = period > 0 and int(clock.get("day_index", -1)) == day-1 and season == ((day-1)//period)%4 and season_day == (day-1)%period+1
    return {"day": day, "season": season, "season_day": season_day, "labels": labels, "season_label_matches_snapshot": season_label, "runtime_clock_consistent": consistent, "death_total": total, "death_label_matches": deaths and all(key in tallies for key in DEATH_KEYS), "tallies": tallies}


def _terrain(summer: Capture, winter: Capture, summer_data: dict[str, Any], winter_data: dict[str, Any]) -> dict[str, Any]:
    aligned = all(key in summer_data and summer_data[key] == winter_data.get(key) for key in ("camera_position", "zoom", "viewport_size", "framebuffer_scale"))
    a, b = summer.image(), winter.image()
    mask = np.ones(a.shape[:2], dtype=bool)
    for capture in (summer, winter):
        for entity in capture.entities:
            if entity.visible and entity.kind != "night_chill_tint":
                _exclude(mask, entity.cue_rect or entity.rect, capture.scale, 8)
        for label in capture.labels:
            _exclude(mask, label.rect, capture.scale, 4)
    # The central viewport avoids map-edge/background pixels and untagged screen chrome.
    bounds = np.zeros_like(mask)
    bounds[180:800, 400:1520] = True
    mask &= bounds & (a.min(axis=2) > 8) & (b.min(axis=2) > 8)
    count = int(mask.sum())
    ratios = [float(image[..., 2][mask].mean()/image[..., 0][mask].mean()) for image in (a, b)] if count else [0.0, 0.0]
    gain = ratios[1]/ratios[0] if ratios[0] else 0.0
    return {"pass": aligned and count >= 1000 and gain >= 1.03, "camera_aligned": aligned, "camera_positions": [summer_data.get("camera_position"), winter_data.get("camera_position")], "mask_pixels": count, "formula": "mean(B)/mean(R); winter ratio / summer ratio >= 1.03", "mask": "common central [400:1520,180:800] viewport; visible world object/cue/panel rectangles expanded 8px; screen-space night_chill_tint is color modulation, not an occluder and label rectangles expanded 4px excluded; both RGB minima >8", "summer_blue_red": ratios[0], "winter_blue_red": ratios[1], "winter_over_summer": gain}


def _region_mask(entity: Entity, capture: Capture, repo: Path) -> np.ndarray:
    if min(entity.variant) < 0:
        return np.zeros((1080, 1920), dtype=bool)
    _, opaque = placed(entity, capture, repo)
    return opaque


def _order(entity: Entity) -> tuple[int, float, int]:
    return entity.draw_order[0], entity.world_anchor[1], entity.draw_order[1]


def _ramps(capture: Capture, repo: Path) -> dict[str, Any]:
    image = capture.image().astype(np.int16)
    rows = []
    for entity in capture.entities:
        if entity.kind != "agent" or not entity.visible:
            continue
        expected, opaque = placed(entity, capture, repo, variants=True)
        mask = _region_mask(entity, capture, repo) & opaque
        for other in capture.entities:
            if not other.visible or other is entity or other.alpha <= 0:
                continue
            if other.kind in ("panel", "stockpile_label", "stockpile_wood_label") or _order(other) > _order(entity):
                _exclude(mask, other.cue_rect or other.rect, capture.scale, 1)
        for label in capture.labels:
            _exclude(mask, label.rect, capture.scale, 1)
        count = int(mask.sum())
        agreement = float((np.max(np.abs(image-expected.astype(np.int16)), axis=2)[mask] <= 1).mean()) if count else None
        rows.append({"id": entity.ident, "pixels": count, "agreement": agreement})
    measurable = [row for row in rows if row["pixels"] > 0]
    return {"pass": bool(measurable) and all(row["agreement"] >= .99 for row in measurable), "formula": "all opaque natural-colour and hair texels: framebuffer RGB agrees with placed(variants=True) within +/-1 on every channel; agreement >= .99 per measurable actor", "occlusion": "foreground object/cue rectangles and UI rectangles excluded, including translucent foreground", "actors": rows, "unmeasurable_ids": [row["id"] for row in rows if not row["pixels"]]}


def _ages(capture: Capture, data: dict[str, Any]) -> dict[str, Any]:
    snap = data.get("agent_snapshot", {})
    stages = dict(zip(snap.get("agent_ids", []), snap.get("life_stages", [])))
    rows = [{"id": e.ident, "stage": stages.get(e.ident), "hair": e.variant[1]} for e in capture.entities if e.kind == "agent" and e.visible]
    elders = sum(row["stage"] == 2 for row in rows)
    old_valid = bool(rows) and elders > 0 and all(row["stage"] in (0, 1, 2) and (row["hair"] in (3, 4) if row["stage"] == 2 else row["hair"] in (0, 1, 2)) for row in rows)
    valid = bool(rows) and elders > 0 and all(row["stage"] in (0, 1, 2) and (row["hair"] in (5, 6) if row["stage"] == 2 else row["hair"] in (0, 1, 2, 3, 4)) for row in rows)
    return {"pass": valid, "P5_rule": "elders 5..6; others 0..4", "P4_old_rule": {"pass": old_valid, "definition": "elders 3..4; others 0..2 on same captured codes"}, "visible_elders": elders, "actors": rows, "missing_stage_ids": [row["id"] for row in rows if row["stage"] is None]}


def age_seasons(long_raw: Path, repo: Path, spring_day: int = 40) -> Check:
    days = tuple(dict.fromkeys((10, 18, 24, spring_day, 40)))
    stems = {day: f"l_d{day}_1200_F2" for day in days}
    missing = [stem for stem in stems.values() if not (long_raw/(stem+".png")).is_file() or not (long_raw/(stem+".entities.json")).is_file()]
    if missing:
        return result("A16", False, json.dumps({"missing_captures": missing}))
    captures = {day: load_capture(long_raw/(stem+".png")) for day, stem in stems.items()}
    data = {day: json.loads((long_raw/(stem+".entities.json")).read_text()) for day, stem in stems.items()}
    hud = [_hud(captures[day], data[day], day) for day in days]
    expected = {10: 1, 18: 2, 24: 3, spring_day: 0}
    brief = {"pass": all(row["season"] == expected[row["day"]] for row in hud if row["day"] in expected), "expected": expected, "actual": {row["day"]: row["season"] for row in hud}, "note": "Spring contract uses day %d; day40 independently measures elder hair." % spring_day}
    terrain = _terrain(captures[10], captures[24], data[10], data[24])
    ramps = _ramps(captures[24], repo)
    ages = _ages(captures[40], data[40])
    passed = all(row["season_label_matches_snapshot"] and row["runtime_clock_consistent"] and row["death_label_matches"] for row in hud) and all(component["pass"] for component in (brief, terrain, ramps, ages))
    return result("A16", passed, json.dumps({"hud": hud, "brief_season_contract": brief, "terrain": terrain, "winter_ramps": ramps, "day40_hair": ages}, ensure_ascii=False))
