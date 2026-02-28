extends RefCounted

## [Henrich 2004] Five-state machine for per-settlement tech knowledge.
##
## State transitions:
##   unknown --discover--> known_low --stabilize--> known_stable
##                                                       |
##   forgotten_long <--decay-- forgotten_recent <--lose--+
##                                  |
##                          --rediscover--> known_low (with kintsugi flag)

enum State {
	UNKNOWN,            ## 0: Never discovered / never reached this settlement
	KNOWN_LOW,          ## 1: Discovered but fragile — few practitioners, at risk
	KNOWN_STABLE,       ## 2: Established — enough practitioners to sustain
	FORGOTTEN_RECENT,   ## 3: Lost — cultural memory remains, rediscovery easier
	FORGOTTEN_LONG,     ## 4: Lost long ago — memory nearly gone
}

const STATE_FROM_STRING: Dictionary = {
	"unknown": State.UNKNOWN,
	"known_low": State.KNOWN_LOW,
	"known_stable": State.KNOWN_STABLE,
	"forgotten_recent": State.FORGOTTEN_RECENT,
	"forgotten_long": State.FORGOTTEN_LONG,
}

const STATE_TO_STRING: Dictionary = {
	State.UNKNOWN: "unknown",
	State.KNOWN_LOW: "known_low",
	State.KNOWN_STABLE: "known_stable",
	State.FORGOTTEN_RECENT: "forgotten_recent",
	State.FORGOTTEN_LONG: "forgotten_long",
}

## Locale keys per state (for UI display)
const STATE_LOCALE_KEYS: Dictionary = {
	State.UNKNOWN: "TECH_STATE_UNKNOWN",
	State.KNOWN_LOW: "TECH_STATE_KNOWN_LOW",
	State.KNOWN_STABLE: "TECH_STATE_KNOWN_STABLE",
	State.FORGOTTEN_RECENT: "TECH_STATE_FORGOTTEN_RECENT",
	State.FORGOTTEN_LONG: "TECH_STATE_FORGOTTEN_LONG",
}


## Is this state considered "known" (tech is active/usable)?
static func is_known(state: int) -> bool:
	return state == State.KNOWN_LOW or state == State.KNOWN_STABLE


## Is this state considered "forgotten" (tech was once known)?
static func is_forgotten(state: int) -> bool:
	return state == State.FORGOTTEN_RECENT or state == State.FORGOTTEN_LONG
