# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: python3 -m art.pipeline_polish2.sprite_build --diagnostic <revision> --review <review-json>
"""Record coverage and install complete reviewed variants with frozen transforms."""
from __future__ import annotations

import hashlib
from pathlib import Path
from typing import Final

import numpy as np
from PIL import Image, ImageDraw
from pydantic import JsonValue

from art.compositor.face_variants import ROOT, FacePlan
from art.pipeline_polish2.sprite_colour import (
    BoolPixels,
)
from art.pipeline_polish2.sprite_masks import Box, Masks

DATA: Final = ROOT.parent / "artpolish2-data/masks"
ROUTE: Final = ROOT / "art/character/route_i"
MASK_ROOT: Final = ROOT / "art/character/variant_masks"


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def coverage(mask: BoolPixels, box: Box) -> float:
    region = box.mask(mask.shape)
    return float(np.count_nonzero(mask & region) / max(1, np.count_nonzero(region)))


def box_record(box: Box) -> dict[str, JsonValue]:
    return {"left": box.left, "top": box.top, "right": box.right, "bottom": box.bottom}


def region_record(mask: Masks, box: Box) -> dict[str, JsonValue]:
    region = box.mask(mask.skin.shape)
    total = int(np.count_nonzero(region))
    alpha = int(np.count_nonzero(region & mask.alpha_excluded))
    ink = int(np.count_nonzero(region & mask.outline & ~mask.alpha_excluded))
    selected = int(np.count_nonzero(region & mask.skin))
    return {"box_pixels": total, "selected_skin_pixels": selected,
            "alpha_below_128_pixels": alpha, "protected_ink_pixels": ink,
            "unselected_visible_nonink_pixels": total - alpha - ink - selected,
            "maximum_coverage_preserving_alpha_boundary_and_ink": (total - alpha - ink) / max(1, total)}


def record(mask: Masks) -> dict[str, JsonValue]:
    """Record literal box denominators; never silently exclude failed pixels."""
    faces = coverage(mask.skin, mask.anatomy.face)
    arms = [coverage(mask.skin, box) for box in mask.anatomy.forearms]
    hair_fraction = float(np.count_nonzero(mask.hair & mask.hair_silhouette) / max(1, np.count_nonzero(mask.hair_silhouette)))
    classifier_fraction = float(np.count_nonzero(mask.hair & mask.classifier_hair_silhouette) / max(1, np.count_nonzero(mask.classifier_hair_silhouette)))
    expansion_ratio = float(np.count_nonzero(mask.hair) / max(1, np.count_nonzero(mask.classifier_hair_silhouette)))
    overlap = int(np.count_nonzero((mask.skin | mask.hair) & mask.excluded))
    head = mask.anatomy.head
    head_margin = Box(head.left - 4, head.top - 4, head.right + 4, head.bottom + 4).mask(mask.hair.shape)
    outside_head = int(np.count_nonzero(mask.hair & ~head_margin))
    return {"boxes": {"body": box_record(mask.anatomy.body), "head": box_record(mask.anatomy.head),
                "face": box_record(mask.anatomy.face), "forearms": [box_record(b) for b in mask.anatomy.forearms]},
            "skin_pixels": int(np.count_nonzero(mask.skin)), "source_local_skin_recovery_pixels": int(np.count_nonzero(mask.recovered_skin)),
            "source_local_hair_recovery_pixels": int(np.count_nonzero(mask.recovered_hair)), "hair_pixels": int(np.count_nonzero(mask.hair)),
            "face_region_counts": region_record(mask, mask.anatomy.face),
            "forearm_region_counts": [region_record(mask, box) for box in mask.anatomy.forearms],
            "face_box_coverage": faces, "forearm_box_coverage": [arms[0], arms[1]],
            "hair_source_proxy_coverage": hair_fraction, "classifier_hair_coverage": classifier_fraction,
            "expanded_hair_to_initial_cluster_ratio": expansion_ratio,
            "classifier_metric_status": "DIAGNOSTIC_ONLY_NOT_FORMAL_GATE",
            "hair_source_proxy_pixels": int(np.count_nonzero(mask.hair_silhouette)),
            "hair_source_proxy_outline_pixels": int(np.count_nonzero(mask.hair_silhouette & mask.outline)),
            "hair_outside_head_plus4_pixels": outside_head, "hair_head_containment_pass": outside_head == 0,
            "excluded_cluster_overlap": overlap,
            "face_pass": faces >= .60, "forearms_pass": all(v >= .60 for v in arms),
            "hair_source_proxy_pass": hair_fraction >= .50, "hair_formal_status": "AWAITING_SEMANTIC_AUDIT", "excluded_pass": overlap == 0}


def contact_sheet(plan: FacePlan) -> None:
    """External per-look sheets and eight-look overview for literal image review."""
    folder = DATA / "tinted_idles"
    folder.mkdir(parents=True, exist_ok=True)
    for look in plan.looks:
        sheet = Image.new("RGB", (6 * 200, 320), "#decba9")
        draw = ImageDraw.Draw(sheet)
        for index, variant in enumerate(look.variants):
            with Image.open(ROUTE / look.look / variant.id / "S/idle.png") as source:
                frame = source.convert("RGBA").resize((160, 224), Image.Resampling.NEAREST)
            sheet.paste(frame, (index * 200 + 20, 20), frame)
            draw.text((index * 200 + 8, 250), f"{variant.id} {variant.skin} {variant.hair} {variant.age_band}", fill="black")
            draw.text((index * 200 + 8, 270), variant.beard, fill="black")
        sheet.save(folder / f"{look.look}.png")


def main() -> None:
    """Integrate a complete reviewed diagnostic without recomputing frozen anchors."""
    from art.pipeline_polish2.sprite_integrate import main as integrate

    integrate()


if __name__ == "__main__":
    main()
