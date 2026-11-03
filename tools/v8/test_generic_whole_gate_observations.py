"""Tiny native-schema fixtures; no native simulation or execution authority."""
import json
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path

import generic_whole_gate_observations as subject
from generic_fixture_registry import (
    ExpansionContext,
    Invocation,
    Registry,
    expand,
    load_registry,
)
from generic_gate import GateContractError, document, integer
from generic_process import ProcessSpec, execute, file_hash
from mux2_gate_inputs import Node
from test_generic_observations import receipt


class ProbeRoutingTests(unittest.TestCase):
    def fixture(self, root: Path) -> tuple[Registry, tuple[Invocation, ...]]:
        registry = load_registry(Path(__file__).parent / 'gate_fixtures.json')
        plans = tuple(p for p in expand(registry, ExpansionContext(root, root, root / 'out',
                            root / 'base-probe', root / 'candidate-probe')) if p.fixture_id == 'colony.u10')
        for plan in plans:
            folder = plan.out / 'gate_observations'
            folder.mkdir(parents=True)
            fixture = next(f for f in registry.fixtures if f.id == plan.fixture_id)
            ticks = integer(document(fixture.metadata_json).field('horizon').field('completed_ticks'))
            for seed in plan.seeds:
                for role in plan.roles:
                    raw = receipt()
                    raw.update(seed=seed, occurrence=role, requested_ticks=ticks,
                               requested_terminal_tick=ticks, terminal_tick=ticks)
                    _ = (folder / f'colony_s{seed}_{role}.json').write_text(json.dumps(raw))
        return registry, plans

    def collect(self, registry: Registry, plan: Invocation) -> subject.ProbeEvidence:
        folder = plan.out / 'gate_observations'
        files = tuple((p.name, file_hash(p)) for p in sorted(folder.iterdir()))
        return subject.collect_probe(registry, plan, files, folder.stat().st_mode)

    def test_all_registered_twins_are_parsed_and_paired(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            registry, plans = self.fixture(Path(tmp).resolve())
            left, right = (self.collect(registry, p) for p in plans)
            self.assertEqual(len(subject.probe_safety(left, right)), 8)
            table = subject.routes(registry)
            self.assertEqual(len(table), 40)
            self.assertEqual(sum(r.supported for r in table), 40)
            self.assertEqual({r.fixture for r in table if not r.supported},
                             set())
            play = next(row for row in table if row.fixture == 'ui.play')
            self.assertTrue(play.supported)
            self.assertIsNotNone(play.missing)

    def test_missing_foreign_duplicate_partial_and_forged_census_reject(self) -> None:
        for mutation in ('missing', 'foreign', 'duplicate', 'partial', 'founders', 'symlink'):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as tmp:
                registry, plans = self.fixture(Path(tmp).resolve())
                plan = plans[0]
                folder = plan.out / 'gate_observations'
                path = folder / 'colony_s0_primary.json'
                files = tuple((p.name, file_hash(p)) for p in sorted(folder.iterdir()))
                if mutation == 'missing':
                    path.unlink()
                elif mutation == 'foreign':
                    _ = (folder / 'foreign.json').write_text('{}')
                elif mutation == 'duplicate':
                    files += files[:1]
                elif mutation == 'symlink':
                    saved = folder.parent / 'saved.json'
                    _ = path.rename(saved)
                    path.symlink_to(saved)
                else:
                    raw = receipt()
                    raw.update(requested_ticks=14400, requested_terminal_tick=14400, terminal_tick=14400)
                    if mutation == 'partial':
                        raw['history_complete'] = False
                    else:
                        del raw['initial_population']
                    _ = path.write_text(json.dumps(raw))
                    files = tuple((p.name, file_hash(p)) for p in sorted(folder.iterdir()))
                with self.assertRaises((GateContractError, ValueError)):
                    _ = subject.collect_probe(registry, plan, files, folder.stat().st_mode)

    def test_loss_reference_allowed_but_self_proof_and_candidate_loss_fail(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            registry, plans = self.fixture(Path(tmp).resolve())
            path = plans[0].out / 'gate_observations/colony_s0_primary.json'
            raw = receipt()
            raw.update(requested_ticks=14400, requested_terminal_tick=14400, terminal_tick=14400,
                       knowledge_lost=[{'tick': 2, 'node': 'clay', 'holder': 9}])
            _ = path.write_text(json.dumps(raw))
            left, right = (self.collect(registry, p) for p in plans)
            self.assertEqual(len(subject.probe_safety(left, right)), 8)
            _ = (plans[1].out / 'gate_observations/colony_s0_primary.json').write_text(json.dumps(raw))
            right = self.collect(registry, plans[1])
            with self.assertRaisesRegex(GateContractError, 'KnowledgeLost'):
                _ = subject.probe_safety(left, right)

    def test_late_bytes_and_directory_mode_drift_reject(self) -> None:
        for mode in (False, True):
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as tmp:
                registry, plans = self.fixture(Path(tmp).resolve())
                evidence = self.collect(registry, plans[0])
                folder = plans[0].out / 'gate_observations'
                if mode:
                    folder.chmod(0o700)
                else:
                    _ = (folder / 'colony_s0_primary.json').write_text('{}')
                with self.assertRaises(GateContractError):
                    subject.recheck_probe(evidence)

    def test_changed_occurrences_or_replaced_observation_cannot_pass(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            registry, plans = self.fixture(Path(tmp).resolve())
            with self.assertRaises(GateContractError):
                _ = self.collect(registry, replace(plans[0], roles=('primary',)))
            left, right = (self.collect(registry, p) for p in plans)
            world = right.worlds[0]
            forged = replace(world, observation=replace(world.observation, alive=999))
            with self.assertRaises(GateContractError):
                _ = subject.probe_safety(left, replace(right, worlds=(forged, *right.worlds[1:])))


class WholeEvaluationTests(unittest.TestCase):
    def session(self, root: Path) -> Node:
        registry = Path(__file__).parent / 'gate_fixtures.json'
        return document(json.dumps({'output': str(root), 'environment': {},
            'registry': {'path': str(registry), 'sha256': file_hash(registry)},
            'observations': {'probes': []}, 'arms': {}}))

    def test_route_uses_existing_environment_without_world_launch(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve()
            session = self.session(root)
            self.assertEqual(subject.route(session, 1, root / 'step1'), {})
            env = subject.route(session, 9, root / 'step9')
            self.assertEqual(env['WORLDSIM_GATE_OBSERVER'], '1')
            self.assertEqual(env['WORLDSIM_GATE_UI_BOOT'], '1')
            self.assertFalse((root / 'step9').exists())
            with self.assertRaises(GateContractError):
                _ = subject.route(session, 52, root / 'step52')
            with self.assertRaises(GateContractError):
                _ = subject.route(session, 9, root / '../escape')

    def test_missing_journal_and_routes_never_pass(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve()
            result = subject.evaluate(self.session(root), (), root / 'evaluation')
            self.assertEqual(result['status'], 'INCOMPLETE')
            self.assertFalse(result['publication_authority'])
            self.assertIn('NOT_ADMITTED', str(result['stall_waiver']))

    def test_actual_aggregate_nonzero_and_changed_log_stay_fail(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve()
            tool = Path(sys.executable).resolve()
            spec = ProcessSpec((str(tool), '-c', 'raise SystemExit(1)'), root, tool,
                               file_hash(tool), root, (), root / 'process')
            result = execute(spec)
            raw = document(result.receipt.read_text())
            stage = document(json.dumps({'step': 1, 'arm': 'candidate', 'state': 'RETURNED',
                'process_receipt': str(result.receipt),
                'process_receipt_sha256': file_hash(result.receipt),
                'pid': result.pid, 'native_exit': result.native_exit,
                'cwd': str(root), 'environment': raw.field('environment').value}))
            evaluation = subject.evaluate(self.session(root), (stage,), root / 'evaluation')
            self.assertEqual(evaluation['status'], 'FAIL')
            self.assertTrue(evaluation['errors'])

    def test_stall_exit_one_is_settled_but_never_waived_by_missing_routes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve()
            tool = Path(sys.executable).resolve()
            result = execute(ProcessSpec((str(tool), '-c', 'raise SystemExit(1)'), root,
                tool, file_hash(tool), root, (), root / 'stall-process'))
            raw = document(result.receipt.read_text())
            value = {'step': 41, 'arm': 'candidate', 'state': 'RETURNED',
                'process_receipt': str(result.receipt),
                'process_receipt_sha256': file_hash(result.receipt),
                'pid': result.pid, 'native_exit': result.native_exit,
                'cwd': str(root), 'environment': raw.field('environment').value}
            stage = document(json.dumps(value))
            self.assertEqual(subject.authenticate_stage(stage, self.session(root)), 41)
            report = subject.evaluate(self.session(root), (stage,), root / 'evaluation')
            self.assertEqual(report['status'], 'FAIL')
            self.assertEqual(report['waived_stage_exits'], [])
            value['step'] = 40
            with self.assertRaises(GateContractError):
                _ = subject.authenticate_stage(document(json.dumps(value)), self.session(root))
            value['step'] = 41
            _ = (result.receipt.parent / 'stdout.log').write_text('late mutation')
            with self.assertRaises(GateContractError):
                _ = subject.authenticate_stage(document(json.dumps(value)), self.session(root))
