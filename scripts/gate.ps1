$ErrorActionPreference = "Stop"
Write-Host "[gate] repo:" (Get-Location)

git status --porcelain | Out-Host

# Node (pnpm) gate
if (Test-Path "pnpm-lock.yaml") {
  if (!(Get-Command pnpm -ErrorAction SilentlyContinue)) { throw "pnpm not found" }
  pnpm install --frozen-lockfile
  pnpm -s run lint
  Write-Host "[gate] done"
  exit 0
}

# Python (uv) gate
if (Test-Path "uv.lock") {
  if (!(Get-Command uv -ErrorAction SilentlyContinue)) { throw "uv not found (pip install -U uv)" }
  uv sync --frozen --all-extras
  uv run ruff check .
  uv run ruff format --check .
  Write-Host "[gate] done"
  exit 0
}

throw "No pnpm-lock.yaml or uv.lock found. Configure scripts/gate.ps1 for this repo."
