# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 -m unittest discover -s tools/art_pipeline -p test_regions_v4.py
"""Exact reviewed sprite pixels guard anatomy and legacy compatibility."""
import base64
import json
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch
import zlib

import numpy as np
from PIL import Image

import regions

# Lossless source RGBA fixtures from the named reviewed candidates; no cache dependency.
FIXTURES = (
    ('V_b1_09@3', (15, 30), 'adult_b', 'E', 'walk', (5, 14), 5, 2,
     'c$|%tu}T9$6h#*TDr{l25F%J9L<1p(Hrk1WpAd_Hg_ZdMJHaAg<1biS`U7I^52#;YZ(|!d19y2ld2iUjWhcy@d2jCA*{WLo^{Q$a$$8!7HrLmh@qWEg<SjSe-Cj-)wvrg5F0AFGe%<TD<{bTg+>S!l)|6n>Ys7l{<jMQ+-}lE!5sRnH!)XmzaY(*<buUKk-ryh42d$cx&HL5<{(fGQV55BK|JiWR^b7O#c9)*i{C!^(S)B)CI3%B0<Ma8d$f{pFVJwf}Q0|qtSJsh7i2B#3dquBQ^JzM4)s1}i_?Mfb#4{Fa{f;sT1B=wB_);z>vqi&2d?{OEsE@Z|mCwj)%PJRWemO2?OY13%%7d*{^4W6K*0%zWy?t7Ca=4RZ{nS3YoUfM!Wh=@RJ<%ucJ)G1ltarB7-+X<o$$o8~^LgFhoYg#Yx!<}&_Re^HdEc!_(LL#+605m?l`R(0'),
    ('V_b2_01@3', (14, 28), 'adult_b', 'S', 'carry', (5, 16), 4, 5,
     'c$}4#y-Gtt5JndQMu-rx5F*%U5f#Kj8-tAxp#~d~JcONAf{l&6)|NJcjfgLyK7qZBjYZ@PeAmIuZth(e?*4pdXLfe(ru5fZDUC~(GEM7)ey`GQE-e%}zwg%CqOtRxR`rco&!9WrTWQ2eM{M___3_8~w#`lB(4#@`tqzBdzL;EI{TQy>o|*8cTt%Gn{r-G5;qUEknDxgQzCRz#&|M$(&FHh&e^y3(ta}#InEXZ@F|j5s`!G-c@%Es)7tQ;#5SQkDHU_47nrqvbwTM$3UJFZKS~Q+h3{md<Ty&qBBc6-X9s8c_E?eDU_{rtQ@BeypXe>4E|E1^-m2;heFNXh8eTwt<Cu_o~4!&M4r~bFPOX5=v<ldNw^L1P2eeoRkaJAp7#jSQ`tM?4X*ItRk2U^ajejSdQ$kj!OD_IjR($anbr4H_c'),
    ('V_a2_13@3', (26, 31), 'adult_a', 'E', 'build', (13, 3), 4, 5,
     'c$~GCJ4?e*6vqPv6*~ABgbEHmYYS4u2l!ZQu@2(kVxuS`b+Lnspo7x6OIA0Xbx<5#++6AhP(Ohm!8P)~aKho__9pF33WuAf$^UoH>z+DJ{7=Sld~RL5+A||99%<e**8P{K%hXt;2L`vn{=QT>7iVQ<HvX7$0Xse3_OBXyYVDQ7_2$P${L<X0x*p1A)cVt0^dIX5>-^Ar&^_FkR2&%Rz>;7<7Yx{1KBvC7&x=}4w+zlfyr9qHM%uO<9tZN_WOY2GofHQ?AYNE`{yD>zLku1(ol$SvaIk-v)NXC&)cvXKfOQQ}-X(*_&h|Xx8q$xi2RE_vl5?Q@_xl=H8l2knu<zCKfqo}W)EOA%3v|IkywE?DV)$M9j+)@L7scWX9xI%e-|yE!o%3g*|M}r5Yroy8S%GWQ8Ls1S150wCi{GG))oUIpMqCc|&h<?kvgiBTfDwm7H4W{kJVTqTey|lVs+FaVIg))tGXOb2j&ySkGaJ5uu~RN7#=^>sv8N@6Y6x|wn`@Mp!t8`v>HiPa1(@=`gNzy9J9v_VdsG83iWiuj^2Cj1o4k)6q8TA;fNO#SA1GJu?y|PGz@WLtjeLs3A%DOE$K9`n^w8V7`GRY?FEDt&pbkvug50-GGYN}26HR><=Xw752dQ!;g#'),
    ('V_c1_03@1', (14, 23), 'child', 'N', 'idle', (8, 5), 2, 3,
     'c$}4#F$%&!5JeXPT3BdhuSGCbZeXEyRyHDNA$R~w8}SBd?6tNLynuyZ<pJ~t9>F&97k=1`lT8eKCYhO+ne6PwnEVwOlS&!SrR7)Q;cwP!HXirdnS3mLZoX6z7lj=gpNzV>d_GIg?Ln)vUiN?bl|7CJv3)-U27+JKl(pu=&@!+t<KtkNCmD$AUG{zCV6|EB*`1$jeGGJ3Md{<7q^(AfK993Z@E5HoV#s-2H!2Hh9An@#_O_p@dloU|fk}3YjQ_kZSzPu@KP(t>s0mwzJ0G?-9|BKHj%!Y#M_ATjUB092Y_d06-j~f0>&9>5zAHPo=v~fAo1M02+#Hv4_w+yiBw6nZiSn{l'),
    ('V_c2_16@2', (13, 24), 'child', 'S', 'hug', (4, 6), 3, 2,
     'c$}S+Jqp4=5QP^4T3Gl~2(cE#fR&w{=pAA!R-VAl+QutbTY3Qx;04qZcm&&(7kSHM-0?5)$Yk@reY=y5bLA^>&TBd2lAe0C^1^9Wjt@F5KkB7Hjam}rI5#|bw^?E+X6qd3LEd#e^pos4E^}lq{ixxGur|qPKi+S~soyX9fq}SB&2TOBFatszV<<Vlm818>d7;xXi}pH}L9gUv487JGEk+A-NdKp`acn+qn`ejTXPfte&bwKtt@8$5j;q3Zza0v_V<7n9!<{vF$YRi2_;81hEMzg1?18MU|4x68Sg);qThCU;FU&Xkmw5onSeF<'),
 )


def fixture(index: int) -> Image.Image:
    """Decode exact original RGBA, including its transparent texels."""
    row = FIXTURES[index]
    return Image.frombytes('RGBA', row[1], zlib.decompress(base64.b85decode(row[8])))


class RegionV4Tests(unittest.TestCase):
    def test_reviewed_anatomical_pixels(self) -> None:
        for index, row in enumerate(FIXTURES):
            with self.subTest(candidate=row[0]):
                source = fixture(index)
                before = regions.classify(source, row[2])
                after = regions.classify(source, row[2], view=row[3], activity=row[4])
                self.assertEqual(before.getpixel(row[5])[0], row[6])
                self.assertEqual(after.getpixel(row[5])[0], row[7])
                self.assertEqual(source.tobytes(), fixture(index).tobytes())
                values = np.asarray(after)
                np.testing.assert_array_equal(values[...,3], np.asarray(source)[...,3])
                self.assertTrue(np.all(values[...,1] <= 2))
                np.testing.assert_array_equal(values[...,0] == 1, np.asarray(before)[...,0] == 1)

    def test_default_context_keeps_legacy_d3_result(self) -> None:
        source = fixture(0)
        plain = regions.classify(source, 'adult_b')
        explicit = regions.classify(source, 'adult_b', view='', activity='')
        self.assertEqual(plain.tobytes(), explicit.tobytes())
        # Hair cap and cream garment are untouched by the narrow shadow correction.
        contextual = regions.classify(source, 'adult_b', view='E', activity='walk')
        old, new = np.asarray(plain), np.asarray(contextual)
        np.testing.assert_array_equal(old[:10,:,0], new[:10,:,0])
        np.testing.assert_array_equal(old[old[...,0] == 3], new[old[...,0] == 3])

    def test_exact_patch_requires_digest_and_context(self) -> None:
        source = fixture(2)
        x,y = FIXTURES[2][5]
        self.assertEqual(regions.classify(source, 'adult_a', view='E', activity='build').getpixel((x,y))[0], 5)
        wrong = regions.classify(source, 'adult_a', view='S', activity='build')
        self.assertEqual(wrong.getpixel((x,y))[0], 4)
        altered = source.copy()
        altered.putpixel((0,0), (1,2,3,0))
        self.assertEqual(regions.classify(altered, 'adult_a', view='E', activity='build').getpixel((x,y))[0], 4)

    def test_region_build_preserves_rect_and_writes_only_explicit_diagnostic(self) -> None:
        with TemporaryDirectory() as directory:
            root = Path(directory)
            source = fixture(2)
            page = Image.new('RGBA', (64,64))
            page.paste(source, (3,4))
            page.save(root/'world.png')
            manifest = root/'world_atlas.json'
            manifest.write_text(json.dumps({'pages':['res://world.png'], 'sprites':{
                'V_a2_13':{'page':'res://world.png','rect':[3,4,*source.size],
                'body':'adult_a','view':'E','activity':'build'}}}))
            # Keep production palettes read-only while redirecting generated atlas artifacts.
            original = regions.classify
            def local_classify(image: Image.Image, body: str, **context: str) -> Image.Image:
                return original(image, body,
                    palette_path=Path(__file__).with_name('production_palette.json'),
                    overrides_path=Path(__file__).with_name('region_overrides.json'), **context)
            with patch.object(regions, 'ROOT', root), patch.object(regions, 'classify', local_classify):
                result = regions.build_atlases(manifest)
                self.assertFalse((root/'docs').exists())
                diagnostic = root/'production2/regions.png'
                regions.build_atlases(manifest, diagnostic_path=diagnostic)
                self.assertTrue(diagnostic.is_file())
            self.assertEqual(result['sprites']['V_a2_13']['rect'], [3,4,*source.size])
            with Image.open(root/'world_regions.png') as indices:
                self.assertEqual(indices.getpixel((3+13,4+3))[0], 5)


if __name__ == '__main__':
    unittest.main()
