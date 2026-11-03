"""Synthetic checkpoint fixture; aggregate authentication is mocked explicitly by tests."""
import json
import tempfile
from pathlib import Path
from typing import final

import generic_process
from colony_ffi_fixture import state as blank_state
from generic_process import file_hash
from generic_test_history_graph import HistoryEvidence
from generic_test_jobs_plans import ROUTES
from generic_test_plans import JOBS_INPUTS, TESTS
from mux2_gate_inputs import Json, Node

ROOT = Path(generic_process.__file__).resolve().parents[2]
DRAFT = Path(__file__).resolve().parents[2]


def state(tick: int, founders: int, knowledge: bool) -> dict[str, Json]:
    result = blank_state()
    result.update(tick=tick, alive=founders, alive_ids=list(range(1, founders + 1)),
                  raw_agent_population=founders, raw_agent_ids=list(range(1, founders + 1)),
                  knowledge_capability={'enabled': knowledge, 'source': 'resources.rules.colony.knowledge'},
                  knowledge_lost=[] if knowledge else None)
    return result


@final
class Fixture:
    def __init__(self, root: Path) -> None:
        root = root.resolve()
        self.root = root
        self.source = root / 'source'
        self.observer = root / 'stage/observer'
        self.parent = self.observer / 'save_load_pid123'
        self.parent.mkdir(parents=True)
        self.inputs: dict[str, str] = {}
        for name, expected in JOBS_INPUTS.items():
            path = self.source / name
            path.parent.mkdir(parents=True, exist_ok=True)
            source = DRAFT / name if (DRAFT / name).exists() else ROOT / name
            _ = path.write_bytes(source.read_bytes())
            assert file_hash(path) == expected
            self.inputs[name] = expected
        for route in ROUTES:
            checkpoints: list[tuple[str, int, dict[str, Json]]] = []
            setup: dict[str, Json] = {'recipe': route.test, 'founding_boundary': 'post_worker_spawn_tick_zero',
                                      'world_seed': 0, 'width': route.width, 'height': route.width,
                                      'ground_logs': route.logs, 'log_ledger': [route.logs, route.logs, 0, 0, 0]}
            checkpoints.append(('fixture_setup', 0, setup))
            event: dict[str, Json] = {'predicate': route.stop}
            end = 100
            match route.test:
                case 'jobs_chop_one_tree':
                    event['tree_absent'] = True
                    event.update({'used': 100, 'limit': 2000} if route.branch == 'primary' else {'working_ticks': 90})
                case 'jobs_haul_to_stockpile':
                    event.update(used=100, limit=6000, logs_in_zone=38)
                case 'jobs_deliver_and_build_hut':
                    checkpoints.append(('construction_stop', 100, {'used': 100, 'limit': 8000, 'hut_present': True}))
                    event['used'] = 100
                    end = 101
                case 'jobs_two_agents_one_tree':
                    event.update(trees_felled=1, second_worker_took_job=False)
                case 'jobs_cancel_area_mid_delivery':
                    checkpoints.append(('before_cancel', 100, {'used': 100, 'limit': 2000, 'carried': 5, 'worker_id': 1}))
                    event.update(used=100, cancel_count=1)
                    end = 150
                case 'jobs_stuck_cancel':
                    checkpoints.append(('removed_movement', 0, {'movement_rng_absent': True}))
                    event.update(stuck_ticks=240, cancel_count=1, tree_present=True)
                    end = 241
                case 'jobs_children_never_work':
                    checkpoints.extend([('child_inserted', 0, {'child_component_present': True}),
                                        ('before_adult_spawn', 600, {'direct_spawn': True, 'position': [5, 6]}),
                                        ('after_adult_spawn', 600, {'direct_spawn': True, 'position': [5, 6], 'adult_id': 2})])
                    event['adult_has_job'] = True
                    end = 601
                case _:
                    raise AssertionError('fixture route')
            checkpoints.append(('event_stop', end, event))
            self.world(route.test, route.branch, 0, route.founders, False, end, route.stop, checkpoints)
        self.world('jobs_cancel_on_hunger', 'primary', 1, 3, True, 101, 'hunger_intervention_plus1', [
            ('before_log_gift', 0, {'tile': [25, 10], 'count': 20, 'log_ledger': [0, 0, 0, 0, 0]}),
            ('fixture_setup', 0, {'recipe': 'jobs_cancel_on_hunger', 'source': [25, 10], 'cell': [25, 24], 'log_ledger': [20, 20, 0, 0, 0]}),
            ('before_hunger_intervention', 100, {'predicate': 'log_carrier_far_from_hearth', 'carrier_id': 1, 'load': 5,
                                               'position': [25, 10], 'hearth': [10, 10], 'meal_tile': [30, 10], 'food_count': 10, 'requested_hunger': 80.0, 'meal_tile_empty': True}),
            ('after_hunger_intervention', 100, {'carrier_id': 1, 'actual_hunger': 80.0, 'meal_tile': [30, 10], 'direct_food_source': 'Debug', 'requested_food_count': 10, 'meal_count': 10, 'meal_is_porridge': True}),
            ('event_stop', 101, {'predicate': 'needs_cancel_within_one_tick', 'carrier_id': 1, 'cancel_before': 0, 'cancel_after': 1})])
        self.world('hut_sleeps_four', 'gifted_hut', 2, 3, True, 1640, 'actual_hut_completion_plus_1440', [
            ('before_log_gift', 0, {'intervention': 'direct_ledger_create_and_ground_stack_insert', 'item': 'log', 'count': 12, 'tile': [5, 5], 'ground_logs': 0, 'log_ledger': [0, 0, 0, 0, 0]}),
            ('after_log_gift', 0, {'ground_logs': 12, 'log_ledger': [12, 12, 0, 0, 0], 'requested_count': 12, 'tile': [5, 5]}),
            ('accepted_direct_blueprint', 0, {'intervention': 'direct_apply_command', 'accepted_assertion_completed': True, 'command': {'tick': 0, 'cmd': 'place_blueprint', 'kind': 'hut', 'x': 10, 'y': 10}}),
            ('hut_completion', 200, {'actual_completed_ticks': 200, 'limit_ticks': 4320, 'centre': [11, 11], 'owned_hut_assertion_completed': True})])

    def world(self, test: str, branch: str, seed: int, founders: int, knowledge: bool, end: int,
              stop: str, checkpoints: list[tuple[str, int, dict[str, Json]]]) -> None:
        folder = self.parent / f'{test}_s{seed}_{branch}'
        folder.mkdir()
        initial = state(0, founders, knowledge)
        terminal = state(end, founders + int(test == 'jobs_children_never_work'), knowledge)
        if test == 'jobs_cancel_area_mid_delivery':
            terminal['command_log'] = [{'tick': 100, 'command': {'tick': 100, 'cmd': 'cancel_area', 'x0': 20, 'y0': 20, 'x1': 22, 'y1': 22},
                                        'accepted': True, 'count': 1, 'result_debug': 'synthetic accepted'}]
        self.write(folder / 'initial.json', {'schema_version': 1, 'test': test, 'seed': seed, 'branch': branch,
                                            'pid': 123, 'initial': initial, 'history_complete': True, 'status': 'PENDING_NATIVE_TEST_RESULT'})
        self.write(folder / 'terminal.json', {'schema_version': 1, 'initial': initial, 'founding': initial, 'terminal': terminal,
                                             'completed_ticks': end, 'stop_reason': stop, 'prefix': None, 'history_complete': True,
                                             'loaded_prefix_requires_external_authentication': False, 'status': 'ENDPOINT_REACHED_NATIVE_TEST_RESULT_REQUIRED'})
        for name, tick, detail in checkpoints:
            value = terminal if name == 'event_stop' else state(tick, founders, knowledge)
            if name == 'after_adult_spawn':
                value = state(tick, founders + 1, knowledge)
            self.write(folder / f'checkpoint_{name}.json', {'state': value, 'detail': detail})

    @staticmethod
    def write(path: Path, value: Json) -> None:
        _ = path.write_text(json.dumps(value))

    def stage(self, arm: str = 'base') -> Node:
        return Node({'step': 4, 'arm': arm, 'process_receipt': str(self.root / 'stage/process/process.json'),
                     'environment': {'WORLDSIM_GATE_OBSERVER': '1', 'WORLDSIM_GATE_TEST_OUT': str(self.observer)},
                     'observer_artifacts': {str(p): file_hash(p) for p in self.observer.rglob('*') if p.is_file()},
                     'observer_directories': {str(p): p.stat().st_mode for p in (self.observer, *self.observer.rglob('*')) if p.is_dir()}})

    def session(self) -> Node:
        return Node({'arms': {arm: {'root': str(self.source), 'source_inputs': dict(self.inputs)} for arm in ('base', 'candidate')}})

    def graph(self) -> HistoryEvidence:
        files = tuple((p.relative_to(self.observer).as_posix(), file_hash(p)) for p in self.observer.rglob('*') if p.is_file())
        return HistoryEvidence(self.observer, 123, files, self.source, tuple(self.inputs.items()))

    def raw(self) -> str:
        names = TESTS['colony_jobs']
        return 'Running tests/colony_jobs.rs (target/debug/deps/colony_jobs-synthetic)\n' + ''.join(f'test {name} ... ok\n' for name in names) + f'test result: ok. {len(names)} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n'


def source_path(name: str) -> Path:
    draft = DRAFT / name
    return draft if draft.exists() else ROOT / name


def child_emission() -> str:
    """Extra nine synthetic worlds for the existing harmless child integration fixture."""
    with tempfile.TemporaryDirectory() as directory:
        fixture = Fixture(Path(directory))
        payload = {str(path.relative_to(fixture.parent)): path.read_text()
                   for path in fixture.parent.rglob('*.json')
                   if not path.parent.name.startswith('hut_sleeps_four_')}
    return "\nfor name, raw in " + repr(payload) + ".items():\n p=parent/name;p.parent.mkdir(exist_ok=True);r=json.loads(raw)\n if 'pid' in r:r['pid']=os.getpid()\n p.write_text(json.dumps(r))\n"
