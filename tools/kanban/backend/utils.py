import json
import os
from datetime import datetime, timezone
from typing import Optional

from sqlalchemy.orm import Session

from models import Ticket, TicketLog, TicketEvent, Batch

GITHUB_REPO = os.environ.get("GITHUB_REPO_URL", "https://github.com/hyunlord/new-world")

MAX_DIFF_SIZE = 102400  # 100KB

VALID_TRANSITIONS = {
    "todo":        ["claimed", "in_progress"],
    "claimed":     ["in_progress", "todo"],
    "in_progress": ["review", "done", "failed", "todo"],
    "review":      ["done", "failed", "in_progress"],
    "done":        [],
    "failed":      ["todo"],
}


def validate_transition(old_status: str, new_status: str) -> bool:
    allowed = VALID_TRANSITIONS.get(old_status, [])
    return new_status in allowed


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
    batch.updated_at = datetime.now(timezone.utc)


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
