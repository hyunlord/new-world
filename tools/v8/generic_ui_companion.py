"""Admit exact native smoke companions separately from original UI oracle results."""
import hashlib

from generic_gate import boundary, document, integer, require, text
from generic_observations import Observation, keys
from generic_process import environment_for
from generic_ui_history import UIEvidence
from generic_ui_observations import UIInstance
from generic_ui_receipts import validate_ui
from mux2_gate_inputs import Node


def native_commands(node: Node, horizon: int, bootstrap: bool) -> tuple[tuple[int, str, int], ...]:
    """Preserve factory SetSteward and native Steward wrapper records exactly."""
    keys(node, {"total", "ticks", "kinds", "accepted"})
    ticks = tuple(integer(value) for value in node.field("ticks").array())
    kinds = tuple(text(value) for value in node.field("kinds").array())
    accepted = tuple(integer(value, 1) for value in node.field("accepted").array())
    total = integer(node.field("total"))
    require(total >= 1 and len(ticks) == len(kinds) == len(accepted) == total,
            "/commands", "missing or truncated intrinsic native command log")
    rows = tuple(zip(ticks, kinds, accepted))
    require(rows[0] == (0, "SetSteward", 1) and (not bootstrap or total == 1),
            "/commands", "factory bootstrap command differs")
    require(all(kind == "Steward" for _, kind, _ in rows[1:]) and
            all(0 <= tick <= horizon for tick in ticks) and tuple(sorted(ticks)) == ticks,
            "/commands", "unexpected UI command or invalid native command chronology")
    return rows


@boundary
def validate_smoke_companion(actual: UIEvidence, companion: UIEvidence) -> Observation:
    """Require trusted runner-created fresh output roots, never existing user settings.

    Process executor exclusively creates each output_root before launch. Binding
    settings beneath that fresh root proves absence without a caller boolean.
    Successful validation is native equivalence only, not a UI oracle PASS.
    """
    require(actual.plan.producer in {'smoke.boot', 'smoke.labels'}, '/companion', 'unsupported original producer')
    script = 'res://scripts/core/main_bootstrap.gd' if actual.plan.producer == 'smoke.boot' else 'res://tools/v8/smoke.gd'
    require(actual.plan.script == script, '/companion', 'foreign smoke script')
    require(companion.plan.producer == 'smoke.companion' and
            companion.plan.script == 'res://tools/v8/gate_ui_companion.gd', '/companion', 'foreign companion')
    require(actual.plan.process.output_root != companion.plan.process.output_root and
            actual.plan.invocation_id != companion.plan.invocation_id, '/companion', 'reused process identity')
    require(actual.plan.completion == ('native_tree_exit' if actual.plan.producer == 'smoke.boot' else 'producer_finish'),
            '/companion', 'foreign original completion seam')
    for evidence in (actual, companion):
        argv = evidence.plan.process.argv
        if evidence.plan.producer == 'smoke.boot':
            require(environment_for(evidence.plan.process).get('WORLDSIM_GATE_UI_BOOT') == '1',
                    '/companion', 'original boot observer was not enabled')
            require('--script' not in argv and '-s' not in argv, '/companion', 'boot must use original main scene')
        else:
            positions = [i for i, arg in enumerate(argv) if arg == '--script']
            require(len(positions) == 1 and positions[0]+1 < len(argv) and argv[positions[0]+1] == evidence.plan.script,
                    '/companion', 'actual script argument differs')
    states: list[UIInstance] = []
    terminal_logs: list[tuple[tuple[int, str, int], ...]] = []
    for evidence in (actual, companion):
        spec = evidence.plan.process
        environment = environment_for(spec)
        require('settings' in spec.fresh_directories, '/companion', 'settings directory was not runner-created')
        require(environment.get('WORLDSIM_TEST_SETTINGS_DIR') == str(spec.output_root / 'settings'),
                '/companion', 'settings must belong to trusted fresh process output root')
        result = validate_ui(evidence.sidecar, evidence.sidecar_sha256, evidence.plan,
                             evidence.process, evidence.process_sha256)
        require(len(result.instances) == 1 and len(result.instances[0]) == 1 and not result.loaded_unqualified,
                '/companion', 'smoke must contain exactly one native bootstrap without reset/load')
        instance = result.instances[0][0]
        require(instance.reason == 'bootstrap' and instance.state.initial_tick == 0 and not instance.closed,
                '/companion', 'foreign native lifecycle')
        raw = evidence.sidecar.read_bytes()
        require(hashlib.sha256(raw).hexdigest() == evidence.sidecar_sha256, '/companion', 'sidecar changed')
        previous: tuple[tuple[int, str, int], ...] = ()
        for index, checkpoint in enumerate(document(raw.decode()).field('checkpoints').array()):
            current = native_commands(checkpoint.field('commands'), integer(checkpoint.field('clock').field('tick')), index == 0)
            require(current[:len(previous)] == previous, '/commands', 'native command log prefix changed')
            previous = current
        terminal_logs.append(previous)
        states.append(instance)
    require(terminal_logs[0] == terminal_logs[1], "/commands", "original and companion native command logs differ")
    left, right = states
    require(actual.plan.process.executable_sha256 == companion.plan.process.executable_sha256 and
            dict(actual.plan.process.inputs) == dict(companion.plan.process.inputs) and
            actual.plan.process.external_inputs == companion.plan.process.external_inputs,
            '/companion', 'native/source/input bytes differ')
    excluded = {'WORLDSIM_GATE_INVOCATION_ID', 'WORLDSIM_GATE_UI_OUT', 'WORLDSIM_GATE_UI_BOOT', 'WORLDSIM_TEST_SETTINGS_DIR'}
    environments = [{k:v for k,v in environment_for(e.plan.process).items() if k not in excluded}
                    for e in (actual,companion)]
    require(environments[0] == environments[1], '/companion', 'effective native environments differ')
    require((left.seed, left.initial_agent_ids, left.terminal_agent_ids, left.state) ==
            (right.seed, right.initial_agent_ids, right.terminal_agent_ids, right.state),
            '/companion', 'companion native state/history differs at actual horizon')
    require(0 <= left.state.terminal_tick <= 200000, '/companion', 'unsupported native horizon')
    argv = companion.plan.process.argv
    separator = argv.index('--') if '--' in argv else -1
    require(separator >= 0 and argv[separator+1:] == ('--ticks', str(left.state.terminal_tick), '--original-producer',
            actual.plan.producer, '--original-sidecar-sha256', actual.sidecar_sha256), '/companion', 'unbound companion request')
    require(tuple(cp.label for cp in companion.plan.checkpoints) == ('bootstrap','companion_terminal'),
            '/companion', 'unexpected companion workload')
    require(document(companion.plan.checkpoints[1].context_json).value == {'requested_tick':left.state.terminal_tick,
            'original_producer':actual.plan.producer,'original_sidecar_sha256':actual.sidecar_sha256},
            '/companion', 'unbound terminal context')
    return right.state
