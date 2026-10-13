"""A28 checks use raw founders/logs plus actual drawn geometry and visibility."""
from collections import Counter


def check_group(group, raw_agents=None, entities=None) -> list[str]:
    """Global draw submission does not establish that a site is visible."""
    errors: list[str] = []
    if not group.get('trail_visible') or not group.get('marker_visible'):
        errors.append('trail/marker not drawn')
    if not group.get('marker_in_view') or not group.get('site_in_view'):
        errors.append('site marker is outside view')
    if (group.get('dash_px') != 2 or group.get('gap_px') != 3 or
            abs(group.get('trail_alpha',0)-.6) > .001 or group.get('trail_color','').upper() != 'E8D8A8'):
        errors.append('trail style mismatch')
    for bundle in group.get('bundles',[]):
        ident = bundle.get('agent_id')
        entity = (entities or {}).get(ident,{})
        shown = entity.get('visible',bundle.get('agent_visible',True))
        if not bundle.get('agent_in_view') or not shown:
            continue
        suppressed = bundle.get('suppressed','')
        if bundle.get('visible'):
            if suppressed:
                errors.append(f'visible founder {ident} claims both bundle and suppression')
            continue
        raw = (raw_agents or {}).get(ident,{})
        valid = False
        if suppressed == 'real_load':
            valid = raw.get('carried_food',0)>0 or raw.get('carried_wood',0)>0
        elif suppressed in ('eat','asleep'):
            target = 1 if suppressed=='eat' else 3
            valid = raw.get('state')==3 and raw.get('target')==target and suppressed in (
                entity.get('activity'),entity.get('cold_base_pose'))
        if not valid:
            errors.append(f'visible founder {ident} bundle suppression {suppressed!r} unsupported by raw state/load')
    return errors


def check_founding_frame(data) -> list[str]:
    """Compare endpoints, members and one feed event per raw fission log entry."""
    errors: list[str] = []
    raw = data.get('society_snapshot',{})
    presentation = data.get('society_presentation',{})
    groups = presentation.get('founding_groups',[])
    founders = raw.get('founder_agent_ids',[])
    fids = raw.get('founder_fission_ids',[])
    leader_flags = raw.get('founder_leader',[])
    expected_groups = set(fids)
    if {group.get('fission_id') for group in groups} != expected_groups:
        errors.append('drawn groups differ from Founding components (or remain after founding)')
    agents = data.get('agent_snapshot',{})
    positions = dict(zip(agents.get('agent_ids',[]),zip(agents.get('xs',[]),agents.get('ys',[]))))
    raw_agents = {ident:{'state':state,'target':target,'carried_food':food,'carried_wood':wood}
        for ident,state,target,food,wood in zip(agents.get('agent_ids',[]),agents.get('states',[]),
            agents.get('target_kinds',[]),agents.get('carried_foods',[]),agents.get('carried_woods',[]))}
    entities = {row.get('id'):row for row in data.get('entities',[]) if row.get('kind')=='agent'}
    for group in groups:
        errors.extend(check_group(group,raw_agents,entities))
        fid = group.get('fission_id')
        members = [agent for agent,fission in zip(founders,fids) if fission == fid]
        leaders = [agent for agent,fission,flag in zip(founders,fids,leader_flags) if fission == fid and flag == 1]
        if len(leaders) != 1 or group.get('leader_id') != leaders[0]:
            errors.append('leader differs from raw founder_leader flags')
        if set(group.get('founder_ids',[])) != set(members):
            errors.append('founder group members mismatch')
        if tuple(group.get('start_tile',[])) != positions.get(group.get('leader_id')):
            errors.append('trail does not start at leader tile')
        index = fids.index(fid) if fid in fids else -1
        sites_x,sites_y = raw.get('founder_site_xs',[]),raw.get('founder_site_ys',[])
        if index < 0 or group.get('site_tile') != [sites_x[index],sites_y[index]]:
            errors.append('trail does not end at raw founding site')
        visible = {row.get('id') for row in data.get('entities',[]) if row.get('kind') == 'agent' and row.get('visible')}
        bundles = {row.get('agent_id') for row in group.get('bundles',[])}
        if not set(members).intersection(visible).issubset(bundles):
            errors.append('visible founder lacks bundle observation')
    numbers = {r['id']:r['number'] for r in presentation.get('settlements',[])}
    feed = data.get('feed_log',[])
    founded = 0
    cancelled = 0
    for index,fid in enumerate(raw.get('fission_ids',[])):
        for suffix,key,tick_key,args in [
            (f'depart:{fid}','HUD_FEED_FISSION_DEPARTED','fission_depart_ticks',[raw['fission_sizes'][index],numbers.get(raw['fission_parents'][index])]),
            (f'cancel:{fid}','HUD_FEED_FISSION_CANCELLED','fission_cancelled_ticks',[]),
            (f'founded:{raw["fission_daughters"][index]}','HUD_FEED_SETTLEMENT_FOUNDED','fission_founded_ticks',[numbers.get(raw['fission_daughters'][index])])]:
            tick = raw[tick_key][index]
            if tick < 0:
                continue
            matched = [event for event in feed if event.get('society_event_id') == suffix]
            if len(matched) != 1 or matched[0].get('key') != key or matched[0].get('args') != args or matched[0].get('tick') != tick:
                errors.append(f'fission feed missing/duplicated/mismatched {suffix}')
        founded += int(raw['fission_founded_ticks'][index] >= 0)
        cancelled += int(raw['fission_cancelled_ticks'][index] >= 0)
    tallies = data.get('tallies',{})
    if tallies.get('fissions') != founded or tallies.get('fissions_cancelled') != cancelled:
        errors.append('fission log and tallies disagree')
    events = [event.get('society_event_id') for event in feed if 'society_event_id' in event]
    if any(count != 1 for count in Counter(events).values()):
        errors.append('repeated society feed identity')
    return errors
