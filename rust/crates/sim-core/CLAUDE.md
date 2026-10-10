# sim-core components

## Revitalize Pass 3 child substrate

components::Child { born_tick: u64 } records a newborn's simulation birth tick. SocietyRules::life_stage controls attachment and maturation; absent rules preserve adult newborns. The marker itself performs no updates or RNG draws.

## Revitalize Pass 4 aging substrate

components::Life { born_tick: i64, lifespan_ticks: u64 } holds a resident's birth tick (negative for founders, who are generated already aged) and the lifespan drawn from its own salted splitmix64 stream. The zero-sized components::Elder marker is attached once age reaches AgingRule::elder_age_ticks. SocietyRules::aging controls both; absent rules (legacy, v8, v9) attach neither. Neither component updates itself or draws RNG; AgingSystem (priority 146) does the work.

## Revitalize Pass 5 cold-night substrate

components::Cold is a zero-sized marker for a settlement member spending a winter night inside FirewoodRule's cold window while its settlement's fire is out. FireSystem (priority 147, interval 60) attaches and removes it in AgentId order; HungerDecaySystem grows a marked agent's hunger by growth_rate × cold_hunger_permille / 1000 instead of growth_rate. SocietyRules::firewood controls it; absent rules (legacy, v8, v9, v10) never attach it. The marker does not update itself or draw RNG.
