# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 -m unittest discover -s tools/art_pipeline -p test_v4_contracts.py
"""Regression guards for the approved unified scale and chosen-slot semantics."""
from dataclasses import replace
import unittest

from PIL import Image

from v4_contracts import (ContractError, FrameCandidate, FrameSpec, WorldScale,
                          guard_chosen, guard_frame, guard_group, plan_factors, target_size, foot_anchor)


def actor(cell: int = 1, activity: str = 'idle', height: int = 33, sheet: str = 'V_a1',
          attempt: int = 1, body: str = 'adult_a', view: str = 'S') -> FrameSpec:
    return FrameSpec(sheet,cell,attempt,body,view,activity,(19,height))


def candidate(frame: FrameSpec, size: tuple[int,int] = (19,28), berries: int = 0,
              fire: int = 0, drift: float = 0) -> FrameCandidate:
    image = Image.new('RGBA',size,(120,80,40,255))
    return FrameCandidate(frame,image,drift,100.0,True,berries,fire)


class V4ContractTests(unittest.TestCase):
    def test_v5_despeckle_bands(self) -> None:
        item = candidate(actor())
        self.assertNotIn('despeckle_over_3_percent', guard_frame(replace(item, despeckle_fraction=.03)).warnings)
        self.assertIn('despeckle_over_3_percent', guard_frame(replace(item, despeckle_fraction=.10)).warnings)
        self.assertTrue(guard_frame(replace(item, despeckle_fraction=.10)).passed)
        self.assertIn('despeckle_over_10_percent', guard_frame(replace(item, despeckle_fraction=.10001)).errors)

    def test_v5_wood_caps_bend_and_figure_top(self) -> None:
        self.assertEqual(target_size(actor(activity='carry_wood')), (30, 28))
        self.assertEqual(target_size(actor(activity='gather_wood', body='child')), (26, 21))
        bend = replace(candidate(actor(activity='gather_wood'), (29, 18)), frame_index=0)
        self.assertTrue(guard_frame(bend).passed)
        self.assertIn('standing_height', guard_frame(replace(bend, frame_index=1)).errors)
        standing = replace(bend, image=Image.new('RGBA', (29,28), (120,80,40,255)), frame_index=1)
        self.assertTrue(guard_group([bend, standing]).passed)
        self.assertTrue(guard_chosen(adults=[bend, standing], children=[candidate(actor(body='child'), (15,21))]).passed)
        load = replace(candidate(actor(activity='carry_wood'), (30,32)), hair_mask_top=4)
        self.assertTrue(guard_frame(load).passed)
        self.assertTrue(guard_group([load]).passed)
        self.assertIn('standing_height', guard_frame(replace(load, hair_mask_top=None)).errors)

    def test_v5_wood_anchor_uses_feet(self) -> None:
        image = Image.new('RGBA', (30, 30))
        for y in range(27,30):
            for x in range(3,8):
                image.putpixel((x,y), (120,80,40,255))
        self.assertEqual(foot_anchor(image, 'carry_wood'), (5,29))
        self.assertEqual(foot_anchor(image, 'idle'), (15,29))

    def test_all_object_targets(self) -> None:
        cases = [('S1',1,(24,26)),('S1',7,(24,26)),('S1',16,(24,26)),('S1',8,(24,22)),
                 ('S1',14,(24,22)),('S1',15,(24,22)),('S1',11,(34,30)),('S1',13,(34,30)),
                 ('S2',1,(34,30)),('S2',6,(34,30)),('S2',7,(24,22)),
                 ('N1',1,(40,48)),('N1',3,(40,48)),('N1',4,(30,40)),('N1',8,(30,40)),
                 ('N1',16,(30,40)),('N1',14,(16,24)),('N1',9,(24,18)),('N1',13,(24,18)),
                 ('N1',15,(24,18)),('N2',4,(26,22)),('N2',5,(24,22)),('N2',10,(32,30)),
                 ('N2',11,(32,30)),('N2',16,(24,22)),('N3',1,(12,12)),('F1',1,(16,16)),
                 ('F2',1,(16,12)),('U1',1,(16,16))]
        for sheet,cell,target in cases:
            with self.subTest(sheet=sheet,cell=cell):
                self.assertEqual(target_size(FrameSpec(sheet,cell,1)),target)

    def test_scale_escape_is_unified_and_no_upscale(self) -> None:
        scale = WorldScale(33,25)
        self.assertEqual(target_size(actor(),scale),(22,33))
        self.assertEqual(target_size(actor(body='child'),scale),(17,25))
        self.assertEqual(target_size(FrameSpec('N1',1,1),scale),(47,57))
        self.assertEqual(target_size(FrameSpec('U1',1,1),scale),(19,19))
        decisions = plan_factors([actor(height=10),FrameSpec('N1',1,1,native_size=(4,4))],scale)
        self.assertTrue(all(d.factor==1 for d in decisions))
        with self.assertRaises(ContractError):
            WorldScale(33,21)

    def test_group_factor_uses_median_and_not_per_frame(self) -> None:
        frames = [actor(i+1,'walk',height=h) for i,h in enumerate((31,33,33,35))]
        decisions = plan_factors(frames)
        self.assertTrue(all(d.factor==28/33 for d in decisions))

    def test_nonstanding_attempt_and_other_body_attempt_medians(self) -> None:
        frames = [actor(1,'idle',28),actor(2,'walk',40),actor(3,'sit',20),
                  actor(1,'asleep',12,sheet='V_a3'), actor(1,'sit',20,body='adult_b',sheet='V_b3')]
        decisions = plan_factors(frames)
        self.assertAlmostEqual(decisions[2].factor,.85)
        self.assertEqual(decisions[2].source,'attempt_standing_median')
        self.assertAlmostEqual(decisions[3].factor,.85)
        self.assertEqual(decisions[3].source,'body_other_attempts_median')
        self.assertEqual(decisions[4].warnings,('no_standing_reference',))

    def test_no_cross_attempt_contamination(self) -> None:
        frames = [actor(1,'idle',28,attempt=1),actor(2,'sit',10,attempt=1),
                  actor(1,'idle',56,attempt=2),actor(2,'sit',10,attempt=2)]
        decisions = plan_factors(frames)
        self.assertEqual(decisions[1].factor,1)
        self.assertEqual(decisions[3].factor,.5)

    def test_other_attempts_have_equal_weight(self) -> None:
        frames = [actor(1,'idle',28,attempt=1),actor(2,'walk',28,attempt=1),
                  actor(3,'carry',28,attempt=1),actor(1,'idle',56,attempt=2),
                  actor(1,'asleep',12,sheet='V_a3',attempt=3)]
        self.assertEqual(plan_factors(frames)[-1].factor,.75)

    def test_frame_error_does_not_poison_group_peers(self) -> None:
        frames = [candidate(actor(1)),candidate(actor(2),drift=13)]
        self.assertFalse(guard_frame(frames[1]).passed)
        self.assertTrue(guard_group(frames).passed)

    def test_colour_thresholds_are_exact(self) -> None:
        item = candidate(actor())
        self.assertTrue(guard_frame(replace(item,mean_delta_e=6)).passed)
        self.assertEqual(guard_frame(replace(item,mean_delta_e=6)).warnings,())
        self.assertEqual(guard_frame(replace(item,mean_delta_e=12)).warnings,('colour_drift',))
        self.assertIn('gross_colour_error',guard_frame(replace(item,mean_delta_e=12.01)).errors)
        self.assertIn('gross_colour_error',guard_frame(replace(item,mean_delta_e=float('nan'))).errors)

    def test_format_binary_and_feet(self) -> None:
        item = candidate(actor())
        self.assertIn('palette_membership',guard_frame(replace(item,palette_match_percent=99.99)).errors)
        pixels = item.image.copy()
        pixels.putpixel((0,0),(120,80,40,127))
        self.assertIn('binary_alpha',guard_frame(replace(item,image=pixels)).errors)
        padded = Image.new('RGBA',(19,29))
        padded.paste(item.image,(0,0))
        self.assertIn('feet_baseline',guard_frame(replace(item,image=padded)).errors)

    def test_character_geometry_and_group_median(self) -> None:
        self.assertIn('geometry',guard_frame(candidate(actor(),(23,28))).errors)
        self.assertIn('standing_height',guard_frame(candidate(actor(),(19,31))).errors)
        self.assertIn('geometry',guard_frame(candidate(actor(activity='asleep'),(31,16))).errors)
        self.assertIn('geometry',guard_frame(candidate(actor(activity='asleep',body='child'),(24,13))).errors)
        frames = [candidate(actor(i+1),size=(19,30)) for i in range(2)]
        self.assertIn('group_median_height',guard_group(frames).errors)
        frames = [candidate(actor(i+1),size=(19,29)) for i in range(2)]
        self.assertTrue(guard_group(frames).passed)
        self.assertIn('mixed_animation_group',guard_group([frames[0],replace(frames[1],frame=actor(attempt=2))]).errors)

    def test_fire_and_exact_native_icon_canvas(self) -> None:
        fire = candidate(FrameSpec('S1',2,1),(24,26),fire=11)
        self.assertIn('fire_ramp',guard_frame(fire).errors)
        self.assertTrue(guard_frame(replace(fire,fire_texels=12)).passed)
        self.assertTrue(guard_frame(candidate(FrameSpec('U1',1,1),(16,16))).passed)
        self.assertIn('icon_canvas',guard_frame(candidate(FrameSpec('U1',1,1),(15,16))).errors)

    def test_chosen_berry_child_and_basket_guards(self) -> None:
        bushes = [candidate(FrameSpec('N2',i+1,1),(26,22),berries=c) for i,c in enumerate((9,5,1,0))]
        adult = candidate(actor())
        child = candidate(actor(body='child'),(14,21))
        basket = candidate(FrameSpec('S2',11,1),(20,16))
        self.assertTrue(guard_chosen(bushes,[adult],[child],[basket]).passed)
        self.assertIn('berry_stage_order',guard_chosen(list(reversed(bushes))).errors)
        self.assertIn('child_relative_height',guard_chosen(adults=[adult],children=[replace(child,image=Image.new('RGBA',(14,28))) ]).errors)
        self.assertIn('basket_relative_size',guard_chosen(bushes,baskets=[replace(basket,image=Image.new('RGBA',(26,22)))]).errors)
        self.assertIn('missing_chosen_adult_reference',guard_chosen(children=[child]).errors)
        self.assertIn('missing_chosen_bush_reference',guard_chosen(baskets=[basket]).errors)

    def test_four_frame_leg_and_head_guards(self) -> None:
        frames = [candidate(actor(i+1,'walk')) for i in range(4)]
        for i in (1,3):
            for x in range(4):
                frames[i].image.putpixel((x,26),(220,180,90,255))
        self.assertTrue(guard_group(frames).passed)
        frames[1].image.putpixel((3,26),(120,80,40,255))
        self.assertIn('leg_difference',guard_group(frames).errors)
        frames[1] = candidate(actor(2,'walk'),(19,30))
        self.assertIn('head_top',guard_group(frames).errors)

    def test_two_frame_north_carry_is_explicit_pair(self) -> None:
        frames = [candidate(actor(i+1,'carry',view='N')) for i in range(2)]
        for x in range(4):
            frames[1].image.putpixel((x,26),(220,180,90,255))
        report = guard_group(frames)
        self.assertTrue(report.passed)
        self.assertIn('two_frame_cycle',report.warnings)
        partial = guard_group([replace(c,frame=replace(c.frame,view='S')) for c in frames])
        self.assertIn('pending_cycle_completion',partial.warnings)
        self.assertFalse(partial.ready_for_review)


if __name__ == '__main__':
    unittest.main()
