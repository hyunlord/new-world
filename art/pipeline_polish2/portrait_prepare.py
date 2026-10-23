# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "pydantic", "typer"]
# ///
# How to run: python3 -m art.pipeline_polish2.portrait_prepare
"""Freeze variant portrait prompts and hash-bind their actual tinted sprites."""
from __future__ import annotations

from pathlib import Path

import typer
from pydantic import TypeAdapter

from art.compositor.face_variants import (
    PLAN_PATH,
    FacePlan,
    LookVariants,
    Variant,
    baked_frame,
)
from art.pipeline_polish2.blind_round import seal_model
from art.pipeline_polish2.portrait_appearance import appearance_for, enlarge_reference
from art.pipeline_polish2.portrait_models import (
    ANCHOR,
    DATA,
    ROOT,
    AssetRef,
    Request,
    Requests,
    SourceAppearance,
    ToolRequest,
    write_evidence,
)


def prompt_for(look: LookVariants, variant: Variant, appearance: SourceAppearance) -> str:
    """Describe intentional identity axes while preserving sprite outfit/hair shape."""
    facial_hair = {
        "full": "Facial hair: a clearly full, substantial beard covering the chin and jaw. ",
        "short": "Facial hair: a very short, close-cropped beard following the jaw; no bulky or hanging beard. ",
        "clean": "Facial hair: completely clean shaven, with bare visible chin, jaw and upper lip; no beard, moustache or stubble. ",
        "none": "",
    }[variant.beard]
    introduction = (
        "Create ONE square inspector portrait, shoulders-up bust, for the planned identity variant of reference 1. "
        + facial_hair
        + "Facial hair is an intentional change: do not copy any beard from either sprite or style reference. "
        if facial_hair else
        "Create ONE square inspector portrait, shoulders-up bust, for the exact person in reference 1. "
    )
    return introduction + (
        "Reference 1 is that person's actual Route I S/idle sprite tinted to the planned skin and hair "
        "colours. Reference 2 is portrait.png, STYLE ONLY; do not copy its man's identity. "
        "Reference 3 is an exact 4x nearest-neighbor enlargement of reference 1; use it to read "
        "the tiny garment, collar and hairstyle without inventing hidden details. "
        f"Identity brief: {variant.description}. Asset identifier only: {look.look}. "
        f"Source-observed appearance, copied from the source review: {appearance.appearance_brief} "
        "Follow that observed clothing and hairstyle rather than interpreting the asset identifier. "
        f"Skin midtone RGB {variant.skin_rgb}; hair midtone RGB {variant.hair_rgb}. "
        "Preserve the sprite's hairstyle silhouette and exact garment and neckline, including which "
        "shoulder is covered. The written age, facial features and beard are intentional identity "
        "differences; make them clearly visible. The identity brief overrides any source-observed "
        "beard, age or colours, but never its garment coverage or hairstyle silhouette. "
        "Respect no-beard children and women. Do not copy "
        "accessories, ornaments, necklace or face from the style anchor. Neutral natural expression. "
        "Front three-quarter view in controlled dark-brown ink and restrained earth-colour watercolor "
        "washes, natural-history illustration, warm parchment palette, upper-left lighting. "
        "Use broad lit and shaded planes and selective hair lines, not dense all-over hatching. "
        "No photography or 3D. Composition must survive full-frame downsampling to 128x128 and 64x64. "
        "Entire head, ears, hairstyle and flyaways must fit within x=14..114, y=10..104 in final "
        "128px coordinates. Keep face skin inside fixed ellipse center(64,61), radii(44,52). "
        "Shoulders may meet the bottom edge, hair must not meet any edge. Quiet plain warm parchment "
        "background. No text, labels, border, watermark, landscape, jewellery or head ornaments. "
        "Render one finished portrait, not a sheet or alternatives."
    )


def build_request(root: Path, raw_directory: Path, identity: tuple[LookVariants, Variant]) -> Request:
    look, variant = identity
    appearance_ref, appearance = appearance_for(root, look.look)
    sprite = AssetRef.of(baked_frame(root, look, variant))
    enlarged = enlarge_reference(sprite, raw_directory / "references")
    return Request(look=look.look, variant=variant.id, description=variant.description,
                   prompt=prompt_for(look, variant, appearance),
                   references=(sprite, AssetRef.of(root / ANCHOR.relative_to(ROOT)), enlarged),
                   face_plan=AssetRef.of(root / PLAN_PATH.relative_to(ROOT)),
                   appearance_manifest=appearance_ref, source_appearance=appearance,
                   canonical_path=root / "art/portrait/polish2" / look.look / f"{variant.id}.png",
                   raw_directory=raw_directory)


def tool_request(row: Request, request_path: Path) -> ToolRequest:
    return ToolRequest(id=f"{row.look}/{row.variant}", look=row.look, variant=row.variant, attempt=row.attempt,
                       prompt=row.prompt, referenced_image_paths=tuple(str(ref.path) for ref in row.references),
                       references=row.references, canonical_path=row.canonical_path,
                       raw_directory=row.raw_directory, request_path=request_path)


def started(directory: Path) -> bool:
    return any(directory.glob("try*"))


def prepare(root: Path = ROOT, output: Path = DATA, available_only: bool = False, unstarted_only: bool = False) -> Requests:
    """Freeze all currently available tinted inputs; fail if a required input is absent."""
    plan_path = root / PLAN_PATH.relative_to(ROOT)
    plan = FacePlan.read(plan_path)
    priority = {look: index for index, look in enumerate(plan.owner_sample)}
    skipped = sum(started(output / "generation" / look.look / variant.id)
                  for look in plan.looks for variant in look.variants) if unstarted_only else 0
    rows = tuple(
        build_request(root, output / "generation" / look.look / variant.id, (look, variant))
        for look in sorted(plan.looks, key=lambda row: (priority.get(row.look, 999), row.look))
        for variant in look.variants
        if not unstarted_only or not started(output / "generation" / look.look / variant.id)
        if not available_only or baked_frame(root, look, variant).is_file()
    )
    expected = sum(len(look.variants) for look in plan.looks)
    result = Requests(requests=rows, logical_fallbacks=plan.fallbacks,
                      expected_count=expected, available_count=len(rows), complete=len(rows) == expected,
                      skipped_started_count=skipped)
    for row in rows:
        request_path = row.raw_directory / "request.json"
        if request_path.is_file() and any(row.raw_directory.glob("try*")):
            previous = Request.model_validate_json(request_path.read_bytes())
            if previous != row:
                message = f"Cannot rewrite a request after generation began: {row.look}/{row.variant}"
                raise ValueError(message)
        write_evidence(request_path, row)
    write_evidence(output / "requests.json", result)
    queue = tuple(tool_request(row, row.raw_directory / "request.json") for row in rows)
    _ = (output / "queue.json").write_bytes(TypeAdapter(tuple[ToolRequest, ...]).dump_json(queue, indent=2) + b"\n")
    return result


def prepare_retry(request_path: Path, correction: str, root: Path = ROOT) -> Request:
    """Create a separate immutable second-attempt request without rewriting first-attempt evidence."""
    previous = Request.model_validate_json(request_path.read_bytes())
    if previous.attempt != 1:
        message = "Retry preparation requires a first-attempt request; no third attempt is permitted"
        raise ValueError(message)
    if not correction.strip():
        message = "Retry preparation requires an explicit correction prompt"
        raise ValueError(message)
    if not ((previous.raw_directory / "try1.png").is_file() or (previous.raw_directory / "try1/raw.png").is_file()):
        message = "Retry preparation requires preserved first-attempt raw evidence"
        raise ValueError(message)
    previous.face_plan.verify()
    current_plan = root / PLAN_PATH.relative_to(ROOT)
    if AssetRef.of(current_plan).sha256 != previous.face_plan.sha256:
        message = "Retry must use the same frozen face plan as the first attempt"
        raise ValueError(message)
    look = FacePlan.read(current_plan).for_look(previous.look)
    variant = next(row for row in look.variants if row.id == previous.variant)
    fresh = build_request(root, previous.raw_directory, (look, variant))
    retry = fresh.model_copy(update={"attempt": 2, "previous_request": AssetRef.of(request_path),
        "correction": correction, "prompt": f"Second and final attempt. Explicit correction: {correction}\n{fresh.prompt}"})
    destination = previous.raw_directory / "try2.request.json"
    _ = seal_model(destination, retry)
    _ = seal_model(previous.raw_directory / "try2.queue.json", tool_request(retry, destination))
    return retry


def main(available_only: bool = False, unstarted_only: bool = False,
         retry_request: Path | None = None, correction: str = "") -> None:
    if retry_request is not None:
        result = prepare_retry(retry_request, correction)
        print(f"Prepared attempt2 only: {result.raw_directory / 'try2.queue.json'}")
        return
    result = prepare(available_only=available_only, unstarted_only=unstarted_only)
    print(f"Prepared {result.available_count}/{result.expected_count}; complete={result.complete}; queue={DATA / 'queue.json'}")


if __name__ == "__main__":
    typer.run(main)
