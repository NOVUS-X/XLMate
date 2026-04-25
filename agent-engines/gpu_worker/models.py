from __future__ import annotations

from datetime import datetime, timezone
from enum import Enum
import uuid

import chess
from pydantic import BaseModel, Field, field_validator


class AnalysisRequest(BaseModel):
    """Request payload for a single chess position analysis."""

    id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    fen: str
    depth: int | None = Field(default=None, ge=1)
    time_limit_ms: int | None = Field(default=None, ge=1)
    search_moves: list[str] | None = None
    num_pv: int = Field(default=1, ge=1)
    priority: int = 0
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))

    @field_validator("fen")
    @classmethod
    def validate_fen(cls, value: str) -> str:
        """Validate FEN strings using python-chess."""

        normalized = value.strip()
        try:
            chess.Board(normalized)
        except ValueError as exc:
            raise ValueError(f"invalid FEN: {exc}") from exc
        return normalized


class AnalysisResult(BaseModel):
    """Normalized analysis result returned by the worker pool."""

    request_id: str
    best_move: str
    evaluation: float | None = None
    depth: int | None = None
    principal_variation: list[str] = Field(default_factory=list)
    nodes_searched: int | None = None
    time_ms: int | None = None
    gpu_utilization: float | None = None


class WorkerStatus(str, Enum):
    """Runtime status of a worker instance."""

    IDLE = "idle"
    BUSY = "busy"
    ERROR = "error"
    SHUTTING_DOWN = "shutting_down"


class WorkerInfo(BaseModel):
    """Snapshot of worker health and utilization."""

    worker_id: str
    status: WorkerStatus
    gpu_device_id: int
    gpu_memory_used_mb: float = 0.0
    gpu_utilization_pct: float = 0.0
    analyses_completed: int = 0
    uptime_seconds: float = 0.0
