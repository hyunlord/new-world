"""Boundary regressions for exact edge pairing and opaque crowd overlap."""
import unittest
import numpy as np
from ingame_qa_pixels_p4 import edge_values

class EdgeTests(unittest.TestCase):
    def test_four_edges_do_not_wrap_at_viewport_boundary(self):
        pixels = np.zeros((3,3)); pixels[0,0] = .8
        visible = np.zeros((3,3),dtype=bool); visible[0,0] = True
        values = edge_values(pixels, visible, ~visible)
        self.assertEqual(values.tolist(), [.8,.8])

    def test_occupied_neighbour_is_excluded(self):
        pixels = np.zeros((3,3)); pixels[1,1] = .8
        visible = np.zeros((3,3),dtype=bool); visible[1,1] = True
        free = ~visible; free[1,2] = False
        self.assertEqual(edge_values(pixels, visible, free).tolist(), [.8,.8,.8])

from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
from PIL import Image
from ingame_qa_pixels_p4 import crowd, edge_contrast
from test_ingame_qa import entity, capture
import json

class PixelEvidenceTests(unittest.TestCase):
    def test_front_union_counts_each_actor_once_and_fails_two_hidden(self):
        with TemporaryDirectory() as folder:
            root = Path(folder)
            Image.new('RGBA',(8,8),(255,255,255,255)).save(root/'sprite.png')
            actors = tuple(replace(entity(),ident=i,rect=(100,100,32,32),texture='res://sprite.png',world_anchor=(0,i)) for i in range(3))
            outcome = crowd(capture(root/'screen.png',actors),root)
            self.assertEqual(outcome.status,'FAIL')
            self.assertEqual(json.loads(outcome.detail)['half_occluded'],2)

    def test_real_framebuffer_edge_contrast_and_panel_exclusion(self):
        with TemporaryDirectory() as folder:
            root = Path(folder)
            Image.new('RGBA',(8,8),(255,255,255,255)).save(root/'sprite.png')
            screen = Image.new('RGB',(1920,1080),(0,0,0))
            screen.paste((255,255,255),(100,100,132,132)); screen.save(root/'screen.png')
            actor = replace(entity(),rect=(100,100,32,32),texture='res://sprite.png')
            cap = capture(root/'screen.png',(actor,))
            self.assertEqual(edge_contrast(cap,root).status,'PASS')
            panel = replace(entity('panel'),rect=(100,100,32,32))
            self.assertEqual(edge_contrast(replace(cap,entities=(actor,panel)),root).status,'FAIL')

    def test_offscreen_overlap_uses_shared_unclipped_mask_space(self):
        with TemporaryDirectory() as folder:
            root = Path(folder)
            Image.new('RGBA',(8,8),(255,255,255,255)).save(root/'sprite.png')
            actors = tuple(replace(entity(),ident=i,rect=(-24,100,32,32),texture='res://sprite.png',world_anchor=(0,i)) for i in range(3))
            outcome = crowd(capture(root/'screen.png',actors),root)
            detail = json.loads(outcome.detail)
            self.assertEqual(detail['half_occluded'],2)
            self.assertEqual(detail['actors'][0]['opaque'],1024)
            self.assertEqual(detail['actors'][0]['visible_opaque'],256)

    def test_viewport_clipping_alone_does_not_occlude(self):
        with TemporaryDirectory() as folder:
            root = Path(folder)
            Image.new('RGBA',(8,8),(255,255,255,255)).save(root/'sprite.png')
            actor = replace(entity(),rect=(-24,100,32,32),texture='res://sprite.png')
            detail = json.loads(crowd(capture(root/'screen.png',(actor,)),root).detail)
            self.assertEqual(detail['actors'][0]['covered'],0)
            self.assertEqual(detail['half_occluded'],0)
