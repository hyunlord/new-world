# ART-TRIAL3 comparison boards

These owner-facing boards are diagnostic static evidence, not blind review stimuli or route viability approval. Route B has no generated art and is explicitly blank, marked UNAVAILABLE. Route A uses the exact first rendered frame of each final GIF slot. Only source PNG/GIF colour values were copied; no resizing, quantization, contour processing, retouching, or new character art.

- [1x board](../../../art/character/boards/compare_AB_1x.png)
- [2x board](../../../art/character/boards/compare_AB_2x.png)

Both boards show the same numbered12 slots, in row-major order (three pairs per row, four rows). At1x each GIF cell is104x112 pixels; at2x it is208x224. A and B columns are side by side within every numbered pair. Display the PNG at100% for literal sprite scale. Each pasted A cell was byte-compared to its decoded first GIF frame. PNGs are RGB; only layout/background/text are added.

Source: `/Users/rexxa/work/arttrial3-data/lab-final-v2/gifs/`; source selection and hashes verified against `answer_key.private.json`. Animated GIFs have average125ms frames (exact8fps over their1s loop, alternating120/130ms due to GIF timing resolution). Sleep has one merged1000ms frame instead of eight identical frames. These first-frame boards cannot show animation or establish activity readability. Sleep slots05 and09 repeat the bare sleeping body; tops/hair are absent in those slots. A diagnostic subset does not certify the incomplete full route set.

| Slot | Scale | Source GIF | Frames | Loop ms | SHA256 |
|---:|---:|---|---:|---:|---|
| 01 | 1x | a_01_1x.gif | 8 | 1000 | `06ffc9c5c363357bc9e75628ed095be5f9cc413582707db4438b73fa784c47fe` |
| 02 | 1x | a_02_1x.gif | 8 | 1000 | `ec3a3dafee2a6407217b12624309f92ed9727607ff9b99cf3fb7eb7157b1692e` |
| 03 | 1x | a_03_1x.gif | 8 | 1000 | `39efe9fda6aa9aa4e02438cf48777614915e5f96231f83dc046ce3fe24d6fe1e` |
| 04 | 1x | a_04_1x.gif | 8 | 1000 | `2fb008dc65f1c29eaf41f4ffce6b8b7f49e6937cae39ad637e1f6d742560da2a` |
| 05 | 1x | a_05_1x.gif | 1 | 1000 | `9db2bb7bf9713dcb604b45213153da08dc778cdbaa02931fa5b95c3f23f9072f` |
| 06 | 1x | a_06_1x.gif | 8 | 1000 | `65ec0e8cd90472263fa7cd4f16ddc04b2022331819f3e19828056abf56853584` |
| 07 | 1x | a_07_1x.gif | 8 | 1000 | `2d90e4bd4c3f424b594f6d53ba1af057ab768621ca5b30ff49db12868e7e8ac8` |
| 08 | 1x | a_08_1x.gif | 8 | 1000 | `f1f5b2529a6a8a85425c18e9db5635fcf3043645746975bf9e4dc834c78e3728` |
| 09 | 1x | a_09_1x.gif | 1 | 1000 | `9db2bb7bf9713dcb604b45213153da08dc778cdbaa02931fa5b95c3f23f9072f` |
| 10 | 1x | a_10_1x.gif | 8 | 1000 | `010a860fe7f533a9e639d0367ed05075646ab637f67bdc269f5c891016a804ca` |
| 11 | 1x | a_11_1x.gif | 8 | 1000 | `5f086c661a03156d32c92a7f823272f3b8c6e007f33d5cbd0642830755543fdc` |
| 12 | 1x | a_12_1x.gif | 8 | 1000 | `db46ac504dcf694565debb94608c964bd0658a41fd23f0206299677ca0ababb8` |
| 01 | 2x | a_01_2x.gif | 8 | 1000 | `05e21178cd035af0db126162d9c45ce4c635171b264d9dc25b128883d5f3da21` |
| 02 | 2x | a_02_2x.gif | 8 | 1000 | `1eae10f456dff7c136123b634acd8d584a356c683ce455ea484a7db12ba9f445` |
| 03 | 2x | a_03_2x.gif | 8 | 1000 | `7f45920e07e4b63cc81bfce3b3bbbae401788d1efff8ce93f750e7df5ff9f9f1` |
| 04 | 2x | a_04_2x.gif | 8 | 1000 | `f35bf406ec57794724fc0be20d97d9860292c75c27db830fc50618b3748c9802` |
| 05 | 2x | a_05_2x.gif | 1 | 1000 | `a49c8a346fe36a898b32fe52cdbcbe6037535943a7a1077f964fb9d3259c4cc7` |
| 06 | 2x | a_06_2x.gif | 8 | 1000 | `a335b57b6986e7b9096c7c712b1fd7e38a5d56fb1d0e6ff13f52189123138f6e` |
| 07 | 2x | a_07_2x.gif | 8 | 1000 | `17005486986803a94d1d06f9b66457dee07157858cd83019d51d036424ba35be` |
| 08 | 2x | a_08_2x.gif | 8 | 1000 | `a8b500e01adfd4531b4ca9ba7db1340bb792d216b2ea2cab3ff6a446f52aa04d` |
| 09 | 2x | a_09_2x.gif | 1 | 1000 | `a49c8a346fe36a898b32fe52cdbcbe6037535943a7a1077f964fb9d3259c4cc7` |
| 10 | 2x | a_10_2x.gif | 8 | 1000 | `adbb260246744df99572b014e76f8270aa2634ede9319e45e4f76ff89745b3b2` |
| 11 | 2x | a_11_2x.gif | 8 | 1000 | `922f451e56f51a3172cb075e4c29ef927f49883c4c73ad87c966fd4f55108918` |
| 12 | 2x | a_12_2x.gif | 8 | 1000 | `b589c1bfd7a6f73cf1161a86bd52f57db27e2a33a43cee26a16d903081d4d897` |

## Verification

All 16 distinct selected source layer sidecars pass the current per-layer checker. Both rendered boards were visually inspected: all12 paired slots and A/B labels are visible; no text or source cells are clipped. Source cells retain original GIF pixels. No Godot process was launched for board composition.

| Board | Size | Bytes | SHA256 |
|---|---|---:|---|
| compare_AB_1x.png | 786x816 | 277210 | `13162d1b1d5f141403b615a8ef9dc2a81add7df1676783df8203c7a091c8600e` |
| compare_AB_2x.png | 1332x1264 | 348993 | `b99ed7ac2a7af88090a0210e0aa0c59d3a960a6a850555ca4b0e8b61dbeb906a` |
