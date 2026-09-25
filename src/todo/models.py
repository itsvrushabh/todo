from datetime import UTC, datetime
from enum import Enum

from pydantic import BaseModel, Field


class Priority(str, Enum):
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"


class TodoBase(BaseModel):
    title: str = Field(
        ..., min_length=1, max_length=200, description="Title of the task"
    )
    description: str | None = Field(
        default="", max_length=1000, description="Optional details about the task"
    )
    priority: Priority = Field(default=Priority.MEDIUM, description="Priority level")


class TodoCreate(TodoBase):
    pass


class TodoUpdate(BaseModel):
    title: str | None = Field(default=None, min_length=1, max_length=200)
    description: str | None = Field(default=None, max_length=1000)
    completed: bool | None = None
    priority: Priority | None = None


class TodoItem(TodoBase):
    id: int
    completed: bool = False
    created_at: str = Field(default_factory=lambda: datetime.now(UTC).isoformat())
    updated_at: str | None = None
