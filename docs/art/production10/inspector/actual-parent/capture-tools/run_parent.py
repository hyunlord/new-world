"""Later only: python3 run_parent.py NEW_OUTPUT_DIRECTORY. One bounded EN/KO launch."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path('/Users/rexxa/work/new-world-art-prod')
PREP = Path(__file__).resolve().parent
DEADLINE = 1792119780.0
sys.path.insert(0, str(ROOT / 'tools/art_pipeline'))
from run_activity_captures_p10 import bounded_launch, remaining, source_digest, utc, wait_for_quiet


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def bindings() -> dict[str, str]:
    paths = [PREP / 'capture_parent_locales_p10.gd', Path(__file__).resolve(),
             ROOT / 'rust/target/debug/libsim_bridge.dylib', ROOT / 'sim_bridge.gdextension',
             ROOT / 'rust/crates/sim-bridge/src/ffi/world_node.rs',
             ROOT / 'tools/art_pipeline/run_activity_captures_p10.py',
             ROOT / 'tools/art_pipeline/with_window_lock.sh', ROOT / 'tools/art_pipeline/run_bounded.sh',
             ROOT / 'localization/key_registry.json']
    paths += [ROOT / f'localization/compiled/{lang}.json' for lang in ('en', 'ko')]
    paths += [ROOT / f'localization/fluent/{lang}/messages.ftl' for lang in ('en', 'ko')]
    return {str(path): sha(path) for path in paths}


def validate(output: Path) -> list[str]:
    from PIL import Image
    data = json.loads((output / 'parent.json').read_text())
    errors: list[str] = []
    if (data['seed'], data['display_day'], data['hour'], data['tick'], data['zoom'], data['fixed_fps']) != (20261010, 30, 10, 41880, 4, 60):
        errors.append('wrong live world/clock/camera/launch cohort')
    if data['delta_valid'] is not True or data['delta_samples'] <= 0 or data['panel_height'] > 620:
        errors.append('invalid delta or panel dimensions')
    live = {row['agent_id']: row['entity_bits'] for row in data['census']}
    family = data['family']
    if (not family.get('found') or live.get(data['agent_id']) != data['entity_bits']
            or family['partner'] not in live or not any(child in live for child in family['children'])):
        errors.append('selected parent family not verified by live census')
    expected = {(lang, view) for lang in ('en', 'ko') for view in ('top', 'temperament', 'family')}
    if len(data['captures']) != 6 or {(row['locale'], row['view']) for row in data['captures']} != expected:
        errors.append('six locale/view captures required')
    for row in data['captures']:
        if row['tick'] != 41880 or row['panel_height'] > 620 or row['scroll_offset'] < 0:
            errors.append('invalid per-view clock/height/scroll')
        filename = f"parent-{row['locale']}-{row['view']}.png"
        if row['file'] != filename:
            errors.append('unexpected screenshot name')
            continue
        with Image.open(output / filename) as image:
            if image.size != (1920, 1080):
                errors.append('wrong framebuffer size')
        for ring in row['rings']:
            if ring['agent_id'] not in live or ring['agent_id'] not in [family['partner'], *family['children']] or not 0 < ring['alpha'] < 1:
                errors.append('ring not on living partner/child with lower alpha')
        if not data['optional_rings_installed'] and row['rings']:
            errors.append('absent rings implementation emitted visible rings')
    if len(data['clicks']) != 4 or {(row['locale'], row['button_index']) for row in data['clicks']} != {(lang, index) for lang in ('en', 'ko') for index in (0, 3)}:
        errors.append('four real-world relation clicks required')
    for row in data['clicks']:
        target = row['target_agent_id']
        if (target not in live or row['selected_entity_bits'] != live[target]
                or row['detail'].get('agent_id') != target or row['detail'].get('found') is not True):
            errors.append('family click did not resolve live selected entity')
        if row['button_index'] == 0 and target != family['partner']:
            errors.append('partner click points elsewhere')
        if row['button_index'] == 3 and target not in family['children']:
            errors.append('child click points elsewhere')
    return errors


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    remaining(DEADLINE)
    if 'P10_DEADLINE_EPOCH' in os.environ and float(os.environ['P10_DEADLINE_EPOCH']) != DEADLINE:
        raise SystemExit('deadline must equal original1792119780')
    output = Path(sys.argv[1]).resolve()
    before = bindings()
    digest = source_digest(ROOT)
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    output.mkdir(exist_ok=False)
    (output / 'working-tree.patch').write_bytes(subprocess.check_output(['git', 'diff', 'HEAD', '--'], cwd=ROOT))
    (output / 'git-status.txt').write_bytes(subprocess.check_output(['git', 'status', '--short'], cwd=ROOT))
    (output / 'source-before.json').write_text(json.dumps({'head': head, 'source_digest': digest, 'bindings': before}, indent=2)+'\n')
    wait_for_quiet(DEADLINE, output / 'quiet-waits.log', 'parent-en-ko')
    if bindings() != before or source_digest(ROOT) != digest:
        raise SystemExit('source drift while waiting; preserve attempt')
    godot = os.environ.get('GODOT_BIN', '/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot')
    if not os.access(godot, os.X_OK):
        raise SystemExit('GODOT_BIN not executable')
    command = ['bash', 'tools/art_pipeline/with_window_lock.sh', '1800', 'bash', '-c',
               'while [[ -d /Users/rexxa/work/.mac_quiet.lock ]]; do date -u; echo waiting-for-quiet; sleep 300; done; exec "$@"',
               'parent-capture', godot, '--path', str(ROOT), '--fixed-fps', '60', '--resolution', '1920x1080',
               '--script', str(PREP / 'capture_parent_locales_p10.gd')]
    environment = dict(os.environ, P10_PARENT_OUT=str(output))
    started = utc()
    with (output / 'session.log').open('w') as log:
        code = bounded_launch(command, ROOT, environment, log, DEADLINE, 1800)
    errors: list[str] = []
    try:
        errors.extend(validate(output))
    except (OSError, ValueError, KeyError, TypeError, IndexError) as error:
        errors.append(str(error))
    after = bindings()
    head_after = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    if before != after or source_digest(ROOT) != digest or head_after != head:
        errors.append('source/build/localization/HEAD drift')
    log_text = (output / 'session.log').read_text()
    if 'SCRIPT ERROR' in log_text or 'ERROR:' in log_text:
        errors.append('Godot errors in log')
    result = {'started': started, 'finished': utc(), 'exit_code': code, 'deadline_epoch': DEADLINE,
              'command': command, 'validation_errors': errors, 'capture_contract_pass': code == 0 and not errors,
              'visual_acceptance': 'pending', 'pointer_hit_testing': 'not tested; real buttons pressed by signal',
              'source_after': after, 'artifacts': {path.name: sha(path) for path in output.iterdir() if path.is_file()}}
    (output / 'launch.json').write_text(json.dumps(result, indent=2)+'\n')
    return 0 if result['capture_contract_pass'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
