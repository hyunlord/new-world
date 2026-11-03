"""Closed Cargo doctest argument matching without constructor/setup dependencies."""
from pathlib import Path

from generic_gate import require
from generic_observations import keys
from mux2_gate_inputs import Node


def texts(value: Node) -> tuple[str, ...]:
    return tuple(item.text() for item in value.array())


def _row(row: Node) -> tuple[tuple[str, ...], dict[str, str], tuple[str, ...]]:
    keys(row, {'id', 'package_id', 'source', 'source_argument', 'package_manifest',
               'crate_name', 'edition', 'compiler_cwd', 'run_cwd', 'argument_policy'})
    args = row.field('argument_policy')
    keys(args, {'prefix', 'externs', 'suffix'})
    prefix, suffix = texts(args.field('prefix')), texts(args.field('suffix'))
    externs = {name: args.field('externs').field(name).text() for name in args.field('externs').mapping()}
    require(bool(externs) and all(name and name.replace('_', '').isalnum() for name in externs),
            '/externs', 'nonempty unique ordinary crate names required')
    require(all(Path(path).is_absolute() and str(Path(path)) == path for path in externs.values()),
            '/externs', 'canonical absolute dependency spelling required')
    require(row.field('edition').text() == '2021' and bool(row.field('id').text())
            and bool(row.field('package_id').text()), '/row', 'pinned edition/identity required')
    cwd, run = Path(row.field('compiler_cwd').text()), Path(row.field('run_cwd').text())
    source = row.field('source').text(); operand = row.field('source_argument').text()
    require(cwd.is_absolute() and run.is_absolute() and str(cwd/operand) == source
            and Path(source).is_relative_to(run) and row.field('package_manifest').text() == str(run/'Cargo.toml'),
            '/row', 'source/package paths differ')
    types = ('--crate-type', 'cdylib', '--crate-type', 'rlib') if row.field('crate_name').text() == 'sim_bridge' else ('--crate-type', 'lib')
    initial = ('--edition=2021', *types, '--color', 'auto', '--crate-name', row.field('crate_name').text(),
               '--test', operand, '--test-run-directory', str(run), '--test-runtool')
    require(prefix[:len(initial)] == initial, '/prefix', 'source-derived prefix differs')
    rest = prefix[len(initial):]
    require(len(rest) == 9 and Path(rest[0]).is_absolute() and rest[1] == '--test-runtool-arg'
            and Path(rest[2]).is_absolute() and rest[3] == rest[5] == '-L'
            and rest[4] == rest[6] and rest[4].startswith('dependency=')
            and rest[7:] == ('--test-args', '--test-threads=8'), '/prefix', 'closed runner/search/workload differs')
    require(len(suffix) == 8 and suffix[0] == '-C' and suffix[1] in ('embed-bitcode=no', 'lto=thin')
            and suffix[2:5] == ('--check-cfg', 'cfg(docsrs,test)', '--check-cfg')
            and suffix[5].startswith('cfg(feature, values(') and suffix[5].endswith('))')
            and suffix[6:] == ('--error-format', 'human'), '/suffix', 'closed profile/check-cfg differs')
    return prefix, externs, suffix


def validate_policy(policy: Node) -> None:
    """Validate the closed row schema; caller binds constructor/source authority."""
    policy.field('schema_version').equals(4)
    rows = policy.field('invocations').array()
    require(bool(rows), '/policy', 'missing doctest obligations')
    identities: set[str] = set()
    sources: set[str] = set()
    for row in rows:
        _ = _row(row)
        identity, source = row.field('id').text(), row.field('source').text()
        require(identity not in identities and source not in sources, '/policy', 'duplicate obligation')
        identities.add(identity); sources.add(source)


def match_invocation(policy: Node, argv: tuple[str, ...], cwd: Path) -> Node:
    """Match unmodified actual arguments; no path rewriting or inferred flags."""
    validate_policy(policy)
    matches: list[Node] = []
    for row in policy.field('invocations').array():
        prefix, externs, suffix = _row(row)
        if str(cwd) != row.field('compiler_cwd').text() or argv[:len(prefix)] != prefix or argv[-len(suffix):] != suffix:
            continue
        middle = argv[len(prefix):-len(suffix)]
        if len(middle) != 2*len(externs):
            continue
        observed: dict[str, str] = {}
        for index in range(0, len(middle), 2):
            require(middle[index] == '--extern' and '=' in middle[index+1], '/argv', 'closed extern pairs required')
            name, path = middle[index+1].split('=', 1)
            require(name not in observed, '/argv', 'duplicate extern')
            observed[name] = path
        if observed == externs:
            matches.append(row)
    require(len(matches) == 1, '/argv', 'no unique source-owned invocation')
    return matches[0]
