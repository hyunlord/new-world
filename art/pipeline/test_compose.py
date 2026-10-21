import unittest
from dataclasses import replace

import numpy as np
from PIL import Image

from . import compose


class CompositionTests(unittest.TestCase):
    def test_age_boundaries_and_households(self) -> None:
        person = compose.Person(compose.AgentId(7), 'female', 14, compose.HouseholdId(8))
        self.assertEqual(compose.resolve(person).body, 'child')
        self.assertEqual(compose.resolve(replace(person, age=15)).body, 'adult_f')
        self.assertEqual(compose.resolve(replace(person, age=50)).body, 'elder')
        self.assertEqual(compose.resolve(person).tunic, compose.TUNICS[2])

    def test_hash_has_stable_golden_and_independent_domains(self) -> None:
        self.assertEqual(compose.keyed_index(compose.AgentId(7), 'hair_style', 5), 0)
        self.assertEqual(compose.keyed_index(compose.AgentId(7), 'hair_color', 4), 2)
        self.assertEqual(compose.resolve(compose.Person(compose.AgentId(7), 'male', 20, compose.HouseholdId(0))),
                         compose.resolve(compose.Person(compose.AgentId(7), 'male', 20, compose.HouseholdId(0))))

    def test_tint_preserves_material_and_alpha(self) -> None:
        image = Image.new('RGBA', (3, 1))
        image.putdata([(192, 192, 192, 255), (84, 70, 52, 255), (192, 192, 192, 0)])
        result = compose.tint(image, compose.resolve(compose.Person(compose.AgentId(1), 'male', 20, compose.HouseholdId(2))))
        self.assertEqual(result.getpixel((0, 0)), (*compose.TUNICS[2], 255))
        self.assertEqual(result.getpixel((1, 0)), (84, 70, 52, 255))
        self.assertEqual(result.getpixel((2, 0)), (0, 0, 0, 0))

    def test_all_channels_and_shadow_preserve_geometry(self) -> None:
        person = compose.Person(compose.AgentId(7), 'male', 20, compose.HouseholdId(1))
        appearance = compose.resolve(person)
        image = Image.new('RGBA', (4, 1))
        image.putdata([(154, 154, 154, 255), (110, 110, 110, 255),
                       (192, 192, 192, 255), (176, 176, 176, 255)])
        before = image.tobytes()
        result = compose.tint(image, appearance)
        expected = (appearance.skin_color, appearance.tunic, appearance.tunic, appearance.hair_color)
        for index, rgb in enumerate(expected):
            self.assertEqual(result.getpixel((index, 0)), (*rgb, 255))
        self.assertEqual(image.tobytes(), before)
        layers = {key: Image.new('RGBA', (32, 32)) for key in compose.layer_keys(appearance)}
        keys = compose.layer_keys(appearance)
        layers[keys[0]].putpixel((20, 20), (20, 18, 16, 90))
        layers[keys[1]].paste((154, 154, 154, 255), (5, 5, 10, 10))
        combined = compose.compose_person(person, compose.LayerAtlas(layers))
        self.assertEqual(combined.getpixel((20, 20)), (20, 18, 16, 90))
        self.assertEqual(combined.getpixel((5, 5)), (20, 18, 16, 255))
        for frame in ('idle', 'walk_a', 'walk_b', 'work', 'sleep'):
            resolved = compose.resolve(replace(person, state=frame))
            self.assertTrue(all(key.endswith('/' + frame) for key in compose.layer_keys(resolved)))

    def test_missing_frame_fails(self) -> None:
        with self.assertRaises(compose.MissingLayerError):
            _ = compose.compose_person(compose.Person(compose.AgentId(1), 'male', 20, compose.HouseholdId(0)), compose.LayerAtlas({}))

    def test_material_grays_are_not_tinted_by_another_layer_family(self) -> None:
        person = compose.Person(compose.AgentId(7), 'male', 20, compose.HouseholdId(1),
                                job='builder', carry='stone', beard='short')
        appearance = compose.resolve(person)
        keys = compose.layer_keys(appearance)
        grays = (154, 110, 192, 176)
        unchanged = tuple((gray, gray, gray) for gray in grays)
        expected = {
            'tool': unchanged, 'carry': unchanged,
            'top': (unchanged[0], unchanged[1], appearance.tunic, unchanged[3]),
            'hair': (unchanged[0], unchanged[1], unchanged[2], appearance.hair_color),
            'beard': (unchanged[0], unchanged[1], unchanged[2], appearance.hair_color),
        }
        for family, colors in expected.items():
            with self.subTest(family=family):
                layers = {key: Image.new('RGBA', (32, 32)) for key in keys}
                key = next(key for key in keys if key.startswith(family + '/'))
                for index, gray in enumerate(grays):
                    x = 3 + index * 7
                    layers[key].paste((gray, gray, gray, 255), (x, 10, x + 5, 15))
                image = compose.compose_person(person, compose.LayerAtlas(layers))
                for index, color in enumerate(colors):
                    self.assertEqual(image.getpixel((5 + index * 7, 12)), (*color, 255))

    def test_closed_layer_hole_retains_underlying_skin_tint(self) -> None:
        person = compose.Person(compose.AgentId(7), 'male', 20, compose.HouseholdId(1))
        appearance = compose.resolve(person)
        keys = compose.layer_keys(appearance)
        layers = {key: Image.new('RGBA', (32, 32)) for key in keys}
        layers[keys[1]].paste((154, 154, 154, 255), (5, 5, 12, 12))
        layers[keys[2]].paste((192, 192, 192, 255), (5, 5, 12, 12))
        layers[keys[2]].putpixel((8, 8), (154, 154, 154, 255))
        result = compose.compose_person(person, compose.LayerAtlas(layers))
        self.assertEqual(result.getpixel((8, 8)), (*appearance.skin_color, 255))
        self.assertEqual(result.getpixel((9, 8)), (*appearance.tunic, 255))

    def test_order_flip_sleep_and_shape_rejection(self) -> None:
        person = compose.Person(compose.AgentId(3), 'male', 30, compose.HouseholdId(0), job='woodcutter', carry='logs', beard='short', head='hood')
        layers: dict[str, Image.Image] = {}
        for direction in ('south', 'east'):
            for key in compose.layer_keys(compose.resolve(replace(person, direction=direction))):
                image = Image.new('RGBA', (32, 32))
                image.paste((154, 154, 154, 255), (5, 5, 16, 16))
                layers[key] = image
        resolved = compose.resolve(person)
        keys = compose.layer_keys(resolved)
        layers[keys[-1]].paste((84, 70, 52, 255), (5, 5, 16, 16))
        atlas = compose.LayerAtlas(layers)
        south = compose.compose_person(person, atlas)
        self.assertEqual(south.getpixel((9, 9)), (84, 70, 52, 255))
        east = compose.compose_person(replace(person, direction='east'), atlas)
        west = compose.compose_person(replace(person, direction='west'), atlas)
        self.assertEqual(west.tobytes(), east.transpose(Image.Transpose.FLIP_LEFT_RIGHT).tobytes())
        sleeper = replace(person, state='sleep', direction='west')
        sleeping = {key: Image.new('RGBA', (32, 16), (154, 154, 154, 255)) for key in compose.layer_keys(compose.resolve(sleeper))}
        self.assertEqual(compose.compose_person(sleeper, compose.LayerAtlas(sleeping)).size, (32, 16))
        layers[keys[1]] = Image.new('RGBA', (16, 32))
        with self.assertRaises(compose.CompositionError):
            _ = compose.compose_person(person, atlas)
        self.assertTrue(np.array(south)[:, :, 3].any())
