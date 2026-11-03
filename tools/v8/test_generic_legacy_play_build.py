# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 -m unittest test_generic_legacy_play_build
"""Actual harmless child waits; source/build bytes are explicit synthetic fixtures."""
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import gate_play_child
from generic_gate import GateContractError, document
from generic_process import file_hash
from mux2_gate_inputs import Json, Node

from generic_legacy_play_build import validate_play_build


class PlayBuildFixture:
    root: Path
    binary: Path
    receipt: Path
    commit: str
    def __init__(self, case: unittest.TestCase) -> None:
        temp=tempfile.TemporaryDirectory();case.addCleanup(temp.cleanup)
        self.root=Path(temp.name).resolve();(self.root/'rust').mkdir()
        tools=self.root/'bin';tools.mkdir();cargo=tools/'cargo'
        self.binary=self.root/'rust/target/debug/libsim_bridge.dylib'
        _=cargo.write_text('#!/bin/sh\nmkdir -p target/debug\nprintf fixture > target/debug/libsim_bridge.dylib\n')
        cargo.chmod(0o700)
        roles: dict[str,Json]={'build':{'argv':['cargo','build','-p','sim-bridge'],'cwd':str(self.root/'rust'),
              'executable':str(cargo),'executable_sha256':file_hash(cargo),'inputs':{},'artifacts':[str(self.binary)],
              'output':str(self.root/'child'),'timeout_seconds':None,'merged':True}}
        cases: dict[str,Json]={}
        for index,label in enumerate(gate_play_child.CASES):
            if index in (1,2):cases[label]=None;continue
            plan=self.root/f'case{index}.json';_=plan.write_text(json.dumps({'schema_version':1,'roles':roles}))
            cases[label]={'path':str(plan),'sha256':file_hash(plan)}
        suite=self.root/'suite.json';_=suite.write_text(json.dumps({'schema_version':1,'cases':cases}))
        selection=gate_play_child.select_case(suite,file_hash(suite),'--headless-check m1','build')
        before=Path.cwd()
        try:
            os.chdir(self.root/'rust')
            with patch.dict(os.environ,{'PATH':str(tools)+':/usr/bin:/bin'},clear=True):
                code=gate_play_child.run(selection.plan,selection.plan_sha256,'build',('cargo','build','-p','sim-bridge'),selection)
        finally:os.chdir(before)
        case.assertEqual(code,0)
        self.receipt=self.root/'child/receipt.json';child=document(self.receipt.read_text())
        self.commit='892edca1d31b26e535a3e9b62b37d62b99bb1130'
        self.join: dict[str,Json]={'schema':'play_build_join_v1','version':'m1',
            'source':{'source_sha':self.commit,'source_clean':True,'root':str(self.root)},
            'bridge':str(self.binary),'bridge_sha256':file_hash(self.binary),'build_receipt':str(self.receipt),
            'build_receipt_sha256':file_hash(self.receipt),'build_pid':child.field('pid').value,
            'authority':'JOINED_ORIGINAL_CHILD_RECEIPT_NOT_GENERIC_BUILD_RESULT','observer_world_qualification':'INCOMPLETE'}


class LegacyPlayBuildTests(unittest.TestCase):
    def test_real_original_child_receipt_join(self) -> None:
        fixture=PlayBuildFixture(self)
        self.assertEqual(validate_play_build(Node(fixture.join),fixture.commit,fixture.binary),file_hash(fixture.binary))

    def test_native_receipt_mutation_rejected(self) -> None:
        fixture=PlayBuildFixture(self);_=fixture.receipt.write_text('{}')
        with self.assertRaisesRegex(GateContractError,'child receipt changed'):
            _=validate_play_build(Node(fixture.join),fixture.commit,fixture.binary)

    def test_pid_and_source_join_mutations_rejected(self) -> None:
        fixture=PlayBuildFixture(self)
        for key,value in (('build_pid',1),('version','v7'),('authority','generic BuildResult')):
            row=dict(fixture.join);row[key]=value
            with self.assertRaises(GateContractError):_=validate_play_build(Node(row),fixture.commit,fixture.binary)

    def test_binary_mutation_rejected(self) -> None:
        fixture=PlayBuildFixture(self);_=fixture.binary.write_bytes(b'changed')
        with self.assertRaisesRegex(GateContractError,'bridge changed'):
            _=validate_play_build(Node(fixture.join),fixture.commit,fixture.binary)
