"""Source-bound original M1 chain; command descriptions confer no execution authority."""
import shlex
from dataclasses import dataclass
from pathlib import Path
from typing import Final, Literal

from generic_gate import require


@dataclass(frozen=True, slots=True)
class CargoCommand:
    identity: str
    package: str
    targets: tuple[str, ...]
    filter: str | None
    threads: int | None
    kind: Literal['integration', 'library']

    @property
    def argv(self) -> tuple[str, ...]:
        selection = tuple(part for target in self.targets for part in ('--test', target))
        if self.kind == 'library':
            selection = ('--lib',)
        tail = () if self.filter is None else (self.filter,)
        if self.threads is not None:
            tail += ('--', f'--test-threads={self.threads}')
        return ('cargo', 'test', '--release', '-p', self.package, *selection, *tail)


COMMANDS: Final = (
    CargoCommand('integration', 'sim-test',
                 ('colony_commands', 'colony_jobs', 'colony_world', 'engine_colony_ffi'),
                 None, 8, 'integration'),
    CargoCommand('probe', 'sim-test', (), 'probe::colony', None, 'library'),
    CargoCommand('jobs', 'sim-systems', (), 'jobs::', None, 'library'),
    CargoCommand('colony', 'sim-core', (), 'colony::', None, 'library'),
)
LOCALE: Final = (
    'bash tools/v8/run_bounded.sh 120 "$GODOT_BIN" --headless --path . '
    '--script res://tools/v8/test_locale_trf.gd > "$0/m1-trf.log" 2>&1; '
    'godot_code=$?; cat "$0/m1-trf.log" || exit 1; '
    '(( godot_code == 0 )) && grep -q "^TRF_TEST_PASS" "$0/m1-trf.log"'
)
BODY: Final = ('cd rust && ' + ' && '.join(shlex.join(row.argv) for row in COMMANDS)
               + ' && cd .. || exit $?; ' + LOCALE)


def authenticate_source(path: Path) -> str:
    """Require the complete original shell and aggregate-summary relationship."""
    lines = path.read_text().splitlines()
    selected = [index for index, line in enumerate(lines) if line.startswith('step 17 ')]
    require(len(selected) == 1, '/m1', 'missing or duplicate original step17')
    index = selected[0]
    require(lines[index] == f'step 17 m1_tests bash -c {shlex.quote(BODY)} "$LOG"',
            '/m1', 'original M1 command chain changed')
    require(index + 1 < len(lines) and lines[index + 1] == 'summarize_tests m1_tests "$?"',
            '/m1', 'original aggregate summary changed')
    return BODY
