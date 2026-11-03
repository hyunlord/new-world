"""Three ordinary fresh-world cases only; fourteen knowledge cases stay unresolved."""
import re
from pathlib import Path
from typing import Final

from gate_test_summary import validate_cases
from generic_gate import require
from generic_observations import Observation, ordinary_pair
from generic_process import file_hash
from generic_test_endpoint import authenticate_endpoint
from generic_test_endpoint_types import ExpectedOccurrence
from generic_test_history_graph import identity, resolve_graph
from generic_whole_gate_aggregate import LIMIT, aggregate_output
from generic_whole_gate_endpoints import bind_observer
from generic_whole_gate_m1 import sections
from generic_whole_gate_projection import project_endpoint
from mux2_gate_inputs import Json, Node

TARGET: Final = 'colony_knowledge'
TESTS: Final = ("founders_are_seeded_by_the_goal_table_on_seeds_0_to_7", "the_aggregate_of_a_fresh_colony_matches_its_seeding", "clay_banks_lie_near_the_camp_on_30_of_32_seeds")
ALL_TESTS: Final = ('founders_are_seeded_by_the_goal_table_on_seeds_0_to_7', 'a_non_practitioner_is_never_given_a_gated_job', 'a_door_site_is_never_built_by_a_passer_by_without_the_doorway', 'the_quality_cap_follows_proficiency', 'talk_at_the_fire_teaches_knapping_at_350_with_the_keyed_chance', 'an_apprenticeship_makes_a_door_maker_within_four_days', 'an_apprenticeship_ends_when_the_teacher_dies', 'an_unused_skill_fades_below_its_threshold_in_20_days_and_gates_again', 'a_curious_hafter_with_stone_discovers_stone_grinding', 'the_aggregate_of_a_fresh_colony_matches_its_seeding', 'losing_the_only_door_maker_loses_doors', 'era_progress_follows_marker_practitioners_and_tops_out_at_642', 'clay_banks_lie_near_the_camp_on_30_of_32_seeds', 'the_pottery_chain_runs_from_a_clay_pit_to_a_fired_pot', 'a_ground_axe_breaks_after_120_chops', 'an_apprenticeship_whose_partners_cannot_meet_sends_nobody', 'the_evening_telling_never_spends_its_lesson_on_a_node_that_cannot_rise')
PINS: Final = {'rust/crates/sim-test/tests/colony_knowledge.rs': '1179dc45cbd09a9988afa18ecabbde927acdc378ad02cf3ed41bf9363c14520f', 'rust/crates/sim-test/tests/colony_support/mod.rs': 'b5fb68e830fee08ff7e04694a23919ec62cfdcdab3431a69aed58899db616525', 'rust/crates/sim-test/tests/save_gate_observation/mod.rs': 'ab7c324c94846f231128040f7ccf34fbd2c8a9339a927a65962c1f2c17ce2cdf', 'rust/crates/sim-test/tests/save_gate_observation/external.rs': '8ddf8a602f0eb81f819c9929a3ed1cc54133262c2d264dc126030f60ee9e2bcc', 'rust/crates/sim-test/src/lib.rs': 'd5a126519bca132d236972c8813faf27a8f58dec8c514c1e1a96c95a16ec9a49', 'rust/crates/sim-test/src/scenes.rs': '7fd693a1db13dafcbce414897ea0f98f2d40e888e8a1085532b8438c13418173', 'rust/crates/sim-test/src/probe/colony.rs': '97d9bc0216240cfb91f39e52871d4227919ba7d379cfd9c3ee8f9eb8f9111cfd'}


def occurrences() -> tuple[ExpectedOccurrence, ...]:
    return tuple(ExpectedOccurrence(test,test,seed,'primary',0,end,True)
                 for test,seeds,end in ((TESTS[0],tuple(range(8)),0),(TESTS[1],(0,3,6),1),(TESTS[2],tuple(range(32)),0))
                 for seed in seeds)


def check_original_cases(raw: str, step: int) -> None:
    headers=list(re.finditer(r'^\s*(?:Running (?:unittests )?[^\n]+|Doc-tests [^\n]+)\s*$',raw,re.MULTILINE))
    selected=[raw[h.start():headers[i+1].start() if i+1<len(headers) else len(raw)]
              for i,h in enumerate(headers) if re.match(r'\s*Running tests/colony_knowledge\.rs \(',h[0])]
    require(len(selected)==1,'/knowledge','missing or repeated original target')
    parsed=sections(selected[0]);require(len(parsed)==1,'/knowledge','ambiguous native section')
    require(parsed[0].profile==('debug' if step==4 else 'release'),'/knowledge','wrong actual profile')
    validate_cases(parsed[0].summary,ALL_TESTS,())


def collect(session: Node, stage: Node) -> tuple[Observation, ...]:
    step=stage.field('step').value;require(step in (4,5),'/knowledge','workspace4/5 only')
    assert isinstance(step,int)
    raw,_=aggregate_output(session,stage);check_original_cases(raw,step)
    product=session.field('arms').field(stage.field('arm').text())
    root=Path(product.field('root').text())
    inputs={name:Node(value).text() for name,value in product.field('source_inputs').mapping().items()}
    for name,digest in PINS.items():
        require(inputs.get(name)==digest and file_hash(root/name)==digest,name,'unreviewed subset source')
    evidence,graph,folder=bind_observer(stage,root,inputs,TARGET,TESTS)
    plans=occurrences()
    require({p.name for p in folder.iterdir()}=={identity(row) for row in plans},'/knowledge','missing or extra observed subset occurrence')
    endpoints=tuple(authenticate_endpoint(evidence,row) for row in plans)
    histories=resolve_graph(graph,plans,{identity(row):endpoint for row,endpoint in zip(plans,endpoints,strict=True)})
    for endpoint in endpoints:
        require(len(endpoint.initial.raw_agent_ids)==len(endpoint.initial.alive_ids)==10, '/knowledge', 'fresh colony founder census differs')
        require(endpoint.stop_reason=='original_endpoint' and not endpoint.initial.commands and not endpoint.terminal.commands,
                '/knowledge','changed stop or command intervention')
    return tuple(project_endpoint(endpoint,history) for endpoint,history in zip(endpoints,histories,strict=True))


def evaluate_family(session: Node, base_stage: Node, candidate_stage: Node) -> dict[str, Json]:
    base_stage.field('arm').equals('base');candidate_stage.field('arm').equals('candidate')
    candidate_stage.field('step').equals(base_stage.field('step').value)
    left,right=collect(session,base_stage),collect(session,candidate_stage)
    safety: list[Json]=[ordinary_pair(a,b) for a,b in zip(left,right,strict=True)]
    coverage: list[Json]=[{'target':TARGET,'test':row.test,'seed':row.seed,'branch':row.branch,'ordinary_safety':True,
                          'scope':'THREE_FRESH_CASES_ONLY'} for row in occurrences()]
    return {'status':'INCOMPLETE','scope':'THREE_OF_SEVENTEEN_KNOWLEDGE_CASES',
            'safety':safety,'coverage':coverage,'uncovered_tests':[t for t in ALL_TESTS if t not in TESTS],
            'target_gap_retained':True,'qualification':LIMIT}
