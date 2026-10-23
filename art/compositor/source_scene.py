# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic>=2,<3"]
# ///
# How to run: uv run art/compositor/source_scene.py /Users/rexxa/work/artenv1-data
"""Package native-probe-verified source scenes without inventing simulation fields."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path
from typing import ClassVar, Final, Literal

from pydantic import BaseModel, ConfigDict

ROOT: Final = Path(__file__).resolve().parents[2]
COMMIT: Final = "3a3b236c125c0c25e9b74261be88536473fc01e9"
SCENES: Final = ("day6_noon", "day6_night", "winter_noon")
STAGES: Final = {0: "adult", 1: "child", 2: "elder"}
SEASONS: Final = ("spring", "summer", "autumn", "winter")


class Frozen(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")


class NativePerson(Frozen):
    id: int
    x: int
    y: int
    female: bool | None
    life_stage_code: int
    asleep: bool
    state_code: int
    target_code: int
    job_code: int
    carry_kind: int
    carry_count: int


class RichScene(Frozen):
    seed: int
    tick: int
    width: int
    height: int
    display_day: int
    clock_minute: int
    season_index: int
    camp: tuple[int, int]
    terrain_rows: tuple[str, ...]
    glyph_rows: tuple[str, ...]
    people: tuple[NativePerson, ...]
    native_ascii_equal: bool
    native_people_multiset_equal: bool


class Person(NativePerson):
    index: int
    glyph: str
    life_stage: str
    activity: Literal["awake", "sleep", "carry", "chop"]
    source: str = "native_ascii_verified_bridge_collectors"


class Provenance(Frozen):
    probe_commit: str
    binary_sha256: str
    extractor_sha256: str
    command: str
    extract_command: str
    ascii_sha256: str
    people_sha256: str
    rich_sha256: str
    command_script_sha256: str
    cargo_lock_sha256: str


class Scene(Frozen):
    schema_version: int = 1
    scene_id: str
    seed: int
    tick: int
    width: int
    height: int
    display_day: int
    clock_minute: int
    season: str
    camp: tuple[int, int]
    terrain_rows: tuple[str, ...]
    glyph_rows: tuple[str, ...]
    people: tuple[Person, ...]
    provenance: Provenance
    limitations: tuple[str, ...]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def package_scene(data: Path, name: str) -> Scene:
    raw = data / "scenes"
    probe = data / "probe-worktree"
    rich_path = raw / f"rich_{name}.json"
    rich = RichScene.model_validate_json(rich_path.read_bytes())
    if not (rich.native_ascii_equal and rich.native_people_multiset_equal):
        raise RuntimeError(f"Native agreement failed: {name}")
    rows = (raw / f"scene_{name}.txt").read_text().splitlines()
    if tuple(rows) != rich.glyph_rows:
        raise RuntimeError(f"ASCII changed after extraction: {name}")
    people: list[Person] = []
    for index, person in enumerate(rich.people):
        activity: Literal["awake", "sleep", "carry", "chop"] = "awake"
        if person.asleep:
            activity = "sleep"
        elif person.carry_count > 0:
            activity = "carry"
        elif person.job_code == 1 and person.state_code == 3:
            activity = "chop"
        people.append(Person(
            id=person.id, x=person.x, y=person.y, female=person.female,
            life_stage_code=person.life_stage_code, asleep=person.asleep,
            state_code=person.state_code, target_code=person.target_code,
            job_code=person.job_code, carry_kind=person.carry_kind,
            carry_count=person.carry_count, index=index,
            glyph=rows[person.y][person.x], life_stage=STAGES[person.life_stage_code],
            activity=activity,
        ))
    command = (
        f"./rust/target/release/worldsim_probe --scene colony --seeds 0..1 --ascii --tick {rich.tick} "
        f"--commands tools/v8/soak_script.json --out {raw}/scene_{name}.txt "
        f"--people-json {raw}/people_{name}.json"
    )
    extract_command = (
        f"./rust/target/release/env1-probe-extract 0 {rich.tick} tools/v8/soak_script.json "
        f"{raw}/scene_{name}.txt {raw}/people_{name}.json {rich_path}"
    )
    provenance = Provenance(
        probe_commit=COMMIT,
        binary_sha256=sha256(probe / "rust/target/release/worldsim_probe"),
        extractor_sha256=sha256(probe / "rust/target/release/env1-probe-extract"),
        command=command, extract_command=extract_command,
        ascii_sha256=sha256(raw / f"scene_{name}.txt"),
        people_sha256=sha256(raw / f"people_{name}.json"),
        rich_sha256=sha256(rich_path),
        command_script_sha256=sha256(probe / "tools/v8/soak_script.json"),
        cargo_lock_sha256=sha256(probe / "rust/Cargo.lock"),
    )
    return Scene(
        scene_id=f"scene_{name}", seed=rich.seed, tick=rich.tick,
        width=rich.width, height=rich.height, display_day=rich.display_day,
        clock_minute=rich.clock_minute, season=SEASONS[rich.season_index],
        camp=rich.camp, terrain_rows=rich.terrain_rows, glyph_rows=rich.glyph_rows,
        people=tuple(people), provenance=provenance,
        limitations=(
            "Native people JSON supplies positions only; demographics and exact terrain are from the separately verified read-only extractor.",
            "ASCII l/L conflates every ground item kind; it does not prove that a stack contains logs.",
            "ASCII omits M3-M5 space/workshops/animals/apparel; declared overlay objects are not simulated observations.",
            "ASCII is flattened: terrain is recovered exactly by the extractor but hidden foreground objects remain absent.",
            "Native ASCII runner applies script entries once; every_days repeats and after-condition retries of the non-ASCII colony runner are not executed.",
            "Top/hair/look and animation facing are art defaults; ID, sex, stage, carry and job fields are actual simulation observations.",
        ),
    )


def main() -> None:
    data = Path(sys.argv[1])
    destination = ROOT / "art/compositor/scenes"
    destination.mkdir(parents=True, exist_ok=True)
    records: list[dict[str, str | int]] = []
    for name in SCENES:
        scene = package_scene(data, name)
        path = destination / f"scene_{name}.json"
        _ = path.write_text(scene.model_dump_json() + "\n")
        records.append({
            "scene_id": scene.scene_id, "path": str(path.relative_to(ROOT)),
            "sha256": sha256(path), "bytes": path.stat().st_size,
            "seed": scene.seed, "tick": scene.tick, "people": len(scene.people),
            "display_day": scene.display_day, "clock_minute": scene.clock_minute,
            "season": scene.season,
        })
        print(f"{path}: {scene.width}x{scene.height}, {len(scene.people)} people, camp={scene.camp}")
    manifest = {
        "schema_version": 1, "probe_commit": COMMIT,
        "probe_worktree": str(data / "probe-worktree"),
        "raw_directory": str(data / "scenes"), "scenes": records,
        "native_ascii_equal": True, "native_people_multiset_equal": True,
        "source_builder": "art/compositor/source_scene.py",
        "read_only_extractor": "art/compositor/probe_extract/src/main.rs",
        "calendar": {"ticks_per_day": 1440, "offset_ticks": 480, "days_per_season": 6},
    }
    _ = (ROOT / "docs/v8/artenv1/scenes/source_manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n",
    )


if __name__ == "__main__":
    main()
