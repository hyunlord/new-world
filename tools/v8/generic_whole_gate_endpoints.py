"""Bind an emitted native endpoint folder to original aggregate output anchors."""
import re
from pathlib import Path
from generic_gate import boundary, integer, require, text
from generic_process import real_path
from generic_test_endpoint import EndpointEvidence
from generic_test_history_graph import HistoryEvidence
from mux2_gate_inputs import Node

def _files(node: Node) -> dict[str,str]:
    return {name:text(Node(value)) for name,value in node.mapping().items()}

@boundary
def bind_observer(stage: Node, root: Path, inputs: dict[str,str], target: str, tests: tuple[str,...]) -> tuple[EndpointEvidence, HistoryEvidence, Path]:
    receipt_path=Path(text(stage.field('process_receipt')))
    environment = _files(stage.field('environment'))
    require(environment.get('WORLDSIM_GATE_OBSERVER') == '1', '/environment', 'native observer not enabled')
    observer = Path(environment['WORLDSIM_GATE_TEST_OUT'])
    require(observer.is_relative_to(receipt_path.parent.parent), '/observer', 'foreign observation directory')
    real_path(observer, directory=True)
    anchors = _files(stage.field('observer_artifacts'))
    modes = {name: integer(Node(value)) for name, value in stage.field('observer_directories').mapping().items()}
    require(str(observer) in modes and observer.stat().st_mode == modes[str(observer)], '/observer', 'unbound observer directory')
    folders: list[Path] = []
    for folder in observer.glob('save_load_pid*'):
        real_path(folder, directory=True)
        if any(any(p.name.startswith(test + '_s') for test in tests) for p in folder.iterdir()):
            folders.append(folder)
    require(len(folders) == 1, target, 'missing or duplicated fixed family emitter')
    folder = folders[0]
    match = re.fullmatch(r'save_load_pid([1-9][0-9]*)', folder.name)
    require(match is not None, '/pid', 'invalid native emitter PID')
    assert match is not None
    emitted_pid = int(match.group(1))
    files = tuple((str(Path(name).relative_to(observer)), digest) for name, digest in anchors.items()
                  if Path(name).is_relative_to(folder))
    actual_paths = {str(p) for p in folder.rglob('*') if p.is_file()}
    require(actual_paths == {str(observer / name) for name, _ in files}, target, 'unbound or missing native endpoint bytes')
    for directory in (folder, *(p for p in folder.rglob('*') if p.is_dir())):
        real_path(directory, directory=True)
        require(modes.get(str(directory)) == directory.stat().st_mode, str(directory), 'original directory mode changed')
    evidence = EndpointEvidence(observer, emitted_pid, files)
    graph = HistoryEvidence(observer, emitted_pid, files, root, tuple(sorted(inputs.items())))
    return evidence, graph, folder
