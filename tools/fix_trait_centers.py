#!/usr/bin/env python3
"""Fix trait_defs_v2.json cond_center values for proper sigmoid calibration."""

import json
import sys
import os

INPUT_PATH = "data/personality/trait_defs_v2.json"
OUTPUT_PATH = INPUT_PATH  # in-place

def fix_centers(data: list) -> tuple[list, int]:
    """Apply center corrections. Returns (fixed_data, change_count)."""
    changes = 0

    for d in data:
        category = d.get("category", "")
        if category == "facet":
            continue  # facet traits use salience_center, not cond_center

        conditions = d.get("conditions", [])
        n_conds = len(conditions)

        for c in conditions:
            direction = c.get("direction", "high")
            old_center = c.get("cond_center")
            if old_center is None:
                continue

            new_center = old_center  # default: no change

            if direction == "low":
                if old_center == 0.85:
                    # Bug A: clearly wrong (HIGH value in LOW condition)
                    new_center = 0.35
                elif old_center >= 0.4:
                    if category == "dark":
                        new_center = 0.25
                    elif n_conds >= 3:
                        new_center = 0.30
                    else:
                        new_center = 0.35

            elif direction == "high":
                if old_center == 0.6:
                    if category == "dark":
                        new_center = 0.75
                    elif n_conds >= 3:
                        new_center = 0.70
                    else:
                        new_center = 0.65

            if new_center != old_center:
                print(f"  {d['id']:30s} facet={c['facet']:3s} dir={direction:4s} "
                      f"center: {old_center} -> {new_center}")
                c["cond_center"] = new_center
                changes += 1

    return data, changes


def main():
    if not os.path.exists(INPUT_PATH):
        print(f"ERROR: {INPUT_PATH} not found", file=sys.stderr)
        sys.exit(1)

    with open(INPUT_PATH, "r", encoding="utf-8") as f:
        data = json.load(f)

    print(f"Loaded {len(data)} trait definitions")
    print()
    print("Applying cond_center corrections:")

    data, changes = fix_centers(data)

    print()
    print(f"Total conditions changed: {changes}")

    if changes == 0:
        print("No changes needed.")
        return

    with open(OUTPUT_PATH, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2, ensure_ascii=False)
        f.write("\n")

    print(f"Written to {OUTPUT_PATH}")


if __name__ == "__main__":
    main()
