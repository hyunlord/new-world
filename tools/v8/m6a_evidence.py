"""Strict JSON evidence boundaries and content-addressed local references."""
from __future__ import annotations

import hashlib
import json
import math
import subprocess
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Final, TypeAlias

Json: TypeAlias = None | bool | int | float | str | list["Json"] | dict[str, "Json"]
INPUT_ROOTS: Final = ("rust", "scripts", "scenes", "shaders", "assets", "localization", "tools", "project.godot", "sim_bridge.gdextension", "sim_bridge.gdextension.uid")


class EvidenceError(ValueError):
    """Malformed, stale or incomplete measurement evidence."""


def _pairs(pairs: list[tuple[str, Json]]) -> dict[str, Json]:
    result: dict[str, Json] = {}
    for key, value in pairs:
        if key in result:
            raise EvidenceError(f"duplicate JSON key {key}")
        result[key] = value
    return result


def _constant(value: str) -> None:
    raise EvidenceError(f"nonfinite JSON constant {value}")


def _finite_float(value: str) -> float:
    number = float(value)
    if not math.isfinite(number):
        raise EvidenceError(f"nonfinite JSON number {value}")
    return number


@dataclass(frozen=True, slots=True)
class Node:
    """Validated JSON access without implicit truthiness or numeric coercion."""

    value: Json

    @classmethod
    def parse(cls, text: str) -> Node:
        return cls(json.loads(text, object_pairs_hook=_pairs, parse_constant=_constant, parse_float=_finite_float))

    @classmethod
    def load(cls, path: Path) -> Node:
        return cls.parse(path.read_text())

    def get(self, key: str) -> Node:
        if not isinstance(self.value, dict) or key not in self.value:
            raise EvidenceError(f"missing object field {key}")
        return Node(self.value[key])

    def rows(self) -> tuple[Node, ...]:
        if not isinstance(self.value, list):
            raise EvidenceError("expected JSON array")
        return tuple(Node(value) for value in self.value)

    def fields(self) -> tuple[tuple[str, Node], ...]:
        if not isinstance(self.value, dict):
            raise EvidenceError("expected JSON object")
        return tuple((key, Node(value)) for key, value in self.value.items())

    def integer(self) -> int:
        if type(self.value) is not int or self.value < 0:
            raise EvidenceError("expected nonnegative integer")
        return self.value

    def text(self) -> str:
        if not isinstance(self.value, str):
            raise EvidenceError("expected string")
        return self.value

    def boolean(self) -> bool:
        if type(self.value) is not bool:
            raise EvidenceError("expected boolean")
        return self.value


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


@dataclass(frozen=True, slots=True)
class Reference:
    path: str
    sha256: str
    size: int

    @classmethod
    def capture(cls, path: Path) -> Reference:
        path = path.resolve(strict=True)
        return cls(str(path), digest(path), path.stat().st_size)

    @classmethod
    def parse(cls, node: Node) -> Reference:
        if {key for key, _ in node.fields()} != {"path", "sha256", "size"}:
            raise EvidenceError("reference keys differ")
        ref = cls(node.get("path").text(), node.get("sha256").text(), node.get("size").integer())
        if ref != cls.capture(Path(ref.path)):
            raise EvidenceError(f"reference changed: {ref.path}")
        return ref

    def read(self) -> Node:
        if self != Reference.capture(Path(self.path)):
            raise EvidenceError(f"reference changed: {self.path}")
        return Node.load(Path(self.path))


def git(root: Path, *args: str) -> str:
    return subprocess.check_output(["git", "-C", str(root), *args], text=True).strip()


def source_hash(root: Path, head: str) -> str:
    """Bind committed and live runtime/input files, rejecting dirty or extra input."""
    if git(root, "rev-parse", "HEAD") != head:
        raise EvidenceError("source HEAD changed")
    if git(root, "status", "--porcelain", "--untracked-files=all", "--", *INPUT_ROOTS):
        raise EvidenceError("runtime/input source is dirty or untracked")
    names = git(root, "ls-files", "--", *INPUT_ROOTS).splitlines()
    if not names:
        raise EvidenceError("source manifest is empty")
    seal = hashlib.sha256()
    for name in names:
        seal.update(name.encode() + b"\0" + digest(root / name).encode() + b"\n")
    return seal.hexdigest()


def write_json(path: Path, value: Json) -> None:
    with path.open("x") as stream:
        json.dump(value, stream, indent=2, sort_keys=True, allow_nan=False)
        stream.write("\n")


def reference_json(path: Path) -> dict[str, Json]:
    return asdict(Reference.capture(path))
