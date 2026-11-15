"""Regenerate every 3D asset + manifest.json from scratch.

    blender -b --factory-startup -P tools/3d/blender/build_all.py

Then check the VAT outputs:  python3 tools/3d/blender/verify_vat.py
And review visually:         blender -b --factory-startup -P tools/3d/blender/render_contact.py
"""
import os
import sys
import time
import traceback

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402
import animals  # noqa: E402
import chars_b  # noqa: E402
import flora  # noqa: E402
import items  # noqa: E402
import people  # noqa: E402
import rocks  # noqa: E402
import stations  # noqa: E402

FAMILIES = [flora, rocks, stations, items, people, animals, chars_b]


def main():
    t0 = time.time()
    # Never deletes anything: every output file is overwritten in place, and the manifest is
    # rewritten (not merged) so models removed from the tables disappear from it.
    entries = {}
    for fam in FAMILIES:
        C.reset_scene()
        n = len(entries)
        fam.build(entries)
        print(f"[build_all] {fam.__name__}: {len(entries) - n} models")
    C.update_manifest(entries, replace=True)
    C.reset_scene(); __import__("stations_w3d").build()  # writes its own fragment, not manifest.json
    C.reset_scene(); __import__("chars_c").build()  # writes its own fragment, not manifest.json
    print(f"[build_all] {len(entries)} models -> {C.MANIFEST} in {time.time() - t0:.1f}s")


try:
    main()
except Exception:
    traceback.print_exc()
    sys.exit(1)  # make headless Blender return a failing exit code
