"""Execute registered direct fixtures and inspect their native world observations.

The full-gate owner supplies authenticated build products and the global lane.
This module does not upgrade incomplete registry coverage into publication.
"""
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path

from generic_fixture_registry import Fixture, Invocation
from generic_gate import GateContractError, document, integer, require, typed_json
from generic_observations import Observation, ProbeWorld, ordinary_pair, parse_probe
from generic_process import ProcessResult, ProcessSpec, atomic_json, execute, file_hash
from generic_sources import inventory
from mux2_gate_inputs import Json
from read_summary_normalise import first_difference, normalise_pair

_normalise: Callable[[Json, Json, str | None], tuple[object, object, list[str]]] = normalise_pair


@dataclass(frozen=True)
class ProbeProduct:
    source_sha: str
    executable: Path
    executable_sha256: str
    # Includes selected runtime libraries/content, pinned by the build owner.
    input_root: Path
    input_files: tuple[tuple[str, str], ...]


def run_invocation(plan: Invocation, product: ProbeProduct, inputs: tuple[tuple[str, str], ...],
                   *, resolved_blockers: tuple[str, ...] = ()) -> ProcessResult:
    """Run one already-authenticated plan; preserve actual child failure receipts."""
    require(set(plan.blockers) == set(resolved_blockers), plan.id, "unresolved or unregistered adapter obligations")
    require(plan.argv is not None and plan.argv[0] == str(product.executable), plan.id, "wrong product executable")
    if plan.argv is None:
        raise AssertionError("validated argv")
    require(product.input_root == plan.cwd, plan.id, "effective input root must match producer working directory")
    require(bool(product.input_files), plan.id, "missing authenticated effective input inventory")
    merged = dict(product.input_files)
    for name, digest in inputs:
        require(name not in merged or merged[name] == digest, name, "conflicting runtime input binding")
        merged[name] = digest
    external_inputs: tuple[tuple[Path, str], ...] = ()
    if plan.generated_input is not None:
        require(plan.generated_input.sha256 is not None, plan.id, "generated replay input not authenticated")
        generated_sha = plan.generated_input.sha256
        if generated_sha is None:
            raise AssertionError("validated generated replay input digest")
        generated = plan.generated_input.path
        require(file_hash(generated) == plan.generated_input.sha256, plan.id, "generated replay input changed")
        if generated.is_relative_to(product.input_root):
            name = generated.relative_to(product.input_root).as_posix()
            require(name not in merged or merged[name] == plan.generated_input.sha256,
                    name, "conflicting generated input binding")
            merged[name] = generated_sha
        else:
            external_inputs = ((generated, generated_sha),)
    plan.out.parent.mkdir(parents=True, exist_ok=True)
    environment = dict(plan.environment)
    require(not set(environment) & {"WORLDSIM_GIT_SHA", "WORLDSIM_GATE_OBSERVER"}, plan.id, "reserved observer environment override")
    environment.update(WORLDSIM_GIT_SHA=product.source_sha, WORLDSIM_GATE_OBSERVER="1")
    return execute(ProcessSpec(plan.argv, plan.cwd, product.executable, product.executable_sha256,
                               product.input_root, tuple(sorted(merged.items())), plan.out,
                               tuple(sorted(environment.items())), external_inputs=external_inputs))


def fixed_observations(fixture: Fixture, plan: Invocation, result: ProcessResult) -> dict[tuple[int, str], Observation]:
    require(result.status == "PASS" and result.native_exit == 0, plan.id, "fixture producer did not complete successfully")
    require(result.receipt == plan.out / "process.json", plan.id, "foreign child receipt")
    metadata = document(fixture.metadata_json)
    require(fixture.id == plan.fixture_id, plan.id, "fixture and invocation differ")
    require(metadata.field("horizon").field("kind").value == "requested_days", plan.id, "event-conditioned adapter required")
    duration = integer(metadata.field("horizon").field("completed_ticks"))
    require(plan.argv is not None and "--scene" in plan.argv, plan.id, "fixed probe scene unavailable")
    if plan.argv is None:
        raise AssertionError("validated argv")
    scene = plan.argv[plan.argv.index("--scene") + 1]
    directory = plan.out / "gate_observations"
    expected = {f"{scene}_s{seed}_{role}.json" for seed in plan.seeds for role in plan.roles}
    require(directory.is_dir() and {path.name for path in directory.iterdir()} == expected,
            plan.id, "missing or extra native world sidecars")
    observations: dict[tuple[int, str], Observation] = {}
    for seed in plan.seeds:
        for role in plan.roles:
            path = directory / f"{scene}_s{seed}_{role}.json"
            require(path.is_file() and not path.is_symlink(), str(path), "unsafe observer output")
            observations[seed, role] = parse_probe(path.read_text(), ProbeWorld(scene, seed, role, duration))
    return observations


def compare_ordinary(fixture: Fixture, base: Invocation, candidate: Invocation,
                     base_result: ProcessResult, candidate_result: ProcessResult,
                     base_sha: str, output: Path) -> dict[str, Json]:
    """Strict no-declaration comparison; declaration admission is a separate owner step."""
    require(base.arm == "base" and candidate.arm == "candidate" and base.seeds == candidate.seeds
            and base.roles == candidate.roles and base.repeat == candidate.repeat,
            fixture.id, "unmatched arm occurrence plans")
    metadata = document(fixture.metadata_json)
    require(metadata.field("role").value in {"ordinary", "ordinary_observed"}, fixture.id, "dedicated fault/pack oracle required")
    left = fixed_observations(fixture, base, base_result)
    right = fixed_observations(fixture, candidate, candidate_result)
    require(left.keys() == right.keys(), fixture.id, "different world occurrence coverage")
    safety: list[Json] = []
    failures: list[Json] = []
    for key in left:
        row: dict[str, Json] = {"seed": key[0], "instance": key[1]}
        try:
            row["safety"] = ordinary_pair(left[key], right[key])
            row["status"] = "PASS"
        except GateContractError as error:
            row.update(status="FAIL", error=str(error))
            failures.append(str(error))
        safety.append(row)
    raw_a, raw_b, masked = _normalise(document((base.out / "summary.json").read_text()).value,
                                      document((candidate.out / "summary.json").read_text()).value, base_sha)
    a, b = typed_json(raw_a), typed_json(raw_b)
    difference = first_difference(a, b)
    result: dict[str, Json] = {"fixture_id": fixture.id, "base": base.id, "candidate": candidate.id,
                              "semantic_equal": difference is None, "difference": difference,
                              "excluded_paths": [path for path in masked], "safety": safety,
                              "safety_failures": failures,
                              "base_outputs": {key: value for key, value in inventory(base.out, ["summary.json", "gate_observations"]).items()},
                              "candidate_outputs": {key: value for key, value in inventory(candidate.out, ["summary.json", "gate_observations"]).items()},
                              "publication_authority": False}
    atomic_json(output, result)
    require(not failures, fixture.id, f"ordinary world safety rejected: {failures}")
    require(difference is None, fixture.id, f"undeclared semantic difference: {difference}")
    return result
