#!/usr/bin/env python3
"""Generate localization/ko/traits.json and localization/en/traits.json
from data/species/human/personality/trait_definitions.json."""

import json
import os

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)

TRAIT_DEF_PATH = os.path.join(PROJECT_ROOT, "data/species/human/personality/trait_definitions.json")
KO_DIR = os.path.join(PROJECT_ROOT, "localization/ko")
EN_DIR = os.path.join(PROJECT_ROOT, "localization/en")


def main():
    with open(TRAIT_DEF_PATH, "r", encoding="utf-8") as f:
        traits = json.load(f)

    ko_traits = {}
    en_traits = {}

    for t in traits:
        tid = "TRAIT_" + t["id"].upper()
        ko_traits[tid] = t.get("name_kr", t.get("name_en", t["id"]))
        en_traits[tid] = t.get("name_en", t["id"])
        desc_kr = t.get("description_kr", "")
        desc_en = t.get("description_en", "")
        if desc_kr:
            ko_traits[tid + "_DESC"] = desc_kr
        if desc_en:
            en_traits[tid + "_DESC"] = desc_en

    os.makedirs(KO_DIR, exist_ok=True)
    os.makedirs(EN_DIR, exist_ok=True)

    with open(os.path.join(KO_DIR, "traits.json"), "w", encoding="utf-8") as f:
        json.dump(ko_traits, f, ensure_ascii=False, indent=2)
    with open(os.path.join(EN_DIR, "traits.json"), "w", encoding="utf-8") as f:
        json.dump(en_traits, f, ensure_ascii=False, indent=2)

    print(f"Generated traits.json: {len(ko_traits)} ko entries, {len(en_traits)} en entries")


if __name__ == "__main__":
    main()
