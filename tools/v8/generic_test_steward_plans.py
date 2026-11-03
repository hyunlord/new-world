"""Named gate37 production worlds; partial/lethal tests retain native assertions."""
from generic_gate import document, integer, require
from generic_test_endpoint_types import ExpectedOccurrence, bound_bytes, fields, snapshot
from generic_test_history_graph import HistoryEvidence, extends
from generic_test_plans import Planned
from mux2_gate_inputs import Json, Node

from generic_test_steward_scope import (STOCK, CANCEL, REPLAY, PRODUCTION, HOUSING, DEFENCE,
    IGNORED as IGNORED, SOURCE_PINS as SOURCE_PINS, TESTS as TESTS, EXTRA_SOURCE, EXTRA_HASH)

def steward_plan(evidence: HistoryEvidence, target: str) -> tuple[tuple[Planned,...],dict[str,Json]]:
    inputs=dict(evidence.inputs)
    if target == 'colony_steward':
        require(inputs.get(EXTRA_SOURCE)==EXTRA_HASH,EXTRA_SOURCE,'unregistered housing source')
        _=bound_bytes(evidence.input_root/EXTRA_SOURCE,EXTRA_HASH)
    hashes=dict(evidence.output_hashes)
    def read(test:str,seed:int,branch:str,file:str) -> Node:
        name=f'save_load_pid{evidence.pid}/{test}_s{seed}_{branch}/{file}.json'
        require(name in hashes,name,'missing planned steward evidence')
        return document(bound_bytes(evidence.root/name,hashes[name]).decode())
    def annotation(test:str,seed:int,branch:str,label:str,tick:int,detail:dict[str,Json]) -> None:
        node=read(test,seed,branch,'checkpoint_'+label);fields(node,'state detail')
        node.field('detail').equals(detail)
        state=snapshot(node.field('state'));require(state.tick==tick,label,'setup coordinate differs')
        start=snapshot(read(test,seed,branch,'initial').field('initial'))
        end=snapshot(read(test,seed,branch,'terminal').field('terminal'))
        extends(start,state,label);extends(state,end,label)
    rows:list[Planned]=[];setup:dict[str,Json]={}
    def add(test:str,seed:int,branch:str,start:int,end:int,stop:str,ordinary:bool=True) -> None:
        native='construction::'+test if test in HOUSING else test
        rows.append(Planned(ExpectedOccurrence(test,native,seed,branch,start,end,ordinary),stop))
    if target=='colony_steward':
        add(STOCK,0,'original',0,1,'native_assertions_reached')
        add(CANCEL,19,'original',0,721,'native_assertions_reached')
        add(CANCEL,19,'loaded',361,721,'360_ticks_after_load')
        add(REPLAY,29,'original',0,720,'native_assertions_reached')
        add(REPLAY,29,'replay',0,720,'720_ticks')
        annotation(REPLAY,29,'replay','replay_setup',0,{'intervention':'enable replay and inject original sourced commands'})
        add(PRODUCTION,0,'original',0,360,'native_assertions_reached')
        for test in HOUSING:
            first=read(test,29,'original','save_first_housed').field('state')
            tick=integer(first.field('tick'));require(0<tick<=14400,test,'housing completion outside source limit')
            read(test,29,'original','checkpoint_first_housed').equals(first.value)
            detail:dict[str,Json]={'classification':'controlled_full_production','terrain':test==HOUSING[0],
                'recipe':'all Position to hearth; 25x25 Plain; clear radius8 trees/nodes/material; optional two Hill tiles; rebuild nav'}
            annotation(test,29,'original','before_setup',0,detail)
            after=snapshot(read(test,29,'original','checkpoint_after_setup'))
            before=snapshot(read(test,29,'original','initial').field('initial'))
            require(after.tick==0 and after.alive_ids==before.alive_ids,test,'setup changed native time or founders')
            extends(before,after,test);extends(after,snapshot(first),test)
            setup[test]=detail
            add(test,29,'original',0,tick+1440,'first_housed_plus_1440')
            add(test,29,'replay',tick,tick+1440,'original_completion_tick')
            annotation(test,29,'replay','replay_setup',tick,{'intervention':'enable replay and inject original commands'})
            add(test,29,'food',tick,tick+1440,'scope_or_cooldown_assertions_reached')
            add(test,29,'cooldown',tick,tick+1800,'scope_or_cooldown_assertions_reached',False)
            annotation(test,29,'cooldown','before_cooldown',tick,{'classification':'mechanism','rule':3,'until':tick+1440})
            for n in range(4):
                add(test,29,f'manual_{n}',tick,tick,'load_only_manual_intent')
                add(test,29,f'restored_{n}',tick,tick+1440,'1440_ticks_after_manual_load')
    else:
        require(target=='steward_defence',target,'unregistered steward target')
        for branch in ('original','replay'):
            add(DEFENCE,0,branch,0,721,'initial_tick_plus_721')
            detail={'classification':'controlled_full_production','intervention':'tick once; arm adults with created spear; spawn four wolves pack42; enable steward'} if branch=='original' else {'intervention':'same arm and incoming setup as original; reset steward and clear log; replay exported bytes retained under original occurrence'}
            annotation(DEFENCE,0,branch,'before_setup',0,detail)
            armed=read(DEFENCE,0,branch,'checkpoint_before_arming');fields(armed,'state detail')
            armed.field('state').field('tick').equals(1)
            ids=armed.field('detail').field('adult_ids').array()
            require(bool(ids) and len({integer(value) for value in ids})==len(ids),'/arming','missing or duplicate armed adults')
            setup[branch]={'recipe':detail,'adult_ids':armed.field('detail').field('adult_ids').value}
        for branch,label in (('original','after_setup'),('replay','before_replay_reset'),('replay','after_replay_reset')):
            read(DEFENCE,0,branch,'checkpoint_'+label).field('tick').equals(1)
        # Retained export bytes are an original source oracle input, not a new replay run.
        name=f'save_load_pid{evidence.pid}/{DEFENCE}_s0_original/input_exported_replay.bin'
        require(name in hashes,name,'original exported replay bytes not retained')
        records=document('{"rows":'+bound_bytes(evidence.root/name,hashes[name]).decode()+'}').field('rows').array()
        require({record.field('phase').text() for record in records if 'phase' in record.mapping()}=={'after_assignment','before_flight'},'/replay','source phases missing')
    return tuple(rows),setup
