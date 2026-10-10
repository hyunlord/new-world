"""Regression checks for evidence rules that must not accidentally pass."""
from dataclasses import replace
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
import json
import numpy as np
from PIL import Image
from ingame_qa import interface, variety
from ingame_qa_pixels import checker
from ingame_qa_types import Capture, Entity, Label, load_capture


def entity(kind: str = 'agent') -> Entity:
    return Entity(kind,1,(0,0,64,64),True,'frame','frame',(8,8),'adult_a',(0,0,0),(0,0),
                  'idle','S','',-1,False,'',(0,0,0,0),'Panel')


def capture(path: Path, entities: tuple[Entity,...]) -> Capture:
    return Capture(path.stem,path,entities,(),4,1,-1,0,0,0,0,(),(),0)


class IngameQARegression(unittest.TestCase):
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

    def test_variety_rejects_identical_neighbors(self) -> None:
        actors = tuple(replace(entity(),ident=i,tile=(i,0)) for i in range(12))
        self.assertEqual(variety(capture(Path('unused'),actors)).status,'FAIL')
        diverse = tuple(replace(actor,variant=(i%4,i%5,i%6)) for i,actor in enumerate(actors))
        self.assertEqual(variety(capture(Path('unused'),diverse)).status,'PASS')

if __name__ == '__main__':
    unittest.main()
