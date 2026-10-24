# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: python3 -m art.flora.provenance
"""Freeze every generated attempt and reference; count rejected calls as calls."""
from __future__ import annotations

import hashlib
from collections import Counter
from pathlib import Path
from typing import ClassVar, Final

from pydantic import BaseModel, ConfigDict, Field, JsonValue, TypeAdapter

ROOT: Final = Path(__file__).resolve().parents[2]
DATA: Final = Path('/Users/rexxa/work/arttrees-data')


class AttemptInput(BaseModel):
    """Original unabridged ledger fields remain preserved alongside typed inputs."""

    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra='allow')
    id: str
    attempt: int = Field(ge=1, le=3)
    source: Path
    generated_source: Path | None = None
    prompt: str
    reference: Path | None = None
    branch_reference: Path | None = None
    references: tuple[Path, ...] | None = None


class ReferencePolicy(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra='forbid')
    id: str
    attempt: int
    references: tuple[Path, ...]
    evidence: str


class HashedFile(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    path: str
    sha256: str
    preserved_path: str
    byte_count: int


class AttemptEvidence(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    id: str
    attempt: int
    generation_output_id: str
    source: HashedFile
    references: tuple[HashedFile, ...]
    reference_evidence: str
    original_record: dict[str, JsonValue]


class AttemptsEvidence(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True)
    schema_version: int = 1
    total_recorded_calls: int
    generation_budget: int = 120
    calls_by_asset: dict[str, int]
    maximum_attempt: int
    all_attempts_lte3: bool
    within_generation_budget: bool
    all_originals_preserved: bool
    ledger_inputs: tuple[HashedFile, ...]
    attempts: tuple[AttemptEvidence, ...]


def preserve(path: Path, destination: Path) -> HashedFile:
    """Copy bytes once, verifying both existing immutable copies and new copies."""
    content = path.read_bytes()
    digest = hashlib.sha256(content).hexdigest()
    frozen = destination / f'{path.stem}_{digest[:16]}{path.suffix}'
    if frozen.exists() and frozen.read_bytes() != content:
        message = f'Immutable provenance copy differs: {frozen}'
        raise ValueError(message)
    if not frozen.exists():
        _ = frozen.write_bytes(content)
    return HashedFile(path=str(path), sha256=digest, preserved_path=str(frozen), byte_count=len(content))


def references_for(row: AttemptInput, policies: tuple[ReferencePolicy, ...]) -> tuple[tuple[Path, ...], str]:
    """Require recorded attachments or explicit confirmation; do not infer from prose."""
    if row.references is not None:
        return row.references, 'Original per-call references list, preserved verbatim in original_record.'
    if row.reference is not None:
        refs = (row.reference, row.branch_reference) if row.branch_reference is not None else (row.reference,)
        return refs, 'Original per-call reference and optional branch_reference fields.'
    match = next((item for item in policies if item.id == row.id and item.attempt == row.attempt), None)
    if match is None:
        message = f'Actual attachment list is unrecorded: {row.id} attempt{row.attempt}'
        raise ValueError(message)
    return match.references, match.evidence


def validate_budget(rows: tuple[AttemptInput, ...]) -> Counter[str]:
    """Reject duplicate call slots, missing attempts, per-asset excess and total excess."""
    slots = {(row.id, row.attempt) for row in rows}
    count = Counter(row.id for row in rows)
    if len(slots) != len(rows) or len(rows) > 120 or any(value > 3 for value in count.values()):
        message = 'Generation budget or unique-call invariant failed'
        raise ValueError(message)
    for asset_id, calls in count.items():
        if {row.attempt for row in rows if row.id == asset_id} != set(range(1, calls + 1)):
            message = f'Attempt numbering has gaps: {asset_id}'
            raise ValueError(message)
    return count


def main() -> int:
    """Include core and optional bush ledgers; freeze original and reference inputs."""
    ledgers = tuple(path for path in (DATA / 'generation_attempts.json', DATA / 'bush_generation_attempts.json') if path.exists())
    rows = tuple(row for path in ledgers for row in TypeAdapter(tuple[AttemptInput, ...]).validate_json(path.read_bytes()))
    count = validate_budget(rows)
    policy_path = DATA / 'generation_reference_policy.json'
    policies = TypeAdapter(tuple[ReferencePolicy, ...]).validate_json(policy_path.read_bytes())
    frozen = DATA / 'all_attempts'
    frozen.mkdir(parents=True, exist_ok=True)
    entries: list[AttemptEvidence] = []
    for row in rows:
        refs, evidence = references_for(row, policies)
        if row.generated_source is not None and row.generated_source.read_bytes() != row.source.read_bytes():
            message = f"Preserved bush source differs from generated original: {row.source}"
            raise ValueError(message)
        entries.append(AttemptEvidence(id=row.id, attempt=row.attempt, generation_output_id=(row.generated_source or row.source).stem,
                                       source=preserve(row.source, frozen),
                                       references=tuple(preserve(path, frozen) for path in refs),
                                       reference_evidence=evidence, original_record=row.model_dump(mode='json', exclude_none=True)))
    record = AttemptsEvidence(total_recorded_calls=len(rows), calls_by_asset=dict(sorted(count.items())),
                              maximum_attempt=max(row.attempt for row in rows), all_attempts_lte3=True,
                              within_generation_budget=True, all_originals_preserved=True,
                              ledger_inputs=tuple(preserve(path, frozen) for path in (*ledgers, policy_path)),
                              attempts=tuple(entries))
    _ = (ROOT / 'docs/v8/arttrees/GENERATION_ATTEMPTS.json').write_text(record.model_dump_json(indent=2) + '\n')
    print(f'{len(rows)} recorded calls, max {record.maximum_attempt} per asset; all source/reference originals preserved')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
