# Final delivery verification

This is an offline candidate delivery. Owner approval and game installation are
not claimed. Item style/readability remain FAIL; final combined type checking is
NOT_GREEN because warnings remain.

From repository root, 2026-10-24 KST:

```sh
uv run --offline --with pillow --with numpy --with scipy --with scikit-image --with pydantic --with pytest --with typer python -m pytest art/compositor art/pipeline_polish/test_blind_score.py art/pipeline_polish/animal_tests.py art/pipeline_polish/portrait_test.py art/pipeline_polish/item_test.py -q
```

Result: **96 passed in 2.11s**, exit0. The first combined test environment lacked
the existing compositor dependency `typer`; the corrected ephemeral environment
includes it. No repository dependency declaration was added or changed.

All22 changed Python files were enumerated from the base diff plus new allowed
art paths, then checked with the repository's default Ruff configuration:
`uvx --offline ruff check <22 explicit paths>` → all checks passed, exit0.

`uvx --offline basedpyright --pythonpath /opt/homebrew/bin/python3 <22 explicit paths>`
→ **0 errors,14 warnings,0 notes**, exit1. Warnings: compositor10, animal3, item1.
An earlier invocation used an expired temporary uv interpreter and could not
resolve installed libraries; it is not treated as product typing evidence.
The stable-interpreter result above governs the final report even where individual
lane environments reported fewer warnings. No warning suppression was added.

Raw logs: `/Users/rexxa/work/artpolish-data/verification/final_tests.log`,
`final_ruff.log`, `final_types.log`.

Additional final checks:

- Every Markdown target in `docs/v8/artpolish/*.md` resolves locally. The final
  review caught and corrected a repository-relative link-depth error.
- All76 candidate paths plus34 identity-sprite hashes match the final handoff.
- All60 generation tool/raw pairs and214 input-reference hashes match; no failed
  tool calls, no unaccounted reservations, no budget overrun.
- Ten owner PNGs and all changed files satisfy1,500,000bytes per file.
- Only `art/**` and `docs/v8/artpolish/**` enter the delivery. Existing unrelated
  `.gitignore`, `.ignore`, `.omo/`, and `docs/art/` work is excluded.
- Independent wall review,1,278 family assertions, and1,000 item assertions are
  recorded in `INDEPENDENT_REVIEW.md`; owner judgement remains pending.

Game-wide gate, runtime integration, frame-rate testing and human acceptance are
outside this art-only pass and were not run. Final committed blob sizes, scoped
staging and remote SHA are checked again during publication; its raw receipt is
`/Users/rexxa/work/artpolish-data/publication.json`.
