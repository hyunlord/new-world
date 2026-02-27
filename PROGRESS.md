# Phase: Debug Log Flag Control

## Classification Table

| Ticket | Description | 🟢/🔴 | Tool | Status |
|--------|-------------|--------|------|--------|
| T1 | GameConfig DEBUG_* 플래그 6개 추가 | 🔴 DIRECT | — | ✅ Done |
| T2 | stress_system — DEBUG_STRESS_LOG 플래그 적용 | 🟢 DISPATCH | executor | ✅ Done |
| T3 | mental_break_system — DEBUG_MENTAL_BREAK_LOG 플래그 적용 | 🟢 DISPATCH | executor | ✅ Done |
| T4 | trauma_scar_system — DEBUG_TRAUMA_LOG 플래그 적용 | 🟢 DISPATCH | executor | ✅ Done |
| T5 | trait_violation_system — DEBUG_TRAIT_VIOLATION_LOG 플래그 적용 | 🟢 DISPATCH | executor | ✅ Done |
| T6 | mortality_system + family_system — DEBUG_DEMOGRAPHY_LOG 플래그 적용 | 🟢 DISPATCH | executor | ✅ Done |
| T7 | main.gd _log_balance() — DEBUG_BALANCE_LOG 플래그 적용 | 🟢 DISPATCH | executor | ✅ Done |

**Dispatch ratio: 6/7 = 86% ✅**

## Commit
`c96920f` — perf(debug): gate all per-tick print() calls behind DEBUG_* flags
