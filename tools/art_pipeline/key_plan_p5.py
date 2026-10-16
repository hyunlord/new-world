# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow", "numpy"]
# ///
# How to run: python3 tools/art_pipeline/key_plan_p5.py
"""Freeze fill targets and key-sheet prompts without changing the target universe."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Final

from fill_catalog import description, load_catalog, prompt_for_fill
from wood_candidates_p4 import Cell, cells as wood_cells

ROOT: Final = Path(__file__).resolve().parents[2]
OUTPUT: Final = ROOT / 'docs/art/production5'
TARGETS: Final = {
    'V_a6': [*(f'V_a1_{i:02}' for i in range(5, 13)), 'V_a3_01', 'V_a3_02', 'V_a2_13', 'V_a2_14',
             'P2V_a_N_build_01', 'P2V_a_N_build_02', 'V_a2_09', 'V_a2_10'],
    'V_a5': [f'V_a4_{i:02}' for i in range(1, 17)],
    'V_b5': [f'V_b4_{i:02}' for i in range(1, 17)],
    'V_b6': ['V_b2_13', 'V_b2_14', 'P2V_b_N_build_01', 'P2V_b_N_build_02', 'V_b2_09', 'V_b2_10',
             'P2V_b_N_gather_01', 'P2V_b_N_gather_02', 'P2V_b_N_drink_01', 'P2V_b_N_drink_02',
             'V_b3_01', 'V_b3_02', 'V_b3_16', 'V_b1_01', 'V_b1_02', 'V_b1_04'],
    'V_c4': [*(f'P2V_c_S_carry_{i:02}' for i in range(1, 5)), 'P2V_c_S_gather_01', 'P2V_c_S_gather_02',
             'P2V_c_E_gather_01', 'P2V_c_E_gather_02', 'P2V_c_E_drink_01', 'P2V_c_E_drink_02',
             'P2V_c_E_build_01', 'P2V_c_E_build_02', 'V_c2_03', 'V_c2_04', 'V_c2_09', 'V_c2_10'],
}


def pose_description(cell: Cell) -> str:
    """Spell out the requested phase differences instead of repeating a pose."""
    facing = {'S': 'facing the viewer', 'N': 'facing away', 'E': 'facing right'}[cell['view']]
    if cell['activity'] == 'gather':
        pose = 'reaching into a berry bush' if cell['frame'] == 0 else 'picking berries from that bush'
        return f'{facing}, {pose}, no basket'
    if cell['activity'] == 'idle':
        pose = 'weight on the left foot' if cell['frame'] == 0 else 'weight on the right foot'
        return f'{facing}, standing idle, {pose}, relaxed empty hands'
    if cell['activity'] == 'carry' and cell['view'] == 'N':
        pose = 'left foot forward' if cell['frame'] == 0 else 'right foot forward'
        return f'{facing}, carrying a physical basket of berries, walking {pose}'
    return description(cell)


def prepare() -> Path:
    """Resolve all eighty cells against existing slots, retaining stable source context."""
    source = ROOT / 'docs/art/production2/final_selection.json'
    final = json.loads(source.read_text())
    atlas = json.loads((ROOT / 'assets/art/atlas/world_atlas.json').read_text())['sprites']
    sheets = []
    for sheet_id, targets in TARGETS.items():
        body = {'a': 'adult_a', 'b': 'adult_b', 'c': 'child'}[sheet_id[2]]
        rows = []
        for index, target in enumerate(targets, 1):
            entry = atlas.get(target)
            if entry is None and sheet_id in ('V_a5', 'V_b5'):
                entry = wood_cells(target[:4])[index - 1]
            if entry is None:
                message = f'Existing target missing: {target}'
                raise ValueError(message)
            rows.append({'cell': index, 'target_slot': target,
                         'name': f'{entry["activity"]}_{entry["view"]}_{entry["frame"]}',
                         'activity': entry['activity'], 'view': entry['view'], 'frame': entry['frame']})
        sheets.append({'id': sheet_id, 'body': body, 'derive_missing': False, 'key_hair': body != 'child',
                       'reference_candidate': final['references'][body], 'cells': rows})
    catalog = {'schema_version': 1, 'source_selection': str(source.relative_to(ROOT)),
               'source_selection_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
               'checkpoint_attempted_cells': 16, 'sheets': sheets}
    path = OUTPUT / 'fill_catalog.json'
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(catalog, indent=2) + '\n')
    load_catalog(path)
    for sheet in sheets:
        sheet_id, body = sheet['id'], sheet['body']
        if sheet_id in ('V_a5', 'V_b5'):
            prior = 'V_a4' if body == 'adult_a' else 'V_b4'
            prompt = (ROOT / f'docs/art/production4/wood/prompts/{prior}.txt').read_text().strip()
        else:
            prompt = prompt_for_fill(sheet)
            old_rows = ['Row ' + str(r + 1) + ': ' + '; '.join(description(c) for c in sheet['cells'][r * 4:r * 4 + 4]) + '.' for r in range(4)]
            new_rows = ['Row ' + str(r + 1) + ': ' + '; '.join(pose_description(c) for c in sheet['cells'][r * 4:r * 4 + 4]) + '.' for r in range(4)]
            prompt = prompt.replace(' '.join(old_rows), ' '.join(new_rows))
            prompt += f' Output 1254 × 1254. Image 1 is the D3 style reference. Image 2 is the {body} identity reference enlarged 8× nearest. Each figure holds its own physical prop; no floating icons.'
        if sheet['key_hair']:
            prompt += ' The hair is painted only in flat teal #00DCD2, with #00AAA2 and #007D76 for darker hair shading, exactly as in Image 2. Teal appears nowhere else: not on skin, eyebrows, clothing, props, outline or background.'
        prompt += ' In the four walking cells of a row the head stays at the same height and the load stays in the same place.'
        folder = OUTPUT / 'prompts'
        folder.mkdir(exist_ok=True)
        (folder / f'{sheet_id}.txt').write_text(prompt + '\n')
    return path


if __name__ == '__main__':
    print(prepare())
