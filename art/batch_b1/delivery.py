# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic", "pillow", "numpy", "scipy", "typer"]
# ///
# How to run: python3 -m art.batch_b1.delivery assemble --evidence /absolute/B1
"""Assemble reviewed candidates and an unapplied import handoff."""

from __future__ import annotations

from pathlib import Path
from typing import Annotated, Final

import typer

from art.batch_b1.delivery_assets import collect_new, plan_copies
from art.batch_b1.delivery_drafts import PENDING, context, draft_payloads
from art.batch_b1.delivery_models import CopyPair, FileRow, Inventory
from art.batch_b1.models import FACINGS, POSES, REPO
from art.batch_b1.portraits import VARIANTS


def write_preflighted(payloads: dict[Path, bytes]) -> None:
    """Check every output before writing; never replace a conflicting file."""
    for path, payload in payloads.items():
        if path.exists() and path.read_bytes() != payload:
            raise FileExistsError(path)
        for parent in path.parents:
            if parent.exists() and not parent.is_dir():
                raise NotADirectoryError(parent)
    written: list[Path] = []
    try:
        for path, payload in payloads.items():
            if not path.exists():
                path.parent.mkdir(parents=True, exist_ok=True)
                with path.open("xb") as stream:
                    _ = stream.write(payload)
                written.append(path)
    except BaseException:
        for path in reversed(written):
            path.unlink()
        raise


def assemble(root: Path, evidence: Path) -> Inventory:
    """Preflight all new art, then byte-copy 544 old PNGs into common roots."""
    catalog, _, old = context(root, evidence)
    rows = list(collect_new(root, catalog))  # Must finish before any baseline write.
    pairs: list[CopyPair] = []
    for look in old:
        for facing in FACINGS:
            for pose in POSES:
                suffix = f"{look}/{facing}/{pose}.png"
                pairs.append(
                    CopyPair(
                        source=root / "art/character/route_i" / suffix,
                        destination=root / "art/character/batch_b1" / suffix,
                    )
                )
        for variant in VARIANTS:
            suffix = f"{look}/{variant}.png"
            pairs.append(
                CopyPair(
                    source=root / "art/portrait/polish2" / suffix,
                    destination=root / "art/portrait/batch_b1" / suffix,
                )
            )
    copies = plan_copies(pairs)
    payloads = draft_payloads(root, evidence)
    for copy in copies:
        role = "portrait" if "portrait" in copy.destination.parts else "people_frame"
        look = copy.source.relative_to(
            root
            / (
                "art/portrait/polish2"
                if role == "portrait"
                else "art/character/route_i"
            )
        ).parts[0]
        rows.append(
            FileRow(
                source=copy.destination.relative_to(root).as_posix(),
                sha256=copy.sha256,
                role=role,
                look=look,
            )
        )
        payloads[copy.destination] = copy.source.read_bytes()
    logical = catalog.proposed_look_grid.logical_looks()
    inventory = Inventory(
        actual_looks=tuple(
            l for l in logical if l not in catalog.proposed_look_grid.fallbacks
        ),
        logical_looks=logical,
        files=tuple(rows),
        baseline_copies=copies,
        pending=PENDING,
    )
    inventory_path = evidence / "records/delivery_inventory.json"
    if inventory_path.exists():
        previous = Inventory.model_validate_json(inventory_path.read_bytes())
        transient = {"baseline_copies": {"__all__": {"existed"}}}
        if previous.model_dump(exclude=transient) != inventory.model_dump(
            exclude=transient
        ):
            raise FileExistsError(inventory_path)
        inventory = previous  # Keep the original pre-copy existence evidence on reruns.
    payloads[inventory_path] = (inventory.model_dump_json(indent=2) + "\n").encode()
    write_preflighted(payloads)
    return inventory


app: Final = typer.Typer()


@app.command("draft")
def draft_cli(evidence: Annotated[Path, typer.Option()], root: Path = REPO) -> None:
    """Write unapplied config fragments and appearance order without copying assets."""
    write_preflighted(draft_payloads(root, evidence))


@app.command("assemble")
def assemble_cli(evidence: Annotated[Path, typer.Option()], root: Path = REPO) -> None:
    """Assemble only after every new frameset and portrait family passes."""
    inventory = assemble(root, evidence)
    typer.echo(
        f"{len(inventory.actual_looks)} actual looks; {len(inventory.files)} PNGs"
    )


@app.command("audit")
def audit_cli(
    commit: str, evidence: Annotated[Path, typer.Option()], root: Path = REPO
) -> None:
    """Audit the pinned importer's plan against committed blobs, without game writes."""
    from art.batch_b1.delivery_audit import audit

    audit(commit, root, evidence)


if __name__ == "__main__":
    app()
