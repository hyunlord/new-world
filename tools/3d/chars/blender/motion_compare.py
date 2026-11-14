"""Compose activity/rest before-after sheets without resizing source pixels.

python3 motion_compare.py BEFORE_REVIEW AFTER_REVIEW DESTINATION_DIRECTORY
"""
import sys
from pathlib import Path

from PIL import Image, ImageDraw


def main() -> None:
    before, after, destination = (Path(value).resolve() for value in sys.argv[1:])
    assert before.is_dir() and after.is_dir() and destination.is_dir()
    activity, rest = ("work", "sleep") if (after / "work_00_close.png").is_file() else ("graze", "lie_down")
    for view, filename in (("close", "motion_before_after.png"),
                           ("game2x", "motion_before_after_game2x.png")):
        output = destination / filename
        assert not output.exists(), f"Preserve existing evidence: {output}"
        sheet = Image.new("RGB", (4096, 2176), "#20252a")
        draw = ImageDraw.Draw(sheet)
        for row, (clip, directory, label) in enumerate([
                (activity, before, "BEFORE"), (activity, after, "AFTER"),
                (rest, before, "BEFORE"), (rest, after, "AFTER")]):
            for sample in range(8):
                source = Image.open(directory / f"{clip}_{sample:02d}_{view}.png").convert("RGB")
                assert source.size == (512, 512)
                x, y = sample * 512, row * 544
                sheet.paste(source, (x, y + 32))
                draw.text((x + 8, y + 8), f"{destination.name} {clip} {label} {view} sample{sample}", fill="white")
                assert sheet.crop((x, y + 32, x + 512, y + 544)).tobytes() == source.tobytes()
        sheet.save(output, optimize=True)
        assert output.stat().st_size < 5_000_000, output
        print(output, output.stat().st_size, "bytes; 32 native512 cells preserved")


if __name__ == "__main__":
    main()
