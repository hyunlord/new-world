import json
import os
import uuid
from datetime import datetime, timedelta
from typing import Optional

from fastapi import FastAPI, Depends, HTTPException, WebSocket, WebSocketDisconnect, Query
from fastapi.middleware.cors import CORSMiddleware
from sqlalchemy import func
from sqlalchemy.orm import Session

import sqlite3
from database import engine, Base, get_db, DB_PATH
from models import Ticket, TicketLog, TicketEvent, Batch
from schemas import TicketCreate, TicketUpdate, LogCreate, DiffCreate, BatchCreate, BatchUpdate
from websocket_manager import ConnectionManager

# ---------------------------------------------------------------------------
# App setup
# ---------------------------------------------------------------------------

app = FastAPI(title="Kanban Backend")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

Base.metadata.create_all(bind=engine)


def migrate_db():
    """Add missing columns to existing tables."""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    cursor.execute("PRAGMA table_info(tickets)")
    columns = [col[1] for col in cursor.fetchall()]

    migrations = {
        "body": "ALTER TABLE tickets ADD COLUMN body TEXT",
        "retry_of": "ALTER TABLE tickets ADD COLUMN retry_of TEXT",
        "retry_count": "ALTER TABLE tickets ADD COLUMN retry_count INTEGER DEFAULT 0",
        "commit_hash": "ALTER TABLE tickets ADD COLUMN commit_hash TEXT",
        "commit_url": "ALTER TABLE tickets ADD COLUMN commit_url TEXT",
        "dismissed": "ALTER TABLE tickets ADD COLUMN dismissed INTEGER DEFAULT 0",
    }

    for col_name, sql in migrations.items():
        if col_name not in columns:
            cursor.execute(sql)

    conn.commit()
    conn.close()


migrate_db()

manager = ConnectionManager()

GITHUB_REPO = os.environ.get("GITHUB_REPO_URL", "https://github.com/hyunlord/new-world")

# ---------------------------------------------------------------------------
# Helper converters
# ---------------------------------------------------------------------------

def ticket_to_dict(ticket: Ticket) -> dict:
    files = ticket.files
    if isinstance(files, str):
        try:
            files = json.loads(files)
        except (ValueError, TypeError):
            files = []
    if files is None:
        files = []

    dependencies = ticket.dependencies
    if isinstance(dependencies, str):
        try:
            dependencies = json.loads(dependencies)
        except (ValueError, TypeError):
            dependencies = []
    if dependencies is None:
        dependencies = []

    return {
        "id": ticket.id,
        "title": ticket.title,
        "description": ticket.description,
        "status": ticket.status,
        "priority": ticket.priority,
        "assignee": ticket.assignee,
        "system": ticket.system,
        "files": files,
        "dependencies": dependencies,
        "dispatch_method": ticket.dispatch_method,
        "batch_id": ticket.batch_id,
        "created_by": ticket.created_by,
        "ticket_number": ticket.ticket_number,
        "branch": ticket.branch,
        "diff_summary": ticket.diff_summary,
        "diff_full": ticket.diff_full,
        "body": ticket.body,
        "retry_of": ticket.retry_of,
        "retry_count": ticket.retry_count or 0,
        "commit_hash": ticket.commit_hash,
        "commit_url": ticket.commit_url,
        "dismissed": bool(ticket.dismissed) if ticket.dismissed else False,
        "error_message": ticket.error_message,
        "started_at": ticket.started_at.isoformat() + "Z" if ticket.started_at else None,
        "completed_at": ticket.completed_at.isoformat() + "Z" if ticket.completed_at else None,
        "created_at": ticket.created_at.isoformat() + "Z" if ticket.created_at else None,
        "updated_at": ticket.updated_at.isoformat() + "Z" if ticket.updated_at else None,
    }


def log_to_dict(log: TicketLog) -> dict:
    return {
        "id": log.id,
        "ticket_id": log.ticket_id,
        "timestamp": log.timestamp.isoformat() + "Z" if log.timestamp else None,
        "level": log.level,
        "message": log.message,
        "source": log.source,
    }


def event_to_dict(event: TicketEvent) -> dict:
    return {
        "id": event.id,
        "ticket_id": event.ticket_id,
        "timestamp": event.timestamp.isoformat() + "Z" if event.timestamp else None,
        "event_type": event.event_type,
        "old_value": event.old_value,
        "new_value": event.new_value,
        "actor": event.actor,
    }


def batch_to_dict(batch: Batch) -> dict:
    return {
        "id": batch.id,
        "title": batch.title,
        "description": batch.description,
        "source_prompt": batch.source_prompt,
        "total_tickets": batch.total_tickets,
        "completed_tickets": batch.completed_tickets,
        "status": batch.status,
        "created_at": batch.created_at.isoformat() + "Z" if batch.created_at else None,
        "updated_at": batch.updated_at.isoformat() + "Z" if batch.updated_at else None,
    }


def recalculate_batch_status(db: Session, batch_id: str):
    """Recalculate batch status based on its tickets."""
    batch = db.query(Batch).filter(Batch.id == batch_id).first()
    if not batch:
        return
    tickets = db.query(Ticket).filter(Ticket.batch_id == batch_id).all()
    batch.total_tickets = len(tickets)
    terminal = [t for t in tickets if t.status in ("done", "failed")]
    batch.completed_tickets = len(terminal)
    failed_count = len([t for t in terminal if t.status == "failed"])
    if batch.completed_tickets < batch.total_tickets:
        batch.status = "active"
    elif failed_count > 0:
        batch.status = "partial"
    else:
        batch.status = "completed"
    batch.updated_at = datetime.utcnow()


def calculate_quality_score(tickets: list) -> Optional[int]:
    """Calculate batch prompt quality score (0-100)."""
    if not tickets:
        return None

    done_tickets = [t for t in tickets if t.status == "done"]
    failed_tickets = [t for t in tickets if t.status == "failed"]
    completed_count = len(done_tickets) + len(failed_tickets)

    if completed_count == 0:
        return None

    # 1) Success Rate (40 max)
    success_rate = len(done_tickets) / completed_count
    success_points = success_rate * 40.0

    # 2) Speed (30 max)
    durations_min = [
        (t.completed_at - t.started_at).total_seconds() / 60.0
        for t in done_tickets
        if t.started_at and t.completed_at and t.completed_at >= t.started_at
    ]

    if not durations_min:
        speed_points = 15.0
    else:
        avg_min = sum(durations_min) / len(durations_min)
        if avg_min <= 5:
            speed_points = 30.0
        elif avg_min >= 20:
            speed_points = 0.0
        else:
            speed_points = 30.0 * (1.0 - (avg_min - 5.0) / 15.0)

    # 3) No Retries Bonus (20 max)
    retry_count = len([t for t in tickets if (t.title or "").startswith("[Retry]")])
    retry_ratio = retry_count / len(tickets)
    no_retries_points = 20.0 * (1.0 - retry_ratio)

    # 4) Dispatch Ratio (10 max)
    codex_count = len([t for t in tickets if t.dispatch_method == "codex"])
    dispatch_ratio = codex_count / len(tickets)
    dispatch_points = 10.0 * min(dispatch_ratio / 0.6, 1.0)

    total_score = success_points + speed_points + no_retries_points + dispatch_points
    return int(round(max(0.0, min(100.0, total_score))))


# ---------------------------------------------------------------------------
# REST Endpoints
# ---------------------------------------------------------------------------

# ---------------------------------------------------------------------------
# Batch Endpoints
# ---------------------------------------------------------------------------

@app.post("/api/batches")
async def create_batch(body: BatchCreate, db: Session = Depends(get_db)):
    now = datetime.utcnow()
    batch = Batch(
        id=str(uuid.uuid4()),
        title=body.title,
        description=body.description,
        source_prompt=body.source_prompt,
        status="active",
        total_tickets=0,
        completed_tickets=0,
        created_at=now,
        updated_at=now,
    )
    db.add(batch)
    db.commit()
    db.refresh(batch)
    d = batch_to_dict(batch)
    await manager.broadcast("batch_created", None, d)
    return d


@app.get("/api/batches")
def list_batches(
    status: Optional[str] = Query(None),
    search: Optional[str] = Query(None),
    date_from: Optional[str] = Query(None),
    date_to: Optional[str] = Query(None),
    sort: str = Query("created_at"),
    order: str = Query("desc"),
    limit: int = Query(20),
    offset: int = Query(0),
    db: Session = Depends(get_db),
):
    q = db.query(Batch)
    if status:
        statuses = [s.strip() for s in status.split(",") if s.strip()]
        if statuses:
            q = q.filter(Batch.status.in_(statuses))
    if search:
        pattern = f"%{search}%"
        q = q.filter(Batch.title.like(pattern))
    if date_from:
        try:
            q = q.filter(Batch.created_at >= datetime.fromisoformat(date_from))
        except ValueError:
            pass
    if date_to:
        try:
            q = q.filter(Batch.created_at <= datetime.fromisoformat(date_to))
        except ValueError:
            pass
    total = q.count()
    sort_col = getattr(Batch, sort, Batch.created_at)
    if order == "asc":
        q = q.order_by(sort_col.asc())
    else:
        q = q.order_by(sort_col.desc())
    batches = q.offset(offset).limit(limit).all()

    result_batches = []
    for b in batches:
        d = batch_to_dict(b)
        tickets = db.query(Ticket).filter(Ticket.batch_id == b.id).all()
        d["quality_score"] = calculate_quality_score(tickets) if tickets else None
        result_batches.append(d)

    return {
        "total": total,
        "batches": result_batches,
        "limit": limit,
        "offset": offset,
    }


@app.get("/api/batches/{batch_id}")
def get_batch(batch_id: str, db: Session = Depends(get_db)):
    batch = db.query(Batch).filter(Batch.id == batch_id).first()
    if not batch:
        raise HTTPException(status_code=404, detail="Batch not found")
    tickets = (
        db.query(Ticket)
        .filter(Ticket.batch_id == batch_id)
        .order_by(Ticket.ticket_number.asc())
        .all()
    )
    result = batch_to_dict(batch)
    result["tickets"] = [ticket_to_dict(t) for t in tickets]
    codex_count = len([t for t in tickets if t.dispatch_method == "codex"])
    direct_count = len([t for t in tickets if t.dispatch_method == "direct"])
    result["dispatch_ratio"] = {
        "codex": codex_count,
        "direct": direct_count,
        "percentage": round(codex_count / len(tickets) * 100) if tickets else 0,
    }
    result["quality_score"] = calculate_quality_score(tickets)
    return result


@app.patch("/api/batches/{batch_id}")
async def update_batch(
    batch_id: str, body: BatchUpdate, db: Session = Depends(get_db)
):
    batch = db.query(Batch).filter(Batch.id == batch_id).first()
    if not batch:
        raise HTTPException(status_code=404, detail="Batch not found")
    data = body.model_dump(exclude_unset=True)
    for field, value in data.items():
        setattr(batch, field, value)
    batch.updated_at = datetime.utcnow()
    db.commit()
    db.refresh(batch)
    d = batch_to_dict(batch)
    await manager.broadcast("batch_updated", None, d)
    return d


@app.post("/api/tickets")
async def create_ticket(body: TicketCreate, db: Session = Depends(get_db)):
    now = datetime.utcnow()
    ticket_id = body.id if body.id else str(uuid.uuid4())

    ticket = Ticket(
        id=ticket_id,
        title=body.title,
        description=body.description,
        body=body.body,
        status="todo",
        priority=body.priority or "medium",
        system=body.system,
        files=json.dumps(body.files if body.files is not None else []),
        dependencies=json.dumps(body.dependencies if body.dependencies is not None else []),
        dispatch_method=body.dispatch_method or "codex",
        batch_id=body.batch_id,
        created_by=body.created_by or "manual",
        ticket_number=body.ticket_number,
        branch=body.branch,
        created_at=now,
        updated_at=now,
    )
    db.add(ticket)
    db.commit()
    db.refresh(ticket)

    if ticket.batch_id:
        batch = db.query(Batch).filter(Batch.id == ticket.batch_id).first()
        if batch:
            batch.total_tickets += 1
            batch.updated_at = now
            db.commit()

    d = ticket_to_dict(ticket)
    await manager.broadcast("ticket_created", ticket.id, d)
    return d


@app.get("/api/tickets")
def list_tickets(
    status: Optional[str] = Query(None),
    system: Optional[str] = Query(None),
    assignee: Optional[str] = Query(None),
    batch_id: Optional[str] = Query(None),
    created_by: Optional[str] = Query(None),
    search: Optional[str] = Query(None),
    date_from: Optional[str] = Query(None),
    date_to: Optional[str] = Query(None),
    sort: str = Query("created_at"),
    order: str = Query("desc"),
    limit: int = Query(50),
    offset: int = Query(0),
    include_dismissed: bool = Query(False),
    db: Session = Depends(get_db),
):
    q = db.query(Ticket)
    if not include_dismissed:
        q = q.filter((Ticket.dismissed == 0) | (Ticket.dismissed == None))

    if status:
        statuses = [s.strip() for s in status.split(",") if s.strip()]
        if statuses:
            q = q.filter(Ticket.status.in_(statuses))

    if system:
        q = q.filter(Ticket.system == system)

    if assignee:
        q = q.filter(Ticket.assignee == assignee)

    if batch_id:
        q = q.filter(Ticket.batch_id == batch_id)

    if created_by:
        q = q.filter(Ticket.created_by == created_by)

    if search:
        pattern = f"%{search}%"
        q = q.filter(
            (Ticket.title.like(pattern)) | (Ticket.description.like(pattern))
        )

    if date_from:
        try:
            dt_from = datetime.fromisoformat(date_from)
            q = q.filter(Ticket.created_at >= dt_from)
        except ValueError:
            pass

    if date_to:
        try:
            dt_to = datetime.fromisoformat(date_to)
            q = q.filter(Ticket.created_at <= dt_to)
        except ValueError:
            pass

    total = q.count()

    # Sorting
    sort_col = getattr(Ticket, sort, Ticket.created_at)
    if order == "asc":
        q = q.order_by(sort_col.asc())
    else:
        q = q.order_by(sort_col.desc())

    tickets = q.offset(offset).limit(limit).all()

    return {
        "total": total,
        "tickets": [ticket_to_dict(t) for t in tickets],
        "limit": limit,
        "offset": offset,
    }


@app.get("/api/tickets/{ticket_id}")
def get_ticket(ticket_id: str, db: Session = Depends(get_db)):
    ticket = db.query(Ticket).filter(Ticket.id == ticket_id).first()
    if not ticket:
        raise HTTPException(status_code=404, detail="Ticket not found")

    logs = (
        db.query(TicketLog)
        .filter(TicketLog.ticket_id == ticket_id)
        .order_by(TicketLog.timestamp.asc())
        .all()
    )
    events = (
        db.query(TicketEvent)
        .filter(TicketEvent.ticket_id == ticket_id)
        .order_by(TicketEvent.timestamp.asc())
        .all()
    )

    d = ticket_to_dict(ticket)
    d["logs"] = [log_to_dict(l) for l in logs]
    d["events"] = [event_to_dict(e) for e in events]
    return d


@app.patch("/api/tickets/{ticket_id}")
async def update_ticket(
    ticket_id: str, body: TicketUpdate, db: Session = Depends(get_db)
):
    ticket = db.query(Ticket).filter(Ticket.id == ticket_id).first()
    if not ticket:
        raise HTTPException(status_code=404, detail="Ticket not found")

    now = datetime.utcnow()
    data = body.model_dump(exclude_unset=True)

    old_status = ticket.status
    old_assignee = ticket.assignee

    # Apply fields
    for field, value in data.items():
        if field == "files":
            setattr(ticket, field, json.dumps(value if value is not None else []))
        elif field == "dependencies":
            setattr(ticket, field, json.dumps(value if value is not None else []))
        else:
            setattr(ticket, field, value)

    if "commit_hash" in data and data["commit_hash"]:
        ticket.commit_url = f"{GITHUB_REPO}/commit/{data['commit_hash']}"
    if "dismissed" in data and data["dismissed"] is not None:
        ticket.dismissed = 1 if data["dismissed"] else 0

    ticket.updated_at = now

    # Status change events and timestamp updates
    new_status = ticket.status
    if "status" in data and new_status != old_status:
        if new_status == "in_progress" and ticket.started_at is None:
            ticket.started_at = now
        if new_status in ("done", "failed"):
            ticket.completed_at = now

        db.add(TicketEvent(
            ticket_id=ticket_id,
            timestamp=now,
            event_type="status_change",
            old_value=old_status,
            new_value=new_status,
            actor="system",
        ))

    # Assignee change event
    new_assignee = ticket.assignee
    if "assignee" in data and new_assignee != old_assignee:
        db.add(TicketEvent(
            ticket_id=ticket_id,
            timestamp=now,
            event_type="assignee_change",
            old_value=old_assignee or "",
            new_value=new_assignee or "",
            actor="system",
        ))

    db.commit()
    db.refresh(ticket)

    if ticket.batch_id and "status" in data:
        recalculate_batch_status(db, ticket.batch_id)
        db.commit()
        await manager.broadcast("batch_updated", None,
            batch_to_dict(db.query(Batch).filter(Batch.id == ticket.batch_id).first()))

    d = ticket_to_dict(ticket)
    await manager.broadcast("ticket_updated", ticket.id, d)
    return d


@app.post("/api/tickets/clear")
def clear_all_tickets(db: Session = Depends(get_db)):
    db.query(TicketLog).delete()
    db.query(TicketEvent).delete()
    db.query(Ticket).delete()
    db.query(Batch).update({
        Batch.total_tickets: 0,
        Batch.completed_tickets: 0,
        Batch.status: "active",
    })
    db.commit()
    return {"ok": True, "message": "All tickets cleared"}


@app.delete("/api/tickets/{ticket_id}")
def delete_ticket(ticket_id: str, db: Session = Depends(get_db)):
    ticket = db.query(Ticket).filter(Ticket.id == ticket_id).first()
    if not ticket:
        raise HTTPException(status_code=404, detail="Ticket not found")
    batch_id = ticket.batch_id
    db.delete(ticket)
    db.commit()
    if batch_id:
        recalculate_batch_status(db, batch_id)
        db.commit()
    return {"ok": True}


@app.post("/api/tickets/{ticket_id}/logs")
async def add_log(
    ticket_id: str, body: LogCreate, db: Session = Depends(get_db)
):
    ticket = db.query(Ticket).filter(Ticket.id == ticket_id).first()
    if not ticket:
        raise HTTPException(status_code=404, detail="Ticket not found")

    log = TicketLog(
        ticket_id=ticket_id,
        timestamp=datetime.utcnow(),
        level=body.level,
        message=body.message,
        source=body.source,
    )
    db.add(log)
    db.commit()
    db.refresh(log)

    d = log_to_dict(log)
    await manager.broadcast("log_added", ticket_id, d)
    return d


@app.post("/api/tickets/{ticket_id}/diff")
def save_diff(
    ticket_id: str, body: DiffCreate, db: Session = Depends(get_db)
):
    ticket = db.query(Ticket).filter(Ticket.id == ticket_id).first()
    if not ticket:
        raise HTTPException(status_code=404, detail="Ticket not found")

    ticket.diff_summary = body.diff_summary
    ticket.diff_full = body.diff_full
    ticket.updated_at = datetime.utcnow()
    db.commit()
    db.refresh(ticket)
    return ticket_to_dict(ticket)


@app.get("/api/tickets/{ticket_id}/logs")
def get_ticket_logs(ticket_id: str, db: Session = Depends(get_db)):
    logs = (
        db.query(TicketLog)
        .filter(TicketLog.ticket_id == ticket_id)
        .order_by(TicketLog.timestamp.asc())
        .all()
    )
    return [log_to_dict(l) for l in logs]


@app.get("/api/stats")
def get_stats(db: Session = Depends(get_db)):
    total = db.query(Ticket).count()
    active = db.query(Ticket).filter(Ticket.status.in_(["claimed", "in_progress"])).count()
    done = db.query(Ticket).filter(Ticket.status == "done").count()
    failed = db.query(Ticket).filter(Ticket.status == "failed").count()

    completed = done + failed
    rate = int(done / completed * 100) if completed > 0 else 0

    # Systems breakdown
    rows = (
        db.query(Ticket.system, func.count(Ticket.id))
        .filter(Ticket.system.isnot(None))
        .group_by(Ticket.system)
        .all()
    )
    systems = {row[0]: row[1] for row in rows}

    # Average duration for done tickets that have both timestamps
    done_tickets = (
        db.query(Ticket)
        .filter(
            Ticket.status == "done",
            Ticket.started_at.isnot(None),
            Ticket.completed_at.isnot(None),
        )
        .all()
    )
    avg_duration: Optional[float] = None
    if done_tickets:
        durations = [
            (t.completed_at - t.started_at).total_seconds()
            for t in done_tickets
            if t.completed_at >= t.started_at
        ]
        if durations:
            avg_duration = sum(durations) / len(durations)

    # --- Phase 2: Extended stats ---

    # By created_by
    by_created_by = {}
    for creator in ["claude_code", "codex", "manual"]:
        creator_tickets = db.query(Ticket).filter(Ticket.created_by == creator).all()
        if creator_tickets:
            c_done = len([t for t in creator_tickets if t.status == "done"])
            c_failed = len([t for t in creator_tickets if t.status == "failed"])
            by_created_by[creator] = {
                "total": len(creator_tickets),
                "done": c_done,
                "failed": c_failed,
            }

    # By dispatch_method
    by_dispatch_method = {}
    for method in ["codex", "direct"]:
        method_tickets = db.query(Ticket).filter(Ticket.dispatch_method == method).all()
        if method_tickets:
            m_done = len([t for t in method_tickets if t.status == "done"])
            m_failed = len([t for t in method_tickets if t.status == "failed"])
            m_durations = [
                (t.completed_at - t.started_at).total_seconds()
                for t in method_tickets
                if t.status == "done" and t.started_at and t.completed_at and t.completed_at >= t.started_at
            ]
            by_dispatch_method[method] = {
                "total": len(method_tickets),
                "done": m_done,
                "failed": m_failed,
                "avg_duration": round(sum(m_durations) / len(m_durations), 1) if m_durations else None,
            }

    # Dispatch ratio
    codex_total = len(db.query(Ticket).filter(Ticket.dispatch_method == "codex").all())
    dispatch_ratio = round(codex_total / total * 100) if total > 0 else 0

    # Batch counts
    active_batches = db.query(Batch).filter(Batch.status == "active").count()
    total_batches = db.query(Batch).count()

    return {
        "total": total,
        "active": active,
        "done": done,
        "failed": failed,
        "rate": rate,
        "systems": systems,
        "avg_duration_seconds": avg_duration,
        "by_created_by": by_created_by,
        "by_dispatch_method": by_dispatch_method,
        "dispatch_ratio": dispatch_ratio,
        "active_batches": active_batches,
        "total_batches": total_batches,
    }


@app.get("/api/stats/errors")
def get_error_stats(days: int = Query(30), db: Session = Depends(get_db)):
    cutoff = datetime.utcnow() - timedelta(days=days)

    failed_tickets = (
        db.query(Ticket)
        .filter(
            Ticket.status == "failed",
            Ticket.error_message.isnot(None),
            Ticket.created_at >= cutoff,
        )
        .all()
    )

    error_logs = (
        db.query(TicketLog)
        .filter(TicketLog.level == "error", TicketLog.timestamp >= cutoff)
        .all()
    )

    keyword_counts = {}
    all_errors = []

    for t in failed_tickets:
        all_errors.append({
            "ticket_id": t.id,
            "ticket_title": t.title,
            "error": t.error_message,
            "date": t.completed_at.isoformat() + "Z" if t.completed_at else None,
            "system": t.system,
        })
        for word in t.error_message.lower().split():
            if len(word) >= 5:
                keyword_counts[word] = keyword_counts.get(word, 0) + 1

    for log in error_logs:
        for word in log.message.lower().split():
            if len(word) >= 5:
                keyword_counts[word] = keyword_counts.get(word, 0) + 1

    patterns = [
        {"keyword": k, "count": v}
        for k, v in sorted(keyword_counts.items(), key=lambda x: -x[1])
        if v >= 2
    ][:20]

    system_failures = {}
    for t in failed_tickets:
        sys_name = t.system or "unknown"
        system_failures[sys_name] = system_failures.get(sys_name, 0) + 1

    return {
        "total_failures": len(failed_tickets),
        "total_error_logs": len(error_logs),
        "patterns": patterns,
        "system_failures": system_failures,
        "recent_errors": all_errors[:20],
    }


@app.get("/api/stats/daily")
def get_daily_stats(days: int = Query(30), db: Session = Depends(get_db)):
    cutoff = datetime.utcnow() - timedelta(days=days)

    done_tickets = (
        db.query(Ticket)
        .filter(
            Ticket.status == "done",
            Ticket.completed_at >= cutoff,
            Ticket.completed_at.isnot(None),
        )
        .all()
    )
    failed_tickets = (
        db.query(Ticket)
        .filter(
            Ticket.status == "failed",
            Ticket.completed_at >= cutoff,
            Ticket.completed_at.isnot(None),
        )
        .all()
    )

    daily: dict = {}

    for t in done_tickets:
        day = t.completed_at.strftime("%Y-%m-%d")
        if day not in daily:
            daily[day] = {"done": 0, "failed": 0}
        daily[day]["done"] += 1

    for t in failed_tickets:
        day = t.completed_at.strftime("%Y-%m-%d")
        if day not in daily:
            daily[day] = {"done": 0, "failed": 0}
        daily[day]["failed"] += 1

    return {"days": days, "daily": daily}


# ---------------------------------------------------------------------------
# Batch Compare & Agent Stats & Retry
# ---------------------------------------------------------------------------


@app.get("/api/stats/batch-compare")
def batch_compare(ids: str = Query(...), db: Session = Depends(get_db)):
    batch_ids = [batch_id.strip() for batch_id in ids.split(",") if batch_id.strip()]
    if len(batch_ids) < 2 or len(batch_ids) > 5:
        raise HTTPException(status_code=400, detail="2-5 batch IDs required")

    result = []

    for batch_id in batch_ids:
        batch = db.query(Batch).filter(Batch.id == batch_id).first()
        if not batch:
            continue

        tickets = db.query(Ticket).filter(Ticket.batch_id == batch_id).all()
        total_tickets = len(tickets)
        done_tickets = [t for t in tickets if t.status == "done"]
        failed_tickets = [t for t in tickets if t.status == "failed"]

        done_count = len(done_tickets)
        failed_count = len(failed_tickets)
        completed_count = done_count + failed_count
        success_rate = round(done_count / completed_count * 100) if completed_count > 0 else 0

        durations = [
            (t.completed_at - t.started_at).total_seconds()
            for t in done_tickets
            if t.started_at and t.completed_at and t.completed_at >= t.started_at
        ]
        avg_duration_seconds = (sum(durations) / len(durations)) if durations else None
        max_duration_seconds = max(durations) if durations else None
        min_duration_seconds = min(durations) if durations else None

        codex_count = len([t for t in tickets if t.dispatch_method == "codex"])
        dispatch_ratio = round(codex_count / total_tickets * 100) if total_tickets > 0 else 0

        result.append({
            "batch_id": batch.id,
            "title": batch.title,
            "total_tickets": total_tickets,
            "done": done_count,
            "failed": failed_count,
            "success_rate": success_rate,
            "avg_duration_seconds": avg_duration_seconds,
            "max_duration_seconds": max_duration_seconds,
            "min_duration_seconds": min_duration_seconds,
            "dispatch_ratio": dispatch_ratio,
            "created_at": batch.created_at.isoformat() + "Z" if batch.created_at else None,
        })

    return {"batches": result}


@app.get("/api/stats/agents")
def get_agent_stats(db: Session = Depends(get_db)):
    tickets = db.query(Ticket).filter(Ticket.assignee.isnot(None)).all()

    by_agent = {}
    for t in tickets:
        agent = t.assignee
        if agent not in by_agent:
            by_agent[agent] = {
                "agent": agent,
                "total": 0,
                "done": 0,
                "failed": 0,
                "in_progress": 0,
                "_durations": [],
            }

        row = by_agent[agent]
        row["total"] += 1

        if t.status == "done":
            row["done"] += 1
            if (
                t.started_at is not None
                and t.completed_at is not None
                and t.completed_at >= t.started_at
            ):
                row["_durations"].append((t.completed_at - t.started_at).total_seconds())
        elif t.status == "failed":
            row["failed"] += 1
        elif t.status in ("claimed", "in_progress"):
            row["in_progress"] += 1

    agents = []
    for row in by_agent.values():
        completed = row["done"] + row["failed"]
        durations = row.pop("_durations")
        row["success_rate"] = round(row["done"] / completed * 100) if completed > 0 else 0
        row["avg_duration_seconds"] = (
            sum(durations) / len(durations) if durations else None
        )
        agents.append(row)

    agents.sort(key=lambda x: x["total"], reverse=True)
    return {"agents": agents}


@app.post("/api/tickets/{ticket_id}/retry")
async def retry_ticket(ticket_id: str, db: Session = Depends(get_db)):
    original = db.query(Ticket).filter(Ticket.id == ticket_id).first()
    if not original:
        raise HTTPException(status_code=404, detail="Ticket not found")
    if original.status != "failed":
        raise HTTPException(status_code=400, detail="Only failed tickets can be retried")

    now = datetime.utcnow()
    retry_body = None
    if original.body or original.error_message:
        retry_body = (
            f"[RETRY] Previous attempt failed with: "
            f"{original.error_message or 'Unknown error'}\n\n"
            f"{original.body or ''}"
        )

    new_ticket = Ticket(
        id=str(uuid.uuid4()),
        title=f"[Retry] {original.title}",
        description=original.description,
        body=retry_body,
        status="todo",
        priority=original.priority,
        system=original.system,
        files=original.files,
        dependencies=original.dependencies,
        dispatch_method=original.dispatch_method,
        batch_id=original.batch_id,
        created_by="manual",
        ticket_number=original.ticket_number,
        branch=original.branch,
        retry_of=ticket_id,
        retry_count=0,
        created_at=now,
        updated_at=now,
    )

    db.add(new_ticket)
    db.add(
        TicketEvent(
            ticket_id=ticket_id,
            timestamp=now,
            event_type="retried",
            old_value=ticket_id,
            new_value=new_ticket.id,
            actor="manual",
        )
    )
    original.retry_count = (original.retry_count or 0) + 1

    db.commit()
    db.refresh(new_ticket)

    if new_ticket.batch_id:
        recalculate_batch_status(db, new_ticket.batch_id)
        db.commit()

    d = ticket_to_dict(new_ticket)
    await manager.broadcast("ticket_created", new_ticket.id, d)
    return d


# ---------------------------------------------------------------------------
# WebSocket
# ---------------------------------------------------------------------------

@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    await manager.connect(websocket)
    try:
        while True:
            await websocket.receive_text()
    except WebSocketDisconnect:
        manager.disconnect(websocket)
    except Exception:
        manager.disconnect(websocket)
