"""Render corrected native sprites at exact 4x and 1x scales without generation."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Final, TypedDict

import numpy as np
from PIL import Image

from palette_tools import load_palette, rgb_to_lab
from presentation import hud_layer, theme_for
from scene_ground import ground_image, ground_roles
from scene_layout import Placement, layout, load_sprites, render, attribution_mask, clothing_variant

CROP: Final = (272, 377, 752, 647)
MAP_OFFSET: Final = (448, 28)


class VillagerMetric(TypedDict):
    attributed_pixel_count: int
    id: int
    sprite: str
    box: list[int]
    changed_fraction: float
    visible: bool
    mean_luma_difference: float
    mean_delta_e76: float
    box_mean_luma_difference: float
    box_mean_delta_e76: float


class ReadabilityReport(TypedDict):
    zoom: int
    villagers: list[VillagerMetric]
    visible_fraction: float
    mean_luma_difference: float
    mean_delta_e76: float
    box_mean_luma_difference: float
    box_mean_delta_e76: float
    target: float
    target_pass: bool
    method: str


class GroundReport(TypedDict):
    grass: tuple[tuple[int,int,int], ...]
    earth: tuple[tuple[int,int,int], ...]
    water: tuple[tuple[int,int,int], ...]
    shore: tuple[int,int,int]


class PlacementReport(TypedDict):
    sprite: str
    anchor_world: list[int]
    villager_id: int | None


class ClothingChange(TypedDict):
    sprite: str
    hue_degrees: int
    changed_pixels: int


class SceneReport(TypedDict):
    direction: str
    zoom4: ReadabilityReport
    zoom1: ReadabilityReport
    world_size: list[int]
    zoom4_native_crop: list[int]
    overview_offset: list[int]
    sprite_resampling: bool
    ground_roles: GroundReport
    ground_palette_source: str
    layout_source: str
    layout_method: str
    clothing_changes: list[ClothingChange]
    clothing_variation: str
    tile_variant: str
    tile_border_exception: str
    night_tint: list[float]
    glow_rgb: list[float]
    hud_tinted: bool
    placements: list[PlacementReport]


def frame(world: Image.Image, zoom: int, hud: Image.Image) -> Image.Image:
    if zoom == 4:
        result = world.crop(CROP).resize((1920,1080),Image.Resampling.NEAREST)
    elif zoom == 1:
        result = Image.new("RGBA",(1920,1080),(20,24,28,255))
        result.alpha_composite(world,MAP_OFFSET)
    else:
        message = f"Only exact integer zoom 1 and 4 are supported, got {zoom}"
        raise ValueError(message)
    return Image.alpha_composite(result,hud)


def night(world: Image.Image) -> Image.Image:
    rgb = np.asarray(world.convert("RGB"),dtype=np.float64)
    yy,xx = np.indices(rgb.shape[:2])
    glow = np.maximum(0,1-np.hypot(xx-512,yy-498)/67)**2
    rgb = rgb*np.array((.42,.48,.68))+glow[...,None]*np.array((1,.6,.25))*150
    return Image.fromarray(np.clip(np.rint(rgb),0,255).astype(np.uint8)).convert("RGBA")


def screen_box(placement: Placement, zoom: int) -> tuple[int,int,int,int]:
    left,top,right,bottom = placement.box
    if zoom == 4:
        return ((left-CROP[0])*4,(top-CROP[1])*4,(right-CROP[0])*4,(bottom-CROP[1])*4)
    return left+MAP_OFFSET[0],top+MAP_OFFSET[1],right+MAP_OFFSET[0],bottom+MAP_OFFSET[1]


def attribution_frame(placements: list[Placement], zoom: int, hud: Image.Image | None) -> Image.Image:
    world = attribution_mask(placements)
    if zoom == 4:
        result = world.crop(CROP).resize((1920,1080),Image.Resampling.NEAREST)
    else:
        result = Image.new("L",(1920,1080))
        result.paste(world,MAP_OFFSET)
    if hud is not None:
        result.paste(0,(0,0,1920,1080),hud.getchannel("A").point(lambda alpha: 255 if alpha else 0))
    return result


def readability(full: Image.Image, removed: Image.Image, placements: list[Placement], zoom: int,
                hud: Image.Image | None = None) -> ReadabilityReport:
    """Compare two complete scene renders, retaining HUD and object occlusion."""
    first = np.asarray(full.convert("RGB"),dtype=np.float64)
    second = np.asarray(removed.convert("RGB"),dtype=np.float64)
    luma = np.abs((first-second)@np.array((.2126,.7152,.0722)))/255
    delta = np.linalg.norm(rgb_to_lab(first)-rgb_to_lab(second),axis=-1)
    ids = np.asarray(attribution_frame(placements,zoom,hud))
    rows: list[VillagerMetric] = []
    for placement in placements:
        if placement.villager_id is None:
            continue
        x0,y0,x1,y1 = screen_box(placement,zoom)
        x0,y0,x1,y1 = max(0,x0),max(0,y0),min(1920,x1),min(1080,y1)
        patch = luma[y0:y1,x0:x1]
        changed = float(np.mean(patch >= .12)) if patch.size else 0.0
        attributed = ids[y0:y1,x0:x1] == placement.villager_id
        attributed_count = int(attributed.sum())
        delta_patch = delta[y0:y1,x0:x1]
        rows.append({"attributed_pixel_count":attributed_count,"id":placement.villager_id,"sprite":placement.sprite.name,
                     "box":[x0,y0,x1,y1],"changed_fraction":changed,"visible":changed>=.20,
                     "mean_luma_difference":float(patch[attributed].mean()) if attributed_count else 0.0,
                     "mean_delta_e76":float(delta_patch[attributed].mean()) if attributed_count else 0.0,
                     "box_mean_luma_difference":float(patch.mean()) if patch.size else 0.0,
                     "box_mean_delta_e76":float(delta_patch.mean()) if patch.size else 0.0})
    ratio = sum(bool(row["visible"]) for row in rows)/len(rows)
    return {"zoom":zoom,"villagers":rows,"visible_fraction":ratio,
            "mean_luma_difference":float(np.mean([row["mean_luma_difference"] for row in rows])),
            "mean_delta_e76":float(np.mean([row["mean_delta_e76"] for row in rows])),
            "box_mean_luma_difference":float(np.mean([row["box_mean_luma_difference"] for row in rows])),
            "box_mean_delta_e76":float(np.mean([row["box_mean_delta_e76"] for row in rows])),
            "target":.90 if zoom==4 else .75,"target_pass":ratio>=(.90 if zoom==4 else .75),
            "method":"Absolute Rec.709 sRGB luma delta /255; changed pixels >=0.12, visible >=20% full native sprite box. Full scene versus all villagers and their shadows removed; y-sort/object/HUD occlusion retained. Mean contrast uses only opaque texels attributed to that villager by a y-sorted ID mask; nonvillager sprites and HUD clear attribution. Fully hidden villagers have zero mean and attributed_pixel_count=0. Separate box_mean fields preserve full-box values. Underlay is the complete villagers-removed scene, including ground and objects."}


def produce(root: Path, direction: str) -> SceneReport:
    directory = root/direction
    sprites = load_sprites(directory)
    palette = load_palette(directory/"palette_v2.hex")
    ground_directory = root/("D1" if direction=="H" else direction)
    ground_palette_path = directory/"ground_palette_v2.hex"
    ground_palette = load_palette(ground_palette_path if ground_palette_path.exists() else ground_directory/"palette.hex")
    roles = ground_roles(ground_palette)
    placements = layout(sprites,"D1" if direction=="H" else direction)
    ground = ground_image(roles,"D1" if direction=="H" else direction)
    world,without = render(ground,placements),render(ground,placements,False)
    theme = theme_for(palette)
    hud = hud_layer(theme)
    full4, removed4 = frame(world,4,hud),frame(without,4,hud)
    full1, removed1 = frame(world,1,hud),frame(without,1,hud)
    outputs = {"ingame_noon.png":full4,"ingame_night.png":frame(night(world),4,hud_layer(theme,True)),
               "ingame_overview.png":full1,"ingame_noon_without_villagers.png":removed4,
               "ingame_overview_without_villagers.png":removed1}
    for filename,image in outputs.items():
        image.convert("RGB").save(directory/filename)
    clothing_changes: list[ClothingChange] = []
    for name,sprite in sprites.items():
        if "villager" in name:
            for amount in (-1,1):
                original = np.asarray(sprite.image)
                varied = np.asarray(clothing_variant(sprite,amount).image)
                count = int(np.sum(np.any(original[...,:3] != varied[...,:3],axis=-1)))
                clothing_changes.append({"sprite":name,"hue_degrees":amount,"changed_pixels":count})
    report: SceneReport = {"direction":direction,"zoom4":readability(full4,removed4,placements,4,hud),
                              "zoom1":readability(full1,removed1,placements,1,hud),
                              "world_size":[1024,1024],"zoom4_native_crop":list(CROP),
                              "overview_offset":list(MAP_OFFSET),"sprite_resampling":False,
                              "ground_roles":{"grass":roles.grass,"earth":roles.earth,"water":roles.water,"shore":roles.shore},
                              "ground_palette_source":str(ground_palette_path if ground_palette_path.exists() else ground_directory/"palette.hex"),
                              "layout_source":f"{('D1' if direction=='H' else direction)}_F2_noon.png",
                              "layout_method":"Manually read concept arrangements; preserve relative hut/storage/frame/river/forest positions on native world grid.",
                              "clothing_changes":clothing_changes,
                              "clothing_variation":"Attempted -1/0/+1 degree hue using low-saturation central lower-torso proxy; may include skin. Neutral pixels and integer RGB rounding can be no-ops. Not a verified garment segmentation.",
                              "tile_variant":"((x*73856093)^(y*19349663))%4",
                              "tile_border_exception":"Continuous river/shore geometry crosses tiles; local grass, earth marks and water glints stay inside borders.",
                              "night_tint":[.42,.48,.68],"glow_rgb":[1,.6,.25],"hud_tinted":False,
                              "placements":[{"sprite":p.sprite.name,"anchor_world":[p.x,p.y],"villager_id":p.villager_id} for p in placements]}
    (directory/"readability_v2.json").write_text(json.dumps(report,indent=2)+"\n")
    return report


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root",type=Path,default=Path("docs/art"))
    parser.add_argument("--directions",nargs="+",default=["D1","D2","D3","H"])
    args = parser.parse_args()
    for direction in args.directions:
        result = produce(args.root,direction)
        print(json.dumps({"direction":direction,"zoom4":result["zoom4"],"zoom1":result["zoom1"]}))


if __name__ == "__main__":
    main()
