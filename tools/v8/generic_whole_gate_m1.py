"""Original M1 aggregate case inventory; never an independent executable listing."""
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Final

from gate_test_summary import TestSummary, parse_log, validate_cases
from generic_gate import boundary, require, text
from generic_m1_commands import BODY, COMMANDS, authenticate_source
from generic_process import file_hash
from generic_sources import manifest_hash
from generic_whole_gate_aggregate import LIMIT, aggregate_output
from mux2_gate_inputs import Json, Node

FFI: Final = ('colony_reads_follow_the_orders','agent_job_columns_match_the_snapshot_rows')
PROBE: Final = (
 'probe::colony::tests::colony_args_parse_and_reject',
 'probe::colony::tests::ring_order_is_chebyshev_rings_in_y_x_order',
 'probe::colony::tests::a_recorded_command_log_replays_to_a_byte_identical_summary',
 'probe::colony::tests::an_m2_command_log_with_bills_and_priorities_replays_identically',
 'probe::colony::tests::physical_food_commands_round_trip_with_player_and_steward_provenance',
 'probe::colony::tests::command_json_round_trips_through_resolve',
 'probe::colony::mux2_args_tests::evaluation_requires_explicit_arm_and_equivalent_inputs',
 'probe::colony::gate_environment_tests::environment_and_flag_are_independent_opt_ins',
)
SOURCE_PINS: Final = {'rust/crates/sim-test/tests/engine_colony_ffi.rs': '0666f7abbb4fd7deb11cabb0ec8347c231a5c546cee16d7c94561a327230d366', 'rust/crates/sim-test/src/probe/colony.rs': 'b04d435eaf9be48eaee11a631982fa44b932fbca5f484da7c9e263cfdae30365', 'rust/crates/sim-test/src/probe/gate_observation_mirror.rs': '3250c195fd648fd323f57b5d6508ca136cc63d3e940f35f35df65ad11c97b7e3', 'rust/crates/sim-test/src/probe/gate_observation.rs': '23d328e105091e43c63eb24ae598e7c9901bf537e69d3d1bffb3b6fadc75fe61', 'tools/v8/test_locale_trf.gd': 'e5ab7e76920ce98f68f923ceb37eb434d350e48bd3cc59fb8910cfcba9db9487', 'rust/crates/sim-bridge/src/ffi/node.rs': '8f624202216995c4cee88eaf89d189b3041965889410c24cf41e8a5575785af1', 'rust/crates/sim-core/Cargo.toml': '667c583914926240faf8514f0d6450c864322a868d5d1b10a2933f753bc871b7', 'tools/v8/run_bounded.sh': '64b0d037c1a854add96cccf25ae7a0bcfc40be78ab4406f092514e4891009bf7', 'tools/v8/m2_u_script.json': '78261b13ce779811788945c3c77eed4eb5c709c7292fa40a849668520ce3bb75'}
SOURCE_TREES: Final = {'rust/crates/sim-systems/src/runtime/jobs': '21fcddf3f0afd2f48cd3c2e64e228f512df35ec14395eb8691fc6a10530eb2db', 'rust/crates/sim-core/src/colony': '71f78e5657a11e7ef4fcca3a693bd267adf5879148f9172dcd25d5086161aa41'}


@dataclass(frozen=True,slots=True)
class Section:
    target: str
    profile: str
    binary: str
    summary: TestSummary
    raw: str


def sections(raw:str) -> tuple[Section,...]:
    headers=list(re.finditer(r'^\s*Running (?:unittests )?([^\s(]+)\s+\(([^\n]+)\)\s*$',raw,re.MULTILINE))
    result:list[Section]=[]
    for index,match in enumerate(headers):
        payload=raw[match.end():headers[index+1].start() if index+1<len(headers) else len(raw)]
        binary=Path(match.group(2));profile='release' if 'release' in binary.parts else ('debug' if 'debug' in binary.parts else 'UNKNOWN')
        target=Path(match.group(1)).stem
        if match.group(1)=='src/lib.rs':
            candidates=[name for name in ('sim_test','sim_systems','sim_core') if binary.name.startswith(name+'-')]
            require(len(candidates)==1,'/library','unrecognized original library binary label')
            target=candidates[0]
        summary=parse_log(payload)
        names=[row.name for row in summary.outcomes]
        require(summary.summaries==1 and bool(names) and len(names)==len(set(names)),target,'empty/duplicate/multiple native inventory')
        require(summary.failed==summary.measured==0 and all(row.status in {'ok','ignored'} for row in summary.outcomes),target,'native case failure')
        running=[line.strip() for line in payload.splitlines() if line.strip().startswith('running ')]
        require(running==[f'running {len(names)} '+('test' if len(names)==1 else 'tests')],target,'native running count differs from actual cases')
        result.append(Section(target,profile,str(binary),summary,payload))
    return tuple(result)


def m1_cases(raw:str) -> tuple[Section,...]:
    parsed=sections(raw)
    expected=('colony_commands','colony_jobs','colony_world','engine_colony_ffi','sim_test','sim_systems','sim_core')
    require(len(parsed)==7 and {row.target for row in parsed[:4]}==set(COMMANDS[0].targets)
            and tuple(row.target for row in parsed[4:])==expected[4:],'/m1','source-selected original M1 producer inventory/order differs')
    require(all(row.profile=='release' for row in parsed),'/m1','original M1 requires release profile')
    by_target={row.target:row for row in parsed}
    validate_cases(by_target['engine_colony_ffi'].summary,FFI,())
    validate_cases(by_target['sim_test'].summary,PROBE,())
    for target,command in zip(expected[4:],COMMANDS[1:],strict=True):
        require(command.filter is not None and all(command.filter in case.name for case in by_target[target].summary.outcomes),target,'case outside original substring selection')
        require(by_target[target].summary.ignored==0,target,'unregistered ignored mechanism case')
    return parsed


def check_sources(session:Node,stage:Node) -> None:
    product=session.field('arms').field(text(stage.field('arm')));root=Path(text(product.field('root')))
    inputs={name:text(Node(value)) for name,value in product.field('source_inputs').mapping().items()}
    for name,digest in SOURCE_PINS.items():
        require(inputs.get(name)==digest and file_hash(root/name)==digest,name,'changed M1 source classification')
    for prefix,digest in SOURCE_TREES.items():
        actual={p.relative_to(root).as_posix():file_hash(p) for p in (root/prefix).rglob('*.rs')}
        require({name:value for name,value in inputs.items() if name.startswith(prefix+'/') and name.endswith('.rs')}==actual,
                prefix,'M1 source classification tree not bound')
        manifest: dict[str, Json] = dict(actual)
        require(manifest_hash(manifest)==digest,prefix,'unregistered M1 mechanism tree')


@boundary
def collect_m1(session:Node,stage:Node) -> dict[str,Json]:
    stage.field('step').equals(17)
    check_sources(session,stage)
    harness=session.field('harness');gate=Path(text(harness.field('root')))/'tools/v8/gate.sh'
    _=authenticate_source(gate)
    raw,pid=aggregate_output(session,stage)
    stage.field('argv').equals(['bash','-c',BODY,str(Path(text(session.field('output')))/text(stage.field('arm')))])
    parsed=m1_cases(raw)
    # Original synchronous chain waits for bounded Godot; no invented independent wait record.
    require(any(line.startswith('TRF_TEST_PASS') for line in parsed[-1].raw.splitlines()),'/locale','missing original locale oracle')
    require('SCRIPT ERROR' not in parsed[-1].raw,'/locale','locale script error')
    return {'step':17,'status':'PASS','aggregate_pid':pid,'aggregate_exit':0,'profile':'release',
            'qualification':LIMIT,'case_inventory_authority':'ACTUAL_AGGREGATE_OUTPUT_NOT_INDEPENDENT_LISTING',
            'sections':[{'target':row.target,'binary_label':row.binary,'selected':[case.name for case in row.summary.outcomes],
                         'passed':row.summary.passed,'ignored':row.summary.ignored,'filtered_out':row.summary.filtered_out} for row in parsed],
            'locale':{'scope':'catalogue_and_formatting_no_world','original_bounded_chain_returned':True,
                      'independent_godot_wait':'NOT_RECORDED'},'ordinary_probe_replays':'SEPARATE_REQUIRED_OBSERVATIONS'}
