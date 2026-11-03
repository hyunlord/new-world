"""Actual harmless process/view joins; no native or benchmark qualification."""
import json
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import generic_whole_gate_acceptance as acceptance
import generic_whole_gate_bench as collector
from generic_gate import document
from generic_process import (
    ProcessSpec,
    atomic_json,
    environment_for,
    execute,
    file_hash,
)
from generic_whole_gate_products import publish_products, selected
from mux2_gate_inputs import Node
from test_generic_bench import ProductFixture

import generic_bench as bench
from generic_whole_gate_bench_stage import PERF_COMMAND


class WholeStageProductTests(ProductFixture):
    def prepare_views(self):
        arms = {arm: {'root': str(p.root), 'source_sha': p.source_sha,
                      'source_inputs': dict(p.input_files)}
                for arm, p in zip(('base', 'candidate'), self.products, strict=True)}
        session = Node({'arms': arms, 'output': str(self.root/'whole')})
        products = []
        for arm, old in zip(('base', 'candidate'), self.products, strict=True):
            root = old.root
            (root/'rust').mkdir()
            scripts = root/'tools/v8'; scripts.mkdir(parents=True)
            (scripts/'perf.sh').write_text('#!/bin/sh\nexit 0\n')
            commands = self.root/(arm+'-commands'); commands.mkdir()
            cargo = commands/'cargo'
            cargo.write_text('#!/bin/sh\nmkdir -p target/release\nprintf "#!/bin/sh\\nexit 0\\n" > target/release/worldsim_perf\nchmod 755 target/release/worldsim_perf\n')
            cargo.chmod(0o755)
            output = self.root/'whole/steps'/arm/'12-perf'
            output.mkdir(parents=True)
            inputs = {**dict(old.input_files), 'tools/v8/perf.sh': file_hash(scripts/'perf.sh')}
            arms[arm]['source_inputs'] = inputs
            argv = ('bash', '-c', PERF_COMMAND, str(self.root/'whole'/arm))
            bash = Path('/bin/bash').resolve()
            spec = ProcessSpec((str(bash), '-c', 'exec "$@" 2>&1', 'whole-step', *argv), root,
                bash, file_hash(bash), root, tuple(sorted(inputs.items())), output/'process',
                (('PATH', str(commands)+':/usr/bin:/bin'), ('WORLDSIM_GIT_SHA', old.source_sha)))
            result = execute(spec)
            self.assertEqual(result.native_exit, 0)
            stage = {'schema':'whole_gate_step.v1','step':12,'name':'perf','arm':arm,
                     'state':'RETURNED','native_exit':0,'process_status':result.status,
                     'pid':result.pid,'cwd':str(root),'argv':list(argv),
                     'environment':environment_for(spec),'process_receipt':str(result.receipt),
                     'process_receipt_sha256':file_hash(result.receipt),
                     'selected_artifacts':selected(session,arm,12,0)}
            terminal = output/'stage.json'; atomic_json(terminal,stage)
            publish_products(session,arm,terminal)
            view = document((self.root/'whole/products'/arm/'perf.json').read_text())
            product = acceptance._perf(session,arm,view)
            with patch.object(collector,'current_product_inputs',return_value=view):
                self.assertEqual(product,collector._product(session,arm))
            bench.verify_product(product)
            self.assertIn(('rust/target/release/worldsim_perf',product.binary_sha256),bench.process_inputs(product))
            products.append(product)
        return session,tuple(products)

    def test_actual_a_view_enters_b_and_c_without_legacy_receipt(self):
        session, products = self.prepare_views()
        script = self.root/'commands.json'; script.write_text('[]')
        config = session.mapping()
        config['purpose'] = 'publication'
        config['acceptance'] = {'bench_script':{'path':str(script),'sha256':file_hash(script)},'justifications':[]}
        with patch.object(acceptance,'_perf',side_effect=products), \
             patch.object(acceptance.bench,'wait_for_load',return_value=(1.,2.,3.)):
            result = acceptance._bench(session,(Node({}),Node({})),self.root/'bench')
        self.assertEqual(result.status,'FAIL')
        self.assertEqual(len(list((self.root/'bench').glob('*.finished.json'))),6)
        # This positive admission reached real children; missing native measurements
        # remain a real safety FAIL, never a synthetic benchmark PASS.

    def test_changed_stage_contract_rejected(self):
        _, products = self.prepare_views()
        product = products[0]
        original = json.loads(product.build_receipt.read_text())
        for field,value in (('step',11),('name','other'),('native_exit',1),('state','UNKNOWN'),
                            ('argv',['bash','-c','true','/tmp'])):
            with self.subTest(field=field):
                changed = {**original,field:value}
                product.build_receipt.write_text(json.dumps(changed))
                with self.assertRaises(ValueError):
                    bench.verify_product(replace(product,build_receipt_sha256=file_hash(product.build_receipt)))
        product.build_receipt.write_text(json.dumps(original))
        process = Path(original['process_receipt'])
        process.chmod(0o600)
        raw = json.loads(process.read_text()); raw['errors']=['drift']
        process.write_text(json.dumps(raw))
        changed = {**original,'process_receipt_sha256':file_hash(process)}
        product.build_receipt.write_text(json.dumps(changed))
        with self.assertRaises(ValueError):
            bench.verify_product(replace(product,build_receipt_sha256=file_hash(product.build_receipt)))


if __name__ == '__main__':
    unittest.main()
