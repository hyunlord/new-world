# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
# How to run: imported by art.whole_figure_lab.prod2_score
"""Typed score evidence records, serialized without altering the historical schema."""
from typing import TypedDict


class FieldScore(TypedDict):
    raw: str
    mapped: str
    truth: str
    correct: bool
    mapping_status: str


class Case(TypedDict):
    id: str
    zoom: int
    look: str
    fields: dict[str, FieldScore]


class Bar(TypedDict):
    correct: int
    denominator: int
    bar: int
    passed: bool


class Scale(TypedDict):
    zoom: int
    field_correct: dict[str, int]
    bars: dict[str, Bar]
    all_numeric_bars_met: bool


class Review(TypedDict):
    reviewer: int
    raw_path: str
    raw_sha256: str
    verbatim_copy: str
    summaries: list[Scale]
    cases: list[Case]


class Texture(TypedDict):
    number: int
    facing: str
    activity: str
    files: list[str]
