# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic", "pillow", "numpy"]
# ///
# How to run: python3 -m art.pipeline_polish2.sprite_plan
"""Freeze the six-variant tables and palette targets before image generation."""
from __future__ import annotations

import hashlib
import json
from typing import Final

from pydantic import BaseModel, JsonValue, TypeAdapter

from art.compositor.face_variants import ROOT, FacePlan, LookVariants, Variant

SKIN: Final = {"S1": (224, 189, 142), "S2": (181, 141, 97), "S3": (113, 74, 47)}
HAIR: Final = {"H1": (44, 32, 20), "H2": (83, 66, 45), "H3": (139, 73, 43), "H4": (160, 155, 140)}
ADULT: Final = (
    ("S1", "H2", "young", ("narrow face", "freckles"), "clean"),
    ("S2", "H1", "adult", ("broad nose", "heavy brows"), "full"),
    ("S3", "H3", "adult", ("narrow face", "a scar"), "short"),
    ("S1", "H4", "older", ("round face", "freckles"), "full"),
    ("S2", "H3", "older", ("broad nose", "a scar"), "short"),
    ("S3", "H2", "young", ("round face", "heavy brows"), "clean"),
)
CHILD: Final = (
    ("S1", "H1", "child", ("round face", "freckles"), "none"),
    ("S2", "H2", "child", ("narrow face", "heavy brows"), "none"),
    ("S3", "H3", "child", ("broad nose", "freckles"), "none"),
    ("S1", "H3", "child", ("narrow face", "a scar"), "none"),
    ("S2", "H1", "child", ("broad nose", "heavy brows"), "none"),
    ("S3", "H2", "child", ("round face", "a scar"), "none"),
)


class AcceptedLook(BaseModel):
    look: str


class SourceManifest(BaseModel):
    accepted_looks: tuple[AcceptedLook, ...]
    fallbacks: dict[str, str]


def main() -> None:
    """Write immutable review plan; the source manifest decides source count."""
    path = ROOT / "art/character/route_i/MANIFEST.json"
    source = SourceManifest.model_validate_json(path.read_bytes())
    looks: list[LookVariants] = []
    for entry in source.accepted_looks:
        child = entry.look.startswith("child_")
        table = CHILD if child else ADULT
        variants: list[Variant] = []
        for number, (skin, hair, age, features, beard) in enumerate(table, 1):
            actual_beard = beard if entry.look.startswith("adult_M_") else "none"
            person = ("girl" if "_F_" in entry.look else "boy") if child else ("woman" if "_F_" in entry.look else "man")
            skin_text = {"S1": "light warm skin", "S2": "mid warm brown skin", "S3": "deep brown skin"}[skin]
            hair_text = {"H1": "black hair", "H2": "dark brown hair", "H3": "auburn hair", "H4": "grey hair"}[hair]
            description = f"{age} {person}; {skin_text}; {hair_text}; {', '.join(features)}"
            if age == "older":
                description += "; clearly aged 55-70, visible facial age lines"
            if actual_beard != "none":
                description += f"; {actual_beard} beard" if actual_beard != "clean" else "; clean shaven"
            variants.append(Variant.model_validate({"id": f"v{number}", "skin": skin, "hair": hair,
                "age_band": age, "features": features, "beard": actual_beard,
                "description": description, "skin_rgb": SKIN[skin], "hair_rgb": HAIR[hair]}))
        looks.append(LookVariants(look=entry.look, variants=tuple(variants)))
    # First two lexically sorted accepted looks for each age/sex stratum.
    sample = tuple(entry.look for prefix in ("adult_F_", "adult_M_", "child_F_", "child_M_")
                   for entry in sorted(looks, key=lambda item: item.look) if entry.look.startswith(prefix))
    owner_sample = tuple(name for prefix in ("adult_F_", "adult_M_", "child_F_", "child_M_")
                         for name in [n for n in sample if n.startswith(prefix)][:2])
    identity_sample = tuple(f"{owner_sample[index]}/v{variant}"
                            for index in (0, 1, 2, 4, 6, 7) for variant in range(1, 7))
    plan = FacePlan(schema_version=1, source_manifest=str(path.relative_to(ROOT)),
        source_manifest_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
        palette="art/style05/palette_05.json", rule="((agent_id * 2654435761) & 0xffffffff) % pool_size",
        looks=tuple(looks), fallbacks=source.fallbacks, owner_sample=owner_sample,
        identity_sample=identity_sample)
    _ = (ROOT / "art/character/FACE_VARIANTS.json").write_text(plan.model_dump_json(indent=2) + "\n")
    palette_path = ROOT / plan.palette
    palette = TypeAdapter(dict[str, JsonValue]).validate_json(palette_path.read_bytes())
    palette["character_variant_targets"] = {"schema_version": 1, "skin": {key: list(value) for key, value in SKIN.items()}, "hair": {key: list(value) for key, value in HAIR.items()},
        "purpose": "POLISH2 masked Lab tint targets; original judging palette remains unchanged"}
    _ = palette_path.write_text(json.dumps(palette, indent=2) + "\n")
    print(f"Frozen {len(looks)} unique looks x6; {len(source.fallbacks)} explicit aliases")


if __name__ == "__main__":
    main()
