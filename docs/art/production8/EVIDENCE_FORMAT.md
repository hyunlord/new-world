# 압축된 QA 증거 읽기

[measurements.md](measurements.md)는 판정과 수치 요약입니다. [measurements.json.gz](measurements.json.gz)는 모든 상세 판정, [qa_evidence.json.gz](qa_evidence.json.gz)는 입력 증거입니다. 기존 검사기가 작성한 [ingame_qa.json.gz](ingame_qa.json.gz)와 [ingame_qa.md.gz](ingame_qa.md.gz)도 생략 없이 보존했습니다.

모두 gzip이며 압축 해제하면 원본 바이트와 같습니다. 판정·임계값·상세 기록을 삭제하거나 변경하지 않았습니다. 최초 원본과 SHA 기록은 `/Users/rexxa/work/art-production8-evidence/first-formal-verdict/`에 보존했습니다. 재생성 도구도 같은 압축 형식을 사용합니다.

```python
import gzip
import json
from pathlib import Path
checks = json.loads(gzip.decompress(Path("docs/art/production8/measurements.json.gz").read_bytes()))
```

기존 Markdown은 `gzip -dc docs/art/production8/ingame_qa.md.gz`로 읽을 수 있습니다. `ingame_qa`는 기존 검사기의 중간 결과이고 최종 통합 판정은 `measurements`입니다.
