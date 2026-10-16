"""A27 checks derive expected villages from raw FFI rather than UI summaries."""
import re

PALETTE = ('E0B060','7FB0D8','B88AD0','8CC07A')


def check_village_frame(data, remembered) -> list[str]:
    """Verify labels, lineage, palette and selected membership for one frame."""
    errors: list[str] = []
    raw = data.get('society_snapshot',{})
    settlement = data.get('settlement_snapshot',{})
    present = data.get('society_presentation',{})
    ids = raw.get('settlement_ids',[])
    if ids != settlement.get('settlement_ids'):
        errors.append('society and settlement snapshot alive IDs differ')
    rows = present.get('settlements',[])
    alive = {row['id']:row for row in rows if row.get('alive')}
    if set(alive) != set(ids):
        errors.append('UI alive IDs differ from raw snapshot')
    all_numbers = {row['id']:row['number'] for row in rows}
    if len(set(all_numbers.values())) != len(all_numbers):
        errors.append('village numbers are not unique')
    for ident,number in all_numbers.items():
        if ident in remembered and remembered[ident] != number:
            errors.append(f'village {ident} renumbered')
        remembered[ident] = number
    if sorted(all_numbers.values()) != list(range(1,len(all_numbers)+1)):
        errors.append('ever-seen numbering is not contiguous from one')
    camera,zoom,viewport = (data.get(key,[]) for key in ('camera_position','zoom','viewport_size'))
    scale = data.get('framebuffer_scale',1)
    transform = len(camera)==len(zoom)==len(viewport)==2
    if not transform:
        errors.append('missing view transform')
    homes = dict(zip(raw.get('home_agent_ids',[]),raw.get('home_settlement_ids',[])))
    walking = raw.get('founder_agent_ids',[])
    for index,ident in enumerate(ids):
        row = alive.get(ident,{})
        parent = raw['parent_ids'][index]
        number = all_numbers.get(ident,-1)
        wanted_color = PALETTE[(number-1)%len(PALETTE)]
        if row.get('parent_id') != parent or row.get('color','').upper() != wanted_color:
            errors.append(f'village {ident} parent/color wrong')
        expected_count = raw['member_counts'][index]-sum(homes.get(agent)==ident for agent in walking)
        if row.get('member_count') != expected_count:
            errors.append(f'village {ident} displayed members wrong')
        j = settlement.get('settlement_ids',[]).index(ident)
        tile = [settlement['formation_xs'][j],settlement['formation_ys'][j]]
        in_view = False
        if transform:
            px = ((448+tile[0]*16+8-camera[0])*zoom[0]+viewport[0]/scale/2)*scale
            py = ((28+tile[1]*16+8-camera[1])*zoom[1]+viewport[1]/scale/2)*scale
            in_view = 0<=px<viewport[0] and 0<=py<viewport[1]
        matches = [label for label in present.get('labels',[]) if label.get('id')==ident and not label.get('abandoned')]
        if in_view and (len(matches)!=1 or not matches[0].get('visible')):
            errors.append(f'village {ident} formation in view lacks one visible label')
        for label in matches:
            if label.get('tile') != tile or label.get('number') != number or not 9<=label.get('screen_font_px',0)<=12:
                errors.append(f'village {ident} label tile/number/font mismatch')
            text = label.get('text','')
            if parent >= 0:
                pn = all_numbers.get(parent,-1)
                if label.get('parent_number') != pn or not (f'{number}번 마을 · {pn}번에서' in text or f'Village {number} · from {pn}' in text):
                    errors.append(f'village {ident} daughter label missing actual parent')
            elif not (f'{number}번 마을' in text or f'Village {number}' in text):
                errors.append(f'village {ident} name missing')
    hud = [row.get('text','') for row in data.get('labels',[]) if 'HudTopbar' in row.get('name','')]
    village_labels = [text for text in hud if re.fullmatch(r'(?:마을|Villages)\s+\d+',text)]
    if len(ids)>=2 and not any(re.fullmatch(rf'(?:마을|Villages)\s+{len(ids)}',text) for text in village_labels):
        errors.append('HUD alive village count missing/wrong')
    if len(ids)<2 and village_labels:
        errors.append('HUD village count shown for single village')
    discs = present.get('overview_discs',[])
    if present.get('overview_visible'):
        if {r.get('id') for r in discs} != set(ids) or len({r.get('color') for r in discs}) != len(ids):
            errors.append('overview disc IDs/colors not distinct for alive villages')
    selected = data.get('selected_settlement_id',-1)
    for agent in data.get('entities',[]):
        if agent.get('kind') != 'agent' or not agent.get('visible'):
            continue
        expected = selected>=0 and homes.get(agent.get('id'))==selected
        if bool(agent.get('settlement_outline',False)) != expected:
            errors.append(f'agent {agent.get("id")} selection outline mismatch')
    return errors
