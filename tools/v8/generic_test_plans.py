# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_test_run
"""Existing source-selected native occurrence plans without process projections."""
import re
from dataclasses import dataclass

from generic_gate import document, integer, require
from generic_test_history_graph import (HistoryEvidence, ROUNDTRIP, BASELINE, DEATHS, DETERMINISTIC, BRIDGE, PREGNANCY, SAVE_OCCURRENCES, extends)
from generic_test_endpoint_types import ExpectedOccurrence, bound_bytes, fields, snapshot
from mux2_gate_inputs import Json, Node

SOURCE_PINS = {
    'colony_jobs': 'a5826d9be2af9ffb7ee4ac5792935795befa0a8b5684465af7e70e1f745158c7',
    'save_load': 'fccd2174e5466798f686e7518248c41fe6374338cb78d9c0b67121d1ed1dd21a',
    'colony_world': 'a622f80b35aca286b77a650d87a9125c5f4a04e7363ea81a5dcedcab0f254a32',
    'colony_commands': 'bae084f3e439a93aaae915f9eedc423c60a8626d490b9bdaa7f40e5457328902',
}
TESTS = {
    'save_load': (ROUNDTRIP, BASELINE, DEATHS, DETERMINISTIC, BRIDGE,
                  'load_rejects_other_version', PREGNANCY),
    'colony_world': ('colony_world_shape', 'colony_founder_fix_makes_adults_when_short',
        'colony_viability_no_orders', 'colony_invariants_hold_over_8_seeds_x_10_days',
        'colony_determinism_with_the_u_script', 'colony_camp_lies_on_open_ground'),
    'colony_commands': ('commands_reject_without_the_colony_rule', 'commands_reject_out_of_map',
        'commands_reject_a_hut_on_water', 'commands_reject_a_blocked_hut', 'commands_reject_when_nothing_changes',
        'commands_reject_drafting_in_m1', 'commands_designate_zone_place_and_cancel',
        'commands_keep_the_ledger_and_move_logs_out_of_a_new_footprint',
        'replay_of_the_command_log_reproduces_the_run', 'a_cancelled_site_frees_its_legacy_seeker',
        'a_completed_site_frees_its_legacy_seeker'),
    'colony_jobs': ('jobs_chop_one_tree','jobs_haul_to_stockpile','jobs_deliver_and_build_hut',
        'jobs_two_agents_one_tree','jobs_cancel_on_hunger','jobs_cancel_area_mid_delivery',
        'jobs_stuck_cancel','jobs_children_never_work','hut_sleeps_four'),
}


@dataclass(frozen=True, slots=True)
class Planned:
    occurrence: ExpectedOccurrence
    stop: str


def fixed_plan(target: str) -> tuple[Planned, ...]:
    require(target in {'colony_world', 'colony_commands'}, target, 'unsupported fixed test family')
    rows: list[Planned] = []
    def add(test: str, seed: int, branch: str, end: int, stop: str) -> None:
        rows.append(Planned(ExpectedOccurrence(test, test, seed, branch, 0, end, True), stop))
    if target == 'colony_world':
        for test, branch, stop in (
            ('colony_viability_no_orders', 'ordinary', 'exact_14400_ticks_no_orders'),
            ('colony_invariants_hold_over_8_seeds_x_10_days', 'u_script', 'exact_14400_ticks_u_script'),
        ):
            for seed in range(8):
                add(test, seed, branch, 14400, stop)
        for seed in (0, 4):
            for branch in ('first', 'second'):
                add('colony_determinism_with_the_u_script', seed, branch, 2880, 'exact_2880_ticks_u_script')
    else:
        for branch, stop in (('original', 'exact_1200_scripted_ticks'),
                             ('recorded_replay', 'exact_1200_recorded_replay_ticks'),
                             ('idle_control', 'exact_1200_idle_control_ticks')):
            add('replay_of_the_command_log_reproduces_the_run', 3, branch, 1200, stop)
    return tuple(rows)


def save_plan(evidence: HistoryEvidence) -> tuple[Planned, ...]:
    root = evidence.root
    hashes = dict(evidence.output_hashes)
    def event(test: str, seed: int, branch: str, label: str) -> int:
        name = f'save_load_pid{evidence.pid}/{test}_s{seed}_{branch}/save_{label}.json'
        require(name in hashes, name, 'event checkpoint absent from frozen native evidence')
        return integer(document(bound_bytes(root/name, hashes[name]).decode()).field('state').field('tick'))
    death = event(DEATHS, 0, 'original', 'chain')
    pregnancy = event(PREGNANCY, 1, 'pregnancy_saved', 'event')
    queued = event(PREGNANCY, 1, 'queued_saved', 'event')
    require(20000 < death <= 100000 and pregnancy <= 40*1440 and queued <= 60*1440,
            '/events', 'native event outside source-planned limit')
    rows: list[Planned] = []
    for test, members in SAVE_OCCURRENCES.items():
        for seed, branch in members:
            if test in {ROUNDTRIP, BASELINE}:
                start, end, stop = (2000 if branch == 'loaded' else 0), 3000, 'save_at_plus_1000'
            elif test == DEATHS:
                start, end = {'continuous':(0,death+12000), 'original':(0,death),
                              'loaded_1':(death,death+3000), 'loaded_2':(death+3000,death+12000)}[branch]
                stop = ('replaced_by_chained_load' if branch in {'original','loaded_1'}
                        else 'first_natural_death_after_20000_plus_12000')
            elif test == DETERMINISTIC:
                start, end = (1500 if branch == 'loaded_no_ticks' else 0), 1500
                stop = 'resave_without_ticks' if branch == 'loaded_no_ticks' else 'tick_1500'
            elif test == BRIDGE:
                start, end, stop = (1200 if branch == 'loaded' else 0), 1500, 'tick_1500'
            else:
                require(test == PREGNANCY, test, 'unknown source-planned test')
                if branch == 'queued_probe':
                    start, end, stop = queued, queued+1, 'one_tick_after_queued_birth_load'
                else:
                    cohort = 'pregnancy' if branch.startswith('pregnancy_') else 'queued'
                    at, after = (pregnancy, 1000) if cohort == 'pregnancy' else (queued, 2000)
                    start = at if branch.endswith('_loaded') else 0
                    end = at if branch.endswith('_saved') else at+after
                    stop = f'{cohort}_event_save_only' if branch.endswith('_saved') else f'{cohort}_event_plus_{after}'
            rows.append(Planned(ExpectedOccurrence(test,test,seed,branch,start,end,True),stop))
    require(len(rows) == 46, '/plan', 'source occurrence count changed')
    return tuple(rows)


JOBS_INPUTS = {
    'rust/crates/sim-test/tests/colony_jobs.rs': SOURCE_PINS['colony_jobs'],
    'rust/crates/sim-core/src/rules/colony.rs': '3fa9680569223cb974acdcf7247f0aabf8296da707752f17ac24e94f02471946',
    'rust/crates/sim-test/tests/colony_support/mod.rs': 'b5fb68e830fee08ff7e04694a23919ec62cfdcdab3431a69aed58899db616525',
    'rust/crates/sim-test/tests/save_gate_observation/mod.rs': 'ab7c324c94846f231128040f7ccf34fbd2c8a9339a927a65962c1f2c17ce2cdf',
}
JOBS_RULES = 'rust/crates/sim-core/src/rules/colony.rs'


def hut_evidence(evidence: HistoryEvidence) -> tuple[Planned, dict[str,Json]]:
    """Bind gifted setup separately from ordinary history; no queued-command fiction."""
    inputs = dict(evidence.inputs)
    for name, digest in JOBS_INPUTS.items():
        require(inputs.get(name) == digest, name, 'unreviewed gifted-hut observer/setup source')
        _ = bound_bytes(evidence.input_root/name, digest)
    require(JOBS_RULES in inputs, JOBS_RULES, 'missing source-bound M1 bed constant')
    rules = bound_bytes(evidence.input_root/JOBS_RULES, inputs[JOBS_RULES]).decode()
    constants = [match.group(1) for match in re.finditer(r'pub const M1: Self = Self \{(.*?)\n    \};',rules,re.DOTALL)]
    require(len(constants) == 1 and re.findall(r'\bhut_beds:\s*([0-9]+)\s*,',constants[0]) == ['4'],
            JOBS_RULES, 'native terminal assertion no longer checks exactly four beds')
    root = evidence.root
    folder = f'save_load_pid{evidence.pid}/hut_sleeps_four_s2_gifted_hut'
    hashes = dict(evidence.output_hashes)
    def read(name: str) -> Node:
        path = folder+'/'+name+'.json'
        require(path in hashes,path,'missing gifted-hut checkpoint')
        return document(bound_bytes(root/path, hashes[path]).decode())
    first = read('initial').field('initial')
    initial = snapshot(first)
    require(initial.tick == 0, folder, 'must observe constructor before setup')
    names = ('before_log_gift','after_log_gift','accepted_direct_blueprint','hut_completion')
    require({path.name for path in (root/folder).iterdir()} ==
            {'initial.json','terminal.json',*(f'checkpoint_{name}.json' for name in names)},
            folder,'missing or unregistered hut evidence')
    checkpoints = [read('checkpoint_'+name) for name in names]
    for row in checkpoints:
        fields(row,'state detail')
    states = [snapshot(row.field('state')) for row in checkpoints]
    checkpoints[0].field('state').equals(first.value)
    checkpoints[1].field('state').equals(first.value)
    for before,after in zip([initial,*states],states+[snapshot(read('terminal').field('terminal'))],strict=True):
        extends(before,after,folder)
    require(all(state.tick == 0 and state.alive_ids == initial.alive_ids for state in states[:3]),
            folder,'gift/blueprint setup moved time or census')
    before,after,blueprint,completion = [row.field('detail') for row in checkpoints]
    fields(before,'intervention item count tile ground_logs log_ledger')
    fields(after,'ground_logs log_ledger requested_count tile')
    before.field('intervention').equals('direct_ledger_create_and_ground_stack_insert')
    before.field('item').equals('log')
    before.field('count').equals(12)
    after.field('requested_count').equals(12)
    tile = [integer(value) for value in before.field('tile').array()]
    require(len(tile) == 2 and all(value < 2**32 for value in tile),folder,'gift needs actual tile')
    after.field('tile').equals(before.field('tile').value)
    a,b = ([integer(value) for value in row.field('log_ledger').array()] for row in (before,after))
    require(len(a) == len(b) == 5 and b == [a[0]+12,a[1]+12,*a[2:]],folder,'gift ledger delta is not exactly12 logs')
    before.field('ground_logs').equals(a[1])
    after.field('ground_logs').equals(b[1])
    fields(blueprint,'intervention accepted_assertion_completed command')
    blueprint.field('intervention').equals('direct_apply_command')
    blueprint.field('accepted_assertion_completed').equals(True)
    command = blueprint.field('command')
    fields(command,'tick cmd kind x y')
    command.field('tick').equals(0)
    command.field('cmd').equals('place_blueprint')
    command.field('kind').equals('hut')
    x,y = integer(command.field('x')),integer(command.field('y'))
    require(x < 2**32-1 and y < 2**32-1,folder,'hut coordinate overflow')
    fields(completion,'actual_completed_ticks limit_ticks centre owned_hut_assertion_completed')
    used = integer(completion.field('actual_completed_ticks'))
    require(used < 4320 and states[3].tick == used,folder,'actual completion missing or outside original strict limit')
    completion.field('limit_ticks').equals(4320)
    completion.field('centre').equals([x+1,y+1])
    completion.field('owned_hut_assertion_completed').equals(True)
    planned = Planned(ExpectedOccurrence('hut_sleeps_four','hut_sleeps_four',2,'gifted_hut',0,used+1440,True),
                      'actual_hut_completion_plus_1440')
    setup: dict[str,Json] = {'log_gift':before.value, 'after_log_gift':after.value,
                            'direct_blueprint':blueprint.value, 'required_beds':4}
    return planned, setup
