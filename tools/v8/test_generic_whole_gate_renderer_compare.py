"""Offline ordinary renderer artifacts; never producer admission."""
import unittest
from dataclasses import replace
from pathlib import Path

from generic_whole_gate_renderer_compare import RendererArtifacts, compare_renderer

FIXTURES = Path(__file__).resolve().parent / 'fixtures/renderer_qualification'


class RendererTests(unittest.TestCase):
    def original(self) -> RendererArtifacts:
        return RendererArtifacts(*( (FIXTURES / name).read_bytes() for name in
            ('text_people.json', 'godot_people.json', 'colony_s100_commands.json')))

    def test_retained_identical(self) -> None:
        value = self.original()
        compare_renderer(value, value, value.commands)

    def test_changed_artifacts(self) -> None:
        value = self.original()
        for field, content in (('text', value.text + b' '), ('godot', value.godot + b' '),
                               ('commands', value.commands + b' ')):
            with self.subTest(field=field), self.assertRaises(ValueError):
                compare_renderer(value, replace(value, **{field: content}), value.commands)

    def test_wrong_common_state(self) -> None:
        value = self.original()
        for old, new in ((b'"seed": 100', b'"seed": 101'),
                         (b'"tick": 900', b'"tick": 901'),
                         (b'"count": 10', b'"count": true')):
            bad = replace(value, text=value.text.replace(old, new))
            with self.subTest(new=new), self.assertRaises(ValueError):
                compare_renderer(bad, bad, value.commands)

    def test_common_unsettled(self) -> None:
        value = self.original()
        bad = replace(value, godot=value.godot.replace(b'"settled": true', b'"settled": false'))
        with self.assertRaises(ValueError):
            compare_renderer(bad, bad, value.commands)

    def test_unknown_field_and_boolean_change(self) -> None:
        value = self.original()
        for changed in (value.text.replace(b'"count": 10', b'"unknown": 1, "count": 10'),
                        value.text.replace(b'"agent_id": 0', b'"agent_id": false', 1),
                        b''):
            with self.subTest(changed=changed[:30]), self.assertRaises(ValueError):
                compare_renderer(value, replace(value, text=changed), value.commands)

    def test_common_tiles_disagree(self) -> None:
        value = self.original()
        bad = replace(value, text=value.text.replace(b'78,', b'77,'))
        with self.assertRaises(ValueError):
            compare_renderer(bad, bad, value.commands)

    def test_common_invalid_replay_ticks(self) -> None:
        value = self.original()
        for commands in (value.commands.replace(b'"tick": 360', b'"tick": 900'),
                         value.commands.replace(b'"tick": 360', b'"tick": 500', 1)
                         .replace(b'"tick": 360', b'"tick": 400', 1)):
            bad = replace(value, commands=commands)
            with self.subTest(commands=commands[:80]), self.assertRaises(ValueError):
                compare_renderer(bad, bad, commands)

    def test_wrong_expected_commands(self) -> None:
        value = self.original()
        with self.assertRaises(ValueError):
            compare_renderer(value, value, b'[]')

    def test_common_appearance_disagreement(self) -> None:
        value = self.original()
        bad = replace(value, text=value.text.replace(b'"look": 27', b'"look": 99', 1))
        with self.assertRaises(ValueError):
            compare_renderer(bad, bad, value.commands)


if __name__ == '__main__':
    _ = unittest.main()
