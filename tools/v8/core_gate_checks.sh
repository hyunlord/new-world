#!/usr/bin/env bash
# CORE source checks, frozen seed-0 fixtures, and strict shadow evidence checks.
# Fixture execution never launches calibration/held-out worlds or retries a round.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
mode="${1:?expected source, shadow or fixture}"
shift
case "$mode" in
baseline_isolation)
    [[ $# == 1 ]] || exit 2
    python3 - <<'PYSCAN'
from pathlib import Path
import re
root = Path('rust/crates/sim-systems/src/runtime')
assert not (root / 'decision').exists(), 'old runtime/decision must be moved'
assert not (root / 'jobs/seam.rs').exists(), 'dead colony seam must be deleted'
assert (root / 'baseline').is_dir(), 'baseline module missing'
for path in root.rglob('*.rs'):
    if path.is_relative_to(root / 'baseline'):
        continue
    source = re.sub(r'(?m)^\s*//.*$', '', path.read_text())
    assert not re.search(r'\bbaseline\s*::|\bdecision\s*::|\buse\b[^;]*\bbaseline\b', source), f'legacy selector dependency: {path}'
print('BASELINE_ISOLATION source PASS')
PYSCAN
    cargo test --manifest-path rust/Cargo.toml -p sim-systems --lib registration_tests -- --nocapture 2>&1 | tee "$1"
    grep -Eq 'test result: ok\. [1-9][0-9]* passed' "$1"
    ;;
data_only)
    [[ $# == 1 ]] || exit 2
    cargo test --manifest-path rust/Cargo.toml -p sim-systems --lib runtime::mind::system_tests::data_only_warm_at_hearth_reaches_real_target_and_retains_deciding_record -- --exact 2>&1 | tee "$1"
    grep -Eq 'test result: ok\. 1 passed' "$1"
    ;;
source)
    [[ $# == 1 ]] || { echo 'usage: core_gate_checks.sh source NEW_EXTERNAL_LOG_DIR' >&2; exit 2; }
    LOG="$1"
    python3 - "$ROOT" "$LOG" <<'PY'
from pathlib import Path
import sys
root, output = map(Path, sys.argv[1:])
if output.resolve().is_relative_to(root.resolve()):
    raise SystemExit('CORE logs must be external')
output.mkdir(parents=True, exist_ok=False)
PY
    run_test() {
        local name="$1"; shift
        "$@" 2>&1 | tee "$LOG/$name.log"
        # A renamed filter that selects no tests cannot satisfy a source gate.
        grep -Eq 'test result: ok\. [1-9][0-9]* passed' "$LOG/$name.log"
    }
    run_test mind_schema cargo test --manifest-path rust/Cargo.toml -p sim-core --lib colony::mind::
    run_test mind_records cargo test --manifest-path rust/Cargo.toml -p sim-core --lib colony::mind_records::
    run_test mind_save cargo test --manifest-path rust/Cargo.toml -p sim-systems --lib save::mind_tests::
    run_test mind_ffi cargo test --manifest-path rust/Cargo.toml -p sim-test --test engine_mind_ffi
    run_test mind_runtime cargo test --manifest-path rust/Cargo.toml -p sim-systems --features mind_shadow --lib runtime::mind::
    run_test old_observer cargo test --manifest-path rust/Cargo.toml -p sim-systems --features mind_shadow --lib mind_observer::tests
    run_test shadow_retired_stub cargo test --manifest-path rust/Cargo.toml -p sim-test --features mind_shadow --lib mind_shadow::tests
    run_test shadow_retired_cli cargo test --manifest-path rust/Cargo.toml -p sim-test --features mind_shadow --test mind_shadow_retired
    run_test story_yield cargo test --manifest-path rust/Cargo.toml -p sim-test --lib story_yield::
    python3 -m unittest discover -s tools/v8 -p 'test_core_*.py' 2>&1 | tee "$LOG/python.log"
    echo 'CORE_SOURCE PASS (runtime fixture/calibration/performance gates remain separate)'
    ;;
evaluation)
    [[ $# == 1 ]] || { echo 'usage: core_gate_checks.sh evaluation NEW_EXTERNAL_DIR' >&2; exit 2; }
    : "${CORE_EVAL_CONFIG:?external held-out/performance config required}"
    : "${CORE_EVAL_CONFIG_SHA256:?independently retained evaluation config SHA256 required}"
    python3 - "$ROOT" "$1" "$CORE_EVAL_CONFIG" "$CORE_EVAL_CONFIG_SHA256" <<'PYEVAL'
import subprocess
import sys
from pathlib import Path
sys.path.insert(0, 'tools/v8')
import core_shadow_freeze as freeze
import core_perf_check as perf

root, output, config_path = map(Path, sys.argv[1:4])
freeze.external(root, config_path)
freeze.pin(config_path, sys.argv[4])
config = freeze.load(config_path)
freeze.version(config)
required = {'schema_version', 'final_head', 'final_rust_source_sha256', 'mind_binary', 'base_binary',
            'mind_summary', 'mind_receipt', 'base_summary', 'base_receipt', 'perf_config', 'perf_receipt',
            'story_base_report', 'story_base_receipt', 'story_base_binary', 'story_base_provenance',
            'story_mind_report', 'story_mind_receipt'}
if set(config) != required:
    raise SystemExit('invalid evaluation config fields')
if config['final_head'] != perf.git('rev-parse', 'HEAD') or config['final_rust_source_sha256'] != perf.source_hash():
    raise SystemExit('evaluation source no longer matches current final source')
paths = {key: freeze.read_reference(root, config[key]) for key in required -
         {'schema_version', 'final_head', 'final_rust_source_sha256'}}
perf_inputs = freeze.load(paths['perf_config'])
policy_sha = freeze.digest(root / 'rust/crates/sim-core/data/colony/mind.ron')
for arm in ('mind', 'base'):
    receipt = freeze.load(paths[arm + '_receipt'])
    if receipt['binary_sha256'] != freeze.digest(paths[arm + '_binary']):
        raise SystemExit(f'{arm} binary does not match held-out receipt')
    if arm == 'mind' and (receipt['source_sha'] != perf_inputs['candidate_head'] or receipt['policy_sha256'] != policy_sha):
        raise SystemExit('held-out mind source/policy is not current final source')
perf_config, perf_receipt = paths['perf_config'], paths['perf_receipt']
if perf_config.name != 'config.json' or perf_receipt != perf_config.parent / 'receipt.json':
    raise SystemExit('performance references must identify one complete session')
if freeze.load(paths['base_receipt'])['source_sha'] not in (perf_inputs['base_head'], perf_inputs['expected_base']):
    raise SystemExit('held-out BASE source differs from measured performance BASE')
freeze.external(root, output)
output.mkdir(parents=True, exist_ok=False)

def run(command, name, accepted=(0,)):
    with (output / (name + '.log')).open('x') as stream:
        result = subprocess.run([str(arg) for arg in command], cwd=root, stdout=stream,
                                stderr=subprocess.STDOUT, check=False)
    if result.returncode not in accepted:
        raise subprocess.CalledProcessError(result.returncode, command)
    return result.returncode

def story_binding(mind, base, mind_binary_sha, base_binary_sha, measured_head, policy_sha, provenance_sha, expected_base):
    if mind['binary_sha256'] != mind_binary_sha or mind['source_sha'] != measured_head or mind['policy_sha256'] != policy_sha:
        raise ValueError('story mind does not match measured mind binary/source/policy')
    if base['binary_sha256'] != base_binary_sha or base['base_instrument_provenance_sha256'] != provenance_sha:
        raise ValueError('story BASE instrument binary/provenance mismatch')
    if base['runtime_base_sha'] != expected_base:
        raise ValueError('story BASE instrument does not represent nominated BASE')

def performance_gate(receipt, exit_code):
    # CORE §0 retains measurement and reporting even when its two ratio bars miss.
    report_only = {'N1000 mind/old decision>0.5', 'N1000 tick ratio exceeds accepted arithmetic bar'}
    failures = receipt['failures']
    if not isinstance(failures, list) or any(type(item) is not str for item in failures):
        raise ValueError('malformed performance failure categories')
    if receipt['saturation_companion']['status'] != 'MEASURED':
        raise ValueError('performance companion evidence pending or invalid')
    expected_status, expected_exit = ('FAIL', 1) if failures else ('PASS', 0)
    if receipt['status'] != expected_status or exit_code != expected_exit:
        raise ValueError('performance status/exit mismatch or invalid evidence')
    if any(item not in report_only for item in failures):
        raise ValueError('standing performance/C3 failure: ' + ', '.join(failures))
    return failures

run([sys.executable, 'tools/v8/core_score.py', paths['mind_summary'], '--receipt', paths['mind_receipt'],
     '--phase', 'heldout', '--base-summary', paths['base_summary'], '--base-receipt', paths['base_receipt'],
     '--output', output / 'outcomes.json'], 'outcomes')
run([sys.executable, 'tools/v8/core_why_check.py', '--summary', paths['mind_summary'],
     '--receipt', paths['mind_receipt'], '--output', output / 'why.json'], 'why')
story_binding(freeze.load(paths['story_mind_receipt']), freeze.load(paths['story_base_receipt']),
              freeze.digest(paths['mind_binary']), freeze.digest(paths['story_base_binary']),
              perf_inputs['candidate_head'], policy_sha, freeze.digest(paths['story_base_provenance']),
              perf_inputs['expected_base'])
run([sys.executable, 'tools/v8/core_story_check.py', '--base-report', paths['story_base_report'],
     '--base-receipt', paths['story_base_receipt'], '--mind-report', paths['story_mind_report'],
     '--mind-receipt', paths['story_mind_receipt'], '--base-provenance', paths['story_base_provenance'],
     '--output', output / 'story.json'], 'story')
perf_exit = run([sys.executable, 'tools/v8/core_perf_check.py', 'check', perf_config.parent,
     '--output', output / 'performance.json'], 'performance', accepted=(0, 1))
if freeze.digest(output / 'performance.json') != freeze.digest(perf_receipt):
    raise SystemExit('fresh performance verification differs from pinned receipt')
misses = performance_gate(freeze.load(output / 'performance.json'), perf_exit)
print('CORE_PERF BAR_MISS report-only: ' + ', '.join(misses) if misses else 'CORE_PERF BARS_PASS')
# Frozen inputs remain unchanged; only new gate outputs are written.
for key, path in paths.items():
    freeze.read_reference(root, config[key])
freeze.pin(config_path, sys.argv[4])
if config['final_head'] != perf.git('rev-parse', 'HEAD') or config['final_rust_source_sha256'] != perf.source_hash():
    raise SystemExit('source changed while validating evaluation evidence')
print('CORE_EVALUATION PASS held-out outcomes + why + story completeness + raw performance revalidation; no simulations launched')
PYEVAL
    ;;
fixture_run)
    [[ $# == 1 ]] || { echo 'usage: core_gate_checks.sh fixture_run NEW_EXTERNAL_DIR' >&2; exit 2; }
    : "${CORE_SHADOW_CONFIG:?external frozen fixture config required}"
    : "${CORE_SHADOW_CONFIG_SHA256:?independently retained config SHA256 required}"
    python3 - "$ROOT" "$1" "$CORE_SHADOW_CONFIG" "$CORE_SHADOW_CONFIG_SHA256" <<'PYRUN'
import json
import subprocess
import sys
from pathlib import Path
sys.path.insert(0, 'tools/v8')
import core_shadow_freeze as freeze

root, output, config_path = map(Path, sys.argv[1:4])
freeze.external(root, config_path)
freeze.pin(config_path, sys.argv[4])
config = freeze.load(config_path)
freeze.version(config)
required = {'schema_version', 'manifest', 'manifest_sha256', 'calibration'}
migration_fields = {'test_migration', 'test_migration_checker_sha256'}
if set(config) not in (required, required | migration_fields):
    raise SystemExit('invalid fixture config fields')
manifest_path = Path(config['manifest'])

def check_frozen():
    freeze.pin(config_path, sys.argv[4])
    if 'test_migration' not in config:
        freeze.check(root, manifest_path, config['manifest_sha256'])
        return
    freeze.pin(root / 'tools/v8/core_shadow_test_migration.py', config['test_migration_checker_sha256'])
    amendment = freeze.read_reference(root, config['test_migration'])
    import core_shadow_test_migration as migration
    migration.check(root, manifest_path, config['manifest_sha256'],
                    amendment, config['test_migration']['sha256'])

check_frozen()
manifest = freeze.load(manifest_path)
shadow = freeze.read_reference(root, manifest['shadow'])
control = freeze.read_reference(root, manifest['control'])
if set(config['calibration']) != {'on', 'off'}:
    raise SystemExit('both steward modes required')
freeze.external(root, output)
output.mkdir(parents=True, exist_ok=False)
helper = root / 'tools/v8/core_gate_checks.sh'

def run(command, log):
    with log.open('x') as stream:
        subprocess.run([str(arg) for arg in command], cwd=root, stdout=stream,
                       stderr=subprocess.STDOUT, check=True)

for steward in ('off', 'on'):
    approved = config['calibration'][steward]
    if set(approved) != {'shadow', 'control', 'feature_off', 'review'}:
        raise SystemExit('calibration must bind all reviewed report artifacts')
    paths = {key: freeze.read_reference(root, value) for key, value in approved.items()}
    run(['bash', helper, 'shadow', paths['shadow'], paths['control'], paths['feature_off'], paths['review']],
        output / f'calibration-{steward}-validation.log')
    review = freeze.load(paths['review'])
    old_report = freeze.load(paths['shadow'])
    inputs = old_report['inputs']
    if inputs['steward'] is not (steward == 'on'):
        raise SystemExit('calibration steward mismatch')
    if review['policy_sha256'] != freeze.digest(root / 'rust/crates/sim-core/data/colony/mind.ron'):
        raise SystemExit('calibration policy does not match frozen/current policy')
    flags = ['--scene', 'colony', '--seeds', '0..1', '--days', '20', '--workers', '1', '--steward', steward]
    defence = inputs['defence']
    if defence is not None:
        if type(defence) is not bool:
            raise SystemExit('invalid calibration defence input')
        flags += ['--defence', 'on' if defence else 'off']
    for name, allowed in [('story', ('on', 'off', 'no-beasts')), ('neighbours', ('on', 'off'))]:
        if inputs[name] not in allowed:
            raise SystemExit(f'invalid calibration input {name}')
        flags += ['--' + name, inputs[name]]
    mode_dir = output / steward
    mode_dir.mkdir()
    for name, binary, prefix in [('observed', shadow, ['mind-shadow', '--observe', 'on']),
                                  ('control', shadow, ['mind-shadow', '--observe', 'off']),
                                  ('feature-off', control, [])]:
        check_frozen()
        run([binary, *prefix, *flags, '--out', mode_dir / name], mode_dir / f'{name}.log')
    check_frozen()
    observed_path = mode_dir / 'observed/mind_shadow.json'
    twin_path = mode_dir / 'control/mind_shadow.json'
    off_path = mode_dir / 'feature-off/summary.json'
    observed = freeze.load(observed_path)
    categories = {}
    for world in observed['worlds']:
        for key, count in world['observation']['disagreement_categories'].items():
            if type(count) is not int or count < 0 or key not in review['categories']:
                raise SystemExit(f'unreviewed fixture category: {key}')
            approved_category = review['categories'][key]
            categories[key] = {**approved_category, 'count': count}
    fixture_review = {'schema_version': 1, 'fixture_run': True,
        'reviewer': review['reviewer'], 'categories': categories,
        'feature_off_inputs': inputs, 'policy_sha256': review['policy_sha256'],
        'shadow_sha256': freeze.digest(observed_path), 'control_sha256': freeze.digest(twin_path),
        'feature_off_sha256': freeze.digest(off_path),
        'category_review_origin': approved['review'],
        'review_method': 'Inherited reviewed calibration categories; counts and artifact hashes from fresh fixture.',
        'freeze_manifest_sha256': config['manifest_sha256'], 'fixture_config_sha256': sys.argv[4]}
    review_path = mode_dir / 'fixture-review.json'
    with review_path.open('x') as stream:
        json.dump(fixture_review, stream, indent=2, sort_keys=True)
    run(['bash', helper, 'fixture', observed_path, twin_path, off_path, review_path], mode_dir / 'validation.log')
freeze.pin(config_path, sys.argv[4])
check_frozen()
print('CORE_SHADOW_FIXTURE PASS fresh seed=0 days=20 steward=off,on frozen observed/control/feature-OFF')
PYRUN
    ;;
shadow|fixture)
    [[ $# == 4 ]] || { echo 'usage: core_gate_checks.sh shadow|fixture SHADOW_JSON SAME_FEATURE_CONTROL_JSON FEATURE_OFF_SUMMARY_JSON CATEGORY_REVIEW_JSON' >&2; exit 2; }
    python3 - "$mode" "$@" <<'PY'
# Strict artifact boundary. SHA binding attests bytes, not authorship or chronology.
import hashlib
import json
import re
from pathlib import Path
import sys

def require(condition, message):
    if not condition:
        raise ValueError(message)

def pairs(items):
    result = {}
    for key, value in items:
        require(key not in result, f'duplicate key: {key}')
        result[key] = value
    return result

def read(path):
    return json.loads(path.read_text(), object_pairs_hook=pairs,
                      parse_constant=lambda value: (_ for _ in ()).throw(ValueError(value)))

def integer(value):
    require(type(value) is int and value >= 0, 'expected nonnegative integer, not bool')
    return value

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def worlds(report, field):
    result = {}
    for row in report[field]:
        seed = integer(row['seed'])
        require(seed not in result, 'duplicate seed')
        result[seed] = row
    require(set(result) == set(expected_seeds), 'unexpected or incomplete seed set')
    return result

try:
    mode = sys.argv[1]
    expected_seeds = list(range(8)) if mode == 'shadow' else [0]
    observed_path, control_path, off_path, review_path = map(Path, sys.argv[2:])
    observed, control, off, review = map(read, (observed_path, control_path, off_path, review_path))
    require(review['schema_version'] == 1 and type(review['schema_version']) is int, 'review schema')
    require(isinstance(review['reviewer'], str) and review['reviewer'].strip(), 'named category reviewer required')
    if mode == 'shadow':
        require(1 <= integer(review['calibration_round']) <= 4, 'round outside admitted budget')
    else:
        require(review.get('fixture_run') is True and 'calibration_round' not in review, 'fixture is not a calibration round')
    for key, path in [('shadow_sha256', observed_path), ('control_sha256', control_path), ('feature_off_sha256', off_path)]:
        require(review[key] == sha(path), f'unbound {key}')
    require(review['policy_sha256'] == sha(observed_path.parent / 'mind_policy.ron'), 'unbound policy')
    require((observed_path.parent / 'mind_policy.ron').read_bytes() ==
            (control_path.parent / 'mind_policy.ron').read_bytes(), 'different control policy')
    for report, enabled in [(observed, True), (control, False)]:
        require(type(report['schema_version']) is int and report['schema_version'] == 1, 'shadow schema')
        require(report['scene'] == 'colony' and integer(report['days']) == 20, 'calibration scene/duration')
        require(report['observe'] is enabled and report['calibration_contract'] is (mode == 'shadow'), 'capture mode')
        require(report['seeds'] == expected_seeds and all(type(x) is int for x in report['seeds']), 'seed manifest')
        require(report['coverage_complete'] is True, 'incomplete capture')
    require(observed['inputs'] == control['inputs'] == review['feature_off_inputs'], 'input mismatch')
    require(type(observed['inputs']['steward']) is bool and observed['inputs']['player_orders'] is False, 'steward/orders contract')
    require(observed['policy_revision'] == control['policy_revision'], 'policy revision mismatch')
    require(off['scene'] == 'colony' and integer(off['days']) == 20, 'feature-OFF duration')
    old, same, feature_off = worlds(observed, 'worlds'), worlds(control, 'worlds'), worlds(off, 'seeds')
    categories, denominator, matches = {}, 0, 0
    for seed in expected_seeds:
        row, twin, baseline = old[seed], same[seed], feature_off[seed]
        for world in (row, twin):
            require(integer(world['end_tick']) == integer(world['expected_ticks']) == 20 * 1440, 'partial tick coverage')
            require(set(world['fingerprints']) == {str(day) for day in range(1, 21)}, 'missing daily fingerprints')
            require(all(isinstance(value, str) and re.fullmatch(r'[0-9a-f]{16}-[0-9a-f]{16}', value) for value in world['fingerprints'].values()), 'malformed daily fingerprints')
        for key in ['initial_fingerprint', 'initial_colony_fingerprint', 'fingerprint', 'colony_fingerprint', 'fingerprints']:
            require(row[key] == twin[key], f'observer identity divergence seed {seed}: {key}')
        require(row['fingerprints'] == baseline['fingerprints'], f'feature-OFF identity divergence seed {seed}')
        require(twin['observation'] is None, 'control accidentally enabled observation')
        stats = row['observation']
        counts = stats['counts']
        for value in counts.values(): integer(value)
        require(bool(stats['by_arm']), 'missing arm breakdown')
        for key, total in counts.items():
            require(sum(integer(arm[key]) for arm in stats['by_arm'].values()) == total, f'arm accounting mismatch: {key}')
        for key in ['error_count', 'unmatched_predictions']:
            require(integer(stats[key]) == 0, f'nonzero {key}')
        for key in ['uncovered', 'missing_prediction', 'pending_settlement']:
            require(integer(counts[key]) == 0, f'nonzero {key}')
        total, port, agreed = counts['person_ticks'], counts['port_person_ticks'], counts['port_behaviour_matches']
        require(0 < port <= total <= counts['evaluations'], 'invalid observation denominator')
        require(agreed <= port and counts['port_band_matches'] <= port and counts['port_both_matches'] <= min(agreed, counts['port_band_matches']), 'invalid agreement counts')
        require(integer(stats['prediction_evaluations']) == counts['evaluations'], 'evaluation coverage mismatch')
        require(integer(stats['replay_evaluations']) == counts['evaluations'] - total, 'replay accounting mismatch')
        denominator += port
        matches += agreed
        for key, count in stats['disagreement_categories'].items():
            categories[key] = categories.get(key, 0) + integer(count)
    require(matches * 100 >= denominator * 97, 'port behavior agreement must be at least 97 percent')
    require(set(review['categories']) == set(categories), 'unreviewed or extraneous disagreement categories')
    for key, count in categories.items():
        item = review['categories'][key]
        require(integer(item['count']) == count, 'review category count mismatch')
        require(item['reviewed'] is True and isinstance(item['explanation'], str) and item['explanation'].strip(), 'category lacks review explanation')
    print(f'CORE_SHADOW PASS mode={mode} steward={observed["inputs"]["steward"]} matched={matches} port={denominator} categories={len(categories)}')
    print('Limited gate: requires both steward modes; round chronology, binary provenance, held-out, why coverage and performance remain separate.')
except (OSError, ValueError, KeyError, TypeError) as error:
    raise SystemExit(f'CORE_SHADOW FAIL: {error}')
PY
    ;;
*) echo "unknown CORE gate mode: $mode" >&2; exit 2 ;;
esac
