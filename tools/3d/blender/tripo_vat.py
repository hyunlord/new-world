# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: blender -b --python-exit-code 1 -P tools/3d/blender/tripo_vat.py -- woman
"""Bake all processed Tripo LODs using the existing living-character VAT pipeline.

Blender --background --python tools/3d/blender/tripo_vat.py -- [woman|goat] [0|1|2]
LOD0 retains the base id; crowd `_lod` aliases the LOD2 assets without rebaking.
"""
import json
import os
import shutil
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import chars_c as V
import chars_b as B
import common as C

FRAGMENT = os.path.join(C.MODELS_DIR, 'tripo_vat_manifest.json')


def build(subjects: list[str], levels: list[int]) -> None:
    """Bake requested processed LODs and preserve unrelated fragment entries."""
    source_commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=C.REPO, text=True).strip()
    entries = {}
    if os.path.exists(FRAGMENT):
        with open(FRAGMENT) as stream:
            entries = json.load(stream)['models']
    for subject in subjects:
        assert subject in ('woman', 'goat')
        for lod in levels:
            assert lod in (0, 1, 2)
            stem = f'tripo_{subject}_lod{lod}'
            cid = f'tripo_{subject}' + (f'_lod{lod}' if lod else '')
            V.build_one(cid, os.path.join(C.MODELS_DIR, stem + '.glb'), subject,
                        os.path.join(C.MODELS_DIR, stem + '_albedo.png'),
                        B.HUMAN if subject == 'woman' else B.ANIMAL,
                        f'models/{stem}.glb', entries, add_tool=False)
            entries[cid]['source_commit'] = source_commit
            meta_path = os.path.join(C.VAT_DIR, f'{cid}_vat.json')
            with open(meta_path) as stream:
                meta = json.load(stream)
            meta['source_commit'] = source_commit
            meta['source_provenance'] = 'Owner-supplied Tripo GLB; processed locally by tripo_build.py and tripo_rig.py'
            with open(meta_path, 'w') as stream:
                json.dump(meta, stream, indent=1)
                stream.write('\n')
            if lod == 2:
                alias = f'tripo_{subject}_lod'
                row = json.loads(json.dumps(entries[cid]))
                for suffix in ('static.glb', 'vat.bin', 'vat.json'):
                    shutil.copyfile(os.path.join(C.VAT_DIR, f'{cid}_{suffix}'),
                                    os.path.join(C.VAT_DIR, f'{alias}_{suffix}'))
                for field, suffix in (('static', 'static.glb'), ('bin', 'vat.bin'), ('json', 'vat.json')):
                    row['vat'][field] = f'vat/{alias}_{suffix}'
                entries[alias] = row
    with open(FRAGMENT, 'w') as stream:
        json.dump({'models': dict(sorted(entries.items()))}, stream, indent=1)
        stream.write('\n')


if __name__ == '__main__':
    args = C.script_args()
    build(args[:1] or ['woman', 'goat'], [int(args[1])] if len(args) > 1 else [0, 1, 2])
