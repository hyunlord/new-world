# Merged live bridge contract

2c52dd436842247bcbc3bf754f89b367b793c7fe

## Live world constructors

181:                &WorldGenConfig::game_v19(DEFAULT_GAME_SEED),
182:                WorldRules::game_v19(DEFAULT_GAME_SEED),
228:            &WorldGenConfig::game_v19(self.world_seed),
229:            WorldRules::game_v19(self.world_seed),

## get_tallies keys

fn tallies_to_dict(tallies: sim_engine::Tallies) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("births", tallies.births as i64);
    dict.set("deaths_starvation", tallies.deaths_starvation as i64);
    dict.set("deaths_dehydration", tallies.deaths_dehydration as i64);
    dict.set("deaths_combat", tallies.deaths_combat as i64);
    dict.set("deaths_old_age", tallies.deaths_old_age as i64);
    // Revitalize Pass 10: cold-night deaths under the exposure rule.
    dict.set("deaths_exposure", tallies.deaths_exposure as i64);
    // Revitalize Pass 11 tallies (display-only).
    dict.set("fissions", tallies.fissions as i64);
    dict.set("fissions_cancelled", tallies.fissions_cancelled as i64);
    dict.set("remnant_merges", tallies.remnant_merges as i64);
    // Revitalize Pass 12: wood a raining settlement burned wet (display-only).
    dict.set("wet_burn_wood", tallies.wet_burn_wood as i64);
    // Revitalize Pass 13: settled-area floor wood placed (display-only).
    dict.set("area_floor_placed", tallies.area_floor_placed as i64);
    // Revitalize Pass 5 tallies (display-only).
    dict.set("meals_from_store", tallies.meals_from_store as i64);
    dict.set("ground_meals", tallies.ground_meals as i64);
    dict.set("wood_trips", tallies.wood_trips as i64);
    dict.set("wood_gathered", tallies.wood_gathered as i64);
    dict.set("gather_preempted", tallies.gather_preempted as i64);
    dict.set("cold_member_hours", tallies.cold_member_hours as i64);
    dict
}
