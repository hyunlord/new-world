# /// script
# requires-python = ">=3.11"
# dependencies = ["pydantic"]
# ///
# How to run: python3 -m pytest art/compositor/test_face_variants.py -q.
"""Validated face plans and deterministic person-to-variant selection for art."""
from __future__ import annotations

from pathlib import Path
from typing import ClassVar, Final, Literal, Self

from pydantic import BaseModel, ConfigDict, model_validator

ROOT: Final = Path(__file__).resolve().parents[2]
PLAN_PATH: Final = ROOT / "art/character/FACE_VARIANTS.json"
VariantId = Literal["v1", "v2", "v3", "v4", "v5", "v6"]


class Variant(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")
    id: VariantId
    skin: Literal["S1", "S2", "S3"]
    hair: Literal["H1", "H2", "H3", "H4"]
    age_band: Literal["young", "adult", "older", "child"]
    features: tuple[str, ...]
    beard: Literal["full", "short", "clean", "none"]
    description: str
    skin_rgb: tuple[int, int, int]
    hair_rgb: tuple[int, int, int]


class LookVariants(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")
    look: str
    variants: tuple[Variant, ...]

    @model_validator(mode="after")
    def check_axes(self) -> Self:
        if tuple(v.id for v in self.variants) != ("v1", "v2", "v3", "v4", "v5", "v6"):
            message = f"{self.look}: six ordered variants required"
            raise ValueError(message)
        if {v.skin for v in self.variants} != {"S1", "S2", "S3"}:
            message = f"{self.look}: all three skin axes required"
            raise ValueError(message)
        if len({v.hair for v in self.variants}) < 3:
            message = f"{self.look}: three hair axes required"
            raise ValueError(message)
        children = self.look.startswith("child_")
        if children and any(v.hair == "H4" or v.beard != "none" or v.age_band != "child" for v in self.variants):
            message = f"{self.look}: child age, hair or beard invalid"
            raise ValueError(message)
        if not children:
            older = [v for v in self.variants if v.age_band == "older"]
            if len(older) != 2 or not any(v.hair == "H4" for v in older):
                message = f"{self.look}: two older variants including grey required"
                raise ValueError(message)
            if self.look.startswith("adult_M_") and {v.beard for v in self.variants} != {"full", "short", "clean"}:
                message = f"{self.look}: all three beard types required"
                raise ValueError(message)
        return self


class FacePlan(BaseModel):
    model_config: ClassVar[ConfigDict] = ConfigDict(frozen=True, extra="forbid")
    schema_version: Literal[1]
    source_manifest: str
    source_manifest_sha256: str
    palette: str
    rule: str
    looks: tuple[LookVariants, ...]
    fallbacks: dict[str, str]
    owner_sample: tuple[str, ...]
    identity_sample: tuple[str, ...]

    @classmethod
    def read(cls, path: Path = PLAN_PATH) -> Self:
        return cls.model_validate_json(path.read_bytes())

    def for_look(self, look: str) -> LookVariants:
        resolved = self.fallbacks.get(look, look)
        for entry in self.looks:
            if entry.look == resolved:
                return entry
        message = f"Unknown character look: {look}"
        raise ValueError(message)


def choose_variant(agent_id: int, age: int, look: LookVariants) -> Variant:
    """Apply uint32 multiplicative hash, with older-only selection from age 50."""
    if agent_id < 0 or age < 0:
        message = "Person id and age must be nonnegative"
        raise ValueError(message)
    pool = look.variants
    if age >= 50 and not look.look.startswith("child_"):
        pool = tuple(v for v in pool if v.age_band == "older")
    return pool[((agent_id * 2654435761) & 0xFFFFFFFF) % len(pool)]


def baked_frame(root: Path, look: LookVariants, variant: Variant) -> Path:
    """Resolve the tinted south-idle reference used by portraits and boards."""
    return root / "art/character/route_i" / look.look / variant.id / "S/idle.png"
