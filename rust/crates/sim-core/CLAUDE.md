# sim-core components

## Revitalize Pass 3 child substrate

components::Child { born_tick: u64 } records a newborn's simulation birth tick. SocietyRules::life_stage controls attachment and maturation; absent rules preserve adult newborns. The marker itself performs no updates or RNG draws.

## Revitalize Pass 4 aging substrate

components::Life { born_tick: i64, lifespan_ticks: u64 } holds a resident's birth tick (negative for founders, who are generated already aged) and the lifespan drawn from its own salted splitmix64 stream. The zero-sized components::Elder marker is attached once age reaches AgingRule::elder_age_ticks. SocietyRules::aging controls both; absent rules (legacy, v8, v9) attach neither. Neither component updates itself or draws RNG; AgingSystem (priority 146) does the work.
