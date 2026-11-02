"""Real tiny Python children emit synthetic save graphs; no native simulations."""
import json
import sys
import tempfile
import unittest
from collections.abc import Callable, Generator
from contextlib import contextmanager
from dataclasses import dataclass, replace
from pathlib import Path

from generic_gate import GateContractError, document
from generic_process import ProcessSpec, execute, file_hash
from generic_sources import inventory
from generic_test_history import FORMAT_SOURCE, resolve
from generic_test_observations import Binding, ExpectedOccurrence, PendingReason
from mux2_gate_inputs import Json, Node

CHILD = '''
import json,os,pathlib,sys
root=pathlib.Path(os.environ['WORLDSIM_GATE_TEST_OUT'])
parent=root/f'save_load_pid{os.getpid()}';parent.mkdir()
rows=json.loads(pathlib.Path('plan.json').read_text())
def state(t):
 return {'tick':t,'alive':2,'alive_ids':[1,2],'raw_agent_population':2,'raw_agent_ids':[1,2],'census_semantics':'alive excludes BodyHealth::is_dead; raw includes every Agent; wrapper must check ordinary endpoint agreement','knowledge_capability':{'enabled':True,'source':'resources.rules.colony.knowledge'},'knowledge_lost':[],'history_order_valid':True,'chronicle_entries':0,'command_log':[],'births':0,'deaths':{k:0 for k in ('starvation','dehydration','exposure','combat','old_age')}}
for r in rows:
 folder=parent/f"{r['test']}_s{r['seed']}_{r['branch']}";folder.mkdir()
 initial=state(r['start']); terminal=state(r['end']); founding=None if r.get('external_origin') else state(0)
 prefix=None
 if r.get('parent') or r.get('external'):
  owner=r.get('parent',r['branch']);label=r.get('label','external_input')
  save_dir=parent/f"{r['test']}_s{r['seed']}_{owner}";save_dir.mkdir(exist_ok=True)
  artifact=save_dir/f'save_{label}.wsave';checkpoint=save_dir/f'save_{label}.json'
  raw=b'WORLDSIM'+(15).to_bytes(4,'little')+b'synthetic saved bytes'
  artifact.write_bytes(raw)
  checkpoint.write_text(json.dumps({'state':initial,'save_artifact':str(artifact),'byte_count':len(raw),'magic':list(raw[:8]),'input_version_le_bytes':list(raw[8:12]),'current_format_version':15,'external_sha256_required':True}))
  prefix={'save_artifact':str(artifact),'checkpoint':str(checkpoint),'external_hash_authentication_required':True}
  if r.get('external'):
   prefix.update(kind='external_legacy_save',founding_census=None)
   (folder/'external-prefix.json').write_text(json.dumps(prefix))
  else:
   prefix['native_prefix_equal']=True
   (folder/'load-link.json').write_text(json.dumps({'prefix':prefix,'founding':founding,'initial':initial}))
 (folder/'initial.json').write_text(json.dumps({'schema_version':1,'test':r['test'],'seed':r['seed'],'branch':r['branch'],'pid':os.getpid(),'initial':initial,'history_complete':r['start']==0,'status':'PENDING_NATIVE_TEST_RESULT'}))
 (folder/'terminal.json').write_text(json.dumps({'schema_version':1,'initial':initial,'founding':founding,'terminal':terminal,'completed_ticks':r['end']-r['start'],'stop_reason':'synthetic planned boundary','prefix':prefix,'history_complete':prefix is None and r['start']==0,'loaded_prefix_requires_external_authentication':prefix is not None or r['start']!=0,'status':'ENDPOINT_REACHED_NATIVE_TEST_RESULT_REQUIRED'}))
for test in sorted({r['test'] for r in rows}): print('test '+test+' ... ok')
sys.exit(int(sys.argv[1]))
'''


def cases() -> list[dict[str, Json]]:
    rows: list[dict[str, Json]] = []
    def add(test: str, seed: int, branch: str, start: int, end: int, parent: str = '', label: str = '') -> None:
        row: dict[str, Json] = {'test': test, 'seed': seed, 'branch': branch, 'start': start, 'end': end}
        if parent:
            row.update(parent=parent, label=label)
        rows.append(row)
    for test, seeds in (('save_load_roundtrip', range(8)), ('save_load_roundtrip_baseline', range(2))):
        for seed in seeds:
            add(test, seed, 'continuous', 0, 3000)
            add(test, seed, 'saved', 0, 3000)
            add(test, seed, 'loaded', 2000, 3000, 'saved', 'main')
    test = 'save_load_roundtrip_through_deaths_and_births'
    add(test, 0, 'continuous', 0, 37000)
    add(test, 0, 'original', 0, 25000)
    add(test, 0, 'loaded_1', 25000, 28000, 'original', 'chain')
    add(test, 0, 'loaded_2', 28000, 37000, 'loaded_1', 'chain')
    test = 'save_is_deterministic'
    add(test, 3, 'original', 0, 1500)
    add(test, 3, 'twin', 0, 1500)
    add(test, 3, 'loaded_no_ticks', 1500, 1500, 'original', 'first')
    test = 'bridge_save_files_round_trip'
    add(test, 0, 'original', 0, 1500)
    add(test, 0, 'loaded', 1200, 1500, 'original', 'bridge_file')
    test = 'save_load_roundtrip_through_a_pregnancy_and_a_due_birth'
    for prefix, at, after in (('pregnancy', 5000, 1000), ('queued', 8000, 2000)):
        add(test, 1, prefix + '_continuous', 0, at + after)
        add(test, 1, prefix + '_saved', 0, at)
        add(test, 1, prefix + '_loaded', at, at + after, prefix + '_saved', 'event')
    add(test, 1, 'queued_probe', 8000, 8001, 'queued_continuous', 'queued_probe')
    return rows


@dataclass(frozen=True, slots=True)
class Graph:
    spec: ProcessSpec
    pid: int
    expected: tuple[ExpectedOccurrence, ...]

    def binding(self) -> Binding:
        return Binding(self.spec, file_hash(self.spec.output_root / 'process.json'), self.pid,
                       tuple(inventory(self.spec.output_root, [f'save_load_pid{self.pid}']).items()))

    def folder(self, name: str) -> Path:
        return self.spec.output_root / f'save_load_pid{self.pid}' / name

    def change(self, name: str, edit: Callable[[dict[str, Json]], None]) -> None:
        path = self.folder(name)
        data = document(path.read_text()).mapping()
        edit(data)
        _ = path.write_text(json.dumps(data))


@contextmanager
def graph(rows: list[dict[str, Json]] | None = None, *, exit_code: int = 0) -> Generator[Graph]:
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary).resolve()
        rows = cases() if rows is None else rows
        _ = (root / 'plan.json').write_text(json.dumps(rows))
        _ = (root / 'child.py').write_text(CHILD)
        source = root / FORMAT_SOURCE
        source.parent.mkdir(parents=True)
        _ = source.write_text('pub const FORMAT_VERSION: u32 = 15;\n')
        python = Path(sys.executable).resolve()
        output = root / 'run'
        spec = ProcessSpec((str(python), str(root / 'child.py'), str(exit_code)), root, python, file_hash(python),
                           root, tuple(inventory(root, ['plan.json', 'child.py', FORMAT_SOURCE]).items()), output,
                           (('WORLDSIM_GATE_OBSERVER', '1'), ('WORLDSIM_GATE_TEST_OUT', str(output))))
        result = execute(spec)
        assert result.pid is not None
        expected = tuple(ExpectedOccurrence(Node(row['test']).text(), Node(row['test']).text(),
                          int(Node(row['seed']).number()), Node(row['branch']).text(),
                          int(Node(row['start']).number()), int(Node(row['end']).number()), True) for row in rows)
        yield Graph(spec, result.pid, expected)


class HistoryTests(unittest.TestCase):
    def test_all46_authenticate_17_links_and29_fresh_roots(self) -> None:
        with graph() as data:
            resolved = resolve(data.binding(), data.expected)
            self.assertEqual(len(resolved), 46)
            self.assertEqual(sum(bool(row.edges) for row in resolved), 17)
            self.assertTrue(all(row.history_complete and not row.pending for row in resolved))
            self.assertTrue(all(row.founding is not None and row.founding.tick == 0 for row in resolved))
            child = next(row for row in resolved if row.expected.branch == 'loaded_2')
            self.assertEqual(len(child.edges), 2)
            self.assertFalse(child.observation.history_complete)
            probe = next(row for row in resolved if row.expected.branch == 'queued_probe')
            self.assertEqual(probe.edges[0].tick, 8000)
            zero = next(row for row in resolved if row.expected.branch == 'loaded_no_ticks')
            self.assertEqual(zero.observation.initial.tick, zero.observation.terminal.tick)

    def test_missing_source_occurrence_or_unplanned_folder_rejected(self) -> None:
        with graph() as data:
            with self.assertRaises(GateContractError):
                _ = resolve(data.binding(), data.expected[:-1])
            data.folder('unexpected').mkdir()
            with self.assertRaises(GateContractError):
                _ = resolve(data.binding(), data.expected)

    def test_missing_hash_and_stale_saved_bytes_fail(self) -> None:
        with graph() as data:
            binding = data.binding()
            omitted = replace(binding, output_hashes=tuple((p, h) for p, h in binding.output_hashes if not p.endswith('saved/save_main.wsave')))
            with self.assertRaises(GateContractError):
                _ = resolve(omitted, data.expected)
            _ = data.folder('save_load_roundtrip_s0_saved/save_main.wsave').write_bytes(b'tampered')
            with self.assertRaises(GateContractError):
                _ = resolve(binding, data.expected)

    def test_forged_founding_and_checkpoint_rejected(self) -> None:
        for filename, key in (('terminal.json', 'founding'), ('save_main.json', 'state')):
            with self.subTest(key=key), graph() as data:
                folder = 'save_load_roundtrip_s0_loaded' if key == 'founding' else 'save_load_roundtrip_s0_saved'
                data.change(folder + '/' + filename, lambda row, key=key: Node(row[key]).mapping().update(births=1))
                with self.assertRaises(GateContractError):
                    _ = resolve(data.binding(), data.expected)

    def test_wrong_parent_or_self_cycle_cannot_rebind_prefix(self) -> None:
        for owner in ('save_load_roundtrip_s1_saved', 'save_load_roundtrip_s0_loaded'):
            with self.subTest(owner=owner), graph() as data:
                for name in ('terminal.json', 'load-link.json'):
                    data.change('save_load_roundtrip_s0_loaded/' + name, lambda row, owner=owner: Node(row['prefix']).mapping().update(
                        save_artifact=str(data.folder(owner + '/save_main.wsave')),
                        checkpoint=str(data.folder(owner + '/save_main.json'))))
                with self.assertRaises(GateContractError):
                    _ = resolve(data.binding(), data.expected)

    def test_native_failure_and_missing_outcome_never_resolve(self) -> None:
        with graph(exit_code=7) as data:
            result = resolve(data.binding(), data.expected)
            self.assertTrue(all(not row.history_complete for row in result))
            self.assertTrue(all(PendingReason.NATIVE_PROCESS_FAILED in row.pending for row in result))
        with graph() as data:
            expected = tuple(replace(row, native_test='missing::' + row.test) for row in data.expected)
            result = resolve(data.binding(), expected)
            self.assertTrue(all(not row.history_complete for row in result))

    def test_external_origin_and_current_resave_descendant_stay_unresolved(self) -> None:
        test = 'genuine_legacy_inventory_preserves_fields_and_current_policy_continuation'
        rows: list[dict[str, Json]] = [{'test': test, 'seed': 5, 'branch': 'fixture_0_initial', 'start': 2000, 'end': 2000,
                                         'external': True, 'external_origin': True},
                                      {'test': test, 'seed': 5, 'branch': 'fixture_0_twin', 'start': 2000, 'end': 3000,
                                           'parent': 'fixture_0_initial', 'label': 'current', 'external_origin': True}]
        with graph(rows) as data:
            result = resolve(data.binding(), data.expected)
            self.assertTrue(all(row.founding is None and not row.history_complete for row in result))
            self.assertIn(PendingReason.EXTERNAL_ORIGIN_UNRESOLVED, result[1].pending)
            self.assertEqual(len(result[1].edges), 1)

    def test_version_is_derived_from_bound_source(self) -> None:
        with graph() as data:
            binding = data.binding()
            changed = replace(binding, process=replace(binding.process, inputs=tuple((p, h) for p, h in binding.process.inputs if p != FORMAT_SOURCE)))
            with self.assertRaises(GateContractError):
                _ = resolve(changed, data.expected)
            data.change('save_load_roundtrip_s0_saved/save_main.json', lambda row: row.update(current_format_version=14))
            with self.assertRaises(GateContractError):
                _ = resolve(data.binding(), data.expected)

    def test_checkpoint_preserves_duplicate_losses_without_later_parent_events(self) -> None:
        with graph() as data:
            parent = 'save_load_roundtrip_s0_saved/'
            child = 'save_load_roundtrip_s0_loaded/'

            def losses(row: dict[str, Json], key: str, count: int) -> None:
                events: list[Json] = [{'tick': 2000, 'node': 'Fire', 'holder': None}] * count
                Node(row[key]).mapping().update(knowledge_lost=events, chronicle_entries=count)

            data.change(parent + 'terminal.json', lambda row: losses(row, 'terminal', 3))
            data.change(parent + 'save_main.json', lambda row: losses(row, 'state', 2))
            for filename in ('initial.json', 'terminal.json', 'load-link.json'):
                data.change(child + filename, lambda row: losses(row, 'initial', 2))
            data.change(child + 'terminal.json', lambda row: losses(row, 'terminal', 2))
            result = resolve(data.binding(), data.expected)
            loaded = next(row for row in result if row.expected.test == 'save_load_roundtrip'
                          and row.expected.seed == 0 and row.expected.branch == 'loaded')
            self.assertTrue(loaded.history_complete)
            self.assertIsNotNone(loaded.observation.terminal.losses)
            self.assertEqual(len(loaded.observation.terminal.losses or ()), 2)
            data.change(parent + 'terminal.json', lambda row: losses(row, 'terminal', 1))
            with self.assertRaisesRegex(GateContractError, 'loss sequence prefix changed'):
                _ = resolve(data.binding(), data.expected)

    def test_checkpoint_command_not_in_parent_history_rejected(self) -> None:
        with graph() as data:
            def command(row: dict[str, Json], key: str) -> None:
                commands: list[Json] = [{'tick': 2000, 'command': {'Pause': True},
                                        'accepted': True, 'count': 1, 'result_debug': 'synthetic'}]
                Node(row[key]).mapping().update(command_log=commands)

            data.change('save_load_roundtrip_s0_saved/save_main.json', lambda row: command(row, 'state'))
            for filename in ('initial.json', 'terminal.json', 'load-link.json'):
                data.change('save_load_roundtrip_s0_loaded/' + filename, lambda row: command(row, 'initial'))
            data.change('save_load_roundtrip_s0_loaded/terminal.json', lambda row: command(row, 'terminal'))
            with self.assertRaisesRegex(GateContractError, 'command sequence prefix changed'):
                _ = resolve(data.binding(), data.expected)
