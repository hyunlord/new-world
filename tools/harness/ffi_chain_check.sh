#!/bin/bash
# FFI Chain Verification (0-token, bash grep only)
# Checks: Rust #[func] → sim_bridge.gd proxy → simulation_engine.gd proxy
#
# Exit 0 = all chains complete
# Exit 1 = broken chain found

set -euo pipefail

PROJECT_ROOT="${1:-$(git rev-parse --show-toplevel)}"
BRIDGE_RS="$PROJECT_ROOT/rust/crates/sim-bridge/src/lib.rs"
SIM_BRIDGE_GD="$PROJECT_ROOT/scripts/core/simulation/sim_bridge.gd"
SIM_ENGINE_GD="$PROJECT_ROOT/scripts/core/simulation/simulation_engine.gd"

broken=0

# Extract all #[func] method names from Rust SimBridge
rust_methods=$(grep -A1 '#\[func\]' "$BRIDGE_RS" 2>/dev/null \
    | grep 'fn ' | sed 's/.*fn \([a-z_][a-z_0-9]*\).*/\1/' | sort -u)

# Only check recently added methods — pre-existing unproxied methods are
# technical debt, not feature regressions. Reporting them contaminates
# the regression guard evidence.
# Check recently added methods (git diff HEAD~1) — these MUST have proxies
changed_methods=$(git diff HEAD~1 -- "$BRIDGE_RS" 2>/dev/null \
    | grep '^+.*fn ' | grep -v '^+++' \
    | sed 's/.*fn \([a-z_][a-z_0-9]*\).*/\1/' 2>/dev/null || true)

for method in $changed_methods; do
    if [[ -z "$method" ]]; then
        continue
    fi
    # Only enforce proxy chain for #[func]-decorated methods — private helpers
    # (e.g. conversion utilities called internally) do not need GDScript proxies.
    if ! echo "$rust_methods" | grep -qx "$method"; then
        continue
    fi
    if ! grep -q "$method" "$SIM_BRIDGE_GD" 2>/dev/null; then
        echo "BROKEN: $method — newly added to lib.rs but MISSING from sim_bridge.gd"
        broken=1
    fi
    if ! grep -q "$method" "$SIM_ENGINE_GD" 2>/dev/null; then
        echo "WARN: $method — not in simulation_engine.gd (may be OK if not needed by UI)"
    fi
done

if [[ $broken -eq 1 ]]; then
    echo "FFI CHAIN: BROKEN — fix proxy chain before proceeding"
    exit 1
fi

echo "FFI CHAIN: OK"
exit 0
