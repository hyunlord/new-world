# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic", "pillow", "numpy", "scipy", "typer"]
# ///
# How to run: python3 -m art.batch_b1.delivery draft --evidence /absolute/B1
"""Build unapplied import, identity and bindings proposals in catalog order."""

import json
from pathlib import Path
from typing import Final

from pydantic import JsonValue

from art.batch_b1.delivery_models import Catalog, JsonDocument
from art.compositor.face_variants import FacePlan, LookVariants

PENDING: Final = (
    "Runtime skin/hair masks and measured variant transforms for all new frames remain pending.",
    "Runtime job/era selector and top-code mapping remain pending; no simulation codes are assigned here.",
    "The pinned importer verify() assumes 36 logical looks and needs a reviewed runtime update.",
    "The appearance grid changes indices of old adult looks; bridge and renderer must move together.",
    "This is an unapplied candidate fragment, not game activation or a claim of runtime variant parity.",
)


def object_value(value: JsonValue) -> dict[str, JsonValue]:
    """Narrow JSON at the external config boundary."""
    if not isinstance(value, dict):
        raise TypeError("expected JSON object")
    return value


def strings(value: JsonValue) -> tuple[str, ...]:
    """Reject malformed look lists rather than coercing their entries."""
    if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
        raise ValueError("expected string list")
    return tuple(item for item in value if isinstance(item, str))


def context(
    root: Path, evidence: Path
) -> tuple[Catalog, dict[str, JsonValue], tuple[str, ...]]:
    catalog = Catalog.model_validate_json(
        (root / "art/batch_b1/CATALOG.json").read_bytes()
    )
    config = JsonDocument.model_validate_json(
        (evidence / "records/reference_art_import.json").read_bytes()
    ).root
    old = strings(object_value(config["people"])["looks"])
    logical = catalog.proposed_look_grid.logical_looks()
    actual = tuple(
        look for look in logical if look not in catalog.proposed_look_grid.fallbacks
    )
    if (
        len(old) != 34
        or len(logical) != 120
        or len(actual) != 118
        or set(actual) != set(old) | {look.id for look in catalog.looks}
        or object_value(config["look_grid"])["fallbacks"]
        != catalog.proposed_look_grid.fallbacks
    ):
        raise ValueError(
            "B1 grid must preserve 34 baseline looks and both old fallbacks"
        )
    return catalog, config, old


def draft_payloads(root: Path, evidence: Path) -> dict[Path, bytes]:
    """Build fragments only; preserve the live config and baseline face identities."""
    catalog, config, _ = context(root, evidence)
    grid = catalog.proposed_look_grid
    actual = tuple(look for look in grid.logical_looks() if look not in grid.fallbacks)
    people = dict(object_value(config["people"]))
    people.update(
        looks=list(actual),
        frame_source="art/character/batch_b1/{look}/{facing}/{pose}.png",
        portrait_source="art/portrait/batch_b1/{look}/{variant}.png",
    )
    fragment = {"people": people, "look_grid": grid.model_dump(mode="json")}
    bindings = JsonDocument.model_validate_json(
        (evidence / "records/reference_bindings.json").read_bytes()
    ).root
    binding_proposal = {
        "status": "PROPOSAL_NOT_RUNTIME_ACTIVATION",
        "reference_game_commit": catalog.reference_game_commit,
        "people": bindings["people"],
        "tool_state_selector_proposal": {
            "status": "pending_runtime_selector",
            "catalog": "art/batch_b1/tools_manifest.json",
            "source": "art/character/batch_b1_tools/{look}/{facing}/{tool_state}.png",
            "fallback": "Retain people.pose_for_state until a job/tool selector is implemented.",
            "job_code_mapping": None,
            "era_rule": "Use per-activity era overrides: wooden_bow is neolithic even in paleo_food.",
        },
        "top_code_mapping_proposal": {
            "status": "pending_review_no_new_simulation_codes",
            "reference": "rust/crates/sim-bridge/data/appearance.ron",
            "current_bridge_top_codes": {
                "0": 0,
                "1": 1,
                "2": 2,
                "3": 1,
                "4": 1,
                "5": 1,
                "6": 1,
            },
            "current_items": {
                "0": "loincloth",
                "1": "hide_wrap",
                "2": "fur_cloak",
                "3": "linen_tunic",
                "4": "linen_wrap",
                "5": "dyed_tunic",
                "6": "leather_armor",
            },
            "candidate_tops": [
                {
                    "id": top.id,
                    "grid_index": grid.tops.index(top.id),
                    "simulation_code": None,
                }
                for top in catalog.tops
            ],
        },
    }
    baseline = FacePlan.read(root / "art/character/FACE_VARIANTS.json")
    additions: list[LookVariants] = []
    for look in catalog.looks:
        source = baseline.for_look(f"{look.age}_{look.sex}_hide_wrap_{look.hair}")
        additions.append(source.model_copy(update={"look": look.id}))
    plan = baseline.model_copy(
        update={
            "looks": baseline.looks + tuple(additions),
            "fallbacks": dict(grid.fallbacks),
        }
    )
    # Validate the derived identity plan; this does not measure runtime tint transforms.
    _ = FacePlan.model_validate(plan.model_dump())
    directory = root / "art/batch_b1/drafts"
    note = [
        "# B1 import proposal — unapplied",
        "",
        *PENDING,
        "",
        "The fragment retains baseline people.data references. The extended FACE_VARIANTS.json",
        "is an identity plan copied from matching baseline age/sex/hair pools; it is not wired",
        "into the fragment until masks and measured transforms support the new frames.",
        "",
        "Apply only people and look_grid over the pinned full config; preserve every other section.",
        "Logical enumeration is age → sex → top → hair, exactly CATALOG.json order.",
        "Planned complete delivery: 120 logical IDs, 118 actual looks, 1180 frames, and 708 portraits. Counts are conditional on successful full assembly; they do not claim completed adoption.",
        "",
        "| index | logical ID | frame source look |",
        "| --- | --- | --- |",
    ]
    note.extend(
        f"| {index} | {look} | {grid.fallbacks.get(look, look)} |"
        for index, look in enumerate(grid.logical_looks())
    )
    return {
        directory / "people.fragment.json": (
            json.dumps(fragment, indent=2) + "\n"
        ).encode(),
        directory / "bindings.proposal.json": (
            json.dumps(binding_proposal, indent=2) + "\n"
        ).encode(),
        directory / "FACE_VARIANTS.json": (
            plan.model_dump_json(indent=2) + "\n"
        ).encode(),
        directory / "APPEARANCE_ORDER.md": ("\n".join(note) + "\n").encode(),
    }
