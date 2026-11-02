"""Warm antecedent preparation with actual tiny Python compiler receipts only."""
import json
import shutil
import subprocess
import unittest
from dataclasses import replace

import generic_build as build
import generic_setup_prepare as setup
import test_generic_build as build_fixtures
from generic_process import file_hash
from test_generic_setup_prepare import Fixture


class WarmTests(unittest.TestCase):
    def fixture(self) -> tuple[setup.SetupInputs, build.WorkspaceBuildPair]:
        producer = build_fixtures.WorkspaceBuildTests(); producer.setUp(); self.addCleanup(producer.doCleanups)
        producer.workspace_spec()
        home = producer.home/'setup'; home.mkdir(); template = Fixture(home)
        for name, _ in template.inputs.effective_inputs:
            destination = producer.root/name; destination.parent.mkdir(parents=True, exist_ok=True)
            _ = shutil.copy2(template.root/name, destination)
        config = producer.root/'rust/.cargo/config.toml'
        _ = config.write_text('[build]\njobs=8\n[net]\noffline=true\n')
        for name in ('cargo', 'rustc'):
            (template.bin/name).unlink(); (template.bin/name).symlink_to(producer.compiler)
        _ = subprocess.run(('git','-C',str(producer.root),'add','.'),check=True,capture_output=True)
        _ = subprocess.run(('git','-C',str(producer.root),'-c','user.name=Test','-c','user.email=t@example.invalid','commit','-qm','setup source'),check=True,capture_output=True)
        sha = subprocess.check_output(('git','-C',str(producer.root),'rev-parse','HEAD'),text=True).strip()
        names = subprocess.check_output(('git','-C',str(producer.root),'ls-files'),text=True).splitlines()
        effective = tuple(sorted((name,file_hash(producer.root/name)) for name in set(names)|{'rust/Cargo.lock'}))
        _ = template.origin.write_text(json.dumps({'path':str(producer.root/'rust/Cargo.lock'),'sha256':file_hash(producer.root/'rust/Cargo.lock')}))
        configs = ((str(config),file_hash(config)),)
        env = dict(template.inputs.environment)
        env.update(CARGO_HOME=str(producer.cargo_home),RUSTC_WRAPPER='',RUSTC_WORKSPACE_WRAPPER='')
        tools = tuple(replace(tool,path=producer.compiler,sha256=file_hash(producer.compiler)) if tool.name in ('cargo','rustc') else tool for tool in template.inputs.tools)
        inputs = replace(template.inputs,root=producer.root,source_sha=sha,cargo_home=producer.cargo_home,
            effective_inputs=effective,configs=configs,environment=tuple(sorted(env.items())),tools=tools,
            lock_origin_sha256=file_hash(template.origin))
        debug = replace(producer.spec,source_sha=sha,profile='debug',output=producer.home/'debug',config_inputs=configs)
        release = replace(debug,profile='release',output=producer.home/'release')
        pair = build.build_workspace_pair(debug,release,producer.home/'pair')
        self.assertEqual(pair.status,'BUILD_ANTECEDENTS_ONLY')
        return inputs,pair

    def test_actual_pair_prepares_only_and_recipe_rechecks(self) -> None:
        inputs,pair=self.fixture()
        prepared=setup.prepare_warm(inputs,pair)
        self.assertEqual(prepared.preparation_kind,'WARM_BUILD_ANTECEDENTS')
        self.assertFalse(prepared.environment_equal)
        self.assertFalse(prepared.execution_authority)
        self.assertEqual(setup.recipe(prepared,2).argv[2],'cd rust && cargo build -p sim-bridge')
        self.assertIn(('HOME',None,dict(inputs.environment)['HOME']),prepared.environment_delta)
        with self.assertRaises(ValueError): _=setup.prepare(inputs)
        (inputs.root/'rust/target/late-empty').mkdir()
        with self.assertRaises(ValueError): _=setup.recipe(prepared,2)

    def test_presence_sensitive_environment_and_identity_reject(self) -> None:
        inputs,pair=self.fixture()
        for defect in ('wrapper','jobs','locale','source','cargo_home','receipt_only'):
            with self.subTest(defect=defect):
                changed=inputs; selected=pair;env=dict(inputs.environment)
                if defect=='wrapper': del env['RUSTC_WORKSPACE_WRAPPER']
                if defect=='jobs': env['CARGO_BUILD_JOBS']='9'
                if defect=='locale': env['LANG']='en_US.UTF-8'
                if defect=='source': changed=replace(inputs,source_sha='a'*40)
                if defect=='cargo_home': changed=replace(inputs,cargo_home=inputs.root)
                if defect=='receipt_only': selected=replace(pair,retained_anchor=None)
                changed=replace(changed,environment=tuple(sorted(env.items())))
                with self.assertRaises(ValueError): _=setup.prepare_warm(changed,selected)

    def test_antecedent_evidence_mutation_and_output_overlap_reject(self) -> None:
        inputs,pair=self.fixture()
        with self.assertRaises(ValueError): _=setup.prepare_warm(replace(inputs,output=pair.receipt.parent/'new'),pair)
        prepared=setup.prepare_warm(inputs,pair)
        _=pair.receipt.write_text('{}')
        with self.assertRaises(ValueError): _=setup.recipe(prepared,2)
