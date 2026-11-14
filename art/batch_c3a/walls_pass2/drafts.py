"""Additive importer and drystone capability proposals; never runtime configuration."""

import json
from pathlib import Path


def write_drafts(output: Path, approved: bool) -> None:
    """Propose only these wall files, preserving every existing material/default."""
    prefix = "art/batch_c3a/walls_pass2/outputs"
    names = [f"mask_{mask:02}" for mask in range(16)]
    names += [
        "end_n",
        "end_e",
        "end_s",
        "end_w",
        "corner_ne",
        "corner_se",
        "corner_nw",
        "corner_sw",
    ]
    names += [
        f"door_{orientation}_{state}"
        for orientation in ("ew", "ns")
        for state in ("closed", "open")
    ]
    draft = {
        "status": "VISUAL_APPROVED_DRAFT_NOT_APPLIED"
        if approved
        else "HOLD_REFERENCE_DOOR_GAP_NOT_APPLIED",
        "source_branch": "v8/art-batch-c3a",
        "source_commit": "PIN_C3A_PASS4_ASSET_COMMIT",
        "meta_sources": {"structure": "art/batch_c3a/workshops/stations_manifest.json"},
        "merge_instruction": "Append files only; retain existing config. Ground/item/structure metadata does not apply to wall_piece_texture. The importer eagerly reads this valid existing assets list but does not index it for this role.",
        "files": [
            {
                "role": "wall_piece_texture",
                "id": f"drystone_{name}",
                "source": f"{prefix}/{name}.png",
                "dest": f"materials/textures/drystone/pieces/{name}.png",
                "from": "PIN_C3A_PASS4_ASSET_COMMIT",
                "meta_source": "art/batch_c3a/workshops/stations_manifest.json",
            }
            for name in names
        ],
    }
    runtime_rows: list[str] = []
    patch = {
        "status": "HOLD_REFERENCE_DOOR_GAP_NOT_APPLIED",
        "target_style": "drystone",
        "material": "granite",
        "era_binding": "iron",
        "operation": "Set only styles[id=drystone].wall_pieces after importing the pinned rows; preserve default_styles, textures, material looks, palettes and all unrelated keys.",
        "wall_pieces": {
            "canvas": [64, 64],
            "anchor": [32, 59],
            "pieces": {str(mask): f"{prefix}/mask_{mask:02}.png" for mask in range(16)},
            "doors": {
                orientation: {
                    state: f"{prefix}/door_{orientation}_{state}.png"
                    for state in ("closed", "open")
                }
                for orientation in ("ew", "ns")
            },
        },
        "caps": {
            name: f"{prefix}/{name}.png"
            for name in names
            if name.startswith(("end_", "corner_"))
        },
        "caps_policy": "Mask pieces already include exposed ends and corners; aliases are review/integration references, never extra runtime overlays.",
        "runtime_rows_to_append": runtime_rows,
        "limits": "Visual art pass only. Game occupancy, chunk joins, tinting, construction-state and site-ghost parity require integration verification. Geometry matches shipped W2; old pass1 atlas remains held.",
    }
    for name, content in (
        ("art_import.draft.json", draft),
        ("construction_styles.patch.json", patch),
    ):
        _ = (output / name).write_text(json.dumps(content, indent=2) + "\n")
