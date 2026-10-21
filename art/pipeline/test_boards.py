import unittest
from dataclasses import replace

from PIL import Image

from . import boards
from .compose import AgentId, HouseholdId, Person
from .models import PipelineError


def sprite(index: int) -> boards.PersonSprite:
    person = Person(AgentId(index), 'male', 25, HouseholdId(index))
    image = Image.new('RGBA', (32, 32), (index, 90, 100, 255))
    return boards.PersonSprite(person, image)


class BoardTests(unittest.TestCase):
    def test_scene_counts_crops_and_native_pixels(self) -> None:
        env = {name: Image.new('RGBA', size, (30, 50, 70, 255)) for name, size in boards.REQUIRED_ENV.items()}
        people = {role: sprite(index) for index, role in enumerate(boards.PERSON_POSITIONS)}
        people['child'] = boards.PersonSprite(replace(people['child'].person, age=10), people['child'].image)
        people['elder'] = boards.PersonSprite(replace(people['elder'].person, age=60), people['elder'].image)
        people['sleeper'] = boards.PersonSprite(replace(people['sleeper'].person, state='sleep'), Image.new('RGBA', (32, 16)))
        for role in ('carry_1', 'carry_2'):
            people[role] = boards.PersonSprite(replace(people[role].person, carry='logs'), people[role].image)
        for role in ('chop_1', 'chop_2'):
            people[role] = boards.PersonSprite(replace(people[role].person, job='woodcutter', state='work'), people[role].image)
        result = boards.render_scene(env, people)
        self.assertEqual(result.image.size, (640, 960))
        self.assertEqual(len(result.crops), 10)
        self.assertEqual(sum(p.name.startswith('tree_') for p in result.placements), 15)
        self.assertEqual(sum(p.name == 'lean_to' for p in result.placements), 3)
        self.assertEqual(sum(p.name.startswith('person:') for p in result.placements), 10)
        self.assertEqual(sum(p.name.startswith('stockpile_floor') for p in result.placements), 16)
        self.assertEqual(result.image.getpixel((0, 0)), (30, 50, 70, 255))
        del env['hut']
        with self.assertRaises(PipelineError):
            _ = boards.render_scene(env, people)

    def test_ab_crops_without_resizing(self) -> None:
        scene = Image.new('RGBA', (640, 960), (1, 2, 3, 255))
        baseline = Image.new('RGBA', (1920, 1080), (4, 5, 6, 255))
        baseline.putpixel((10, 10), (90, 80, 70, 255))
        baseline.putpixel((650, 70), (99, 98, 97, 255))
        result = boards.render_ab(scene, baseline, baseline)
        self.assertEqual(result.size, (1920, 1080))
        self.assertEqual(result.getpixel((650, 70)), (99, 98, 97, 255))
        self.assertEqual(result.getpixel((0, 60)), scene.getpixel((0, 0)))
        self.assertEqual(boards.ab_crop_rect(baseline), (640, 60, 1280, 1020))
        explicit = boards.render_ab(scene, baseline, baseline, crop_origins=((0, 0), (640, 60)))
        self.assertEqual(explicit.getpixel((650, 70)), (90, 80, 70, 255))
        self.assertEqual(explicit.getpixel((1290, 70)), (99, 98, 97, 255))
        with self.assertRaises(PipelineError):
            _ = boards.render_ab(scene, baseline, baseline, crop_origins=((1281, 0), None))
        with self.assertRaises(PipelineError):
            _ = boards.ab_crop_rect(baseline, (-1, 0))
        with self.assertRaises(PipelineError):
            _ = boards.render_ab(scene, Image.new('RGBA', (100, 100)), baseline)

    def test_people_require_unique_24_and_all_bodies_directions(self) -> None:
        people: list[boards.PersonSprite] = []
        for index in range(24):
            body = index % 4
            age = (25, 25, 10, 60)[body]
            person = replace(sprite(index).person, sex='female' if body == 1 else 'male', age=age,
                             direction='south' if (index // 4) % 2 == 0 else 'east')
            people.append(boards.PersonSprite(person, sprite(index).image))
        result = boards.render_people(people)
        self.assertEqual(result.size, (1600, 1440))
        self.assertEqual(result.getpixel((152, 8)), people[0].image.getpixel((0, 0)))
        people[-1] = people[0]
        with self.assertRaises(PipelineError):
            _ = boards.render_people(people)
