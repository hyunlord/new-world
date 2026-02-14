$ErrorActionPreference = "Stop"
Write-Host "[gate] repo:" (Get-Location)

git status --porcelain | Out-Host

# TODO: 아래는 프로젝트 스택에 맞게 하나만 남기면 됨.
# Node(pnpm) 예시:
if (Test-Path "pnpm-lock.yaml") {
  if (!(Get-Command pnpm -ErrorAction SilentlyContinue)) { throw "pnpm not found" }
  pnpm install --frozen-lockfile
  pnpm -s run lint
  Write-Host "[gate] done"
  exit 0
}

# Python(uv) 예시:
if (Test-Path "uv.lock") {
  if (!(Get-Command uv -ErrorAction SilentlyContinue)) { throw "uv not found (pip install -U uv)" }
  uv sync --frozen --all-extras
  uv run ruff check .
  uv run ruff format --check .
  Write-Host "[gate] done"
  exit 0
}

Write-Host "[gate] No known lockfile found (pnpm-lock.yaml or uv.lock). Configure gate.ps1"
exit 1
