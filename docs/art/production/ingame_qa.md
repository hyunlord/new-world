# 실제 게임 캡처 QA

원본 windowed framebuffer와 실제 렌더 노드 증거. N/A는 통과가 아니다.

## CAPTURE_SET: PASS

92/92 entries; missing=[]; extra=[]

## A1: PASS

92 frames tested; failing (std,colours)=[]

## A2: PASS

raw checker zero mixed pixels: [(4.0, True), (6.0, True)]

## A3: PASS

1164 drawn rects; wrong=[]; adult S-idle frames=4, aspect failures=[]; absent S-idle cannot pass

## A4: PASS

[(4.0, 'north', 4976, 1.0), (4.0, 'south', 5216, 1.0), (6.0, 'north', 11196, 1.0), (6.0, 'south', 11736, 1.0)]

## RAMP: PASS

actual framebuffer agrees with region-indexed ramp texels: [(4.0, 10, 1.0), (4.0, 11, 1.0), (4.0, 12, 1.0), (4.0, 13, 1.0), (4.0, 14, 1.0), (4.0, 15, 1.0), (4.0, 16, 1.0), (4.0, 17, 1.0), (4.0, 18, 1.0), (4.0, 19, 1.0), (4.0, 20, 1.0), (4.0, 21, 1.0), (6.0, 10, 1.0), (6.0, 11, 1.0), (6.0, 12, 1.0), (6.0, 13, 1.0), (6.0, 14, 1.0), (6.0, 15, 1.0), (6.0, 16, 1.0), (6.0, 17, 1.0), (6.0, 18, 1.0), (6.0, 19, 1.0), (6.0, 20, 1.0), (6.0, 21, 1.0)]

## A5: FAIL

median=0.2378; min=0.0548; per actor=[(0, 0.08840042781367397), (23, 0.18964160370600955), (22, 0.054779287370862406), (3, 0.23353733500208584), (18, 0.241989772359648), (13, 0.26610269057736685), (10, 0.24251788431509355), (21, 0.15038351804573677), (20, 0.3271184313725488), (1, 0.14114410902793345), (14, 0.21150444444444444), (19, 0.25853098039215683), (11, 0.35166772162386084), (24, 0.27449568627450976)]; unmeasurable=[15, 5, 9, 2, 4, 6, 16, 12]; opaque overlaps conservatively excluded

## A6: PASS

noon/night world=[0.2951148627009281, 0.15229333911914344]; 48px hearth means=[0.2523826126574295]; each hearth tested

## A7: PASS

moving samples=40; distinct frames/full moving second={0: 5, 1: 4, 2: 4}; changing fraction=1.000; duration=3.956s; interval range=0.054..0.117s

## A8: PASS

89 scenes checked; failures=[]

## A9: PASS

90 scenes checked; failures=[]

## A10: PASS

n=300, median=5.929ms, p95=15.266ms; live simulation=True

## A11: FAIL

{"count": 24, "absent_activities": ["build", "drink", "sleep", "child"], "observer_labels_sha256": "b32d76447e14b10675e1f0262d4d9f94161ef55e192112b93a00403b131cafd4", "confusion": {"4x": {"talk": {"gather": 2}, "sit_or_idle": {"carry": 1, "unidentifiable": 5}, "gather": {"gather": 3}, "carry": {"unidentifiable": 5, "gather": 1}, "walk": {"carry": 4, "walk": 1, "unidentifiable": 2}}, "2x": {"talk": {"gather": 2}, "sit_or_idle": {"carry": 1, "unidentifiable": 5}, "gather": {"gather": 3}, "carry": {"unidentifiable": 5, "gather": 1}, "walk": {"carry": 4, "walk": 1, "unidentifiable": 2}}}, "accuracy_4x": 0.16666666666666666, "accuracy_2x": 0.16666666666666666}

## A12: PASS

22 tuples/22 villagers; identical neighbors=0/13 (0.000)
