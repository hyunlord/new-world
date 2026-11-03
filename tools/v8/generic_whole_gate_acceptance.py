# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_whole_gate_acceptance
"""Whole-shell acceptance slots, composed from the original validators."""
import argparse
import json
import stat
import traceback
from dataclasses import dataclass
from pathlib import Path
from typing import Final

import generic_bench_run as bench
import generic_core_evaluation as core
import generic_direct_run as direct
import generic_whole_gate_contract as shared
import read_summary_normalise as normalizer
from generic_declarations import Change, Declarations
from generic_fixture_registry import load_registry
from generic_fixture_run import Products
from generic_fixtures import ProbeProduct
from generic_gate import document, require
from generic_process import atomic_json, file_hash
from mux2_gate_inputs import Json, Node

from generic_bench import Product

STEPS: Final = {25:'bench',44:'core_shadow',45:'core_evaluation',47:'m6a_food_fixture',
               48:'m6a_food_data_only',50:'read_preservation'}


@dataclass(frozen=True, slots=True)
class Result:
    status: str
    component_status: str
    receipt: Path | None
    detail: str = ''


def _texts(node: Node) -> tuple[str, ...]:
    return tuple(row.text() for row in node.array())


def parse_declarations(node: Node) -> Declarations:
    changes = tuple(Change(row.field('fixture_id').text(),row.field('base').text(),
        row.field('behavior_commit').text(),row.field('reason').text(),
        row.field('declaration_path').text(),row.field('declaration_commit').text(),
        _texts(row.field('behavior_paths'))) for row in node.field('changes').array())
    files: list[tuple[str,str,str]] = []
    for row in node.field('files').array():
        values = _texts(row)
        require(len(values)==3, '/declarations/files', 'invalid file binding')
        files.append((values[0], values[1], values[2]))
    return Declarations(node.field('candidate').text(),node.field('base').text(),node.field('purpose').text(),
        _texts(node.field('known_fixtures')), _texts(node.field('ancestors')), changes,
        tuple(files), _texts(node.field('ignored_historical')), node.field('source_manifest_sha256').text())


def _probe(session: Node, arm: str, selected: Node) -> ProbeProduct:
    root = Path(session.field('arms').field(arm).field('root').text())
    inputs = shared.hashes(selected.field('source_inputs'))
    for name, digest in shared.hashes(selected.field('runtime_inputs')).items():
        path = Path(name)
        require(path.is_absolute() and path.is_relative_to(root), name, 'runtime outside arm')
        relative = str(path.relative_to(root))
        require(relative not in inputs or inputs[relative]==digest, name, 'conflicting runtime binding')
        inputs[relative] = digest
    return ProbeProduct(session.field('arms').field(arm).field('source_sha').text(),
        Path(selected.field('executable').text()), selected.field('executable_sha256').text(),
        root, tuple(sorted(inputs.items())))


def _perf(session: Node, arm: str, selected: Node) -> Product:
    from generic_whole_gate_bench_product import from_view
    return from_view(session, arm, selected)


def _tree(root: Path) -> dict[str, tuple[int, str | None]]:
    result: dict[str,tuple[int,str|None]] = {}
    for path in (root, *sorted(root.rglob('*'))):
        mode = path.lstat().st_mode
        require(stat.S_ISREG(mode) or stat.S_ISDIR(mode), str(path), 'special evidence path')
        result[str(path)] = (mode, file_hash(path) if stat.S_ISREG(mode) else None)
    return result


def _inputs(session: Node, selected: tuple[Node, Node]) -> dict[str,tuple[int,str]]:
    paths = shared.hashes(session.field('external_inputs'))
    for arm, product in zip(('base','candidate'), selected, strict=True):
        root = Path(session.field('arms').field(arm).field('root').text())
        paths.update({str(root/name):digest for name,digest in shared.hashes(product.field('source_inputs')).items()})
        paths.update(shared.hashes(product.field('runtime_inputs')))
        for key in ('executable','build_receipt'):
            paths[product.field(key).text()] = product.field(key+'_sha256').text()
    harness = session.field('harness')
    paths.update({str(Path(harness.field('root').text())/name):digest
                  for name,digest in shared.hashes(harness.field('inputs')).items()})
    result: dict[str,tuple[int,str]] = {}
    for name,digest in paths.items():
        path = Path(name); mode = path.lstat().st_mode
        require(stat.S_ISREG(mode) and file_hash(path)==digest, name, 'changed bound input')
        result[name] = (mode,digest)
    return result


def _check_inputs(original: dict[str,tuple[int,str]]) -> None:
    for name,(mode,digest) in original.items():
        path = Path(name)
        require(path.lstat().st_mode==mode and file_hash(path)==digest, name, 'original input changed')


def _bench(session: Node, selected: tuple[Node,Node], output: Path) -> Result:
    config = session.field('acceptance'); script = config.field('bench_script')
    path = Path(script.field('path').text()); digest = script.field('sha256').text()
    justifications = tuple(shared.hashes(row) for row in config.field('justifications').array())
    products = tuple(_perf(session,arm,node) for arm,node in zip(('base','candidate'),selected,strict=True))
    purpose = session.field('purpose').text()
    terminal: Path | None = output/'result.json'
    try:
        _ = bench.run_bench_owned(products[0],products[1],purpose,path,digest,output,justifications)
    except bench.BenchOwnershipUnknown as error:
        return Result('UNKNOWN','UNKNOWN',error.ownership_receipt,'native ownership unresolved; retain lane')
    except bench.BenchSettledFailure as error:
        terminal = error.failure_receipt
    before = _tree(output)
    checked = bench.authenticate_result(products[0],products[1],purpose,path,digest,output,
        receipt_sha256=file_hash(terminal) if terminal is not None else None,justifications=justifications)
    require(_tree(output)==before, '/bench', 'validator changed original artifacts')
    return Result('COMPONENT_PASS' if checked.status=='COMPONENT_PASS' else 'FAIL',
                  checked.status,checked.receipt,'; '.join(checked.errors))


def _dispatch(session: Node, selected: tuple[Node,Node], step: int, output: Path) -> Result:
    if step==25:
        return _bench(session,selected,output)
    registry_ref = session.field('registry'); registry_path = Path(registry_ref.field('path').text())
    require(file_hash(registry_path)==registry_ref.field('sha256').text(), '/registry', 'registry changed')
    registry = load_registry(registry_path)
    declarations = parse_declarations(session.field('declarations'))
    products = Products(*(_probe(session,arm,node) for arm,node in zip(('base','candidate'),selected,strict=True)))
    require(declarations.purpose==session.field('purpose').text() and
            declarations.base==products.base.source_sha and declarations.candidate==products.candidate.source_sha,
            '/declarations', 'pre-layer declaration sources or purpose differ')
    if step==45:
        anchor = shared.prior_acceptance(session,44)
        acceptance = Path(anchor.field('path').text())
        require(file_hash(acceptance)==anchor.field('sha256').text(), '/prior', 'original44 acceptance changed')
        prior_node = document(acceptance.read_text()); prior_node.field('step').equals(44)
        prior_node.field('status').equals('COMPONENT_PASS')
        receipt = acceptance.parent/'component'/'receipt.json'
        files = tuple(sorted((str(Path(name).relative_to(receipt.parent)),digest)
            for name,digest in shared.hashes(anchor.field('inventory')).items() if Path(name).is_relative_to(receipt.parent)))
        prior = core.PriorArtifacts(receipt,prior_node.field('component_receipt_sha256').text(),files)
        evaluated = core.evaluate(registry,products,declarations,prior,output)
        raw = document(evaluated.receipt.read_text())
        passed = evaluated.status=='PENDING' and raw.field('evaluated_scope').value=='PASS'
        return Result('COMPONENT_PASS' if passed else 'FAIL',evaluated.status,evaluated.receipt,
                      'local evaluation only; original PENDING preserved')
    component = direct.run_step(registry,products,declarations,step,output)
    return Result('COMPONENT_PASS' if component.status=='PASS' else 'FAIL',component.status,component.receipt)


def normalizer_path(session: Node) -> Path:
    """Bind the loaded module to admitted H or its exact H-role materialization."""
    relative = 'tools/v8/read_summary_normalise.py'
    reference = session.field('acceptance').field('normalizer')
    expected = reference.field('sha256').text()
    harness = session.field('harness')
    origin = Path(harness.field('root').text())/relative
    require(Path(reference.field('path').text()) == origin and
            shared.hashes(harness.field('inputs')).get(relative) == expected and
            file_hash(origin) == expected, '/normalizer', 'normalizer is not admitted H source')
    actual = Path(normalizer.__file__).resolve(strict=True)
    require(file_hash(actual) == expected, '/normalizer', 'loaded normalizer bytes differ')
    if actual == origin:
        return actual
    arms = [Node(value) for value in session.field('arms').mapping().values()
            if Path(Node(value).field('root').text())/relative == actual]
    require(len(arms) == 1, '/normalizer', 'loaded normalizer is outside admitted roots')
    arm = arms[0]
    require(shared.hashes(arm.field('source_inputs')).get(relative) == expected,
            '/normalizer', 'normalizer absent from effective inputs')
    effective = arm.field('effective_source').field('manifest').field(relative).array()
    require(len(effective) == 2 and effective[0].text() in {'100644','100755'} and
            effective[1].text() == expected, '/normalizer', 'effective normalizer differs')
    role_path = Path(arm.field('layer_manifest_path').text())
    digest = arm.field('layer_manifest_sha256').text()
    require(shared.hashes(session.field('external_inputs')).get(str(role_path)) == digest and
            file_hash(role_path) == digest, '/normalizer', 'role map is not session-bound')
    roles = document(role_path.read_text())
    roles.field('schema').equals('whole_gate_layers.v1')
    roles.field('product_sha').equals(arm.field('source_sha').text())
    roles.field('harness_sha').equals(harness.field('source_sha').text())
    selected = [row for row in roles.field('files').array() if row.field('path').text() == relative]
    if selected:
        require(len(selected) == 1 and selected[0].field('role').text() == 'H' and
                selected[0].field('post_sha256').text() == expected,
                '/normalizer', 'loaded normalizer is not an H-role postimage')
    else:
        # A omits unchanged paths from its exact overlay role map. Authenticate
        # the original P entry, not a fabricated no-op H overlay.
        original = arm.field('source_before')
        original.field('root').equals(arm.field('root').value)
        original.field('source_sha').equals(arm.field('source_sha').value)
        original.field('source_clean').equals(True)
        entry = original.field('tracked_manifest').field(relative).array()
        require(len(entry) == 3 and entry[0].text() == effective[0].text() and
                entry[2].text() == expected,
                '/normalizer', 'unchanged original normalizer differs from admitted H')
    return actual


def run_step(manifest: Path, digest: str, step: int, output: Path) -> Result:
    """Execute one closed paired component; global qualification belongs to the owner."""
    require(type(step) is int and step in STEPS, '/step', 'unsupported acceptance slot')
    session = shared.load_session(manifest,digest)
    expected = shared.stage_root(session,'candidate',step,STEPS[step])/'acceptance'
    require(output==expected and output.is_absolute() and not output.exists(), '/output', 'fresh fixed output required')
    config = session.field('acceptance')
    require(set(config.mapping())=={'normalizer','bench_script','justifications'}, '/acceptance', 'unknown configuration')
    _ = normalizer_path(session)
    for key in ('normalizer','bench_script'):
        ref = config.field(key); path = Path(ref.field('path').text())
        require(shared.hashes(session.field('external_inputs')).get(str(path))==ref.field('sha256').text(),
                '/acceptance/'+key, 'configuration input is not session-bound')
    kind = 'perf' if step==25 else 'probe'
    selected = (shared.current_product_inputs(session,'base',kind),shared.current_product_inputs(session,'candidate',kind))
    inputs = _inputs(session,selected)
    output.mkdir(parents=True)
    # Fresh mkdir(0777) and atomic_json's open(0666) share the unchanged umask.
    intended_mode = stat.S_IFREG | (stat.S_IMODE(output.lstat().st_mode) & 0o666)
    result = _dispatch(session,selected,step,output/'component')
    original = _tree(output)
    _ = shared.load_session(manifest,digest)
    require(_tree(output)==original, '/output', 'original producer output changed')
    _check_inputs(inputs)
    value: dict[str,Json] = {'schema':'whole_gate_acceptance.v1','step':step,'status':result.status,
        'component_status':result.component_status,'component_receipt':str(result.receipt) if result.receipt else None,
        'component_receipt_sha256':file_hash(result.receipt) if result.receipt else None,
        'detail':result.detail,'publication_authority':False,'session_sha256':digest,
        'original_artifacts':{name:sha for name,(_,sha) in original.items() if sha is not None},
        'original_modes':{name:mode for name,(mode,_) in original.items()},
        'base':session.field('arms').field('base').field('source_sha').text(),
        'candidate':session.field('arms').field('candidate').field('source_sha').text(),
        'harness':session.field('harness').field('source_sha').text()}
    receipt = output/'acceptance.json'
    intended = (json.dumps(value,sort_keys=True,indent=2)+'\n').encode()
    atomic_json(receipt,value)
    require(receipt.read_bytes()==intended, '/receipt', 'publication changed intended result')
    original[str(receipt)] = (intended_mode,file_hash(receipt))
    _ = shared.load_session(manifest,digest)
    _check_inputs(inputs)
    require(_tree(output)==original, '/output', 'publication changed original evidence')
    return result


class Arguments(argparse.Namespace):
    session: Path = Path()
    sha256: str = ''
    step: int = 0
    output: Path = Path()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    _ = parser.add_argument('operation',choices=('run',)); _ = parser.add_argument('session',type=Path)
    _ = parser.add_argument('sha256'); _ = parser.add_argument('step',type=int); _ = parser.add_argument('output',type=Path)
    args = parser.parse_args(namespace=Arguments())
    try:
        result = run_step(args.session,args.sha256,args.step,args.output)
    except BaseException:  # noqa: BLE001 - CLI boundary retains UNKNOWN for every unreturned producer.
        # An unreturned producer is never classified as a settled oracle failure.
        traceback.print_exc()
        return 2
    return {'COMPONENT_PASS':0,'FAIL':1,'UNKNOWN':2}[result.status]


if __name__=='__main__':
    raise SystemExit(main())
