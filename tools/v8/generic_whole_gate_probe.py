# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -B -m unittest test_generic_whole_gate_observations
"""Fixed probe observation semantics over producer-completion file anchors."""
from dataclasses import dataclass
from pathlib import Path

from generic_fixture_registry import DIRECT_IDS, Invocation, Registry
from generic_gate import boundary, document, integer, require
from generic_observations import Observation, ProbeWorld, ordinary_pair, parse_probe
from generic_process import check_digest, file_hash, real_path
from mux2_gate_inputs import Json


@dataclass(frozen=True, slots=True)
class World:
    seed: int
    role: str
    observation: Observation


@dataclass(frozen=True, slots=True)
class ProbeEvidence:
    plan: Invocation
    registry_sha256: str
    worlds: tuple[World, ...]
    # Original digest anchors from producer completion, never recaptured authority.
    files: tuple[tuple[str, str], ...]
    directory_mode: int


def _check_files(root: Path, expected: tuple[tuple[str, str], ...], mode: int) -> None:
    real_path(root, directory=True)
    require(root.stat().st_mode == mode, str(root), 'observer directory mode changed')
    require(len(dict(expected)) == len(expected), str(root), 'duplicate output bindings')
    require({p.name for p in root.iterdir()} == dict(expected).keys(), str(root), 'missing or foreign observer output')
    for name, digest in expected:
        require(Path(name).name == name and name not in {'', '.', '..'}, name, 'unsafe sidecar name')
        check_digest(digest, name)
        path = root / name
        real_path(path, directory=False)
        require(file_hash(path) == digest, name, 'original producer bytes changed')


@boundary
def collect_probe(registry: Registry, plan: Invocation,
                  original_files: tuple[tuple[str, str], ...], original_mode: int) -> ProbeEvidence:
    """Parse complete fixed-horizon ordinary worlds after owner provenance binding.

    original_files names are relative to the original gate_observations directory;
    digest and directory mode anchors must originate at actual producer completion.
    This API does not claim that a digest-shaped argument establishes provenance.
    """
    require(plan.fixture_id in DIRECT_IDS - {'m6a.food72', 'm6a.food_data_only'}, plan.id, 'dedicated route required')
    fixtures = [f for f in registry.fixtures if f.id == plan.fixture_id]
    require(len(fixtures) == 1, plan.id, 'missing or duplicate fixture')
    fixture = fixtures[0]
    require(plan.arm in {'base', 'candidate'} and plan.repeat in fixture.repeats,
            plan.id, 'unknown arm/repeat')
    require(plan.seeds == fixture.seeds and plan.roles == fixture.roles, plan.id, 'changed occurrence coverage')
    require(plan.argv is not None and '--scene' in plan.argv, plan.id, 'missing scene')
    if plan.argv is None:
        raise AssertionError('validated argv')
    scene = plan.argv[plan.argv.index('--scene') + 1]
    require(scene == fixture.argv[fixture.argv.index('--scene') + 1], plan.id, 'foreign scene')
    horizon = document(fixture.metadata_json).field('horizon')
    horizon.field('kind').equals('requested_days')
    duration = integer(horizon.field('completed_ticks'))
    expected = {f'{scene}_s{s}_{r}.json' for s in fixture.seeds for r in fixture.roles}
    require(set(dict(original_files)) == expected, plan.id, 'missing/duplicate/foreign world binding')
    directory = plan.out / 'gate_observations'
    _check_files(directory, original_files, original_mode)
    worlds = tuple(World(seed, role, parse_probe(
        (directory / f'{scene}_s{seed}_{role}.json').read_text(),
        ProbeWorld(scene, seed, role, duration))) for seed in fixture.seeds for role in fixture.roles)
    _check_files(directory, original_files, original_mode)
    return ProbeEvidence(plan, registry.sha256, worlds, original_files, original_mode)


def recheck_probe(evidence: ProbeEvidence) -> None:
    """Keep the original observation byte closure through later evaluation."""
    _check_files(evidence.plan.out / 'gate_observations', evidence.files, evidence.directory_mode)


def probe_safety(base: ProbeEvidence, candidate: ProbeEvidence) -> tuple[Json, ...]:
    """Existing strict safety: candidate loss fails, including BASE-as-candidate self-proof."""
    left, right = base.plan, candidate.plan
    require(left.arm == 'base' and right.arm == 'candidate', '/pair', 'wrong arms')
    require((left.fixture_id, left.repeat, left.seeds, left.roles, base.registry_sha256) ==
            (right.fixture_id, right.repeat, right.seeds, right.roles, candidate.registry_sha256),
            '/pair', 'foreign occurrence pair')
    require(left.out != right.out, '/pair', 'shared arm output')
    require(tuple((w.seed, w.role) for w in base.worlds) ==
            tuple((s, r) for s in left.seeds for r in left.roles), '/pair', 'BASE coverage changed')
    require(tuple((w.seed, w.role) for w in candidate.worlds) ==
            tuple((s, r) for s in right.seeds for r in right.roles), '/pair', 'candidate coverage changed')
    recheck_probe(base)
    recheck_probe(candidate)
    rows: list[Json] = []
    for a, b in zip(base.worlds, candidate.worlds):
        # Reparse original bound bytes: a caller-replaced Observation is not evidence.
        for evidence, world in ((base, a), (candidate, b)):
            directory = evidence.plan.out / 'gate_observations'
            argv = evidence.plan.argv
            require(argv is not None, '/pair', 'missing original argv')
            if argv is None:
                raise AssertionError('validated argv')
            scene = argv[argv.index('--scene') + 1]
            actual = parse_probe((directory / f'{scene}_s{world.seed}_{world.role}.json').read_text(),
                                 ProbeWorld(scene, world.seed, world.role, world.observation.terminal_tick))
            require(actual == world.observation, '/pair', 'observation differs from original bytes')
        rows.append(ordinary_pair(a.observation, b.observation))
    recheck_probe(base)
    recheck_probe(candidate)
    return tuple(rows)
