# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: imported by source-owned play build-return and runtime child grants.
"""Prepare explicit historical observer exports after actual owned original builds."""
from dataclasses import dataclass
from pathlib import Path

from generic_gate import document, require
from generic_play_sources import SourceGrants
from generic_process import file_hash
from generic_sources import freeze_source
from mux2_gate_inputs import Json, Node

from generic_legacy_overlay import Request, materialize
from generic_legacy_play_build import validate_play_build


@dataclass(frozen=True, slots=True)
class PlayProjection:
    version: str
    original_root: Path
    export_root: Path
    commit: str
    original_build_join: Path
    original_build_join_sha256: str
    observer_sha256: str
    immutable_inputs: tuple[tuple[Path,str], ...]
    qualification: str = 'SOURCE_BOUND_OBSERVER_EXPORT_RUNTIME_UNQUALIFIED'

    def recheck(self) -> None:
        for path,digest in self.immutable_inputs:
            require(file_hash(path)==digest,str(path),'observer projection input changed')
        require(file_hash(self.original_build_join)==self.original_build_join_sha256,'/join','actual build join changed')
        expected={path for path,_ in self.immutable_inputs}
        actual={path for path in self.export_root.rglob('*') if path.is_file() and
                not path.is_relative_to(self.export_root/'.godot')}
        require(actual==expected,'/projection','foreign or missing export source member')

    def command(self, role: str, argv: tuple[str,...], cwd: Path) -> tuple[tuple[str,...],Path]:
        """Requested command remains recorded; only the explicit effective cwd is projected."""
        require(role in {'import-1','import-2','headless'},'/role','projection forbidden for version/build/control')
        require(cwd==self.original_root,'/cwd','foreign requested historical worktree')
        suffix=('--headless','--path','.', '--quit-after','300') if role=='headless' else (
            '--headless','--path','.', '--import','--editor','--quit')
        require(len(argv)==len(suffix)+1 and argv[1:]==suffix,'/argv','original historical workload changed')
        self.recheck()
        return argv,self.export_root

    def declaration(self) -> dict[str,Json]:
        self.recheck()
        return {'schema':'play_observer_projection.v1','version':self.version,'product_commit':self.commit,
                'requested_root':str(self.original_root),'effective_root':str(self.export_root),
                'original_build_join':str(self.original_build_join),'original_build_join_sha256':self.original_build_join_sha256,
                'observer_sha256':self.observer_sha256,'qualification':self.qualification,
                'immutable_inputs':{str(path):digest for path,digest in self.immutable_inputs}}


def prepare_legacy_projection(grants: SourceGrants, version: str, build_join: Path,
                              expected_join_sha256: str, destination: Path,
                              observer: Path, observer_sha256: str) -> PlayProjection:
    """Consumes live owner's actual source/build grant; never creates a BuildResult."""
    require(version in {'m1','v7'},'/version','main requires its separately qualified current observer projection')
    grants.check();grants.check_built(version)
    source=grants.source(version);built=grants.built[version]
    require(file_hash(build_join)==expected_join_sha256,'/join','foreign build-return join')
    joined=document(build_join.read_text())
    require(validate_play_build(joined,source.commit,built.path)==built.sha256,'/build','owner/child bridge mismatch')
    joined.field('build_receipt').equals(str(built.receipt));joined.field('build_receipt_sha256').equals(built.receipt_sha256)
    joined.field('build_pid').equals(built.pid)
    require(file_hash(observer)==observer_sha256,'/observer','observer differs from caller O binding')
    original=grants.play_root/version
    before=freeze_source(original,source.commit)
    require(before==grants.started[version],'/source','original source grant changed')
    exported=materialize(Request(grants.launcher,source.commit,destination,built.path,build_join,expected_join_sha256,observer))
    provenance=document((exported/'gate_observer/provenance.json').read_text())
    provenance.field('product_commit').equals(source.commit);provenance.field('observer_sha256').equals(observer_sha256)
    paths={exported/name for name in provenance.field('source_files_sha256').mapping()}
    paths|={exported/'gate_observer'/name for name in ('legacy.gd','build-receipt.json','provenance.json')}
    paths.add(exported/'rust/target/debug/libsim_bridge.dylib')
    result=PlayProjection(version,original,exported,source.commit,build_join,expected_join_sha256,observer_sha256,
                          tuple((path,file_hash(path)) for path in sorted(paths)))
    require(freeze_source(original,source.commit)==before,'/source','observer export changed original worktree')
    grants.check();result.recheck()
    return result


def read_projection(path: Path) -> PlayProjection:
    """Read a proposed owner projection; the live owner separately grants its exact declaration."""
    row=document(path.read_text())
    require(set(row.mapping())=={'schema','version','product_commit','requested_root','effective_root','original_build_join',
            'original_build_join_sha256','observer_sha256','qualification','immutable_inputs'},'/projection','foreign declaration fields')
    row.field('schema').equals('play_observer_projection.v1')
    row.field('qualification').equals('SOURCE_BOUND_OBSERVER_EXPORT_RUNTIME_UNQUALIFIED')
    result=PlayProjection(row.field('version').text(),Path(row.field('requested_root').text()),Path(row.field('effective_root').text()),
                          row.field('product_commit').text(),Path(row.field('original_build_join').text()),
                          row.field('original_build_join_sha256').text(),row.field('observer_sha256').text(),
                          tuple((Path(name),Node(value).text()) for name,value in row.field('immutable_inputs').mapping().items()))
    result.recheck()
    return result
