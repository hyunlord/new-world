#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic", "scikit-image"]
# ///
# How to run: PYTHONPATH=. python -m art.pipeline_batch_a3.batch EVIDENCE_ROOT
"""Process complete generation records using the catalogue's count policy."""

from __future__ import annotations

import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import ClassVar, Final

from pydantic import AliasChoices, BaseModel, ConfigDict, Field, ValidationError

from art.pipeline_batch_a3.process import ExportRequest, export_record
from art.pipeline_batch_a3.report import report
from art.pipeline_struct.normalize import digest

FAMILIES: Final = (
    "food",
    "metal",
    "bronze",
    "iron",
    "tools_textiles",
    "crop_items",
    "misc",
    "variants",
)
ALIASES: Final = {"grain": "wild_grain"}
CLI_ARGC: Final = 2


class DiscoveryRecord(BaseModel):
    """Identify completed generations among pending requests and review files."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")
    id: str | None = None
    attempt: int = 1
    raw: Path | None = Field(
        default=None, validation_alias=AliasChoices("raw", "source", "path")
    )
    prompt: str = ""


class CatalogueItem(BaseModel):
    """Only the catalogue identity and count policy affect processing."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")
    id: str
    count_buckets: tuple[str, ...] = ()


class Catalogue(BaseModel):
    """Typed boundary for the research catalogue."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="ignore")
    rows: tuple[CatalogueItem, ...]


@dataclass(frozen=True, slots=True)
class BatchOutcome:
    """Trace every considered record, including pending or non-generation files."""

    record: str
    id: str | None
    attempt: int
    status: str
    detail: str


def run_batch(root: Path) -> int:
    """Export each complete unique attempt; preserve pending inputs untouched."""
    catalogue_path = root / "catalogue-research.json"
    catalogue = Catalogue.model_validate_json(catalogue_path.read_text())
    policies = {row.id: row for row in catalogue.rows}
    seen: dict[tuple[str, int], str] = {}
    outcomes: list[BatchOutcome] = []
    for family in FAMILIES:
        for path in sorted((root / family).rglob("*.json")):
            try:
                row = DiscoveryRecord.model_validate_json(path.read_text())
            except ValidationError as error:
                outcomes.append(BatchOutcome(str(path), None, 0, "SKIPPED", str(error)))
                continue
            if row.id is None or row.raw is None or not row.prompt.strip():
                outcomes.append(
                    BatchOutcome(
                        str(path),
                        row.id,
                        row.attempt,
                        "SKIPPED",
                        "not a complete raw generation record",
                    )
                )
                continue
            asset_id = ALIASES.get(row.id, row.id)
            raw = row.raw if row.raw.is_absolute() else path.parent / row.raw
            if not raw.is_file():
                outcomes.append(
                    BatchOutcome(
                        str(path), asset_id, row.attempt, "SKIPPED", "raw pending"
                    )
                )
                continue
            key = (asset_id, row.attempt)
            raw_hash = digest(raw)
            if key in seen:
                if seen[key] != raw_hash:
                    message = f"Conflicting generation records for {key}: {path}"
                    raise ValueError(message)
                outcomes.append(
                    BatchOutcome(
                        str(path),
                        asset_id,
                        row.attempt,
                        "SKIPPED",
                        "duplicate raw attempt",
                    )
                )
                continue
            seen[key] = raw_hash
            policy = policies.get(asset_id)
            result = export_record(
                ExportRequest(
                    record=path,
                    output=root / "icon_candidates",
                    counts=bool(policy and policy.count_buckets),
                    asset_id=asset_id,
                )
            )
            detail = (
                f"{len(result.exports)} exports; catalogue policy"
                if policy
                else "UNKNOWN_CATALOGUE_ID; single only"
            )
            outcomes.append(
                BatchOutcome(str(path), asset_id, row.attempt, result.status, detail)
            )
            print(f"{asset_id} try{row.attempt}: {result.status}; {detail}")
    summary = {
        "catalogue": str(catalogue_path),
        "catalogue_sha256": digest(catalogue_path),
        "aliases": ALIASES,
        "records": [asdict(row) for row in outcomes],
    }
    _ = (root / "icon_batch_summary.json").write_text(
        json.dumps(summary, indent=2) + "\n"
    )
    return report(root / "icon_candidates", root / "sheets/icon_batch")


def main() -> int:
    """Run the bounded A3 evidence inventory without changing source records."""
    if len(sys.argv) != CLI_ARGC:
        print("Usage: batch EVIDENCE_ROOT", file=sys.stderr)
        return 2
    return run_batch(Path(sys.argv[1]).resolve())


if __name__ == "__main__":
    raise SystemExit(main())
