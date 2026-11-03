"""Pure specification tests; no native processes or qualification evidence."""
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from generic_fixtures import ProbeProduct
from generic_gate import GateContractError
from generic_renderer_qualification_specs import independent_roots, renderer_spec


class SpecsTest(unittest.TestCase):
    def test_modes_preserve_original_arguments(self) -> None:
        with TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            commands = root/'commands.json'
            _ = commands.write_text('[]')
            product = ProbeProduct('a'*40, root/'probe', 'b'*64, root, ())
            for kind in ('generator', 'ascii', 'godot'):
                off = renderer_spec(product, kind, root/'out', commands, 'derived_off')
                original = renderer_spec(product, kind, root/'out', commands, 'original_off')
                self.assertEqual(off, original)
                self.assertFalse(any(key.startswith('WORLDSIM_GATE_') for key, _ in off.environment))
                self.assertNotIn('--gate-observations', off.argv)
                on = renderer_spec(product, kind, root/'out', commands, 'derived_on')
                self.assertEqual(on.inputs, off.inputs)
                self.assertEqual(on.external_inputs, off.external_inputs)
                if kind == 'godot':
                    self.assertEqual(on.argv, off.argv)
                    self.assertEqual(dict(on.environment)['WORLDSIM_GATE_OBSERVER'], '1')
                else:
                    self.assertEqual(on.argv[:-1], off.argv)
                    self.assertEqual(on.argv[-1], '--gate-observations')

    def test_unknown_mode_and_producer_reject(self) -> None:
        product = ProbeProduct('a'*40, Path('/probe'), 'b'*64, Path('/root'), ())
        for mode, kind in (('wrong', 'ascii'), ('derived_off', 'wrong')):
            with self.assertRaises(GateContractError):
                _ = renderer_spec(product, kind, Path('/out'), Path('/commands'), mode)

    def test_three_independent_canonical_roots(self) -> None:
        with TemporaryDirectory() as directory:
            base = Path(directory).resolve()
            roots = tuple(base/name for name in ('original', 'off', 'on'))
            for root in roots:
                root.mkdir()
            independent_roots(roots)
            for bad in (roots[:2], (roots[0], roots[0], roots[2]),
                        (base, roots[1], roots[2])):
                with self.assertRaises(GateContractError):
                    independent_roots(bad)
            (base/'alias').symlink_to(roots[0], target_is_directory=True)
            with self.assertRaises(GateContractError):
                independent_roots((base/'alias', roots[1], roots[2]))


if __name__ == '__main__':
    _ = unittest.main()
