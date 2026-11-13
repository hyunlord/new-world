"""Write additive station import adapters and symbolic layered-template proposals."""

import json
from pathlib import Path
from typing import ClassVar

from pydantic import BaseModel, ConfigDict

from art.compositor.models import AssetEntry, AssetManifest


class Station(BaseModel):
    """Only published geometry fields cross the sidecar boundary."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")
    id: str
    canvas: tuple[int, int]
    anchor: tuple[int, int]
    footprint: tuple[int, int]


def write_drafts(output: Path, root: Path) -> None:
    """Keep source pinning explicit and preserve all unrelated importer rows."""
    entries: list[AssetEntry] = []
    files: list[dict[str, str]] = []
    for path in sorted((output / "stations").glob("*.png")):
        row = Station.model_validate_json(path.with_suffix(".json").read_bytes())
        source = str(path.relative_to(root))
        entries.append(
            AssetEntry(
                id=row.id,
                path=source,
                width=row.canvas[0],
                height=row.canvas[1],
                anchor=row.anchor,
                footprint=row.footprint,
                family="structure",
            )
        )
        files.append(
            {
                "role": "structure",
                "id": row.id,
                "source": source,
                "dest": f"structures/c3a/{row.id}.png",
                "from": "PIN_C3A_ASSET_COMMIT",
                "meta_source": "art/batch_c3a/workshops/stations_manifest.json",
            }
        )
    manifest = AssetManifest(assets=tuple(entries))
    _ = (output / "stations_manifest.json").write_text(
        manifest.model_dump_json(indent=2) + "\n"
    )
    draft = {
        "status": "DRAFT_NOT_APPLIED_REVIEW_REQUIRED",
        "source_branch": "v8/art-batch-c3a",
        "source_commit_requirement": "Replace PIN_C3A_ASSET_COMMIT with the immutable commit containing PNGs and metadata; require final visual review first.",
        "merge_instruction": "Add files; preserve existing config. Per-row from/meta_source require v8/visual-v3 importer capability.",
        "meta_sources": {"structure": "art/batch_c3a/workshops/stations_manifest.json"},
        "files": files,
        "layered_recipes": {
            family: {
                "idle": f"rooms/{family}_built/manifest.json",
                "working": f"rooms/{family}_built_working/manifest.json",
            }
            for family in ("smithy", "bloomery_house")
        },
    }
    _ = (output / "art_import.draft.json").write_text(
        json.dumps(draft, indent=2) + "\n"
    )
    bindings = {
        "status": "PROPOSED_NOT_APPLIED",
        "numeric_codes": "UNASSIGNED_DO_NOT_INVENT",
        "scales": {"item_scale": "N/A_station", "world_scale": 1},
        "stations": {
            row.id: {"structure": row.id, "era": "iron", "world_scale": 1}
            for row in manifest.assets
        },
        "templates": draft["layered_recipes"],
        "template_stages": "rooms/<family>_<foundation|half_built|damaged|ruined>/manifest.json",
        "blueprints": "blueprints.json",
        "limits": "Layered manifests are not accepted full-room sprite import rows; integration must preserve independent origins/anchors and depth order.",
    }
    _ = (output / "bindings.draft.json").write_text(
        json.dumps(bindings, indent=2) + "\n"
    )
