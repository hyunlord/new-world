"""Synthetic A16 failures prevent metadata-only or vacuous acceptance."""
from __future__ import annotations

from dataclasses import replace
import json
from pathlib import Path
import tempfile
import unittest

import numpy as np
from PIL import Image

from ingame_qa_age_p3 import _ages, _hud, _ramps, _terrain, age_seasons
from ingame_qa_types import Capture, Entity, Label


def actor(ident: int = 7, hair: int = 5) -> Entity:
    return Entity('agent', ident, (600, 300, 8, 8), True, 'a', 'a', (2, 2), 'adult_a', (1, hair), (1, 1), 'idle', 'S', '', -1, False, 'res://atlas.png', (0, 0, 2, 2), '', world_anchor=(1, 1))


def capture(path: Path, entities: tuple[Entity, ...] = ()) -> Capture:
    return Capture(path.stem, path, entities, (), 4, 1, -1, 0, 0, 0, 0, (), (), 0)


class AgeSeasonTests(unittest.TestCase):
    def test_hair_requires_observed_elder_and_resolved_ids(self) -> None:
        cap = capture(Path('unused.png'), (actor(),))
        self.assertTrue(_ages(cap, {'agent_snapshot': {'agent_ids': [7], 'life_stages': [2]}})['pass'])
        self.assertFalse(_ages(cap, {'agent_snapshot': {'agent_ids': [7], 'life_stages': [2]}})['P4_old_rule']['pass'])
        self.assertFalse(_ages(cap, {'agent_snapshot': {'agent_ids': [8], 'life_stages': [2]}})['pass'])
        self.assertFalse(_ages(cap, {'agent_snapshot': {'agent_ids': [7], 'life_stages': [0]}})['pass'])
        young = replace(cap, entities=(actor(hair=1),))
        self.assertFalse(_ages(young, {'agent_snapshot': {'agent_ids': [7], 'life_stages': [0]}})['pass'])

    def test_hud_checks_runtime_clock_and_all_four_deaths(self) -> None:
        cap = replace(capture(Path('unused.png')), labels=(Label('40일차 12:00 · 낮 · 여름 5일째', '', '/HudTopbar/clock', (0,0,100,20)), Label('사망 10', '', '/HudTopbar/death', (0,20,100,20))))
        data = {'time_snapshot': {'season': 1, 'day_of_season': 5, 'day_index': 39, 'days_per_season': 7}, 'tallies': {'deaths_starvation': 1, 'deaths_dehydration': 2, 'deaths_combat': 3, 'deaths_old_age': 4}}
        row = _hud(cap, data, 40)
        self.assertTrue(row['runtime_clock_consistent'])
        self.assertTrue(row['season_label_matches_snapshot'])
        self.assertTrue(row['death_label_matches'])
        data['tallies']['deaths_old_age'] = 5
        self.assertFalse(_hud(cap, data, 40)['death_label_matches'])
        data['time_snapshot']['day_of_season'] = 4
        self.assertFalse(_hud(cap, data, 40)['runtime_clock_consistent'])

    def test_terrain_pixel_ratio_and_alignment(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            Image.new('RGB', (1920,1080), (100,80,50)).save(root/'summer.png')
            Image.new('RGB', (1920,1080), (100,80,52)).save(root/'winter.png')
            a, b = capture(root/'summer.png'), capture(root/'winter.png')
            meta = {'camera_position': [10,10], 'zoom': [4,4], 'viewport_size': [1920,1080], 'framebuffer_scale': 1}
            self.assertTrue(_terrain(a,b,meta,meta)['pass'])
            self.assertFalse(_terrain(a,b,meta,{**meta, 'camera_position': [11,10]})['pass'])
            Image.new('RGB', (1920,1080), (100,80,50)).save(root/'winter.png')
            self.assertFalse(_terrain(a,b,meta,meta)['pass'])

    def test_ramps_use_framebuffer_and_exclude_foreground(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root/'assets/art').mkdir(parents=True)
            Image.new('RGBA', (2,2), (120,80,60,255)).save(root/'atlas.png')
            Image.new('RGBA', (2,2), (0,0,0,255)).save(root/'atlas_regions.png')
            Image.new('RGB', (3,14), (120,80,60)).save(root/'assets/art/villager_ramps.png')
            pixels = np.zeros((1080,1920,3), dtype=np.uint8)
            pixels[300:308,600:608] = (115,77,58)
            Image.fromarray(pixels).save(root/'screen.png')
            cap = capture(root/'screen.png', (actor(),))
            self.assertTrue(_ramps(cap,root)['pass'])
            pixels[300:308,600:604] = (0,0,255)
            Image.fromarray(pixels).save(root/'screen.png')
            self.assertFalse(_ramps(cap,root)['pass'])
            panel = replace(actor(), kind='panel', rect=(600,300,3,8))
            self.assertTrue(_ramps(replace(cap,entities=(actor(),panel)),root)['pass'])
            self.assertFalse(_ramps(replace(cap,entities=(actor(),replace(panel,rect=(600,300,8,8)))),root)['pass'])

    def test_missing_evidence_fails(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            outcome = age_seasons(Path(folder), Path(folder))
            self.assertEqual(outcome.status, 'FAIL')
            self.assertEqual(len(json.loads(outcome.detail)['missing_captures']), 4)


if __name__ == '__main__':
    unittest.main()
