"""Tiny Git trees and synthetic build receipts; never Cargo/Godot/real launcher."""
import fcntl
import os
import shutil
import subprocess
import unittest
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import gate_oracle_child
import generic_play_owner as owner
import test_generic_play_owner as owner_fixtures
from generic_gate import GateContractError
from generic_play_sources import (
    HistoricalBindings,
    HistoricalSource,
    SourceGrants,
    check_bindings,
    verify_target,
)
from generic_process import atomic_json, file_hash
from generic_sources import freeze_source
from mux2_gate_inputs import EvidenceError, Node


class SourceTests(unittest.TestCase):
    def git(self, root: Path, *args: str) -> str:
        result = subprocess.run(['git', '-C', str(root), *args], capture_output=True, check=True,
            env={**{k: v for k, v in os.environ.items() if not k.startswith('GIT_')},
                'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': '/dev/null'})
        return result.stdout.decode().strip()

    def fixture(self, base: Path) -> tuple[SourceGrants, Node, Node]:
        launcher = base / 'launcher'; launcher.mkdir()
        _ = self.git(launcher, 'init', '-q')
        _ = self.git(launcher, 'config', 'user.name', 'Fixture')
        _ = self.git(launcher, 'config', 'user.email', 'fixture@example.invalid')
        (launcher / 'rust').mkdir()
        _ = (launcher / 'rust/Cargo.toml').write_text('[workspace]\n')
        _ = (launcher / '.gitignore').write_text('rust/Cargo.lock\nrust/target/\n.godot/\nignored.rs\n')
        _ = (launcher / 'sim_bridge.gdextension').write_text('macos.debug = "res://rust/target/debug/libsim_bridge.dylib"\n')
        _ = self.git(launcher, 'add', '.')
        _ = self.git(launcher, 'commit', '-qm', 'fixture')
        commit = self.git(launcher, 'rev-parse', 'HEAD')
        _ = self.git(launcher, 'update-ref', 'refs/remotes/origin/v8/main', commit)
        play = base / 'play'; play.mkdir(); root = play / 'main'
        _ = self.git(launcher, 'worktree', 'add', '-q', '--detach', str(root), commit)
        _ = (root / 'rust/Cargo.lock').write_text('fixture lock\n')
        source = freeze_source(root, commit)
        digest = source['tracked_manifest_sha256']; assert isinstance(digest, str)
        main = HistoricalSource('main', commit, digest, file_hash(root / 'rust/Cargo.lock'))
        sources = (main, replace(main, version='m1', commit='892edca1d31b26e535a3e9b62b37d62b99bb1130'),
            replace(main, version='v7', commit='b1ca032f8fa84cee87d68a185e471b8e2c2bddce', tag_oid='7db2907651b069f2db9dd843599abe1872e581b2'))
        tools = base / 'tools'; tools.mkdir()
        for name in ('cargo', 'rustc'):
            path = tools / name; _ = path.write_text('#!/bin/sh\nexit 0\n'); path.chmod(0o755)
        found = shutil.which('git'); assert found is not None
        git = Path(found).resolve()
        (tools / 'git').symlink_to(git)
        home = base / 'home'; home.mkdir()
        configs = {p / '.cargo' / name for p in (root / 'rust', *(root / 'rust').parents) for name in ('config', 'config.toml')}
        configs |= {home / '.cargo' / name for name in ('config', 'config.toml')}
        configs |= {home / '.gitconfig', home / '.config/git/config', Path('/etc/gitconfig').resolve(), launcher / '.git/config'}
        configs |= {Path(self.git(root, 'rev-parse', '--absolute-git-dir')) / 'config.worktree', launcher / '.git/config.worktree'}
        binding = HistoricalBindings(sources, (git, file_hash(git)), (tools / 'cargo', file_hash(tools / 'cargo')),
            (tools / 'rustc', file_hash(tools / 'rustc')), tuple((p, file_hash(p) if p.exists() else None) for p in sorted(configs)))
        entry = Node({'executable': str(tools / 'cargo'), 'executable_sha256': file_hash(tools / 'cargo'),
            'artifacts': [str(root / 'rust/target/debug/libsim_bridge.dylib')]})
        payload = Node({'argv': ['cargo', 'build', '-p', 'sim-bridge'], 'cwd': str(root / 'rust'),
            'environment': {'HOME': str(home), 'PATH': str(tools)}})
        return SourceGrants(binding, launcher, play), entry, payload

    def complete(self, grants: SourceGrants) -> Path:
        root = grants.play_root / 'main'; bridge = root / 'rust/target/debug/libsim_bridge.dylib'
        bridge.parent.mkdir(parents=True); _ = bridge.write_bytes(b'actual fixture build output')
        receipt = root.parent.parent / 'build-receipt.json'
        atomic_json(receipt, {'native_exit': 0, 'timed_out': False, 'errors': [], 'pid': 123,
            'argv': ['cargo', 'build', '-p', 'sim-bridge'], 'cwd': str(root / 'rust'),
            'executable': str(grants.bindings.cargo[0]), 'executable_sha256': grants.bindings.cargo[1],
            'declared_artifacts': [str(bridge)], 'artifact_hashes': {str(bridge): file_hash(bridge)}})
        return receipt

    def test_actual_generated_bridge_join_and_original_commands(self) -> None:
        with TemporaryDirectory() as folder:
            grants, entry, payload = self.fixture(Path(folder).resolve())
            record = grants.request('main', 'build', entry, payload)
            self.assertEqual(record['qualification'], 'SOURCE_BINDING_ONLY')
            receipt = self.complete(grants)
            joined = grants.returned('main', receipt)
            self.assertEqual(joined['build_receipt_sha256'], file_hash(receipt))
            self.assertEqual(joined['observer_world_qualification'], 'INCOMPLETE')
            for role in ('import-1', 'import-2', 'headless'):
                args = ['--quit-after', '300'] if role == 'headless' else ['--import', '--editor', '--quit']
                runtime = Node({'argv': ['/bound/godot', '--headless', '--path', '.', *args],
                    'cwd': str(grants.play_root / 'main'), 'environment': payload.field('environment').value})
                _ = grants.request('main', role, entry, runtime)
            grants.check()

    def test_prebuild_source_and_recipe_rejections(self) -> None:
        for mutation in ('manifest', 'ref', 'dirty', 'ignored', 'ignored-bound', 'lock', 'config', 'argv', 'cwd', 'artifact'):
            with self.subTest(mutation=mutation), TemporaryDirectory() as folder:
                grants, entry, payload = self.fixture(Path(folder).resolve()); root = grants.play_root / 'main'
                if mutation == 'manifest': grants.bindings = replace(grants.bindings, sources=(replace(grants.bindings.sources[0], tracked_manifest_sha256='0'*64), *grants.bindings.sources[1:]))
                elif mutation == 'ref': _ = self.git(grants.launcher, 'update-ref', '-d', 'refs/remotes/origin/v8/main')
                elif mutation == 'dirty': _ = (root / 'rust/Cargo.toml').write_text('changed')
                elif mutation == 'ignored': _ = (root / 'ignored.rs').write_text('extra source')
                elif mutation == 'ignored-bound':
                    ignored = root / 'ignored.rs'; _ = ignored.write_text('extra source')
                    grants.bindings = replace(grants.bindings, configs=(*grants.bindings.configs, (ignored, file_hash(ignored))))
                elif mutation == 'lock': _ = (root / 'rust/Cargo.lock').write_text('changed')
                elif mutation == 'config':
                    path = root / '.cargo/config.toml'; path.parent.mkdir(); _ = path.write_text('changed')
                elif mutation == 'argv': payload = Node({**payload.mapping(), 'argv': ['cargo', 'build', '-p', 'sim-bridge', '--release']})
                elif mutation == 'cwd': payload = Node({**payload.mapping(), 'cwd': str(root)})
                else: entry = Node({**entry.mapping(), 'artifacts': [str(root / 'other.dylib')]})
                with self.assertRaises((GateContractError, EvidenceError, OSError)):
                    _ = grants.request('main', 'build', entry, payload)
                self.assertFalse(grants.started)

    def test_build_return_and_later_mutation_reject(self) -> None:
        for mutation in ('source', 'lock', 'config', 'bridge', 'receipt'):
            with self.subTest(mutation=mutation), TemporaryDirectory() as folder:
                grants, entry, payload = self.fixture(Path(folder).resolve())
                _ = grants.request('main', 'build', entry, payload)
                receipt = self.complete(grants); _ = grants.returned('main', receipt)
                root = grants.play_root / 'main'
                if mutation == 'source': _ = (root / 'rust/Cargo.toml').write_text('changed')
                elif mutation == 'lock': _ = (root / 'rust/Cargo.lock').write_text('changed')
                elif mutation == 'config':
                    path = root / '.cargo/config'; path.parent.mkdir(); _ = path.write_text('late config')
                elif mutation == 'bridge': _ = grants.built['main'].path.write_bytes(b'replaced')
                else: _ = receipt.write_text('{}')
                with self.assertRaises((GateContractError, EvidenceError, OSError)): grants.check()

    def test_runtime_before_build_and_failed_build_reject(self) -> None:
        with TemporaryDirectory() as folder:
            grants, entry, payload = self.fixture(Path(folder).resolve())
            with self.assertRaises(GateContractError): _ = grants.request('main', 'headless', entry, payload)
            _ = grants.request('main', 'build', entry, payload)
            receipt = self.complete(grants)
            _ = receipt.write_text(receipt.read_text().replace('"native_exit": 0', '"native_exit": 1'))
            with self.assertRaises((GateContractError, EvidenceError)): _ = grants.returned('main', receipt)
            self.assertFalse(grants.built)

    def test_configuration_appearance_during_build_blocks_return(self) -> None:
        with TemporaryDirectory() as folder:
            grants, entry, payload = self.fixture(Path(folder).resolve())
            _ = grants.request('main', 'build', entry, payload)
            receipt = self.complete(grants)
            config = grants.play_root / 'main/.cargo/config.toml'
            config.parent.mkdir(); _ = config.write_text('late build config')
            with self.assertRaises(GateContractError): _ = grants.returned('main', receipt)
            self.assertFalse(grants.built)

    def test_enabled_owner_refuses_uncreated_source_before_native_build(self) -> None:
        self.assert_enabled_owner_rejects(foreign_git=False)

    def test_enabled_owner_refuses_foreign_git_before_first_native(self) -> None:
        self.assert_enabled_owner_rejects(foreign_git=True)

    def assert_enabled_owner_rejects(self, *, foreign_git: bool) -> None:
        with TemporaryDirectory() as folder:
            base = Path(folder).resolve(); source_base = base / 'source'; source_base.mkdir()
            grants, _, _ = self.fixture(source_base)
            plan = owner_fixtures.OwnerTests().fixture(base)
            prefix = grants.bindings.git[0].parent
            if foreign_git:
                prefix = base / 'foreign-tools'; prefix.mkdir()
                fake = prefix / 'git'; _ = fake.write_text('#!/bin/sh\nexit 77\n'); fake.chmod(0o755)
            old_path = dict(plan.child_environments[0][2])['PATH']
            new_path = str(prefix) + ':' + old_path
            env_file = plan.root / 'child-env.json'
            _ = env_file.write_text(env_file.read_text().replace(old_path, new_path))
            projections = tuple((case, role, tuple((key, new_path if key == 'PATH' else value) for key, value in values))
                                for case, role, values in plan.child_environments)
            plan = replace(plan, child_environments=projections)
            for name in ('generic_play_sources.py',):
                _ = shutil.copy2(Path(__file__).parent / name, plan.root / 'tools/v8' / name)
            inputs = tuple((path, file_hash(path)) for path in plan.root.rglob('*') if path.is_file())
            inputs += tuple((path, digest) for path, digest in plan.inputs if not path.is_relative_to(plan.root))
            plan = replace(plan, inputs=inputs, historical=grants.bindings)
            lock = base / 'lane'; lock.touch()
            with lock.open('r') as held, patch.object(gate_oracle_child, 'LOCK', lock):
                fcntl.flock(held.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                live = owner.PlayOwner(plan, held.fileno())
                try:
                    live.launch()
                    with self.assertRaises(owner.PlayOwnershipUnknown): _ = live.wait()
                    self.assertFalse((base / 'native-3-build').exists())
                    self.assertEqual(live.case_exits, [] if foreign_git else [3, 2, 4])
                    self.assertEqual(live.roles, [] if foreign_git else ['version'])
                    if foreign_git: self.assertFalse((base / 'native-0-version').exists())
                    self.assertFalse(live.closed)
                    held.close()
                    with lock.open('r') as contender, self.assertRaises(BlockingIOError):
                        fcntl.flock(contender.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                finally:
                    for descriptor in (live.control_write, live.request_read):
                        if descriptor >= 0: os.close(descriptor)
                    if live.shell is not None: _ = live.shell.wait(timeout=10)
                    os.close(live.lane_fd)
                    if live.stdout is not None: live.stdout.close()
                    if live.stderr is not None: live.stderr.close()

    def test_actual_child_git_resolution_must_match_pinned_inspector(self) -> None:
        with TemporaryDirectory() as folder:
            grants, entry, payload = self.fixture(Path(folder).resolve())
            fake = grants.bindings.cargo[0].parent / 'git'
            fake.unlink(); _ = fake.write_text('#!/bin/sh\nexit 77\n'); fake.chmod(0o755)
            with self.assertRaises(GateContractError):
                _ = grants.request('main', 'build', entry, payload)
            self.assertFalse(grants.started)

    def test_enabled_worktree_configuration_requires_binding(self) -> None:
        with TemporaryDirectory() as folder:
            grants, entry, payload = self.fixture(Path(folder).resolve())
            _ = self.git(grants.launcher, 'config', 'extensions.worktreeConfig', 'true')
            common = grants.launcher / '.git/config'
            admin = Path(self.git(grants.play_root / 'main', 'rev-parse', '--absolute-git-dir'))
            config = admin / 'config.worktree'
            _ = config.write_text('[core]\n\tignorecase = false\n')
            grants.bindings = replace(grants.bindings, configs=tuple(
                (path, file_hash(path) if path == common else digest)
                for path, digest in grants.bindings.configs if path != config))
            with self.assertRaises(GateContractError):
                _ = grants.request('main', 'build', entry, payload)
            self.assertFalse(grants.started)

    def test_real_annotated_tag_requires_distinct_peeled_commit(self) -> None:
        with TemporaryDirectory() as folder:
            grants, _, _ = self.fixture(Path(folder).resolve())
            _ = self.git(grants.launcher, 'tag', '-a', 'v7-final-art', '-m', 'fixture tag')
            tag = self.git(grants.launcher, 'rev-parse', 'v7-final-art')
            source = replace(grants.bindings.sources[0], version='v7', tag_oid=tag)
            self.assertNotEqual(tag, source.commit)
            verify_target(grants.launcher, source)
            with self.assertRaises(GateContractError): verify_target(grants.launcher, replace(source, commit=tag))
            with self.assertRaises(GateContractError): verify_target(grants.launcher, replace(source, tag_oid=source.commit))
            wrong = replace(grants.bindings.sources[2], commit='7db2907651b069f2db9dd843599abe1872e581b2')
            with self.assertRaises(GateContractError): check_bindings(replace(grants.bindings, sources=(*grants.bindings.sources[:2], wrong)))

    def test_bound_active_worktree_configuration_then_mutation(self) -> None:
        with TemporaryDirectory() as folder:
            grants, entry, payload = self.fixture(Path(folder).resolve())
            _ = self.git(grants.launcher, 'config', 'extensions.worktreeConfig', 'true')
            common = grants.launcher / '.git/config'
            admin = Path(self.git(grants.play_root / 'main', 'rev-parse', '--absolute-git-dir'))
            config = admin / 'config.worktree'; _ = config.write_text('[core]\n\tignorecase = false\n')
            grants.bindings = replace(grants.bindings, configs=tuple(
                (path, file_hash(path) if path in (common, config) else digest) for path, digest in grants.bindings.configs))
            _ = grants.request('main', 'build', entry, payload)
            _ = config.write_text('[core]\n\tignorecase = true\n')
            with self.assertRaises(GateContractError): grants.check()
