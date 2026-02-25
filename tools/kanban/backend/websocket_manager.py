from fastapi import WebSocket
from typing import List
from datetime import datetime, timezone
import json


class ConnectionManager:
    def __init__(self):
        self.active_connections: List[WebSocket] = []

    async def connect(self, websocket: WebSocket):
        await websocket.accept()
        self.active_connections.append(websocket)

    def disconnect(self, websocket: WebSocket):
        if websocket in self.active_connections:
            self.active_connections.remove(websocket)

    async def broadcast(self, message_type: str, ticket_id: str, data: dict):
        msg = json.dumps({
            "type": message_type,
            "ticket_id": ticket_id,
            "data": data,
            "timestamp": datetime.now(timezone.utc).isoformat() + "Z"
        }, default=str)
        dead = []
        for conn in self.active_connections:
            try:
                await conn.send_text(msg)
            except Exception:
                dead.append(conn)
        for conn in dead:
            self.active_connections.remove(conn)
