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
	&"LAW":            { "H_fairness": 0.35, "C_prudence": 0.25, "C_organization": 0.20, "H_sincerity": 0.20 },
	&"LOYALTY":        { "E_sentimentality": 0.30, "E_dependence": 0.25, "H_sincerity": 0.25, "A_forgivingness": 0.20 },
	&"FAMILY":         { "E_sentimentality": 0.40, "E_dependence": 0.25, "H_sincerity": 0.20, "A_patience": 0.15 },
	&"FRIENDSHIP":     { "X_sociability": 0.30, "E_sentimentality": 0.25, "H_sincerity": 0.25, "A_forgivingness": 0.20 },
	&"POWER":          { "X_social_boldness": 0.30, "H_modesty": -0.35, "H_greed_avoidance": -0.25, "X_social_self_esteem": 0.10 },
	&"TRUTH":          { "H_sincerity": 0.35, "H_fairness": 0.30, "H_greed_avoidance": 0.20, "C_prudence": 0.15 },
	&"CUNNING":        { "H_sincerity": -0.30, "O_unconventionality": 0.25, "O_creativity": 0.25, "X_social_boldness": 0.20 },
	&"ELOQUENCE":      { "X_social_boldness": 0.30, "X_social_self_esteem": 0.25, "X_sociability": 0.25, "O_creativity": 0.20 },
	&"FAIRNESS":       { "H_fairness": 0.40, "H_sincerity": 0.25, "H_greed_avoidance": 0.20, "A_patience": 0.15 },
	&"DECORUM":        { "H_modesty": 0.30, "A_patience": 0.25, "A_gentleness": 0.25, "C_prudence": 0.20 },
	&"TRADITION":      { "C_prudence": 0.30, "O_unconventionality": -0.35, "H_modesty": 0.20, "A_patience": 0.15 },
	&"ARTWORK":        { "O_aesthetic": 0.50, "O_creativity": 0.30, "O_unconventionality": 0.20 },
	&"COOPERATION":    { "A_flexibility": 0.30, "A_gentleness": 0.25, "A_patience": 0.25, "A_forgivingness": 0.20 },
	&"INDEPENDENCE":   { "E_dependence": -0.35, "X_social_self_esteem": 0.25, "X_social_boldness": 0.20, "O_unconventionality": 0.20 },
	&"STOICISM":       { "E_anxiety": -0.25, "E_fearfulness": -0.25, "E_sentimentality": -0.25, "A_patience": 0.25 },
	&"INTROSPECTION":  { "O_inquisitiveness": 0.30, "O_aesthetic": 0.25, "O_creativity": 0.25, "C_prudence": 0.20 },
	&"SELF_CONTROL":   { "C_prudence": 0.35, "C_perfectionism": 0.25, "C_organization": 0.20, "A_patience": 0.20 },
	&"TRANQUILITY":    { "E_anxiety": -0.30, "E_fearfulness": -0.20, "A_patience": 0.30, "A_gentleness": 0.20 },
	&"HARMONY":        { "A_forgivingness": 0.25, "A_flexibility": 0.25, "A_gentleness": 0.25, "A_patience": 0.25 },
	&"MERRIMENT":      { "X_liveliness": 0.40, "X_sociability": 0.25, "X_social_self_esteem": 0.20, "O_creativity": 0.15 },
	&"CRAFTSMANSHIP":  { "C_perfectionism": 0.35, "C_diligence": 0.30, "O_aesthetic": 0.20, "C_organization": 0.15 },
	&"MARTIAL_PROWESS":{ "X_social_boldness": 0.30, "E_fearfulness": -0.30, "E_anxiety": -0.20, "C_diligence": 0.20 },
	&"SKILL":          { "C_diligence": 0.30, "C_perfectionism": 0.25, "O_inquisitiveness": 0.25, "O_creativity": 0.20 },
	&"HARD_WORK":      { "C_diligence": 0.45, "C_perfectionism": 0.25, "C_organization": 0.15, "C_prudence": 0.15 },
	&"SACRIFICE":      { "H_greed_avoidance": 0.30, "H_modesty": 0.25, "E_sentimentality": 0.25, "H_sincerity": 0.20 },
	&"COMPETITION":    { "X_social_boldness": 0.30, "X_social_self_esteem": 0.25, "H_modesty": -0.25, "X_liveliness": 0.20 },
	&"PERSEVERANCE":   { "C_diligence": 0.35, "A_patience": 0.30, "E_fearfulness": -0.20, "C_prudence": 0.15 },
	&"LEISURE":        { "X_liveliness": 0.30, "X_sociability": 0.25, "C_diligence": -0.25, "O_aesthetic": 0.20 },
	&"COMMERCE":       { "X_social_boldness": 0.25, "H_greed_avoidance": -0.30, "X_sociability": 0.25, "C_organization": 0.20 },
	&"ROMANCE":        { "E_sentimentality": 0.35, "O_aesthetic": 0.25, "X_sociability": 0.20, "O_creativity": 0.20 },
	&"KNOWLEDGE":      { "O_inquisitiveness": 0.45, "O_creativity": 0.25, "C_diligence": 0.15, "O_unconventionality": 0.15 },
	&"NATURE":         { "O_aesthetic": 0.35, "E_sentimentality": 0.25, "O_inquisitiveness": 0.20, "O_unconventionality": 0.20 },
	&"PEACE":          { "A_gentleness": 0.30, "A_forgivingness": 0.25, "A_patience": 0.25, "E_fearfulness": 0.20 },
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
	2: { "min_values": { &"CUNNING": -0.15 },                                          "min_openness": 0.30 },
	3: { "min_values": { &"LOYALTY": 0.05,  &"HARMONY": 0.05 },                       "min_openness": 0.35 },
	4: { "min_values": { &"LAW": 0.08,      &"FAIRNESS": 0.05 },                      "min_openness": 0.40 },
	5: { "min_values": { &"FAIRNESS": 0.12, &"TRUTH": 0.08 },                         "min_openness": 0.50 },
	6: { "min_values": { &"FAIRNESS": 0.20, &"TRUTH": 0.15, &"SACRIFICE": 0.12 },     "min_openness": 0.60 },
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
