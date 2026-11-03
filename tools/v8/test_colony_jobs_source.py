"""Original Rust token stream is unchanged after removing only observer statements."""
import re
import unittest
from pathlib import Path

DRAFT = Path(__file__).resolve().parents[2]
SOURCE = 'rust/crates/sim-test/tests/colony_jobs.rs'


def strip_observers(source: str) -> str:
    pattern = r'^\s*(?:let (?:again_)?watch = Watch::begin|(?:again_)?watch\.(?:annotate|finish))\('
    for match in reversed(list(re.finditer(pattern, source, re.MULTILINE))):
        depth = 0
        quoted = False
        escaped = False
        for index in range(match.start(), len(source)):
            character = source[index]
            if quoted:
                if escaped:
                    escaped = False
                elif character == '\\':
                    escaped = True
                elif character == '"':
                    quoted = False
            elif character == '"':
                quoted = True
            elif character in '([{':
                depth += 1
            elif character in ')]}':
                depth -= 1
            elif character == ';' and depth == 0:
                source = source[:match.start()] + source[index + 1:]
                break
        else:
            raise AssertionError('observer statement did not terminate')
    return source


def tokens(source: str) -> list[str]:
    parsed = [match.group(0) for match in re.finditer(r'"(?:\\.|[^"\\])*"|//[^\n]*|/\*.*?\*/|\w+|[^\s]', strip_observers(source), re.DOTALL)]
    return [token for index, token in enumerate(parsed) if not token.startswith(('//', '/*'))
            and not (token == ',' and index + 1 < len(parsed) and parsed[index + 1] in {')', '}'} )]


class SourceTests(unittest.TestCase):
    def test_original_setup_commands_loops_oracles_unchanged(self) -> None:
        self.assertEqual(tokens((Path(__file__).parent / 'fixtures/colony_jobs.original.rs').read_text()), tokens((DRAFT / SOURCE).read_text()))

    def test_exact_ten_worlds_including_repeat_and_existing_hut(self) -> None:
        source = (DRAFT / SOURCE).read_text()
        self.assertEqual(source.count('Watch::begin('), 10)
        self.assertEqual(len(re.findall(r'\b(?:again_)?watch\.finish\(', source)), 10)
