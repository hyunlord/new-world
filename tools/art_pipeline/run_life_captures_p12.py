"""Capture fixed live P12 worlds; keep raw references outside anonymous reader bundle.

Run: P12_LIFE_BUILD_PROFILE=debug python3 run_life_captures_p12.py CHECKOUT NEWDIR
Only the leader schedules this windowed runner after the source freeze.
"""
from __future__ import annotations
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path
from run_activity_captures_p12 import QUIET, bounded_launch, bridge_identity, source_digest
from prepare_blind_crops_p12 import crowd_proof


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def prepare_reader_bundle(sources: list[Path], output: Path) -> list[dict[str, str]]:
    """Copy native PNG bytes under opaque names; never put private references inside."""
    if len({path.resolve() for path in sources}) != len(sources):
        raise ValueError('duplicate source images')
    output.mkdir(parents=True, exist_ok=False)
    mapping = []
    for index, source in enumerate(sources, 1):
        name = f'image_{index:04d}.png'
        shutil.copyfile(source, output / name)
        mapping.append({'file': name, 'source': str(source), 'sha256': sha256(source)})
    (output / 'instructions.json').write_text(json.dumps({
        'task': 'For each screenshot, state what you can read. Label guesses as inference and mark unreadable text. Identify the selected person activity in crowded views. Flag visible raw keys, percent/joiner artifacts, or split names. Freeze answers before reference data is opened.',
        'images': [row['file'] for row in mapping], 'pixels': 'native, unchanged'}, indent=2) + '\n')
    return mapping


def life_launch_command(checkout: Path, godot: Path) -> list[str]:
    """Recheck the foreign quiet lock after obtaining the shared window lock."""
    return ['bash', 'tools/art_pipeline/with_window_lock.sh', '1800', 'bash', '-c',
            'while [[ -d "$1" ]]; do date -u; echo "waiting for quiet lock $1"; sleep 30; done; shift; exec "$@"',
            'life-launch', str(QUIET), str(godot), '--path', str(checkout),
            '--fixed-fps', '60', '--resolution', '1920x1080',
            '--script', 'res://scripts/test/art/capture_life_p12.gd']


def run(checkout: Path, output: Path) -> int:
    output.mkdir(parents=True, exist_ok=False)
    profile = os.environ.get('P12_LIFE_BUILD_PROFILE', 'debug')
    identity = bridge_identity(checkout, profile)
    digest = source_digest(checkout)
    godot = Path(os.environ.get('GODOT_BIN', '/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot'))
    head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=checkout, text=True).strip()
    (output / 'working-tree.patch').write_bytes(subprocess.check_output(['git', 'diff', 'HEAD', '--'], cwd=checkout))
    registration = {'head': head, 'source_digest': digest, 'loaded_bridge_sha256': identity.loaded_bridge_sha256,
                    'build_profile': profile, 'build_profile_details': os.environ.get('P12_BUILD_DETAILS', ''),
                    'script_sha256': sha256(checkout / 'scripts/test/art/capture_life_p12.gd'),
                    'runner_sha256': sha256(Path(__file__)), 'cases': ['live', 'multi', 'estranged', 'crowd1', 'crowd2', 'crowd3']}
    (output / 'registration.json').write_text(json.dumps(registration, indent=2) + '\n')
    images = []
    for case in registration['cases']:
        if source_digest(checkout) != digest:
            raise RuntimeError('source changed before next life capture; retain attempt')
        target = output / case
        environment = os.environ.copy()
        environment.update(P12_LIFE_OUT=str(target), P12_LIFE_CASE=case, P12_LIFE_BUILD_PROFILE=profile)
        command = life_launch_command(checkout, godot)
        with (output / f'{case}.log').open('x') as log:
            code = bounded_launch(command, checkout, environment, log, time.time()+1800, 1800)
        errors = []
        reference = target / 'reference-data.json'
        if code or not reference.exists():
            errors.append('capture failed or complete reference missing')
        else:
            data = json.loads(reference.read_text())
            if data.get('provenance', {}).get('head') != head:
                errors.append('inside-launch HEAD differs from registration')
            if not data.get('complete') or not data.get('images'):
                errors.append('incomplete life image set')
            for item in data['images']:
                context = item.get('page_context', {})
                if 'crowd' in context:
                    proof = crowd_proof(context['crowd'])
                    (target / (Path(item['file']).stem + '-crowd-proof.json')).write_text(json.dumps(proof, indent=2) + '\n')
                    if not proof['zero_coverage_crowd_proven']:
                        errors.append('selected crowd geometry failed')
                image = target / item['file']
                if sha256(image) != item['png_sha256']:
                    errors.append('image hash mismatch')
                images.append(image)
        if source_digest(checkout) != digest:
            errors.append('source changed during life capture')
        if subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=checkout, text=True).strip() != head:
            errors.append('HEAD changed during life capture')
        text = (output / f'{case}.log').read_text()
        if 'SCRIPT ERROR' in text or 'ERROR:' in text:
            errors.append('Godot errors in capture log')
        (output / f'{case}-result.json').write_text(json.dumps({'exit_code': code, 'errors': errors,
            'reference_sha256': sha256(reference) if reference.exists() else None}, indent=2) + '\n')
        if errors:
            return 1
    mapping = prepare_reader_bundle(images, output / 'reader')
    (output / 'private-reader-map.json').write_text(json.dumps(mapping, indent=2) + '\n')
    return 0


if __name__ == '__main__':
    raise SystemExit(run(Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()))
