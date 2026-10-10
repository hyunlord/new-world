# sim-core components

## Revitalize Pass 3 child substrate

components::Child { born_tick: u64 } records a newborn's simulation birth tick. SocietyRules::life_stage controls attachment and maturation; absent rules preserve adult newborns. The marker itself performs no updates or RNG draws.
