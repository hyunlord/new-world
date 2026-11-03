"""Exact read-only baseline tick-zero contract; ordinary probe duration stays positive."""
from generic_gate import boundary, integer, require
from generic_observations import Observation
from generic_test_endpoint_types import fields
from mux2_gate_inputs import Node


@boundary
def parse_baseline_zero(node: Node) -> Observation:
    """Caller authenticates the single named source test before admitting this value."""
    fields(node, 'schema_version scene seed occurrence requested_ticks requested_terminal_tick initial_tick '+
           'initial_population terminal_tick terminal_alive terminal_complete history_complete history_source '+
           'knowledge_capability knowledge_lost deaths checkpoints')
    node.field('schema_version').equals(1)
    node.field('scene').equals('baseline')
    node.field('occurrence').equals('primary')
    for key in ('seed', 'requested_ticks', 'requested_terminal_tick', 'initial_tick', 'terminal_tick'):
        node.field(key).equals(0)
    node.field('terminal_complete').equals(True)
    node.field('history_complete').equals(True)
    node.field('history_source').equals('fresh_world_append_only_chronicle')
    capability = node.field('knowledge_capability')
    fields(capability, 'enabled source')
    capability.field('enabled').equals(False)
    capability.field('source').equals('resources.rules.colony.knowledge')
    node.field('knowledge_lost').equals(None)
    node.field('checkpoints').equals([])
    deaths = node.field('deaths')
    fields(deaths, 'starvation dehydration exposure combat old_age')
    for key in deaths.mapping():
        deaths.field(key).equals(0)
    founders = integer(node.field('initial_population'))
    alive = integer(node.field('terminal_alive'))
    require(founders == alive, '/population', 'read-only tick-zero fixture changed population')
    return Observation(0, 0, founders, alive, False, (), '[]')
