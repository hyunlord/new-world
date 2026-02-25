import json
import uuid
from datetime import datetime, timezone
from typing import Optional

from fastapi import APIRouter, Depends, HTTPException, Query, Request
from sqlalchemy.orm import Session

from database import get_db
from models import Ticket, TicketLog, TicketEvent, Batch
from schemas import TicketCreate, TicketUpdate, LogCreate, DiffCreate
from utils import (
    ticket_to_dict, log_to_dict, event_to_dict, batch_to_dict,
    recalculate_batch_status, validate_transition, VALID_TRANSITIONS,
    GITHUB_REPO, MAX_DIFF_SIZE,
)

router = APIRouter(prefix="/api", tags=["tickets"])


@router.post("/tickets")
async def create_ticket(request: Request, body: TicketCreate, db: Session = Depends(get_db)):
    now = datetime.now(timezone.utc)
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

    # Fix race condition: use recalculate instead of manual +1
    if ticket.batch_id:
        recalculate_batch_status(db, ticket.batch_id)
        db.commit()

    d = ticket_to_dict(ticket)
    await request.app.state.ws_manager.broadcast("ticket_created", ticket.id, d)
    return d


@router.get("/tickets")
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


@router.get("/tickets/{ticket_id}")
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


@router.patch("/tickets/{ticket_id}")
async def update_ticket(
    request: Request, ticket_id: str, body: TicketUpdate, db: Session = Depends(get_db)
):
    ticket = db.query(Ticket).filter(Ticket.id == ticket_id).first()
    if not ticket:
        raise HTTPException(status_code=404, detail="Ticket not found")

    now = datetime.now(timezone.utc)
    data = body.model_dump(exclude_unset=True)

    old_status = ticket.status
    old_assignee = ticket.assignee

    # Status transition validation
    if "status" in data:
        new_status = data["status"]
        if new_status != old_status and not validate_transition(old_status, new_status):
            raise HTTPException(
                status_code=400,
                detail=f"Invalid transition: {old_status} → {new_status}. "
                       f"Allowed: {VALID_TRANSITIONS.get(old_status, [])}"
            )

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
        manager = request.app.state.ws_manager
        await manager.broadcast("batch_updated", None,
            batch_to_dict(db.query(Batch).filter(Batch.id == ticket.batch_id).first()))

    d = ticket_to_dict(ticket)
    await request.app.state.ws_manager.broadcast("ticket_updated", ticket.id, d)
    return d


@router.post("/tickets/clear")
async def clear_all_tickets(request: Request, db: Session = Depends(get_db)):
    db.query(TicketLog).delete()
    db.query(TicketEvent).delete()
    db.query(Ticket).delete()
    db.query(Batch).update({
        Batch.total_tickets: 0,
        Batch.completed_tickets: 0,
        Batch.status: "active",
    })
    db.commit()
    await request.app.state.ws_manager.broadcast("tickets_cleared", None, {})
    return {"ok": True, "message": "All tickets cleared"}


@router.delete("/tickets/{ticket_id}")
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


@router.post("/tickets/{ticket_id}/logs")
async def add_log(
    request: Request, ticket_id: str, body: LogCreate, db: Session = Depends(get_db)
):
    ticket = db.query(Ticket).filter(Ticket.id == ticket_id).first()
    if not ticket:
        raise HTTPException(status_code=404, detail="Ticket not found")

    log = TicketLog(
        ticket_id=ticket_id,
        timestamp=datetime.now(timezone.utc),
        level=body.level,
        message=body.message,
        source=body.source,
    )
    db.add(log)
    db.commit()
    db.refresh(log)

    d = log_to_dict(log)
    await request.app.state.ws_manager.broadcast("log_added", ticket_id, d)
    return d


@router.post("/tickets/{ticket_id}/diff")
def save_diff(
    ticket_id: str, body: DiffCreate, db: Session = Depends(get_db)
):
    ticket = db.query(Ticket).filter(Ticket.id == ticket_id).first()
    if not ticket:
        raise HTTPException(status_code=404, detail="Ticket not found")

    ticket.diff_summary = body.diff_summary

    # Cap diff size at 100KB
    diff_full = body.diff_full
    if diff_full and len(diff_full) > MAX_DIFF_SIZE:
        diff_full = diff_full[:MAX_DIFF_SIZE] + "\n\n[TRUNCATED — exceeded 100KB limit]"
    ticket.diff_full = diff_full

    ticket.updated_at = datetime.now(timezone.utc)
    db.commit()
    db.refresh(ticket)
    return ticket_to_dict(ticket)


@router.get("/tickets/{ticket_id}/logs")
def get_ticket_logs(ticket_id: str, db: Session = Depends(get_db)):
    logs = (
        db.query(TicketLog)
        .filter(TicketLog.ticket_id == ticket_id)
        .order_by(TicketLog.timestamp.asc())
        .all()
    )
    return [log_to_dict(l) for l in logs]


@router.post("/tickets/{ticket_id}/retry")
async def retry_ticket(request: Request, ticket_id: str, db: Session = Depends(get_db)):
    original = db.query(Ticket).filter(Ticket.id == ticket_id).first()
    if not original:
        raise HTTPException(status_code=404, detail="Ticket not found")
    if original.status != "failed":
        raise HTTPException(status_code=400, detail="Only failed tickets can be retried")

    now = datetime.now(timezone.utc)
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
    await request.app.state.ws_manager.broadcast("ticket_created", new_ticket.id, d)
    return d
