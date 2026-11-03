"""Direct fixture mutations remain explicit, separate from native command history."""
from generic_gate import integer, require
from generic_test_endpoint_types import fields
from generic_test_jobs_evidence import JobEvidence, coordinates
from mux2_gate_inputs import Json


def adult_spawn(item: JobEvidence) -> dict[str, Json]:
    before, detail = item.checkpoint('before_adult_spawn')
    after, added = item.checkpoint('after_adult_spawn')
    fields(detail, 'direct_spawn position')
    fields(added, 'direct_spawn position adult_id')
    for row in (detail, added):
        row.field('direct_spawn').equals(True)
        row.field('position').equals([5, 6])
    identifier = integer(added.field('adult_id'))
    require(before.tick == after.tick == 600 and before.births == after.births == 0,
            item.folder, 'direct spawn must not become a natural birth')
    require(identifier not in before.raw_agent_ids and
            after.raw_agent_ids == tuple(sorted((*before.raw_agent_ids, identifier))) and
            after.alive_ids == after.raw_agent_ids, item.folder, 'missing actual adult census addition')
    _, child = item.checkpoint('child_inserted')
    fields(child, 'child_component_present')
    child.field('child_component_present').equals(True)
    return {'before': detail.value, 'after': added.value}


def hunger(item: JobEvidence) -> tuple[int, dict[str, Json]]:
    initial, terminal = item.chain(('before_log_gift', 'fixture_setup', 'before_hunger_intervention',
                                    'after_hunger_intervention', 'event_stop'))
    gift_state, gift = item.checkpoint('before_log_gift')
    setup_state, setup = item.checkpoint('fixture_setup')
    fields(gift, 'tile count log_ledger')
    fields(setup, 'recipe source cell log_ledger')
    setup.field('recipe').equals('jobs_cancel_on_hunger')
    gift.field('count').equals(20)
    setup.field('source').equals(gift.field('tile').value)
    _ = coordinates(setup.field('cell'))
    _ = coordinates(gift.field('tile'))
    left = [integer(value) for value in gift.field('log_ledger').array()]
    right = [integer(value) for value in setup.field('log_ledger').array()]
    require(len(left) == len(right) == 5 and right == [left[0] + 20, left[1] + 20, *left[2:]],
            item.folder, 'direct log gift differs from original20')
    require(initial.tick == gift_state.tick == setup_state.tick == 0 and
            initial.alive_ids == gift_state.alive_ids == setup_state.alive_ids,
            item.folder, 'setup changed founder census or time')
    before, detail = item.checkpoint('before_hunger_intervention')
    after, changed = item.checkpoint('after_hunger_intervention')
    fields(detail, 'predicate carrier_id load position hearth meal_tile food_count requested_hunger meal_tile_empty')
    fields(changed, 'carrier_id actual_hunger meal_tile direct_food_source requested_food_count meal_count meal_is_porridge')
    detail.field('predicate').equals('log_carrier_far_from_hearth')
    identifier = integer(detail.field('carrier_id'))
    require(identifier in before.alive_ids and integer(detail.field('load')) > 0,
            item.folder, 'actual carrier must be alive with a positive load')
    position, hearth = coordinates(detail.field('position')), coordinates(detail.field('hearth'))
    require(max(abs(a - b) for a, b in zip(position, hearth, strict=True)) > 10,
            item.folder, 'carrier is not far from hearth')
    meal = coordinates(detail.field('meal_tile'))
    require(max(abs(a - b) for a, b in zip(position, meal, strict=True)) >= 3,
            item.folder, 'meal does not satisfy original separation')
    detail.field('food_count').equals(10)
    detail.field('meal_tile_empty').equals(True)
    detail.field('requested_hunger').equals(80.0)
    changed.field('carrier_id').equals(identifier)
    changed.field('actual_hunger').equals(80.0)
    changed.field('meal_tile').equals(detail.field('meal_tile').value)
    changed.field('direct_food_source').equals('Debug')
    changed.field('requested_food_count').equals(10)
    changed.field('meal_count').equals(10)
    changed.field('meal_is_porridge').equals(True)
    require(1 <= before.tick <= 4000 and before == after and terminal.tick == before.tick + 1,
            item.folder, 'intervention endpoint differs from original search plus one tick')
    event_state, event = item.checkpoint('event_stop')
    fields(event, 'predicate carrier_id cancel_before cancel_after')
    event.field('predicate').equals('needs_cancel_within_one_tick')
    event.field('carrier_id').equals(identifier)
    require(event_state == terminal and integer(event.field('cancel_after')) == integer(event.field('cancel_before')) + 1,
            item.folder, 'missing original one cancellation oracle')
    return terminal.tick, {'gift': gift.value, 'setup': setup.value, 'intervention': detail.value, 'after': changed.value}
