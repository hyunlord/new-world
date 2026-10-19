# /// script
# requires-python = ">=3.11"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/verify_carry_p6.py
"""Verify installed layers against final accepted board pixels, not only generator output."""
import json
from PIL import Image
from carry_boards_p6 import ROOT, installed_frames
from carry_candidate_round2_p6 import candidate, candidate_parts


def main() -> None:
    """Fail on a rejected group or any installed/source/review pixel mismatch."""
    directory = ROOT / "docs/art/production6/carry"
    manifest = json.loads((ROOT / "assets/art/atlas/carry_p6.json").read_text())
    approved = {row["group"] for row in json.loads((directory / "review_round2.json").read_text())["verdicts"] if row["verdict"] == "ok"}
    with Image.open(ROOT / manifest["body_page"].removeprefix("res://")) as body_atlas, Image.open(ROOT / manifest["prop_page"].removeprefix("res://")) as props:
        indexes: dict[str, int] = {}
        count = 0
        for frame in installed_frames():
            if frame.sprite_id not in manifest["sprites"]:
                continue
            entry = manifest["sprites"][frame.sprite_id]
            group = entry["group"]
            assert group in approved and frame.activity == "carry" and frame.body != "child"
            body, prop, anchor = candidate_parts(frame)
            x, y, width, height = entry["rect"]
            assert list(anchor) == entry["anchor"]
            assert body_atlas.crop(frame.box).tobytes() == body.tobytes()
            assert props.crop((x, y, x + width, y + height)).tobytes() == prop.tobytes()
            picture, _, _ = candidate(frame)
            index = indexes.get(group, 0)
            indexes[group] = index + 1
            resized = picture.resize((picture.width * 4, picture.height * 4), Image.Resampling.NEAREST)
            expected = Image.new("RGB", resized.size, "#73816e")
            expected.paste(resized, (0, 0), resized)
            left = 525 - resized.width // 2
            top = index * 160 + 40 + 126 - resized.height
            with Image.open(directory / "boards" / f"candidate_round2_{group}.png") as board:
                actual = board.crop((left, top, left + resized.width, top + resized.height)).convert("RGB")
                assert actual.tobytes() == expected.tobytes(), frame.sprite_id
            count += 1
    assert count == 20 and len(indexes) == 6
    print(f"CARRY_BYTE_BINDING PASS: {count} installed body/prop compositions equal accepted round2 board pixels; rejected wood absent")


if __name__ == "__main__":
    main()
