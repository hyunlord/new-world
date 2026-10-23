# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic", "numpy", "typer"]
# ///
# How to run: python3 -m art.compositor.polish2_faces (after final input freeze).
"""Keep all48 exact-scale pairs, splitting only after full-sheet optimization."""
from __future__ import annotations

import hashlib
import shutil
import tempfile
from pathlib import Path
from typing import ClassVar, Final, Literal

from PIL import Image
from pydantic import BaseModel, ConfigDict

from art.compositor.assets import REPO
from art.compositor.face_variants import FacePlan, VariantId, baked_frame
from art.compositor.polish2_layout import (
    BoardLayout,
    BoardPanel,
    BoardRecord,
    make_board,
)
from art.compositor.render import sha256

CELL_WIDTH: Final = 304
ROW_HEIGHT: Final = 276
FILE_LIMIT: Final = 1_500_000
EXTERNAL: Final = Path("/Users/rexxa/work/artpolish2-data/boards")


class FaceMember(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")
    look: str
    resolved_look: str
    variant: VariantId
    full_panel_indices: tuple[int, int]
    output_path: str
    output_panel_indices: tuple[int, int]


class FaceBoardIndex(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")
    schema_version: Literal[1] = 1
    layout: Literal["single", "adult_child_split"]
    file_limit_bytes: Literal[1_500_000] = FILE_LIMIT
    owner_sample: tuple[str, ...]
    parsed_face_plan_sha256: str
    full_sheet: BoardRecord
    outputs: tuple[BoardRecord, ...]
    members: tuple[FaceMember, ...]
    owner_approval: Literal["PENDING"] = "PENDING"


def face_layout(root: Path, plan: FacePlan, names: tuple[str, ...], title: str) -> BoardLayout:
    """Validate canonical dimensions before copying each full portrait and frame."""
    panels: list[BoardPanel] = []
    for row, name in enumerate(names):
        look = plan.for_look(name)
        for column, variant in enumerate(look.variants):
            x, y = 12 + column * CELL_WIDTH, 66 + row * ROW_HEIGHT
            portrait = root / "art/portrait/polish2" / look.look / f"{variant.id}.png"
            sprite = baked_frame(root, look, variant)
            for path, expected in ((portrait, (128, 128)), (sprite, (80, 112))):
                with Image.open(path) as source:
                    if source.size != expected:
                        message = f"Owner input must be exactly {expected}: {path}: {source.size}"
                        raise ValueError(message)
            label = f"{look.look} / {variant.id} / {variant.skin} {variant.hair}"
            panels.append(BoardPanel(portrait, (0, 0, 128, 128), (x, y), label))
            panels.append(BoardPanel(sprite, (0, 0, 80, 112), (x + 136, y), "", 2))
    return BoardLayout((24 + 6 * CELL_WIDTH, 66 + len(names) * ROW_HEIGHT), title, tuple(panels))


def publish_board(record: BoardRecord, source: Path, destination: Path, root: Path) -> BoardRecord:
    """Copy encoded bytes unchanged and relocate only their provenance path."""
    destination.parent.mkdir(parents=True, exist_ok=True)
    _ = shutil.copyfile(source, destination)
    path = str(destination.relative_to(root)) if destination.is_relative_to(root) else str(destination)
    published = record.model_copy(update={"path": path})
    _ = destination.with_suffix(".json").write_text(published.model_dump_json(indent=2) + "\n")
    return published


def faces_board(plan: FacePlan, *, root: Path = REPO, external: Path = EXTERNAL) -> FaceBoardIndex:
    """Stage complete final inputs; publish one sheet or an indexed adult/child pair."""
    adults = tuple(name for name in plan.owner_sample if name.startswith("adult_"))
    children = tuple(name for name in plan.owner_sample if name.startswith("child_"))
    if len(plan.owner_sample) != 8 or len(set(plan.owner_sample)) != 8 or len(adults) != 4 or len(children) != 4:
        message = "Owner sample requires four adult and four child looks"
        raise ValueError(message)
    layout = face_layout(root, plan, plan.owner_sample,
                         "FACE VARIANTS / all8 looks x6 / portrait128 + matching sprite2x")
    inputs = {panel.source: sha256(panel.source) for panel in layout.panels}
    external.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="face-board-", dir=external) as temporary:
        staging = Path(temporary)
        full_path = staging / "full.png"
        full = make_board(full_path, layout, enforce_cap=False, optimize_oversize=True)
        split = full.bytes > FILE_LIMIT
        groups = (adults, children) if split else (plan.owner_sample,)
        staged: list[tuple[BoardRecord, Path, str]] = []
        if split:
            for names, filename, title in (
                (adults, "faces_owner.png", "FACE VARIANTS / part1of2: adult4x6 / portrait128 + sprite2x / companion: faces_owner.children.png"),
                (children, "faces_owner.children.png", "FACE VARIANTS / part2of2: child4x6 / portrait128 + sprite2x / companion: faces_owner.png"),
            ):
                path = staging / filename
                record = make_board(path, face_layout(root, plan, names, title))
                staged.append((record, path, filename))
        else:
            staged.append((full, full_path, "faces_owner.png"))
        changed = [str(path) for path, digest in inputs.items() if sha256(path) != digest]
        if changed:
            message = f"Face-board inputs changed during rendering: {changed}"
            raise ValueError(message)
        full = publish_board(full, full_path, external / f"faces_owner.full.{full.sha256}.png", root)
        outputs = tuple(publish_board(record, source, root / "art/boards" / name, root)
                        for record, source, name in staged)
    members: list[FaceMember] = []
    for full_row, name in enumerate(plan.owner_sample):
        output_index = next(index for index, group in enumerate(groups) if name in group)
        output_row = groups[output_index].index(name)
        look = plan.for_look(name)
        for column, variant in enumerate(look.variants):
            full_panel, output_panel = (full_row * 6 + column) * 2, (output_row * 6 + column) * 2
            members.append(FaceMember(look=name, resolved_look=look.look, variant=variant.id,
                full_panel_indices=(full_panel, full_panel + 1), output_path=outputs[output_index].path,
                output_panel_indices=(output_panel, output_panel + 1)))
    index = FaceBoardIndex(layout="adult_child_split" if split else "single", owner_sample=plan.owner_sample,
        parsed_face_plan_sha256=hashlib.sha256(plan.model_dump_json().encode()).hexdigest(),
        full_sheet=full, outputs=outputs, members=tuple(members))
    _ = (root / "art/boards/faces_owner.index.json").write_text(index.model_dump_json(indent=2) + "\n")
    if not split:
        (root / "art/boards/faces_owner.children.png").unlink(missing_ok=True)
        (root / "art/boards/faces_owner.children.json").unlink(missing_ok=True)
    return index


def main() -> None:
    print(faces_board(FacePlan.read()).model_dump_json(indent=2))


if __name__ == "__main__":
    main()
