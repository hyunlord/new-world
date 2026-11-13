"""Verify independent depth layers and a shared, unscaled footprint."""

from art.batch_c3a.workshops.build import stages


def test_stages_preserve_ring_and_leave_the_door_opening() -> None:
    """Given a4x4 workshop, construction keeps one physical ring."""
    # Given / When: both standard workshops use the same declared perimeter.
    rows = stages()
    # Then: full walls never overlap, each state has12 ring cells minus doorway.
    assert len(rows) == 5
    for _, courses in rows:
        cells = [cell for _, group in courses for cell in group]
        assert len(cells) == len(set(cells))
        assert (2, 3) not in cells
        assert all(0 <= x < 4 and 0 <= y < 4 for x, y in cells)
    assert len([cell for _, group in rows[2][1] for cell in group]) == 11


def test_smithy_keeps_pinned_forge_pixels_and_world_depth() -> None:
    """Given the published forge, layering must preserve its pixels and base."""
    from art.batch_c3a.workshops.build import layers, pinned

    # Given: the independently pinned A2 forge is the station source.
    source, _, _ = pinned("forge_idle")
    # When: the complete room is assembled.
    result = layers("smithy", False, stages()[2][1], "built")
    # Then: ground is16 whole cells; the forge keeps literal pixels and base107.
    assert sum(layer.kind == "floor" for layer in result) == 16
    forge = next(layer for layer in result if layer.source.endswith("forge_idle.png"))
    assert forge.image.tobytes() == source.tobytes()
    assert forge.origin[1] + forge.anchor[1] == 107
    keys = [
        (
            layer.kind != "floor",
            layer.origin[1] + layer.anchor[1],
            layer.origin[0] + layer.anchor[0],
        )
        for layer in result
    ]
    assert keys == sorted(keys)
