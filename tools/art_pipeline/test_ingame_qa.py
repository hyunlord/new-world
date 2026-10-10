"""Regression checks for evidence rules that must not accidentally pass."""
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch
import json
import numpy as np
from PIL import Image, ImageFont
from ingame_qa import interface, variety
from ingame_qa_pixels import checker, contrast
from ingame_qa_types import Capture, Entity, Label, load_capture
from make_blind_grid import cue_box, compose_grid


def entity(kind: str = 'agent') -> Entity:
    return Entity(kind,1,(0,0,64,64),True,'frame','frame',(8,8),'adult_a',(0,0),(0,0),
                  'idle','S','',-1,False,'',(0,0,0,0),'Panel')


def capture(path: Path, entities: tuple[Entity,...]) -> Capture:
    return Capture(path.stem,path,entities,(),4,1,-1,0,0,0,0,(),(),0)


class IngameQARegression(unittest.TestCase):
    def test_blind_grid_preserves_tall_cues_and_last_row(self) -> None:
        cue = Image.new('RGB',(72,200),(255,0,0))
        grid = compose_grid([cue]*24,ImageFont.load_default())
        self.assertEqual(grid.size,(1200,984))
        pixels = np.asarray(grid)
        red = (pixels == (255,0,0)).all(axis=2)
        self.assertEqual(int(red.sum()),24*72*200)
        for row in range(4):
            self.assertFalse(red[row*246:row*246+38].any())
            self.assertTrue(red[row*246+38:row*246+238,64:136].all())
        self.assertFalse(red[-8:].any())


    def test_blind_crop_includes_icon_without_changing_body_bounds(self) -> None:
        actor = replace(entity(),rect=(100,120,64,96),cue_rect=(100,80,64,136))
        self.assertEqual(cue_box(actor,2),(50,40,82,108))
        self.assertEqual(actor.rect,(100,120,64,96))
        self.assertEqual(cue_box(entity(),1),(0,0,64,64))

    def test_checker_requires_both_zooms_and_detects_one_mixed_pixel(self) -> None:
        with TemporaryDirectory() as temporary:
            directory = Path(temporary)
            records = []
            for zoom in [4,6]:
                block = zoom*2
                patch = np.repeat(np.repeat(np.indices((8,8)).sum(0)%2==0,block,0),block,1)
                image = np.zeros((1080*2,1920*2,3),dtype=np.uint8)
                image[:8*block,:8*block] = patch[...,None]*255
                path = directory/f'z{zoom}.png'
                Image.fromarray(image).save(path)
                actor = replace(entity('checker'),role='checker',rect=(0,0,8*block,8*block))
                records.append(replace(capture(path,(actor,)),zoom=zoom,scale=2))
            self.assertEqual(checker(records).status,'PASS')
            self.assertEqual(checker(records[:1]).status,'FAIL')
            with Image.open(records[0].path) as source:
                corrupted = source.copy()
            corrupted.putpixel((1,1),(128,128,128))
            corrupted.save(records[0].path)
            self.assertEqual(checker(records).status,'FAIL')

    def test_interface_rejects_overflow_latin_and_wrong_font(self) -> None:
        panel = replace(entity('panel'),rect=(0,0,100,40),name='Panel')
        label = Label('인구 4','Galmuri11 Regular','/root/UI/Panel/Label',(5,5,80,25))
        record = replace(capture(Path('unused'),(panel,)),labels=(label,))
        self.assertEqual(interface(record).status,'PASS')
        for bad in [replace(label,text='Population 4'),replace(label,font='System'),replace(label,rect=(80,5,80,25))]:
            self.assertEqual(interface(replace(record,labels=(bad,))).status,'FAIL')

    def test_need_counts_exclude_hidden_but_keep_offscreen_drawn_actors(self) -> None:
        with TemporaryDirectory() as temporary:
            path = Path(temporary)/'capture.png'
            data = {'zoom':[4,4], 'entities':[
                {'kind':'agent','id':1,'rect':[3000,0,72,112],'visible':False,'shown':True},
                {'kind':'agent','id':2,'rect':[10,0,72,112],'visible':False,'shown':False}],
                'agent_snapshot':{'agent_ids':[1,2],'hungers':[76,90],'thirsts':[0,0],'sleeps':[0,0]}}
            path.with_suffix('.entities.json').write_text(json.dumps(data))
            self.assertEqual(load_capture(path).need_ids,(1,))

    def test_faded_canopy_keeps_actual_blended_pixels_measurable(self) -> None:
        actor = replace(entity(),rect=(50,50,4,4),world_anchor=(50,54),draw_order=(4,1))
        tree = replace(entity('tree'),ident=2,rect=(50,50,4,4),world_anchor=(50,60),draw_order=(4,2),alpha=.45)
        record = capture(Path('unused'),(actor,tree))
        image = np.full((1080,1920,3),20,dtype=np.uint8)
        image[50:54,50:54] = 80
        mask = np.zeros((1080,1920),dtype=bool)
        mask[50:54,50:54] = True
        with patch('ingame_qa_types.Capture.image',return_value=image), patch('ingame_qa_pixels.placed',return_value=(image,mask)):
            measured = contrast(record,Path('.'))
            self.assertEqual(measured.status,'PASS')
            self.assertIn('0.2353',measured.detail)
            hidden = contrast(replace(record,entities=(actor,replace(tree,alpha=1))),Path('.'))
            self.assertEqual(hidden.status,'FAIL')
            behind = contrast(replace(record,entities=(actor,replace(tree,alpha=1,world_anchor=(50,40)))),Path('.'))
            self.assertEqual(behind.status,'PASS')

    def test_variety_rejects_identical_neighbors(self) -> None:
        actors = tuple(replace(entity(),ident=i,tile=(i,0)) for i in range(12))
        self.assertEqual(variety(capture(Path('unused'),actors)).status,'FAIL')
        diverse = tuple(replace(actor,variant=(i%3,i%5)) for i,actor in enumerate(actors))
        self.assertEqual(variety(capture(Path('unused'),diverse)).status,'PASS')

if __name__ == '__main__':
    unittest.main()
