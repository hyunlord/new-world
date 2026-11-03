# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_ui
"""UI evidence from an actual outer stage; no independent Godot wait is invented."""
from dataclasses import dataclass
from pathlib import Path

from generic_gate import boundary, document, integer, require, text
from generic_observations import keys
from generic_process import file_hash, real_path
from mux2_gate_inputs import Json, Node

from generic_ui_receipts import (
    CheckpointPlan,
    InstancePlan,
    ValidatedUI,
    godot_visible_arguments,
    validate_lifecycle,
)


@dataclass(frozen=True, slots=True)
class WholeUIPlan:
    sidecar: Path
    producer: str
    argv: tuple[str, ...]
    script: str
    helper: str
    source_pins: tuple[tuple[str, str], ...]
    checkpoints: tuple[CheckpointPlan, ...]
    instances: tuple[tuple[InstancePlan, ...], ...]
    classification: str = 'native_world_producer'
    completion: str = 'producer_finish'
    no_native_reason: str = ''


@dataclass(frozen=True, slots=True)
class WholeUIEvidence:
    plan: WholeUIPlan
    validated: ValidatedUI
    emitted_pid: int
    aggregate_pid: int
    sidecar_sha256: str
    process_sha256: str
    qualification: str = 'SOURCE_BOUND_AGGREGATE_ONLY'
    independent_child_wait: bool = False


def _mapping(node: Node) -> dict[str, str]:
    return {name: text(Node(value)) for name, value in node.mapping().items()}


def check_namespace(stage: Node, directory: Path) -> None:
    """Require the original complete UI namespace, including empty directories."""
    files = _mapping(stage.field('observer_artifacts'))
    modes = {name: integer(Node(value)) for name,value in stage.field('observer_directories').mapping().items()}
    expected_files = {name:digest for name,digest in files.items() if Path(name).is_relative_to(directory)}
    expected_dirs = {name:mode for name,mode in modes.items() if Path(name).is_relative_to(directory)}
    real_path(directory, directory=True)
    require(str(directory) in expected_dirs, '/ui', 'UI root lacks original directory anchor')
    actual_files: set[str] = set()
    actual_dirs: set[str] = {str(directory)}
    for path in directory.rglob('*'):
        require(not path.is_symlink(), '/ui', 'UI output link')
        if path.is_dir():
            actual_dirs.add(str(path))
        else:
            real_path(path, directory=False)
            actual_files.add(str(path))
    require(actual_files == expected_files.keys() and actual_dirs == expected_dirs.keys(),
            '/ui', 'UI namespace changed after native return')
    for name,digest in expected_files.items():
        require(file_hash(Path(name)) == digest, '/ui', 'UI original bytes changed')
    for name,mode in expected_dirs.items():
        require(Path(name).stat().st_mode == mode, '/ui', 'UI directory mode changed')


def bind_outer(session: Node, stage: Node) -> tuple[Path, Node]:
    """Join original stage/actual aggregate receipt, retaining source-only child scope."""
    stage.field('schema').equals('whole_gate_step.v1')
    stage.field('state').equals('RETURNED')
    stage.field('native_exit').equals(0)
    stage.field('process_status').equals('PASS')
    arm = text(stage.field('arm'))
    require(arm in {'base','candidate'}, '/arm', 'unknown UI arm')
    product = session.field('arms').field(arm)
    root = Path(text(product.field('root')))
    receipt = Path(text(stage.field('process_receipt')))
    real_path(receipt, directory=False)
    require(receipt.is_relative_to(Path(text(session.field('output')))/'steps'/arm), '/process', 'foreign aggregate receipt')
    require(file_hash(receipt) == text(stage.field('process_receipt_sha256')), '/process', 'aggregate receipt changed')
    raw = document(receipt.read_text())
    for name in ('native_exit','pid','cwd','environment'):
        raw.field(name).equals(stage.field(name).value)
    raw.field('status').equals('PASS')
    raw.field('errors').equals([])
    raw.field('cwd').equals(str(root))
    raw.field('input_root').equals(str(root))
    require(integer(raw.field('pid')) > 0, '/process', 'missing actual aggregate PID')
    bash = session.field('bash')
    bash_path = Path(text(bash.field('path')))
    bash_hash = text(bash.field('sha256'))
    raw.field('executable').equals(str(bash_path))
    for key in ('expected_executable_sha256','before_executable_sha256','after_executable_sha256'):
        raw.field(key).equals(bash_hash)
    require(file_hash(bash_path)==bash_hash, '/process', 'aggregate executable changed')
    external = _mapping(session.field('external_inputs'))
    for key in ('expected_external_inputs','before_external_inputs','after_external_inputs'):
        observed_external = _mapping(raw.field(key))
        require(all(observed_external.get(name)==digest for name,digest in external.items()),
                '/source', 'aggregate external binding incomplete')
    for name,digest in external.items():
        require(file_hash(Path(name))==digest, '/source', 'original external input changed')
    raw.field('argv').equals([text(bash.field('path')), '-c', 'exec "$@" 2>&1', 'whole-step',
                              *[text(v) for v in stage.field('argv').array()]])
    for channel in ('stdout','stderr'):
        require(file_hash(receipt.parent/(channel+'.log')) == text(raw.field(channel+'_sha256')), '/process', 'aggregate output changed')
    inputs = _mapping(product.field('source_inputs'))
    for key in ('expected_inputs','before_inputs','after_inputs'):
        observed = _mapping(raw.field(key))
        require(all(observed.get(name)==digest for name,digest in inputs.items()), '/source', 'aggregate source binding incomplete')
    for name,digest in inputs.items():
        require(not Path(name).is_absolute() and '..' not in Path(name).parts, '/source', 'unsafe source path')
        require(file_hash(root/name)==digest, '/source', 'original source changed')
    return root, raw


@boundary
def collect_ui(session: Node, stage: Node, plan: WholeUIPlan) -> WholeUIEvidence:
    """Authenticate emitted UI semantics; caller supplies source-owned occurrence plan.

    The session must already be authenticated by A.load_session. Source-derived
    argv and occurrence coverage remain the family selector's responsibility.
    Emitted PID is not upgraded to an independently observed child wait.
    """
    root, outer = bind_outer(session,stage)
    environment = _mapping(stage.field('environment'))
    require(environment.get('WORLDSIM_GATE_OBSERVER')=='1', '/environment', 'observer not enabled')
    directory = Path(environment.get('WORLDSIM_GATE_UI_OUT',''))
    require(directory.is_absolute() and plan.sidecar.is_relative_to(directory), '/ui', 'foreign UI directory')
    check_namespace(stage,directory)
    files = _mapping(stage.field('observer_artifacts'))
    require(str(plan.sidecar) in files, '/ui', 'sidecar lacks original anchor')
    node = document(plan.sidecar.read_text())
    keys(node, {'schema_version','producer','invocation_id','pid','argv','user_args','project','script',
                'script_sha256','helper_sha256','complete','completion','native_exit_code','classification',
                'no_native_reason','observed_native_nodes','checkpoints','errors'})
    pid = integer(node.field('pid'))
    require(pid>0 and plan.sidecar.name==f'{plan.producer}-{pid}.json', '/ui', 'foreign producer/PID filename')
    expected: tuple[tuple[str, Json], ...] = (('schema_version',1),('producer',plan.producer),('complete',True),('errors',[]),
                       ('invocation_id',environment.get('WORLDSIM_GATE_INVOCATION_ID')),
                       ('script',plan.script),('classification',plan.classification),
                       ('no_native_reason',plan.no_native_reason),('observed_native_nodes',len(plan.instances)))
    for name,value in expected:
        node.field(name).equals(value)
    require(plan.completion in {'producer_finish','native_tree_exit'}, '/completion', 'unknown completion policy')
    node.field('completion').equals(plan.completion)
    node.field('native_exit_code').equals(0 if plan.completion=='producer_finish' else -1)
    require(text(node.field('project')).rstrip('/')==str(root), '/ui', 'foreign project')
    args,user_args = godot_visible_arguments(plan.argv)
    node.field('argv').equals(args)
    node.field('user_args').equals(user_args)
    source = _mapping(session.field('arms').field(text(stage.field('arm'))).field('source_inputs'))
    pins = dict(plan.source_pins)
    require(len(pins)==len(plan.source_pins) and bool(pins), '/source', 'missing/duplicate source policy pins')
    require(all(source.get(name)==digest for name,digest in pins.items()), '/source', 'unreviewed source semantics')
    for script,key in ((plan.script,'script_sha256'),(plan.helper,'helper_sha256')):
        require(script.startswith('res://') and script[6:] in pins, '/source', 'script/helper not source-pinned')
        node.field(key).equals(pins[script[6:]])
    if plan.classification=='synthetic_renderer_only':
        require(not plan.instances and not plan.checkpoints and bool(plan.no_native_reason), '/ui', 'synthetic producer includes native worlds')
    else:
        require(plan.classification=='native_world_producer' and bool(plan.instances) and not plan.no_native_reason, '/ui', 'invalid native producer')
    result = validate_lifecycle(node,plan.checkpoints,plan.instances)
    check_namespace(stage,directory)
    return WholeUIEvidence(plan,result,pid,integer(outer.field('pid')),files[str(plan.sidecar)],text(stage.field('process_receipt_sha256')))
