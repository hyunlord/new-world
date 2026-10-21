# /// script
# dependencies = ["numpy", "Pillow"]
# ///
# How to run: python3 -m art.pipeline.build_palette (from repository root).
"""Reproducible frequency-weighted Lab palette with exact semantic reservations."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Final, TypedDict

import numpy as np
from numpy.typing import NDArray
from PIL import Image, ImageDraw

from art.pipeline.color import rgb_to_lab, squared_length, vector_length
from art.pipeline.models import decode_json, json_array, json_object, json_string

ROOT: Final = Path(__file__).resolve().parents[2]
SOURCES: Final = ("assets/v8/tiles.png", "docs/images/v8_target_view_b.png")
CHANNELS: Final = {"outline": "#141210", "skin": "#9A9A9A", "loincloth": "#6E6E6E",
                  "hair": "#B0B0B0", "top": "#C0C0C0"}
TRIAL_SKINS: Final = ("#B9815F", "#915C40", "#68402E")


class ColorEntry(TypedDict):
    name: str
    hex: str
    source: str


class SourceEntry(TypedDict):
    path: str
    sha256: str
    opaque_pixels: int


class ClusterEntry(TypedDict):
    name: str
    hex: str
    population: int


class DropEntry(TypedDict):
    name: str
    hex: str
    reason: str
    nearest: str
    delta_e: float


def hex_rgb(value: str) -> NDArray[np.uint8]:
    """Decode one palette hex value to an RGB triple."""
    return np.array([int(value[index:index + 2], 16) for index in (1, 3, 5)], dtype=np.uint8)


def source_distribution(root: Path) -> tuple[NDArray[np.uint8], NDArray[np.float64], list[SourceEntry]]:
    """Count all fully opaque pixels, retaining the natural frequency of each image."""
    pixels: list[NDArray[np.uint8]] = []
    records: list[SourceEntry] = []
    for name in SOURCES:
        path = root / name
        with Image.open(path) as image:
            rgba = np.asarray(image.convert("RGBA"), dtype=np.uint8).reshape(-1, 4)
        opaque = rgba[rgba[:, 3] == 255, :3]
        pixels.append(opaque)
        records.append({"path": name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                        "opaque_pixels": len(opaque)})
    unique, counts = np.unique(np.concatenate(pixels), axis=0, return_counts=True)
    return unique, counts.astype(np.float64), records


def cluster_colors(rgb: NDArray[np.uint8], weights: NDArray[np.float64]) -> tuple[list[ClusterEntry], int, bool]:
    """Lloyd k-means with deterministic weighted farthest-point initialisation."""
    lab = rgb_to_lab(rgb)
    chosen = [int(np.argmax(weights))]
    distances: NDArray[np.float64] = squared_length(lab - lab[chosen[0], :], axis=1)
    for _ in range(39):
        chosen.append(int(np.argmax(weights * distances)))
        distances = np.minimum(distances, squared_length(lab - lab[chosen[-1], :], axis=1))
    centers: NDArray[np.float64] = lab[chosen].copy()
    labels = np.zeros(len(lab), dtype=np.int64)
    converged = False
    iteration = 0
    for iteration in range(1, 201):
        # Nx40 is bounded by unique colours, not full image pixel count.
        distance_matrix: NDArray[np.float64] = squared_length(lab[:, None, :] - centers[None, :, :], axis=2)
        updated_labels = np.empty(len(lab), dtype=np.intp)
        _ = np.argmin(distance_matrix, axis=1, out=updated_labels)
        if iteration > 1 and np.array_equal(updated_labels, labels):
            converged = True
            break
        labels = updated_labels
        for index in range(40):
            members: NDArray[np.bool_] = np.equal(labels, index)
            if members.any():
                centers[index] = np.average(lab[members], axis=0, weights=weights[members])
    clusters: list[ClusterEntry] = []
    for index in range(len(centers)):
        center: NDArray[np.float64] = centers[index, :]
        snapped: NDArray[np.uint8] = rgb[int(np.argmin(squared_length(lab - center, axis=1))), :]
        clusters.append({"name": f"source_cluster_{index:02d}",
                         "hex": f"#{int(snapped.item(0)):02X}{int(snapped.item(1)):02X}{int(snapped.item(2)):02X}",
                         "population": int(np.sum(weights[np.equal(labels, index)], dtype=np.float64))})
    return sorted(clusters, key=lambda item: (-item["population"], item["name"])), iteration, converged


def reserved_colors(root: Path) -> tuple[list[ColorEntry], dict[str, list[str] | str]]:
    """Read source people colours from the committed sheet's generator metadata."""
    palette = json_object(json_object(decode_json((root / "assets/v8/tiles.json").read_text()))["palettes"])
    people: dict[str, list[str] | str] = {
        "skins": ["#" + json_string(palette["skin"]).upper(), *TRIAL_SKINS],
        "hairs": ["#" + json_string(value).upper() for value in json_array(palette["hairs"])],
        "tunics": ["#" + json_string(value).upper() for value in json_array(palette["tunics"])],
        "elder_hair": "#" + json_string(palette["elder_hair"]).upper(),
    }
    entries: list[ColorEntry] = [{"name": name, "hex": value, "source": "TRIAL_GOAL.md exact semantic channel"}
                                for name, value in CHANNELS.items()]
    for group in ("skins", "hairs", "tunics"):
        values = people[group]
        if not isinstance(values, list):
            raise TypeError(group)
        for index, value in enumerate(values):
            trial = group == "skins" and index > 0
            entries.append({"name": f"{group}_{index}", "hex": value,
                            "source": "trial-derived warm-earth skin; D2" if trial else
                            f"assets/v8/tiles.json palettes; tools/v8/gen_tiles.py PALETTES {group}"})
    entries.append({"name": "elder_hair", "hex": str(people["elder_hair"]),
                    "source": "assets/v8/tiles.json palettes.elder_hair"})
    return entries, people


def build_palette(root: Path, destination: Path) -> None:
    """Write deterministic JSON provenance and a labelled 8-column swatch PNG."""
    rgb, weights, sources = source_distribution(root)
    clusters, iterations, converged = cluster_colors(rgb, weights)
    colors, people = reserved_colors(root)
    dropped: list[DropEntry] = []
    for cluster in clusters:
        candidate: NDArray[np.float64] = rgb_to_lab(hex_rgb(cluster["hex"]))
        distances: NDArray[np.float64] = vector_length(rgb_to_lab(np.array([hex_rgb(item["hex"]) for item in colors])) - candidate, axis=1)
        nearest = int(np.argmin(distances))
        if distances[nearest] < 4 or len(colors) >= 48:
            dropped.append({"name": cluster["name"], "hex": cluster["hex"],
                            "reason": "delta_e_below_4" if distances[nearest] < 4 else "48_color_cap",
                            "nearest": colors[nearest]["hex"], "delta_e": round(float(distances.item(nearest)), 6)})
        else:
            colors.append({"name": cluster["name"], "hex": cluster["hex"],
                           "source": "opaque source pixel nearest weighted Lab k-means center"})
    reserved, _ = reserved_colors(root)
    exceptions: list[DropEntry] = []
    for index, left in enumerate(reserved):
        for right in reserved[index + 1:]:
            distance = float(vector_length(rgb_to_lab(hex_rgb(left["hex"])) - rgb_to_lab(hex_rgb(right["hex"])), axis=0).item())
            if distance < 4:
                exceptions.append({"name": left["name"], "hex": left["hex"], "reason": "D3 exact semantic reservation",
                                   "nearest": right["hex"], "delta_e": round(distance, 6)})
    metadata_paths = ("assets/v8/tiles.json", "tools/v8/gen_tiles.py", "scripts/world/colony/view.gd")
    payload = {
        "colors": colors, "channels": CHANNELS, "people": people, "blueprint": "#6FB0E8",
        "method": {
            "color_space": "CIE Lab, sRGB transfer, D65 white, Euclidean DeltaE76",
            "algorithm": "40-cluster weighted Lloyd k-means; snap final centers to nearest existing opaque RGB",
            "frequency_weighting": "Every alpha=255 pixel has weight one across both images; no image normalization or subsampling",
            "initialization": "Most frequent RGB first; maximize pixel_count * squared nearest-center Lab distance thereafter",
            "tie_break": "RGB lexicographic order; cluster index for equal center distances",
            "clusters_requested": 40, "iterations": iterations, "converged": converged,
            "iteration_cap": 200, "unique_opaque_colors": len(rgb), "sources": sources,
            "people_source_hashes": {path: hashlib.sha256((root / path).read_bytes()).hexdigest() for path in metadata_paths},
            "selection": "Reserve semantic and people entries first; clusters descending population; reject DeltaE <4; cap 48",
            "trial_skins": "D2: source #D8A884 plus authored darker warm-earth tones #B9815F #915C40 #68402E; not claimed as source pixels",
            "blueprint": "Reuse legacy household tunic #6FB0E8; no extra palette slot",
            "clusters": clusters, "dropped": dropped, "semantic_delta_e_exceptions": exceptions,
        },
    }
    destination.mkdir(parents=True, exist_ok=True)
    _ = (destination / "worldsim_b.json").write_text(json.dumps(payload, indent=2) + "\n")
    swatch = Image.new("RGB", (8 * 144, ((len(colors) + 7) // 8) * 72), "#141210")
    draw = ImageDraw.Draw(swatch)
    for index, color in enumerate(colors):
        x, y = (index % 8) * 144, (index // 8) * 72
        draw.rectangle((x + 4, y + 4, x + 139, y + 39), fill=color["hex"])
        draw.text((x + 4, y + 42), color["name"], fill="white")
        draw.text((x + 4, y + 55), color["hex"], fill="white")
    swatch.save(destination / "worldsim_b.png", optimize=False)


if __name__ == "__main__":
    build_palette(ROOT, ROOT / "art/palette")
