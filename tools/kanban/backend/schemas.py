from pydantic import BaseModel
from typing import Optional, List, Dict
from datetime import datetime


class BatchCreate(BaseModel):
    title: str
    description: Optional[str] = None
    source_prompt: Optional[str] = None


class BatchUpdate(BaseModel):
    title: Optional[str] = None
    description: Optional[str] = None
    source_prompt: Optional[str] = None


class BatchResponse(BaseModel):
    model_config = {"from_attributes": True}

    id: str
    title: str
    description: Optional[str]
    source_prompt: Optional[str]
    total_tickets: int
    completed_tickets: int
    status: str
    created_at: datetime
    updated_at: datetime


class BatchListResponse(BaseModel):
    total: int
    batches: List[BatchResponse]
    limit: int
    offset: int


class TicketCreate(BaseModel):
    id: Optional[str] = None
    title: str
    description: Optional[str] = None
    body: Optional[str] = None
    priority: Optional[str] = "medium"
    system: Optional[str] = None
    files: Optional[List[str]] = None
    dependencies: Optional[List[str]] = None
    dispatch_method: Optional[str] = "codex"
    branch: Optional[str] = None
    batch_id: Optional[str] = None
    created_by: Optional[str] = "manual"
    ticket_number: Optional[int] = None


class TicketUpdate(BaseModel):
    status: Optional[str] = None
    priority: Optional[str] = None
    assignee: Optional[str] = None
    system: Optional[str] = None
    files: Optional[List[str]] = None
    dependencies: Optional[List[str]] = None
    dispatch_method: Optional[str] = None
    branch: Optional[str] = None
    diff_summary: Optional[str] = None
    diff_full: Optional[str] = None
    error_message: Optional[str] = None
    description: Optional[str] = None
    body: Optional[str] = None
    commit_hash: Optional[str] = None
    batch_id: Optional[str] = None
    created_by: Optional[str] = None
    ticket_number: Optional[int] = None
    dismissed: Optional[bool] = None


class LogCreate(BaseModel):
    level: str
    message: str
    source: Optional[str] = "codex"


class DiffCreate(BaseModel):
    diff_summary: str
    diff_full: Optional[str] = None


class TicketResponse(BaseModel):
    model_config = {"from_attributes": True}

    id: str
    title: str
    description: Optional[str]
    status: str
    priority: str
    assignee: Optional[str]
    system: Optional[str]
    files: str
    dependencies: str
    dispatch_method: str
    batch_id: Optional[str]
    created_by: Optional[str]
    ticket_number: Optional[int]
    branch: Optional[str]
    retry_of: Optional[str]
    retry_count: int
    commit_hash: Optional[str]
    commit_url: Optional[str]
    diff_summary: Optional[str]
    diff_full: Optional[str]
    error_message: Optional[str]
    started_at: Optional[datetime]
    completed_at: Optional[datetime]
    created_at: datetime
    updated_at: datetime


class LogResponse(BaseModel):
    model_config = {"from_attributes": True}

    id: int
    ticket_id: str
    timestamp: datetime
    level: str
    message: str
    source: Optional[str]


class EventResponse(BaseModel):
    model_config = {"from_attributes": True}

    id: int
    ticket_id: str
    timestamp: datetime
    event_type: str
    old_value: str
    new_value: str
    actor: str


class TicketListResponse(BaseModel):
    total: int
    tickets: List[TicketResponse]
    limit: int
    offset: int


class StatsResponse(BaseModel):
    total: int
    active: int
    done: int
    failed: int
    rate: int
    systems: Dict[str, int]
    avg_duration_seconds: Optional[float]


class DailyStatsResponse(BaseModel):
    days: int
    daily: Dict[str, Dict[str, int]]
