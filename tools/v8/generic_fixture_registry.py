"""Strict, immutable gate plans. Expansion is planning, never execution or qualification."""
import hashlib
import json
import re
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import TypeAlias

from generic_gate import decode_json

Json: TypeAlias = None | bool | int | float | str | list['Json'] | dict[str, 'Json']
SUPPORTED = frozenset({'baseline_sidecar', 'colony_sidecar', 'f10_boundary_sidecar'})
DIRECT_IDS = frozenset(['identity.baseline_samples', 'identity.baseline_twins', 'determinism.baseline', 'colony.u10', 'core.colony_u10', 'read.determinism', 'replay.generate', 'replay.recorded', 'replay.scripted', 'economy.soak_no_beasts', 'story.script48', 'neighbours.script48', 'read.story48', 'read.neighbours48', 'steward.on24_twins', 'read.steward_on24', 'read.m6a_steward_on72', 'read.soak20', 'core.colony_soak20', 'core.baseline10', 'core.steward_off', 'm6a.lifecycle_off', 'core.steward_on', 'm6a.lifecycle_on', 'knowledge.f10_injected', 'knowledge.f10_no_kill', 'm6a.food72', 'm6a.food_data_only'])
BACKLOG_IDS = frozenset(['perf.natural_quick', 'bench.timed_rss', 'nav.soak', 'ui.smoke', 'ui.play', 'renderer.agreement', 'render_and_pack', 'visibility', 'why_toast_layout', 'stall', 'save_roundtrip', 'named_integration_boundaries'])
PLACEHOLDERS = frozenset({'probe','inputs','out','harness_root','seed','commands_out','chronicle_out','replay_generate_same_arm_commands','f10_companion'})

class RegistryError(ValueError):
    """Malformed or incomplete registry; cannot admit a gate result."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RegistryError(message)


def obj(value: Json) -> dict[str, Json]:
    require(isinstance(value, dict), 'expected object')
    assert isinstance(value, dict)
    return value


def array(value: Json) -> list[Json]:
    require(isinstance(value, list), 'expected array')
    assert isinstance(value, list)
    return value


def text(value: Json) -> str:
    require(isinstance(value, str) and bool(value), 'expected nonempty string')
    assert isinstance(value, str)
    return value


def strings(value: Json) -> tuple[str, ...]:
    return tuple(text(item) for item in array(value))


def integer(value: Json) -> int:
    require(type(value) is int and value >= 0, 'expected nonnegative integer')
    assert isinstance(value, int)
    return value


def canonical(value: Json) -> str:
    return json.dumps(value, sort_keys=True, separators=(',', ':'), allow_nan=False)


def unique_object(pairs: list[tuple[str, Json]]) -> dict[str, Json]:
    result: dict[str, Json] = {}
    for key, value in pairs:
        require(key not in result, f'duplicate JSON key: {key}')
        result[key] = value
    return result


def reject_constant(value: str) -> None:
    raise RegistryError(f'non-finite JSON constant: {value}')


def relative(name: str) -> str:
    path = PurePosixPath(name)
    require(bool(name) and not path.is_absolute() and '..' not in path.parts and path.as_posix() == name and name != '.', 'unsafe relative path')
    return name


@dataclass(frozen=True, slots=True)
class Fixture:
    id: str
    argv: tuple[str, ...]
    seeds: tuple[int, ...]
    repeats: tuple[str, ...]
    roles: tuple[str, ...]
    adapter: str
    metadata_json: str


@dataclass(frozen=True, slots=True)
class Registry:
    sha256: str
    fixtures: tuple[Fixture, ...]
    backlog: tuple[str, ...]
    metadata_json: str

    def require_complete(self) -> None:
        """Fail closed: this first registry deliberately retains unresolved producers."""
        require(not self.backlog, 'unresolved producer families: ' + ', '.join(text(obj(decode_json(row))['id']) for row in self.backlog))
        require(all(obj(decode_json(row.metadata_json))['adapter_status'] == 'QUALIFIED' for row in self.fixtures), 'unqualified direct adapters')


# Independent standing workload contract, transcribed from the Stage 0 scope.
# Do not derive these constraints from gate_fixtures.json: it is the input being checked.
# New IDs are additive. Revising a required row needs an explicit contract change;
# a semantic-output declaration cannot shorten its workload or remove a consumer.
REQUIRED_WORKLOADS = '''
identity.baseline_samples|game_v26|0..32|120|6|--clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30 --sample-every 120 --chronicle-out {chronicle_out}|10
identity.baseline_twins|game_v26|0..4|120|4|--check-determinism|10
determinism.baseline|baseline|0..4|120|4|--check-determinism|11
colony.u10|colony|0..4|10|4|--commands {inputs}/tools/v8/u_script.json --check-determinism|18
core.colony_u10|colony|0..4|10|4|--commands {inputs}/tools/v8/u_script.json --check-determinism|44
read.determinism|colony|0..4|10|4|--commands {inputs}/tools/v8/u_script.json --check-determinism|50
replay.generate|colony|5..7|3|2|--commands {inputs}/tools/v8/u_script.json --commands-out {commands_out}|19
replay.recorded|colony|5..6|3|1|--commands {replay_generate_same_arm_commands}/colony_s5_commands.json|19
replay.scripted|colony|5..6|3|1|--commands {inputs}/tools/v8/u_script.json|19
economy.soak_no_beasts|colony|0..4|20|4|--commands {inputs}/tools/v8/soak_script.json --story no-beasts --check-determinism|26
story.script48|colony|0..2|48|2|--commands {inputs}/tools/v8/m5_u_script.json|29
neighbours.script48|colony|0..2|48|2|--commands {inputs}/tools/v8/m55_u_script.json|30
read.story48|colony|0..2|48|2|--commands {inputs}/tools/v8/m5_u_script.json|50
read.neighbours48|colony|0..2|48|2|--commands {inputs}/tools/v8/m55_u_script.json|50
steward.on24_twins|colony|0..2|24|2|--steward on --check-determinism|35
read.steward_on24|colony|0..2|24|2|--steward on|50
read.m6a_steward_on72|colony|0..2|72|2|--steward on|50
read.soak20|colony|0..4|20|4|--commands {inputs}/tools/v8/soak_script.json|50
core.colony_soak20|colony|0..2|20|2|--commands {inputs}/tools/v8/soak_script.json|44
core.baseline10|game_v26|0..4|10|4||44
core.steward_off|colony|0..1|20|1|--steward off --story on --neighbours on|generic retained CORE ordinary row
core.steward_on|colony|0..1|20|1|--steward on --story on --neighbours on|generic retained CORE ordinary row
m6a.lifecycle_off|colony|0..1|20|1|--steward off --defence on --story on --neighbours on --check-determinism|retained M6a lifecycle fixture
m6a.lifecycle_on|colony|0..1|20|1|--steward on --defence on --story on --neighbours on --check-determinism|retained M6a lifecycle fixture
knowledge.f10_injected|colony|0..8|4|4|--commands {inputs}/tools/v8/m4_f10_script.json --commands-out {commands_out}|28
knowledge.f10_no_kill|colony|0..8|4|4|--commands {f10_companion}|Stage0 required companion
'''.strip()


def enforce_required_workload(row: dict[str, Json], flags: dict[str, str]) -> None:
    """Retain every standing workload independently of self-consistent registry metadata."""
    name = text(row['id'])
    if name not in DIRECT_IDS:
        return
    if name.startswith('m6a.food'):
        food72 = name == 'm6a.food72'
        require(strings(row['argv']) == (('{probe}','m6a','--seed','{seed}','--days','72','--out','{out}') if food72 else ('{probe}','m6a-data-only','--out','{out}')), f'{name}: required workload changed')
        require(array(row['seeds']) == ([0,1] if food72 else [0]), f'{name}: required seeds changed')
        require(array(row['consumers']) == ([47] if food72 else [48]), f'{name}: required consumer changed')
        require(row['adapter'] == ('m6a_runner_safety' if food72 else 'm6a_data_only_safety'), f'{name}: required adapter changed')
        require(row['role'] == ('ordinary_observed' if food72 else 'ordinary_selected_pack'), f'{name}: required role changed')
        require(row['producer'] == ('worldsim_probe m6a' if food72 else 'worldsim_probe m6a-data-only'), f'{name}: required producer changed')
        require(array(row['input_paths']) == ([] if food72 else ['rust/crates/sim-test/data/m6a_apple/pack.ron','rust/crates/sim-test/data/m6a_apple/items.ron','rust/crates/sim-test/data/m6a_apple/plants.ron']), f'{name}: required input changed')
        require(row['comparison'] == ('strict_normalised_or_declared_plus_safety' if food72 else 'retain chain/ledger oracle plus paired safety at equal actual horizon'), f'{name}: comparison weakened')
        if food72:
            require(integer(obj(row['horizon']).get('ticks_per_day',1440)) == 1440, 'required tick scale changed')
            require({'m6a_score fixture ledger and food invariants','complete source-bound per-seed outputs'} <= set(strings(row.get('retain_oracles',[]))), 'required food oracles omitted')
        if not food72:
            require(obj(row['horizon']) == {'kind':'event_conditioned','end':'chain complete or six calendar days','actual_end_tick':'required per arm; matched-horizon safety companion required'}, 'required event horizon changed')
        return
    contract = next(line.split('|') for line in REQUIRED_WORKLOADS.splitlines() if line.startswith(name + '|'))
    _, scene, seeds, days, workers, extra, consumer = contract
    expected = {'--scene':scene,'--seeds':seeds,'--days':days,'--workers':workers,'--out':'{out}'}
    tokens = iter(extra.split())
    for flag in tokens:
        expected[flag] = '' if flag == '--check-determinism' else next(tokens)
    require(flags == expected, f'{name}: required workload changed')
    require(array(row['consumers']) == [int(consumer) if consumer.isdecimal() else consumer], f'{name}: required consumer changed')
    require(integer(obj(row['horizon'])['ticks_per_day']) == 1440, f'{name}: required tick scale changed')
    require(row['role'] == ('explicit_fault_control' if name == 'knowledge.f10_injected' else 'ordinary'), f'{name}: required role changed')
    baseline = scene in {'baseline','game_v26'}
    require(row['producer'] == 'worldsim_probe', f'{name}: required producer changed')
    require(row['adapter'] == ('baseline_sidecar' if baseline else 'f10_boundary_sidecar' if name == 'knowledge.f10_injected' else 'colony_sidecar'), f'{name}: required adapter changed')
    expected_inputs = [expected['--commands'][len('{inputs}/'):]] if expected.get('--commands','').startswith('{inputs}/') else ['tools/v8/gate_f10_no_kill.json'] if name == 'knowledge.f10_no_kill' else []
    require(array(row['input_paths']) == expected_inputs, f'{name}: required inputs changed')
    require(row['comparison'] == ('baseline_identity' if baseline else 'retain exact negative-control oracle; ordinary safety through pre-injection checkpoint' if name == 'knowledge.f10_injected' else 'strict_normalised_or_declared_plus_safety'), f'{name}: comparison weakened')
    expected_oracles = {'exit0','exact seed/horizon/instance coverage'}
    if '--check-determinism' in expected:
        expected_oracles.add('determinism PASS and equal twin trajectories')
    if name == 'economy.soak_no_beasts':
        expected_oracles.update({'ledger/reservations0','all death causes0','m4.lost present empty','workshops>=2; crafts>=1; trees_felled>=1'})
    if name in {'story.script48','read.story48'}:
        expected_oracles.add('existing m5_score.py --story assertions where this is the standing scored consumer')
    if name in {'neighbours.script48','read.neighbours48'}:
        expected_oracles.add('existing m55_score.py --neighbours assertions where this is the standing scored consumer')
    if name == 'steward.on24_twins':
        expected_oracles.add('ledger/reservations0')
    if name == 'replay.recorded':
        expected_oracles.add('byte-identical summary to replay.scripted in same arm')
    require(expected_oracles <= set(strings(row.get('retain_oracles',[]))), f'{name}: required oracle omitted')


def parse_fixture(raw: Json) -> Fixture:
    row = obj(raw)
    required = {'id','consumers','producer','cwd','arms','argv','seeds','horizon','role','input_paths','adapter','adapter_status','comparison'}
    optional = {'instance_axes','retain_oracles','note','effective_defaults','generated_input','injection','known_conflict','environment'}
    require(required <= row.keys() <= required | optional, 'unknown or missing fixture keys')
    name = text(row['id'])
    require(re.fullmatch(r'[a-z][a-z0-9_]*(?:[.][a-z0-9_]+)+', name) is not None, 'invalid fixture ID')
    require(strings(row['arms']) == ('base','candidate'), 'both arms required in fixed order')
    require(row['cwd'] == '{harness_root}', 'unexpected working directory')
    consumers = array(row['consumers'])
    require(bool(consumers) and all(type(x) is int or isinstance(x, str) and bool(x) for x in consumers), 'invalid consumers')
    argv = strings(row['argv'])
    require(argv[0] == '{probe}', 'unsupported executable placeholder')
    placeholders = {match.group(1) for arg in argv for match in re.finditer(r'\{([^{}]+)\}', arg)}
    require(placeholders <= PLACEHOLDERS, 'unknown argv placeholder')
    require(all('{' not in re.sub(r'\{[^{}]+\}', '', arg) and '}' not in re.sub(r'\{[^{}]+\}', '', arg) for arg in argv), 'malformed placeholder')
    seeds = tuple(integer(seed) for seed in array(row['seeds']))
    require(bool(seeds) and tuple(sorted(set(seeds))) == seeds, 'duplicate/unsorted/empty seeds')
    axes = obj(row.get('instance_axes', {'invocation_repeat':[1], 'determinism_instance':['primary']}))
    require(set(axes) == {'invocation_repeat','determinism_instance'}, 'unknown occurrence axes')
    repeats = tuple(str(x) for x in array(axes['invocation_repeat']))
    require(bool(repeats) and len(set(repeats)) == len(repeats) and all(re.fullmatch('[A-Za-z0-9_-]+', repeat) for repeat in repeats), 'invalid invocation repeats')
    if name in DIRECT_IDS:
        require(repeats == (('a','b') if name.startswith('m6a.lifecycle_') else ('1',)), 'required invocation repeats changed')
    roles = strings(axes['determinism_instance'])
    require(roles == (('primary','twin') if '--check-determinism' in argv else ('primary',)), 'twin coverage mismatch')
    adapter = text(row['adapter'])
    require(adapter in SUPPORTED | {'m6a_runner_safety','m6a_data_only_safety'}, 'unknown adapter')
    require(row['adapter_status'] in {'IMPLEMENTED_UNQUALIFIED','REQUIRED_NOT_IMPLEMENTED'}, 'unqualified registry cannot assert qualification')
    flags: dict[str, str] = {}
    index = 1 if adapter in SUPPORTED else 2
    while index < len(argv):
        flag = argv[index]
        require(flag.startswith('--') and flag not in flags, 'duplicate/nonflag argument')
        if flag == '--check-determinism':
            flags[flag] = ''
            index += 1
        else:
            require(index + 1 < len(argv) and not argv[index+1].startswith('--'), 'missing flag value')
            flags[flag] = argv[index+1]
            index += 2
    allowed_flags = {'--scene','--seeds','--days','--workers','--clock-samples','--sample-every','--chronicle-out','--out','--check-determinism','--commands','--commands-out','--story','--neighbours','--steward','--defence'} if adapter in SUPPORTED else {'--seed','--days','--out'}
    require(flags.keys() <= allowed_flags, 'unknown probe flag')
    require(flags.get('--out') == '{out}', 'exclusive output required')
    require(flags.get('--chronicle-out','{chronicle_out}') == '{chronicle_out}' and flags.get('--commands-out','{commands_out}') == '{commands_out}', 'shared auxiliary output')
    require(row['role'] in {'ordinary','ordinary_observed','ordinary_selected_pack','explicit_fault_control'}, 'unknown observation role')
    require((row['role'] == 'explicit_fault_control') == (name == 'knowledge.f10_injected'), 'fault role cannot waive ordinary fixtures')
    for flag in ('--story','--neighbours','--steward','--defence'):
        if flag in flags:
            require(flags[flag] in ({'on','off','no-beasts'} if flag == '--story' else {'on','off'}), 'invalid mode value')
    environment = obj(row.get('environment',{}))
    expected_env = {'WORLDSIM_CONTENT_PACK':'{inputs}/rust/crates/sim-test/data/m6a_apple/pack.ron'} if name == 'm6a.food_data_only' else {}
    require(environment == expected_env, 'unexpected environment/input binding')
    horizon = obj(row['horizon'])
    require(horizon.keys() <= {'kind','days','ticks_per_day','completed_ticks','clock_offset','end','actual_end_tick'}, 'unknown horizon key')
    if horizon['kind'] == 'requested_days':
        days = integer(horizon['days'])
        require(days > 0 and flags.get('--days') == str(days), 'days mismatch')
        require(integer(horizon['completed_ticks']) == days * integer(horizon.get('ticks_per_day',1440)), 'horizon tick mismatch')
    else:
        require(name == 'm6a.food_data_only' and horizon['kind'] == 'event_conditioned', 'unknown horizon')
    if adapter in SUPPORTED:
        require(flags.get('--scene') in {'colony','baseline','game_v26'}, 'unknown scene')
        require(flags.get('--seeds') == f'{seeds[0]}..{seeds[-1]+1}' and seeds == tuple(range(seeds[0],seeds[-1]+1)), 'seed argv mismatch')
        require(flags.get('--workers','').isdigit() and 1 <= int(flags['--workers']) <= 8, 'invalid workers')
    paths = strings(row['input_paths'])
    require(len(set(paths)) == len(paths), 'duplicate input path')
    for path in paths:
        _ = relative(path)
    if '--commands' in flags and flags['--commands'].startswith('{inputs}/'):
        require(flags['--commands'][len('{inputs}/'):] in paths, 'unregistered command input')
    if '--commands' in flags:
        require(flags['--commands'].startswith('{inputs}/') or name in {'replay.recorded','knowledge.f10_no_kill'}, 'unknown command input provenance')
    generated = obj(row.get('generated_input', {}))
    if name == 'replay.recorded':
        require(generated.get('producer') == 'replay.generate' and generated.get('arm') == 'same' and generated.get('file') == 'colony_s5_commands.json', 'invalid replay dependency')
        require(flags.get('--commands') == '{replay_generate_same_arm_commands}/colony_s5_commands.json', 'replay input mismatch')
    elif name == 'knowledge.f10_no_kill':
        require(generated.get('source') == 'tools/v8/m4_f10_script.json' and flags.get('--commands') == '{f10_companion}', 'invalid companion dependency')
    else:
        require(not generated, 'unexpected generated input')
    enforce_required_workload(row, flags)
    return Fixture(name, argv, seeds, repeats, tuple('determinism_twin' if r == 'twin' else r for r in roles), adapter, canonical(row))


def load_registry(path: Path) -> Registry:
    """Parse duplicate-free JSON into immutable plans and preserve all source metadata."""
    data = path.read_bytes()
    try:
        raw = obj(decode_json(data, pairs_hook=unique_object, constant_hook=reject_constant))
        require(set(raw) == {'schema','status','inspected_source_sha','BASE0','published','scope','placeholders','common_contract','inspected_input_sha256','direct_fixture_drafts','adapter_backlog'}, 'unknown/missing registry key')
        require(raw['schema'] == 'worldsim_gate_fixtures_v1' and raw['status'] == 'INCOMPLETE_ADAPTERS', 'unsupported registry schema/status')
        require(raw['published'] is True and re.fullmatch('[a-f0-9]{40}',text(raw['BASE0'])) is not None, 'BASE0 not bound')
        fixtures = tuple(parse_fixture(row) for row in array(raw['direct_fixture_drafts']))
        require(len({f.id for f in fixtures}) == len(fixtures) and DIRECT_IDS <= {f.id for f in fixtures}, 'missing/duplicate fixture coverage')
        backlog = tuple(canonical(row) for row in array(raw['adapter_backlog']))
        require(len(backlog) == len(BACKLOG_IDS) and {text(obj(decode_json(row))['id']) for row in backlog} == set(BACKLOG_IDS), 'missing/duplicate backlog coverage')
        for encoded in backlog:
            row = obj(decode_json(encoded))
            required_backlog = {'id','consumers','producer','known_scenario','status','required_resolution','runtime_status'}
            optional_backlog = {'arms','outer_rounds','timed_child_argv','scripted_suffix','rss_child_argv','rss_repeats','rss_scenes','scenes','child_counts','note','known_argv'}
            require(required_backlog <= row.keys() <= required_backlog | optional_backlog, 'unknown/missing backlog key')
            require(row['status'] == 'UNRESOLVED_ADAPTER_OR_ENUMERATION' and row['runtime_status'] == 'NOT_RUN', 'invented backlog completion')
        for name, digest in obj(raw['inspected_input_sha256']).items():
            _ = relative(name)
            require(re.fullmatch('[a-f0-9]{64}',text(digest)) is not None, 'invalid inspected input hash')
        hashes = obj(raw['inspected_input_sha256'])
        for fixture in fixtures:
            require(all(path in hashes for path in strings(obj(decode_json(fixture.metadata_json))['input_paths'])), 'missing inspected input hash')
        return Registry(hashlib.sha256(data).hexdigest(), fixtures, backlog, canonical(raw))
    except (KeyError, TypeError, json.JSONDecodeError) as error:
        raise RegistryError(f'invalid registry: {error}') from error


@dataclass(frozen=True, slots=True)
class ExpansionContext:
    harness_root: Path
    inputs: Path
    output_root: Path
    base_probe: Path
    candidate_probe: Path
    f10_companion: Path | None = None


@dataclass(frozen=True, slots=True)
class GeneratedInput:
    producer_invocation: str
    path: Path
    sha256: str | None = None  # Must be bound from actual generated bytes after dependency succeeds.


@dataclass(frozen=True, slots=True)
class Invocation:
    id: str
    fixture_id: str
    arm: str
    repeat: str
    seeds: tuple[int, ...]
    roles: tuple[str, ...]
    cwd: Path
    out: Path
    argv: tuple[str, ...] | None
    environment: tuple[tuple[str, str], ...]
    generated_input: GeneratedInput | None
    blockers: tuple[str, ...]


def expand(registry: Registry, context: ExpansionContext) -> tuple[Invocation, ...]:
    """Deterministic plans only: never execute, hash generated output prematurely, or omit blocked rows."""
    for path in (context.harness_root, context.inputs, context.output_root, context.base_probe, context.candidate_probe):
        require(path.is_absolute() and '..' not in path.parts, 'expansion paths must be absolute and normalized')
    result: list[Invocation] = []
    for fixture in registry.fixtures:
        row = obj(decode_json(fixture.metadata_json))
        for arm, probe in (('base',context.base_probe),('candidate',context.candidate_probe)):
            for repeat in fixture.repeats:
                seed_groups = tuple((seed,) for seed in fixture.seeds) if '{seed}' in fixture.argv else (fixture.seeds,)
                for seeds in seed_groups:
                    suffix = f'{repeat}/seed-{seeds[0]}' if '{seed}' in fixture.argv else repeat
                    identity = f'{arm}/{fixture.id}/{suffix}'
                    out = context.output_root / identity
                    values = {'probe':str(probe),'harness_root':str(context.harness_root),'inputs':str(context.inputs),'out':str(out),'seed':str(seeds[0]),'commands_out':str(out/'commands'),'chronicle_out':str(out/'chronicle'),'replay_generate_same_arm_commands':str(context.output_root/arm/'replay.generate'/'1'/'commands')}
                    blockers = [] if fixture.adapter in SUPPORTED else ['adapter_not_implemented']
                    dependency = None
                    if fixture.id == 'replay.recorded':
                        dependency = GeneratedInput(f'{arm}/replay.generate/1',Path(values['replay_generate_same_arm_commands'])/'colony_s5_commands.json')
                    if fixture.id == 'knowledge.f10_no_kill':
                        blockers.append('companion_exact_delta_and_hash_binding_required')
                        if context.f10_companion is not None:
                            require(context.f10_companion.is_absolute(), 'companion path must be absolute')
                            values['f10_companion'] = str(context.f10_companion)
                    argv = None if '{f10_companion}' in fixture.argv and context.f10_companion is None else tuple(arg.format_map(values) for arg in fixture.argv)
                    if argv is not None and fixture.adapter in SUPPORTED:
                        argv += ('--gate-observations',)
                    environment = tuple((key,text(value).format_map(values)) for key,value in sorted(obj(row.get('environment',{})).items()))
                    result.append(Invocation(identity,fixture.id,arm,repeat,seeds,fixture.roles,context.harness_root,out,argv,environment,dependency,tuple(blockers)))
    require(len({row.id for row in result}) == len(result) and len({row.out for row in result}) == len(result), 'output/identity collision')
    return tuple(result)
