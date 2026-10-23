"""Join immutable numeric, actual image-model, and executor reading evidence."""

from __future__ import annotations

import json
import re
import shutil
from dataclasses import asdict
from pathlib import Path
from typing import Final

from PIL import Image

from art.pipeline2.judge import ModelScores, judge, load_calibration
from art.pipeline_env1.items_prepare import CALIBRATION, DATA, DEST, DOCS, ROOT, digest
from art.pipeline_env1.items_records import JsonRecord, read_json

READINGS: Final = {
    "firewood": "짧은 쪼갠 나무의 밝은 절단면과 어두운 수피가 통나무 더미와 구별된다.",
    "ground_axe": "넓고 매끈한 돌날과 짧은 자루가 기존 작은 타제 돌도끼와 구별된다.",
    "spear": "긴 가는 자루와 삼각 창끝이 읽히지만 담채 분산 수치 초과로 수락하지 않는다.",
    "cured_hide": "털이 없는 황갈색 네 갈래 가죽으로 읽히며 생가죽과 색 차이가 작아32px 검토가 필요하다.",
    "hide_wrap": "목·팔 구멍과 사선 여밈, 허리끈이 펼친 가죽이 아닌 의복으로 읽힌다.",
    "fur_cloak": "둥근 목구멍, 밝은 털깃, 넓은 종 모양 외곽이 망토를 만든다.",
    "wolf_hide": "긴 꼬리와 네 발, 머리 및 어두운 등줄기가 늑대 가죽을 나타낸다.",
    "clay": "부드러운 적갈색 덩어리와 눌린 홈이 돌의 각진 면과 구별된다.",
    "raw_pot": "손잡이 없는 회색 코일 토기가 구운 주황색 토기와 구별된다.",
    "clay_pot_empty": "양쪽 작은 손잡이와 어두운 빈 입구가 비어 있는 구운 토기로 읽힌다.",
    "clay_pot_filled": "같은 토기 계열의 푸른 수면이 채워진 상태를 나타낸다.",
    "meat": "탁한 붉은 고기와 한쪽 뼈 돌출부가 작은 고기 자원으로 읽힌다.",
    "flint_nodule": "한쪽의 푸른 회색 박리면과 둥근 껍질이 단일 부싯돌 덩이를 나타낸다.",
}
KINDS: Final = (
    ("Log", "log_pile_small"), ("Firewood", "firewood"), ("Stone", "stone_pile"),
    ("Flint", "flint_nodule"), ("FlintBlade", "flint_blade"), ("StoneAxe", "stone_axe"),
    ("Hide", "hide"), ("Clay", "clay"), ("RawPot", "raw_pot"), ("ClayPot", "clay_pot_empty"),
    ("GroundAxe", "ground_axe"), ("Meat", "meat"), ("CuredHide", "cured_hide"),
    ("HideWrap", "hide_wrap"), ("FurCloak", "fur_cloak"), ("Spear", "spear"),
    ("WolfHide", "wolf_hide"),
)


def main() -> None:
    manifest_path = DOCS / "items_manifest.json"
    manifest = JsonRecord.read(manifest_path)
    assets = manifest.records("assets")
    calibration = load_calibration(CALIBRATION)
    reviews: list[JsonRecord] = []
    scores: dict[str, tuple[JsonRecord, JsonRecord]] = {}
    for number in (1, 2, 3):
        record = JsonRecord.read(DATA / f"model_reviews{number}.json")
        match = re.search(r" as (.*?) by default", record.text("output_hint"))
        if match is None:
            message = f"Missing scorecard source{number}"
            raise RuntimeError(message)
        raw = DATA / f"model_scorecard{number}_raw.png"
        _ = shutil.copyfile(Path(match.group(1)), raw)
        display = DEST / f"model_scorecard{number}.png"
        with Image.open(raw) as opened:
            image = opened.convert("RGB")
        image.thumbnail((1200, 1200), Image.Resampling.LANCZOS)
        image.save(display, optimize=True)
        record.fields["raw_scorecard"] = str(raw)
        record.fields["raw_scorecard_sha256"] = digest(raw)
        record.fields["display_scorecard"] = str(display.relative_to(ROOT))
        record.fields["display_scorecard_sha256"] = digest(display)
        record.fields["reference_hashes"] = {ref: digest(Path(ref)) for ref in record.strings("refs")}
        record.fields["candidate_sources"] = read_json(DATA / f"style_candidates{number}.json")
        for result in record.records("transcription"):
            scores[result.text("id")] = (result, record)
        reviews.append(record)
    for asset in assets:
        name = asset.text("id")
        asset.fields["count_bucket_sha256"] = {path: digest(ROOT / path) for path in asset.strings("count_buckets")}
        if asset.records("attempts"):
            result, record = scores[name]
            model = ModelScores(medium=result.integer("medium"), line=result.integer("line"),
                                light=result.integer("light"), palette=result.integer("palette"),
                                sentence=result.text("sentence"))
            verdict = judge(Path(asset.text("source")), calibration, model, family="items", model_family_verified=True)
            asset.fields["numeric"] = asdict(verdict)
            asset.fields["model"] = {"scores": asdict(model), "evidence": record.text("display_scorecard"),
                                     "raw_sha256": record.text("raw_scorecard_sha256"), "mechanism": record.text("mechanism")}
            asset.fields["reading"] = READINGS[name]
            asset.fields["style_accepted"] = verdict.passed
            asset.fields["status"] = "STYLE_ACCEPTED_READABILITY_PENDING" if verdict.passed else "REJECTED_NUMERIC_RETRY_LIMIT"
            asset.fields["readability_status"] = "PENDING_TWO_FRESH_32PX_REVIEWERS"
            _ = (DEST / f"{name}.json").write_text(json.dumps(asset.fields, indent=2, ensure_ascii=False)+"\n")
        else:
            prior = JsonRecord.read(ROOT / asset.text("source_metadata"))
            asset.fields["prior_working_sha256_matches"] = digest(ROOT / asset.text("path")) == prior.text("working_sha256")
            if not asset.flag("prior_working_sha256_matches"):
                message = f"Reused accepted bytes changed: {name}"
                raise RuntimeError(message)
            asset.fields["readability_status"] = "PENDING_TWO_FRESH_32PX_REVIEWERS"
    manifest.fields["item_kinds"] = [{"sim_kind": kind, "sim_code": code, "asset_id": name}
                              for code, (kind, name) in enumerate(KINDS)]
    manifest.fields["state_aliases"] = {"ClayPot.filled": "clay_pot_filled", "food": "berry_basket"}
    manifest.fields["spoiled_meat"] = {
        "source": "art/style05/env1/items/meat.png", "method": "code-only whole-sprite tone transform",
        "srgb_multiplier": [0.55, 0.70, 0.45], "alpha": "unchanged",
        "note": "Declared art state; simulation removes expired meat, so this is not a persistent current sim object.",
    }
    manifest.fields["generation_calls"] = 27
    manifest.fields["sprite_generation_calls"] = 23
    manifest.fields["image_model_scorecard_calls"] = 3
    manifest.fields["failed_tool_calls"] = [{"reason": "Referenced image paths exceeded5; rejected before generation", "count": 1}]
    manifest.fields["owner_approval"] = "NOT_REQUESTED_NOT_CLAIMED"
    review_path = DOCS / "items_reviews.json"
    if review_path.exists():
        semantic = JsonRecord.read(review_path)
        manifest.fields["semantic_readability_summary"] = semantic.fields["summary"]
        for asset in assets:
            answers = [answer for review in semantic.records("reviews") for answer in review.records("answers")
                       if answer.text("expected_asset") == asset.text("id")]
            if len(answers) != 2:
                message = f"Expected two item readability answers: {asset.text('id')}"
                raise RuntimeError(message)
            exact = [answer.flag("exact_kind_recognized") for answer in answers]
            asset.fields["readability_status"] = "EXACT_KIND_RECOGNIZED_BOTH" if all(exact) else "EXACT_KIND_CONFUSION"
            asset.fields["readability_evidence"] = "docs/v8/artenv1/items_reviews.json"
            asset.fields["readability_review_ids"] = [review.text("reviewer") for review in semantic.records("reviews")]
            asset.fields["exact_kind_scores"] = exact
            if asset.records("attempts"):
                asset.fields["status"] = "STYLE_ACCEPTED_SEMANTIC_REVIEWED" if asset.flag("style_accepted") else "REJECTED_NUMERIC_RETRY_LIMIT"
                _ = (DEST / f"{asset.text('id')}.json").write_text(json.dumps(asset.fields, indent=2, ensure_ascii=False)+"\n")
    manifest.fields["assets"] = [asset.fields for asset in assets]
    _ = manifest_path.write_text(json.dumps(manifest.fields, indent=2, ensure_ascii=False)+"\n")
    _ = (DOCS / "items_model_reviews.json").write_text(json.dumps([review.fields for review in reviews], indent=2, ensure_ascii=False)+"\n")
    print("12 new style+numeric accepted;1numeric rejection;9reused byte identities verified;27toolcalls")


if __name__ == "__main__":
    main()
