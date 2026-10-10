"""Explicit physical fill cells and stable runtime targets; original catalog stays intact."""
from __future__ import annotations
import json
from pathlib import Path
from production_sheets import CellSpec


def load_catalog(path: Path) -> dict:
    catalog=json.loads(path.read_text())
    if catalog.get('schema_version')!=1:raise ValueError('Unknown fill catalog schema')
    ids=set()
    for sheet in catalog['sheets']:
        if sheet['id'] in ids:raise ValueError('Duplicate fill sheet')
        ids.add(sheet['id'])
        if len(sheet['cells'])!=16:raise ValueError('Every fill sheet must have sixteen cells')
        if [cell['cell'] for cell in sheet['cells']]!=list(range(1,17)):raise ValueError('Physical cells must be row-major1..16')
        if len({cell['target_slot'] for cell in sheet['cells']})!=16:raise ValueError('Fill targets must be unique within a sheet')
        if sheet['body'] not in ('','adult_a','adult_b','child'):raise ValueError('Unknown body')
        for cell in sheet['cells']:
            if not sheet['body'] and not cell.get('budget_alias'):raise ValueError('Objects require canonical budget aliases')
    return catalog


def review_definitions(catalog: dict) -> dict[str,list[CellSpec]]:
    return {sheet['id']:[CellSpec(cell['name'],(1,1),cell['view'],cell['activity'],cell['frame'])
                          for cell in sheet['cells']] for sheet in catalog['sheets']}


def description(cell: dict) -> str:
    activity=cell['activity'];frame=cell['frame']
    if not cell['view']:return cell.get('description',cell['name'].replace('_',' '))
    facing={'S':'facing the viewer','N':'facing away','E':'facing right','W':'facing left'}[cell['view']]
    phases=('left foot forward','feet passing','right foot forward','feet passing')
    poses={'idle':'standing at rest, relaxed arms', 'gather':'bending and reaching toward a berry bush with an empty hand, no basket',
           'build':'stone hammer raised to strike' if frame%2==0 else 'striking with a stone hammer',
           'drink':'kneeling with cupped hands brought to the mouth', 'eat':'bringing berries to the mouth with one hand',
           'sit':'sitting on the ground with knees up', 'talk':'talking with a hand gesture',
           'asleep':'lying horizontally asleep on a fur bedroll', 'crouch':'crouching to tend a fire',
           'crossed':'standing with arms crossed','hug':'sitting and hugging knees','hop':'hopping in place'}
    pose=poses.get(activity,activity)
    if activity in ('walk','carry','run'):
        pose=f'{activity}, {phases[frame%4]}'
    if activity=='carry':pose+=', holding a basket of berries at the chest'
    elif activity not in ('eat',):pose+=', no basket'
    return f'{facing}, {pose}'


def prompt_for_fill(sheet: dict) -> str:
    frame=('A 4 × 4 grid sprite sheet on a solid flat magenta (#FF00FF) background. '
           'Every item is centred in its own equal square cell with at least 22 % empty margin on every side; '
           'nothing touches a cell edge or another item; no shadows on the background; no text, numbers or labels. '
           'Same pixel-art style, palette and lighting as the reference image. Items, left to right, top to bottom: ')
    rows=['Row '+str(row+1)+': '+'; '.join(description(c) for c in sheet['cells'][row*4:row*4+4])+'.' for row in range(4)]
    identity=(' All sixteen cells show the same person as the reference image, same hair, face and clothing. '
              'Same body proportions, clothes, colours and size in every cell; only the pose changes. ') if sheet['body'] else ' Objects only, no people. '
    style=('Flat, bold pixel art with strong silhouettes and minimal texture; large simple shapes that stay readable at small size; '
           'a dark outline around every figure and object; light from the upper left; the facing direction and pose specified in each cell; '
           'Stone Age materials, simple hide and fur clothing with warm skin tones for people; crisp square pixels, no anti-aliasing, no blur, no gradients.')
    return frame+' '.join(rows)+' Exactly four rows and four columns, sixteen cells total. Do not add extra rows, columns, pose panels or conceptual panels.'+identity+style
