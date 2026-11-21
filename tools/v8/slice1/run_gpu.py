"""Capture and measure Slice 1 in one inherited GPU lane, with fresh output per attempt."""
import argparse
import json
import os
from pathlib import Path
import re
import statistics
import subprocess
import time

from PIL import Image, ImageDraw

SHOTS = (
    ("day", "z1"), ("day", "z4"), ("day", "low"), ("day", "forest"), ("day", "roof"),
    ("dusk", "z1"), ("dusk", "z4"), ("dusk", "low"),
)
ARMS = ("current", "slice")
FEATURES = ("pbr", "vegetation", "people", "sdfgi", "ssao", "ssil", "volumetric", "taa", "shadows", "sun", "resolution")
GODOT = os.environ.get("GODOT_BIN", "/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot")
PYTHON = "/opt/homebrew/bin/python3"


def run(out, label, args, marker):
    settings = out / "settings"
    settings.mkdir(exist_ok=True)
    command = ["bash", "tools/v8/run_bounded.sh", "360", GODOT, "--path", ".",
               "--windowed", "--resolution", "1920x1080",
               "--script", "res://tools/v8/slice1.gd", "--", *args]
    log = out / f"{label}.log"
    with log.open("x") as stream:
        result = subprocess.run(command, stdout=stream, stderr=subprocess.STDOUT,
                                env=dict(os.environ, WORLDSIM_TEST_SETTINGS_DIR=str(settings)),
                                check=False)
    text = log.read_text(errors="replace")
    if result.returncode or re.search(r"(?m)^(?:SCRIPT ERROR|ERROR:|Parse Error)", text):
        raise RuntimeError(f"Godot failed ({result.returncode}). See {log}")
    rows = [line.removeprefix(marker) for line in text.splitlines() if line.startswith(marker)]
    if not rows:
        raise RuntimeError(f"Missing {marker!r} in {log}")
    return rows


def capture(out, framing):
    images = out / "compare"
    for arm in ARMS:
        args = [*framing, "--mode=capture", f"--out={images}"]
        if arm == "slice":
            args.append("--quality=ultra")
        shots = run(out, f"capture-{arm}", args, "SLICE1_SHOT ")
        expected = [f"{arm}_{period}_{shot}.png" for period, shot in SHOTS]
        if shots != expected:
            raise RuntimeError(f"Missing or out-of-order captures for {arm}: {shots}")
    sheet = Image.new("RGB", (1296, len(SHOTS) * 390), "#171b1e")
    draw = ImageDraw.Draw(sheet)
    for y, (period, shot) in enumerate(SHOTS):
        for x, arm in enumerate(ARMS):
            with Image.open(images / f"{arm}_{period}_{shot}.png") as original:
                if original.size != (1920, 1080) or original.convert("RGB").getextrema() == ((0, 0),) * 3:
                    raise RuntimeError(f"Invalid capture {original.filename}")
                image = original.convert("RGB").resize((640, 360), Image.Resampling.LANCZOS)
            origin = (x * 648 + 4, y * 390)
            sheet.paste(image, (origin[0], origin[1] + 25))
            draw.text((origin[0] + 8, origin[1] + 5), f"{period.upper()} {shot.upper()}  |  {arm.upper()}", fill="#eee9dc")
    sheet.save(images / "contact-sheet.png")


def admit(out, label):
    with (out / "admission.jsonl").open("a") as stream:
        while True:
            load = os.getloadavg()[0]
            print(json.dumps({"label": label, "load_1m": load, "time": time.time()}),
                  file=stream, flush=True)
            if load < 8:
                return load
            time.sleep(30)


def measure(out, framing, stress, shot, omitted):
    mode = "stress" if stress else "slice"
    base = ["--stress=500"] if stress else framing
    base = [*base, "--mode=fps", f"--shot={shot}"]
    results = []
    for attempt, order in enumerate((("ultra", omitted), (omitted, "ultra")), 1):
        for group in order:
            label = f"{mode}-{shot}-{omitted}-{attempt}-{group}"
            load = admit(out, label)
            args = list(base)
            if group != "current":
                args.append("--quality=ultra")
            if group not in ("current", "ultra"):
                args.append(f"--without={group}")
            if stress:
                args.append("--live=1")
            records = run(out, label, args, "SLICE1_FPS ")
            if len(records) != 1:
                raise RuntimeError(f"Expected one measurement for {label}")
            row = json.loads(records[0])
            if row["size"] != [1920, 1080] or row["frames"] < 2 or row["fps"] <= 0:
                raise RuntimeError(f"Unusable measurement for {label}: {row}")
            if row["stress"] != stress or row["shot"] != shot or row["ultra"] != (group != "current"):
                raise RuntimeError(f"Mismatched measurement for {label}: {row}")
            if row["paused"] == stress or (stress and (row["rendered_people"] != 500 or row["end_tick"] <= row["tick"])):
                raise RuntimeError(f"Unexpected simulation mode for {label}: {row}")
            row.update(group=group, admitted_load=load, attempt=attempt)
            results.append(row)
            with (out / "fps.jsonl").open("a") as journal:
                print(json.dumps({"pair": omitted, "result": row}), file=journal)
    medians = {group: statistics.median(row["fps"] for row in results if row["group"] == group)
               for group in ("ultra", omitted)}
    return {"scene": mode, "shot": shot, "omitted": omitted, "fps": medians,
            "frame_cost_ms": 1000 / medians["ultra"] - 1000 / medians[omitted]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--fixture", type=Path, required=True, help="JSON with save, crop, focus and close")
    parser.add_argument("--capture-only", action="store_true")
    parser.add_argument("--fps-only", action="store_true")
    args = parser.parse_args()
    subprocess.run([PYTHON, "tools/v8/mux2_gate_lock.py", "--check"], check=True,
                   pass_fds=(int(os.environ["WORLDSIM_MUX2_LANE_FD"]),))
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    data = json.loads(args.fixture.read_text())
    framing = [f"--save={Path(data['save']).resolve()}"]
    for option, name in (("crop", "crop"), ("crop-size", "size"), ("focus", "focus"),
                         ("close", "close"), ("forest", "forest"), ("roof", "roof")):
        framing.append(f"--{option}=" + ",".join(map(str, data[name])))
    if not args.fps_only:
        capture(out, framing)
    if not args.capture_only:
        groups = [(False, "z1", "current"), (False, "z4", "current"), (True, "z1", "current")]
        groups.extend((False, "z1" if setting == "vegetation" else "z4", setting) for setting in FEATURES)
        results = [measure(out, framing, stress, shot, omitted) for stress, shot, omitted in groups]
        (out / "fps-summary.json").write_text(json.dumps(results, indent=2) + "\n")
    print("SLICE1_GPU_DONE", flush=True)


if __name__ == "__main__":
    main()
