# A4 tool reuse for B4

`assets.py`, `sheets.py` and `test_assets.py` are copied byte-for-byte from `386248ffbab91993960145f59fbc3636890cea63` (A4). These existing tools were not present in the ART-TREES base. The source module names and A4 sheet header are retained deliberately; B4 output paths and captions identify their content. Images are generated separately; these tools only export/measure images and compose review boards.

Verification: `python3 -m pytest art/techmap/test_assets.py -q`.
