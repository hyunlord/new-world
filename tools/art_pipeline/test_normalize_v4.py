# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 -m unittest discover -s tools/art_pipeline -p test_normalize_v4.py
"""Literal Art Pass2 section6.4 tests, including the mandated80% comparison."""
from __future__ import annotations
import unittest
import numpy as np
from PIL import Image
from test_normalize_v3 import enlarged
from pixel_normalize_v3 import recover_cell_v3, quantize_native
from pixel_normalize_v4 import fit_cell_v4, measure_cell_v4, palette_mode

PALETTE=[(72,64,44),(95,84,60),(131,110,80),(174,150,111),(212,182,142),(210,25,30),(30,130,70),(40,100,200)]

class NormalizeV4Test(unittest.TestCase):
    def test_random_sprite_fits_and_agrees_with_nearest_at_least80percent(self) -> None:
        native=np.array(PALETTE,dtype=np.uint8)[np.random.default_rng(91).integers(0,8,(33,19))]
        result=fit_cell_v4(enlarged(native,5.9,1.7),PALETTE,28/33)
        self.assertLessEqual(abs(result.native.height-28),1)
        reference=Image.fromarray(native).resize(result.native.size,Image.Resampling.NEAREST)
        agreement=float(np.all(np.asarray(result.native)[...,:3]==np.asarray(reference),axis=2).mean())
        self.assertGreaterEqual(agreement,.80,f'Literal nearest agreement {agreement:.6f}')

    def test_physical_origin_is_preserved_and_factor_one_matches_v3(self) -> None:
        native=np.array(PALETTE,dtype=np.uint8)[np.random.default_rng(91).integers(0,8,(33,19))]
        source=enlarged(native,5.9,1.7)
        old,metadata=recover_cell_v3(source)
        unchanged=fit_cell_v4(source,PALETTE,1,measurement=metadata)
        np.testing.assert_array_equal(np.asarray(unchanged.native),np.asarray(quantize_native(old,PALETTE)))
        fitted=fit_cell_v4(source,PALETTE,28/33,measurement=metadata)
        for axis in range(2):
            metrics=fitted.metrics
            self.assertAlmostEqual(metrics['phase_xy'][axis]-metrics['period_xy'][axis],
                                   metrics['sampling_phase_xy'][axis]-metrics['new_period_xy'][axis])
            self.assertAlmostEqual(metrics['phase_origin_xy'][axis],
                                   metrics['sampling_phase_xy'][axis]-metrics['new_period_xy'][axis])

    def test_factor_never_exceeds_one(self) -> None:
        native=np.full((12,8,3),[95,84,60],dtype=np.uint8)
        result=fit_cell_v4(enlarged(native,5.9,1.7),PALETTE,2)
        self.assertEqual(result.metrics['factor'],1)
        self.assertLessEqual(result.native.height,result.metrics['native_before'][1])

    def test_mode_majority_and_darker_tie(self) -> None:
        self.assertEqual(palette_mode(np.array([[200,200,200],[20,20,20],[200,200,200]],dtype=np.uint8)),(200,200,200))
        self.assertEqual(palette_mode(np.array([[200,200,200],[20,20,20]],dtype=np.uint8)),(20,20,20))

    def test_outline_survives_fitting_and_alpha_binary(self) -> None:
        native=np.full((33,19,3),[174,150,111],dtype=np.uint8)
        native[[0,-1],:]=[72,64,44];native[:,[0,-1]]=[72,64,44]
        result=fit_cell_v4(enlarged(native,5.9,1.7),PALETTE,28/33)
        rgba=np.asarray(result.native)
        self.assertTrue(np.isin(rgba[...,3],[0,255]).all())
        dark=np.array([72,64,44],dtype=np.uint8)
        for border in (rgba[0,:,:3],rgba[-1,:,:3],rgba[:,0,:3],rgba[:,-1,:3]):
            self.assertTrue(np.all(border==dark),'Original dark border lost before outline addition')
        self.assertEqual(result.metrics['palette_match_percent'],100)

    def test_icon_native_canvas_exact16(self) -> None:
        native=np.full((10,12,3),[174,150,111],dtype=np.uint8)
        source=enlarged(native,5.9,1.7)
        measured=measure_cell_v4(source)
        factor=min(1,*(16/size for size in measured['native_size']))
        result=fit_cell_v4(source,PALETTE,factor,icon_canvas=True,measurement=measured)
        self.assertEqual(result.native.size,(16,16))
        self.assertTrue(result.metrics['binary_alpha'])
        self.assertEqual(result.metrics['outlined_canvas'],list(result.sprite.size))

if __name__=='__main__':unittest.main()
