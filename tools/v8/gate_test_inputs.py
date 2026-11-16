#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by gate_test_evidence.py.
"""Frozen debug-suite inputs; deliberately no Cargo subprocesses."""
import hashlib
import os
from pathlib import Path
import subprocess
import shutil
import re
import tomllib

from mux2_gate_inputs import Json, digest, git, require

COMMAND = 'cargo test --workspace --no-fail-fast -- --test-threads=8'
FILE_ENV = ('C3_C2B_SAVE_PATH', 'C2_BASE_SAVE_PATH', 'M6B_BASE_V14_SAVE',
            'M6B_BASE_V14_OFF_SAVE', 'M6B_BASE_V14_SUMMARY', 'WORLDSIM_B5_LEGACY9',
            'WORLDSIM_B4_MANIFEST')
DIR_ENV = ('C2_A22_FIXTURES', 'M6A_FORMAT13_FIXTURE_DIR', 'MUX2_V10_FIXTURES',
           'MUX_V6_FIXTURES', 'WORLDSIM_C1_FORMAT20_FIXTURES',
           'WORLDSIM_M6B_FORMAT16_FIXTURES', 'WORLDSIM_MUX_FIXTURES')
NORMALIZED_ENV = ('WORLDSIM_TEST_SETTINGS_DIR', 'WORLDSIM_MUX2_LANE_FD')



def inventory(root: Path) -> list[str]:
    workspace = tomllib.loads((root / 'rust/Cargo.toml').read_text())['workspace']
    require(not workspace.get('exclude'), 'unsupported workspace exclusions')
    result = []
    for member in workspace['members']:
        require('*' not in member, 'unsupported workspace member glob')
        base = root / 'rust' / member
        manifest = tomllib.loads((base / 'Cargo.toml').read_text())
        package = manifest['package']
        name = package['name'].replace('-', '_')
        require(not manifest.get('test') and package.get('autotests', True),
                'explicit/disabled test inventory needs inventory support')
        lib = manifest.get('lib', {})
        if (base / lib.get('path', 'src/lib.rs')).is_file():
            require(lib.get('test', True) and lib.get('doctest', True), 'disabled lib tests unsupported')
            result.extend([f'unit:{name}:lib', f'doc:{name}'])
        bins = {row['name']: row for row in manifest.get('bin', [])}
        if package.get('autobins', True):
            if (base / 'src/main.rs').is_file():
                bins.setdefault(package['name'], {'path': 'src/main.rs'})
            for path in (base / 'src/bin').glob('*.rs'):
                bins.setdefault(path.stem, {'path': str(path.relative_to(base))})
            for path in (base / 'src/bin').glob('*/main.rs'):
                bins.setdefault(path.parent.name, {'path': str(path.relative_to(base))})
        for bin_name, row in bins.items():
            require(row.get('test', True) and not row.get('required-features'), 'conditional binary unsupported')
            result.append(f'unit:{bin_name.replace("-", "_")}:{row.get("path", "src/bin/" + bin_name + ".rs")}')
        for path in (base / 'tests').glob('*'):
            if path.suffix == '.rs' or (path.is_dir() and (path / 'main.rs').is_file()):
                result.append('test:' + (path.stem if path.is_file() else path.name))
    require(bool(result), 'ambiguous/empty target inventory')
    return sorted(result)


def tracked_hashes(root: Path, paths: set[str]) -> dict[str, str]:
    root = root.resolve()
    files = {}
    for name in sorted(paths):
        path = root / name
        if '__pycache__' in path.parts or path.suffix == '.pyc':
            continue
        if path.is_symlink() and path.is_dir():
            target = path.resolve()
            require(target.is_relative_to(root), 'external tracked directory link unsupported')
            files[name] = hashlib.sha256(os.readlink(path).encode()).hexdigest()
            require(not any(p.is_symlink() and p.is_dir() for p in target.rglob('*')),
                    'nested directory link unsupported')
            files.update({name + '/' + str(p.relative_to(target)): digest(p)
                          for p in sorted(target.rglob('*')) if p.is_file()})
        else:
            files[name] = digest(path)
    return files


def capture(root: Path) -> dict[str, Json]:
    root = root.resolve()
    require(not git(root, 'status', '--porcelain', '--untracked-files=all'), 'worktree must be clean')
    paths = set(git(root, 'ls-files', '-z').split('\0')) - {''}
    # Include ignored/untracked runtime data and fixtures; exclude build/import caches.
    for folder in ('rust/crates', 'data', 'fixtures', 'assets', 'localization', 'rules', 'content', 'tools/v8', 'scripts'):
        directory = root / folder
        require(not directory.is_symlink(), 'symlink input root unsupported')
        require(not any(p.is_symlink() and p.is_dir() for p in directory.rglob('*')),
                'symlink input directory unsupported')
        paths.update(str(p.relative_to(root)) for p in directory.rglob('*') if p.is_file())
    for name in ('rust/Cargo.lock', 'rust/rust-toolchain', 'rust/rust-toolchain.toml',
                 'rust-toolchain', 'rust-toolchain.toml'):
        if (root / name).is_file():
            paths.add(name)
    require((root / 'rust/Cargo.lock').is_file(), 'missing Cargo.lock')
    files = tracked_hashes(root, paths)
    configs = {}
    for directory in [root / 'rust', root, *root.parents]:
        for filename in ('config', 'config.toml'):
            path = directory / '.cargo' / filename
            if path.is_file():
                configs[str(path)] = digest(path)
    cargo_home = Path(os.environ.get('CARGO_HOME', str(Path.home() / '.cargo')))
    for filename in ('config', 'config.toml'):
        path = cargo_home / filename
        if path.is_file():
            configs[str(path)] = digest(path)
    # Literal Rust environment readers extend the scalar registry automatically.
    registry = set(FILE_ENV + DIR_ENV + ('C3_OUTCROP_KIND',))
    for source in (root / 'rust/crates').rglob('*.rs'):
        registry.update(re.findall(r'env::var(?:_os)?\(\s*"([A-Z][A-Z0-9_]+)"', source.read_text()))
    registry.difference_update(NORMALIZED_ENV)
    # Explicit content packs can load referenced paths outside their descriptor directory.
    require('WORLDSIM_CONTENT_PACK' not in os.environ, 'external content pack reuse unsupported')
    fixture_inputs = {}
    for key in FILE_ENV + DIR_ENV:
        value = os.environ.get(key)
        if value is None:
            fixture_inputs[key] = None
            continue
        path = Path(value)
        require(path.is_absolute(), 'fixture input must be absolute: ' + key)
        require(path.is_file() if key in FILE_ENV else path.is_dir(), 'missing fixture input ' + key)
        require(not path.is_dir() or not any(p.is_symlink() and p.is_dir() for p in path.rglob('*')),
                'symlink fixture directory unsupported')
        fixture_inputs[key] = (digest(path) if key in FILE_ENV else
                               {str(p.relative_to(path)): digest(p) for p in sorted(path.rglob('*'))
                                if p.is_file()})
    env = {k: hashlib.sha256(v.encode()).hexdigest() for k, v in sorted(os.environ.items())
           if k not in NORMALIZED_ENV
           and (k.startswith(('CARGO_', 'RUST', 'SIM_', 'WORLDSIM_', 'DYLD_', 'LD_'))
           or k in ('PATH', 'HOME', 'LANG', 'LC_ALL', 'TZ', 'TMPDIR', 'MACOSX_DEPLOYMENT_TARGET', 'SDKROOT',
                    'CC', 'CXX', 'AR', 'CFLAGS', 'CXXFLAGS', 'LDFLAGS', 'NUM_JOBS'))}
    env.update({key: (hashlib.sha256(os.environ[key].encode()).hexdigest()
                      if key in os.environ else None) for key in sorted(registry)})
    settings = os.environ.get('WORLDSIM_TEST_SETTINGS_DIR')
    if settings:
        directory = Path(settings)
        require(directory.is_dir() and not directory.is_symlink(), 'missing/linked test settings directory')
        require(not any(p.is_symlink() and p.is_dir() for p in directory.rglob('*')),
                'symlink settings directory unsupported')
        configs['test-settings'] = {str(p.relative_to(directory)): digest(p)
                                    for p in sorted(directory.rglob('*')) if p.is_file()}
    # Version metadata plus executable hashes bind the installed toolchain without invoking Cargo.
    for executable in ('rustc', 'cargo', 'rustup'):
        found = shutil.which(executable)
        require(found is not None, 'missing toolchain executable ' + executable)
        configs['executable:' + executable] = digest(Path(found).resolve())
    for executable in ('cargo', 'rustc'):
        selected = subprocess.run(['rustup', 'which', executable], cwd=root / 'rust',
                                  capture_output=True, text=True, check=True).stdout.strip()
        configs['selected:' + executable] = digest(Path(selected))
    version = subprocess.run(['rustc', '--version', '--verbose'], cwd=root / 'rust',
                             capture_output=True, text=True, check=True).stdout
    return {'commit': git(root, 'rev-parse', 'HEAD'), 'tree': git(root, 'rev-parse', 'HEAD^{tree}'),
            'files': files, 'configs': configs, 'fixtures': fixture_inputs, 'environment': env, 'rustc': version,
            'command': COMMAND, 'profile': 'test', 'features': 'default',
            'inventory': inventory(root)}
