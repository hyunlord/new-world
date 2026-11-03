"""All ten original colony_jobs worlds; no mechanical-world safety exemption."""
from typing import Final, NamedTuple

from generic_gate import document, integer, require
from generic_test_endpoint_types import ExpectedOccurrence, fields
from generic_test_history_graph import HistoryEvidence
from generic_test_jobs_evidence import JobEvidence
from generic_test_jobs_interventions import adult_spawn, hunger
from generic_test_plans import Planned, hut_evidence
from mux2_gate_inputs import Json


class JobRoute(NamedTuple):
    test: str
    branch: str
    founders: int
    width: int
    logs: int
    stop: str
    extra: tuple[str, ...]


# Exact original fixtures, including both independently executed chop worlds.
ROUTES: Final = (
    JobRoute('jobs_chop_one_tree', 'primary', 1, 24, 0, 'tree_felled', ()),
    JobRoute('jobs_chop_one_tree', 'again', 1, 24, 0, 'tree_felled', ()),
    JobRoute('jobs_haul_to_stockpile', 'primary', 2, 30, 38, 'all_38_logs_in_zone', ()),
    JobRoute('jobs_deliver_and_build_hut', 'primary', 3, 30, 15, 'hut_completion_plus1', ('construction_stop',)),
    JobRoute('jobs_two_agents_one_tree', 'primary', 2, 24, 0, 'reserved_tree_felled', ()),
    JobRoute('jobs_cancel_area_mid_delivery', 'primary', 1, 30, 10, 'cancel_delivery_plus50', ('before_cancel',)),
    JobRoute('jobs_stuck_cancel', 'primary', 1, 24, 0, 'stuck_cancel', ('removed_movement',)),
    JobRoute('jobs_children_never_work', 'primary', 1, 24, 5, 'child_600_then_adult1', ('child_inserted', 'before_adult_spawn', 'after_adult_spawn')),
)


def job_plan(item: JobEvidence, route: JobRoute) -> tuple[Planned, Json]:
    _, _, founders, width, logs, stop, extra = route
    initial, terminal = item.chain(('fixture_setup', *extra, 'event_stop'))
    setup_state, setup = item.checkpoint('fixture_setup')
    fields(setup, 'recipe founding_boundary world_seed width height ground_logs log_ledger')
    setup.field('recipe').equals(item.test)
    setup.field('founding_boundary').equals('post_worker_spawn_tick_zero')
    setup.field('world_seed').equals(0)
    setup.field('width').equals(width)
    setup.field('height').equals(width)
    setup.field('ground_logs').equals(logs)
    setup.field('log_ledger').equals([logs, logs, 0, 0, 0])
    require(initial == setup_state and len(initial.alive_ids) == founders and
            initial.alive_ids == initial.raw_agent_ids and initial.births == initial.chronicle_entries == 0 and
            not initial.knowledge_enabled and initial.losses is None,
            item.folder, 'post-worker founding census or original M1 capability differs')
    event_state, event = item.checkpoint('event_stop')
    require(event_state == terminal, item.folder, 'event stop is not actual final endpoint')
    event.field('predicate').equals(stop)
    end = terminal.tick
    mutations: Json = None
    match item.test:
        case 'jobs_chop_one_tree':
            event.field('tree_absent').equals(True)
            if item.branch == 'primary':
                fields(event, 'predicate tree_absent used limit')
                event.field('limit').equals(2000)
                require(integer(event.field('used')) == end and 0 < end < 2000, item.folder, 'original strict tree limit differs')
            else:
                fields(event, 'predicate tree_absent working_ticks')
                event.field('working_ticks').equals(90)
                require(end > 0, item.folder, 'original repeated chop never advanced')
        case 'jobs_haul_to_stockpile':
            fields(event, 'predicate used limit logs_in_zone')
            event.field('limit').equals(6000)
            event.field('logs_in_zone').equals(38)
            require(integer(event.field('used')) == end and 0 < end < 6000, item.folder, 'haul stop differs')
        case 'jobs_deliver_and_build_hut':
            fields(event, 'predicate used')
            state, detail = item.checkpoint('construction_stop')
            fields(detail, 'used limit hut_present')
            detail.field('limit').equals(8000)
            detail.field('hut_present').equals(True)
            used = integer(detail.field('used'))
            event.field('used').equals(used)
            require(state.tick == used and 0 < used < 8000 and end == used + 1, item.folder, 'builder cleanup tick missing')
        case 'jobs_two_agents_one_tree':
            fields(event, 'predicate trees_felled second_worker_took_job')
            event.field('trees_felled').equals(1)
            event.field('second_worker_took_job').equals(False)
            require(1 <= end <= 2001, item.folder, 'original one tick plus search changed')
        case 'jobs_cancel_area_mid_delivery':
            fields(event, 'predicate used cancel_count')
            state, detail = item.checkpoint('before_cancel')
            fields(detail, 'used limit carried worker_id')
            detail.field('limit').equals(2000)
            detail.field('carried').equals(5)
            require(integer(detail.field('worker_id')) in state.alive_ids, item.folder, 'missing actual worker')
            used = integer(detail.field('used'))
            event.field('used').equals(used)
            event.field('cancel_count').equals(1)
            require(state.tick == used and 0 < used < 2000 and end == used + 50, item.folder, 'cancel continuation changed')
            require(len(terminal.commands) == 1 and terminal.commands[0].accepted and
                    terminal.commands[0].tick == used, item.folder, 'original submitted cancellation missing')
            document(terminal.commands[0].command_json).equals({'tick': used, 'cmd': 'cancel_area', 'x0': 20, 'y0': 20, 'x1': 22, 'y1': 22})
            require(terminal.commands[0].count == 1, item.folder, 'cancellation count differs')
        case 'jobs_stuck_cancel':
            fields(event, 'predicate stuck_ticks cancel_count tree_present')
            event.field('stuck_ticks').equals(240)
            event.field('cancel_count').equals(1)
            event.field('tree_present').equals(True)
            require(end == 241, item.folder, 'original stuck cancellation horizon changed')
            state, detail = item.checkpoint('removed_movement')
            fields(detail, 'movement_rng_absent')
            detail.field('movement_rng_absent').equals(True)
            require(state == initial, item.folder, 'movement removal changed census/history')
            mutations = detail.value
        case 'jobs_children_never_work':
            fields(event, 'predicate adult_has_job')
            event.field('adult_has_job').equals(True)
            require(end == 601, item.folder, 'original child/adult horizon changed')
            mutations = adult_spawn(item)
        case _:
            raise AssertionError('unregistered source-selected job case')
    return Planned(ExpectedOccurrence(item.test, item.test, 0, item.branch, 0, end, True), stop), {
        'setup': setup.value, 'direct_mutations': mutations}


def jobs_plan(graph: HistoryEvidence) -> tuple[tuple[Planned, ...], dict[str, Json]]:
    gifted, gifted_setup = hut_evidence(graph)
    plans: list[Planned] = []
    setups: dict[str, Json] = {'hut_sleeps_four': gifted_setup}
    for route in ROUTES:
        item = JobEvidence(graph, route.test, route.branch, 0)
        plan, setup = job_plan(item, route)
        plans.append(plan)
        setups[route.test + '/' + route.branch] = setup
    hunger_item = JobEvidence(graph, 'jobs_cancel_on_hunger', 'primary', 1)
    end, setup = hunger(hunger_item)
    plans.append(Planned(ExpectedOccurrence(hunger_item.test, hunger_item.test, 1, 'primary', 0, end, True), 'hunger_intervention_plus1'))
    setups[hunger_item.test] = setup
    plans.append(gifted)
    return tuple(plans), setups
