"""Synthetic numeric checks for the R1 shipped-look comparison."""

import json
import subprocess
import sys
from dataclasses import replace
from math import isclose
from pathlib import Path

import pytest
from PIL import Image

from art.batch_r1 import metrics


def fixture() -> tuple[Image.Image, metrics.FrameRegions]:
    """Known ten-pixel silhouette with two-pixel dark boundary."""
    image = Image.new("RGBA", (20, 20))
    for y in range(3, 17):
        for x in range(5, 15):
            image.putpixel((x, y), (30, 20, 10, 255) if x < 7 else (200, 100, 100, 255))
    regions = metrics.FrameRegions(
        (5,),
        (12,),
        frozenset({(8, 8), (9, 8)}),
        (((5, 8), (6, 8), (7, 8), (8, 8)),),
        60,
    )
    return image, regions


def test_known_measurements() -> None:
    # Given a known rectangular silhouette and explicit clothing/probe samples.
    image, regions = fixture()
    # When measured.
    result = metrics.measure_frame(image, regions)
    # Then the values match independent arithmetic.
    assert result.shoulder_width == 10
    assert result.hip_width == 10
    assert isclose(result.clothing_saturation, 0.5, rel_tol=1e-6, abs_tol=1e-12)
    assert isclose(result.clothing_value, 200 / 255, rel_tol=1e-6, abs_tol=1e-12)
    assert isclose(
        result.outline_thickness, 2 * (1 - 30 / 255), rel_tol=1e-6, abs_tol=1e-12
    )


def test_rejects_changed_width_and_outline() -> None:
    # Given a wider candidate with a thicker outline.
    image, regions = fixture()
    candidate = image.copy()
    for y in range(3, 17):
        candidate.putpixel((15, y), (200, 100, 100, 255))
        candidate.putpixel((16, y), (200, 100, 100, 255))
    candidate.putpixel((7, 8), (30, 20, 10, 255))
    # When compared.
    result = metrics.compare_frames(image, candidate, regions)
    # Then a width ratio 1.2 fails while the fixed two-pixel edge band is unchanged.
    assert not result.passed
    assert result.ratios.shoulder_width == 1.2
    assert result.ratios.outline_thickness == 1


def test_accepts_exact_ten_percent_boundary() -> None:
    # Given ten-pixel reference and eleven-pixel candidate widths.
    image, regions = fixture()
    candidate = image.copy()
    for y in range(3, 17):
        candidate.putpixel((15, y), (200, 100, 100, 255))
    # When compared at the inclusive boundary.
    result = metrics.compare_frames(image, candidate, regions)
    # Then every metric passes.
    assert result.passed


@pytest.mark.parametrize(
    "field", ["shoulder_rows", "hip_rows", "clothing", "outline_probes"]
)
def test_rejects_empty_regions(field: str) -> None:
    # Given an empty mandatory annotation.
    image, regions = fixture()
    changes: dict[str, frozenset[metrics.Point] | tuple[()]] = {
        field: frozenset() if field == "clothing" else ()
    }
    # When measured, then missing evidence fails closed.
    with pytest.raises(metrics.MeasurementError):
        _ = metrics.measure_frame(image, replace(regions, **changes))


def test_rejects_missing_candidate_clothing() -> None:
    # Given transparent candidate pixels inside the fixed reference clothing mask.
    image, regions = fixture()
    candidate = image.copy()
    candidate.putpixel((8, 8), (0, 0, 0, 0))
    # When compared, then the mask cannot silently shrink.
    with pytest.raises(metrics.MeasurementError):
        _ = metrics.compare_frames(image, candidate, regions)


def test_rejects_all_dark_probe() -> None:
    # Given a probe with no observed lighter interior.
    image, regions = fixture()
    regions = replace(regions, outline_probes=(((5, 8), (6, 8)),))
    # When measured, then truncated outline evidence fails.
    with pytest.raises(metrics.MeasurementError):
        _ = metrics.binary_outline_thickness(image, regions)


def test_probe_allows_one_pixel_contour_shift() -> None:
    # Given a fixed span starting outside both silhouettes.
    image, regions = fixture()
    regions = replace(regions, outline_probes=(tuple((x, 8) for x in range(3, 10)),))
    candidate = image.copy()
    candidate.putpixel((4, 8), (30, 20, 10, 255))
    candidate.putpixel((6, 8), (200, 100, 100, 255))
    # When compared, then equal two-pixel ink runs survive an allowed width shift.
    result = metrics.compare_frames(image, candidate, regions)
    assert result.passed
    assert result.ratios.outline_thickness == 1


def test_rejects_washed_out_clothing() -> None:
    # Given clothing with unchanged value and half the reference saturation.
    image, regions = fixture()
    candidate = image.copy()
    for point in regions.clothing:
        candidate.putpixel(point, (200, 150, 150, 255))
    # When compared, then colour alone fails the acceptance band.
    result = metrics.compare_frames(image, candidate, regions)
    assert not result.passed
    assert isclose(result.ratios.clothing_saturation, 0.5, rel_tol=1e-6, abs_tol=1e-12)


def test_region_json_roundtrip(tmp_path: Path) -> None:
    # Given reviewed annotations written to disk.
    _, regions = fixture()
    path = tmp_path / "regions.json"
    metrics.write_regions(path, regions)
    # When read back, then no coordinates or threshold change.
    assert metrics.read_regions(path) == regions


@pytest.mark.parametrize(
    "bad", [None, {"shoulder_rows": [True]}, {"outline_max_value": 5}]
)
def test_rejects_malformed_region_json(tmp_path: Path, bad: metrics.Json) -> None:
    # Given invalid external annotation JSON.
    path = tmp_path / "regions.json"
    _ = path.write_text(json.dumps(bad))
    # When parsed, then the boundary rejects the input.
    with pytest.raises(metrics.MeasurementError):
        _ = metrics.read_regions(path)


def test_cli_writes_report_and_rejects_colour_failure(tmp_path: Path) -> None:
    # Given actual PNGs and an annotation file on disk.
    image, regions = fixture()
    reference, candidate = tmp_path / "ref.png", tmp_path / "candidate.png"
    image.save(reference)
    for point in regions.clothing:
        image.putpixel(point, (200, 150, 150, 255))
    image.save(candidate)
    mask, report = tmp_path / "mask.json", tmp_path / "report.json"
    metrics.write_regions(mask, regions)
    # When driven through the real CLI.
    result = subprocess.run(
        [
            sys.executable,
            "-m",
            "art.batch_r1.metrics",
            "compare",
            str(reference),
            str(candidate),
            str(mask),
            "--output",
            str(report),
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    # Then a numeric failure is persisted and the process reports rejection.
    assert result.returncode == 1, result.stderr
    assert '"passed": false' in report.read_text()
    assert '"clothing_saturation": 0.5' in report.read_text()


def test_overlay_cli_writes_inspectable_mask(tmp_path: Path) -> None:
    # Given a real frame and reviewed annotation file.
    image, regions = fixture()
    reference, mask, overlay = (
        tmp_path / "ref.png",
        tmp_path / "mask.json",
        tmp_path / "overlay.png",
    )
    image.save(reference)
    metrics.write_regions(mask, regions)
    # When rendering the actual CLI overlay.
    result = subprocess.run(
        [
            sys.executable,
            "-m",
            "art.batch_r1.metrics",
            "render-masks",
            str(reference),
            str(mask),
            "--output",
            str(overlay),
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    # Then clothing is visibly marked cyan at the exact selected coordinate.
    assert result.returncode == 0, result.stderr
    with Image.open(overlay) as rendered:
        assert rendered.getpixel((9, 8)) == (0, 255, 255, 255)


def test_all_shipped_annotations_have_nonempty_measurements() -> None:
    # Given every matching hide/fur reference across the requested twelve combinations.
    root = Path(__file__).resolve().parents[2]
    names = {
        f"{body}_{sex}_{garment}_{hair}"
        for body in ("adult", "child")
        for sex in ("M", "F")
        for garment in ("hide_wrap", "fur_cloak")
        for hair in ("short", "bun", "long_tied")
        if (body, sex, hair) != ("adult", "M", "long_tied")
    }
    # When measuring their independently reviewed masks against the actual shipped PNGs.
    values: list[metrics.FrameMetrics] = []
    for name in sorted(names):
        region = metrics.read_regions(root / "art/batch_r1/regions" / f"{name}.json")
        with Image.open(root / "art/character/route_i" / name / "S/idle.png") as image:
            values.append(metrics.measure_frame(image, region))
    # Then all 22 distinct shipped looks have complete positive evidence.
    assert len(values) == 22
    assert all(
        value.shoulder_width > 0
        and value.hip_width > 0
        and value.clothing_saturation > 0
        and value.clothing_value > 0
        and value.outline_thickness > 0
        for value in values
    )


def test_diagnostics_retain_valid_fields_when_outline_fails() -> None:
    # Given valid anatomy/clothing with an unmeasurable dark-only probe.
    image, regions = fixture()
    regions = replace(regions, outline_probes=(((0, 0), (1, 0)),))
    # When diagnosing instead of certifying.
    result = metrics.diagnose_frame(image, regions)
    # Then four valid measurements remain and the invalid field is explicit null.
    assert result.metrics.shoulder_width == 10
    assert result.metrics.hip_width == 10
    assert result.metrics.clothing_saturation is not None
    assert isclose(result.metrics.clothing_saturation, 0.5, rel_tol=1e-6, abs_tol=1e-12)
    assert result.metrics.clothing_value is not None
    assert isclose(
        result.metrics.clothing_value, 200 / 255, rel_tol=1e-6, abs_tol=1e-12
    )
    assert result.metrics.outline_thickness is None
    assert len(result.errors) == 1


def test_diagnostics_do_not_average_uncovered_clothing() -> None:
    # Given a missing garment pixel and valid other evidence.
    image, regions = fixture()
    image.putpixel((9, 8), (255, 0, 0, 0))
    # When diagnosing, then invalid colour evidence is null rather than a biased mean.
    result = metrics.diagnose_frame(image, regions)
    assert result.metrics.clothing_saturation is None
    assert result.metrics.clothing_value is None
    assert result.metrics.outline_thickness is not None
    assert isclose(
        result.metrics.outline_thickness,
        2 * (1 - 30 / 255),
        rel_tol=1e-6,
        abs_tol=1e-12,
    )
    assert result.metrics.shoulder_width == 10


@pytest.mark.parametrize("alpha", [16, 113, 128, 255])
def test_effective_outline_integrates_fractional_black_ink(alpha: int) -> None:
    # Given one half-covered black edge pixel followed by white fill.
    image, regions = fixture()
    image.putpixel((5, 8), (0, 0, 0, alpha))
    image.putpixel((6, 8), (255, 255, 255, 255))
    # When measuring effective ink in the first two occupied pixels.
    result = metrics.measure_frame(image, regions)
    # Then coverage is fractional pixels, not an integer threshold count.
    assert isclose(result.outline_thickness, alpha / 255, rel_tol=1e-6, abs_tol=1e-12)


def test_effective_outline_ignores_subthreshold_halo() -> None:
    # Given a subthreshold halo before a gray partial-coverage edge and white fill.
    image, regions = fixture()
    regions = replace(regions, outline_probes=(((4, 8), (5, 8), (6, 8), (7, 8)),))
    image.putpixel((4, 8), (0, 0, 0, 15))
    image.putpixel((5, 8), (127, 100, 50, 128))
    image.putpixel((6, 8), (255, 255, 255, 255))
    # When measuring, then the halo is skipped and darkness uses max RGB channel.
    result = metrics.measure_frame(image, regions)
    assert isclose(
        result.outline_thickness,
        128 / 255 * (1 - 127 / 255),
        rel_tol=1e-6,
        abs_tol=1e-12,
    )


def test_clothing_allows_one_missing_pixel_of_120() -> None:
    # Given a 120-pixel garment mask with one pixel falling below alpha128.
    image, regions = fixture()
    clothing = frozenset((x, y) for y in range(3, 15) for x in range(5, 15))
    regions = replace(regions, clothing=clothing)
    image.putpixel((8, 8), (255, 0, 0, 127))
    # When measuring, then all119visible garment pixels contribute without a color filter.
    result = metrics.measure_frame(image, regions)
    assert isclose(
        result.clothing_saturation,
        (24 * 2 / 3 + 95 * 0.5) / 119,
        rel_tol=1e-6,
        abs_tol=1e-12,
    )
    assert isclose(
        result.clothing_value,
        (24 * 30 + 95 * 200) / (119 * 255),
        rel_tol=1e-6,
        abs_tol=1e-12,
    )


@pytest.mark.parametrize("missing,accepted", [(12, True), (13, False)])
def test_clothing_enforces_inclusive_ninety_percent_coverage(
    missing: int, accepted: bool
) -> None:
    # Given a120-pixel frozen mask with exactly90% or just under90% visible coverage.
    image, regions = fixture()
    regions = replace(
        regions, clothing=frozenset((x, y) for y in range(3, 15) for x in range(5, 15))
    )
    for point in sorted(regions.clothing, key=lambda p: (p[1], p[0]))[:missing]:
        image.putpixel(point, (255, 0, 0, 0))
    # When measured, then exactly90% passes and13missing pixels fails closed.
    if accepted:
        assert metrics.measure_frame(image, regions).clothing_value > 0
    else:
        with pytest.raises(
            metrics.MeasurementError, match="clothing coverage below 90%"
        ):
            _ = metrics.measure_frame(image, regions)
