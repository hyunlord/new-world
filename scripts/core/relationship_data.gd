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
