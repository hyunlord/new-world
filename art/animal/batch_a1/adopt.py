# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "scipy", "pydantic"]
# ///
# How to run: python3 -m art.animal.batch_a1.adopt /absolute/selection.json
"""Assemble verified, reviewed animal candidates without changing source PNG bytes."""
from __future__ import annotations

import json
import shutil
import sys
from dataclasses import asdict, dataclass
from datetime import datetime
from pathlib import Path
from typing import ClassVar, Final, Literal

from PIL import Image, ImageDraw
from pydantic import BaseModel, ConfigDict, Field, JsonValue, TypeAdapter

from art.animal.batch_a1.check import Manifest, inspect_frame, verify
from art.pipeline_env1.animal_normalize import digest

ROOT: Final = Path(__file__).resolve().parents[3]
BASE: Final = ROOT / "art/animal/batch_a1"
STATUS: Final = "ACCEPTED_ART_CANDIDATE"
JSON_OBJECT: Final = TypeAdapter[dict[str, JsonValue]](dict[str, JsonValue])


class Entry(BaseModel):
    """One explicit selection and its independent review evidence."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")
    species: str = Field(pattern=r"^[a-z][a-z0-9_]*$")
    base_manifest: Path
    attack_manifest: Path | None = None
    review_paths: tuple[Path, ...] = Field(min_length=1)


class Selection(BaseModel):
    """Root owns visual acceptance; the assembler checks the selected bytes."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")
    review_status: Literal["pass"]
    selections: tuple[Entry, ...] = Field(min_length=1)
    evidence_dir: Path = Path("/Users/rexxa/work/art-batch/A1")


class SetSpec(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    id: str
    frame_count: int
    attack: bool


class Catalogue(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    required_set_count: int
    required_frame_count: int
    sets: tuple[SetSpec, ...]
    optional_after_required: tuple[SetSpec, ...]


class Frame(BaseModel):
    """Importer-compatible frame row; full provenance remains in its sidecar."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    species: str
    facing: str
    pose: str
    image: Path
    image_sha256: str
    metadata: Path
    canvas_px: tuple[int, int]
    anchor_px: tuple[int, int]
    footprint_tiles: tuple[int, int] = (1, 1)
    structural_pass: bool = True
    status: str = STATUS


class Legacy(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    frames: tuple[Frame, ...]


@dataclass(frozen=True, slots=True)
class Candidate:
    frame: Frame
    source_manifest: Path
    reviews: tuple[Path, ...]


def copy_exact(source: Path, target: Path, expected_sha: str) -> None:
    """Reject changed source bytes and copy without decoding or re-encoding."""
    if digest(source) != expected_sha:
        message = f"Source hash changed: {source}"
        raise ValueError(message)
    target.parent.mkdir(parents=True, exist_ok=True)
    _ = shutil.copyfile(source, target)
    if digest(target) != expected_sha:
        message = f"Copied bytes differ: {target}"
        raise ValueError(message)


def collect(selection: Selection) -> tuple[Candidate, ...]:
    """Validate the complete catalogue matrix before creating delivery files."""
    catalogue = Catalogue.model_validate_json((BASE / "catalog.json").read_text())
    required = {row.id for row in catalogue.sets}
    available = {row.id: row for row in (*catalogue.sets, *catalogue.optional_after_required)}
    names = [entry.species for entry in selection.selections]
    if (len(names) != len(set(names)) or required - set(names) or set(names) - available.keys()
            or len(required) != catalogue.required_set_count
            or sum(row.frame_count for row in catalogue.sets) != catalogue.required_frame_count):
        message = f"Selection must contain every required set exactly once; missing={sorted(required - set(names))}"
        raise ValueError(message)
    result: list[Candidate] = []
    for entry in selection.selections:
        for review in entry.review_paths:
            _ = digest(review)
        selected: list[Candidate] = []
        paths = (entry.base_manifest,) + ((entry.attack_manifest,) if entry.attack_manifest is not None else ())
        for path in paths:
            if not verify(ROOT / path):
                message = f"Selected manifest failed: {path}"
                raise ValueError(message)
            manifest = Manifest.model_validate_json((ROOT / path).read_text())
            if manifest.spec.get("species") != entry.species:
                message = f"Species differs between selection and manifest: {path}"
                raise ValueError(message)
            selected.extend(Candidate(Frame(species=entry.species, facing=f.facing, pose=f.pose,
                image=f.image, image_sha256=f.image_sha256, metadata=f.image.with_suffix(".json"),
                canvas_px=f.canvas_px, anchor_px=f.anchor_px), path, entry.review_paths) for f in manifest.frames)
        poses = ("stand", "walk", "run", "graze", "dead") + (("attack",) if available[entry.species].attack else ())
        expected = {(f, p) for f in ("S", "E") for p in poses}
        if len(selected) != available[entry.species].frame_count or {(r.frame.facing, r.frame.pose) for r in selected} != expected:
            message = f"Missing, duplicated or unexpected pose for {entry.species}"
            raise ValueError(message)
        result.extend(selected)
    legacy_path = Path("docs/v8/artpolish/animal_manifest.json")
    legacy = Legacy.model_validate_json((ROOT / legacy_path).read_text())
    for frame in legacy.frames:
        source = ROOT / frame.image
        with Image.open(source) as image:
            good = all(asdict(inspect_frame(image, frame.pose == "dead", 56 if frame.species == "wolf" else 64)).values())
        if frame.species not in {"deer", "wolf"} or digest(source) != frame.image_sha256 or not good:
            message = f"Existing reference failed integrity: {source}"
            raise ValueError(message)
        result.append(Candidate(frame, legacy_path, ()))
    if len(legacy.frames) != 20:
        message = "Expected exactly twenty existing deer/wolf frames"
        raise ValueError(message)
    return tuple(result)


def board(frames: tuple[Frame, ...], target: Path) -> None:
    """Compact native 1x and nearest 2x comparison, never a game screenshot."""
    height = ((len(frames) + 4) // 5) * 240 + 30
    if height > 2000:
        message = f"Contact sheet too tall: {height}"
        raise ValueError(message)
    canvas = Image.new("RGB", (1100, height), "#d8cfba")
    draw = ImageDraw.Draw(canvas)
    draw.text((10, 8), "A1 candidates | native 1x above / nearest 2x below | deer + wolf references", fill="#292a24")
    for i, frame in enumerate(frames):
        x, y = i % 5 * 220, i // 5 * 240 + 30
        draw.text((x + 6, y + 2), f"{frame.species} {frame.facing}/{frame.pose}", fill="#292a24")
        with Image.open(ROOT / frame.image) as opened:
            sprite = opened.convert("RGBA")
        for scale, baseline in ((1, y + 84), (2, y + 229)):
            enlarged = sprite.resize((sprite.width * scale, sprite.height * scale), Image.Resampling.NEAREST)
            canvas.paste(enlarged, (x + 110 - frame.anchor_px[0] * scale, baseline - frame.anchor_px[1] * scale), enlarged)
    target.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(target, optimize=True)


def assemble(path: Path) -> Path:
    """Publish an art candidate pack and provenance after all selections verify."""
    selection = Selection.model_validate_json(path.read_text())
    candidates = collect(selection)
    frames: list[Frame] = []
    for candidate in candidates:
        old = candidate.frame
        target = BASE / "adopted" / old.species / old.facing / f"{old.pose}.png"
        copy_exact(ROOT / old.image, target, old.image_sha256)
        current = old.model_copy(update={"image": target.relative_to(ROOT), "metadata": target.with_suffix(".json").relative_to(ROOT), "status": STATUS})
        meta = JSON_OBJECT.validate_json((ROOT / old.metadata).read_text())
        meta.update({"source_status": meta.get("status"), "status": STATUS, "import_status": "PENDING",
            "source_image": str(old.image), "source_sidecar": str(old.metadata), "source_sidecar_sha256": digest(ROOT / old.metadata),
            "source_manifest": str(candidate.source_manifest), "source_manifest_sha256": digest(ROOT / candidate.source_manifest),
            "selection": str(path.resolve()), "selection_sha256": digest(path), "review_status": selection.review_status,
            "review_evidence": [{"path": str(p.resolve()), "sha256": digest(p)} for p in candidate.reviews],
            "reference_reuse": old.species in {"deer", "wolf"}, "image": str(current.image), "image_sha256": old.image_sha256,
            "canvas_px": list(old.canvas_px), "anchor_px": list(old.anchor_px), "footprint_tiles": [1, 1]})
        _ = (ROOT / current.metadata).write_text(json.dumps(meta, indent=2) + "\n")
        frames.append(current)
    refs = tuple(f for f in frames if f.species in {"deer", "wolf"} and f.pose == "stand")
    sheets: list[Path] = []
    groups = {species: tuple(f for f in frames if f.species == species and f.pose != "attack") + refs
              for species in sorted({f.species for f in frames} - {"deer", "wolf"})}
    groups["overview"] = tuple(f for f in frames if f.pose == "stand")
    groups["attacks"] = tuple(f for f in frames if f.pose == "attack") + refs
    stamp = datetime.now().astimezone().strftime("%H%M")
    for name, group in groups.items():
        sheet = BASE / "boards" / f"{name}.png"
        board(group, sheet)
        evidence = selection.evidence_dir / "sheets" / f"{stamp}_adopted_{name}.png"
        copy_exact(sheet, evidence, digest(sheet))
        sheets.append(sheet)
    catalogue = Catalogue.model_validate_json((BASE / "catalog.json").read_text())
    required = {s.id for s in catalogue.sets}
    manifest = ROOT / "docs/v8/artbatch_a1/animal_manifest.json"
    manifest.parent.mkdir(parents=True, exist_ok=True)
    _ = manifest.write_text(json.dumps({"status": STATUS, "owner_adopted": False, "import_status": "PENDING",
        "selection": str(path.resolve()), "selection_sha256": digest(path), "review_status": selection.review_status,
        "frames": [f.model_dump(mode="json") for f in frames], "frame_count": len(frames),
        "coverage": {"required_sets": len(required), "required_frames": catalogue.required_frame_count,
            "required_selected_frames": sum(f.species in required for f in frames), "required_complete": True,
            "optional_selected_frames": sum(f.species not in required | {"deer", "wolf"} for f in frames), "existing_reference_frames": 20},
        "boards": [{"path": str(p.relative_to(ROOT)), "sha256": digest(p)} for p in sheets],
        "limitations": ["One pose key per direction, not animation cycles", "Game import and owner approval pending", "Visual pass is root's recorded selection decision"]}, indent=2) + "\n")
    print(json.dumps({"manifest": str(manifest), "frames": len(frames), "boards": len(sheets)}))
    return manifest


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("Usage: python3 -m art.animal.batch_a1.adopt selection.json")
    _ = assemble(Path(sys.argv[1]))
