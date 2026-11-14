"""Required plans must exercise corner, T and X joins explicitly."""

from pathlib import Path

from art.batch_c3a.walls_pass2.build import topology


def test_required_plans_include_both_t_and_x() -> None:
    """Given the planner's hollow junctions, review must include a real X."""
    from art.batch_c3a.walls_pass2.review_boards import plans

    cases = plans()
    assert [p.name for p in cases] == ["5x4", "L", "T+X"]
    masks = {topology(cell, cases[2].cells) for cell in cases[2].cells}
    assert 15 in masks
    assert masks & {7, 11, 13, 14}


def test_zoom_includes_all_join_pixels_without_filtering(tmp_path: Path) -> None:
    """Given a marked room pixel, the4x output preserves its literal block."""
    from PIL import Image

    from art.batch_c3a.walls_pass2.review_boards import JoinView, write_zooms

    room = Image.new("RGBA", (352, 304), (180, 150, 120, 255))
    room.putpixel((80, 80), (17, 35, 57, 255))
    write_zooms(
        (JoinView("T+X", (1, 1), 15, {"shipped log": room, "native stone": room}),),
        tmp_path,
    )
    with Image.open(next((tmp_path / "join_zooms_4x").glob("*.png"))) as board:
        assert board.size == (768, 420)
        # Pixel(80,80) lies16px inside the crop rooted at(64,64).
        assert board.crop((64, 94, 68, 98)).getcolors() == [(16, (17, 35, 57))]
