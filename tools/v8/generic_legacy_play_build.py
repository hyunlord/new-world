# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_legacy_play_build
"""Authenticate a real source-owned historical play build join without conversion."""
import re
from pathlib import Path

from generic_gate import boundary, document, integer, require
from generic_process import file_hash
from mux2_gate_inputs import Node


@boundary
def validate_play_build(join: Node, commit: str, binary: Path) -> str:
    """Caller independently binds the join bytes; validate its retained actual child."""
    join.field('schema').equals('play_build_join_v1')
    version=join.field('version').text()
    expected={'m1':'892edca1d31b26e535a3e9b62b37d62b99bb1130','v7':'b1ca032f8fa84cee87d68a185e471b8e2c2bddce'}
    require(re.fullmatch('[0-9a-f]{40}',commit) is not None and
            ((version in expected and expected[version]==commit) or version=='main'),
            '/source','foreign historical play source')
    source=join.field('source');source.field('source_sha').equals(commit);source.field('source_clean').equals(True)
    root=Path(source.field('root').text())
    require(binary==root/'rust/target/debug/libsim_bridge.dylib','/binary','foreign original selected bridge')
    join.field('bridge').equals(str(binary));digest=join.field('bridge_sha256').text()
    require(file_hash(binary)==digest,'/binary','actual built bridge changed')
    join.field('authority').equals('JOINED_ORIGINAL_CHILD_RECEIPT_NOT_GENERIC_BUILD_RESULT')
    join.field('observer_world_qualification').equals('INCOMPLETE')
    receipt=Path(join.field('build_receipt').text())
    require(file_hash(receipt)==join.field('build_receipt_sha256').text(),'/build','original child receipt changed')
    child=document(receipt.read_text())
    child.field('role').equals('build');child.field('case').equals('--headless-check '+version)
    child.field('native_exit').equals(0);child.field('launcher_exit').equals(0)
    child.field('timed_out').equals(False);child.field('errors').equals([])
    pid=integer(child.field('pid'));require(pid>0,'/pid','missing actual native PID');join.field('build_pid').equals(pid)
    child.field('argv').equals(['cargo','build','-p','sim-bridge']);child.field('cwd').equals(str(root/'rust'))
    child.field('declared_artifacts').equals([str(binary)]);child.field('artifact_hashes').equals({str(binary):digest})
    child.field('after_inputs').equals(child.field('expected_inputs').value)
    require(file_hash(Path(child.field('executable').text()))==child.field('executable_sha256').text(),'/tool','actual compiler tool changed')
    start=document((receipt.parent/'started.json').read_text())
    for field in ('pid','wrapper_pid','parent_pid','argv','cwd','environment','plan_sha256','executable_sha256'):
        start.field(field).equals(child.field(field).value)
    child.field('parent_pid_after_wait').equals(child.field('parent_pid').value)
    for name in ('stdout','stderr'):
        require(file_hash(receipt.parent/(name+'.log'))==child.field(name+'_sha256').text(),'/log','actual child log changed')
    return digest
