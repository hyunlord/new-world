"""Eight original ASCII worlds, including the authentic first-carrier stop."""
from typing import Final

from generic_gate import boundary, document, integer, require
from generic_test_endpoint_types import (
    ExpectedOccurrence,
    Snapshot,
    bound_bytes,
    fields,
    snapshot,
)
from generic_test_history_graph import HistoryEvidence, extends
from generic_test_plans import Planned

GRID: Final = 'the_grid_has_one_character_per_tile_of_the_map_or_rect'
PEOPLE: Final = 'every_living_person_is_drawn_where_the_agent_snapshot_puts_them'
HEARTH: Final = 'the_hearth_star_is_where_the_building_snapshot_says'
TREES: Final = 'standing_trees_are_t_and_felled_ones_leave_a_stump_for_a_day'
CARRIER: Final = 'awake_carriers_have_a_plus_on_their_right'
CROP: Final = 'a_rect_crops_the_full_grid_exactly'
TESTS: Final = (GRID, PEOPLE, HEARTH, TREES, CARRIER, CROP)
COMMANDS: Final = ('designate_chop_tiles', 'add_stockpile', 'place_blueprint')


@boundary
def command_boundary(state: Snapshot) -> None:
    """The original half-open replay has not issued tick360 commands at360."""
    expected = () if state.tick <= 360 else COMMANDS
    require(len(state.commands) == len(expected), '/ascii', 'original script command count differs at boundary')
    for record, name in zip(state.commands, expected, strict=True):
        require(record.tick == 360, '/ascii', 'unregistered command tick')
        command = document(record.command_json)
        command.field('cmd').equals(name)
        command.field('tick').equals(360)


@boundary
def ascii_plan(evidence: HistoryEvidence) -> tuple[Planned, ...]:
    hashes = dict(evidence.output_hashes)
    folder = f'save_load_pid{evidence.pid}/{CARRIER}_s100_primary'

    def read(name: str):
        relative = f'{folder}/{name}.json'
        require(relative in hashes, relative, 'missing carrier source event evidence')
        return document(bound_bytes(evidence.root / relative, hashes[relative]).decode())

    start = read('checkpoint_search_start')
    fields(start, 'state detail')
    start.field('detail').equals({'start_tick': 600})
    start_state = snapshot(start.field('state'))
    require(start_state.tick == 600, '/carrier', 'search starts at original tick600')
    event = read('checkpoint_event_stop')
    fields(event, 'state detail')
    detail = event.field('detail')
    fields(detail, 'predicate checked_carriers start_tick maximum_additional_ticks actual_tick attempts_before_success')
    detail.field('predicate').equals('awake_carrier_free_right_tile')
    detail.field('start_tick').equals(600)
    detail.field('maximum_additional_ticks').equals(1440)
    actual = integer(detail.field('actual_tick'))
    require(600 <= actual < 2040 and integer(detail.field('checked_carriers')) > 0,
            '/carrier', 'original successful predicate or checked-before-tick horizon missing')
    detail.field('attempts_before_success').equals(actual - 600)
    state = snapshot(event.field('state'))
    initial = snapshot(read('initial').field('initial'))
    terminal = snapshot(read('terminal').field('terminal'))
    require(initial.tick == 0 and state.tick == terminal.tick == actual, '/carrier', 'event/terminal coordinates differ')
    extends(initial, start_state, '/carrier')
    extends(start_state, state, '/carrier')
    extends(state, terminal, '/carrier')
    command_boundary(start_state)
    rows = ((GRID, 'primary', 0), (PEOPLE, 'primary', 900), (HEARTH, 'primary', 900),
            (TREES, 'before_commands', 360), (TREES, 'felled', 900), (TREES, 'expired', 2340),
            (CARRIER, 'primary', actual), (CROP, 'primary', 900))
    return tuple(Planned(ExpectedOccurrence(test, test, 100, branch, 0, end, True),
                         'first_awake_carrier' if test == CARRIER else 'original_endpoint')
                 for test, branch, end in rows)
