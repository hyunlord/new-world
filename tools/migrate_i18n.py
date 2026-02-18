#!/usr/bin/env python3
"""
migrate_i18n.py — i18n 구조 전면 정비
TICKET-A: data/locales/ traits_events 6키 → localization/*/ui.json 병합
TICKET-B: mental_breaks, trauma_scars, trait_definitions_fixed 텍스트 필드 제거
"""
import json
import shutil
from pathlib import Path

ROOT = Path(__file__).parent.parent
DATA = ROOT / "data"
LOC = ROOT / "localization"
LOCALES = ["ko", "en"]


def load_json(path: Path) -> dict | list:
    return json.loads(path.read_text(encoding="utf-8"))


def save_json(path: Path, data) -> None:
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"  ✅ saved {path.relative_to(ROOT)}")


# ─── TICKET-A ──────────────────────────────────────────────────────────────────
def ticket_a():
    print("\n=== TICKET-A: traits_events 병합 + data/locales/ 삭제 ===")

    for locale in LOCALES:
        events_path = DATA / "locales" / locale / "traits_events.json"
        ui_path = LOC / locale / "ui.json"
        if not events_path.exists():
            print(f"  ⚠️  {events_path} 없음, 스킵")
            continue
        events = load_json(events_path)
        ui = load_json(ui_path)
        added = 0
        for k, v in events.items():
            if k not in ui:
                ui[k] = v
                added += 1
                print(f"  [{locale}] 추가: {k}")
        save_json(ui_path, ui)
        print(f"  [{locale}] {added}개 키 병합 완료")

    # data/locales/ 삭제
    locales_dir = DATA / "locales"
    if locales_dir.exists():
        shutil.rmtree(locales_dir)
        print(f"  🗑️  삭제: {locales_dir.relative_to(ROOT)}")
    else:
        print(f"  ⚠️  data/locales/ 이미 없음")


# ─── TICKET-B: trauma_scars ────────────────────────────────────────────────────
def ticket_b_trauma_scars():
    print("\n=== TICKET-B1: trauma_scars.json 텍스트 필드 제거 ===")
    path = DATA / "trauma_scars.json"
    data = load_json(path)  # dict keyed by scar_id
    text_fields = ["name_kr", "name_en", "description_kr", "description_en"]

    # SCAR_{id} 키 이미 ui.json에 존재 확인
    ui_ko = load_json(LOC / "ko" / "ui.json")
    for sid, item in data.items():
        nk = f"SCAR_{sid}"
        if nk not in ui_ko:
            print(f"  ⚠️  {nk} not in ko/ui.json — 수동 확인 필요")
        item["name_key"] = nk
        for f in text_fields:
            item.pop(f, None)

    save_json(path, data)
    print(f"  {len(data)}개 항목 처리 완료")


# ─── TICKET-B: mental_breaks ───────────────────────────────────────────────────
def ticket_b_mental_breaks():
    print("\n=== TICKET-B2: mental_breaks.json 텍스트 필드 제거 ===")
    path = DATA / "mental_breaks.json"
    data = load_json(path)  # dict keyed by break_id

    # 기존 MENTAL_BREAK_TYPE_{ID} 키 매핑 + 없는 DESC 키를 localization에 추가
    for locale in LOCALES:
        ui_path = LOC / locale / "ui.json"
        ui = load_json(ui_path)
        added = 0
        for bid, item in data.items():
            desc_key = f"MENTAL_BREAK_TYPE_{bid.upper()}_DESC"
            if desc_key not in ui:
                field = "description_kr" if locale == "ko" else "description_en"
                ui[desc_key] = item.get(field, "")
                added += 1
                print(f"  [{locale}] 추가: {desc_key}")
        if added:
            save_json(ui_path, ui)
        print(f"  [{locale}] {added}개 DESC 키 추가")

    # data JSON 정리
    text_fields = ["name_kr", "name_en", "description_kr", "description_en"]
    for bid, item in data.items():
        item["name_key"] = f"MENTAL_BREAK_TYPE_{bid.upper()}"
        item["desc_key"] = f"MENTAL_BREAK_TYPE_{bid.upper()}_DESC"
        for f in text_fields:
            item.pop(f, None)

    save_json(path, data)
    print(f"  {len(data)}개 항목 처리 완료")


# ─── TICKET-B: trait_definitions_fixed ────────────────────────────────────────
def ticket_b_trait_definitions_fixed():
    print("\n=== TICKET-B3: trait_definitions_fixed.json 텍스트 필드 제거 ===")
    path = DATA / "personality" / "trait_definitions_fixed.json"
    data = load_json(path)  # list of dicts
    items = data if isinstance(data, list) else list(data.values())

    # localization/ko/traits.json에서 key 형식 확인 (소문자: TRAIT_{id}_NAME)
    traits_ko = load_json(LOC / "ko" / "traits.json")
    text_fields = ["name_kr", "name_en", "description_kr", "description_en"]
    missing_keys = []

    for item in items:
        tid = item.get("id", "")
        nk = f"TRAIT_{tid}_NAME"
        dk = f"TRAIT_{tid}_DESC"
        if nk not in traits_ko:
            missing_keys.append(nk)
        item["name_key"] = nk
        item["desc_key"] = dk
        for f in text_fields:
            item.pop(f, None)

    if missing_keys:
        print(f"  ⚠️  traits.json에 없는 name_key {len(missing_keys)}개: {missing_keys[:5]}")
    else:
        print(f"  ✅ 모든 name_key가 localization/ko/traits.json에 존재")

    save_json(path, data)
    print(f"  {len(items)}개 항목 처리 완료")


# ─── TICKET-B: 오래된 personality 파일 텍스트 필드 제거 (inactive) ───────────
def ticket_b_inactive_personality():
    """스크립트에서 로드하지 않는 구파일들 정리 (안전하게 텍스트만 제거)"""
    print("\n=== TICKET-B4: inactive personality 파일 텍스트 필드 정리 ===")
    inactive_files = [
        DATA / "personality" / "trait_definitions.json",
        DATA / "personality" / "trait_definitions_derived.json",
        DATA / "personality" / "hexaco_definition.json",
        DATA / "species" / "human" / "emotions" / "dyad_definition.json",
    ]
    text_fields = ["name_kr", "name_en", "description_kr", "description_en",
                   "label_kr", "label_en", "title_kr", "title_en"]

    for fpath in inactive_files:
        if not fpath.exists():
            print(f"  ⚠️  없음: {fpath.relative_to(ROOT)}")
            continue
        data = load_json(fpath)
        count = _remove_text_fields_recursive(data, text_fields)
        if count > 0:
            save_json(fpath, data)
            print(f"  {fpath.name}: {count}개 텍스트 필드 제거")
        else:
            print(f"  {fpath.name}: 텍스트 필드 없음 (이미 클린)")


def _remove_text_fields_recursive(obj, fields: list) -> int:
    count = 0
    if isinstance(obj, dict):
        for f in fields:
            if f in obj:
                del obj[f]
                count += 1
        for v in obj.values():
            count += _remove_text_fields_recursive(v, fields)
    elif isinstance(obj, list):
        for item in obj:
            count += _remove_text_fields_recursive(item, fields)
    return count


# ─── 검증 ──────────────────────────────────────────────────────────────────────
def validate():
    print("\n=== 검증 ===")
    errors = []

    # 1. data/locales 삭제 확인
    if (DATA / "locales").exists():
        errors.append("data/locales/ 아직 존재")
    else:
        print("  ✅ data/locales/ 삭제됨")

    # 2. localization/ko/traits.json 존재 + 키 수
    traits_path = LOC / "ko" / "traits.json"
    if traits_path.exists():
        t = load_json(traits_path)
        print(f"  ✅ localization/ko/traits.json: {len(t)}개 키")
    else:
        errors.append("localization/ko/traits.json 없음")

    # 3. trauma_scars — name_key 존재, 텍스트 필드 없음
    ts = load_json(DATA / "trauma_scars.json")
    for sid, item in ts.items():
        if "name_kr" in item or "name_en" in item:
            errors.append(f"trauma_scars/{sid}: 텍스트 필드 잔존")
        if "name_key" not in item:
            errors.append(f"trauma_scars/{sid}: name_key 없음")
    print(f"  ✅ trauma_scars: {len(ts)}개 항목 클린" if not [e for e in errors if "trauma_scars" in e] else "")

    # 4. mental_breaks — name_key + desc_key 존재
    mb = load_json(DATA / "mental_breaks.json")
    for bid, item in mb.items():
        if "name_kr" in item or "description_kr" in item:
            errors.append(f"mental_breaks/{bid}: 텍스트 필드 잔존")
        if "name_key" not in item or "desc_key" not in item:
            errors.append(f"mental_breaks/{bid}: key 필드 없음")
    print(f"  ✅ mental_breaks: {len(mb)}개 항목 클린" if not [e for e in errors if "mental_breaks" in e] else "")

    # 5. trait_definitions_fixed — name_key 존재
    tf = load_json(DATA / "personality" / "trait_definitions_fixed.json")
    items = tf if isinstance(tf, list) else list(tf.values())
    bad = [i for i in items if "name_kr" in i or "name_key" not in i]
    if bad:
        errors.append(f"trait_definitions_fixed: {len(bad)}개 항목 문제")
    else:
        print(f"  ✅ trait_definitions_fixed: {len(items)}개 항목 클린")

    # 6. traits_events 키가 ui.json에 존재
    ui_ko = load_json(LOC / "ko" / "ui.json")
    required = ["CHRONICLE_TRAIT_DISPLAYED", "CHRONICLE_TRAIT_STRENGTHENED",
                "CHRONICLE_TRAIT_WEAKENED", "CHRONICLE_TRAIT_ARCHETYPE",
                "UI_TRAIT_SALIENCE_BAR", "UI_TRAIT_NO_DOMINANT"]
    for k in required:
        if k not in ui_ko:
            errors.append(f"ko/ui.json에 {k} 없음")
    if not any(k not in ui_ko for k in required):
        print(f"  ✅ traits_events 6개 키 ko/ui.json에 존재")

    if errors:
        print(f"\n  ❌ 오류 {len(errors)}개:")
        for e in errors:
            print(f"    - {e}")
    else:
        print("\n  ✅ 모든 검증 통과")
    return len(errors) == 0


if __name__ == "__main__":
    ticket_a()
    ticket_b_trauma_scars()
    ticket_b_mental_breaks()
    ticket_b_trait_definitions_fixed()
    ticket_b_inactive_personality()
    ok = validate()
    import sys
    sys.exit(0 if ok else 1)
