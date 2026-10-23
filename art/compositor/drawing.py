# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy", "pydantic"]
# ///
# How to run: imported by python3 -m art.compositor.render.
"""Native figure anchoring, true-colour LOD marks and geometric blueprint overlay."""
from __future__ import annotations

import hashlib
from dataclasses import dataclass
from pathlib import Path
from typing import Final, assert_never

from PIL import Image, ImageDraw, ImageFilter, ImageFont
from pydantic import JsonValue

from art.compositor.assets import (
    REPO,
    Catalog,
    choose_look,
    frame_path,
    read_image,
    top_colour,
)
from art.compositor.face_variants import FacePlan, Variant, choose_variant
from art.compositor.geometry import HUD_HEIGHT, WINDOW, Viewport
from art.compositor.models import Activity, Person
from art.compositor.placements import RenderPerson, Sprite

INK: Final = (35, 29, 23, 255)
FONT: Final = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 14)


def variant_selection_age(person: Person) -> int:
    """Map captured life-stage classes to selector pools without inventing ages."""
    match person.life_stage:
        case "child":
            return 10
        case "adult":
            return 30
        case "elder":
            return 50
    assert_never(person.life_stage)


def activity_frame(activity: Activity) -> str:
    match activity:
        case Activity.AWAKE:
            return "idle"
        case Activity.SLEEP:
            return "sleep"
        case Activity.CARRY:
            return "carry"
        case Activity.CHOP:
            return "work_a"
    assert_never(activity)


def site_overlay(source: Image.Image) -> Image.Image:
    alpha = source.getchannel("A")
    wash = source.copy()
    wash.putalpha(alpha.point([round(value * .25) for value in range(256)]))
    edge = alpha.filter(ImageFilter.MaxFilter(3))
    interior = alpha.filter(ImageFilter.MinFilter(3))
    from PIL import ImageChops
    outline = Image.new("RGBA", source.size, (218, 222, 205, 255))
    outline.putalpha(ImageChops.subtract(edge, interior))
    return Image.alpha_composite(wash, outline)


def activity_glyph(draw: ImageDraw.ImageDraw, activity: Activity, point: tuple[int, int]) -> None:
    x, y = point
    match activity:
        case Activity.CHOP:
            draw.line((x + 1, y + 12, x + 9, y + 3), fill=INK, width=3)
            draw.line((x + 1, y + 12, x + 9, y + 3), fill=(187, 139, 78), width=1)
            draw.polygon(((x + 5, y + 3), (x + 9, y), (x + 13, y + 1),
                          (x + 13, y + 6), (x + 9, y + 6)), fill=(205, 210, 194), outline=INK)
        case Activity.CARRY:
            draw.rounded_rectangle((x, y + 1, x + 13, y + 7), 2, fill=(157, 109, 66), outline=INK)
            draw.line((x + 2, y + 3, x + 8, y + 3), fill=(201, 148, 84))
            draw.ellipse((x + 8, y, x + 13, y + 7), fill=(215, 171, 109), outline=INK)
            draw.arc((x + 10, y + 2, x + 12, y + 5), 40, 310, fill=(128, 85, 44))
        case Activity.SLEEP:
            draw.text((x, y - 4), "zz", fill=(232, 231, 205), font=FONT, stroke_width=1, stroke_fill=INK)
        case Activity.AWAKE:
            pass


@dataclass(frozen=True, slots=True)
class ForegroundPainter:
    image: Image.Image
    viewport: Viewport
    catalog: Catalog
    lod: bool
    coverage: Image.Image | None = None
    face_plan: FacePlan | None = None

    def chopping_context(self, actor: RenderPerson, foot: tuple[int, int]) -> dict[str, JsonValue] | None:
        if not actor.declared or actor.activity is not Activity.CHOP:
            return None
        source = self.catalog.image("stump")
        scale = .5 * self.viewport.zoom
        context = source.resize((max(1, round(source.width * scale)), max(1, round(source.height * scale))),
                                Image.Resampling.NEAREST)
        center = (foot[0] + round(22 * self.viewport.zoom), foot[1] - round(4 * self.viewport.zoom))
        left, upper = center[0] - context.width // 2, center[1] - context.height // 2
        self.image.alpha_composite(context, (left, upper))
        if self.coverage is not None:
            self.coverage.paste(0, (left, upper), context.getchannel("A").point([255 if n >= 128 else 0 for n in range(256)]))
        return {"asset": "stump", "declared": True, "source": "declared_activity_context",
                "scale": scale, "status": self.catalog.asset("stump").status,
                "bbox": [left, upper, left + context.width, upper + context.height]}

    def sprite(self, sprite: Sprite) -> dict[str, JsonValue] | None:
        if not self.viewport.visible(sprite.position, round(160 * self.viewport.zoom)):
            return None
        source = self.catalog.image(sprite.asset)
        px, py = self.viewport.project(sprite.position)
        if sprite.zone:
            half = self.viewport.tile // 2
            bbox = [px - half, py - half, px + half, py + half]
            overlay = Image.new("RGBA", self.image.size)
            ImageDraw.Draw(overlay).rectangle((bbox[0], bbox[1], bbox[2] - 1, bbox[3] - 1), outline=(174, 172, 126, 75), width=1)
            self.image.alpha_composite(overlay)
            foreground_bbox = bbox
        elif sprite.category == "animal" and self.lod and self.viewport.zoom <= .5:
            size = 10 if self.viewport.zoom == .5 else 6
            colour = (184, 67, 54) if sprite.asset.startswith("wolf") else (194, 157, 97)
            ImageDraw.Draw(self.image).polygon(((px, py - size // 2), (px - size // 2, py + size // 2 - 1),
                                                (px + size // 2 - 1, py + size // 2 - 1)),
                                               fill=colour, outline=INK)
            if self.coverage is not None:
                ImageDraw.Draw(self.coverage).polygon(((px, py - size // 2), (px - size // 2, py + size // 2 - 1),
                                                       (px + size // 2 - 1, py + size // 2 - 1)), fill=0)
            bbox = [px - size // 2, py - size // 2, px + size // 2, py + size // 2]
            foreground_bbox = bbox
        else:
            scaled = source.resize((max(1, round(source.width * self.viewport.zoom)),
                                    max(1, round(source.height * self.viewport.zoom))), Image.Resampling.NEAREST)
            if sprite.site:
                scaled = site_overlay(scaled)
            anchor = self.catalog.asset(sprite.asset).anchor or (source.width / 2, source.height / 2)
            ground_offset = round(11 * self.viewport.zoom) if sprite.category == "animal" else 0
            left = px - round(anchor[0] * self.viewport.zoom)
            upper = py + ground_offset - round(anchor[1] * self.viewport.zoom)
            self.image.alpha_composite(scaled, (left, upper))
            if self.coverage is not None:
                self.coverage.paste(0, (left, upper), scaled.getchannel("A").point([255 if n >= 128 else 0 for n in range(256)]))
            bbox = [left, upper, left + scaled.width, upper + scaled.height]
            alpha_box = scaled.getchannel("A").point([255 if n >= 16 else 0 for n in range(256)]).getbbox()
            foreground_bbox = ([left + alpha_box[0], upper + alpha_box[1], left + alpha_box[2], upper + alpha_box[3]]
                               if alpha_box is not None else bbox)
        result: dict[str, JsonValue] = {"id": sprite.id, "asset": sprite.asset, "declared": sprite.declared,
                "position": list(sprite.position), "bbox": [value for value in bbox],
                "foreground_bbox": [value for value in foreground_bbox],
                "status": "CODE_ZONE_OUTLINE" if sprite.zone else self.catalog.asset(sprite.asset).status,
                "source": "declared_overlay" if sprite.declared else sprite.source,
                "native_kind": sprite.native_kind, "native_count": sprite.native_count, "site": sprite.site}
        return result

    def person(self, actor: RenderPerson, coverage_label: int = 1) -> dict[str, JsonValue] | None:
        px, py = self.viewport.project(actor.position)
        if not (0 <= px < WINDOW[0] and HUD_HEIGHT <= py < WINDOW[1]):
            return None
        look = choose_look(actor.person, actor.top)
        variant: Variant | None = None
        selection_age = 0
        if self.face_plan is not None:
            selection_age = variant_selection_age(actor.person)
            variant = choose_variant(actor.person.id, selection_age, self.face_plan.for_look(look))
        colour = top_colour(look)
        draw = ImageDraw.Draw(self.image)
        foot_y = py + round(11 * self.viewport.zoom)
        context: dict[str, JsonValue] | None = None
        source_path: Path | None = None
        if self.lod and self.viewport.zoom <= .5:
            size = 10 if self.viewport.zoom == .5 else 6
            bbox = [px - size // 2, foot_y - size // 2,
                    px - size // 2 + size, foot_y - size // 2 + size]
            draw.ellipse((bbox[0], bbox[1], bbox[2] - 1, bbox[3] - 1), fill=colour, outline=INK, width=1)
            if self.coverage is not None:
                ImageDraw.Draw(self.coverage).ellipse((bbox[0], bbox[1], bbox[2] - 1, bbox[3] - 1), fill=coverage_label)
            if self.viewport.zoom == .5:
                activity_glyph(draw, actor.activity, (px + 6, foot_y - 7))
            foreground_bbox = bbox
        else:
            frame = activity_frame(actor.activity)
            path = (REPO / "art/character/route_i" / look / variant.id / "S" / f"{frame}.png"
                    if variant is not None else frame_path(look, frame))
            source = read_image(path)
            source_path = path
            scaled = source.resize((round(source.width * self.viewport.zoom), round(source.height * self.viewport.zoom)),
                                   Image.Resampling.NEAREST)
            anchor_x, anchor_y = (56, 72) if frame == "sleep" else (40, 104)
            left, upper = px - round(anchor_x * self.viewport.zoom), foot_y - round(anchor_y * self.viewport.zoom)
            shadow = Image.new("RGBA", self.image.size)
            ImageDraw.Draw(shadow).ellipse((px - round(14 * self.viewport.zoom), foot_y - round(4 * self.viewport.zoom),
                                           px + round(14 * self.viewport.zoom), foot_y + round(4 * self.viewport.zoom)),
                                          fill=(16, 15, 12, 95))
            self.image.alpha_composite(shadow)
            context = self.chopping_context(actor, (px, foot_y))
            self.image.alpha_composite(scaled, (left, upper))
            if self.coverage is not None:
                self.coverage.paste(coverage_label, (left, upper), scaled.getchannel("A").point([255 if n >= 128 else 0 for n in range(256)]))
            bbox = [left, upper, left + scaled.width, upper + scaled.height]
            alpha_box = scaled.getchannel("A").point([255 if n >= 16 else 0 for n in range(256)]).getbbox()
            if alpha_box is None:
                raise ValueError(f"Empty adopted person frame: {look}/{frame}")
            foreground_bbox = [left + alpha_box[0], upper + alpha_box[1],
                               left + alpha_box[2], upper + alpha_box[3]]
        mark_anchor = [px - 5, foreground_bbox[1] - 18]
        if actor.mark:
            draw.text((mark_anchor[0], mark_anchor[1]), actor.mark, font=FONT, fill=(246, 238, 207), stroke_width=2, stroke_fill=INK)
        result: dict[str, JsonValue] = {"id": actor.person.id, "source_position": [actor.person.x, actor.person.y],
                "position": list(actor.position), "screen_foot": [px, foot_y], "bbox": [value for value in bbox],
                "look": look, "top_colour": list(colour), "activity": actor.activity.value,
                "source_activity": actor.person.activity.value, "declared": actor.declared,
                "source_female": actor.person.female, "source_life_stage": actor.person.life_stage,
                "source_carry_kind": actor.person.carry_kind, "source_carry_count": actor.person.carry_count,
                "mark": actor.mark, "source": actor.person.source}
        result["foreground_bbox"] = [value for value in foreground_bbox]
        if variant is not None:
            result["variant"] = variant.id
            result["variant_selection_age"] = selection_age
            result["variant_age_basis"] = "life_stage_representative_not_captured_age"
            result["variant_description"] = variant.description
            if source_path is not None:
                result["variant_frame"] = str(source_path.relative_to(REPO))
                result["variant_frame_sha256"] = hashlib.sha256(source_path.read_bytes()).hexdigest()
        result["mark_anchor"] = [value for value in mark_anchor] if actor.mark else None
        result["declared_visual_context"] = context
        if self.coverage is not None:
            result["opaque_pixels_when_drawn"] = self.coverage.histogram()[coverage_label]
        return result
