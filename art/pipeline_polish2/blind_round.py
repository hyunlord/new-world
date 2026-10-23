# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: imported by python3 -m art.pipeline_polish2.blind_packets ROSTER.
"""Freeze round inputs and bind independently launched review agents to evidence."""
from __future__ import annotations

from pathlib import Path

from pydantic import BaseModel

from art.compositor.face_variants import PLAN_PATH
from art.pipeline_polish2.blind_models import (
    PrivateKey,
    ReviewerStamp,
    ReviewRoster,
    RoundManifest,
)
from art.pipeline_polish2.portrait_models import AssetRef, CandidateSet


def seal(path: Path, payload: bytes) -> AssetRef:
    """Write immutable evidence, accepting an identical repeat but never replacement."""
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        if path.read_bytes() != payload:
            message = f"Conflicting frozen evidence: {path}"
            raise ValueError(message)
    else:
        with path.open("xb") as stream:
            _ = stream.write(payload)
    return AssetRef.of(path)


def seal_model(path: Path, value: BaseModel) -> AssetRef:
    return seal(path, (value.model_dump_json(indent=2) + "\n").encode())


def check_new_reviewers(current: ReviewRoster, previous: ReviewRoster | None) -> None:
    """A retry must use two actual agents disjoint from the first round."""
    if previous is None:
        if current.round != 0:
            message = "Round1 requires the original round0 reviewer evidence"
            raise ValueError(message)
        return
    if current.round != 1 or previous.round != 0:
        message = "Retry reviewer evidence must connect round1 to round0"
        raise ValueError(message)
    tasks = {row.agent.agent_task for row in previous.assignments}
    sessions = {row.agent.session_id for row in previous.assignments if row.agent.session_id is not None}
    if any(row.agent.agent_task in tasks or row.agent.session_id in sessions for row in current.assignments):
        message = "Round1 must use two NEW review agents/sessions, disjoint from round0"
        raise ValueError(message)


def copy_asset(source: AssetRef, directory: Path) -> AssetRef:
    source.verify()
    return seal(directory / f"{source.sha256}.png", source.path.read_bytes())


def freeze_round(base: Path, candidate_index: Path, roster: ReviewRoster) -> AssetRef:
    """Copy exact candidate/plan bytes before either reviewer receives any packet."""
    previous_ref = AssetRef.of(base.parent / "round0/private/round.json") if roster.round == 1 else None
    previous = RoundManifest.model_validate_json(previous_ref.path.read_bytes()) if previous_ref is not None else None
    check_new_reviewers(roster, previous.roster if previous is not None else None)
    for assignment in roster.assignments:
        assignment.agent.spawn_receipt.verify()
    source_index = AssetRef.of(candidate_index)
    candidates = CandidateSet.model_validate_json(candidate_index.read_bytes())
    private = base / "private"
    frozen = CandidateSet(candidates=tuple(row.model_copy(update={
        "portrait": copy_asset(row.portrait, private / "assets"),
        "sprite": copy_asset(row.sprite, private / "assets"),
    }) for row in candidates.candidates))
    manifest = RoundManifest(round=roster.round, source_index=source_index,
        source_snapshot=seal(private / "source_candidates.json", candidate_index.read_bytes()),
        frozen_candidates=seal_model(private / "frozen_candidates.json", frozen),
        source_plan=AssetRef.of(PLAN_PATH), frozen_plan=seal(private / "face_plan.json", PLAN_PATH.read_bytes()),
        roster=roster, previous_round=previous_ref)
    return seal_model(private / "round.json", manifest)


def verify_manifest(manifest: RoundManifest) -> CandidateSet:
    """Reject mutable source/index/plan or frozen-copy changes before another stage."""
    for reference in (manifest.source_index, manifest.source_snapshot, manifest.frozen_candidates,
                      manifest.source_plan, manifest.frozen_plan):
        reference.verify()
    source = CandidateSet.model_validate_json(manifest.source_snapshot.path.read_bytes())
    frozen = CandidateSet.model_validate_json(manifest.frozen_candidates.path.read_bytes())
    if tuple(row.key for row in source.candidates) != tuple(row.key for row in frozen.candidates):
        message = "Frozen candidate keys differ from the source snapshot"
        raise ValueError(message)
    for original, copy in zip(source.candidates, frozen.candidates, strict=True):
        for reference in (original.portrait, original.sprite, copy.portrait, copy.sprite):
            reference.verify()
        if original.portrait.sha256 != copy.portrait.sha256 or original.sprite.sha256 != copy.sprite.sha256:
            message = f"Frozen image differs from source: {original.key}"
            raise ValueError(message)
    for assignment in manifest.roster.assignments:
        assignment.agent.spawn_receipt.verify()
    previous = None
    if manifest.previous_round is not None:
        manifest.previous_round.verify()
        previous = RoundManifest.model_validate_json(manifest.previous_round.path.read_bytes())
    check_new_reviewers(manifest.roster, previous.roster if previous is not None else None)
    return frozen


def load_round(key: PrivateKey) -> tuple[RoundManifest, CandidateSet]:
    key.round_manifest.verify()
    manifest = RoundManifest.model_validate_json(key.round_manifest.path.read_bytes())
    if key.round != manifest.round or manifest.roster.round != manifest.round:
        message = "Private key round differs from its frozen round evidence"
        raise ValueError(message)
    assignment = next(row for row in manifest.roster.assignments if row.reviewer == key.reviewer)
    if assignment.agent != key.agent:
        message = "Private key agent differs from the frozen reviewer assignment"
        raise ValueError(message)
    return manifest, verify_manifest(manifest)


def check_response_agent(key: PrivateKey, response: ReviewerStamp) -> None:
    if (response.reviewer, response.agent_task, response.session_id) != (key.reviewer, key.agent.agent_task, key.agent.session_id):
        message = "Response was not produced by the assigned actual review agent/session"
        raise ValueError(message)
