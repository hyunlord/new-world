#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# Run: python3 tools/art_pipeline/test_generate_agent_sections_p10.py
"""Discoverable offline preservation checks for diagnostic generation."""
from pathlib import Path
import unittest

from generate_agent_sections_p10 import DRAW, NAMES, PROCESS, MarkerError, generate


class AgentSectionGeneratorTests(unittest.TestCase):
    """Exercise current source and invalid variants without launching Godot."""

    def setUp(self) -> None:
        self.root = Path(__file__).resolve().parents[2]
        self.source = (self.root / 'scripts/ui/agent_renderer.gd').read_text()
        self.generated = generate(self.source)

    def test_preserves_original_statements_and_sprite_application(self) -> None:
        stripped = [line for line in self.generated.splitlines()
                    if line.strip() and '# P10_PROFILE' not in line]
        self.assertEqual(stripped, [line for line in self.source.splitlines() if line.strip()])
        self.assertIn('SpriteLibrary.apply(sprite, sprite_id)', self.generated)

    def test_accounts_for_all_spans_and_callback_exit_paths(self) -> None:
        self.assertEqual(self.generated.count('_p10_sections.begin()'), 1)
        self.assertEqual(self.generated.count('_p10_sections.finish(true)'), 1)
        self.assertEqual(self.generated.count('_p10_sections.finish(false,'), 2)
        for index, name in enumerate(NAMES):
            with self.subTest(section=name):
                self.assertIn(f'_p10_sections.mark({index})', self.generated)

    def test_every_missing_marker_is_rejected(self) -> None:
        for marker, _ in (*PROCESS, *DRAW):
            with self.subTest(marker=marker):
                lines = self.source.splitlines(keepends=True)
                index = next(i for i, line in enumerate(lines) if line.startswith(marker))
                lines[index] = '# REMOVED ' + lines[index]
                with self.assertRaises(MarkerError):
                    generate(''.join(lines))

    def test_duplicate_marker_is_rejected(self) -> None:
        mutated = self.source.replace(
            '\t_prepare_talk_cache(xs, ys)',
            '\t_prepare_talk_cache(xs, ys)\n\t_prepare_talk_cache(xs, ys)',
        )
        with self.assertRaises(MarkerError):
            generate(mutated)

    def test_new_early_return_is_rejected(self) -> None:
        mutated = self.source.replace('\tvar world_info:', '\treturn\n\tvar world_info:', 1)
        with self.assertRaises(MarkerError):
            generate(mutated)

    def test_reinstrumentation_is_rejected(self) -> None:
        with self.assertRaises(MarkerError):
            generate(self.generated)

    def test_clock_schema_covers_sections_and_excludes_child_callback(self) -> None:
        clock = (self.root / 'scripts/test/art/agent_section_clock_p10.gd').read_text()
        for name in NAMES:
            with self.subTest(section=name):
                self.assertIn(f'"{name}_usec"', clock)
        self.assertIn('unavailable_separate_callback', clock)


if __name__ == '__main__':
    unittest.main()
