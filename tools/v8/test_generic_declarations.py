"""Real temporary Git histories exercise committed declaration boundaries."""
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from typing import Literal

from generic_declarations import Declarations, read_declarations
from generic_gate import document
from generic_sources import freeze_source
from mux2_gate_inputs import Json


class DeclarationFixture(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.git('init', '-q')
        self.git('config', 'user.name', 'Test')
        self.git('config', 'user.email', 't@example.invalid')
        (self.root / 'simulation.rs').write_text('base')
        self.base = self.commit('base')
        (self.root / 'simulation.rs').write_text('changed')
        self.behavior = self.commit('change behavior')
        self.path = 'docs/v8/m6b/CHANGED_FIXTURES.json'

    def git(self, *args: str) -> str:
        return subprocess.check_output(['git', '-C', str(self.root), *args], stderr=subprocess.PIPE).decode().strip()

    def commit(self, message: str) -> str:
        self.git('add', '.')
        self.git('commit', '-qm', message)
        return self.git('rev-parse', 'HEAD')

    def write_declaration(self, changes: list[dict[str, str]] | None = None, base: str | None = None, path: str | None = None) -> None:
        destination = self.root / (path or self.path)
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(json.dumps({'schema_version': 1, 'base_commit': base or self.base,
            'changes': changes if changes is not None else [
                {'fixture_id': 'colony.test', 'behavior_commit': self.behavior, 'reason': 'Changed simulation behavior'}]}))

    def read(self, purpose: Literal['publication', 'self_proof', 'defect_proof'] = 'publication', known: tuple[str, ...] = ('colony.test',)) -> Declarations:
        return read_declarations(self.root, self.git('rev-parse', 'HEAD'), self.base, purpose, known)


class DeclarationTests(DeclarationFixture):
    def test_real_later_declaration_is_bound(self):
        self.write_declaration()
        candidate = self.commit('record fixture change')
        result = read_declarations(self.root, candidate, self.base, 'publication', ('colony.test',))
        self.assertEqual(result.changes[0].behavior_commit, self.behavior)
        self.assertEqual(result.changes[0].declaration_commit, candidate)

    def test_uncommitted_declaration_is_rejected(self):
        self.write_declaration()
        with self.assertRaises(ValueError):
            read_declarations(self.root, self.behavior, self.base, 'publication', ('colony.test',))

    def test_proofs_reject_actual_active_changes(self):
        self.write_declaration()
        self.commit('record fixture change')
        for purpose in ('self_proof', 'defect_proof'):
            with self.subTest(purpose=purpose), self.assertRaisesRegex(ValueError, 'zero declarations'):
                self.read(purpose)

    def test_proofs_allow_empty_committed_declaration(self):
        self.write_declaration(changes=[])
        self.commit('zero changes')
        self.assertEqual(self.read('defect_proof').changes, ())

    def test_unchanged_historical_file_is_not_carried_forward(self):
        self.write_declaration()
        prior = self.commit('previous stage declaration')
        (self.root / 'simulation.rs').write_text('next stage')
        candidate = self.commit('next stage behavior')
        result = read_declarations(self.root, candidate, prior, 'publication', ('colony.test',))
        self.assertEqual(result.changes, ())
        self.assertEqual(result.ignored_historical, (self.path,))

    def test_stale_base_rejected(self):
        self.write_declaration(base=self.behavior)
        self.commit('stale declaration')
        with self.assertRaises(ValueError):
            self.read()

    def test_unknown_and_protected_ids_rejected(self):
        self.write_declaration()
        self.commit('declaration')
        with self.assertRaisesRegex(ValueError, 'unknown'):
            self.read(known=('different.fixture',))
        self.write_declaration(changes=[{'fixture_id': 'knowledge.f10_injected',
                                        'behavior_commit': self.behavior, 'reason': 'must not exempt'}])
        self.commit('protected declaration')
        with self.assertRaisesRegex(ValueError, 'protected'):
            self.read(known=('knowledge.f10_injected',))

    def test_duplicate_ids_across_files_rejected(self):
        self.write_declaration()
        self.write_declaration(path='docs/another/CHANGED_FIXTURES.json')
        self.commit('duplicate declarations')
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            self.read()

    def test_empty_reason_rejected(self):
        self.write_declaration(changes=[{'fixture_id': 'colony.test', 'behavior_commit': self.behavior, 'reason': '  '}])
        self.commit('empty reason')
        with self.assertRaisesRegex(ValueError, 'empty'):
            self.read()

    def test_self_and_abbreviated_behavior_tokens_rejected(self):
        for value in ('SELF', self.behavior[:8]):
            self.write_declaration(changes=[{'fixture_id': 'colony.test', 'behavior_commit': value, 'reason': 'invalid'}])
            self.commit('invalid behavior reference')
            with self.subTest(value=value), self.assertRaisesRegex(ValueError, 'full literal'):
                self.read()

    def test_base_behavior_is_not_post_base(self):
        self.write_declaration(changes=[{'fixture_id': 'colony.test', 'behavior_commit': self.base, 'reason': 'invalid'}])
        self.commit('base behavior')
        with self.assertRaisesRegex(ValueError, 'post-BASE'):
            self.read()

    def test_documentation_commit_is_not_behavior(self):
        (self.root / 'README.md').write_text('docs')
        doc_commit = self.commit('documentation only')
        self.write_declaration(changes=[{'fixture_id': 'colony.test', 'behavior_commit': doc_commit, 'reason': 'invalid'}])
        self.commit('doc behavior declaration')
        with self.assertRaisesRegex(ValueError, 'documentation-only'):
            self.read()

    def test_behavior_must_precede_declaration_in_ancestry(self):
        original = self.behavior
        self.git('checkout', '-qb', 'later-behavior')
        (self.root / 'simulation.rs').write_text('future behavior branch')
        future = self.commit('future behavior')
        self.git('checkout', '--detach', original)
        self.write_declaration(changes=[{'fixture_id': 'colony.test', 'behavior_commit': future, 'reason': 'not yet ancestor'}])
        self.commit('declaration before behavior merge')
        self.git('merge', '--no-ff', '-m', 'merge later behavior', 'later-behavior')
        with self.assertRaises(ValueError):
            self.read()

    def test_unknown_schema_field_and_boolean_version_rejected(self):
        self.write_declaration()
        path = self.root / self.path
        data = document(path.read_text()).mapping()
        data['file_committed'] = True
        path.write_text(json.dumps(data))
        self.commit('untrusted assertion')
        with self.assertRaisesRegex(ValueError, 'schema'):
            self.read()
        del data['file_committed']
        data['schema_version'] = True
        path.write_text(json.dumps(data))
        self.commit('boolean version')
        with self.assertRaises(ValueError):
            self.read()

    def test_duplicate_json_key_rejected(self):
        self.write_declaration()
        path = self.root / self.path
        path.write_text(path.read_text().replace('{', '{"schema_version":1,', 1))
        self.commit('duplicate JSON key')
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            self.read()

    def test_clean_source_required_at_end(self):
        from unittest.mock import patch
        self.write_declaration()
        self.commit('declaration')
        calls = 0
        def drift(root: Path, sha: str) -> dict[str, Json]:
            nonlocal calls
            calls += 1
            if calls == 2:
                (root / 'simulation.rs').write_text('changed during read')
            return freeze_source(root, sha)
        with patch('generic_declarations.freeze_source', side_effect=drift), self.assertRaises(ValueError):
            self.read()

    def test_validator_entries_are_derived_from_committed_bytes(self):
        self.write_declaration()
        candidate = self.commit('declaration')
        result = self.read(known=('colony.test', 'colony.other'))
        fields = result.validator_fields('colony.test')
        self.assertEqual(fields['file_commit'], candidate)
        self.assertEqual(fields['declarations'], [{'fixture_id': 'colony.test', 'base': self.base,
                         'behavior_commit': self.behavior, 'reason': 'Changed simulation behavior'}])
        self.assertEqual(result.validator_fields('colony.other')['declarations'], [])


class PathTests(unittest.TestCase):
    def test_only_canonical_pass_document_paths(self):
        from generic_declarations import declaration_path
        declaration_path('docs/v8/m6b/CHANGED_FIXTURES.json')
        declaration_path('docs/mypass/CHANGED_FIXTURES.json')
        for path in ('/docs/pass/CHANGED_FIXTURES.json', 'docs/../CHANGED_FIXTURES.json',
                     'docs/CHANGED_FIXTURES.json', 'docs/pass/other.json', 'tools/pass/CHANGED_FIXTURES.json',
                     'docs//pass/CHANGED_FIXTURES.json'):
            with self.subTest(path=path), self.assertRaises(ValueError):
                declaration_path(path)


class CompanionDeclarationTests(DeclarationFixture):
    def test_ordinary_f10_companion_can_declare_equality_change_but_not_loss(self):
        from generic_gate import validate_declared_fixture
        self.write_declaration(changes=[{'fixture_id': 'knowledge.f10_no_kill',
                                        'behavior_commit': self.behavior, 'reason': 'Ordinary companion behavior changed'}])
        self.commit('ordinary companion declaration')
        bundle = self.read(known=('knowledge.f10_no_kill',))
        fields = bundle.validator_fields('knowledge.f10_no_kill')
        fields['semantic_equal'] = False
        safety: dict[str, Json] = {'classification': 'ordinary', 'complete': True, 'initial_tick': 0,
                  'end_tick': 10, 'planned_end_tick': 10, 'founders': 10, 'founder_census_tick': 0,
                  'base_alive': 8, 'candidate_alive': 8, 'lost': [], 'base_lost': [],
                  'prior_loss_history_complete': True, 'base_prior_loss_history_complete': True}
        fields['safety'] = safety
        validate_declared_fixture(json.dumps(fields))
        safety['lost'] = [[5, 'knowledge_node', 1]]
        with self.assertRaisesRegex(ValueError, 'KnowledgeLost'):
            validate_declared_fixture(json.dumps(fields))
