extends RefCounted

## StatModifier: 외부 요인이 스탯에 가하는 일시적/영구적 수정.
## 불변 데이터 클래스. 생성 후 값 변경 없음.

enum ModType {
	ADD,      ## base += value
	MULTIPLY, ## base *= value (독립 적용: 0.7 × 0.8 = 0.56)
	CLAMP,    ## clamp(base, clamp_min, clamp_max)
	OVERRIDE  ## = value (최고 priority가 승리)
}

var id: StringName = &""
var target: StringName = &""
var mod_type: ModType = ModType.ADD
var value: float = 0.0
var clamp_min: float = 0.0
var clamp_max: float = 99999.0
var source: String = ""
var duration: int = -1           ## 남은 틱 수. -1 = 영구.
var decay_rate: float = 0.0      ## 틱당 value 감쇠율 (0 = 감쇠 없음)
var stack_group: String = ""     ## 같은 그룹 → max(|value|) 하나만 적용
var priority: int = 0            ## OVERRIDE 충돌 해소

## 편의 생성자
static func make_add(id_: StringName, target_: StringName,
		val: float, src: String, duration_: int = -1) -> RefCounted:
	var m = load("res://scripts/core/stat_modifier.gd").new()
	m.id = id_; m.target = target_
	m.mod_type = ModType.ADD; m.value = val
	m.source = src; m.duration = duration_
	return m

static func make_multiply(id_: StringName, target_: StringName,
		val: float, src: String, duration_: int = -1) -> RefCounted:
	var m = load("res://scripts/core/stat_modifier.gd").new()
	m.id = id_; m.target = target_
	m.mod_type = ModType.MULTIPLY; m.value = val
	m.source = src; m.duration = duration_
	return m

## 직렬화
func to_dict() -> Dictionary:
	return {
		"id": str(id), "target": str(target),
		"mod_type": mod_type, "value": value,
		"clamp_min": clamp_min, "clamp_max": clamp_max,
		"source": source, "duration": duration,
		"decay_rate": decay_rate, "stack_group": stack_group,
		"priority": priority
	}

static func from_dict(d: Dictionary) -> RefCounted:
	var m = load("res://scripts/core/stat_modifier.gd").new()
	m.id = StringName(d.get("id", ""))
	m.target = StringName(d.get("target", ""))
	m.mod_type = int(d.get("mod_type", 0)) as ModType
	m.value = float(d.get("value", 0.0))
	m.clamp_min = float(d.get("clamp_min", 0.0))
	m.clamp_max = float(d.get("clamp_max", 99999.0))
	m.source = d.get("source", "")
	m.duration = int(d.get("duration", -1))
	m.decay_rate = float(d.get("decay_rate", 0.0))
	m.stack_group = d.get("stack_group", "")
	m.priority = int(d.get("priority", 0))
	return m
