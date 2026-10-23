# /// script
# requires-python = ">=3.11"
# dependencies = ["pytest", "pillow", "pydantic", "numpy"]
# ///
# How to run: python3 -m pytest art/compositor/test_polish2_boards.py -q.
"""Check real painter selection and pixel-preserving owner-board composition."""
from pathlib import Path

from PIL import Image

from art.compositor.assets import Catalog, choose_look
from art.compositor.drawing import ForegroundPainter
from art.compositor.face_variants import FacePlan, choose_variant
from art.compositor.geometry import Viewport
from art.compositor.models import Activity, Scene
from art.compositor.placements import RenderPerson


def test_painter_records_age_class_selection_without_inventing_source_age() -> None:
    # Given an elder in the actual scene, positioned visibly for the painter.
    scene = Scene.read(Path("art/compositor/scenes/scene_day6_noon.json"))
    person = scene.people[0].model_copy(update={"life_stage": "elder"})
    actor = RenderPerson(person, scene.camp, Activity.AWAKE)
    plan = FacePlan.read()
    image = Image.new("RGBA", (1600, 900))
    painter = ForegroundPainter(image, Viewport(scene.camp, .5), Catalog({}), True, face_plan=plan)
    # When selecting the half-scale marker (no baked images required).
    record = painter.person(actor)
    # Then selection uses the older pool without claiming a captured age.
    assert record is not None
    variant = choose_variant(person.id, 50, plan.for_look(choose_look(person)))
    assert record["variant"] == variant.id
    assert record["variant_selection_age"] == 50
    assert record["variant_age_basis"] == "life_stage_representative_not_captured_age"


def test_legacy_painter_does_not_require_face_plan_or_baked_variants() -> None:
    # Given the unchanged caller contract.
    scene = Scene.read(Path("art/compositor/scenes/scene_day6_noon.json"))
    actor = RenderPerson(scene.people[0], scene.camp, Activity.AWAKE)
    image = Image.new("RGBA", (1600, 900))
    painter = ForegroundPainter(image, Viewport(scene.camp, .5), Catalog({}), True)
    # When drawing the legacy marker.
    record = painter.person(actor)
    # Then no variant selection is claimed and the pixel size stays 10×10.
    assert record is not None
    assert "variant" not in record
    bounds = image.getbbox()
    assert bounds is not None
    assert (bounds[2] - bounds[0], bounds[3] - bounds[1]) == (10, 10)


def test_board_preserves_exact_native_and_doubled_source_pixels(tmp_path: Path) -> None:
    # Given a tiny real source carrying a distinctive pixel.
    from art.compositor.polish2_layout import BoardLayout, BoardPanel, make_board

    source = Image.new("RGB", (8, 8), (17, 44, 201))
    path = tmp_path / "source.png"
    source.save(path)
    output = tmp_path / "board.png"
    panels = (BoardPanel(path, (0, 0, 8, 8), (10, 50), "native"),
              BoardPanel(path, (0, 0, 8, 8), (30, 50), "double", 2))
    # When assembling both declared scales.
    record = make_board(output, BoardLayout((300, 100), "Test", panels))
    # Then each art region remains pixel-exact, and provenance binds the file.
    with Image.open(output) as board:
        assert board.crop((10, 50, 18, 58)).tobytes() == source.tobytes()
        assert board.crop((30, 50, 46, 66)).tobytes() == source.resize((16, 16), Image.Resampling.NEAREST).tobytes()
    assert len(record.panels) == 2
    assert record.bytes == output.stat().st_size
