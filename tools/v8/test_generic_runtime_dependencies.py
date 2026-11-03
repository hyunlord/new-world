"""Dependency-record checks only; no native execution or admission authority."""
import json
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path

import generic_full_gate as gate
from generic_bench import AuditError, Product
from generic_gate import document
from generic_process import file_hash
from mux2_gate_inputs import Json


class RuntimeDependencyTests(unittest.TestCase):
    def products(self, root: Path) -> tuple[Product, Product]:
        rows: list[Product] = []
        for arm in ('base', 'candidate'):
            folder = root / arm
            folder.mkdir()
            source = folder / 'source'
            _ = source.write_bytes(b'source')
            rows.append(Product(folder, folder / 'binary', 'a' * 40, 'b' * 64,
                                folder / 'build.json', 'c' * 64, (('source', file_hash(source)),)))
        return rows[0], rows[1]

    def records(self, base: Product, candidate: Product) -> tuple[dict[str, Json], ...]:
        return (gate.perf_dependencies(gate.PerfContext(base, candidate, 100.0, 'self_proof')),
                gate.bench_dependencies(gate.BenchContext(base, candidate, 'self_proof',
                    candidate.root / 'script', 'd' * 64)))

    def test_empty_records_preserve_exact_original_schema_and_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base, candidate = self.products(Path(temporary).resolve())
            products: dict[str, Json] = {arm: {'root': str(p.root), 'source_sha': p.source_sha,
                'binary': str(p.binary), 'binary_sha256': p.binary_sha256,
                'build_receipt': str(p.build_receipt), 'build_receipt_sha256': p.build_receipt_sha256,
                'inputs': dict(p.input_files)} for arm, p in (('base', base), ('candidate', candidate))}
            expected: tuple[dict[str, Json], ...] = (
                {'deadline': 100.0, 'purpose': 'self_proof', 'embedded_preparation': 'NOT_QUALIFIED',
                 'compile_root_authority': 'CALLER_OWNED_NOT_CERTIFIED_HERE', 'products': products},
                {'purpose': 'self_proof', 'script': str(candidate.root / 'script'),
                 'script_sha256': 'd' * 64, 'justifications': [],
                 'build_authority': 'CALLER_OWNED_NOT_CERTIFIED_HERE', 'products': products})
            for actual, old in zip(self.records(base, candidate), expected, strict=True):
                self.assertEqual(json.dumps(actual), json.dumps(old))

    def test_nonempty_binding_and_late_drift_rejects_without_refresh(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base, candidate = self.products(Path(temporary).resolve())
            runtime = candidate.root / 'runtime'
            _ = runtime.write_bytes(b'runtime')
            candidate = replace(candidate, runtime_inputs=(('runtime', file_hash(runtime)),))
            for record in self.records(base, candidate):
                products = document(json.dumps(record)).field('products')
                products.field('candidate').field('runtime_inputs').equals(dict(candidate.runtime_inputs))
                products.field('candidate').field('inputs').equals(dict(candidate.input_files))
                self.assertNotIn('runtime_inputs', products.field('base').mapping())
            _ = runtime.write_bytes(b'changed')
            for call in (lambda: gate.perf_dependencies(gate.PerfContext(base, candidate, 100.0, 'self_proof')),
                         lambda: gate.bench_dependencies(gate.BenchContext(base, candidate, 'self_proof', runtime, 'd'*64))):
                with self.assertRaises((AuditError, ValueError)):
                    _ = call()

    def test_conflicting_runtime_binding_rejects(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base, candidate = self.products(Path(temporary).resolve())
            candidate = replace(candidate, runtime_inputs=(('source', 'f' * 64),))
            with self.assertRaises((AuditError, ValueError)):
                _ = self.records(base, candidate)
