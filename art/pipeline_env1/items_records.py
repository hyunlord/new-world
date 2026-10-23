"""Typed JSON boundaries shared by the item evidence scripts."""

from __future__ import annotations

import json
from collections.abc import Callable, Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import TypeAlias, assert_never

JsonValue: TypeAlias = str | int | float | bool | None | Sequence["JsonValue"] | Mapping[str, "JsonValue"]


def json_decoder() -> Callable[[str], JsonValue]:
    """Expose the standard decoder's JSON value contract at the input boundary."""
    return json.loads


def read_json(path: Path) -> JsonValue:
    return json_decoder()(path.read_text())


def json_sequence(value: JsonValue) -> Sequence[JsonValue]:
    match value:
        case str() | int() | float() | None | Mapping():
            message = "Expected a JSON array"
            raise TypeError(message)
        case Sequence() as values:
            return values
    assert_never(value)


def json_text(value: JsonValue) -> str:
    match value:
        case str() as text:
            return text
        case int() | float() | None | Mapping() | Sequence():
            message = "Expected a JSON string"
            raise TypeError(message)
    assert_never(value)


@dataclass(frozen=True, slots=True)
class JsonRecord:
    """Mutable evidence fields preserve unknown provenance keys during updates."""

    fields: dict[str, JsonValue]

    @classmethod
    def parse(cls, value: JsonValue) -> JsonRecord:
        match value:
            case Mapping() as fields:
                return cls(dict(fields))
            case str() | int() | float() | None | Sequence():
                message = "Expected a JSON object"
                raise TypeError(message)
        assert_never(value)

    @classmethod
    def read(cls, path: Path) -> JsonRecord:
        return cls.parse(read_json(path))

    def text(self, key: str) -> str:
        return json_text(self.fields[key])

    def integer(self, key: str) -> int:
        value = self.fields[key]
        match value:
            case bool():
                message = f"Expected a JSON integer: {key}"
                raise TypeError(message)
            case int() as number:
                return number
            case str() | float() | None | Mapping() | Sequence():
                message = f"Expected a JSON integer: {key}"
                raise TypeError(message)
        assert_never(value)

    def flag(self, key: str) -> bool:
        value = self.fields[key]
        match value:
            case bool() as flag:
                return flag
            case str() | int() | float() | None | Mapping() | Sequence():
                message = f"Expected a JSON boolean: {key}"
                raise TypeError(message)
        assert_never(value)

    def strings(self, key: str) -> list[str]:
        return [json_text(value) for value in json_sequence(self.fields[key])]

    def records(self, key: str) -> list[JsonRecord]:
        return [self.parse(value) for value in json_sequence(self.fields[key])]

    def record(self, key: str) -> JsonRecord:
        return self.parse(self.fields[key])
