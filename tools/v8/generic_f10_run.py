# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: python3 -B -m unittest discover -s tools/v8 -p test_generic_f10_run.py
"""Execute registered F10 pairs; no declarations or full-gate authority."""

import json
import re
import sys
from collections.abc import Callable
from dataclasses import asdict
from pathlib import Path
from typing import Final

from generic_f10 import companion_delta, injection, paired_precondition
from generic_fixture_registry import Invocation, Registry, parse_fixture
from generic_fixture_run import ComponentResult, Products
from generic_fixtures import ProbeProduct, fixed_observations, run_invocation
from generic_gate import document, integer, require, typed_json
from generic_observations import Observation, ordinary_pair
from generic_process import ProcessResult, ProcessSpec, atomic_json, execute, file_hash
from generic_sources import inventory
from mux2_gate_inputs import Json, Node
from read_summary_normalise import first_difference, normalise_pair

INJECTED: Final = "knowledge.f10_injected"
COMPANION: Final = "knowledge.f10_no_kill"
SCRIPT: Final = "tools/v8/m4_f10_script.json"
NO_KILL: Final = "tools/v8/gate_f10_no_kill.json"
SCORER: Final = "docs/v8/m4/m4_score.py"
BLOCKER: Final = "companion_exact_delta_and_hash_binding_required"
_normalise: Callable[[Json, Json, str | None], tuple[object, object, list[str]]] = (
    normalise_pair
)


def prepare(
    registry: Registry, plans: tuple[Invocation, ...], products: Products
) -> None:
    """Resolve only the registered companion obligation, before any child starts."""
    require(
        len(plans) == 4
        and {(p.arm, p.fixture_id) for p in plans}
        == {
            (arm, name)
            for arm in ("base", "candidate")
            for name in (INJECTED, COMPANION)
        },
        "/plans",
        "both injected and companion arms required exactly once",
    )
    require(
        len({p.id for p in plans}) == len({p.out for p in plans}) == 4,
        "/plans",
        "duplicate identity or output",
    )
    fixtures = {f.id: f for f in registry.fixtures}
    require(
        {INJECTED, COMPANION} <= fixtures.keys(), "/registry", "missing F10 fixtures"
    )
    for plan in plans:
        fixture = fixtures[plan.fixture_id]
        require(
            fixture == parse_fixture(document(fixture.metadata_json).value),
            plan.id,
            "fixture metadata differs",
        )
        require(
            fixture.seeds == plan.seeds == tuple(range(8))
            and fixture.roles == plan.roles == ("primary",)
            and fixture.repeats == ("1",)
            and plan.repeat == "1",
            plan.id,
            "changed F10 world coverage",
        )
        meta = document(fixture.metadata_json)
        require(
            integer(meta.field("horizon").field("days")) == 4
            and integer(meta.field("horizon").field("completed_ticks")) == 5760,
            plan.id,
            "changed F10 horizon",
        )
        product = products.arm(plan.arm)
        require(
            plan.id == f"{plan.arm}/{plan.fixture_id}/1"
            and plan.cwd == product.input_root,
            plan.id,
            "identity/input root mismatch",
        )
        blockers = (BLOCKER,) if plan.fixture_id == COMPANION else ()
        require(
            plan.blockers == blockers
            and plan.generated_input is None
            and not plan.environment,
            plan.id,
            "unexpected obligations, dependency or environment",
        )
        values = {
            "probe": str(product.executable),
            "inputs": str(product.input_root),
            "out": str(plan.out),
            "commands_out": str(plan.out / "commands"),
            "f10_companion": str(product.input_root / NO_KILL),
        }
        require(
            plan.argv
            == tuple(arg.format_map(values) for arg in fixture.argv)
            + ("--gate-observations",),
            plan.id,
            "changed registered argv or companion path",
        )
        inputs = dict(product.input_files)
        for name in (SCRIPT, NO_KILL, SCORER):
            require(
                name in inputs and file_hash(product.input_root / name) == inputs[name],
                plan.id,
                f"missing/changed authenticated input: {name}",
            )
        companion_delta(
            (product.input_root / SCRIPT).read_text(),
            (product.input_root / NO_KILL).read_text(),
        )
    require(
        all(
            dict(products.base.input_files)[name]
            == dict(products.candidate.input_files)[name]
            for name in (SCRIPT, NO_KILL, SCORER)
        ),
        "/inputs",
        "cross-arm fault/companion/oracle bytes differ",
    )


def _oracle(plan: Invocation, product: ProbeProduct) -> dict[str, Json]:
    """Retain the original scorer and its stdout oracle, including zero-exit FAIL."""
    python = Path(sys.executable).resolve(strict=True)
    summary = plan.out / "summary.json"
    result = execute(
        ProcessSpec(
            (str(python), str(product.input_root / SCORER), "--f10", str(summary)),
            product.input_root,
            python,
            file_hash(python),
            product.input_root,
            product.input_files,
            plan.out / "f10-scorer",
            external_inputs=((summary, file_hash(summary)),),
        )
    )
    stdout = (result.receipt.parent / "stdout.log").read_text()
    terminals = re.findall(r"^F10: .*$", stdout, re.MULTILINE)
    passed = result.status == "PASS" and terminals == ["F10: 8/8 PASS"]
    return {
        "invocation": plan.id,
        "status": "PASS" if passed else "FAIL",
        "receipt": str(result.receipt),
        "receipt_sha256": file_hash(result.receipt),
        "native_exit": result.native_exit,
        "terminal": terminals[0] if len(terminals) == 1 else None,
        "stdout_sha256": file_hash(result.receipt.parent / "stdout.log"),
        "scorer_sha256": file_hash(product.input_root / SCORER),
        "outputs": dict(
            inventory(
                result.receipt.parent, [p.name for p in result.receipt.parent.iterdir()]
            )
        ),
    }


def _summary(plan: Invocation) -> Node:
    summary = document((plan.out / "summary.json").read_text())
    summary.field("scene").equals("colony")
    summary.field("days").equals(4)
    rows = summary.field("seeds").array()
    require(
        tuple(integer(row.field("seed")) for row in rows) == plan.seeds,
        plan.id,
        "summary seed coverage differs",
    )
    for row in rows:
        row.field("days").equals(4)
    return summary


def run_registered(
    registry: Registry, plans: tuple[Invocation, ...], products: Products, output: Path
) -> ComponentResult:
    """Run four fresh children and retain native faults, ordinary safety and strict comparisons."""
    prepare(registry, plans, products)
    require(not output.exists(), str(output), "component receipt already exists")
    output.parent.mkdir(parents=True, exist_ok=True)
    results: dict[str, ProcessResult] = {}
    artifacts: dict[str, dict[str, str]] = {}
    executions: list[Json] = []
    errors: list[Json] = []
    oracles: list[Json] = []
    worlds: list[Json] = []
    comparisons: list[Json] = []
    for plan in plans:
        try:
            blockers = (BLOCKER,) if plan.fixture_id == COMPANION else ()
            result = run_invocation(
                plan, products.arm(plan.arm), (), resolved_blockers=blockers
            )
            results[plan.id] = result
            require(
                not (plan.out / "f10-scorer").exists(),
                plan.id,
                "producer wrote reserved scorer directory",
            )
            artifacts[plan.id] = inventory(
                plan.out, [p.name for p in plan.out.iterdir()]
            )
            executions.append(
                {
                    "id": plan.id,
                    "status": result.status,
                    "receipt": str(result.receipt),
                    "receipt_sha256": file_hash(result.receipt),
                }
            )
            require(result.status == "PASS", plan.id, "producer failed")
        except (OSError, ValueError) as error:
            errors.append({"invocation": plan.id, "error": str(error)})
    fixtures = {f.id: f for f in registry.fixtures}
    for name in (INJECTED, COMPANION):
        selected = [
            next(p for p in plans if p.fixture_id == name and p.arm == arm)
            for arm in ("base", "candidate")
        ]
        base, candidate = selected
        try:
            require(all(p.id in results for p in selected), name, "producer incomplete")
            observed = [
                fixed_observations(fixtures[name], p, results[p.id]) for p in selected
            ]
            summaries = [_summary(p) for p in selected]
            raw_a, raw_b, masked = _normalise(
                summaries[0].value, summaries[1].value, products.base.source_sha
            )
            difference = first_difference(typed_json(raw_a), typed_json(raw_b))
            comparisons.append(
                {
                    "fixture": name,
                    "semantic_equal": difference is None,
                    "difference": difference,
                    "excluded_paths": [path for path in masked],
                    "base_outputs": dict(
                        inventory(base.out, ["summary.json", "gate_observations"])
                    ),
                    "candidate_outputs": dict(
                        inventory(candidate.out, ["summary.json", "gate_observations"])
                    ),
                }
            )
            if difference is not None:
                errors.append(
                    {
                        "fixture": name,
                        "error": "undeclared semantic difference",
                        "difference": difference,
                    }
                )
            if name == INJECTED:
                for plan in selected:
                    try:
                        oracle = _oracle(plan, products.arm(plan.arm))
                        oracles.append(oracle)
                        require(
                            oracle["status"] == "PASS",
                            plan.id,
                            "original F10 scorer rejected",
                        )
                    except (OSError, ValueError) as error:
                        errors.append({"invocation": plan.id, "error": str(error)})
                for plan in selected:
                    expected = {f"colony_s{seed}_commands.json" for seed in plan.seeds}
                    directory = plan.out / "commands"
                    require(
                        directory.is_dir()
                        and {p.name for p in directory.iterdir()} == expected,
                        plan.id,
                        "missing/extra native command logs",
                    )
            for seed in base.seeds:
                row: dict[str, Json] = {
                    "fixture": name,
                    "seed": seed,
                    "instance": "primary",
                    "status": "FAIL",
                }
                worlds.append(row)
                try:
                    left, right = (arm[seed, "primary"] for arm in observed)
                    row.update(
                        base=document(json.dumps(asdict(left))).value,
                        candidate=document(json.dumps(asdict(right))).value,
                    )
                    safety: list[Observation] = []
                    for plan, actual, summary in zip(
                        selected, (left, right), summaries, strict=True
                    ):
                        if name == INJECTED:
                            command_path = (
                                plan.out / f"commands/colony_s{seed}_commands.json"
                            )
                            row[plan.arm + "_commands_sha256"] = file_hash(command_path)
                            safety.append(
                                injection(
                                    actual,
                                    (
                                        products.arm(plan.arm).input_root / SCRIPT
                                    ).read_text(),
                                    command_path.read_text(),
                                    json.dumps(
                                        summary.field("seeds").array()[seed].value
                                    ),
                                )
                            )
                        else:
                            safety.append(actual)
                    row["safety"] = (
                        paired_precondition(safety[0], safety[1])
                        if name == INJECTED
                        else ordinary_pair(safety[0], safety[1])
                    )
                    row["status"] = "PASS"
                except (OSError, ValueError) as error:
                    row["error"] = str(error)
                    errors.append({"fixture": name, "seed": seed, "error": str(error)})
        except (OSError, ValueError) as error:
            errors.append({"fixture": name, "error": str(error)})
    for plan in plans:
        if plan.id not in artifacts:
            continue
        try:
            current = inventory(
                plan.out, [p.name for p in plan.out.iterdir() if p.name != "f10-scorer"]
            )
            require(
                current == artifacts[plan.id],
                plan.id,
                "native artifacts changed during component",
            )
        except (OSError, ValueError) as error:
            errors.append({"postflight": plan.id, "error": str(error)})
    for oracle in oracles:
        try:
            oracle_node = Node(oracle)
            folder = Path(oracle_node.field("receipt").text()).parent
            require(
                inventory(folder, [p.name for p in folder.iterdir()])
                == oracle_node.field("outputs").value,
                str(folder),
                "scorer artifacts changed during component",
            )
        except (OSError, ValueError) as error:
            errors.append({"postflight": "scorer", "error": str(error)})
    for product in (products.base, products.candidate):
        try:
            require(
                file_hash(product.executable) == product.executable_sha256
                and inventory(
                    product.input_root, [name for name, _ in product.input_files]
                )
                == dict(product.input_files),
                product.source_sha,
                "product inputs or executable changed during component",
            )
        except (OSError, ValueError) as error:
            errors.append({"postflight": product.source_sha, "error": str(error)})
    status = "FAIL" if errors else "PASS"
    bindings: dict[str, Json] = {}
    for arm in ("base", "candidate"):
        product = products.arm(arm)
        bindings[arm] = {
            name: dict(product.input_files)[name] for name in (SCRIPT, NO_KILL, SCORER)
        }
    atomic_json(
        output,
        {
            "schema": "f10_component_v1",
            "status": status,
            "publication_authority": False,
            "registry_status": "INCOMPLETE_ADAPTERS",
            "registry_sha256": registry.sha256,
            "base": products.base.source_sha,
            "candidate": products.candidate.source_sha,
            "input_bindings": bindings,
            "producer_artifacts": {
                key: dict(value) for key, value in artifacts.items()
            },
            "companion_transform": "remove exactly registered tick1800 doorway kill only",
            "executions": executions,
            "comparisons": comparisons,
            "oracles": oracles,
            "worlds": worlds,
            "errors": errors,
        },
    )
    return ComponentResult(output, status)
