"""Analyze a completed v19 Part 0 probe; never run or modify the simulation.

Usage: python3 tools/art_pipeline/measure_days_p8.py PROBE_DIRECTORY
Uses only the Python standard library.
"""
from __future__ import annotations

from dataclasses import asdict
import hashlib
import json
import sys
from pathlib import Path

from probe_models_p8 import CastDay, Clock, Death, Run, Sample, SeedSelection, Summary


def samples(path: Path) -> tuple[Sample, ...]:
    with path.open(encoding="utf-8") as stream:
        return tuple(Sample.model_validate_json(line) for line in stream if line.strip())


def calendar(rows: tuple[Sample, ...]) -> Clock:
    first = rows[0]
    # Infer the day length from repeated 08:00 rows, not the previous art pass.
    starts = sorted({row.tick for row in rows if row.minute_of_day == first.minute_of_day})
    clock = Clock(ticks_per_day=starts[1] - starts[0], offset=first.minute_of_day)
    assert first.tick == 0 and clock.ticks_per_day == 1440, "Unsupported clock unit"
    return clock


def select(run: Run, rows: tuple[Sample, ...]) -> SeedSelection:
    clock = calendar(rows)
    fissions = sorted(run.pass11.fission_log, key=lambda row: row.depart_tick)
    departures = tuple(clock.day(row.depart_tick) for row in fissions)
    three_alive = next((row.tick for row in rows if row.settlements_alive >= 3), None)
    founded_before_sample = [f.founded_tick for f in fissions
                             if f.founded_tick is not None and three_alive is not None
                             and f.founded_tick < three_alive]
    three_event_tick = max(founded_before_sample) + 1 if founded_before_sample else None
    day120 = next(row.settlements_alive for row in rows
                  if clock.day(row.tick) == 120 and row.minute_of_day == 480)
    founded = [row for row in fissions if row.founded_tick is not None]
    qualifies = (len(founded) >= 2 and 20 <= departures[0] <= 60
                 and fissions[1].depart_tick > fissions[0].depart_tick
                 and three_alive is not None and day120 >= 2)
    pressure = {(row[0], row[1]): row for row in run.pass13.pressure_checks}
    candidates = []
    first_tick = fissions[0].depart_tick if fissions else 0
    for check in run.pass12.fission_checks:
        tick, sid, members, _, food, _, _, scarcity, hazard, _, split, _ = check
        p = pressure.get((tick, sid))
        # A logged check passed the live age/member/cooldown/season gates.
        if sid == 1 and tick < first_tick and scarcity > 0 and hazard > 0 and split == 0 and p:
            candidates.append(CastDay(day=clock.day(tick), tick=tick, members=members,
                                      food=food, scarcity_line=int(members * 2.4 * 4),
                                      pressure=p[2], threshold=p[3], hazard=hazard))
    return SeedSelection(seed=run.seed, qualifies=qualifies, departure_days=departures,
                         first_three_alive_tick=three_event_tick,
                         first_three_alive_sample_tick=three_alive,
                         alive_at_display_day_120=day120,
                         alive_at_probe_day_120=run.settlements_alive_daily[-1],
                         cast_days=tuple(candidates[-3:]))


def main(source: Path) -> None:
    summary = Summary.model_validate_json((source / "summary.json").read_text())
    assert summary.scene == "game_v19", "Only a fresh merged v19 probe is accepted"
    output = Path("docs/art/production8")
    destination = output / "measurements"
    destination.mkdir(parents=True, exist_ok=True)
    tables = {}
    choices = []
    provenance = [{"path": str(source / "summary.json"),
                   "sha256": hashlib.sha256((source / "summary.json").read_bytes()).hexdigest()}]
    for run in sorted(summary.runs, key=lambda item: item.seed):
        path = source / f"game_v19_s{run.seed}.jsonl"
        rows = samples(path)
        assert all(row.scene == "game_v19" and row.seed == run.seed for row in rows)
        choices.append(select(run, rows))
        tables[run.seed] = rows
        provenance.append({"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                           "git_sha": rows[0].git_sha, "wall_seconds": run.wall_seconds})
    assert set(range(20261010, 20261043)).issubset(tables), "Complete live and 32-seed scan required"
    eligible = [choice for choice in choices if choice.seed != 20261010 and choice.qualifies]
    selected = eligible[0] if eligible else None
    live = tables[20261010]
    clock = calendar(live)
    seasons = live[0].days_per_season
    winter_days = {3 * seasons + 2 + year * 4 * seasons + night
                   for year in range(3) for night in range(seasons)}
    cold = [row for row in live if clock.day(row.tick) in winter_days and row.minute_of_day == 120]
    autumn = [row for row in live if row.minute_of_day == 900 and
              (clock.day(row.tick) - 1) % (4 * seasons) // seasons == 2 and clock.day(row.tick) <= 3 * 4 * seasons]
    # Cumulative cold-member-hours cannot decrease: equality proves no cold
    # hourly update in the entire preceding 12 hours, beyond sparse samples.
    by_tick = {row.tick: row for row in live}
    morning_ticks = {(day - 1) * clock.ticks_per_day + 435 - clock.offset
                     for day in range(2, 61)}
    required_warm_ticks = morning_ticks | {tick - 720 for tick in morning_ticks}
    assert required_warm_ticks.issubset(by_tick), "Missing 07:15/19:15 samples for warm-control evidence"
    warm = []
    for row in live:
        prior = by_tick.get(row.tick - 720)
        night = by_tick.get(row.tick - 315)
        if row.minute_of_day == 435 and clock.day(row.tick) <= 60 and prior and night:
            if prior.cold_member_hours_total == row.cold_member_hours_total and night.fire_lit > 0 and row.cold_members == 0:
                warm.append(row)
    inspected_seeds = [20261010] + ([selected.seed] if selected else [])
    deaths = []
    histories = []
    for seed in inspected_seeds:
        run = next(run for run in summary.runs if run.seed == seed)
        histories.append({"seed": seed, "fissions": [asdict(f) for f in run.pass11.fission_log]})
        death_path = source / f"game_v19_s{seed}_deaths.jsonl"
        with death_path.open(encoding="utf-8") as stream:
            for line in stream:
                row = Death.model_validate_json(line)
                if row.cause == "exposure":
                    deaths.append({"seed": seed, **asdict(row), "internal_tick": row.tick - 1,
                                   "display_time": clock.label(row.tick - 1)})
        provenance.append({"path": str(death_path), "sha256": hashlib.sha256(death_path.read_bytes()).hexdigest()})
    report = {"scene": summary.scene, "clock": asdict(clock), "provenance": provenance,
              "society_seed": selected.seed if selected else None,
              "fallbacks": [choice.seed for choice in eligible[1:3]],
              "selection": [asdict(choice) for choice in choices],
              "live_winter_0200": [asdict(row) for row in cold],
              "live_autumn_1500": [asdict(row) for row in autumn],
              "warm_control_0715": [asdict(row) for row in warm],
              "exposure_deaths": deaths, "fission_histories": histories}
    (destination / "probe_calendar_p8.json").write_text(json.dumps(report, indent=2) + "\n")
    text = ["# Art Pass 8 — v19 측정 날짜", "", f"원자료: `{source}`. 신규 game_v19 33시드 실측. 이전 날짜 재사용 없음.",
            f"시계: {asdict(clock)}. probe N일 체크포인트 = 화면 N+1일 08:00. 사건 내부 tick 다음 tick부터 관측 가능.",
            "", "## 사회 시드 스캔", "",
            "첫 출발 화면 d20–60, 이후 두 번째 출발, 동시 생존 3마을, 화면 d120 생존 ≥2 조건을 모두 적용했다.",
            f"선택: **{selected.seed if selected else '없음'}**; 다음 두 fallback: {[choice.seed for choice in eligible[1:3]]}.",
            "", "| seed | 출발 화면일 | 3마을 창립 직후 후보 tick | d120 생존 | 적격 |", "|---|---|---:|---:|---|"]
    text.extend(f"| {c.seed} | {c.departure_days} | {c.first_three_alive_tick} | {c.alive_at_display_day_120} | {c.qualifies} |" for c in choices)
    text += ["", "3마을 tick은 ≥3 생존 샘플 직전 창립 tick+1이며, 캡처 getter로 당일 생존수를 확정해야 한다. 실제 최초 ≥3 샘플 tick도 JSON에 별도 보존한다."]
    text += ["", "## 기본 시드 첫 세 겨울 02:00", "", "| 화면 | tick | cold | fire | severity |", "|---|---:|---:|---:|---:|"]
    text.extend(f"| {clock.label(r.tick)} | {r.tick} | {r.cold_members} | {r.fire_lit} | {r.winter_severity} |" for r in cold)
    text += ["", "## 기본 시드 가을 15:00", "", "| 화면 | tick | wood gatherers | wood in hand | 모든 운반 |", "|---|---:|---:|---:|---:|"]
    text.extend(f"| {clock.label(r.tick)} | {r.tick} | {r.wood_gatherers_now} | {r.wood_in_hand} | {r.hauling_now} |" for r in autumn)
    text += ["", "운반 수는 모든 자원 운반이다. 목재 운반 ID는 캡처 getter 관측으로 확정한다.",
             "", "## 120일의 모든 exposure 사망", "", "```json", json.dumps(deaths, ensure_ascii=False, indent=2), "```",
             "", "## 기본·사회 시드의 모든 분열", "", "```json", json.dumps(histories, indent=2), "```",
             "", "## 따뜻한 아침 대조", "",
             f"후보: {[clock.label(r.tick) for r in warm] or '첫 60일 내 없음'}.",
             "07:15 전 12시간의 누적 cold_member_hours 증가 0, 아침 cold 0, 당일 02:00 점화로 확인했다.",
             "", "## A33 개입일 계획", ""]
    if selected:
        text += ["```json", json.dumps([asdict(c) for c in selected.cast_days], indent=2), "```"]
    text += ["위 food/line/P는 자연 세계의 06:00 적격 검사 실측이다. 각 paired-world의 실제 cast 직전 getter 값도 별도로 기록해야 한다.",
             "scarcity line = floor(members × 2.4 × 4); 2.4는 food_economy.rs adult_need_per_day(1440), 4는 rules.rs V19_HAZARD.scarcity_days.",
             "", "원자료 SHA256, 모든 시드의 선택 판단과 수치는 `measurements/probe_calendar_p8.json`에 보존한다.",
             "이 측정은 시뮬레이션 날짜 선택 근거이며 시각·성능 합격 판정이 아니다."]
    (output / "MEASURED_DAYS.md").write_text("\n".join(text) + "\n")
    print(f"v19 measurement complete: society={report['society_seed']}, fallback={report['fallbacks']}")


if __name__ == "__main__":
    main(Path(sys.argv[1]))
