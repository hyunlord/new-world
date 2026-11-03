"""Actual inert build/import/workspace tools; Cargo topology and smoke pins are synthetic."""
import os
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import gate_oracle_child as child
import generic_cli_godot_products as godot
import generic_cli_setup_handoff as handoff
import generic_setup_clippy as clippy
import generic_ui_smoke_run as smoke
import generic_workspace_run as runner
import test_generic_cli_setup_handoff as helpers
import test_generic_workspace_endpoints as endpoints
from generic_cli_products import CliProducts
from generic_fixtures import ProbeProduct
from generic_process import atomic_json, file_hash
from generic_setup_prepare import PreparedSetup, prepare_warm
from generic_workspace_endpoints import GeneratedState, generated
from test_generic_setup_bridge import unused_wait


class GodotTests(unittest.TestCase):
    def fixture(self, home: Path) -> tuple[helpers.HandoffTests, CliProducts, PreparedSetup, PreparedSetup, runner.WorkspaceRequest]:
        helper = helpers.HandoffTests()
        self.addCleanup(helper.doCleanups)
        original = endpoints.EndpointTests.fixture
        def add_scene(instance: endpoints.EndpointTests, folder: Path, *, fail: bool = False) -> tuple[runner.WorkspaceRequest, PreparedSetup]:
            request, prepared = original(instance, folder, fail=fail)
            scene = prepared.inputs.root/'scenes/main.tscn'
            scene.parent.mkdir(exist_ok=True)
            _ = scene.write_text('synthetic scene, never executed')
            return request, prepared
        with patch.object(endpoints.EndpointTests, 'fixture', add_scene):
            views, candidate, request = helper.fixture(home)
        root = views.probes.base.input_root
        selection = views.builds.prepared.selectors.base
        settings = home/'base-settings'
        env = dict(candidate.inputs.environment)
        env.update(CARGO_HOME=str(selection.cargo_home), WORLDSIM_TEST_SETTINGS_DIR=str(settings))
        lock_origin = home/'base-lock-origin.json'
        atomic_json(lock_origin, {'path': str(root/'rust/Cargo.lock'), 'sha256': file_hash(root/'rust/Cargo.lock')})
        base = prepare_warm(replace(candidate.inputs, root=root, source_sha=selection.source_sha,
            cargo_home=selection.cargo_home, environment=tuple(sorted(env.items())),
            effective_inputs=tuple((name, file_hash(root/name)) for name, _digest in candidate.inputs.effective_inputs), configs=selection.configs,
            settings=settings, output=home/'base-bridge', lock_origin=lock_origin, lock_origin_sha256=file_hash(lock_origin)), views.builds.base)
        return helper, views, base, candidate, request

    def test_actual_two_arm_admission_and_drift(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            home = Path(temp).resolve()
            helper, views, base, candidate, request = self.fixture(home)
            setup = godot.begin_godot_setup(views, base, candidate)
            with self.assertRaises(ValueError):
                godot.before_base_setup(replace(setup))
            godot.before_base_setup(setup)
            lock = home/'runtime/lane'
            with patch('generic_setup_bridge.LOCK', lock), patch.object(child, 'LOCK', lock), patch.object(runner, 'LOCK', lock), \
                 patch('generic_setup_bridge.qualify_wait', return_value=()), patch('generic_setup_import.qualify_wait', return_value=()), \
                 patch.object(clippy, 'qualify_clippy', return_value=()), patch.object(smoke, 'PINS', dict[str, str]()):
                ready = godot.run_base_setup_owned(setup, unused_wait(base), home/'base-imports', request.held_lane_fd)
                prior = ready.predecessor
                owner, result = prior.owner, prior.result
                with self.assertRaises(ValueError):
                    _ = godot.finish_base_setup(setup, replace(prior, result=replace(result, status='FAIL_STEP3', shell_exit=1)))
                owner.controller.closed = False
                try:
                    with self.assertRaises(ValueError):
                        _ = godot.finish_base_setup(setup, prior)
                    _ = os.fstat(request.held_lane_fd)
                finally:
                    owner.controller.closed = True
                godot.check_base_current(ready)
                interruption = KeyboardInterrupt()
                # Boundary-only injection reuses already observed objects; no second producer launch.
                with patch.object(godot, 'before_base_setup', return_value=None), \
                     patch.object(godot, 'run_bridge_owned', return_value=owner.prior.result), \
                     patch.object(godot, 'start_import_pair', return_value=owner), \
                     patch.object(owner, 'finish', return_value=result), \
                     patch.object(godot, 'finish_base_setup', side_effect=interruption), \
                     self.assertRaises(godot.BaseSetupFailure) as caught:
                    _ = godot.run_base_setup_owned(setup, unused_wait(base), home/'not-launched', request.held_lane_fd)
                self.assertIs(caught.exception.primary, interruption)
                self.assertIs(caught.exception.owner, owner)
                self.assertIs(caught.exception.bridge, owner.prior.result)
                self.assertEqual(caught.exception.held_lane_fd, request.held_lane_fd)
                _ = os.fstat(request.held_lane_fd)
                # Old candidate-only current whole-BASE closure must not silently admit this transition.
                with self.assertRaises(ValueError):
                    _ = handoff.begin_setup_handoff(views, candidate)
                endpoint = helper.finish(home, candidate, request)
                products = godot.finish_godot_products(ready, endpoint)
                godot.check_godot_products(products)
                self.assertFalse(products.full_gate_claim)
                witness = godot.base_probe_witness(ready)
                godot.check_base_probe_witness(witness)
                self.assertIs(witness.product, views.probes.base)
                self.assertIn('rust/target/debug/libsim_bridge.dylib', dict(witness.runtime_inputs))
                self.assertFalse(any(name.startswith('rust/target/') for name, _ in witness.product.input_files))
                with self.assertRaises(ValueError):
                    godot.check_base_probe_witness(replace(witness))
                candidate_probe = views.probes.candidate.executable
                original_probe = candidate_probe.read_bytes()
                _ = candidate_probe.write_bytes(b'later candidate embedded producer')
                godot.check_base_probe_witness(witness)
                with self.assertRaises(ValueError):
                    godot.check_godot_products(products)
                _ = candidate_probe.write_bytes(original_probe)
                base_probe = witness.product.executable
                original_base = base_probe.read_bytes()
                _ = base_probe.write_bytes(b'unowned BASE drift')
                with self.assertRaises(ValueError):
                    godot.check_base_probe_witness(witness)
                _ = base_probe.write_bytes(original_base)
                godot.check_base_probe_witness(witness)
                for consumer in ('godot', 'witness'):
                    for fault in ('source', 'history', 'mode'):
                        with self.subTest(consumer=consumer, fault=fault):
                            victim = (views.normal[1].products[0].source if fault == 'source' else
                                      owner.output/'receipt.json' if fault == 'history' else base_probe)
                            data, mode = victim.read_bytes(), victim.stat().st_mode
                            calls = 0
                            def late_generated(prepared: PreparedSetup, consumer: str = consumer, fault: str = fault,
                                               victim: Path = victim, mode: int = mode) -> GeneratedState:
                                nonlocal calls
                                state = generated(prepared)
                                selected = candidate if consumer == 'godot' else base
                                if prepared is selected:
                                    calls += 1
                                    if calls == (1 if consumer == 'godot' else 2):
                                        if fault == 'mode':
                                            victim.chmod(0o700 if mode & 0o777 != 0o700 else 0o755)
                                        else:
                                            _ = victim.write_bytes(b'terminal callback drift')
                                return state
                            try:
                                with patch.object(godot, 'generated', side_effect=late_generated), self.assertRaises(ValueError):
                                    if consumer == 'godot':
                                        godot.check_godot_products(products)
                                    else:
                                        godot.check_base_probe_witness(witness)
                                self.assertEqual(calls, 1 if consumer == 'godot' else 2)
                            finally:
                                _ = victim.write_bytes(data)
                                victim.chmod(mode)
                            godot.check_godot_products(products)
                            godot.check_base_probe_witness(witness)
                self.assertIn('.godot/first', dict(products.products.base.input_files))
                self.assertEqual(products.products.base.executable, next(t.path for t in base.inputs.tools if t.name == 'godot'))
                with self.assertRaises(ValueError):
                    godot.check_godot_products(replace(products))
                original_check = smoke.check_product
                late = base.inputs.root/'.godot/callback-empty'
                def changed(product: ProbeProduct) -> None:
                    original_check(product)
                    if product is products.products.candidate:
                        late.mkdir()
                with patch.object(godot, 'check_product', side_effect=changed), self.assertRaises(ValueError):
                    godot.check_godot_products(products)
                late.rmdir()
                victim = base.inputs.root/'.godot/late-empty'
                victim.mkdir()
                with self.assertRaises(ValueError):
                    godot.check_godot_products(products)
                victim.rmdir()
                evidence = owner.output/'receipt.json'
                old = evidence.read_bytes()
                _ = evidence.write_bytes(b'{}')
                with self.assertRaises(ValueError):
                    godot.check_godot_products(products)
                _ = evidence.write_bytes(old)
                godot.check_godot_products(products)

    def test_foreign_arm_and_early_drift_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            home = Path(temp).resolve()
            _helper, views, base, candidate, _request = self.fixture(home)
            with self.assertRaises(ValueError):
                _ = godot.begin_godot_setup(views, candidate, candidate)
            setup = godot.begin_godot_setup(views, base, candidate)
            extra = base.inputs.root/'rust/target/unowned'
            extra.mkdir()
            with self.assertRaises(ValueError):
                godot.before_base_setup(setup)
