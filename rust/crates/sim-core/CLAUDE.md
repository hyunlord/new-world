# sim-core components

## Revitalize Pass 3 child substrate

components::Child { born_tick: u64 } records a newborn's simulation birth tick. SocietyRules::life_stage controls attachment and maturation; absent rules preserve adult newborns. The marker itself performs no updates or RNG draws.

## Revitalize Pass 4 aging substrate

components::Life { born_tick: i64, lifespan_ticks: u64 } holds a resident's birth tick (negative for founders, who are generated already aged) and the lifespan drawn from its own salted splitmix64 stream. The zero-sized components::Elder marker is attached once age reaches AgingRule::elder_age_ticks. SocietyRules::aging controls both; absent rules (legacy, v8, v9) attach neither. Neither component updates itself or draws RNG; AgingSystem (priority 146) does the work.

## Revitalize Pass 5 cold-night substrate

components::Cold is a zero-sized marker for a settlement member spending a winter night inside FirewoodRule's cold window while its settlement's fire is out. FireSystem (priority 147, interval 60) attaches and removes it in AgentId order; HungerDecaySystem grows a marked agent's hunger by growth_rate × cold_hunger_permille / 1000 instead of growth_rate. SocietyRules::firewood controls it; absent rules (legacy, v8, v9, v10) never attach it. The marker does not update itself or draw RNG.

## Revitalize Pass 9 keyed draws and lineage

rng (sim_core::rng) holds the keyed-draw primitive: keyed_u64(seed, stream, subject, tick, slot, replicate) = five chained splitmix64 outputs, a pure function of its key; keyed_unit_f64, below / keyed_below (multiply-shift, one draw) and keyed_step derive from it. Stream constants (STREAM_MOVE_PAUSE, STREAM_MOVE_DX, STREAM_MOVE_DY, STREAM_LIFESPAN, STREAM_FOUNDER, STREAM_NEWBORN) are listed in STREAMS and must stay distinct. NoiseFork { tick, replicate } names a noise fork (SimResources::noise_fork, sim-engine).

components::Lineage { key: u64, births: u32 } is a resident's identity for keyed draws: founders Lineage::founder(seed, spawn index), newborns first_parent.child(seed) (advances the parent's births). AgentRules::keyed_draws (game_v15) controls attachment; v8–v14 never attach it. It never updates itself; it is never derived from AgentId.

## Revitalize Pass 16 temperament (`game_v20`)

components::Temperament { ns, ha, rd, p: u16 } (permille 0–1,000, mean 500; Cloninger's novelty seeking, harm avoidance, reward dependence, persistence) is an identity trait drawn once from the lineage key on STREAM_TEMPERAMENT (replicate 0): founders Temperament::founder(seed, key) (integer mean of three keyed uniforms per axis), newborns Temperament::child(seed, child_key, [parents]) (heritability 0.5 on the mid-parent, fresh deviation × 0.935). It exists only under AgentRules::temperament (None in every scene before game_v20) with keyed draws, and is inserted exactly where Hardiness is. TemperamentRule (rules.rs) holds the spans and the pure per-person functions tether / bedtime / social_rate / food_haul / wood_haul / departure_key; with effects false (game_v20_inert) each returns its base value. This is the first reduced step of the long-term temperament pipeline (genes → TCI → HEXACO → …), see docs/revitalize/TEMPERAMENT.md.
