"""Native sprite placement and deterministic clothing-only variation."""
from __future__ import annotations

import colorsys
import json
from dataclasses import dataclass
from collections.abc import Mapping
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw


@dataclass(frozen=True, slots=True)
class Sprite:
    name: str
    image: Image.Image
    anchor: tuple[int, int]


@dataclass(frozen=True, slots=True)
class Placement:
    sprite: Sprite
    x: int
    y: int
    villager_id: int | None = None

    @property
    def box(self) -> tuple[int, int, int, int]:
        x, y = self.x-self.sprite.anchor[0], self.y-self.sprite.anchor[1]
        return x, y, x+self.sprite.image.width, y+self.sprite.image.height


def load_sprites(directory: Path) -> Mapping[str, Sprite]:
    manifest = json.loads((directory / "sprites_v2/manifest.json").read_text())
    sprites: dict[str, Sprite] = {}
    for entry in manifest["sprites"]:
        name = str(entry["name"])
        filename = str(entry.get("file", name + ".png"))
        with Image.open(directory / "sprites_v2" / filename) as source:
            image = source.convert("RGBA")
        anchor = entry["anchor"]
        sprites[name] = Sprite(name, image, (int(anchor[0]), int(anchor[1])))
    return sprites


def clothing_variant(sprite: Sprite, amount: int) -> Sprite:
    """Apply a low-saturation central lower-torso proxy; skin can overlap this mask."""
    rgba = np.array(sprite.image)
    hsv = np.array(sprite.image.convert("RGB").convert("HSV"))
    height, width = rgba.shape[:2]
    yy, xx = np.indices((height, width))
    mask = ((yy >= height*.53) & (yy < height*.83) & (xx >= width*.22)
            & (xx < width*.78) & (rgba[...,3] > 0) & (hsv[...,1] < 85))
    for color in np.unique(rgba[mask,:3],axis=0):
        h,s,v = colorsys.rgb_to_hsv(*(int(c)/255 for c in color))
        shifted = tuple(round(c*255) for c in colorsys.hsv_to_rgb((h+amount/360)%1,s,v))
        matches = mask & np.all(rgba[...,:3] == color,axis=-1)
        rgba[matches,:3] = shifted
    return Sprite(sprite.name, Image.fromarray(rgba), sprite.anchor)


def layout(sprites: Mapping[str, Sprite], direction: str) -> list[Placement]:
    placements: list[Placement] = []
    def put(name: str, x: int, y: int) -> None:
        placements.append(Placement(sprites[name], 512+x, 512+y))
    huts = ((-48,-35),(10,-68),(73,-27))
    storage, frame = (-46,61), (78,63)
    if direction == "D2":
        huts = ((-30,-52),(69,-37),(-48,63))
        storage, frame = (94,23), (84,87)
    for x,y in huts:
        put("02_lean_to",x,y)
    put("01_campfire",0,0)
    put("03_storage_pit",*storage)
    put("04_shelter_frame",*frame)
    put("16_berry_basket",storage[0]+22,storage[1]+5)
    for index,(x,y) in enumerate(((74,-77),(101,-62),(128,-77),(146,-54)),5):
        put(f"{index:02d}_berry_bush_{('full','two','one','bare')[index-5]}",x,y)
    put("13_boulder",149,-25)
    for x,y in ((-105,-40),(-108,41),(-110,101)):
        put("15_reed_tuft",x,y)
    for y in range(272,785,32):
        for x in range(272,785,32):
            dx,dy=x-512,y-512
            if (dx/165)**2+(dy/113)**2 > 1 and not 345<x<411:
                put("14_young_tree",dx+((y//32)%2)*8,dy)
    people = ((-22,-4),(28,9),(-9,29),(48,-23),(-51,8),(-74,34),(-63,-24),
              (16,48),(51,51),(83,45),(107,61),(-35,60),(-16,82),(-76,83),
              (95,-52),(115,-28),(48,-66),(-91,-7),(-99,65),(27,85))
    types = ("09_villager_gatherer","10_villager_builder","11_villager_elder","12_villager_child")
    for index,(x,y) in enumerate(people):
        sprite = clothing_variant(sprites[types[index%4]], (index%3)-1)
        placements.append(Placement(sprite,512+x,512+y,index+1))
    return sorted(placements,key=lambda p:(p.y,p.x))


def render(ground: Image.Image, placements: list[Placement], villagers: bool = True) -> Image.Image:
    """A separate shared 10x3 shadow is drawn immediately before each y-sorted actor."""
    result = ground.copy()
    shadow = Image.new("RGBA",(10,3))
    ImageDraw.Draw(shadow).ellipse((0,0,9,2),fill=(0,0,0,72))
    for placement in placements:
        if placement.villager_id is not None and not villagers:
            continue
        result.alpha_composite(shadow,(placement.x-5,placement.y-2))
        result.alpha_composite(placement.sprite.image,placement.box[:2])
    return result


def attribution_mask(placements: list[Placement]) -> Image.Image:
    """Track the last opaque sprite at each world texel in the render order."""
    result = Image.new("L", (1024, 1024))
    for placement in placements:
        opaque = placement.sprite.image.getchannel("A").point(lambda alpha: 255 if alpha else 0)
        result.paste(placement.villager_id or 0, placement.box, opaque)
    return result
