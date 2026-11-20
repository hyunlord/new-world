# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy"]
# ///
"""Read BVH as inert data and evaluate FK in Blender axes (X lateral, -Y forward, Z up)."""

from dataclasses import dataclass
import io
from pathlib import Path
import re

import numpy as np
from numpy.typing import NDArray


class BvhError(ValueError):
    """Malformed or unsupported BVH input."""


@dataclass(frozen=True, slots=True)
class Motion:
    joints: dict[str, int]
    positions: NDArray[np.float64]
    rotation: NDArray[np.float64]
    frame_time: float


@dataclass(frozen=True, slots=True)
class Joint:
    name: str
    parent: int
    offset: tuple[float, float, float]
    channels: tuple[str, ...]


# BVH X, Y up, Z forward to Blender X, Y backward, Z up.
BASIS = np.array(((1., 0., 0.), (0., 0., -1.), (0., 1., 0.)))
AXES = {"X": 0, "Y": 1, "Z": 2}


def hierarchy(source: str) -> list[Joint]:
    tokens = iter(re.findall(r"[^\s{}]+|[{}]", source))
    joints: list[Joint] = []

    def expect(expected: str) -> None:
        received = next(tokens)
        if received != expected:
            raise BvhError(f"Expected {expected}, received {received}")

    def joint(parent: int, depth: int) -> None:
        if depth > 48 or len(joints) > 128:
            raise BvhError("BVH hierarchy exceeds supported size")
        name = next(tokens)
        expect("{")
        expect("OFFSET")
        offset = (float(next(tokens)), float(next(tokens)), float(next(tokens)))
        expect("CHANNELS")
        count = int(next(tokens))
        if not 0 <= count <= 6:
            raise BvhError(f"Invalid channel count for {name}")
        channels = tuple(next(tokens) for _ in range(count))
        if any(re.fullmatch(r"[XYZ](?:rotation|position)", channel) is None for channel in channels):
            raise BvhError(f"Invalid channels for {name}")
        index = len(joints)
        joints.append(Joint(name, parent, offset, channels))
        for token in tokens:
            match token:
                case "JOINT":
                    joint(index, depth + 1)
                case "End":
                    expect("Site")
                    expect("{")
                    expect("OFFSET")
                    for _ in range(3):
                        float(next(tokens))
                    expect("}")
                case "}":
                    return
                case _:
                    raise BvhError(f"Invalid token in {name}: {token}")
        raise BvhError(f"Unclosed joint {name}")

    try:
        expect("HIERARCHY")
        expect("ROOT")
        joint(-1, 0)
        if next(tokens, None) is not None:
            raise BvhError("Unexpected data after BVH hierarchy")
    except (StopIteration, RuntimeError) as error:
        raise BvhError("Truncated BVH hierarchy") from error
    if len({item.name for item in joints}) != len(joints):
        raise BvhError("Duplicate BVH joint name")
    return joints


def _rotation(axis: int, angles: NDArray[np.float64]) -> NDArray[np.float64]:
    matrices = np.zeros((len(angles), 3, 3))
    a, b = (axis + 1) % 3, (axis + 2) % 3
    matrices[:, axis, axis] = 1
    matrices[:, a, a] = matrices[:, b, b] = np.cos(angles)
    matrices[:, b, a] = np.sin(angles)
    matrices[:, a, b] = -matrices[:, b, a]
    return matrices


def read_bvh(path: Path) -> Motion:
    """Validate the take layout, then apply translations and rotations in BVH channel order."""
    if path.stat().st_size > 12_000_000:
        raise BvhError(f"BVH exceeds size limit: {path.name}")
    source, marker, samples = path.read_text(encoding="ascii").partition("\nMOTION")
    header = re.match(r"\s*Frames:\s*(\d+)[ \t]*\nFrame Time:\s*(\S+)[ \t]*\n", samples)
    if not marker or not header:
        raise BvhError(f"Missing motion header: {path.name}")
    frames, frame_time = int(header[1]), float(header[2])
    if not 2 <= frames <= 24_000 or not .001 < frame_time < 1.:
        raise BvhError(f"Unsupported frame layout: {path.name}")
    joints = hierarchy(source)
    values = np.loadtxt(io.StringIO(samples[header.end():]), ndmin=2)
    if values.shape != (frames, sum(len(joint.channels) for joint in joints)) or not np.isfinite(values).all():
        raise BvhError(f"Invalid frame rows: {path.name}")
    positions = np.empty((frames, len(joints), 3))
    rotations = np.empty((frames, len(joints), 3, 3))
    channel_index = 0
    for i, joint in enumerate(joints):
        translation = np.broadcast_to(joint.offset, (frames, 3)).copy()
        rotation = np.broadcast_to(np.eye(3), (frames, 3, 3))
        for channel in joint.channels:
            axis = AXES[channel[0]]
            track = values[:, channel_index]
            channel_index += 1
            if channel.endswith("position"):
                translation += rotation[:, :, axis] * track[:, None]
            else:
                rotation = rotation @ _rotation(axis, np.deg2rad(track))
        if joint.parent >= 0:
            parent = rotations[:, joint.parent]
            translation = positions[:, joint.parent] + np.einsum("fij,fj->fi", parent, translation)
            rotation = parent @ rotation
        positions[:, i] = translation
        rotations[:, i] = rotation
    return Motion({joint.name: i for i, joint in enumerate(joints)}, positions @ BASIS.T,
                  BASIS @ rotations @ BASIS.T, frame_time)
