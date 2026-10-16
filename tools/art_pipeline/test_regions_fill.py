"""Lossless fill-source fixtures protect the bounded second-review region corrections."""
import base64
import hashlib
import json
from pathlib import Path
import unittest
import zlib

import numpy as np
from PIL import Image

import regions

FIXTURES = (('P2_fill_03_01@1', (13, 23), 'child', 'S', 'walk', (5, 11), 2, 'c${Nkzy!<;4FAzF7G)S}`Ptb}XhP~iIM782jIGoqvD$|$rmrCe#xOlFjI0OUEg<>&M3evCPWoWIi7sL|^$2h?!_5S-ug!}9(;y7ei!EGWdO-3Z^~f0JUUWSmx!<2Ig58L$9whhe*#<ZpT@N-4lf&gcbUR=e7CyMb8(n>GT@1naK~{f#Z#h`cieyckdO&h8^`LNs>A~iIWIJH`VCrFdPG)N1^gBomNIkM1n0dIu8l?8rkzTl7e0pFQX2<;cV7Pv4c?p|+=xl6x2%8>kVN6I5p}b9r9%9Q5QuP1;7Ala6'), ('P2_fill_02_01@1', (18, 32), 'adult_b', 'E', 'walk', (4, 27), 5, 'c$}S-u}VWh5JeXPDx}cZMX*sJ7zh?tijDXY5fM^Y`2}`%0h|1SbxMCgto;G?3+!!dBX{A3>%7^0lRUV*<?ZfyGqZO#rSj8BX;D*UYn8FRxserR>ALYzuc5>4)=C_YaX#q(W39_zeo+qpOO5t+^Zu{L)2v!vT2JgF1K)Iigu|H0!J7<dr`rj~17SaSka8%-8R5_`PnZ1$ClU`Czuzxe7{A{SGtYXltv(o&1EDTpMZBzQGzYS-hG`i~iJ=@@i;w5)%=vtMSerAyJ6`;}O_wo6dfsv{R)%tFy{a6X4eBvD+s{2R8xL|`oG;xUHWr)@azbq&aM-6fH?Fhw@^S)4a{`b1Fz5Nv<0ykdy=r%?{q8YJoSF|y9L=rGKkO~b)p|sF-S#xZd#Bdx8i#t>dc@;<aljwmTFar2JkHI<LG~B#xzak!!|Qep$M8Qm(8=wzk-v?}=BymbtE1(3u`3IHubl@u{HJ=!W48G;+83*EF!u+)g{H~'), ('P2_fill_02_05@1', (14, 30), 'adult_b', 'S', 'carry', (6, 16), 5, 'c$}4#ze)o^5Qh(gMu^~_LWqz`AsDc-GM(T%#70D{d<i?zCa+*^X=!I|X(K*?y%shpPQHcTIyafUkb_}w_kQ2b?A**&)&J9}s=Cdp;PINx?%G0DYx}+RdUCLx?BTTEn3p*AmqzB-+Wg@M&FAfLHs(irI}OgpuDI|chIPw%gTbsHag1;gt98%G*mFGW&*dCOxwmo3r!<}u^ZWIjv`^QE$#`c?XNk@z&FtqruQAtNr8?5LmyK?Hzq*n}eS5nZrI-d6(>Vr{o{_j1#YH}C&f#XGappLu)kNIo$)KK2hV{qedCm{{=N+G(QykK2WDjbu#QAet7B1>1JKySck`EY0;`Y7hYnwCae__mJVx#`EjOW|>{pxHtYfH5farm@3LvMYRIQ>7}2{7NUca6N1IDANF`zX2xea~r5o#4NXi|$S0h2x(t;-cCq<)yTb?fYdodw(p$MJCN3$XzDw'), ('P2_fill_06_09@1', (19, 23), 'child', 'S', 'build', (2, 3), 5, 'c${NkfB<F&hX2SIml!^PuAW$o%PeJXj{omoUHX4=$F%=tS%D~etkfm{2f8Q`Z59ZF%$d-f`9C7e4b0cq5c{9#A_g`G#Ey1W1Jn4-LzY8!6UYpZewbMx8r^;P%(^x&;(xrQEZiKJ7zo4MitILg{sie+k*o=K3(OoCCe<utEWpi-AqI0JA+vhJ)!}NfnSpE$A+ylUKo2is(=9IZVA$J99~_qC<N=UAm^oGPj{mRkEhiXOgyI3@9*{XOvtVu`#Vn8+$mXnDS&R~Pq?k3oJ{WG!sUy8$8kR;#2{U9f`Z`15`4yID`Ptb>icgqX=;o}QTl3$|*@S4bZqylr)q(W(*2TchN{n>}V{;p0qVhK^4w2mhG7sbykQvArUs(*&2Qv%g7myh+b3o;hrkWz$Z7}s9b+GV5HVY(<Y!A#Fm>D3qp!*Rd4=R^&)e*3A8y_a64j`xg006xU20j'), ('P2_fill_05_14@1', (35, 20), 'adult_b', 'S', 'asleep', (14, 14), 5, 'c$~eJy-Gtt5QQ%URJesdML@wuMMV$`Q;Ch}8;CzB7AbuKJA;K_V`Hzrf{mq!rKOGf1om3kSVYdkxt>gBbAQwX4x8P3cjlY3lTDIT`_z)8ou`VSD{%}D4s_+WYyD(7X{7D5wec|DnHdf9ez8~|z(E$}`Y#@@7vKpwn15yNXAY0NGyE%t&$pW(^<Z&gEaZJXUJe2e)^rcX>2#<vPmIH|kU{l$xR80mJr5V#rTy`K6O8wKwHxLmX|1D2l>>X=6Q5nTHUkHwewFcoLJTlq1Opiv*YgF^ddSmiHVWNEdZyUh+^C1|*bjk2Yxg{{EQqIEWI=q$W3e@mMc?mLd)^xQ2et65SfBQgWm%f9XMOf^cND(A-=B8Wh`5mDLLD|gMIXtcik%rb^fyCUPrbW?{e)4axouq3tT84J&NFVCpZ-v{ms?>CBZWEk>KtRNE*1OIyG!{F&U0^dD)1n#7f07)?`ZvqX=_toY|rR<kOl0B!)KO<@6ZQwF+bM#*Jr!Dx1LxQp1Icxe5Sb|vg=*KJ$e_}-u(J_irz0}#k#eRy`C<jqIYke^nZ_YyT*J)ILdr_)%Ke_-;Z;daR'), ('P2_fill_03_09@1', (13, 23), 'child', 'N', 'walk', (5, 4), 3, 'c${NkKn2VU4F8ca10H~`mXKa_`9K#XFvh1BSxjF;?7z2@K3ETk4Z{5FY$$rr&96^10n_;Ofv_?+2i!d{J;)fQ50@XJoz*b(!1ST(xi&8X#V^Fz19C4mJ;>_O{RG1T+{|!wAPnMT3vXn8Z(R)7oyZtp{D9;^dO+qQ>sgVkiPH|4o>ND9;e2d%A*%=BwR3C0damy+2m1vkelk-Fs~(ttK<Z&Q!q*f<9-BQdwJ<k=*!bKB!Z1A`^)O5*tdY&c7uM+EjY|){d<TmULVB>}S7P)KDjO)#0{|i=g$D'))

class FillRegionTests(unittest.TestCase):
    def test_inspected_anatomy_and_props_preserve_source(self) -> None:
        for name, size, body, view, activity, point, expected, packed in FIXTURES:
            with self.subTest(candidate=name):
                raw = zlib.decompress(base64.b85decode(packed))
                image = Image.frombytes('RGBA', size, raw)
                result = regions.classify(image, body, view=view, activity=activity)
                self.assertEqual(result.getpixel(point)[0], expected)
                self.assertEqual(image.tobytes(), raw)
                source = np.asarray(image)
                output = np.asarray(result)
                np.testing.assert_array_equal(source[..., 3], output[..., 3])
                legacy = np.asarray(regions.classify(image, body))
                np.testing.assert_array_equal(legacy[..., 0] == 1, output[..., 0] == 1)
                self.assertTrue(np.all(output[..., 1] <= 2))

    def test_child_cream_hide_and_basket_gripping_hand_remain_separate(self) -> None:
        for index, point, expected in ((0, (8, 13), 4), (2, (2, 12), 2)):
            row = FIXTURES[index]
            image = Image.frombytes('RGBA', row[1], zlib.decompress(base64.b85decode(row[7])))
            result = regions.classify(image, row[2], view=row[3], activity=row[4])
            self.assertEqual(result.getpixel(point)[0], expected)

    def test_original_reviewed_overrides_are_unchanged(self) -> None:
        data = json.loads(Path(regions.__file__).with_name('region_overrides.json').read_text())
        digest = hashlib.sha256(json.dumps(data['reviewed_images'][:34], sort_keys=True).encode()).hexdigest()
        self.assertEqual(digest, 'c537b00e324f685f8645f4b4f6a9c568f095f405ec109c0e776755b49bd9986f')
        for entry in data['reviewed_images'][34:]:
            self.assertTrue(entry['candidate_id'].startswith('P2_fill_'))
            self.assertEqual(len(entry['rgba_sha256']), 64)
            self.assertTrue(entry['patches'])
            for patch in entry['patches']:
                self.assertTrue(patch['rule'])
                self.assertTrue(patch['inspected_bounds'])
                for y, x0, x1 in patch['spans']:
                    self.assertTrue(0 <= y < entry['size'][1])
                    self.assertTrue(0 <= x0 < x1 <= entry['size'][0])

if __name__ == '__main__':
    unittest.main()
