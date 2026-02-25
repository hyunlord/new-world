from datetime import datetime, timezone, timedelta
from typing import Optional

from fastapi import APIRouter, Depends, HTTPException, Query
from sqlalchemy import func, case
from sqlalchemy.orm import Session

from database import get_db
from models import Ticket, TicketLog, TicketEvent, Batch
from utils import batch_to_dict, ticket_to_dict, calculate_quality_score

router = APIRouter(prefix="/api", tags=["stats"])


@router.get("/stats")
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

    # Average duration for done tickets
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

    # By created_by — aggregate query instead of N+1
    by_created_by = {}
    creator_rows = (
        db.query(
            Ticket.created_by,
            func.count(Ticket.id).label("total"),
            func.sum(case((Ticket.status == "done", 1), else_=0)).label("done"),
            func.sum(case((Ticket.status == "failed", 1), else_=0)).label("failed"),
        )
        .filter(Ticket.created_by.isnot(None))
        .group_by(Ticket.created_by)
        .all()
    )
    for row in creator_rows:
        by_created_by[row[0]] = {
            "total": row[1],
            "done": row[2] or 0,
            "failed": row[3] or 0,
        }

    # By dispatch_method — aggregate query instead of N+1
    by_dispatch_method = {}
    method_rows = (
        db.query(
            Ticket.dispatch_method,
            func.count(Ticket.id).label("total"),
            func.sum(case((Ticket.status == "done", 1), else_=0)).label("done"),
            func.sum(case((Ticket.status == "failed", 1), else_=0)).label("failed"),
        )
        .filter(Ticket.dispatch_method.isnot(None))
        .group_by(Ticket.dispatch_method)
        .all()
    )
    for row in method_rows:
        # Get avg duration per method
        method_done = (
            db.query(Ticket)
            .filter(
                Ticket.dispatch_method == row[0],
                Ticket.status == "done",
                Ticket.started_at.isnot(None),
                Ticket.completed_at.isnot(None),
            )
            .all()
        )
        m_durations = [
            (t.completed_at - t.started_at).total_seconds()
            for t in method_done
            if t.completed_at >= t.started_at
        ]
        by_dispatch_method[row[0]] = {
            "total": row[1],
            "done": row[2] or 0,
            "failed": row[3] or 0,
            "avg_duration": round(sum(m_durations) / len(m_durations), 1) if m_durations else None,
        }

    # Dispatch ratio
    codex_total = db.query(Ticket).filter(Ticket.dispatch_method == "codex").count()
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


@router.get("/stats/errors")
def get_error_stats(days: int = Query(30), db: Session = Depends(get_db)):
    cutoff = datetime.now(timezone.utc) - timedelta(days=days)

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


@router.get("/stats/daily")
def get_daily_stats(days: int = Query(30), db: Session = Depends(get_db)):
    cutoff = datetime.now(timezone.utc) - timedelta(days=days)

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


@router.get("/stats/batch-compare")
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


@router.get("/stats/agents")
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
