"""Add review labels outside preserved native render cells; uses installed Pillow.

python3 motion_labels.py REVIEW_DIRECTORY
"""
import json
import sys
from pathlib import Path

from PIL import Image, ImageDraw


def grid(directory: Path, tag: str, paths: list[Path]) -> None:
    """Keep every source cell at native512px in a four-column review grid."""
    rows = (len(paths) + 3) // 4
    result = Image.new("RGB", (2048, rows * 544), "#20252a")
    draw = ImageDraw.Draw(result)
    for index, path in enumerate(paths):
        source = Image.open(path).convert("RGB")
        assert source.size == (512, 512), path
        x, y = index % 4 * 512, index // 4 * 544
        result.paste(source, (x, y + 32))
        draw.text((x + 8, y + 8), f"{directory.name} / {path.stem}", fill="white")
        assert result.crop((x, y + 32, x + 512, y + 544)).tobytes() == source.tobytes()
    result.save(directory / f"{tag}_grid.png")


def main() -> None:
    directory = Path(sys.argv[1]).resolve()
    for path in sorted(directory.glob("*_review.png")):
        source = Image.open(path).convert("RGB")
        rows = source.height // 512
        result = Image.new("RGB", (source.width, source.height + rows * 32), "#20252a")
        draw = ImageDraw.Draw(result)
        name = path.stem.removesuffix("_review")
        labels = ["close", "native game2x"]
        if name == "work_shoulders4x":
            labels = ["right shoulder/elbow nearest4x", "left shoulder/elbow nearest4x"]
        if name.endswith("_side"):
            labels = ["side close"]
        for row in range(rows):
            result.paste(source.crop((0, row * 512, source.width, (row + 1) * 512)), (0, row * 544 + 32))
            for frame in range(8):
                draw.text((frame * 512 + 8, row * 544 + 8), f"{directory.name} / {name} / {labels[row]} / sample{frame}", fill="white")
        result.save(path.with_name(path.stem + "_labelled.png"))
    receipt = json.loads((directory / "review_receipt.json").read_text())
    for clip in sorted({sample["clip"] for sample in receipt["samples"]}):
        for view in ("close", "game2x", "side"):
            paths = [directory / f"{clip}_{index:02d}_{view}.png" for index in range(8)]
            if paths[0].is_file():
                assert all(path.is_file() for path in paths)
                grid(directory, f"{clip}_{view}", paths)
        if clip == "work":
            paths = [directory / f"work_{index:02d}_{side}_shoulder4x.png"
                     for side in ("L", "R") for index in range(8)]
            grid(directory, "work_shoulders4x", paths)


if __name__ == "__main__":
    main()
