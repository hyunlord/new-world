# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -B -m unittest discover -s tools/v8 -p test_generic_direct_run.py
"""Coordinate all registered direct fixtures, without full-gate authority."""

from collections.abc import Mapping
from pathlib import Path
from types import MappingProxyType
from typing import Final

import generic_baseline_run as baseline
import generic_f10_run as f10
import generic_fixture_run as ordinary
import generic_m6a_run as m6a
from generic_declarations import Declarations
from generic_fixture_registry import (
    BACKLOG_IDS,
    DIRECT_IDS,
    ExpansionContext,
    Invocation,
    Registry,
    expand,
)
from generic_fixture_run import ComponentResult, Products
from generic_gate import document, require
from generic_process import atomic_json, file_hash
from generic_sources import inventory
from mux2_gate_inputs import Json, Node

MODULES: Final = {"baseline": baseline, "ordinary": ordinary, "f10": f10, "m6a": m6a}
SCHEMAS: Final = {
    "baseline": "protected_baseline_component_v1",
    "ordinary": "ordinary_colony_component_v1",
    "f10": "f10_component_v1",
    "m6a": "m6a_component_v1",
}


# Imported by ordered planning and execution; unsupported steps stay in the map.
DIRECT_STEP_FIXTURES: Final[Mapping[int, tuple[str, ...]]] = MappingProxyType({
    10: ("identity.baseline_samples", "identity.baseline_twins"),
    11: ("determinism.baseline",),
    18: ("colony.u10",),
    19: ("replay.generate", "replay.recorded", "replay.scripted"),
    26: ("economy.soak_no_beasts",),
    28: ("knowledge.f10_injected", "knowledge.f10_no_kill"),
    29: ("story.script48",),
    30: ("neighbours.script48",),
    35: ("steward.on24_twins",),
    44: ("core.baseline10", "core.colony_u10", "core.colony_soak20", "core.steward_off",
         "core.steward_on", "m6a.lifecycle_off", "m6a.lifecycle_on"),
    47: ("m6a.food72",),
    48: ("m6a.food_data_only",),
    50: ("read.determinism", "read.soak20", "read.story48", "read.neighbours48",
         "read.steward_on24", "read.m6a_steward_on72"),
})
UNSUPPORTED_DIRECT_STEPS: Final = frozenset({47, 48})


def step_fixtures(step: int) -> tuple[str, ...]:
    """Return the closed original obligation sequence, never a caller-selected subset."""
    require(type(step) is int and step in DIRECT_STEP_FIXTURES, "/step", "unknown direct step")
    assigned = [fixture for fixtures in DIRECT_STEP_FIXTURES.values() for fixture in fixtures]
    require(len(assigned) == len(set(assigned)) and frozenset(assigned) == DIRECT_IDS,
            "/step-map", "missing, duplicate or extra direct fixture")
    return DIRECT_STEP_FIXTURES[step]


def partition(registry: Registry) -> dict[str, frozenset[str]]:
    """Reject gaps, extras and overlap instead of dropping unknown producers."""
    groups = {
        "baseline": baseline.SUPPORTED,
        "ordinary": ordinary.SUPPORTED,
        "f10": frozenset({f10.INJECTED, f10.COMPANION}),
        "m6a": frozenset({m6a.FIXED, m6a.SELECTED}),
    }
    fixture_ids = [fixture.id for fixture in registry.fixtures]
    require(
        len(fixture_ids) == len(DIRECT_IDS) and frozenset(fixture_ids) == DIRECT_IDS,
        "/registry",
        "missing, duplicate or extra direct fixture",
    )
    assigned = [name for ids in groups.values() for name in ids]
    require(
        len(assigned) == len(set(assigned)) and frozenset(assigned) == DIRECT_IDS,
        "/components",
        "component partition has overlap, missing or extra fixture",
    )
    metadata = document(registry.metadata_json)
    metadata.field("status").equals("INCOMPLETE_ADAPTERS")
    backlog = [document(raw) for raw in registry.backlog]
    require(
        len(backlog) == len(BACKLOG_IDS)
        and frozenset(row.field("id").text() for row in backlog) == BACKLOG_IDS,
        "/backlog",
        "missing, duplicate or extra unresolved family",
    )
    for row in backlog:
        row.field("status").equals("UNRESOLVED_ADAPTER_OR_ENUMERATION")
        row.field("runtime_status").equals("NOT_RUN")
    return groups


def _plans(
    registry: Registry,
    products: Products,
    output: Path,
    groups: dict[str, frozenset[str]],
) -> dict[str, tuple[Invocation, ...]]:
    """Expand independently per product root; preserve repeats, seed splits and twins."""
    plans: list[Invocation] = []
    for arm in ("base", "candidate"):
        product = products.arm(arm)
        context = ExpansionContext(
            product.input_root,
            product.input_root,
            output / "runs",
            products.base.executable,
            products.candidate.executable,
            product.input_root / f10.NO_KILL,
        )
        plans.extend(plan for plan in expand(registry, context) if plan.arm == arm)
    require(
        len({p.id for p in plans}) == len({p.out for p in plans}) == len(plans),
        "/plans",
        "duplicate expanded identity/output",
    )
    expected_count = sum(
        2 * len(f.repeats) * (len(f.seeds) if "{seed}" in f.argv else 1)
        for f in registry.fixtures
    )
    require(
        len(plans) == expected_count,
        "/plans",
        "registered invocation multiplicity changed",
    )
    return {
        name: tuple(plan for plan in plans if plan.fixture_id in ids)
        for name, ids in groups.items()
    }


def _prepare(name: str, registry: Registry, plans: tuple[Invocation, ...], products: Products) -> None:
    """Keep concrete typed adapter boundaries for whole paired fixture batches."""
    if name == "baseline":
        baseline.prepare(registry, plans, products)
    elif name == "ordinary":
        ordinary.prepare(registry, plans, products)
    elif name == "f10":
        f10.prepare(registry, plans, products)
    else:
        require(name == "m6a", "/component", "unknown component")
        m6a.prepare(registry, plans, products)


def _run(
    name: str,
    registry: Registry,
    plans: tuple[Invocation, ...],
    products: Products,
    declarations: Declarations,
    output: Path,
    step: int | None = None,
) -> ComponentResult:
    if name == "baseline":
        return baseline.run_registered(registry, plans, products, output)
    if name == "ordinary":
        if step == 50:
            return ordinary.run_preservation(registry, plans, products, declarations, output)
        return ordinary.run_registered(registry, plans, products, declarations, output)
    if name == "f10":
        return f10.run_registered(registry, plans, products, output)
    require(name == "m6a", "/component", "unknown component")
    return m6a.run_registered(registry, plans, products, declarations, output)


def _attest(
    name: str,
    result: ComponentResult,
    expected: Path,
    registry: Registry,
    products: Products,
    declarations: Declarations,
    row: dict[str, Json],
) -> None:
    require(result.receipt == expected, name, "foreign component receipt path")
    row["receipt_sha256"] = file_hash(expected)
    receipt = document(expected.read_text())
    receipt.field("schema").equals(SCHEMAS[name])
    receipt.field("registry_sha256").equals(registry.sha256)
    receipt.field("base").equals(products.base.source_sha)
    receipt.field("candidate").equals(products.candidate.source_sha)
    receipt.field("publication_authority").equals(False)
    status = receipt.field("status").text()
    row["original_receipt_status"] = status
    require(
        status in {"PASS", "FAIL"} and status == result.status,
        name,
        "returned and actual component status differ",
    )
    if name in {"ordinary", "m6a"}:
        receipt.field("declaration_manifest_sha256").equals(
            declarations.source_manifest_sha256
        )
    require(
        file_hash(expected) == row["receipt_sha256"],
        name,
        "component receipt changed during reading",
    )
    row["status"] = status


def _freeze_artifacts(name: str, receipt: Node, plans: tuple[Invocation, ...],
                      output: Path) -> dict[Path, dict[str, str]]:
    """Read only the four producer schemas; constrain every root to its registered plan."""
    paths = {plan.id: plan.out for plan in plans}
    if name == 'm6a':
        selected = next(p for p in plans if p.arm == 'base' and p.fixture_id == m6a.SELECTED)
        paths[selected.id + '/matched-companion'] = selected.out.parent / 'matched-companion'
    key = {'baseline': 'producer_outputs', 'ordinary': 'producer_outputs',
           'f10': 'producer_artifacts', 'm6a': 'artifacts'}[name]
    manifests = receipt.field(key).mapping()
    require(set(manifests) <= paths.keys(), name, 'foreign producer artifact identity')
    if receipt.field('status').value == 'PASS':
        require(paths.keys() <= manifests.keys(), name, 'missing producer artifact manifest')
    expected: dict[Path, Json] = {paths[identity]: raw for identity, raw in manifests.items()}
    if name == 'ordinary':
        allowed = {p.out / 'standing-scorer' for p in plans}
        for raw_path, hashes in receipt.field('scorer_outputs').mapping().items():
            folder = Path(raw_path)
            require(folder in allowed and folder not in expected, name, 'foreign or duplicate scorer artifact root')
            expected[folder] = hashes
    elif name == 'f10':
        for oracle in receipt.field('oracles').array():
            identity = oracle.field('invocation').text()
            require(identity in paths, name, 'foreign scorer invocation')
            folder = paths[identity] / 'f10-scorer'
            require(oracle.field('receipt').text() == str(folder / 'process.json') and folder not in expected,
                    name, 'foreign or duplicate scorer artifact root')
            expected[folder] = oracle.field('outputs').value
    elif name == 'm6a':
        for scorer in receipt.field('scorers').array():
            arm = scorer.field('arm').text()
            selected = next((p for p in plans if p.arm == arm and p.fixture_id == m6a.FIXED), None)
            require(selected is not None, name, 'foreign scorer arm')
            if selected is None:
                raise AssertionError('validated scorer arm')
            folder = selected.out.parent / 'standing-scorer'
            require(folder not in expected, name, 'duplicate scorer artifact root')
            for field, path in (('receipt', folder / 'process/process.json'), ('score', folder / 'score.json')):
                reference = scorer.field(field)
                require(reference.field('path').text() == str(path) and
                        reference.field('sha256').text() == file_hash(path), name, 'M6a scorer reference changed')
            expected[folder] = dict(inventory(folder, [p.name for p in folder.iterdir()]))
    frozen: dict[Path, dict[str, str]] = {}
    for folder, hashes in expected.items():
        require(folder.is_relative_to(output / 'runs') and folder == folder.resolve(strict=True),
                name, 'unsafe artifact root')
        actual = inventory(folder, [p.name for p in folder.iterdir() if p not in expected])
        require(actual == hashes, str(folder), 'component artifact manifest differs from actual files')
        # Include separately authenticated nested scorers in the terminal snapshot.
        frozen[folder] = inventory(folder, [p.name for p in folder.iterdir()])
    return frozen


def _run_batch(
    registry: Registry,
    products: Products,
    declarations: Declarations,
    freshoutput: Path,
    step: int | None,
) -> ComponentResult:
    """Shared execution with global preflight before any closed step selection."""
    require(
        freshoutput.is_absolute()
        and freshoutput == freshoutput.resolve()
        and not freshoutput.exists(),
        str(freshoutput),
        "fresh canonical external output required",
    )
    require(
        all(
            not freshoutput.is_relative_to(p.input_root)
            and not p.input_root.is_relative_to(freshoutput)
            for p in (products.base, products.candidate)
        ),
        str(freshoutput),
        "output overlaps product inputs",
    )
    freshoutput.mkdir(parents=True, exist_ok=False)
    required = step_fixtures(step) if step is not None else tuple(sorted(DIRECT_IDS))
    rows: dict[str, dict[str, Json]] = {
        name: {
            "component": name,
            "status": "NOT_RUN",
            "receipt": str(freshoutput / f"{name}.json"),
            "receipt_sha256": None,
            "invocations": [],
        }
        for name in MODULES
    }
    errors: list[Json] = []
    artifacts: dict[str, dict[Path, dict[str, str]]] = {}
    groups: dict[str, tuple[Invocation, ...]] = {}
    try:
        selected = partition(registry)
        require(
            declarations.base == products.base.source_sha
            and declarations.candidate == products.candidate.source_sha,
            "/declarations",
            "declaration sources differ from products",
        )
        require(
            DIRECT_IDS <= set(declarations.known_fixtures),
            "/declarations",
            "missing direct fixture declaration coverage",
        )
        groups = _plans(registry, products, freshoutput, selected)
        if step is not None:
            required_ids = frozenset(required)
            groups = {name: tuple(sorted((p for p in plans if p.fixture_id in required_ids),
                                         key=lambda p: (required.index(p.fixture_id), p.arm)))
                      for name, plans in groups.items() if required_ids.intersection(selected[name])}
            rows = {name: rows[name] for name in groups}
            require(frozenset(p.fixture_id for plans in groups.values() for p in plans) == required_ids,
                    "/step", "missing mapped fixture expansion")
        for name, plans in groups.items():
            rows[name]["invocations"] = [
                {
                    "id": p.id,
                    "fixture": p.fixture_id,
                    "arm": p.arm,
                    "repeat": p.repeat,
                    "seeds": list(p.seeds),
                    "roles": list(p.roles),
                    "cwd": str(p.cwd),
                    "out": str(p.out),
                    "argv": None if p.argv is None else list(p.argv),
                }
                for p in plans
            ]
        # All selected complete component batches validate before any producer starts.
        for name, plans in groups.items():
            if step not in UNSUPPORTED_DIRECT_STEPS:
                _prepare(name, registry, plans, products)
    except (OSError, ValueError) as error:
        errors.append({"phase": "preflight", "error": str(error)})
    if not errors:
        for name, plans in groups.items():
            row = rows[name]
            if step in UNSUPPORTED_DIRECT_STEPS:
                row["status"] = "NOT_IMPLEMENTED"
                row["reason"] = "Joint M6a adapter cannot authenticate separate original step occurrences"
                continue
            row["status"] = "FAIL"
            path = freshoutput / f"{name}.json"
            try:
                result = _run(name, registry, plans, products, declarations, path, step)
                _attest(name, result, path, registry, products, declarations, row)
                artifacts[name] = _freeze_artifacts(name, document(path.read_text()), plans, freshoutput)
                row['artifact_snapshots'] = {str(folder): dict(hashes) for folder, hashes in artifacts[name].items()}
                require(
                    result.status == "PASS",
                    name,
                    "component failed; original receipt retained",
                )
            except (OSError, ValueError) as error:
                row["status"] = "FAIL"
                row["error"] = str(error)
                # An exception may follow a written failure receipt; retain its actual bytes.
                if path.exists():
                    try:
                        row["receipt_sha256"] = file_hash(path)
                    except (OSError, ValueError) as hash_error:
                        row["receipt_hash_error"] = str(hash_error)
                try:
                    failed = _failed_artifacts(name, plans, freshoutput)
                    artifacts.setdefault(name, {}).update(failed)
                    row["failed_artifact_snapshots"] = {str(folder): dict(hashes) for folder, hashes in failed.items()}
                except (OSError, ValueError) as freeze_error:
                    row["artifact_freeze_error"] = str(freeze_error)
                errors.append({"component": name, "error": str(error)})
        for name, row in rows.items():
            if row["receipt_sha256"] is not None or name in artifacts:
                try:
                    require(
                        row["receipt_sha256"] is None or file_hash(freshoutput / f"{name}.json")
                        == row["receipt_sha256"],
                        name,
                        "component receipt changed during direct run",
                    )
                    for folder, hashes in artifacts.get(name, {}).items():
                        require(inventory(folder, [p.name for p in folder.iterdir()]) == hashes,
                                str(folder), 'component artifacts changed during direct run')
                except (OSError, ValueError) as error:
                    row["status"] = "FAIL"
                    row["postflight_error"] = str(error)
                    errors.append(
                        {"component": name, "phase": "postflight", "error": str(error)}
                    )
    status = "FAIL" if errors else "INCOMPLETE" if step in UNSUPPORTED_DIRECT_STEPS else "PASS"
    receipt = freshoutput / "receipt.json"
    atomic_json(
        receipt,
        {
            "schema": "direct_components_v1" if step is None else "direct_step_components_v1",
            **({} if step is None else {"step": step, "obligations": [f"{step}:{fixture}" for fixture in required],
                                       "fixtures": list(required), "whole_step_complete": False,
                                       "original_order_executed": False}),
            "status": status,
            "publication_authority": False,
            "full_gate_status": "NOT_QUALIFIED",
            "registry_status": "INCOMPLETE_ADAPTERS",
            "registry_sha256": registry.sha256,
            "base": products.base.source_sha,
            "candidate": products.candidate.source_sha,
            "declaration_manifest_sha256": declarations.source_manifest_sha256,
            "products": {
                arm: {
                    "root": str(products.arm(arm).input_root),
                    "executable": str(products.arm(arm).executable),
                    "executable_sha256": products.arm(arm).executable_sha256,
                }
                for arm in ("base", "candidate")
            },
            "components": [row for row in rows.values()],
            "errors": errors,
            "unresolved_backlog": [document(raw).value for raw in registry.backlog],
            "limit": "Direct registered fixture components only. Twelve backlog families remain unresolved; "
            + "build/source authentication and native full-gate qualification belong to the owner.",
        },
    )
    return ComponentResult(receipt, status)


def _failed_artifacts(name: str, plans: tuple[Invocation, ...], output: Path) -> dict[Path, dict[str, str]]:
    """Freeze actual failed outputs even if the producer omitted or never wrote its receipt."""
    folders = {plan.out for plan in plans}
    if name == "m6a":
        folders.update(plan.out.parent / "standing-scorer" for plan in plans if plan.fixture_id == m6a.FIXED)
        folders.update(plan.out.parent / "matched-companion" for plan in plans
                       if plan.arm == "base" and plan.fixture_id == m6a.SELECTED)
    result: dict[Path, dict[str, str]] = {}
    for folder in sorted(folders):
        if folder.exists() or folder.is_symlink():
            require(folder.is_relative_to(output / "runs") and folder == folder.resolve(strict=True),
                    str(folder), "unsafe failed artifact root")
            result[folder] = inventory(folder, [p.name for p in folder.iterdir()])
    return result


def run_all(registry: Registry, products: Products, declarations: Declarations, freshoutput: Path) -> ComponentResult:
    """Retain the complete development batch API and its original component receipt schema."""
    return _run_batch(registry, products, declarations, freshoutput, None)


def run_step(registry: Registry, products: Products, declarations: Declarations,
             step: int, freshoutput: Path) -> ComponentResult:
    """Run only the closed direct obligations at one step; never claim the whole gate step."""
    return _run_batch(registry, products, declarations, freshoutput, step)
