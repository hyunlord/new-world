# Kanban Extension Progress

## Part 2: Features 6-10 — Kanban Extension Part 2

### Context
Adding 5 new features to the kanban board: batch comparison, agent monitoring, auto-retry, quality scoring, and GitHub integration.

### Tickets
| Ticket | Title | Action | Dispatch Tool | Reason |
|--------|-------|--------|---------------|--------|
| T01 | Batch Compare API | DISPATCH | ask_codex | standalone endpoint |
| T02 | Batch Compare UI | DISPATCH | ask_codex | new component + api |
| T03 | Agent Monitoring API | DISPATCH | ask_codex | standalone endpoint |
| T04 | Agent Monitoring UI | DISPATCH | ask_codex | new component + nav |
| T05 | Auto-Retry Backend | DISPATCH | ask_codex | standalone endpoint |
| T06 | Auto-Retry UI | DISPATCH | ask_codex | button + api function |
| T07 | Quality Score Backend | DISPATCH | ask_codex | standalone function + api mod |
| T08 | Quality Score UI | DISPATCH | ask_codex | component modification |
| T09 | GitHub Integration Backend | DISPATCH | ask_codex | model + api + script |
| T10 | GitHub Integration UI | DISPATCH | ask_codex | component modifications |
| T11 | DB Migration | DIRECT | — | shared schema, migrate_db() |
| T12 | Integration Test + Build | DIRECT | — | integration wiring + verification |

### Dispatch ratio: 10/12 = 83%

### Dispatch strategy
- Wave 1 (parallel): T01, T03, T05, T07, T09 — all backend, no file overlap
- Wave 2 (parallel): T02, T04, T06, T08, T10 — all frontend, dispatched after Wave 1
- T11 DIRECT: DB migration (shared schema changes in migrate_db, models.py, schemas.py)
- T12 DIRECT: docker-compose rebuild + smoke tests

### Results
- Gate: PASS (docker-compose up --build, all APIs responding, frontend serving)
- Dispatch ratio: 10/12 = 83%
- Dispatch tool used: ask_codex (10 tickets, agent_role: executor)
- Files changed: 10
  - main.py (backend — 5 new endpoints + quality_score function + commit_url logic)
  - models.py (backend — 4 new columns)
  - schemas.py (backend — 4 new response fields + commit_hash update field)
  - kanban_helpers.sh (commit_hash in kanban_done)
  - BatchCompare.jsx (NEW — batch comparison with charts)
  - AgentView.jsx (NEW — agent performance table + charts)
  - App.jsx (2 new routes)
  - TopBar.jsx (Agents nav item)
  - api.js (3 new functions)
  - TicketDetail.jsx (retry button + commit link)
  - BatchView.jsx (ScoreBadge + quality score breakdown)
  - HistoryTable.jsx (commit column)

### Smoke Test Results
- Batch Compare API: returns empty for non-existent IDs, 400 for <2 IDs
- Agent Stats API: returns agent array
- Retry: creates "[Retry] ..." ticket with retry_of populated
- Quality Score: present in batch list/detail APIs
- Commit Hash: auto-generates commit_url from GITHUB_REPO env var
- Frontend: serving on :3100
