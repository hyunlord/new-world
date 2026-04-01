---
name: kanban-workflow
description: |
  Kanban board integration, Codex dispatch rules, autopilot/ultrapilot workflow,
  ticket management, and batch lifecycle. Use when dispatching tickets via
  codex MCP tool, creating batches, running autopilot workflow, or managing
  progress tracker files and ticket templates.
  Updated for Rust-first development: all tickets dispatched via codex MCP tool.
---

# Kanban & Dispatch Workflow — SKILL.md

> This skill covers kanban board integration, Codex dispatch rules, autopilot workflow, and common workflow mistakes.
> For code conventions and project architecture → see CLAUDE.md
> For localization and Rust coding standards → see .claude/skills/worldsim-code/SKILL.md

---

## Part 1: Kanban Board Integration

(Unchanged from original — see existing kanban-workflow/SKILL.md for full kanban API details)

Key points:
- Kanban server: `http://localhost:8800`
- Batch creation: `kanban_create_batch`
- Ticket creation: `kanban_create_ticket`
- One dispatch path: `codex` MCP tool (manual kanban update required)

---

## Part 2: Codex Dispatch Rules

### ⚠️ DISPATCH TOOL ROUTING [ABSOLUTE RULE]

**✅ VALID Codex dispatch methods:**
- `codex` MCP tool — ALL tickets (Rust, GDScript, any language)
  - `approval-policy: "never"`, `sandbox: "workspace-write"`
- `codex-reply` MCP tool — Continue existing session (follow-up, error fixes)
  - Requires `threadId` from previous `codex` call

**❌ INVALID — NOT Codex dispatch:**
- `Task` tool — counts as DIRECT, not dispatch
- Implementing code yourself — not dispatch
- `ask_codex` — DEPRECATED, do not use
- `codex_dispatch.sh` — DEPRECATED, do not use

### Dispatch Route Selection

```
Ticket ready?
  ↓
Call `codex` MCP tool with full ticket as prompt
  + approval-policy: "never"
  + sandbox: "workspace-write"
  ↓
Codex completes → check result
  ↓
Need follow-up? → Call `codex-reply` with threadId
  ↓
Integration + gate check by Claude Code (lead worktree)
```

모든 언어(Rust, GDScript) 동일하게 `codex` MCP tool 사용.
Mixed 티켓은 여전히 3개로 분리하되, 각각 `codex` MCP tool로 디스패치.

### Default is DISPATCH. DIRECT is the exception.

DIRECT only if ALL THREE:
1. Modifies shared interfaces (EventBus events, ECS component structs, SimBridge API)
2. Pure integration wiring (<50 lines)
3. Cannot be split smaller

**Dispatch ratio MUST be ≥60%.**

---

## Part 3: Autopilot Workflow (Rust-first)

1. **Plan** — Split into tickets. Each ticket targets ONE crate/module or ONE GDScript file.
2. **Sequence** — Rust foundation first, then bridge, then UI:
   ```
   sim-core changes → sim-systems → sim-engine → sim-bridge → GDScript UI
   ```
3. **Classify** — 🟢 DISPATCH (≥60%) or 🔴 DIRECT (shared interfaces only)
4. **Write the progress tracker FIRST**
5. **Dispatch Rust tickets first** — they don't need Godot
6. **Then dispatch GDScript tickets** — may depend on Rust results
7. **Gate** — `cargo test --workspace && bash scripts/gate.sh`
8. **Summarize** — dispatch ratio, tools used, files changed

### Rust-specific dispatch pattern

```
Step 1: 🔴 DIRECT — New ECS component (sim-core, shared type)
Step 2: 🟢 DISPATCH — System logic (sim-systems, pure Rust)
Step 3: 🟢 DISPATCH — Unit tests (same crate)
Step 4: 🔴 DIRECT — EventBus event addition (shared interface)
Step 5: 🟢 DISPATCH — SimBridge getter (sim-bridge)
Step 6: 🟢 DISPATCH — GDScript UI panel update
Step 7: 🔴 DIRECT — Integration verification

Dispatch ratio: 4/7 = 57% → split Step 1 if possible to hit ≥60%
```

---

## Part 4: Progress Tracker — Mandatory Logging

Same format as before, with added fields:

```markdown
| Ticket | Title | Language | Action | Dispatch Tool | Reason |
|--------|-------|----------|--------|---------------|--------|
| t-XXX | Component | Rust | 🔴 DIRECT | — | shared type |
| t-XXX | System | Rust | 🟢 DISPATCH | codex MCP | pure Rust |
| t-XXX | Bridge | Rust | 🟢 DISPATCH | codex MCP | sim-bridge |
| t-XXX | UI panel | GDScript | 🟢 DISPATCH | codex MCP | UI only |
```

---

## Part 5: Ticket Template (Rust)

```markdown
## Objective
[One sentence]

## Crate & Module
- Crate: sim-systems
- Module: src/runtime/psychology.rs

## Non-goals
[Explicitly NOT in scope]

## Scope
Files to create/modify:
- rust/crates/sim-systems/src/runtime/psychology.rs — add coping system
- rust/crates/sim-core/src/config.rs — add COPING_* constants (lead provides values)

## Acceptance Criteria
- [ ] cargo test --workspace: PASS
- [ ] cargo clippy --workspace -- -D warnings: PASS
- [ ] New function has #[cfg(test)] unit tests
- [ ] All pub items have /// doc comments
- [ ] No unwrap() in production code

## Context
[Links to relevant code, formulas, academic references]
```

---

## Part 6: Common Workflow Mistakes

1-12 from original, plus:
13. **Putting Rust logic in GDScript** — all simulation in Rust, GDScript is UI only
14. **Dispatching sim-bridge work without Godot context** — sim-bridge needs gdext, clarify in ticket
15. **Forgetting SimBridge getter** when adding a new Rust system — UI can't see results without it
16. **Mixed-language tickets** — always split Rust and GDScript into separate tickets
17. **Running cargo test on sim-bridge without Godot** — sim-bridge needs Godot headers, test other crates independently
