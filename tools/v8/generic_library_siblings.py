"""Closed library recipes and pure accounting; no build, dispatch or settlement authority."""
from dataclasses import dataclass
from enum import IntEnum
from pathlib import Path
from typing import Final, Literal

from gate_test_summary import TestSummary, parse_log, validate_cases
from generic_gate import require
from generic_workspace_cases import (
    Listing,
    ListingProcess,
    digest,
    lines,
    parse_listing,
    process_output,
)


class LibraryStep(IntEnum):
    BASELINE_ISOLATION = 42
    MIND_DATA_ONLY = 43
    READ_DATA_ONLY = 49


REGISTRATION: Final = (
    'registration_tests::world_registration_preserves_baseline_order_and_matches_membership',
    'registration_tests::world_membership_distinguishes_exclusive_systems',
    'registration_tests::system_worlds_document_matches_real_registration',
    'registration_tests::switch_production_membership_has_exactly_one_world_specific_brain',
)
MIND: Final = 'runtime::mind::system_tests::data_only_warm_at_hearth_reaches_real_target_and_retains_deciding_record'
DIAGNOSIS: Final = 'colony::diagnosis::tests::data_only_check_addition'


@dataclass(frozen=True, slots=True)
class Recipe:
    step: LibraryStep
    package: str
    target_name: str
    profile: Literal['debug', 'release']
    target_source: str
    filter: str
    exact: bool
    nocapture: bool
    expected: tuple[str, ...]
    source_classification: Literal['REGISTRATION_NO_ADVANCING_SIMULATION', 'CONTROLLED_COLONY_MECHANISM', 'DATA_SCHEMA_NO_SIMULATION']
    source_described_lit_branches: tuple[bool, ...]
    target_kind: Literal['lib'] = 'lib'


def recipe(step: LibraryStep) -> Recipe:
    step = LibraryStep(step)
    match step:
        case LibraryStep.BASELINE_ISOLATION:
            return Recipe(step, 'sim-systems', 'sim_systems', 'debug', 'rust/crates/sim-systems/src/lib.rs',
                          'registration_tests', False, True, REGISTRATION, 'REGISTRATION_NO_ADVANCING_SIMULATION', ())
        case LibraryStep.MIND_DATA_ONLY:
            return Recipe(step, 'sim-systems', 'sim_systems', 'debug', 'rust/crates/sim-systems/src/lib.rs',
                          MIND, True, False, (MIND,), 'CONTROLLED_COLONY_MECHANISM', (False, True))
        case LibraryStep.READ_DATA_ONLY:
            return Recipe(step, 'sim-core', 'sim_core', 'release', 'rust/crates/sim-core/src/lib.rs',
                          DIAGNOSIS, True, False, (DIAGNOSIS,), 'DATA_SCHEMA_NO_SIMULATION', ())


@dataclass(frozen=True, slots=True)
class PreparedRecipe:
    recipe: Recipe
    outer_argv: tuple[str, ...]
    outer_cwd: Path
    cargo_argv: tuple[str, ...]
    cargo_cwd: Path
    dispatch: Literal['NOT_RUN'] = 'NOT_RUN'
    source_build_admission: Literal['PENDING'] = 'PENDING'
    wrapper_admission: Literal['PENDING'] = 'PENDING'


def prepare(step: LibraryStep, root: Path, log: Path) -> PreparedRecipe:
    """Spell original commands only; paths/tool names are unqualified until owner admission."""
    require(root.is_absolute() and log.is_absolute() and not log.is_relative_to(root), '/recipe', 'external absolute log/root required')
    selected = recipe(step)
    match step:
        case LibraryStep.BASELINE_ISOLATION | LibraryStep.MIND_DATA_ONLY:
            mode = 'baseline_isolation' if step == LibraryStep.BASELINE_ISOLATION else 'data_only'
            args = ('--nocapture',) if selected.nocapture else ('--exact',)
            return PreparedRecipe(selected, ('bash', 'tools/v8/core_gate_checks.sh', mode, str(log)), root,
                                  ('cargo', 'test', '--manifest-path', 'rust/Cargo.toml', '-p', selected.package,
                                   '--lib', selected.filter, '--', *args), root)
        case LibraryStep.READ_DATA_ONLY:
            cargo = ('cargo', 'test', '--release', '-p', 'sim-core', '--lib', DIAGNOSIS, '--', '--exact')
            original = 'cd rust && cargo test --release -p sim-core --lib "'+DIAGNOSIS+'" -- --exact'
            return PreparedRecipe(selected, ('bash', '-c', original), root, cargo, root/'rust')


@dataclass(frozen=True, slots=True)
class ListingScope:
    """Caller-supplied byte expectations, explicitly not an authenticated library build."""
    executable: Path
    executable_sha256: str
    cwd: Path
    environment: tuple[tuple[str, str], ...]
    required_files: tuple[tuple[Path, str], ...]


@dataclass(frozen=True, slots=True)
class IndependentListings:
    full: ListingProcess
    ignored: ListingProcess


@dataclass(frozen=True, slots=True)
class ListingEvidence:
    full: bytes
    ignored: bytes
    receipt_hashes: tuple[str, str]
    source_build_admission: Literal['PENDING'] = 'PENDING'
    runtime_admission: Literal['PENDING'] = 'PENDING'
    libtest_semantics_admission: Literal['PENDING'] = 'PENDING'


def authenticate_listings(scope: ListingScope, bindings: IndependentListings) -> ListingEvidence:
    """Reuse strict process receipts; no native execution, owner settlement or build proof."""
    environment = dict(scope.environment)
    required = {str(path): digest for path, digest in scope.required_files}
    require(len(environment) == len(scope.environment) and len(required) == len(scope.required_files), '/listings', 'duplicate expectations')
    require(required.get(str(scope.executable), scope.executable_sha256) == scope.executable_sha256, '/listings', 'conflicting executable')
    required[str(scope.executable)] = scope.executable_sha256
    left, right = bindings.full.spec, bindings.ignored.spec
    require((left.executable, left.executable_sha256) == (right.executable, right.executable_sha256)
            == (scope.executable, scope.executable_sha256), '/listings', 'different executable')
    require((left.input_root, left.inputs, left.external_inputs) == (right.input_root, right.inputs, right.external_inputs),
            '/listings', 'independent listings changed inputs')
    require(left.output_root != right.output_root,
            '/listings', 'distinct listing processes/outputs required')
    raw: list[bytes] = []
    for binding, suffix in ((bindings.full, ('--list',)), (bindings.ignored, ('--list', '--ignored'))):
        require(binding.result.status == 'PASS' and binding.result.native_exit == 0, '/listings', 'listing failed')
        raw.append(process_output(binding, (str(scope.executable), *suffix), scope.cwd, environment, required))
    full, ignored = parse_listing(raw[0]), parse_listing(raw[1])
    _ = selected_inventory(full, ignored)
    for binding, suffix, original in ((bindings.full, ('--list',), raw[0]), (bindings.ignored, ('--list', '--ignored'), raw[1])):
        require(process_output(binding, (str(scope.executable), *suffix), scope.cwd, environment, required) == original,
                '/listings', 'listing changed during authentication')
    return ListingEvidence(raw[0], raw[1], (bindings.full.receipt_sha256, bindings.ignored.receipt_sha256))


def selected_inventory(full: Listing, ignored: Listing) -> frozenset[str]:
    require(not full.benchmarks and not ignored.benchmarks and set(ignored.names) <= set(full.names),
            '/listings', 'unsupported benchmarks or ignored inventory')
    return frozenset(full.names)


@dataclass(frozen=True, slots=True)
class SelectedEvidence:
    step: LibraryStep
    full_listing: bytes
    ignored_listing: bytes
    native_output: bytes
    native_exit: int | None


@dataclass(frozen=True, slots=True)
class SelectedAccounting:
    recipe: Recipe
    selected: tuple[str, ...]
    ignored_inventory: tuple[str, ...]
    filtered_out: int
    summary: TestSummary
    native_exit: int
    full_listing_sha256: str
    ignored_listing_sha256: str
    native_output_sha256: str
    accounting_status: Literal['PASS', 'FAIL']
    dispatch: Literal['NOT_RUN'] = 'NOT_RUN'
    source_build_admission: Literal['PENDING'] = 'PENDING'
    runtime_admission: Literal['PENDING'] = 'PENDING'
    libtest_semantics_admission: Literal['PENDING'] = 'PENDING'
    world_admission: Literal['INCOMPLETE'] = 'INCOMPLETE'
    branch_observation: Literal['NOT_RECORDED'] = 'NOT_RECORDED'
    settlement_authority: Literal[False] = False


def reconcile_selected(evidence: SelectedEvidence) -> SelectedAccounting:
    """Account raw bytes only; a caller must separately authenticate native capture and owner."""
    selected_recipe = recipe(evidence.step)
    full, ignored = parse_listing(evidence.full_listing), parse_listing(evidence.ignored_listing)
    names = selected_inventory(full, ignored)
    selected = tuple(name for name in full.names if (name == selected_recipe.filter if selected_recipe.exact else selected_recipe.filter in name))
    require(set(selected) == set(selected_recipe.expected), '/selection', 'closed expected selected set differs')
    require(not set(selected) & set(ignored.names), '/selection', 'selected ignored test cannot execute')
    records = lines(evidence.native_output)
    running = [row for row in records if row.startswith('running ')]
    expected_running = f'running {len(selected)} '+('test' if len(selected) == 1 else 'tests')
    require(running == [expected_running], '/native', 'missing/extra/different running section')
    require(type(evidence.native_exit) is int and evidence.native_exit >= 0, '/native', 'missing or signal outcome has no completed accounting')
    assert evidence.native_exit is not None
    summary = parse_log(evidence.native_output.decode('utf-8'))
    validate_cases(summary, selected_recipe.expected, ())
    filtered = len(names)-len(selected)
    require(summary.measured == 0 and summary.ignored == 0 and summary.filtered_out == filtered,
            '/native', 'measured/ignored/filtered accounting differs')
    require(summary.passed+summary.failed == len(selected), '/native', 'selected case count differs')
    require(evidence.native_exit != 0 or summary.failed == 0, '/native', 'successful exit contradicts failed tests')
    return SelectedAccounting(selected_recipe, selected, ignored.names, filtered, summary, evidence.native_exit,
                              full.sha256, ignored.sha256, digest(evidence.native_output), 'PASS' if evidence.native_exit == 0 and summary.failed == 0 else 'FAIL')
