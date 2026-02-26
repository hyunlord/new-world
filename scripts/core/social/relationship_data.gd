extends RefCounted

## Relationship between two entities.
## Stored sparsely — only created on first interaction.

var affinity: float = 0.0            # 0~100, closeness
var trust: float = 50.0              # 0~100, reliability
var romantic_interest: float = 0.0   # 0~100, romantic attraction
var interaction_count: int = 0
var last_interaction_tick: int = 0
var type: String = "stranger"
# Types: stranger, acquaintance, friend, close_friend, romantic, partner, rival

## [Granovetter 1973] Tie strength category derived from affinity.
## absent (< 5), weak (5~30), moderate (30~60), strong (60~85), intimate (85+)
## Updated by RelationshipManager._update_tie_type() whenever affinity changes.
var tie_type: String = "absent"
