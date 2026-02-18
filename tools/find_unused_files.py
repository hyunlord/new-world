#!/usr/bin/env python3
"""
find_unused_files.py — 미참조 파일 탐지 (삭제하지 않음, 후보 목록만 출력)
"""
import json
from pathlib import Path

ROOT = Path(__file__).parent.parent


def get_all_source_text() -> str:
    exts = [".gd", ".tscn", ".json", ".md", ".cfg", ".godot"]
    texts = []
    for ext in exts:
        for f in ROOT.rglob(f"*{ext}"):
            # tools/ 스크립트 자신은 제외
            if "tools/" in str(f) and ext == ".py":
                continue
            try:
                texts.append(f.read_text(errors="ignore"))
            except Exception:
                pass
    return "\n".join(texts)


def find_unused_data_jsons(src: str) -> list[Path]:
    unused = []
    for f in (ROOT / "data").rglob("*.json"):
        stem = f.stem
        rel = str(f.relative_to(ROOT))
        if stem not in src and rel not in src:
            unused.append(f)
    return unused


def find_unused_locale_keys(src: str) -> list[tuple[str, str]]:
    orphans = []
    for f in (ROOT / "localization").rglob("*.json"):
        try:
            data = json.loads(f.read_text())
        except Exception:
            continue
        for key in data:
            # 동적으로 조합되는 key 패턴 (정적 탐지 불가 → false positive 위험)
            if key not in src:
                orphans.append((str(f.relative_to(ROOT)), key))
    return orphans


def find_unused_gd_scripts(src: str) -> list[Path]:
    unused = []
    for f in (ROOT / "scripts").rglob("*.gd"):
        stem = f.stem
        rel = str(f.relative_to(ROOT))
        if stem not in src and rel not in src:
            unused.append(f)
    return unused


def main():
    print("[find_unused_files] 소스 텍스트 로딩 중...")
    src = get_all_source_text()
    print(f"총 소스 크기: {len(src):,} 문자\n")

    print("=== 미참조 data JSON 파일 ===")
    unused_data = find_unused_data_jsons(src)
    if unused_data:
        for f in unused_data:
            print(f"  UNUSED: {f.relative_to(ROOT)}")
    else:
        print("  없음 ✓")

    print("\n=== 미참조 GDScript 파일 ===")
    unused_gd = find_unused_gd_scripts(src)
    if unused_gd:
        for f in unused_gd:
            print(f"  UNUSED: {f.relative_to(ROOT)}")
    else:
        print("  없음 ✓")

    print("\n=== 미참조 localization key (orphan) — 주의: 동적 키 false positive 위험 ===")
    orphans = find_unused_locale_keys(src)
    if orphans:
        for path, key in orphans[:30]:
            print(f"  ORPHAN: [{path}] {key}")
        if len(orphans) > 30:
            print(f"  ... 및 {len(orphans) - 30}개 더")
    else:
        print("  없음 ✓")

    print("\n[완료] 위 목록은 후보일 뿐입니다. 실제 삭제 전 수동 확인 필수.")


if __name__ == "__main__":
    main()
