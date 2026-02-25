from sqlalchemy import Column, String, Text, DateTime, Integer, ForeignKey
from sqlalchemy.orm import relationship
from datetime import datetime

from database import Base


class Batch(Base):
    __tablename__ = "batches"

    id = Column(String, primary_key=True)
    title = Column(String, nullable=False)
    description = Column(Text, nullable=True)
    source_prompt = Column(String, nullable=True)
    total_tickets = Column(Integer, default=0)
    completed_tickets = Column(Integer, default=0)
    status = Column(String, default="active")  # "active" | "completed" | "partial"
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)

    tickets = relationship("Ticket", back_populates="batch")


class Ticket(Base):
    __tablename__ = "tickets"

    id = Column(String, primary_key=True)
    title = Column(String, nullable=False)
    description = Column(Text, nullable=True)
    status = Column(String, default="todo")
    priority = Column(String, default="medium")
    assignee = Column(String, nullable=True)
    system = Column(String, nullable=True)
    files = Column(Text, default="[]")
    dependencies = Column(Text, default="[]")
    dispatch_method = Column(String, default="codex")
    batch_id = Column(String, ForeignKey("batches.id"), nullable=True)
    created_by = Column(String, default="manual")  # "claude_code" | "codex" | "manual"
    ticket_number = Column(Integer, nullable=True)
    branch = Column(String, nullable=True)
    diff_summary = Column(Text, nullable=True)
    diff_full = Column(Text, nullable=True)
    error_message = Column(Text, nullable=True)
    started_at = Column(DateTime, nullable=True)
    completed_at = Column(DateTime, nullable=True)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)

    logs = relationship("TicketLog", back_populates="ticket", cascade="all, delete-orphan")
    events = relationship("TicketEvent", back_populates="ticket", cascade="all, delete-orphan")
    batch = relationship("Batch", back_populates="tickets")


class TicketLog(Base):
    __tablename__ = "ticket_logs"

    id = Column(Integer, primary_key=True, autoincrement=True)
    ticket_id = Column(String, ForeignKey("tickets.id"), nullable=False)
    timestamp = Column(DateTime, default=datetime.utcnow)
    level = Column(String, nullable=False)
    message = Column(Text, nullable=False)
    source = Column(String, nullable=True)

    ticket = relationship("Ticket", back_populates="logs")


class TicketEvent(Base):
    __tablename__ = "ticket_events"

    id = Column(Integer, primary_key=True, autoincrement=True)
    ticket_id = Column(String, ForeignKey("tickets.id"), nullable=False)
    timestamp = Column(DateTime, default=datetime.utcnow)
    event_type = Column(String, nullable=False)
    old_value = Column(String, nullable=False)
    new_value = Column(String, nullable=False)
    actor = Column(String, nullable=False)

    ticket = relationship("Ticket", back_populates="events")
