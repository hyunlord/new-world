#!/usr/bin/env bash
# usage: check_docs.sh [--skip-system-table]
#
# V8 docs check (goal 9.5, 10.1, 10.5, 11.1 step 14). Prints one line per
# check and `docs: PASS` / `docs: FAIL`; exits non-zero on any failure.
#   1. required files exist; CHANGELOG.md has a `## V8-0` entry; HISTORY.md < 400 lines
#   2. every ADR in docs/decisions has the six sections; at least 11 ADRs
#   3. every repository path named in backticks in docs/ARCHITECTURE.md exists
#   4. the ARCHITECTURE system table equals tools/v8/system_table.py output
#      (needs a release build of sim-test; --skip-system-table skips it)
#   5. skills: frontmatter name + description, ≤ 400 lines, a Lessons table whose
#      rows each name a source; .agents/skills/<name> is a symlink to
#      ../../.claude/skills/<name> (git mode 120000) that resolves
#   6. AGENTS.md ≤ 150 lines, no `reset --hard`, no instruction to push
#      lead/main or main; CLAUDE.md is `@AGENTS.md` plus ≤ 30 lines
#   7. no tracked file > 5 MB (except assets/fonts/galmuri/, D27); no binary under docs/ except ≤ 12 images in docs/images/
#   8. Godot UIDs are committed (M1 5.3): every tracked .gd/.gdshader outside a .gdignore'd
#      folder has its tracked .uid, every tracked .uid has its source, every .tscn header has a uid
#   9. a contact sheet of the current game exists (V8-PLAY 3): docs/images/*_contact_sheet.png, and
#      README.md links each one by its path
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
status=0
fail() { echo "FAIL: $*"; status=1; }
ok() { echo "ok: $*"; }

SKILLS=(worldsim-architecture worldsim-godot worldsim-pitfalls worldsim-testing worldsim-docs-upkeep systematic-debugging verification-before-completion)
REQUIRED=(README.md AGENTS.md CLAUDE.md CHANGELOG.md docs/VISION.md docs/ROADMAP.md docs/HISTORY.md docs/ARCHITECTURE.md docs/archive/INDEX.md docs/v8/V8_DIRECTION.md)
for f in "${REQUIRED[@]}"; do [[ -f "$f" ]] || fail "missing $f"; done
for s in "${SKILLS[@]}"; do [[ -f ".claude/skills/$s/SKILL.md" ]] || fail "missing .claude/skills/$s/SKILL.md"; done
grep -q '^## V8-0' CHANGELOG.md && ok "CHANGELOG has ## V8-0" || fail "CHANGELOG.md has no '## V8-0' entry"
lines=$(wc -l < docs/HISTORY.md | tr -d ' '); (( lines < 400 )) && ok "HISTORY.md $lines lines" || fail "HISTORY.md has $lines lines (limit 400)"

adrs=0
for adr in docs/decisions/[0-9][0-9][0-9][0-9]-*.md; do
    [[ -f "$adr" ]] || continue
    adrs=$((adrs + 1))
    for section in Context Decision "Alternatives considered" Consequences Date Source; do
        grep -q "^## $section\$" "$adr" || fail "$adr lacks '## $section'"
    done
done
(( adrs >= 11 )) && ok "$adrs ADRs" || fail "only $adrs ADRs (need 11)"

missing=$(python3 tools/v8/check_docs_helpers.py arch-paths)
[[ -z "$missing" ]] && ok "every path named in ARCHITECTURE.md exists" || fail "ARCHITECTURE.md names missing paths: $(echo "$missing" | tr '\n' ' ')"

if [[ "${1:-}" != --skip-system-table ]]; then
    python3 tools/v8/system_table.py --check > /tmp/check_docs_system_table.$$ 2>&1 && ok "$(cat /tmp/check_docs_system_table.$$)" || fail "$(tail -3 /tmp/check_docs_system_table.$$)"
    rm -f /tmp/check_docs_system_table.$$
fi

for s in "${SKILLS[@]}"; do
    f=".claude/skills/$s/SKILL.md"
    [[ -f "$f" ]] || continue
    head -5 "$f" | grep -q "^name: $s\$" || fail "$f: frontmatter name is not '$s'"
    head -6 "$f" | grep -q '^description: ..*' || fail "$f: frontmatter has no description"
    n=$(wc -l < "$f" | tr -d ' '); (( n <= 400 )) || fail "$f has $n lines (limit 400)"
    link=".agents/skills/$s"
    if [[ -L "$link" && "$(readlink "$link")" == "../../.claude/skills/$s" && -f "$link/SKILL.md" ]]; then
        mode=$(git ls-files -s "$link" | awk '{print $1}')
        [[ -z "$mode" || "$mode" == 120000 ]] || fail "$link is tracked with mode $mode, not 120000"
    else
        fail "$link is not a resolving symlink to ../../.claude/skills/$s"
    fi
done
lesson_check=$(python3 tools/v8/check_docs_helpers.py lessons "${SKILLS[@]:0:5}")
echo "$lesson_check" | sed 's/^/ok: skill /'
echo "$lesson_check" | grep -q 'without a source\|no Lessons' && fail "a skill lesson lacks a source or a skill lacks a Lessons table"

n=$(wc -l < AGENTS.md | tr -d ' '); (( n <= 150 )) && ok "AGENTS.md $n lines" || fail "AGENTS.md has $n lines (limit 150)"
grep -q 'reset --hard' AGENTS.md CLAUDE.md && fail "AGENTS.md/CLAUDE.md mention 'reset --hard'"
grep -nE 'git push[^`]*\b(lead/main|origin main)\b' AGENTS.md CLAUDE.md | grep -viE 'never|not ' && fail "AGENTS.md/CLAUDE.md instruct a push to lead/main or main"
head -1 CLAUDE.md | grep -qx '@AGENTS.md' || fail "CLAUDE.md does not start with @AGENTS.md"
extra=$(( $(wc -l < CLAUDE.md | tr -d ' ') - 1 )); (( extra <= 30 )) && ok "CLAUDE.md = @AGENTS.md + $extra lines" || fail "CLAUDE.md has $extra lines beyond @AGENTS.md (limit 30)"

# The Galmuri fonts are kept by goal 7.5 (assets/fonts/galmuri/**); Galmuri11.ttf
# is 5.4 MB. They are the only allowed exception (DECISIONS D27).
big=$(git ls-files -z | xargs -0 stat -f '%z %N' 2>/dev/null | awk '$1 > 5242880 {print $2}' | grep -v '^assets/fonts/galmuri/' || true)
[[ -z "$big" ]] && ok "no tracked file > 5 MB (assets/fonts/galmuri/ exempt, D27)" || fail "tracked files > 5 MB: $big"
bins=$(git ls-files docs | grep -viE '\.(md|txt|json|csv|py|sh)$' || true)
outside=$(printf '%s\n' "$bins" | grep -v '^docs/images/' | grep -v '^$' || true)
images=$(printf '%s\n' "$bins" | grep -c '^docs/images/' || true)
[[ -z "$outside" ]] || fail "binary files under docs/: $(echo "$outside" | head -5 | tr '\n' ' ')"
(( images <= 12 )) && ok "$images images in docs/images" || fail "$images images in docs/images (limit 12)"

uids=$(python3 tools/v8/check_docs_helpers.py uid)
[[ -z "$uids" ]] && ok "every Godot source has its committed .uid" || fail "UID problems: $(echo "$uids" | head -5 | tr '\n' ' ')"

sheets=$(ls docs/images/*_contact_sheet.png 2>/dev/null || true)
if [[ -z "$sheets" ]]; then
    fail "no docs/images/*_contact_sheet.png (the current game's contact sheet, V8-PLAY 3)"
else
    for sheet in $sheets; do
        grep -qF "($sheet)" README.md && ok "README links $sheet" || fail "README.md does not link $sheet"
    done
fi

if (( status == 0 )); then echo "docs: PASS"; else echo "docs: FAIL"; fi
exit "$status"
