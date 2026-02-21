## [Schwartz (1992)] 33개 가치관 정의 — 상수 및 매핑 테이블
## 참조: const ValueDefs = preload("res://scripts/core/value_defs.gd")
extends RefCounted

## ── 가치관 키 목록 ─────────────────────────────────────
const KEYS: Array = [
	&"LAW", &"LOYALTY", &"FAMILY", &"FRIENDSHIP", &"POWER",
	&"TRUTH", &"CUNNING", &"ELOQUENCE", &"FAIRNESS", &"DECORUM",
	&"TRADITION", &"ARTWORK", &"COOPERATION", &"INDEPENDENCE",
	&"STOICISM", &"INTROSPECTION", &"SELF_CONTROL", &"TRANQUILITY",
	&"HARMONY", &"MERRIMENT", &"CRAFTSMANSHIP", &"MARTIAL_PROWESS",
	&"SKILL", &"HARD_WORK", &"SACRIFICE", &"COMPETITION",
	&"PERSEVERANCE", &"LEISURE", &"COMMERCE", &"ROMANCE",
	&"KNOWLEDGE", &"NATURE", &"PEACE",
]

## ── HEXACO facet → 가치관 초기값 공식 ──────────────────
## [GPT 수치 설계 2025] facet 0~1 → z = 2*(f-0.5) → -1~+1
## 음수 weight = "해당 facet이 높으면 이 가치관이 낮아짐"
const HEXACO_SEED_MAP: Dictionary = {
	&"LAW":            { "fairness": 0.35, "prudence": 0.25, "organization": 0.20, "sincerity": 0.20 },
	&"LOYALTY":        { "sentimentality": 0.30, "dependence": 0.25, "sincerity": 0.25, "forgivingness": 0.20 },
	&"FAMILY":         { "sentimentality": 0.40, "dependence": 0.25, "sincerity": 0.20, "patience": 0.15 },
	&"FRIENDSHIP":     { "sociability": 0.30, "sentimentality": 0.25, "sincerity": 0.25, "forgivingness": 0.20 },
	&"POWER":          { "social_boldness": 0.30, "modesty": -0.35, "greed_avoidance": -0.25, "social_self_esteem": 0.10 },
	&"TRUTH":          { "sincerity": 0.35, "fairness": 0.30, "greed_avoidance": 0.20, "prudence": 0.15 },
	&"CUNNING":        { "sincerity": -0.30, "unconventionality": 0.25, "creativity": 0.25, "social_boldness": 0.20 },
	&"ELOQUENCE":      { "social_boldness": 0.30, "social_self_esteem": 0.25, "sociability": 0.25, "creativity": 0.20 },
	&"FAIRNESS":       { "fairness": 0.40, "sincerity": 0.25, "greed_avoidance": 0.20, "patience": 0.15 },
	&"DECORUM":        { "modesty": 0.30, "patience": 0.25, "gentleness": 0.25, "prudence": 0.20 },
	&"TRADITION":      { "prudence": 0.30, "unconventionality": -0.35, "modesty": 0.20, "patience": 0.15 },
	&"ARTWORK":        { "aesthetic_appreciation": 0.50, "creativity": 0.30, "unconventionality": 0.20 },
	&"COOPERATION":    { "flexibility": 0.30, "gentleness": 0.25, "patience": 0.25, "forgivingness": 0.20 },
	&"INDEPENDENCE":   { "dependence": -0.35, "social_self_esteem": 0.25, "social_boldness": 0.20, "unconventionality": 0.20 },
	&"STOICISM":       { "anxiety": -0.25, "fearfulness": -0.25, "sentimentality": -0.25, "patience": 0.25 },
	&"INTROSPECTION":  { "inquisitiveness": 0.30, "aesthetic_appreciation": 0.25, "creativity": 0.25, "prudence": 0.20 },
	&"SELF_CONTROL":   { "prudence": 0.35, "perfectionism": 0.25, "organization": 0.20, "patience": 0.20 },
	&"TRANQUILITY":    { "anxiety": -0.30, "fearfulness": -0.20, "patience": 0.30, "gentleness": 0.20 },
	&"HARMONY":        { "forgivingness": 0.25, "flexibility": 0.25, "gentleness": 0.25, "patience": 0.25 },
	&"MERRIMENT":      { "liveliness": 0.40, "sociability": 0.25, "social_self_esteem": 0.20, "creativity": 0.15 },
	&"CRAFTSMANSHIP":  { "perfectionism": 0.35, "diligence": 0.30, "aesthetic_appreciation": 0.20, "organization": 0.15 },
	&"MARTIAL_PROWESS":{ "social_boldness": 0.30, "fearfulness": -0.30, "anxiety": -0.20, "diligence": 0.20 },
	&"SKILL":          { "diligence": 0.30, "perfectionism": 0.25, "inquisitiveness": 0.25, "creativity": 0.20 },
	&"HARD_WORK":      { "diligence": 0.45, "perfectionism": 0.25, "organization": 0.15, "prudence": 0.15 },
	&"SACRIFICE":      { "greed_avoidance": 0.30, "modesty": 0.25, "sentimentality": 0.25, "sincerity": 0.20 },
	&"COMPETITION":    { "social_boldness": 0.30, "social_self_esteem": 0.25, "modesty": -0.25, "liveliness": 0.20 },
	&"PERSEVERANCE":   { "diligence": 0.35, "patience": 0.30, "fearfulness": -0.20, "prudence": 0.15 },
	&"LEISURE":        { "liveliness": 0.30, "sociability": 0.25, "diligence": -0.25, "aesthetic_appreciation": 0.20 },
	&"COMMERCE":       { "social_boldness": 0.25, "greed_avoidance": -0.30, "sociability": 0.25, "organization": 0.20 },
	&"ROMANCE":        { "sentimentality": 0.35, "aesthetic_appreciation": 0.25, "sociability": 0.20, "creativity": 0.20 },
	&"KNOWLEDGE":      { "inquisitiveness": 0.45, "creativity": 0.25, "diligence": 0.15, "unconventionality": 0.15 },
	&"NATURE":         { "aesthetic_appreciation": 0.35, "sentimentality": 0.25, "inquisitiveness": 0.20, "unconventionality": 0.20 },
	&"PEACE":          { "gentleness": 0.30, "forgivingness": 0.25, "patience": 0.25, "fearfulness": 0.20 },
}

## ── 가치관 충돌 쌍 [Haidt (2012)] ──────────────────────
const CONFLICT_PAIRS: Array = [
	{ "a": &"LOYALTY",       "b": &"TRUTH",        "tension": 0.8 },
	{ "a": &"LOYALTY",       "b": &"FAIRNESS",      "tension": 0.7 },
	{ "a": &"INDEPENDENCE",  "b": &"LOYALTY",       "tension": 0.6 },
	{ "a": &"INDEPENDENCE",  "b": &"COOPERATION",   "tension": 0.5 },
	{ "a": &"POWER",         "b": &"HARMONY",       "tension": 0.7 },
	{ "a": &"POWER",         "b": &"FAIRNESS",      "tension": 0.6 },
	{ "a": &"CUNNING",       "b": &"TRUTH",         "tension": 0.9 },
	{ "a": &"CUNNING",       "b": &"FAIRNESS",      "tension": 0.8 },
	{ "a": &"TRADITION",     "b": &"INDEPENDENCE",  "tension": 0.6 },
	{ "a": &"COMPETITION",   "b": &"HARMONY",       "tension": 0.6 },
	{ "a": &"COMPETITION",   "b": &"COOPERATION",   "tension": 0.5 },
	{ "a": &"MARTIAL_PROWESS","b": &"PEACE",        "tension": 0.8 },
	{ "a": &"LEISURE",       "b": &"HARD_WORK",     "tension": 0.7 },
	{ "a": &"SELF_CONTROL",  "b": &"MERRIMENT",     "tension": 0.4 },
	{ "a": &"STOICISM",      "b": &"ROMANCE",       "tension": 0.5 },
	{ "a": &"SACRIFICE",     "b": &"INDEPENDENCE",  "tension": 0.5 },
	{ "a": &"COMMERCE",      "b": &"NATURE",        "tension": 0.4 },
]

## ── Kohlberg 도덕 발달 단계별 가중치 [Kohlberg (1969)] ─
const KOHLBERG_MODIFIERS: Dictionary = {
	1: { &"LAW": 0.3, &"POWER": 1.5, &"CUNNING": 1.3, &"FAIRNESS": 0.5, &"SACRIFICE": 0.3 },
	2: { &"COMMERCE": 1.4, &"CUNNING": 1.2, &"COMPETITION": 1.3, &"SACRIFICE": 0.4, &"FAIRNESS": 0.6 },
	3: { &"LOYALTY": 1.4, &"FAMILY": 1.3, &"FRIENDSHIP": 1.3, &"HARMONY": 1.4, &"DECORUM": 1.3, &"INDEPENDENCE": 0.7 },
	4: { &"LAW": 1.5, &"TRADITION": 1.3, &"HARD_WORK": 1.2, &"FAIRNESS": 1.2, &"CUNNING": 0.5 },
	5: { &"FAIRNESS": 1.5, &"TRUTH": 1.3, &"COOPERATION": 1.3, &"LAW": 0.9, &"INDEPENDENCE": 1.2 },
	6: { &"TRUTH": 1.5, &"FAIRNESS": 1.5, &"SACRIFICE": 1.4, &"PEACE": 1.3, &"LOYALTY": 0.8, &"LAW": 0.7 },
}

## Kohlberg 단계별 최소 나이 요건 (index = 단계)
const KOHLBERG_AGE_REQ: Array = [0, 0, 5, 10, 16, 25, 35]

## Kohlberg 단계 진급 조건
const KOHLBERG_THRESHOLDS: Dictionary = {
	2: { "min_values": { &"CUNNING": -0.5 },                                           "min_openness": 0.30 },
	3: { "min_values": { &"LOYALTY": 0.1,  &"HARMONY": 0.1 },                         "min_openness": 0.35 },
	4: { "min_values": { &"LAW": 0.2,      &"FAIRNESS": 0.1 },                        "min_openness": 0.40 },
	5: { "min_values": { &"FAIRNESS": 0.3, &"TRUTH": 0.2 },                           "min_openness": 0.50 },
	6: { "min_values": { &"FAIRNESS": 0.5, &"TRUTH": 0.4, &"SACRIFICE": 0.3 },        "min_openness": 0.60 },
}

## ── 행동 → 가치관 alignment 테이블 ──────────────────────
## 양수 = 이 행동이 해당 가치관에 부합, 음수 = 반함
const ACTION_VALUE_ALIGNMENTS: Dictionary = {
	&"gather_food":    { &"HARD_WORK": 0.20, &"PERSEVERANCE": 0.15, &"FAMILY": 0.10, &"LOYALTY": 0.10, &"LEISURE": -0.15 },
	&"rest":           { &"TRANQUILITY": 0.25, &"LEISURE": 0.20, &"PEACE": 0.10, &"HARD_WORK": -0.15, &"STOICISM": -0.10 },
	&"socialize":      { &"FRIENDSHIP": 0.25, &"MERRIMENT": 0.20, &"COOPERATION": 0.15, &"ROMANCE": 0.10, &"INTROSPECTION": -0.15, &"STOICISM": -0.10 },
	&"build":          { &"CRAFTSMANSHIP": 0.25, &"HARD_WORK": 0.20, &"SKILL": 0.15, &"PERSEVERANCE": 0.15, &"TRADITION": 0.10, &"LEISURE": -0.15 },
	&"gather_wood":    { &"HARD_WORK": 0.15, &"PERSEVERANCE": 0.10, &"NATURE": -0.10 },
	&"gather_stone":   { &"HARD_WORK": 0.15, &"PERSEVERANCE": 0.10, &"CRAFTSMANSHIP": 0.10 },
	&"wander":         { &"INDEPENDENCE": 0.25, &"KNOWLEDGE": 0.15, &"NATURE": 0.15, &"TRADITION": -0.15, &"LOYALTY": -0.10 },
	&"hide":           { &"PEACE": 0.15, &"TRANQUILITY": 0.15, &"POWER": -0.10, &"MARTIAL_PROWESS": -0.10, &"LAW": 0.05 },
	&"grieve":         { &"FAMILY": 0.20, &"LOYALTY": 0.10, &"ROMANCE": 0.15, &"INTROSPECTION": 0.15, &"STOICISM": -0.20 },
	&"confront":       { &"POWER": 0.25, &"COMPETITION": 0.20, &"MARTIAL_PROWESS": 0.20, &"FAIRNESS": 0.10, &"PEACE": -0.20, &"HARMONY": -0.15 },
	&"drink_water":    { &"SELF_CONTROL": 0.05, &"TRANQUILITY": 0.05 },
	&"sit_by_fire":    { &"TRANQUILITY": 0.20, &"FAMILY": 0.10, &"FRIENDSHIP": 0.10, &"INTROSPECTION": 0.10 },
	&"seek_shelter":   { &"LAW": 0.10, &"SELF_CONTROL": 0.10, &"TRANQUILITY": 0.10, &"PEACE": 0.05, &"INDEPENDENCE": -0.05 },
}
