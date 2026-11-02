"""Offline registry coverage/mutation tests; no subprocess or simulated worlds."""
import copy
import json
import tempfile
import unittest
from dataclasses import FrozenInstanceError
from pathlib import Path

from generic_fixture_registry import (
    ExpansionContext,
    RegistryError,
    expand,
    load_registry,
)

REGISTRY = Path(__file__).with_name('gate_fixtures.json')


class RegistryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.raw = json.loads(REGISTRY.read_text())
        self.registry = load_registry(REGISTRY)
        self.context = ExpansionContext(Path('/harness'),Path('/inputs'),Path('/out'),Path('/base/probe'),Path('/candidate/probe'))

    def parse_modified(self, raw):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)/'registry.json'
            _ = path.write_text(json.dumps(raw))
            return load_registry(path)

    def test_all_direct_and_backlog_preserved_and_incomplete(self):
        self.assertEqual(len(self.registry.fixtures),28)
        self.assertEqual(len(self.registry.backlog),12)
        with self.assertRaisesRegex(RegistryError,'unresolved producer'):
            self.registry.require_complete()
        field = 'id'
        with self.assertRaises(FrozenInstanceError):
            setattr(self.registry.fixtures[0], field, 'changed')
        self.assertEqual(json.loads(self.registry.metadata_json),self.raw)

    def test_deterministic_expansion_and_distinct_occurrences(self):
        plans = expand(self.registry,self.context)
        self.assertEqual(plans,expand(self.registry,self.context))
        self.assertEqual(len(plans),62)
        self.assertEqual(len({p.out for p in plans}),62)
        self.assertEqual({p.arm for p in plans},{'base','candidate'})
        for fixture in self.registry.fixtures:
            rows = [p for p in plans if p.fixture_id == fixture.id]
            for arm in ('base','candidate'):
                for repeat in fixture.repeats:
                    matching = [p for p in rows if p.arm == arm and p.repeat == repeat]
                    self.assertEqual(tuple(seed for p in matching for seed in p.seeds),fixture.seeds)
                    self.assertTrue(all(p.roles == fixture.roles for p in matching))
        self.assertTrue(all('--gate-observations' in p.argv for p in plans if p.argv is not None and not p.fixture_id.startswith('m6a.food')))

    def test_same_argv_consumers_are_not_deduplicated(self):
        plans = expand(self.registry,self.context)
        rows = [p for p in plans if p.arm == 'base' and p.fixture_id in {'colony.u10','core.colony_u10','read.determinism'}]
        self.assertEqual(len(rows),3)
        self.assertEqual(len({p.out for p in rows}),3)
        self.assertTrue(all(p.roles == ('primary','determinism_twin') for p in rows))

    def test_generated_replay_is_same_arm_dependency_not_fake_static_hash(self):
        plans = expand(self.registry,self.context)
        for row in (p for p in plans if p.fixture_id == 'replay.recorded'):
            dependency = row.generated_input
            self.assertIsNotNone(dependency)
            assert dependency is not None
            assert row.argv is not None
            self.assertEqual(dependency.producer_invocation,f'{row.arm}/replay.generate/1')
            self.assertIsNone(dependency.sha256)
            producer = next(p for p in plans if p.id == dependency.producer_invocation)
            self.assertEqual(dependency.path,producer.out/'commands'/'colony_s5_commands.json')
            self.assertIn(str(dependency.path),row.argv)

    def test_unresolved_companion_and_m6a_remain_visible_blocked(self):
        plans = expand(self.registry,self.context)
        companion = [p for p in plans if p.fixture_id == 'knowledge.f10_no_kill']
        self.assertEqual(len(companion),2)
        self.assertTrue(all(p.argv is None and p.blockers for p in companion))
        m6a = [p for p in plans if p.fixture_id.startswith('m6a.food')]
        self.assertEqual(len(m6a),6)
        self.assertTrue(all('adapter_not_implemented' in p.blockers for p in m6a))
        self.assertTrue(all(p.argv is not None and '--gate-observations' not in p.argv for p in m6a))
        selected = next(p for p in m6a if p.fixture_id == 'm6a.food_data_only')
        self.assertEqual(selected.environment,(('WORLDSIM_CONTENT_PACK','/inputs/rust/crates/sim-test/data/m6a_apple/pack.ron'),))

    def test_executable_contract_fields_match_source_exactly(self):
        for fixture in self.registry.fixtures:
            source = next(row for row in self.raw['direct_fixture_drafts'] if row['id'] == fixture.id)
            self.assertEqual(json.loads(fixture.metadata_json),source)
            self.assertEqual(fixture.argv,tuple(source['argv']))
        plans = expand(self.registry,self.context)
        lifecycle = [p for p in plans if p.fixture_id.startswith('m6a.lifecycle')]
        self.assertEqual(len(lifecycle),8)
        self.assertTrue(all(p.roles == ('primary','determinism_twin') for p in lifecycle))
        self.assertEqual({p.repeat for p in lifecycle},{'a','b'})

    def test_missing_duplicate_and_unknown_fixture_or_backlog_rejected(self):
        for key in ('direct_fixture_drafts','adapter_backlog'):
            for mode in ('remove','duplicate','unknown'):
                raw = copy.deepcopy(self.raw)
                if mode == 'remove': raw[key].pop()
                elif mode == 'duplicate': raw[key][-1] = raw[key][0]
                else: raw[key][-1]['id'] = 'unknown'
                with self.subTest(key=key,mode=mode),self.assertRaises(RegistryError):
                    _ = self.parse_modified(raw)

    def test_reject_inconsistent_seeds_horizons_twins_outputs_and_flags(self):
        mutations = [
            lambda row: row['seeds'].append(32),
            lambda row: row['horizon'].__setitem__('completed_ticks',1),
            lambda row: row['instance_axes'].__setitem__('determinism_instance',['primary','twin']),
            lambda row: row['argv'].extend(['--out','/shared']),
            lambda row: row['argv'].extend(['--unrecognised','yes']),
            lambda row: row['argv'].__setitem__(0,'{evil}'),
            lambda row: row['input_paths'].append('../escape'),
            lambda row: row.__setitem__('role','explicit_fault_control'),
            lambda row: row.__setitem__('adapter_status','QUALIFIED'),
        ]
        for change in mutations:
            raw = copy.deepcopy(self.raw)
            change(raw['direct_fixture_drafts'][0])
            with self.subTest(change=change),self.assertRaises(RegistryError):
                _ = self.parse_modified(raw)

    def test_duplicate_json_keys_and_unknown_top_level_keys_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)/'bad.json'
            _ = path.write_text('{"schema":"one","schema":"two"}')
            with self.assertRaisesRegex(RegistryError,'duplicate JSON key'):
                _ = load_registry(path)
        self.raw['ignored_exemption'] = True
        with self.assertRaises(RegistryError): self.parse_modified(self.raw)

    def test_no_relative_output_or_cross_arm_replay_override(self):
        context = ExpansionContext(Path('/h'),Path('/i'),Path('relative'),Path('/b'),Path('/c'))
        with self.assertRaises(RegistryError): expand(self.registry,context)
        row = next(row for row in self.raw['direct_fixture_drafts'] if row['id'] == 'replay.recorded')
        row['generated_input']['arm'] = 'base'
        with self.assertRaises(RegistryError): self.parse_modified(self.raw)


    def test_standing_matrix_independent_literal_seeds_horizons_and_twins(self):
        # This table is an independent acceptance contract, not imported from parser constants.
        expected = """
identity.baseline_samples 0 32 120 6 1
identity.baseline_twins 0 4 120 4 2
determinism.baseline 0 4 120 4 2
colony.u10 0 4 10 4 2
core.colony_u10 0 4 10 4 2
read.determinism 0 4 10 4 2
replay.generate 5 7 3 2 1
replay.recorded 5 6 3 1 1
replay.scripted 5 6 3 1 1
economy.soak_no_beasts 0 4 20 4 2
story.script48 0 2 48 2 1
neighbours.script48 0 2 48 2 1
read.story48 0 2 48 2 1
read.neighbours48 0 2 48 2 1
steward.on24_twins 0 2 24 2 2
read.steward_on24 0 2 24 2 1
read.m6a_steward_on72 0 2 72 2 1
read.soak20 0 4 20 4 1
core.colony_soak20 0 2 20 2 1
core.baseline10 0 4 10 4 1
core.steward_off 0 1 20 1 1
core.steward_on 0 1 20 1 1
m6a.lifecycle_off 0 1 20 1 2
m6a.lifecycle_on 0 1 20 1 2
knowledge.f10_injected 0 8 4 4 1
knowledge.f10_no_kill 0 8 4 4 1
"""
        by_id = {row.id:row for row in self.registry.fixtures}
        for line in expected.splitlines():
            if not line.strip(): continue
            name,start,end,days,workers,instances = line.split()
            fixture = by_id[name]
            self.assertEqual(fixture.seeds,tuple(range(int(start),int(end))))
            self.assertEqual(fixture.argv[fixture.argv.index('--days')+1],days)
            self.assertEqual(fixture.argv[fixture.argv.index('--workers')+1],workers)
            self.assertEqual(len(fixture.roles),int(instances))
            self.assertEqual(fixture.repeats,('a','b') if name.startswith('m6a.lifecycle_') else ('1',))
        self.assertEqual(by_id['m6a.food72'].argv,('{probe}','m6a','--seed','{seed}','--days','72','--out','{out}'))
        self.assertEqual(by_id['m6a.food72'].seeds,(0,1))
        self.assertEqual(by_id['m6a.food_data_only'].argv,('{probe}','m6a-data-only','--out','{out}'))
        self.assertEqual(by_id['m6a.food_data_only'].seeds,(0,))
        for name,steward in [('core.steward_off','off'),('core.steward_on','on')]:
            self.assertEqual(by_id[name].argv,('{probe}','--scene','colony','--seeds','0..1','--days','20','--workers','1','--steward',steward,'--story','on','--neighbours','on','--out','{out}'))

    def test_coordinated_duration_weakening_rejected_for_every_fixed_fixture(self):
        for fixture in self.raw['direct_fixture_drafts']:
            if fixture['horizon']['kind'] != 'requested_days': continue
            raw = copy.deepcopy(self.raw)
            row = next(x for x in raw['direct_fixture_drafts'] if x['id'] == fixture['id'])
            row['argv'][row['argv'].index('--days')+1] = '1'
            row['horizon']['days'] = 1
            row['horizon']['completed_ticks'] = 1440
            with self.subTest(fixture=row['id']),self.assertRaisesRegex(RegistryError,'required workload'):
                _ = self.parse_modified(raw)

    def test_coordinated_seed_twin_consumer_and_oracle_weakening_rejected(self):
        for change in ('seeds','twins','consumers','oracles','default_defence'):
            raw = copy.deepcopy(self.raw)
            row = next(x for x in raw['direct_fixture_drafts'] if x['id'] == 'economy.soak_no_beasts')
            if change == 'seeds':
                row['seeds'] = [0]
                row['argv'][row['argv'].index('--seeds')+1] = '0..1'
            elif change == 'twins':
                row['argv'].remove('--check-determinism')
                row['instance_axes']['determinism_instance'] = ['primary']
            elif change == 'consumers': row['consumers'] = [50]
            elif change == 'oracles': row['retain_oracles'].remove('all death causes0')
            else: row['argv'].extend(['--defence','on'])
            with self.subTest(change=change),self.assertRaises(RegistryError): self.parse_modified(raw)

    def test_additive_fixture_extension_does_not_replace_required_contract(self):
        extra = copy.deepcopy(next(row for row in self.raw['direct_fixture_drafts'] if row['id'] == 'core.steward_off'))
        extra['id'] = 'extension.custom_world'
        extra['argv'][extra['argv'].index('--days')+1] = '2'
        extra['horizon']['days'] = 2
        extra['horizon']['completed_ticks'] = 2880
        self.raw['direct_fixture_drafts'].append(extra)
        extended = self.parse_modified(self.raw)
        self.assertEqual(len(extended.fixtures),29)
        self.assertEqual(len(expand(extended,self.context)),64)


if __name__ == '__main__':
    _ = unittest.main()
