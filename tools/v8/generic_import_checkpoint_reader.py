"""Read original import controller records; never adopt a fresh cache inventory."""
from gate_oracle_child import IMPORT_COMMAND, SUITE_SCHEMA, SuitePlan, suite_environment
from gate_stall_child import range_hash
from generic_gate import document, integer, require
from generic_process import file_hash
from generic_test_endpoint_types import fields
from mux2_gate_inputs import Json, Node


def controller_checkpoint(plan: SuitePlan, receipt: Node) -> Node:
    """Caller must bind this plan and complete artifact tree to original source authority."""
    require(plan.import_pair and len(plan.rows) == 2, '/imports', 'original import pair required')
    prepared = document((plan.evidence/'suite-prepared.json').read_text())
    start = document((plan.evidence/'suite-start.json').read_text())
    terminal = document((plan.evidence/'suite-terminal.json').read_text())
    for node in (prepared, start, terminal):
        node.field('schema').equals(SUITE_SCHEMA)
        node.field('plan_sha256').equals(plan.digest)
    shell = integer(start.field('shell_pid'))
    require(shell > 0 and integer(prepared.field('owner_pid')) > 0, '/pid', 'missing original owner')
    start.field('owner_pid').equals(prepared.field('owner_pid').value)
    start.field('cwd').equals(str(plan.root))
    start.field('argv').equals(['bash', '-c', IMPORT_COMMAND+'; '+IMPORT_COMMAND])
    terminal.field('shell_pid').equals(shell)
    receipt.field('shell_pid').equals(shell)
    terminal.field('shell_exit').equals(0)
    terminal.field('settlement').equals('SETTLED')
    terminal.field('completed').equals(['import.first', 'import.second'])
    terminal.field('wrapper_waits_observed').equals(False)
    terminal.field('oracle_admission').equals('CALLER_REQUIRED')
    terminal.field('error').equals('')
    logs: dict[str, Json] = {str(row.log): file_hash(row.log) for row in plan.rows}
    terminal.field('closed_log_sha256').equals(logs)
    previous_end = 0
    checkpoint: Node | None = None
    for row in plan.rows:
        request = document((row.evidence/'request.json').read_text())
        fields(request, 'schema plan_sha256 ordinal identity wrapper_pid parent_pid cwd argv limit log_device log_inode')
        expected: dict[str, Json] = {'schema': SUITE_SCHEMA, 'plan_sha256': plan.digest,
                'ordinal': row.ordinal, 'identity': row.identity, 'parent_pid': shell,
                'cwd': str(plan.root), 'argv': list(row.argv), 'limit': 600}
        for key, value in expected.items():
            request.field(key).equals(value)
        wrapper = integer(request.field('wrapper_pid'))
        require(wrapper > 0 and wrapper != shell, '/pid', 'invalid wrapper identity')
        info = row.log.stat()
        request.field('log_device').equals(info.st_dev)
        request.field('log_inode').equals(info.st_ino)
        grant = document((row.evidence/'wrapper-start.json').read_text())
        grant.field('schema').equals(SUITE_SCHEMA)
        grant.field('request').equals(request.value)
        grant.field('wrapper_wait_observed').equals(False)
        native_start = document((row.evidence/'native-start.json').read_text())
        native_end = document((row.evidence/'native-terminal.json').read_text())
        fields(native_start, 'schema request native_pid environment stream_start')
        fields(native_end, 'schema request native_pid native_exit timed_out settlement error stream_start stream_end stream_sha256 started_monotonic_ns ended_monotonic_ns')
        for node in (native_start, native_end):
            node.field('schema').equals(SUITE_SCHEMA)
            node.field('request').equals(request.value)
        pid = integer(native_start.field('native_pid'))
        require(pid > 0 and pid not in {shell, wrapper}, '/pid', 'invalid native identity')
        native_end.field('native_pid').equals(pid)
        native_start.field('environment').equals({k: v for k, v in suite_environment(plan).items()})
        native_end.field('native_exit').equals(0)
        native_end.field('timed_out').equals(False)
        native_end.field('settlement').equals('SETTLED')
        native_end.field('error').equals('')
        first = integer(native_start.field('stream_start'))
        last = integer(native_end.field('stream_end'))
        require(previous_end <= first <= last, '/stream', 'overlapping/reversed original ranges')
        native_end.field('stream_start').equals(first)
        native_end.field('stream_sha256').equals(range_hash(row.log, first, last))
        require(integer(native_end.field('ended_monotonic_ns')) > integer(native_end.field('started_monotonic_ns')),
                '/time', 'invalid native interval')
        previous_end = last
        checkpoint = document((row.evidence/'import-checkpoint.json').read_text())
        fields(checkpoint, 'target cache settings')
        expected_target: dict[str, Json] = {str(path.relative_to(plan.root)): digest for path, digest in plan.inputs
                           if path.is_relative_to(plan.root/'rust/target')}
        checkpoint.field('target').equals(expected_target)
    require(checkpoint is not None, '/checkpoint', 'missing terminal checkpoint')
    assert checkpoint is not None
    receipt.field('cache_terminal').equals(checkpoint.field('cache').value)
    receipt.field('settings_terminal').equals(checkpoint.field('settings').value)
    return checkpoint
