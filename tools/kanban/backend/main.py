import sqlite3

from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware

from database import engine, Base, DB_PATH
from websocket_manager import ConnectionManager
from routers.batches import router as batches_router
from routers.tickets import router as tickets_router
from routers.stats import router as stats_router

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
app.state.ws_manager = manager

# ---------------------------------------------------------------------------
# Routers
# ---------------------------------------------------------------------------

app.include_router(batches_router)
app.include_router(tickets_router)
app.include_router(stats_router)

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
