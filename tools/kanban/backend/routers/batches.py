import uuid
from datetime import datetime, timezone
from typing import Optional

from fastapi import APIRouter, Depends, HTTPException, Query, Request
from sqlalchemy.orm import Session

from database import get_db
from models import Ticket, Batch
from schemas import BatchCreate, BatchUpdate, BatchDeleteRequest
from utils import batch_to_dict, ticket_to_dict, calculate_quality_score, recalculate_batch_status

router = APIRouter(prefix="/api", tags=["batches"])


@router.post("/batches")
async def create_batch(request: Request, body: BatchCreate, db: Session = Depends(get_db)):
    now = datetime.now(timezone.utc)
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
    await request.app.state.ws_manager.broadcast("batch_created", None, d)
    return d


@router.get("/batches")
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


@router.get("/batches/active")
def get_active_batch(db: Session = Depends(get_db)):
    batch = (
        db.query(Batch)
        .filter(Batch.status == "active")
        .order_by(Batch.created_at.desc())
        .first()
    )
    if not batch:
        raise HTTPException(status_code=404, detail="No active batch")
    return batch_to_dict(batch)


@router.get("/batches/{batch_id}")
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


@router.patch("/batches/{batch_id}")
async def update_batch(
    request: Request, batch_id: str, body: BatchUpdate, db: Session = Depends(get_db)
):
    batch = db.query(Batch).filter(Batch.id == batch_id).first()
    if not batch:
        raise HTTPException(status_code=404, detail="Batch not found")
    data = body.model_dump(exclude_unset=True)
    for field, value in data.items():
        setattr(batch, field, value)
    batch.updated_at = datetime.now(timezone.utc)
    db.commit()
    db.refresh(batch)
    d = batch_to_dict(batch)
    await request.app.state.ws_manager.broadcast("batch_updated", None, d)
    return d


@router.delete("/batches/{batch_id}")
async def delete_batch(request: Request, batch_id: str, db: Session = Depends(get_db)):
    batch = db.query(Batch).filter(Batch.id == batch_id).first()
    if not batch:
        raise HTTPException(status_code=404, detail="Batch not found")

    tickets = db.query(Ticket).filter(Ticket.batch_id == batch_id).all()
    ticket_count = len(tickets)
    for ticket in tickets:
        db.delete(ticket)
    db.delete(batch)
    db.commit()

    await request.app.state.ws_manager.broadcast("batch_deleted", None, {
        "id": batch_id, "deleted_tickets": ticket_count
    })
    return {"ok": True, "deleted_tickets": ticket_count}


@router.post("/batches/bulk-delete")
async def bulk_delete_batches(request: Request, body: BatchDeleteRequest, db: Session = Depends(get_db)):
    if len(body.batch_ids) > 50:
        raise HTTPException(status_code=400, detail="Max 50 batches per request")

    total_tickets = 0
    deleted_ids = []

    for batch_id in body.batch_ids:
        batch = db.query(Batch).filter(Batch.id == batch_id).first()
        if not batch:
            continue
        tickets = db.query(Ticket).filter(Ticket.batch_id == batch_id).all()
        total_tickets += len(tickets)
        for ticket in tickets:
            db.delete(ticket)
        db.delete(batch)
        deleted_ids.append(batch_id)

    db.commit()

    await request.app.state.ws_manager.broadcast("batch_bulk_deleted", None, {
        "ids": deleted_ids, "total_deleted_tickets": total_tickets
    })
    return {"deleted_batches": len(deleted_ids), "deleted_tickets": total_tickets}
