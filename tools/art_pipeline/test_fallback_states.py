"""Player-visible fallback state distinctions and native constraints."""
import unittest
from install_fallbacks import stateful_object, storage_from_base
from PIL import Image
import numpy as np
from v3_semantics import semantic_counts
from pixel_masks import make_mask

class FallbackStateTest(unittest.TestCase):
    def test_hearth_states_show_fire_only_when_lit(self):
        palette=[(12,16,17),(90,70,45),(140,130,110),(255,150,25),(250,220,110)]
        lit=[stateful_object('S1',f'hearth_lit_{i}',(24,26),palette)[0] for i in range(4)]
        self.assertEqual(len({im.tobytes() for im in lit}),4)
        self.assertTrue(all(semantic_counts(im)[1]>=20 for im in lit))
        cold=stateful_object('S1','hearth_unlit',(24,26),palette)[0]
        self.assertEqual(semantic_counts(cold)[1],0)

    def test_storage_construction_and_trees_are_distinct(self):
        palette=[(12,16,17),(90,70,45),(140,130,110),(255,150,25),(250,220,110)]
        for sheet,names,size in [('S1',['storage_empty','storage_half','storage_full'],(34,30)),
              ('S2',['construction_stakes','construction_frame','construction_half','construction_near'],(34,30)),
              ('N1',['broadleaf_large_1','broadleaf_large_2','broadleaf_large_3','conifer_1','dead_tree'],(32,40))]:
            images=[stateful_object(sheet,name,size,palette)[0] for name in names]
            self.assertEqual(len({im.tobytes() for im in images}),len(images))
            self.assertTrue(all(im.width<=size[0] and im.height<=size[1] for im in images))

    def test_effect_and_bird_frames_change(self):
        palette=[(12,16,17),(90,70,45),(140,130,110),(255,150,25)]
        for name in ('bird','smoke','fish','splash'):
            frames=[make_mask(name,palette,(16,16),i) for i in range(1,5)]
            self.assertEqual(len({im.tobytes() for im in frames}),4,name)

    def test_storage_fill_preserves_accepted_base_footprint(self):
        palette=[(12,16,17),(90,70,45),(140,130,110),(255,150,25)]
        base=Image.new('RGBA',(33,21),(90,70,45,255))
        base.putpixel((0,0),(0,0,0,0))
        original=base.tobytes()
        half=storage_from_base(base,palette,1)
        full=storage_from_base(base,palette,2)
        self.assertEqual(base.tobytes(),original)
        self.assertEqual(half.size,base.size)
        self.assertEqual(full.size,base.size)
        self.assertEqual(half.getchannel('A').tobytes(),base.getchannel('A').tobytes())
        self.assertEqual(full.getchannel('A').tobytes(),base.getchannel('A').tobytes())
        changed=lambda image:int(np.any(np.asarray(image)!=np.asarray(base),axis=2).sum())
        self.assertGreater(changed(half),0)
        self.assertGreater(changed(full),changed(half))

if __name__=='__main__':unittest.main()
