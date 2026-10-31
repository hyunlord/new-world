# ADR 0055 — Read-only state diagnosis and why-not records

Status: proposed (M-READ Part 1, 2026-10-31).

## Context

The owner's first playtest verdict was "뭘 해야하는 건지 모르겠네" (M-UX). The Feudal Lord Simulator concluded the root problem was causal illegibility, not interaction modality. Story yield probe #647 found 0 chronicle entries with a verified cause chain of depth >= 3. The M-CORE mind evaluates every node's predicates each choice but records only the winner, so "why is she not eating?" has no answer today.

Current legibility surfaces (inspector "지금 · 이유", "왜?" panel, chronicle "→ 왜?", alerts, goals, lessons, steward toasts, tech map, M6a food/winter lines) show what is happening and sometimes why at depth 1, but:
- The mind records only the winning choice, not why alternatives were blocked.
- Recorded parent links in the chronicle rarely form chains deeper than 1 (#647).
- Alerts show thresholds ("food low") without root causes ("because no harvest because winter").
- No single surface synthesizes the most urgent action for the player.

Roadmap principle 2: "보이지 않으면 없다." M7–M9 must land in a game where causes are already readable.

## Decision

### 1. Read-only state diagnosis

Add a diagnosis framework (`sim-core/data/colony/diagnosis.ron`) that evaluates the current simulation state without modifying it. For a selected subject (person, stockpile, hearth, room, settlement), evaluate an ordered list of checks. Each check has predicates (reusing the mind's operator set over named facts), a text key with parameters, a severity (danger/warning/notice/ok), and an optional `next` pointer to form a chain (subject kind + how to pick the next subject, e.g. "the stockpile nearest this person's home").

Fact providers for diagnosis reuse the mind's providers for person facts and add read-only providers for stockpile, hearth, room, and settlement state. A provider may read the world and resources; it never writes, reserves, or draws randomness. This preserves C2 view independence (ADR 0016): diagnosis never changes simulation state.

Evaluation is on demand for a selected subject (bridge read) and once per in-game hour for the settlement (cached in a presentation resource that is not saved and not hashed). The result is the ordered failing checks, each with parameters and the chain to depth <= 5; the first failing check is the headline.

A check can be added by data only in a test copy of `diagnosis.ron`, and it must appear with its text and chain — verified by the data-only test.

### 2. Why-not records from the mind

For each MindChoice, also record up to 2 blocked candidates: nodes in a higher band than the chosen node, or in the same band with a higher score bound, whose `when` predicates failed. Record the node, band, and the first failing predicate (fact, op, value, actual value) with that fact's cause when it has one.

This extends the existing MindChoice record with bounded additional data per person. The why-not records are presentation state: they inform the "왜?" view ("하려던 것: 먹기 — 막힌 이유: 먹을 것 없음(비축 0)") but do not affect the simulation.

### 3. Why this complements recorded cause links

Recorded cause links in the chronicle (the "→ 왜?" chain) trace backwards from an event to its ancestor events. This is valuable when events have a clear causal sequence, but today chains stop at depth 1 because most events do not record their causal ancestors (#647).

State diagnosis takes a different approach: given the current world state, it evaluates a checklist of known problem patterns and follows `next` pointers to build a chain. This works even when the chronicle has no deep links, because it reads live state rather than historical records.

Why-not records bridge the gap between the two: they explain why the mind did not choose an alternative, which is the information needed to answer "why isn't this person doing X?" — a question that neither the chronicle chain nor the current mind summary can answer.

Together, these three mechanisms (diagnosis checks, why-not records, and recorded cause links) give the player three paths to understanding:
- **Diagnosis:** "What is wrong with this subject right now, and why?" — ordered checks with chains.
- **Why-not:** "Why isn't this person doing what I expected?" — blocked alternatives with failing predicates.
- **Chronicle chain:** "What caused this event?" — historical cause links (improved by future passes).

## Alternatives considered

- **Deeper chronicle cause links only:** Would require every effect to record its full causal ancestry. Addresses historical "why did X happen?" but not live "why isn't X happening?" or "what should I do?". Deferred to M7/M10.
- **AI/LLM explanation:** Out of scope for v1.0 (DEPTH_ROADMAP §1). Diagnosis must be deterministic and data-driven.
- **Simulation-modifying diagnosis (e.g. test what would happen if...):** Violates C2 view independence (ADR 0016). Diagnosis must be read-only.

## Consequences

- The diagnosis framework is data-driven (ADR 0033): checks, facts, and subjects are defined in data files, not hardcoded.
- Why-not records add bounded memory per person per choice; C3 memory ceilings must pass.
- The diagnosis + why-not combination should enable the 30-second test: a reviewer looking at the "왜?" panel should be able to name the first two links of the cause chain for each of the six owner questions.
- Future passes (M7 mind, M10 story) can deepen the chronicle cause links independently.

## Date

2026-10-31.

## Source

- [READ_GOAL.md](../v8/read/READ_GOAL.md) §1.3, §2.1, §2.2.
- [DEPTH_ROADMAP.md](../v8/DEPTH_ROADMAP.md) §3 principle 2, §6 row "상태 진단 모델".
- [ADR 0016](0016-simulation-contract.md) C2 view independence.
- [ADR 0033](0033-extensibility-contract.md) content as data.
- [ADR 0046](0046-colony-mind.md) colony mind architecture.
- Feudal Lord Simulator Stage 1 (`docs/plans/2026-08-07-stage-1-make-cause-visible-design.md`).
- GitHub issues: #647 (story yield), #650 (requests).
