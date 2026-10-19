"""Keep write-skip diagnostic checkpoints conditional and source preserving."""
from pathlib import Path
import unittest

import generate_agent_sections_p10 as p10
import generate_agent_sections_p11 as p11
import generate_agent_sections_p12 as p12
import generate_agent_sections_p13 as p13

GENERATORS = ((p10, '10'), (p11, '11'), (p12, '12'), (p13, '13'))


class WriteSkipSectionGeneratorTests(unittest.TestCase):
    def setUp(self) -> None:
        self.source = (Path(__file__).resolve().parents[2] / 'scripts/ui/agent_renderer.gd').read_text()

    def test_write_miss_sections_are_inside_guard_and_reset_after_it(self) -> None:
        for module, version in GENERATORS:
            with self.subTest(version=version):
                generated = module.generate(self.source)
                begin = generated.index('\t\tif not skip_writes_p13:')
                end = generated.index('\t\tif _activity_oracle_enabled:', begin)
                branch = generated[begin:end]
                for name, anchor in (('variant', 'SpriteLibrary.apply_variant('),
                                     ('meta', 'sprite.flip_h ='), ('children', 'CarryLoad.apply(')):
                    index = module.NAMES.index(name)
                    marker = f'\t\t\t_p{version}_sections.mark({index}) # P{version}_PROFILE\n\t\t\t{anchor}'
                    self.assertIn(marker, branch)
                other = module.NAMES.index('other')
                self.assertTrue(branch.endswith(f'\t\t_p{version}_sections.mark({other}) # P{version}_PROFILE\n'))
                self.assertIn(f'\t\t_p{version}_sections.mark({other}) # P{version}_PROFILE\n\t\tvar write_key_p13:', generated)
                recovered = [line for line in generated.splitlines() if line.strip() and f'# P{version}_PROFILE' not in line]
                self.assertEqual(recovered, [line for line in self.source.splitlines() if line.strip()])

    def test_guarded_markers_fail_closed(self) -> None:
        for module, _ in GENERATORS:
            for marker, _ in module.process_markers(self.source):
                lines = self.source.splitlines(keepends=True)
                index = next(i for i, line in enumerate(lines) if line.startswith(marker))
                lines[index] = '# REMOVED ' + lines[index]
                with self.subTest(module=module.__name__, marker=marker):
                    with self.assertRaises(module.MarkerError):
                        module.generate(''.join(lines))

    def test_unguarded_source_keeps_original_anchor_depth(self) -> None:
        # Reconstruct the pre-guard process shape; other candidate helpers are irrelevant.
        start = self.source.index('\t\tvar write_key_p13:')
        guard = self.source.index('\t\tif not skip_writes_p13:', start)
        end = self.source.index('\t\tif _activity_oracle_enabled:', guard)
        guarded = self.source[guard + len('\t\tif not skip_writes_p13:\n'):end]
        cache = guarded.index('\t\t\tif _write_skip_enabled_p13:')
        ordinary = ''.join(line[1:] if line.startswith('\t') else line for line in guarded[:cache].splitlines(keepends=True))
        original = self.source[:start] + ordinary + self.source[end:]
        for module, version in GENERATORS:
            markers = module.process_markers(original)
            self.assertIn(('\t\tSpriteLibrary.apply_variant(', 'variant'), markers)
            generated = module.generate(original)
            recovered = [line for line in generated.splitlines() if line.strip() and f'# P{version}_PROFILE' not in line]
            self.assertEqual(recovered, [line for line in original.splitlines() if line.strip()])


if __name__ == '__main__':
    unittest.main()
