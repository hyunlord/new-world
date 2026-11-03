"""Source-selected appearance worlds and their actual event stop."""
from typing import Final

from generic_gate import boundary, document, integer, require
from generic_test_endpoint_types import ExpectedOccurrence, bound_bytes, fields, snapshot
from generic_test_history_graph import HistoryEvidence, extends
from generic_test_plans import Planned

SAVE_TEST: Final = 'appearance_is_a_pure_function_across_save_load_and_a_twin'
BASELINE_TEST: Final = 'the_baseline_rows_are_consistent'
KIN_TEST: Final = 'kin_columns_line_up_with_the_snapshot_and_the_kin_component'
RENDER_TEST: Final = 'the_text_renderer_people_json_carries_the_bridge_looks'


@boundary
def appearance_plan(evidence: HistoryEvidence) -> tuple[Planned, ...]:
    """Resolve five Watch occurrences without inventing the kin event horizon."""
    hashes = dict(evidence.output_hashes)
    folder = f'save_load_pid{evidence.pid}/{KIN_TEST}_s3_primary'

    def read(name: str):
        relative = f'{folder}/{name}.json'
        require(relative in hashes, relative, 'missing source-selected appearance event evidence')
        return document(bound_bytes(evidence.root / relative, hashes[relative]).decode())

    event = read('checkpoint_event_stop')
    fields(event, 'state detail')
    detail = event.field('detail')
    fields(detail, 'predicate days_advanced ticks_per_day maximum_days mother_rows partner_rows')
    detail.field('predicate').equals('mother_and_partner_present')
    detail.field('maximum_days').equals(120)
    days = integer(detail.field('days_advanced'))
    ticks_per_day = integer(detail.field('ticks_per_day'))
    require(0 <= days <= 120 and ticks_per_day > 0, '/kin', 'event exceeds original source loop')
    require(integer(detail.field('mother_rows')) > 0 and integer(detail.field('partner_rows')) > 0,
            '/kin', 'source event predicate not reached')
    state = snapshot(event.field('state'))
    terminal = snapshot(read('terminal').field('terminal'))
    initial = snapshot(read('initial').field('initial'))
    require(state.tick == terminal.tick == days * ticks_per_day, '/kin', 'event coordinate differs')
    require(initial.tick == 0 and not initial.knowledge_enabled and not terminal.knowledge_enabled,
            '/kin', 'baseline capability or initial coordinate differs')
    extends(initial, state, '/kin')
    extends(state, terminal, '/kin')
    rows = (
        (SAVE_TEST, 3, 'primary', 0, 3000, '3000_ticks'),
        (SAVE_TEST, 3, 'twin', 0, 2000, '2000_ticks'),
        (SAVE_TEST, 3, 'loaded', 2000, 3000, '3000_ticks'),
        (BASELINE_TEST, 0, 'primary', 0, 100, '100_ticks'),
        (KIN_TEST, 3, 'primary', 0, terminal.tick, 'mother_and_partner_present'),
    )
    return tuple(Planned(ExpectedOccurrence(test, test, seed, branch, start, end, True), stop)
                 for test, seed, branch, start, end, stop in rows)
