"""Mode-aware original renderer workload; specifications grant no producer authority."""
from pathlib import Path

from generic_fixtures import ProbeProduct
from generic_gate import GateContractError, require
from generic_process import ProcessSpec, file_hash, real_path
from generic_renderer_run import PRODUCER, SCRIPT
from generic_whole_gate_qualification import MODES


def independent_roots(roots: tuple[Path, ...]) -> None:
    """All three qualification modes require distinct, nonnested physical roots."""
    require(len(roots) == 3 and len(set(roots)) == 3, '/roots', 'three independent mode roots required')
    for root in roots:
        real_path(root, directory=True)
        require(not any(root.is_relative_to(other) for other in roots if root != other),
                '/roots', 'nested mode roots')


def renderer_spec(product: ProbeProduct, kind: str, output: Path,
                  commands: Path, mode: str) -> ProcessSpec:
    """Only reviewed observer switches differ; replay input remains byte-bound."""
    require(mode in MODES, '/mode', 'unknown qualification mode')
    require(kind in {'generator', 'ascii', 'godot'}, '/producer', 'unknown renderer producer')
    enabled = mode == 'derived_on'
    argv = (str(product.executable), '--scene', 'colony', '--seeds', '100..101')
    environment: tuple[tuple[str, str], ...] = ()
    external: tuple[tuple[Path, str], ...] = ()
    fresh: tuple[str, ...] = ()
    match kind:
        case 'generator':
            argv += ('--days', '1', '--workers', '1', '--commands', 'tools/v8/u_script.json',
                     '--commands-out', str(output/'commands'), '--out', str(output))
        case 'ascii':
            argv += ('--ascii', '--tick', '900', '--commands', 'tools/v8/u_script.json',
                     '--out', str(output/'ascii_t900.txt'), '--people-json', str(output/'text_people.json'))
        case 'godot':
            argv = (str(product.executable), '--headless', '--path', str(product.input_root),
                    '--script', SCRIPT, '--', '--out', str(output/'godot_people.json'),
                    '--commands', str(commands), '--seed', '100', '--tick', '900')
            external = ((commands, file_hash(commands)),)
            fresh = ('settings',)
            environment = (('HOME', str(Path.home())),
                           ('WORLDSIM_TEST_SETTINGS_DIR', str(output/'settings')))
        case unknown:
            raise GateContractError('UNKNOWN', '/producer', 'unreviewed producer '+unknown)
    if enabled:
        if kind == 'godot':
            environment += (('WORLDSIM_GATE_OBSERVER', '1'),
                            ('WORLDSIM_GATE_INVOCATION_ID', PRODUCER+':'+str(output)),
                            ('WORLDSIM_GATE_UI_OUT', str(output)),
                            ('WORLDSIM_GIT_SHA', product.source_sha))
        else:
            argv += ('--gate-observations',)
    return ProcessSpec(argv, product.input_root, product.executable, product.executable_sha256,
                       product.input_root, product.input_files, output, environment, external, fresh, 600)
