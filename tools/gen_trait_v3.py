#!/usr/bin/env python3
"""Generate trait_defs_v3.json and localization for WorldSim Trait System v3.

Phase 1: Personality-based categories (Archetype 55, Shadow 15, Radiance 12).
Phase 2: Physical + Cognitive categories (Corpus 12, Nous 10).
Run from project root:  python3 tools/gen_trait_v3.py
"""

import json
import os

# ─── Threshold Constants (HEXACO mean=0.5, sd=0.15~0.25) ───
T_HH = 0.88   # ↑↑  top ~0.5%
T_H  = 0.83   # ↑   top ~1.4%
T_L  = 0.17   # ↓   bottom ~1.4%
T_LL = 0.12   # ↓↓  bottom ~0.5%

# ─── Category Display ───
DISPLAY = {
    "archetype": {"icon_color": "#C0C0C0", "border_style": "silver"},
    "shadow":    {"icon_color": "#DC143C", "border_style": "crimson"},
    "radiance":  {"icon_color": "#FFD700", "border_style": "gold"},
    "corpus":    {"icon_color": "#CD7F32", "border_style": "bronze"},
    "nous":      {"icon_color": "#007FFF", "border_style": "azure"},
    "awakened":  {"icon_color": "#800080", "border_style": "purple"},
    "bloodline": {"icon_color": "#800000", "border_style": "maroon"},
    "mastery":   {"icon_color": "#50C878", "border_style": "emerald"},
    "bond":      {"icon_color": "#FF007F", "border_style": "rose"},
    "fate":      {"icon_color": "#FF00FF", "border_style": "iridescent"},
}

# ─── Condition Helpers ───
def hx(axis: str, d: str, extreme: bool = False) -> dict:
    """HEXACO axis condition.  hx("H","high",True) → H ≥ 0.88"""
    if d == "high":
        return {"source": "hexaco", "axis": axis, "direction": "high",
                "threshold": T_HH if extreme else T_H}
    return {"source": "hexaco", "axis": axis, "direction": "low",
            "threshold": T_LL if extreme else T_L}

def val(key: str, d: str = "high", extreme: bool = True) -> dict:
    """Entity value condition.  val("martial") → martial ≥ 0.88"""
    if d == "high":
        return {"source": "value", "key": key, "direction": "high",
                "threshold": T_HH if extreme else T_H}
    return {"source": "value", "key": key, "direction": "low",
            "threshold": T_LL if extreme else T_L}

def body(axis: str, d: str = "high", extreme: bool = False) -> dict:
    """Body attribute condition. axis in {str,agi,end,tou,rec,dr}.
    Values are int 0-10000, normalized /10000 for threshold comparison."""
    if d == "high":
        return {"source": "body", "axis": axis, "direction": "high",
                "threshold": T_HH if extreme else T_H}
    return {"source": "body", "axis": axis, "direction": "low",
            "threshold": T_LL if extreme else T_L}

def intel(key: str, d: str = "high", extreme: bool = False) -> dict:
    """Gardner intelligence condition. key in {logical,linguistic,spatial,
    musical,kinesthetic,interpersonal,intrapersonal,naturalistic}.
    Values are float 0.02-0.98."""
    if d == "high":
        return {"source": "intelligence", "key": key, "direction": "high",
                "threshold": T_HH if extreme else T_H}
    return {"source": "intelligence", "key": key, "direction": "low",
            "threshold": T_LL if extreme else T_L}

# ─── Trait Builder ───
ACQ_TYPE_BY_CAT = {
    "archetype": "personality", "shadow": "personality", "radiance": "personality",
    "corpus": "physical", "nous": "cognitive",
    "awakened": "event", "bloodline": "genetic", "mastery": "skill",
    "bond": "relationship", "fate": "complex",
}

def T(tid: str, cat: str, conds: list, rarity: str = "auto",
      incompat: list = None, loss: dict = None) -> dict:
    if rarity == "auto":
        n = len(conds)
        has_extreme = any(c.get("threshold") in (T_HH, T_LL) for c in conds)
        if n >= 3:
            rarity = "legendary"
        elif n == 2:
            rarity = "epic"
        elif has_extreme:
            rarity = "rare"
        else:
            rarity = "uncommon"
    return {
        "id": tid,
        "name_key": "TRAIT_" + tid.upper() + "_NAME",
        "desc_key": "TRAIT_" + tid.upper() + "_DESC",
        "category": cat,
        "rarity": rarity,
        "acquisition": {
            "type": ACQ_TYPE_BY_CAT.get(cat, "personality"),
            "conditions": conds,
            "require_all": True
        },
        "effects": {},
        "display": DISPLAY.get(cat, DISPLAY["archetype"]),
        "incompatible_with": incompat or [],
        "loss_conditions": loss
    }

# ═══════════════════════════════════════════════════════════════
#  ARCHETYPE — Single-axis extremes (12)
# ═══════════════════════════════════════════════════════════════
ARCHETYPE_SINGLE = [
    T("A_incorruptible",  "archetype", [hx("H","high",True)], incompat=["A_serpent_tongue"]),
    T("A_serpent_tongue",  "archetype", [hx("H","low",True)],  incompat=["A_incorruptible"]),
    T("A_glass_heart",     "archetype", [hx("E","high",True)], incompat=["A_stone_blood"]),
    T("A_stone_blood",     "archetype", [hx("E","low",True)],  incompat=["A_glass_heart"]),
    T("A_bonfire",         "archetype", [hx("X","high",True)], incompat=["A_deep_well"]),
    T("A_deep_well",       "archetype", [hx("X","low",True)],  incompat=["A_bonfire"]),
    T("A_open_palm",       "archetype", [hx("A","high",True)], incompat=["A_iron_wall"]),
    T("A_iron_wall",       "archetype", [hx("A","low",True)],  incompat=["A_open_palm"]),
    T("A_clockwork",       "archetype", [hx("C","high",True)], incompat=["A_wildfire"]),
    T("A_wildfire",        "archetype", [hx("C","low",True)],  incompat=["A_clockwork"]),
    T("A_horizon_seeker",  "archetype", [hx("O","high",True)], incompat=["A_root_bound"]),
    T("A_root_bound",      "archetype", [hx("O","low",True)],  incompat=["A_horizon_seeker"]),
]

# ═══════════════════════════════════════════════════════════════
#  ARCHETYPE — 2-axis combos (22)
# ═══════════════════════════════════════════════════════════════
ARCHETYPE_DUAL = [
    T("A_silver_mask",    "archetype", [hx("H","low"),  hx("X","high")]),
    T("A_true_mirror",    "archetype", [hx("H","high"), hx("X","high")]),
    T("A_phantom",        "archetype", [hx("H","low"),  hx("X","low")]),
    T("A_confessor",      "archetype", [hx("H","high"), hx("X","low")]),
    T("A_tempest",        "archetype", [hx("E","high"), hx("A","low")]),
    T("A_still_water",    "archetype", [hx("E","low"),  hx("A","high")]),
    T("A_war_drum",       "archetype", [hx("X","high"), hx("A","low")]),
    T("A_hearth_keeper",  "archetype", [hx("X","high"), hx("A","high")]),
    T("A_obsidian_edge",  "archetype", [hx("E","low"),  hx("C","high")]),
    T("A_powder_keg",     "archetype", [hx("E","high"), hx("C","low")]),
    T("A_living_library", "archetype", [hx("C","high"), hx("O","high")]),
    T("A_autumn_leaf",    "archetype", [hx("C","low"),  hx("O","high")]),
    T("A_iron_oath",      "archetype", [hx("H","high"), hx("C","high")]),
    T("A_quicksilver",    "archetype", [hx("H","low"),  hx("C","low")]),
    T("A_dreamer",        "archetype", [hx("E","high"), hx("O","high")]),
    T("A_fortress_mind",  "archetype", [hx("E","low"),  hx("O","low")]),
    T("A_zealot",         "archetype", [hx("H","high"), hx("A","low")]),
    T("A_fox_path",       "archetype", [hx("H","low"),  hx("A","high")]),
    T("A_mountain_blood", "archetype", [hx("C","high"), hx("A","low")]),
    T("A_spring_wind",    "archetype", [hx("C","low"),  hx("A","high")]),
    T("A_torch_bearer",   "archetype", [hx("O","high"), hx("X","high")]),
    T("A_buried_stone",   "archetype", [hx("O","low"),  hx("X","low")]),
]

# ═══════════════════════════════════════════════════════════════
#  ARCHETYPE — 3-axis combos (8)
# ═══════════════════════════════════════════════════════════════
ARCHETYPE_TRIPLE = [
    T("A_gilded_tongue",   "archetype", [hx("X","high"), hx("H","low"),  hx("O","high")]),
    T("A_silent_judge",    "archetype", [hx("X","low"),  hx("H","high"), hx("C","high")]),
    T("A_embers",          "archetype", [hx("E","high"), hx("X","low"),  hx("O","low")]),
    T("A_dawn_hammer",     "archetype", [hx("C","high"), hx("X","high"), hx("A","low")]),
    T("A_woven_fate",      "archetype", [hx("E","high"), hx("H","high"), hx("A","high")]),
    T("A_tidal_mind",      "archetype", [hx("O","high"), hx("E","high"), hx("C","low")]),
    T("A_unmoving_peak",   "archetype", [hx("E","low"),  hx("A","low"),  hx("C","high")]),
    T("A_wandering_star",  "archetype", [hx("O","high"), hx("X","high"), hx("C","low")]),
]

# ═══════════════════════════════════════════════════════════════
#  ARCHETYPE — Value + personality hybrids (13)
# ═══════════════════════════════════════════════════════════════
ARCHETYPE_VALUE = [
    T("A_blood_price",     "archetype", [val("martial"),       hx("A","low")]),
    T("A_golden_scale",    "archetype", [val("commerce"),      hx("H","high")]),
    T("A_green_covenant",  "archetype", [val("nature"),        hx("O","high")]),
    T("A_eternal_student", "archetype", [val("knowledge"),     hx("O","high")]),
    T("A_broken_chain",    "archetype", [val("independence"),  hx("A","low")]),
    T("A_ancestor_voice",  "archetype", [val("tradition"),     hx("O","low")]),
    T("A_last_stand",      "archetype", [val("sacrifice"),     hx("E","high")]),
    T("A_merrymaker",      "archetype", [val("merriment"),     hx("X","high")]),
    T("A_veiled_blade",    "archetype", [val("cunning"),       hx("H","low")]),
    T("A_scales_keeper",   "archetype", [val("fairness"),      hx("H","high")]),
    T("A_craft_soul",      "archetype", [val("craftsmanship"), hx("C","high")]),
    T("A_lotus_eater",     "archetype", [val("leisure"),       hx("C","low")]),
    T("A_rival_seeker",    "archetype", [val("competition"),   hx("X","high")]),
]

# ═══════════════════════════════════════════════════════════════
#  SHADOW (15)
# ═══════════════════════════════════════════════════════════════
SHADOW = [
    T("S_hollow_crown",    "shadow", [hx("H","low",True), hx("E","low",True), hx("A","low")], "legendary"),
    T("S_puppet_master",   "shadow", [hx("H","low",True), hx("C","high"),     hx("X","high")], "legendary"),
    T("S_mirror_throne",   "shadow", [hx("H","low"),      hx("X","high",True),hx("E","high")], "legendary"),
    T("S_cracked_mirror",  "shadow", [hx("H","low"),      hx("E","high",True),hx("X","low")], "legendary"),
    T("S_red_smile",       "shadow", [hx("A","low",True), hx("E","low",True), val("martial","high",False)], "legendary"),
    T("S_honeyed_venom",   "shadow", [hx("H","low",True), hx("X","high"),     hx("A","high")], "legendary",
                           incompat=["A_incorruptible"]),
    T("S_throne_hunger",   "shadow", [val("power"),        hx("H","low"),      hx("A","low")], "legendary"),
    T("S_ash_prophet",     "shadow", [hx("O","high"),      hx("E","high"),     hx("H","low"), hx("X","high")], "legendary"),
    T("S_cold_harvest",    "shadow", [hx("H","low",True),  hx("C","high"),     hx("A","low"), hx("E","low")], "legendary"),
    T("S_jealous_flame",   "shadow", [hx("E","high",True), hx("A","low"),      val("romance","high",False)], "legendary"),
    T("S_carrion_comfort", "shadow", [hx("E","low",True),  hx("C","low",True), hx("O","low")], "legendary"),
    T("S_web_weaver",      "shadow", [hx("H","low"),       hx("C","high"),     hx("X","high"), hx("A","high")], "legendary"),
    T("S_night_bloom",     "shadow", [hx("X","high"),      hx("E","low"),      hx("H","low"),  hx("A","high")], "legendary"),
    T("S_broken_compass",  "shadow", [hx("H","low",True),  hx("C","low",True)], "legendary",
                           incompat=["A_incorruptible", "A_clockwork"]),
    T("S_iron_cradle",     "shadow", [hx("C","high",True), hx("A","low",True), hx("E","low")], "legendary"),
]

# ═══════════════════════════════════════════════════════════════
#  RADIANCE (12)
# ═══════════════════════════════════════════════════════════════
RADIANCE = [
    T("R_golden_heart",    "radiance", [hx("H","high",True), hx("A","high",True), hx("E","high")], "legendary"),
    T("R_north_star",      "radiance", [hx("H","high",True), hx("C","high",True), hx("X","high")], "legendary"),
    T("R_unbreaking",      "radiance", [hx("E","low"),       hx("C","high"),      hx("A","high"), hx("H","high")], "legendary"),
    T("R_world_bridge",    "radiance", [hx("O","high",True), hx("A","high",True), hx("X","high")], "legendary"),
    T("R_seed_keeper",     "radiance", [hx("C","high"),      hx("O","high"),      val("knowledge")], "legendary"),
    T("R_hearthfire",      "radiance", [hx("A","high",True), hx("X","high"),      hx("E","high")], "legendary"),
    T("R_calm_harbor",     "radiance", [hx("E","low"),       hx("A","high"),      hx("X","low"), hx("H","high")], "legendary"),
    T("R_dawn_singer",     "radiance", [hx("O","high",True), hx("E","high"),      val("artwork")], "legendary"),
    T("R_iron_promise",    "radiance", [hx("H","high",True), val("loyalty"),      hx("C","high")], "legendary"),
    T("R_gentle_thunder",  "radiance", [hx("A","high"),      hx("X","high"),      hx("E","low"), hx("C","high")], "legendary"),
    T("R_wellspring",      "radiance", [hx("A","high",True), hx("E","high"),      hx("O","high")], "legendary"),
    T("R_first_fire",      "radiance", [hx("O","high",True), hx("C","high"),      hx("H","high")], "legendary"),
]

# ═══════════════════════════════════════════════════════════════
#  CORPUS — Body extremes (12)
#  Body axes: str, agi, end, tou, rec, dr (int 0-10000, normalized /10000)
# ═══════════════════════════════════════════════════════════════
CORPUS = [
    T("B_titan",         "corpus", [body("str","high",True), body("tou","high")],
      incompat=["B_withered"]),
    T("B_wraith",        "corpus", [body("agi","high",True), body("str","low")],
      incompat=["B_titan"]),
    T("B_mountain_body", "corpus", [body("tou","high",True), body("end","high",True)]),
    T("B_phoenix_blood", "corpus", [body("rec","high",True), body("dr","high",True)]),
    T("B_glass_cannon",  "corpus", [body("str","high",True), body("tou","low",True)]),
    T("B_iron_lungs",    "corpus", [body("end","high",True)]),
    T("B_paper_skin",    "corpus", [body("tou","low",True),  body("dr","low",True)]),
    T("B_perfect_form",  "corpus", [body("agi","high",True), body("rec","high")]),
    T("B_withered",      "corpus", [body("str","low",True),  body("end","low",True)],
      incompat=["B_titan"]),
    T("B_cat_eyes",      "corpus", [body("agi","high",True)]),
    T("B_slow_healer",   "corpus", [body("rec","low",True)]),
    T("B_undying",       "corpus", [body("tou","high",True), body("rec","high",True)]),
]

# ═══════════════════════════════════════════════════════════════
#  NOUS — Cognitive extremes (10)
#  Intelligence keys: logical, linguistic, spatial, musical,
#  kinesthetic, interpersonal, intrapersonal, naturalistic (float 0.02-0.98)
# ═══════════════════════════════════════════════════════════════
NOUS = [
    T("N_polymath",       "nous", [intel("logical","high",True),  intel("linguistic","high",True)]),
    T("N_silver_voice",   "nous", [intel("linguistic","high",True), intel("interpersonal","high",True)]),
    T("N_architects_eye", "nous", [intel("spatial","high",True),  intel("logical","high")]),
    T("N_beast_tongue",   "nous", [intel("naturalistic","high",True), intel("kinesthetic","high")]),
    T("N_war_savant",     "nous", [intel("kinesthetic","high",True), intel("spatial","high"), intel("logical","high")]),
    T("N_inner_eye",      "nous", [intel("intrapersonal","high",True)]),
    T("N_natures_child",  "nous", [intel("naturalistic","high",True)]),
    T("N_muse_touched",   "nous", [intel("musical","high",True), intel("linguistic","high")]),
    T("N_dim",            "nous", [intel("logical","low",True),   intel("linguistic","low",True)]),
    T("N_feral_mind",     "nous", [intel("kinesthetic","high",True), intel("interpersonal","low",True)]),
]

# ═══════════════════════════════════════════════════════════════
#  AWAKENED — Event-based traits (18)
#  Not auto-evaluated from stats — granted by game systems via grant_trait().
# ═══════════════════════════════════════════════════════════════

def E(tid: str, cat: str, trigger: str, rarity: str = "rare",
      incompat: list = None, loss: dict = None) -> dict:
    """Event-triggered trait builder. No threshold conditions — granted by systems."""
    return {
        "id": tid,
        "name_key": "TRAIT_" + tid.upper() + "_NAME",
        "desc_key": "TRAIT_" + tid.upper() + "_DESC",
        "category": cat,
        "rarity": rarity,
        "acquisition": {
            "type": ACQ_TYPE_BY_CAT.get(cat, "event"),
            "trigger": trigger,
            "conditions": [],
            "require_all": True
        },
        "effects": {},
        "display": DISPLAY.get(cat, DISPLAY["awakened"]),
        "incompatible_with": incompat or [],
        "loss_conditions": loss
    }

AWAKENED = [
    E("W_scarred_soul",      "awakened", "trauma_count_gte_3", "rare"),
    E("W_battle_forged",     "awakened", "combat_survived_gte_5", "rare"),
    E("W_widows_frost",      "awakened", "spouse_or_child_death", "rare"),
    E("W_twice_born",        "awakened", "lethal_injury_survived", "rare"),
    E("W_oath_breaker",      "awakened", "major_promise_broken", "rare"),
    E("W_kinslayer",         "awakened", "family_or_kin_killed", "epic"),
    E("W_exile_risen",       "awakened", "exile_then_success", "epic"),
    E("W_first_kill",        "awakened", "first_kill", "rare"),
    E("W_old_wolf",          "awakened", "age_60_plus_combat_veteran", "epic"),
    E("W_broken_faith",      "awakened", "faith_loss_event", "rare",
      loss={"type": "conversion", "note": "Replaced on new faith adoption"}),
    E("W_touched_by_gods",   "awakened", "player_direct_intervention", "legendary"),
    E("W_famine_survivor",   "awakened", "prolonged_starvation_survived", "rare"),
    E("W_plague_walker",     "awakened", "epidemic_survived", "rare"),
    E("W_crown_weight",      "awakened", "leader_5_plus_years", "epic"),
    E("W_mothers_fury",      "awakened", "child_endangered_or_killed", "rare"),
    E("W_dreaming_prophet",  "awakened", "intrapersonal_high_plus_religious_experience", "epic"),
    E("W_chain_breaker",     "awakened", "escaped_oppression", "rare"),
    E("W_wanderers_return",  "awakened", "long_wandering_then_settled", "epic"),
]

# ═══════════════════════════════════════════════════════════════
#  BLOODLINE — Genetic traits (25)
#  Granted at birth by birth system based on parent genetics.
#  inheritance field: dominant/recessive/maternal/paternal/founder
# ═══════════════════════════════════════════════════════════════

def G(tid: str, trigger: str, inheritance: str, rarity: str = "rare",
      incompat: list = None, loss: dict = None) -> dict:
    """Genetic trait builder with inheritance metadata."""
    d = E(tid, "bloodline", trigger, rarity, incompat, loss)
    d["acquisition"]["inheritance"] = inheritance
    return d

BLOODLINE_POS = [
    G("L_giants_marrow",       "genetic_dominant",  "dominant"),
    G("L_hawks_gaze",          "genetic_dominant",  "dominant"),
    G("L_winter_blood",        "genetic_recessive", "recessive"),
    G("L_summer_veins",        "genetic_recessive", "recessive"),
    G("L_iron_liver",          "genetic_dominant",  "dominant"),
    G("L_mothers_intuition",   "genetic_maternal",  "maternal"),
    G("L_war_seed",            "genetic_paternal",  "paternal"),
    G("L_silver_tongue_blood", "genetic_dominant",  "dominant"),
    G("L_deep_roots",          "genetic_recessive", "recessive"),
    G("L_starlit_mind",        "genetic_recessive", "recessive"),
    G("L_beast_affinity",      "genetic_maternal",  "maternal"),
    G("L_stone_bones",         "genetic_dominant",  "dominant"),
    G("L_dawn_blessed",        "genetic_founder",   "founder", "legendary"),
]

BLOODLINE_NEG = [
    G("L_thin_blood",          "genetic_recessive", "recessive"),
    G("L_moon_sickness",       "genetic_recessive", "recessive"),
    G("L_hollow_bones",        "genetic_recessive", "recessive"),
    G("L_blood_fury",          "genetic_paternal",  "paternal"),
    G("L_cursed_womb",         "genetic_maternal",  "maternal"),
    G("L_short_wick",          "genetic_dominant",  "dominant"),
    G("L_wandering_mind",      "genetic_recessive", "recessive"),
]

BLOODLINE_NEUTRAL = [
    G("L_twin_souled",         "genetic_recessive", "recessive"),
    G("L_old_blood",           "genetic_founder",   "founder", "legendary"),
    G("L_echo_face",           "genetic_dominant",  "dominant"),
    G("L_fey_touched",         "genetic_founder",   "founder", "legendary"),
    G("L_ember_heart",         "genetic_recessive", "recessive"),
]

BLOODLINE = BLOODLINE_POS + BLOODLINE_NEG + BLOODLINE_NEUTRAL

# ═══════════════════════════════════════════════════════════════
#  MASTERY — Skill-based traits (20)
#  Granted when skill ≥ 18. Lost after 3-5 years of non-use.
# ═══════════════════════════════════════════════════════════════
MASTERY = [
    E("M_anvils_echo",     "mastery", "blacksmithing_gte_18", "epic",
      loss={"type": "decay", "years": 5}),
    E("M_green_thumb",     "mastery", "farming_gte_18", "epic",
      loss={"type": "decay", "years": 3}),
    E("M_death_dealer",    "mastery", "combat_skill_gte_18", "epic",
      loss={"type": "decay", "skill_below": 15}),
    E("M_tongue_of_ages",  "mastery", "persuasion_gte_18_diplomacy_gte_15", "legendary",
      loss={"type": "decay", "years": 3}),
    E("M_bone_setter",     "mastery", "medicine_gte_18", "epic",
      loss={"type": "decay", "years": 5}),
    E("M_wall_maker",      "mastery", "architecture_gte_18_fortification_gte_15", "legendary",
      loss={"type": "decay", "years": 5}),
    E("M_thread_weaver",   "mastery", "weaving_gte_18", "epic",
      loss={"type": "decay", "years": 5}),
    E("M_song_keeper",     "mastery", "music_gte_18_poetry_gte_15", "legendary",
      loss={"type": "decay", "years": 3}),
    E("M_shadow_step",     "mastery", "espionage_gte_18", "epic",
      loss={"type": "decay", "years": 5}),
    E("M_kings_hand",      "mastery", "administration_gte_18_logistics_gte_15", "legendary",
      loss={"type": "decay", "years": 3}),
    E("M_fire_tamer",      "mastery", "cooking_gte_18", "epic",
      loss={"type": "decay", "years": 3}),
    E("M_star_reader",     "mastery", "astronomy_gte_18_math_gte_15", "legendary",
      loss={"type": "decay", "years": 5}),
    E("M_horse_whisperer", "mastery", "animal_training_gte_18", "epic",
      loss={"type": "decay", "years": 3}),
    E("M_law_speaker",     "mastery", "law_gte_18_persuasion_gte_15", "legendary",
      loss={"type": "decay", "years": 5}),
    E("M_stone_singer",    "mastery", "mining_gte_18", "epic",
      loss={"type": "decay", "years": 3}),
    E("M_root_finder",     "mastery", "herbalism_gte_18_foraging_gte_15", "legendary",
      loss={"type": "decay", "years": 3}),
    E("M_death_midwife",   "mastery", "medicine_gte_15_deaths_witnessed_10", "epic"),
    E("M_bridge_builder",  "mastery", "negotiation_gte_18_mediations_5", "legendary",
      loss={"type": "betrayal", "note": "Lost after 3 biased mediations"}),
    E("M_edge_walker",     "mastery", "combat_gte_15_espionage_gte_15", "epic",
      loss={"type": "decay", "years": 5}),
    E("M_word_carver",     "mastery", "reading_gte_18_poetry_gte_15", "legendary",
      loss={"type": "decay", "years": 3}),
]

# ═══════════════════════════════════════════════════════════════
#  BOND — Relationship-based traits (20)
# ═══════════════════════════════════════════════════════════════
BOND = [
    E("D_soul_tethered",   "bond", "intimacy_95_10_years", "legendary",
      loss={"type": "conversion", "note": "Converts to W_widows_frost on target death"}),
    E("D_blood_oath",      "bond", "combat_3_survived_together", "epic"),
    E("D_eternal_grudge",  "bond", "intimacy_neg80_harmed", "epic",
      loss={"type": "condition", "note": "Lost on revenge completion or target death"}),
    E("D_shepherds_heart", "bond", "raised_5_children", "epic"),
    E("D_twice_betrayed",  "bond", "betrayed_twice_from_intimacy_60", "epic",
      loss={"type": "condition", "note": "Lost after secure attachment + 10 years"}),
    E("D_pack_alpha",      "bond", "10_most_trusted", "epic",
      loss={"type": "condition", "note": "Lost when followers drop below 5"}),
    E("D_lone_wolf",       "bond", "5_years_no_close_relations", "rare",
      loss={"type": "condition", "note": "Lost when intimacy 60+ formed"}),
    E("D_unrequited",      "bond", "love_80_unreciprocated_3_years", "rare",
      loss={"type": "decay", "years": 5}),
    E("D_kingmaker",       "bond", "supported_leader_elected_3_times", "legendary",
      loss={"type": "condition", "note": "Lost when all supported leaders fail"}),
    E("D_mirror_bond",     "bond", "shared_3_traits_intimacy_85", "legendary",
      loss={"type": "condition", "note": "Lost on target death or 5 year separation"}),
    E("D_debt_of_life",    "bond", "saved_from_lethal_danger", "epic",
      loss={"type": "condition", "note": "Lost when debt repaid (saving benefactor)"}),
    E("D_bitter_mentor",   "bond", "student_surpassed_master", "epic"),
    E("D_orphans_resolve", "bond", "both_parents_died_before_15", "rare"),
    E("D_last_of_line",    "bond", "all_direct_kin_dead", "epic",
      loss={"type": "conversion", "note": "Converts on child birth"}),
    E("D_forged_family",   "bond", "5_non_kin_intimacy_85", "epic",
      loss={"type": "condition", "note": "Lost when 3+ members leave/die"}),
    E("D_cursed_lover",    "bond", "3_lovers_died", "epic"),
    E("D_sworn_enemy",     "bond", "mutual_neg90_sworn", "epic",
      loss={"type": "condition", "note": "Emptiness after target death"}),
    E("D_foster_bond",     "bond", "raised_others_child_3_years", "rare"),
    E("D_river_between",   "bond", "close_to_both_hostile_factions", "epic",
      loss={"type": "condition", "note": "Lost when one faction collapses"}),
    E("D_chain_of_grief",  "bond", "3_close_people_died", "epic"),
]

# ═══════════════════════════════════════════════════════════════
#  FATE — Legendary complex-condition traits (15)
# ═══════════════════════════════════════════════════════════════
FATE = [
    E("F_world_shaper",     "fate", "3_techs_discovered_O_extreme", "legendary"),
    E("F_peoples_flame",    "fate", "revolution_success_X_high_charisma", "legendary"),
    E("F_deathless_name",   "fate", "20_history_events_5_settlements_know", "legendary"),
    E("F_doom_bringer",     "fate", "shadow_trait_3_wars_settlement_destroyed", "legendary"),
    E("F_last_hope",        "fate", "radiance_trait_3_crises_saved", "legendary"),
    E("F_god_touched",      "fate", "player_3_interventions", "legendary"),
    E("F_curse_bearer",     "fate", "2_neg_bloodlines_3_traumas", "legendary"),
    E("F_bridge_of_ages",   "fate", "age_60_mastery_trait_5_students", "legendary"),
    E("F_twin_crowned",     "fate", "simultaneous_leader_2_settlements", "legendary"),
    E("F_seasons_child",    "fate", "winter_blood_and_summer_veins", "legendary"),
    E("F_ender_of_lines",   "fate", "sworn_enemy_3_bloodlines_ended", "legendary"),
    E("F_silent_founder",   "fate", "settlement_founded_prospered_no_leader", "legendary"),
    E("F_prophet_of_ruin",  "fate", "dreaming_prophet_3_prophecies_true", "legendary"),
    E("F_uncrowned_king",   "fate", "charisma_top1_refused_leadership_3", "legendary"),
    E("F_memory_keeper",    "fate", "wanderers_return_word_carver_age_60", "legendary"),
]

# ═══════════════════════════════════════════════════════════════
#  SYNERGY — Trait combinations (40)
#  Auto-detected when entity has required_traits. Display + bonus effects.
# ═══════════════════════════════════════════════════════════════

def Y(tid: str, required: list, rarity: str = "legendary") -> dict:
    """Synergy trait builder. required = list of trait IDs that must all be present."""
    return {
        "id": tid,
        "name_key": "TRAIT_" + tid.upper() + "_NAME",
        "desc_key": "TRAIT_" + tid.upper() + "_DESC",
        "category": "synergy",
        "rarity": rarity,
        "acquisition": {
            "type": "synergy",
            "required_traits": required,
            "conditions": [],
            "require_all": True
        },
        "effects": {},
        "display": DISPLAY.get("synergy", {"icon_color": "#FF00FF", "border_style": "iridescent"}),
        "incompatible_with": [],
        "loss_conditions": {"type": "condition", "note": "Lost when any required trait is lost"}
    }

# 2-trait synergies (30)
SYNERGY_2 = [
    Y("Y_frozen_fury",       ["A_stone_blood", "A_iron_wall"]),
    Y("Y_burning_glass",     ["A_glass_heart", "A_bonfire"]),
    Y("Y_iron_sun",          ["A_incorruptible", "A_bonfire"]),
    Y("Y_velvet_knife",      ["A_silver_mask", "A_fox_path"]),
    Y("Y_storm_crown",       ["A_tempest", "A_war_drum"]),
    Y("Y_silent_forge",      ["A_deep_well", "A_clockwork"]),
    Y("Y_broken_mirror",     ["S_cracked_mirror", "W_scarred_soul"]),
    Y("Y_holy_fire",         ["A_zealot", "R_golden_heart"]),
    Y("Y_wolves_pact",       ["D_blood_oath", "W_battle_forged"]),
    Y("Y_poisoned_well",     ["S_hollow_crown", "S_puppet_master"]),
    Y("Y_bleeding_root",     ["A_root_bound", "L_deep_roots"]),
    Y("Y_scarred_diamond",   ["W_scarred_soul", "R_unbreaking"]),
    Y("Y_autumn_hymn",       ["A_autumn_leaf", "N_muse_touched"]),
    Y("Y_night_garden",      ["A_deep_well", "A_dreamer"]),
    Y("Y_living_monument",   ["B_titan", "A_mountain_blood"]),
    Y("Y_ember_prophet",     ["A_embers", "W_dreaming_prophet"]),
    Y("Y_gentle_cage",       ["A_open_palm", "A_woven_fate"]),
    Y("Y_blood_architect",   ["A_obsidian_edge", "A_dawn_hammer"]),
    Y("Y_truth_serum",       ["A_true_mirror", "N_silver_voice"]),
    Y("Y_chain_march",       ["A_broken_chain", "W_chain_breaker"]),
    Y("Y_fading_star",       ["A_wandering_star", "W_old_wolf"]),
    Y("Y_red_garden",        ["S_red_smile", "D_cursed_lover"]),
    Y("Y_crown_of_thorns",   ["W_crown_weight", "A_woven_fate"]),
    Y("Y_titans_mercy",      ["B_titan", "R_gentle_thunder"]),
    Y("Y_hollow_saint",      ["S_hollow_crown", "R_golden_heart"]),
    Y("Y_mirror_war",        ["A_rival_seeker", "D_sworn_enemy"]),
    Y("Y_winters_bloom",     ["L_winter_blood", "L_ember_heart"]),
    Y("Y_golden_chains",     ["A_iron_oath", "D_debt_of_life"]),
    Y("Y_dream_forge",       ["A_dreamer", "M_anvils_echo"]),
    Y("Y_plague_saint",      ["W_plague_walker", "R_golden_heart"]),
]

# 3-trait synergies (10)
SYNERGY_3 = [
    Y("Y_god_killer",        ["S_hollow_crown", "B_titan", "M_death_dealer"]),
    Y("Y_eternal_flame",     ["R_north_star", "F_peoples_flame", "R_iron_promise"]),
    Y("Y_three_faced",       ["A_silver_mask", "S_puppet_master", "D_kingmaker"]),
    Y("Y_pain_weaver",       ["W_scarred_soul", "R_wellspring", "M_song_keeper"]),
    Y("Y_ashes_dawn",        ["W_famine_survivor", "W_chain_breaker", "F_world_shaper"]),
    Y("Y_unmoving_storm",    ["A_unmoving_peak", "B_titan", "S_iron_cradle"]),
    Y("Y_bleeding_compass",  ["A_wandering_star", "W_scarred_soul", "R_world_bridge"]),
    Y("Y_moons_cradle",      ["L_moon_sickness", "A_tidal_mind", "N_muse_touched"]),
    Y("Y_blood_remembers",   ["L_old_blood", "A_ancestor_voice", "F_memory_keeper"]),
    Y("Y_dawn_of_war",       ["W_battle_forged", "A_dawn_hammer", "A_last_stand"]),
]

SYNERGY = SYNERGY_2 + SYNERGY_3

# ═══════════════════════════════════════════════════════════════
#  EFFECTS — Mechanical effect definitions per trait
#  Keys = trait ID, Values = list of effect dicts
#  Each effect: {system, target, op, value, [condition], [tags], [priority]}
#  Populated incrementally: Part 1 (35 traits), Parts 2-5 (remaining ~207)
# ═══════════════════════════════════════════════════════════════
EFFECTS = {
    # ── Archetype: Single-axis (12 traits) ──
    "A_incorruptible": [
        {"system": "behavior", "target": ["accept_bribe", "steal", "fraud", "embezzle"], "op": "remove", "value": True, "tags": ["integrity"]},
        {"system": "behavior", "target": "trade_pricing", "op": "override", "value": "fair_price_only"},
        {"system": "derived", "target": "trustworthiness", "op": "add", "value": 0.25},
        {"system": "reputation", "target": "tags", "op": "tag", "value": "untarnished"},
        {"system": "behavior", "target": ["compromise", "bluff", "deceive"], "op": "remove", "value": True},
        {"system": "behavior", "target": "diplomacy_filter", "op": "set", "value": "reject_if_unfair"},
        {"system": "stress", "target": "corruption_exposure", "op": "set", "value": 0.1, "condition": "leader.has_tag('corrupt')"},
    ],
    "A_serpent_tongue": [
        {"system": "behavior", "target": ["fraud", "smuggle", "bribe", "forge_document"], "op": "unlock", "value": True},
        {"system": "skill", "target": ["deception", "persuasion"], "op": "mult", "value": 1.4},
        {"system": "relationship", "target": "betrayal_cooldown", "op": "mult", "value": 0.0},
        {"system": "stress", "target": "betrayal_guilt", "op": "immunity", "value": True},
        {"system": "reputation", "target": "negative_event_impact", "op": "mult", "value": 2.0, "condition": "caught_in_act"},
        {"system": "relationship", "target": "trust_repair_rate", "op": "set", "value": 0.0, "condition": "target.knows_betrayal"},
        {"system": "derived", "target": "deception_resistance", "op": "add", "value": 0.3},
    ],
    "A_glass_heart": [
        {"system": "emotion", "target": "intensity_mult", "op": "set", "value": 2.0},
        {"system": "emotion", "target": "decay_rate", "op": "mult", "value": 0.5},
        {"system": "stress", "target": "accumulation_rate", "op": "mult", "value": 1.8},
        {"system": "stress", "target": "mental_break_threshold", "op": "add", "value": -0.15},
        {"system": "stress", "target": "break_types", "op": "replace", "value": {"crying_fit": 0.4, "creative_frenzy": 0.3, "catatonic": 0.2, "berserk": 0.1}},
        {"system": "skill", "target": ["music", "poetry", "painting", "dance", "theater"], "op": "mult", "value": 1.3, "tags": ["art_bonus"]},
        {"system": "memory", "target": "trauma_intensity", "op": "mult", "value": 1.5},
        {"system": "memory", "target": "positive_intensity", "op": "mult", "value": 1.5},
        {"system": "combat", "target": "morale", "op": "add", "value": -0.3},
        {"system": "combat", "target": "flee_threshold", "op": "add", "value": 0.15},
        {"system": "relationship", "target": "bond_event_impact", "op": "mult", "value": 2.0},
    ],
    "A_stone_blood": [
        {"system": "stress", "target": "accumulation_rate", "op": "mult", "value": 0.2},
        {"system": "stress", "target": "mental_break_threshold", "op": "add", "value": 0.3},
        {"system": "emotion", "target": "fear", "op": "ceil", "value": 0.1},
        {"system": "emotion", "target": ["joy", "sadness", "trust", "surprise"], "op": "ceil", "value": 0.5},
        {"system": "relationship", "target": "intimacy_gain_rate", "op": "mult", "value": 0.5},
        {"system": "relationship", "target": "intimacy", "op": "ceil", "value": 70},
        {"system": "behavior", "target": ["console", "empathize", "cry", "express_grief"], "op": "remove", "value": True},
        {"system": "combat", "target": "morale_floor", "op": "set", "value": 0.4},
    ],
    "A_bonfire": [
        {"system": "relationship", "target": "intimacy_gain_rate", "op": "mult", "value": 2.0},
        {"system": "relationship", "target": "first_impression", "op": "add", "value": 25},
        {"system": "aura", "target": "joy", "op": "set", "value": {"radius": 3, "intensity": 0.15, "target_filter": "all"}},
        {"system": "need", "target": "belonging", "op": "set", "value": {"decay_rate_mult": 3.0}},
        {"system": "need", "target": "intimacy", "op": "set", "value": {"decay_rate_mult": 2.5}},
        {"system": "stress", "target": "isolation_stress", "op": "mult", "value": 4.0},
        {"system": "stress", "target": "break_types", "op": "replace", "value": {"sobbing_fit": 0.5, "desperate_socializing": 0.3, "wander": 0.2}, "condition": "isolation_days > 3"},
        {"system": "skill", "target": ["persuasion", "negotiation", "teaching"], "op": "mult", "value": 1.2},
        {"system": "event", "target": "festival_effect", "op": "mult", "value": 1.5},
    ],
    "A_deep_well": [
        {"system": "skill", "target": "all_work", "op": "mult", "value": 1.3, "condition": "is_alone"},
        {"system": "derived", "target": "wisdom", "op": "add", "value": 0.15, "tags": ["introspection"]},
        {"system": "stress", "target": "introspection_recovery", "op": "mult", "value": 2.0},
        {"system": "relationship", "target": "intimacy_gain_rate", "op": "mult", "value": 0.0, "condition": "in_social_event"},
        {"system": "derived", "target": "charisma", "op": "add", "value": -0.3},
        {"system": "stress", "target": "crowd_stress", "op": "set", "value": 0.08, "condition": "nearby_agents > 5"},
        {"system": "stress", "target": "isolation_stress", "op": "set", "value": 0.0},
        {"system": "need", "target": "belonging", "op": "set", "value": {"decay_rate_mult": 0.3}},
    ],
    "A_open_palm": [
        {"system": "skill", "target": ["negotiation", "mediation"], "op": "mult", "value": 1.4},
        {"system": "relationship", "target": "alliance_decay", "op": "mult", "value": 0.7},
        {"system": "behavior", "target": ["refuse_request", "reject_proposal", "deny_entry"], "op": "remove", "value": True},
        {"system": "behavior", "target": "exploitation_detection", "op": "mult", "value": 0.5},
        {"system": "need", "target": "autonomy", "op": "set", "value": {"satisfaction_mult": 0.5}},
        {"system": "stress", "target": "manipulation_resistance", "op": "mult", "value": 0.5},
    ],
    "A_iron_wall": [
        {"system": "stress", "target": "manipulation_resistance", "op": "set", "value": 1.0},
        {"system": "behavior", "target": "intimidation_resistance", "op": "add", "value": 0.5},
        {"system": "skill", "target": "debate", "op": "mult", "value": 1.3},
        {"system": "behavior", "target": "cooperation_accept", "op": "mult", "value": 0.6},
        {"system": "relationship", "target": "intimacy_decay_rate", "op": "mult", "value": 2.0},
        {"system": "behavior", "target": ["compromise", "concede", "apologize"], "op": "set", "value": {"weight_mult": 0.2}},
        {"system": "derived", "target": "intimidation", "op": "add", "value": 0.2},
    ],
    "A_clockwork": [
        {"system": "skill", "target": "all_work", "op": "mult", "value": 1.25},
        {"system": "crafting", "target": "quality_bonus", "op": "add", "value": 1},
        {"system": "behavior", "target": ["break_rule", "skip_task", "shirk_duty"], "op": "remove", "value": True},
        {"system": "behavior", "target": "improvisation", "op": "mult", "value": 0.6},
        {"system": "stress", "target": "unexpected_event_stress", "op": "mult", "value": 2.5},
        {"system": "behavior", "target": "accept_change", "op": "mult", "value": 0.3, "condition": "change_type == 'schedule' OR change_type == 'method'"},
        {"system": "stress", "target": "routine_disruption", "op": "set", "value": 0.15},
    ],
    "A_wildfire": [
        {"system": "behavior", "target": "improvisation", "op": "mult", "value": 1.4},
        {"system": "behavior", "target": "crisis_response_speed", "op": "mult", "value": 1.3},
        {"system": "skill", "target": "long_task_completion", "op": "mult", "value": 0.5},
        {"system": "behavior", "target": "promise_fulfillment", "op": "mult", "value": 0.6},
        {"system": "behavior", "target": "rule_compliance", "op": "mult", "value": 0.3},
        {"system": "stress", "target": "repetition_stress", "op": "set", "value": 0.1, "condition": "same_task_days > 3"},
        {"system": "emotion", "target": "joy", "op": "add", "value": 0.3, "condition": "new_task_started", "tags": ["novelty_burst"]},
    ],
    "A_horizon_seeker": [
        {"system": "skill", "target": "new_skill_learning", "op": "mult", "value": 1.4},
        {"system": "event", "target": "discovery_chance", "op": "mult", "value": 2.0},
        {"system": "behavior", "target": "migration_resistance", "op": "set", "value": 0.0},
        {"system": "need", "target": "competence", "op": "set", "value": {"satisfaction_mult": 0.5}, "condition": "same_job_years > 1"},
        {"system": "stress", "target": "tradition_enforcement", "op": "set", "value": 0.1, "condition": "settlement.values.TRADITION > 0.7"},
        {"system": "behavior", "target": "explore_unknown", "op": "inject", "value": {"priority": 0.7}, "condition": "unknown_entity_nearby OR new_technology_available"},
    ],
    "A_root_bound": [
        {"system": "skill", "target": "traditional_work", "op": "mult", "value": 1.2},
        {"system": "values", "target": "drift_rate", "op": "mult", "value": 0.2},
        {"system": "skill", "target": "new_skill_learning", "op": "mult", "value": 0.5},
        {"system": "behavior", "target": ["adopt_innovation", "experiment", "research"], "op": "remove", "value": True},
        {"system": "behavior", "target": "migration_resistance", "op": "set", "value": 1.0},
        {"system": "stress", "target": "displacement_stress", "op": "set", "value": 0.2, "condition": "not_in_birth_settlement"},
    ],
    # ── Archetype: Dual-axis (18 traits) ──
    "A_silver_mask": [
        {"system": "relationship", "target": "first_impression", "op": "add", "value": 40},
        {"system": "relationship", "target": "trust", "op": "ceil", "value": 50, "condition": "relationship_duration > 365"},
        {"system": "behavior", "target": "maintain_cover", "op": "unlock", "value": True, "tags": ["espionage"]},
        {"system": "behavior", "target": "fake_emotion", "op": "unlock", "value": True},
        {"system": "skill", "target": ["espionage", "diplomacy", "deception"], "op": "mult", "value": 1.5},
        {"system": "behavior", "target": ["express_true_feelings", "confess", "genuine_apology"], "op": "remove", "value": True},
    ],
    "A_true_mirror": [
        {"system": "derived", "target": "charisma", "op": "add", "value": 0.3},
        {"system": "behavior", "target": ["lie", "deceive", "bluff", "fake_emotion"], "op": "remove", "value": True},
        {"system": "skill", "target": ["espionage", "deception"], "op": "mult", "value": 0.0},
        {"system": "relationship", "target": "trust_gain_rate", "op": "mult", "value": 2.0},
        {"system": "derived", "target": "trustworthiness", "op": "add", "value": 0.3},
        {"system": "behavior", "target": "suspicion", "op": "mult", "value": 0.3},
    ],
    "A_phantom": [
        {"system": "reputation", "target": "spread_speed", "op": "mult", "value": 0.1},
        {"system": "skill", "target": ["assassination", "theft", "infiltration"], "op": "mult", "value": 1.5},
        {"system": "relationship", "target": "memory_decay_rate", "op": "mult", "value": 3.0},
        {"system": "derived", "target": "charisma", "op": "add", "value": -0.5},
        {"system": "relationship", "target": "intimacy_gain_rate", "op": "mult", "value": 0.2},
        {"system": "stress", "target": "isolation_stress", "op": "set", "value": 0.0},
        {"system": "need", "target": "recognition", "op": "set", "value": {"decay_rate_mult": 0.1}},
    ],
    "A_confessor": [
        {"system": "relationship", "target": "secret_disclosure_chance", "op": "mult", "value": 2.0},
        {"system": "behavior", "target": "console_effectiveness", "op": "mult", "value": 2.0},
        {"system": "aura", "target": "stress_relief", "op": "set", "value": {"radius": 1, "intensity": 0.1, "target_filter": "interacting_with"}},
        {"system": "relationship", "target": "max_active_relationships", "op": "set", "value": 5},
        {"system": "stress", "target": "crowd_stress", "op": "set", "value": 0.1, "condition": "nearby_agents > 3"},
        {"system": "derived", "target": "trustworthiness", "op": "add", "value": 0.35},
    ],
    "A_tempest": [
        {"system": "derived", "target": "intimidation", "op": "add", "value": 0.4},
        {"system": "crafting", "target": "quality_bonus", "op": "add", "value": 1, "condition": "emotion.anger > 0.5 OR emotion.sadness > 0.5"},
        {"system": "behavior", "target": "argument_style", "op": "set", "value": "explosive"},
        {"system": "relationship", "target": "conflict_damage", "op": "mult", "value": 2.0},
        {"system": "emotion", "target": "anger", "op": "floor", "value": 0.2},
        {"system": "behavior", "target": "rational_decision", "op": "mult", "value": 0.3, "condition": "emotion.anger > 0.6"},
        {"system": "behavior", "target": "apologize", "op": "set", "value": {"weight_mult": 0.1}},
    ],
    "A_still_water": [
        {"system": "skill", "target": "mediation", "op": "mult", "value": 1.5},
        {"system": "emotion", "target": "anger", "op": "ceil", "value": 0.15, "condition": "trigger_type == 'insult' OR trigger_type == 'provocation'"},
        {"system": "aura", "target": "anger", "op": "set", "value": {"radius": 2, "intensity": -0.15, "target_filter": "all"}},
        {"system": "behavior", "target": "crisis_response_speed", "op": "mult", "value": 0.6},
        {"system": "emotion", "target": "intensity_mult", "op": "set", "value": 0.5},
        {"system": "combat", "target": "initial_panic_check", "op": "override", "value": "skip"},
    ],
    "A_war_drum": [
        {"system": "skill", "target": ["agitation", "propaganda"], "op": "mult", "value": 1.5},
        {"system": "aura", "target": "combat_morale", "op": "set", "value": {"radius": 5, "intensity": 0.2, "target_filter": "allies"}, "condition": "in_combat"},
        {"system": "skill", "target": ["peace_negotiation", "diplomacy"], "op": "mult", "value": 0.6},
        {"system": "relationship", "target": "hostility_escalation", "op": "mult", "value": 2.0},
        {"system": "behavior", "target": "pre_battle_speech", "op": "unlock", "value": {"morale_boost": 0.3, "duration": "combat"}},
        {"system": "stress", "target": "peace_boredom", "op": "set", "value": 0.05, "condition": "no_conflict_days > 30"},
    ],
    "A_hearth_keeper": [
        {"system": "aura", "target": "joy", "op": "set", "value": {"radius": 5, "intensity": 0.1, "target_filter": "same_settlement"}},
        {"system": "aura", "target": "conflict_suppression", "op": "set", "value": {"radius": 3, "intensity": 0.3}},
        {"system": "derived", "target": "intimidation", "op": "add", "value": -0.3},
        {"system": "behavior", "target": "harsh_decision", "op": "set", "value": {"weight_mult": 0.2}},
        {"system": "relationship", "target": "intimacy_decay_rate", "op": "mult", "value": 0.5},
        {"system": "behavior", "target": ["punish", "enforce_rule", "exile"], "op": "set", "value": {"weight_mult": 0.1}},
    ],
    "A_obsidian_edge": [
        {"system": "skill", "target": ["tactics", "strategy", "logistics"], "op": "mult", "value": 1.4},
        {"system": "combat", "target": "crit_chance", "op": "add", "value": 0.15},
        {"system": "behavior", "target": ["console", "empathize", "grant_leave"], "op": "remove", "value": True, "condition": "target.role == 'subordinate'"},
        {"system": "relationship", "target": "subordinate_loyalty_decay", "op": "mult", "value": 2.0},
        {"system": "behavior", "target": "punish_failure", "op": "inject", "value": {"priority": 0.9}, "condition": "subordinate_failed_task"},
        {"system": "emotion", "target": "expression_mult", "op": "set", "value": 0.1},
    ],
    "A_powder_keg": [
        {"system": "stress", "target": "mental_break_threshold", "op": "add", "value": -0.3},
        {"system": "stress", "target": "break_types", "op": "replace", "value": {"creative_frenzy": 0.3, "emotional_outburst": 0.3, "berserk": 0.2, "sobbing": 0.2}},
        {"system": "crafting", "target": "quality_bonus", "op": "add", "value": 3, "condition": "mental_state == 'creative_frenzy'"},
        {"system": "emotion", "target": "volatility", "op": "mult", "value": 3.0},
        {"system": "behavior", "target": "action_delay", "op": "set", "value": 0},
        {"system": "behavior", "target": "consequence_evaluation", "op": "mult", "value": 0.2},
        {"system": "behavior", "target": "promise_fulfillment", "op": "mult", "value": 0.4},
    ],
    "A_living_library": [
        {"system": "skill", "target": ["research", "mathematics", "astronomy", "law", "medicine"], "op": "mult", "value": 1.5},
        {"system": "event", "target": "cross_discipline_discovery", "op": "unlock", "value": True},
        {"system": "behavior", "target": "decision_speed", "op": "mult", "value": 0.6},
        {"system": "memory", "target": "knowledge_retention", "op": "mult", "value": 2.0},
        {"system": "behavior", "target": "plan_vs_act", "op": "set", "value": {"plan_weight": 0.8, "act_weight": 0.2}},
        {"system": "need", "target": "competence", "op": "set", "value": {"decay_rate_mult": 2.0}},
    ],
    "A_autumn_leaf": [
        {"system": "skill", "target": ["painting", "poetry", "music", "exploration"], "op": "mult", "value": 1.3},
        {"system": "stress", "target": "settlement_boredom", "op": "set", "value": 0.15, "condition": "same_settlement_years > 3"},
        {"system": "behavior", "target": "wander", "op": "inject", "value": {"priority": 0.6}, "condition": "same_settlement_years > 2"},
        {"system": "need", "target": "materialism", "op": "set", "value": {"satisfaction_mult": 0.2}},
        {"system": "emotion", "target": "joy", "op": "add", "value": 0.4, "condition": "arrived_new_settlement"},
        {"system": "relationship", "target": "long_distance_decay", "op": "mult", "value": 3.0},
    ],
    "A_iron_oath": [
        {"system": "behavior", "target": ["break_promise", "renegotiate", "abandon_task"], "op": "remove", "value": True},
        {"system": "stress", "target": "promise_failure", "op": "set", "value": 0.6, "condition": "promise_broken_by_external"},
        {"system": "stress", "target": "break_types", "op": "replace", "value": {"self_punishment": 0.4, "rage": 0.3, "catatonic": 0.3}, "condition": "trigger == 'promise_failure'"},
        {"system": "relationship", "target": "subordinate_loyalty", "op": "add", "value": 0.3},
        {"system": "relationship", "target": "oathbreaker_response", "op": "set", "value": "permanent_hostility"},
        {"system": "values", "target": "LAW", "op": "floor", "value": 0.7},
    ],
    "A_quicksilver": [
        {"system": "behavior", "target": ["promise_fulfillment", "rule_compliance", "duty_compliance"], "op": "mult", "value": 0.0},
        {"system": "behavior", "target": "faction_loyalty", "op": "set", "value": 0.0},
        {"system": "behavior", "target": "switch_faction", "op": "set", "value": {"weight_mult": 5.0}},
        {"system": "behavior", "target": "faction_entry_resistance", "op": "set", "value": 0.0},
        {"system": "stress", "target": "betrayal_guilt", "op": "immunity", "value": True},
        {"system": "relationship", "target": "trust", "op": "ceil", "value": 40},
        {"system": "behavior", "target": "danger_detection", "op": "mult", "value": 1.5},
    ],
    "A_dreamer": [
        {"system": "skill", "target": ["poetry", "painting", "music", "theology", "mythology"], "op": "mult", "value": 1.5},
        {"system": "skill", "target": ["farming", "construction", "mining", "logging"], "op": "mult", "value": 0.7},
        {"system": "behavior", "target": "danger_detection", "op": "mult", "value": 0.6},
        {"system": "event", "target": "dream_inspiration", "op": "unlock", "value": {"chance_per_night": 0.05}},
        {"system": "memory", "target": "distortion_rate", "op": "mult", "value": 2.0},
        {"system": "need", "target": "transcendence", "op": "set", "value": {"decay_rate_mult": 2.0}},
    ],
    "A_fortress_mind": [
        {"system": "stress", "target": ["propaganda_resistance", "brainwash_resistance"], "op": "set", "value": 1.0},
        {"system": "values", "target": "drift_rate", "op": "set", "value": 0.0},
        {"system": "behavior", "target": ["adopt_innovation", "learn_new_skill", "accept_change", "experiment"], "op": "remove", "value": True},
        {"system": "skill", "target": "existing_skills", "op": "mult", "value": 1.2},
        {"system": "relationship", "target": "stranger_trust", "op": "set", "value": -20},
        {"system": "emotion", "target": "volatility", "op": "mult", "value": 0.2},
    ],
    "A_zealot": [
        {"system": "skill", "target": ["proselytize", "persuasion"], "op": "mult", "value": 1.4},
        {"system": "behavior", "target": "persecute_heretic", "op": "inject", "value": {"priority": 0.8}, "condition": "target.values_diff > 0.5"},
        {"system": "behavior", "target": ["compromise", "concede", "tolerance"], "op": "remove", "value": True},
        {"system": "stress", "target": "source_immunity", "op": "set", "value": ["doubt", "heresy_exposure"]},
        {"system": "relationship", "target": "value_conflict_damage", "op": "mult", "value": 3.0},
        {"system": "relationship", "target": "value_alignment_bonus", "op": "mult", "value": 2.0},
        {"system": "event", "target": "religious_conflict_chance", "op": "mult", "value": 2.0},
    ],
    "A_fox_path": [
        {"system": "relationship", "target": "first_impression", "op": "add", "value": 30},
        {"system": "derived", "target": "charisma", "op": "add", "value": 0.2},
        {"system": "skill", "target": ["espionage", "conspiracy", "manipulation"], "op": "mult", "value": 1.4},
        {"system": "behavior", "target": "maintain_cover", "op": "unlock", "value": True},
        {"system": "relationship", "target": "betrayal_damage", "op": "mult", "value": 3.0},
        {"system": "behavior", "target": ["express_true_feelings", "genuine_apology"], "op": "remove", "value": True},
        {"system": "derived", "target": "wisdom", "op": "add", "value": -0.2},
    ],
    # ── Shadow (5 traits) ──
    "S_hollow_crown": [
        {"system": "emotion", "target": "guilt", "op": "set", "value": 0.0},
        {"system": "emotion", "target": ["trust", "sadness"], "op": "ceil", "value": 0.15},
        {"system": "emotion", "target": "contempt", "op": "floor", "value": 0.2},
        {"system": "stress", "target": "accumulation_rate", "op": "mult", "value": 0.3},
        {"system": "stress", "target": "source_immunity", "op": "set", "value": ["guilt", "grief", "social_rejection", "loneliness"]},
        {"system": "relationship", "target": "mode", "op": "set", "value": "instrumental"},
        {"system": "relationship", "target": "betrayal_cooldown", "op": "set", "value": 0},
        {"system": "relationship", "target": "betrayal_stress", "op": "set", "value": 0.0},
        {"system": "memory", "target": "kill_trauma", "op": "set", "value": False},
        {"system": "combat", "target": "kill_stress", "op": "set", "value": 0.0},
        {"system": "behavior", "target": ["manipulate", "exploit", "charm_offensive", "cold_calculation"], "op": "unlock", "value": True},
        {"system": "behavior", "target": ["genuine_empathy", "altruistic_help", "self_sacrifice", "console"], "op": "remove", "value": True},
        {"system": "derived", "target": "charisma", "op": "add", "value": 0.2},
        {"system": "derived", "target": "trustworthiness", "op": "add", "value": -0.3, "tags": ["long_term_decay"]},
    ],
    "S_puppet_master": [
        {"system": "skill", "target": ["conspiracy", "manipulation", "espionage"], "op": "mult", "value": 1.6},
        {"system": "behavior", "target": "recruit_minion", "op": "unlock", "value": {"loyalty_bonus": 0.4, "method": "calculated_charm"}},
        {"system": "relationship", "target": "mode", "op": "set", "value": "strategic"},
        {"system": "behavior", "target": "plausible_deniability", "op": "unlock", "value": {"detection_reduction": 0.8}},
        {"system": "behavior", "target": "delegate_dirty_work", "op": "inject", "value": {"priority": 0.9}},
        {"system": "behavior", "target": "planning_horizon", "op": "mult", "value": 3.0},
        {"system": "behavior", "target": "improvisation", "op": "mult", "value": 0.5},
    ],
    "S_mirror_throne": [
        {"system": "skill", "target": ["propaganda", "agitation", "leadership"], "op": "mult", "value": 1.5},
        {"system": "emotion", "target": "anger", "op": "trigger", "value": {"on": "receives_criticism", "intensity": 0.8}},
        {"system": "relationship", "target": "critic_response", "op": "set", "value": "permanent_hostility"},
        {"system": "need", "target": "recognition", "op": "set", "value": {"decay_rate_mult": 5.0}},
        {"system": "stress", "target": "break_types", "op": "replace", "value": {"narcissistic_rage": 0.5, "grandiose_speech": 0.3, "self_destruction": 0.2}, "condition": "need.recognition < 0.2"},
        {"system": "derived", "target": "charisma", "op": "add", "value": 0.3, "condition": "is_leader"},
        {"system": "stress", "target": "role_loss", "op": "set", "value": 0.8, "condition": "lost_leadership"},
    ],
    "S_cracked_mirror": [
        {"system": "behavior", "target": "sympathy_manipulation", "op": "unlock", "value": {"effectiveness": 0.6}},
        {"system": "need", "target": "recognition", "op": "set", "value": {"decay_rate_mult": 4.0}},
        {"system": "emotion", "target": "volatility", "op": "mult", "value": 3.0},
        {"system": "behavior", "target": "revenge", "op": "inject", "value": {"priority": 1.0}, "condition": "betrayed_by.intimacy_was > 50"},
        {"system": "behavior", "target": "revenge_proportionality", "op": "set", "value": 0.0},
        {"system": "stress", "target": "coping", "op": "set", "value": {"primary": "self_pity", "secondary": "blame_others"}},
        {"system": "stress", "target": "isolation_stress", "op": "mult", "value": 5.0},
    ],
    "S_red_smile": [
        {"system": "emotion", "target": "joy", "op": "add", "value": 0.3, "condition": "witnessed_suffering"},
        {"system": "skill", "target": ["interrogation", "torture"], "op": "mult", "value": 1.6},
        {"system": "combat", "target": "kill_morale_boost", "op": "set", "value": 0.2},
        {"system": "aura", "target": "fear", "op": "set", "value": {"radius": 3, "intensity": 0.25, "target_filter": "all"}},
        {"system": "skill", "target": ["animal_training", "veterinary"], "op": "mult", "value": 0.0},
        {"system": "behavior", "target": ["mercy", "spare_life", "forgive"], "op": "remove", "value": True},
        {"system": "stress", "target": "violence_withdrawal", "op": "set", "value": 0.1, "condition": "no_violence_days > 7"},
    ],
}

# ═══════════════════════════════════════════════════════════════
#  LOCALIZATION: (en_name, ko_name, en_desc, ko_desc)
# ═══════════════════════════════════════════════════════════════
L = {
    # ── Archetype: Single-axis ──
    "A_incorruptible": ("Incorruptible", "청렴결백",
        "Immune to bribes and temptation. Always deals at fair price. Cannot steal or cheat. Negotiation -20%.",
        "뇌물/유혹 면역, 항상 공정가 거래. 도둑질·사기 불가. 협상력 -20%."),
    "A_serpent_tongue": ("Serpent-Tongue", "사설",
        "Fraud, smuggling, bribery +40%. Betrayal cooldown halved. Reputation damage 2x when caught.",
        "사기·밀수·뇌물 +40%, 배신 쿨다운 절반. 발각 시 평판 피해 2배."),
    "A_glass_heart": ("Glass Heart", "유리심장",
        "Emotion intensity 2x. Art and music learning +30%. Stress gain 1.5x, combat morale -30%.",
        "감정 이벤트 강도 2배, 예술·음악 학습 +30%. 스트레스 1.5배, 전투 사기 -30%."),
    "A_stone_blood": ("Stone-Blood", "돌피",
        "Stress gain -80%, fear immune. Intimacy formation -50%, cannot comfort others.",
        "스트레스 축적 -80%, 공포 면역. 친밀도 형성 -50%, 위로 불가."),
    "A_bonfire": ("Bonfire", "화톳불",
        "Relationship formation 2x speed. Nearby joy +15%. Happiness drops sharply when alone.",
        "관계 형성 2배속, 주변 기쁨 +15%. 혼자이면 행복도 급락."),
    "A_deep_well": ("Deep Well", "깊은 우물",
        "Solo work efficiency +30%, self-reflection 2x. No social relations, leadership -50%.",
        "홀로 작업 효율 +30%, 자기성찰 2배속. 사교 시 관계 불가, 리더 -50%."),
    "A_open_palm": ("Open Palm", "열린 손바닥",
        "Conflict resolution +40%, alliance +30%. Exploitation vulnerability +50%, cannot refuse.",
        "갈등 해결 +40%, 동맹 유지 +30%. 착취 취약 +50%, 거절 불가."),
    "A_iron_wall": ("Iron Wall", "철벽",
        "Exploitation immune, intimidation resist +50%, debate win +30%. Cooperation reject +40%.",
        "착취 면역, 협박 저항 +50%, 논쟁 승률 +30%. 협력 거절 +40%."),
    "A_clockwork": ("Clockwork", "시계태엽",
        "Work efficiency +25%, quality +1 tier, rule compliance 100%. Improvisation -40%.",
        "모든 작업 효율 +25%, 품질 +1단계, 규칙 준수 100%. 즉흥 행동 -40%."),
    "A_wildfire": ("Wildfire", "들불",
        "Improvisation +40%, crisis response +30%. Long projects -50%, promises -40%.",
        "즉흥 행동 +40%, 위기 대응 +30%. 장기 프로젝트 -50%, 약속 이행 -40%."),
    "A_horizon_seeker": ("Horizon-Seeker", "수평선을 쫓는 자",
        "New skill learning +40%, invention 2x. Repetitive work happiness -30%.",
        "새 기술 학습 +40%, 발명 확률 2배. 반복 작업 행복도 -30%."),
    "A_root_bound": ("Root-Bound", "뿌리 박힌",
        "Traditional work +20%, cultural identity firm. New skills -50%, innovation blocked.",
        "전통 작업 +20%, 문화 정체성 견고. 새 기술 학습 -50%, 혁신 불가."),
    # ── Archetype: 2-axis ──
    "A_silver_mask": ("Silver Mask", "은빛 가면",
        "Social but all performance. First impression +50%. Long-term trust impossible. Spy aptitude.",
        "사교적이나 모두 연기. 첫인상 +50%, 장기 신뢰 불가. 스파이·외교관 적성."),
    "A_true_mirror": ("True Mirror", "진실의 거울",
        "Charisma +30%, cannot lie. Natural leader but diplomacy disadvantage.",
        "카리스마 +30%, 거짓말 불가(외교 불리). 자연스러운 지도자."),
    "A_phantom": ("Phantom", "유령",
        "Assassination and theft +50%. Nobody remembers. No leadership possible.",
        "암살·절도 +50%, 존재감 제로. 리더십 불가, 관계 형성 극난."),
    "A_confessor": ("Confessor", "고해사제",
        "Secret information +40%, counseling 2x effect. Very limited social range.",
        "비밀 정보 수집 +40%, 상담 효과 2배. 사교 범위 극히 제한."),
    "A_tempest": ("Tempest", "폭풍",
        "Intimidation +40%, artistic inspiration +30%. Relationship destruction 2x.",
        "논쟁 위압 +40%, 예술 영감 +30%. 관계 파괴 확률 2배."),
    "A_still_water": ("Still Water", "고요한 물",
        "Mediation +50%, unresponsive to insults. Slow crisis reaction.",
        "중재 성공률 +50%, 모욕에 무반응. 위기 시 반응 느림."),
    "A_war_drum": ("War Drum", "전쟁의 북",
        "Incitement +50%, unit morale +30%. Peace negotiation -40%, makes enemies.",
        "선동 +50%, 부대 사기 +30%. 평화 협상 -40%, 적 양산."),
    "A_hearth_keeper": ("Hearth-Keeper", "화덕지기",
        "Settlement happiness +10%, conflict prevention. Decisiveness as leader -30%.",
        "정착지 행복도 +10%, 갈등 예방. 리더 결단력 -30%."),
    "A_obsidian_edge": ("Obsidian Edge", "흑요석 날",
        "Strategy +40%, lethal precision. Zero empathy for subordinates.",
        "전략 +40%, 치명적 정확도. 부하 공감 0, 이탈률 증가."),
    "A_powder_keg": ("Powder Keg", "화약통",
        "Inspiration bursts producing random masterwork. Mental break chance 3x.",
        "영감 폭발(랜덤 걸작), 멘탈 브레이크 확률 3배."),
    "A_living_library": ("Living Library", "살아있는 도서관",
        "Research speed +50%, technology fusion possible. Analysis paralysis -20%.",
        "연구 속도 +50%, 기술 융합 가능. 분석 마비 -20%."),
    "A_autumn_leaf": ("Autumn Leaf", "가을 낙엽",
        "Art and discovery +30%. Cannot settle; happiness -40% after 3 years in place.",
        "예술·발견 +30%. 정착 불가(3년 이상 체류 시 행복도 -40%)."),
    "A_iron_oath": ("Iron Oath", "철의 맹세",
        "Contract fulfillment 100%, subordinate loyalty +30%. Cannot adapt to change.",
        "계약 이행 100%, 부하 충성 +30%. 상황 변해도 번복 불가."),
    "A_quicksilver": ("Quicksilver", "수은",
        "Opportunism +50%, free to join or leave any faction. Nobody trusts.",
        "기회주의 +50%, 진영 합류/이탈 자유. 누구도 믿지 않음."),
    "A_dreamer": ("Dreamer", "몽상가",
        "Art, religion, myth creation +50%. Work efficiency -30%, danger awareness -40%.",
        "예술·종교·신화 창작 +50%. 현실 업무 -30%, 위험 인지 -40%."),
    "A_fortress_mind": ("Fortress Mind", "요새 정신",
        "Brainwashing and incitement immune, cultural assimilation resist. Rejects everything new.",
        "세뇌·선동 면역, 문화 동화 저항. 새로운 것 일체 거부."),
    "A_zealot": ("Zealot", "광신도",
        "Proselytizing +40%, heresy persecution. No compromise, triggers religious conflict.",
        "포교·설득 +40%, 이단 박해. 타협 불가, 종교 갈등 촉발."),
    "A_fox_path": ("Fox Path", "여우길",
        "Double agent aptitude, long-term conspiracy +40%. Smiles while planning betrayal.",
        "이중 첩자 적성, 장기 음모 +40%. 미소 뒤의 칼."),
    "A_mountain_blood": ("Mountain Blood", "산의 피",
        "Administration optimal, discipline enforcement +40%. No mercy, resentment builds.",
        "행정·관료 최적, 규율 집행 +40%. 인정사정 없음, 반감 축적."),
    "A_spring_wind": ("Spring Wind", "봄바람",
        "Child raising +30%, comfort 2x effect. Avoids responsibility, unfit to lead.",
        "아이 양육 +30%, 위로 효과 2배. 책임 회피, 리더 부적합."),
    "A_torch_bearer": ("Torch-Bearer", "횃불 든 자",
        "New idea spread +50%, follower recruitment 2x. Always clashes with existing order.",
        "새 아이디어 전파 +50%, 추종자 모집 2배. 기존 질서와 항상 충돌."),
    "A_buried_stone": ("Buried Stone", "묻힌 돌",
        "Hermit self-sufficiency +40%. Zero social presence, no community contribution.",
        "은둔자 자급자족 +40%. 사회적 존재감 0, 공동체 기여 불가."),
    # ── Archetype: 3-axis ──
    "A_gilded_tongue": ("Gilded Tongue", "금빛 혀",
        "Persuasion, diplomacy, fraud all +35%. Even they cannot tell if they are sincere.",
        "설득·외교·사기 모두 +35%. 진심인지 연기인지 본인도 모름."),
    "A_silent_judge": ("Silent Judge", "침묵의 심판관",
        "Corruption detection +50%, judicial efficiency top tier. Has no friends.",
        "부정 적발 +50%, 사법 효율 최상. 친구가 없음."),
    "A_embers": ("Dying Embers", "꺼져가는 불씨",
        "Poetry and literature creation +40%. Chronic depression tendency.",
        "시·문학 창작 +40%, 만성 우울 경향."),
    "A_dawn_hammer": ("Dawn Hammer", "새벽의 망치",
        "Settlement productivity +30%, forced obedience. Seeds of rebellion grow.",
        "정착지 생산성 +30%, 복종 강제. 반란 씨앗."),
    "A_woven_fate": ("Woven Fate", "엮인 운명",
        "Healing and counseling +50%. Absorbs others' trauma, burnout risk 3x.",
        "치유·상담 +50%. 타인 트라우마 전이, 번아웃 3배."),
    "A_tidal_mind": ("Tidal Mind", "조석의 정신",
        "Invention chance 3x. Masterwork or catastrophe, extreme variance. Unpredictable.",
        "발명 확률 3배, 걸작 or 대참사(결과 분산 극대). 예측 불가."),
    "A_unmoving_peak": ("Unmoving Peak", "부동봉",
        "Administration efficiency top +40%. Ignores all emotions and relationships.",
        "행정 효율 최상(+40%), 감정·인간관계 일체 무시. 공포의 관료."),
    "A_wandering_star": ("Wandering Star", "떠도는 별",
        "Exploration and trade +40%. Cannot settle anywhere. Legendary storyteller.",
        "탐험·교역 +40%, 어디에도 정착 못함. 전설적 이야기꾼."),
    # ── Archetype: Value+Personality ──
    "A_blood_price": ("Blood Price", "피의 대가",
        "Combat proficiency +30%, loot +50%. Rejects peace, prefers war.",
        "전투 숙련도 +30%, 전리품 +50%. 평화 협정 거부, 전쟁 선호."),
    "A_golden_scale": ("Golden Scale", "황금 저울",
        "Trade profit +30%, fair trade reputation. Cannot make losing deals.",
        "거래 이윤 +30%, 공정 거래 명성. 손해 보는 거래 절대 불가."),
    "A_green_covenant": ("Green Covenant", "녹색 서약",
        "Gathering and farming +30%, animal affinity. Refuses nature destruction.",
        "채집·농업 +30%, 동물 친화. 자연 파괴 행위(벌목 등) 거부."),
    "A_eternal_student": ("Eternal Student", "영원한 학도",
        "All learning +25%, teaching 2x effect. Practical application -30%.",
        "모든 학습 +25%, 교육 효과 2배. 실전 적용 -30%(이론만)."),
    "A_broken_chain": ("Broken Chain", "끊어진 사슬",
        "Obedience immune, freedom fighter leader aptitude. Cannot stay in any organization.",
        "복종 면역, 자유 투쟁 리더 적성. 어떤 조직에도 오래 못 있음."),
    "A_ancestor_voice": ("Ancestor's Voice", "선조의 목소리",
        "Traditional skills and rituals +30%, cultural preservation. Rejects all innovation.",
        "전통 기술·의식 효율 +30%, 문화 보존. 모든 혁신 거부."),
    "A_last_stand": ("Last Stand", "최후의 보루",
        "Combat power 2x when allies endangered, morale immune. Self-preservation -50%.",
        "아군 위기 시 전투력 2배, 사기 붕괴 면역. 자기 보존 -50%."),
    "A_merrymaker": ("Merrymaker", "흥을 돋우는 자",
        "Festival effect 2x, nearby happiness +15%. Inappropriate in crises, response -30%.",
        "축제 효과 2배, 주변 행복 +15%. 위기 대응 -30%."),
    "A_veiled_blade": ("Veiled Blade", "칼을 숨긴 자",
        "Conspiracy and assassination +40%, double identity. Instant exile if caught.",
        "음모·암살 +40%, 이중 정체 유지. 발각 시 즉각 추방/처형."),
    "A_scales_keeper": ("Scales-Keeper", "저울의 수호자",
        "Judicial fairness 100%, resists injustice. No exceptions allowed, no mercy.",
        "재판 공정성 100%, 불의에 대한 저항. 예외 허용 불가, 자비 없음."),
    "A_craft_soul": ("Craft-Soul", "장인의 혼",
        "Crafting quality +2 tiers, masterwork 3x chance. Rejects mass production, speed -40%.",
        "제작 품질 +2단계, 걸작 확률 3배. 대량생산 거부, 속도 -40%."),
    "A_lotus_eater": ("Lotus Eater", "연꽃을 먹는 자",
        "Leisure happiness recovery 2x, art appreciation bonus. Labor efficiency -40%.",
        "여가 시 행복 회복 2배, 예술 감상 보너스. 노동 효율 -40%."),
    "A_rival_seeker": ("Rival-Seeker", "라이벌을 찾는 자",
        "Ability +30% when competing, skill growth accelerated. Cooperation -30%, rage on defeat.",
        "경쟁 시 능력 +30%, 스킬 성장 가속. 협력 -30%, 패배 시 분노."),
    # ── Shadow ──
    "S_hollow_crown": ("Hollow Crown", "텅 빈 왕관",
        "Primary psychopath. Zero empathy, zero guilt. Charming predator. Tyrannical but efficient.",
        "1차 사이코패스. 공감 0, 죄책감 0, 매력적인 포식자. 학정하나 효율적."),
    "S_puppet_master": ("Puppet Master", "인형사",
        "Long-term conspiracy +60%, loyal minion recruitment. All relationships are chess pieces.",
        "장기 음모 +60%, 충성 하수인 모집. 모든 관계가 체스 말."),
    "S_mirror_throne": ("Mirror Throne", "거울의 옥좌",
        "Grandiose narcissist. Follower recruitment +50%. Rage on criticism. Self-destructs on failure.",
        "과시형 자기도취. 추종자 +50%, 비판에 분노 폭발. 실패 시 자멸."),
    "S_cracked_mirror": ("Cracked Mirror", "금 간 거울",
        "Vulnerable narcissist. Manipulates through pity. Endless need for validation.",
        "취약형 자기도취. 동정으로 조종, 끝없는 인정 갈구. 배신 시 파괴적 복수."),
    "S_red_smile": ("Red Smile", "붉은 미소",
        "Sadist. Pleasure from pain. Interrogation +60%, personal combat morale +30%. Spreads fear.",
        "사디스트. 타인 고통에서 쾌감. 심문·고문 +60%, 전투 사기 +30%."),
    "S_honeyed_venom": ("Honeyed Venom", "꿀바른 독",
        "Con artist. Perfect trust performance. Long-term fraud +60%. Best colleague until caught.",
        "사기꾼. 완벽한 신뢰 연기, 장기 사기 +60%. 발각 전까지 최고의 동료."),
    "S_throne_hunger": ("Throne Hunger", "옥좌에 대한 갈증",
        "Tyrant. Power seizure +50%, loyalty enforcement. Always breeds resistance eventually.",
        "폭군. 권력 장악 +50%, 충성 강요. 반드시 저항 세력 생성."),
    "S_ash_prophet": ("Ash Prophet", "재의 예언자",
        "Cult leader. Religion creation +60%, fanatic followers. Settlement division risk.",
        "사이비 교주. 종교/사상 창시 +60%, 광적 추종자. 정착지 분열."),
    "S_cold_harvest": ("Cold Harvest", "차가운 수확",
        "Systematic exploiter. Emotionless extraction +40%. Causes population decline.",
        "체계적 약탈자. 감정 없는 착취, 자원 추출 +40%. 인구 감소 유발."),
    "S_jealous_flame": ("Jealous Flame", "질투의 화염",
        "Paranoid lover. Partner monopoly. Betrayal detection +50%. Destroys relationships.",
        "편집증적 연인. 파트너 독점, 배신 탐지 +50%. 의심으로 관계 파괴."),
    "S_carrion_comfort": ("Carrion Comfort", "썩은 위안",
        "Complete apathy. Doesn't care about self or others. Neglect destroys surroundings.",
        "완전한 무관심. 자기조차 안 돌봄, 위협 못 느낌. 방치 = 주변 파괴."),
    "S_web_weaver": ("Web-Weaver", "거미줄의 직공",
        "Social parasite. All-faction connections. Information monopoly +50%, provokes conflict.",
        "사회 기생자. 모든 파벌에 줄. 정보 독점 +50%, 분쟁 조장."),
    "S_night_bloom": ("Night Bloom", "밤에 피는 꽃",
        "Predatory charmer. Seduction +60%. Every meeting is a hunt. Exploits then abandons.",
        "매력적 포식자. 유혹 +60%, 모든 만남이 사냥. 파트너 착취 후 버림."),
    "S_broken_compass": ("Broken Compass", "부러진 나침반",
        "Antisocial. Ignores all rules and duties. Crime tendency +50%, wandering.",
        "반사회적 자유인. 모든 규칙·약속·의무 무시. 범죄 성향 +50%."),
    "S_iron_cradle": ("Iron Cradle", "철의 요람",
        "Cruel perfectionist. Punishes substandard work. Quality +2 but subordinate stress 3x.",
        "잔혹한 완벽주의자. 기준 미달은 처벌. 품질 +2, 부하 스트레스 3배."),
    # ── Radiance ──
    "R_golden_heart": ("Golden Heart", "금빛 심장",
        "Saint. Unlimited self-sacrifice, nearby happiness +20%, healing 2x. Burnout death risk.",
        "성인. 자기희생 무제한, 주변 행복 +20%, 치유 2배. 번아웃·과로사 위험."),
    "R_north_star": ("North Star", "북극성",
        "Ideal leader. Governance +40%, loyalty +50%. Stress 3x when failing own standards.",
        "이상적 지도자. 통치 +40%, 부하 충성 +50%. 자기 기준 미달 시 스트레스 3배."),
    "R_unbreaking": ("Unbreaking", "부서지지 않는",
        "Indomitable guardian. All abilities +30% in crisis, morale immune. Ordinary in peace.",
        "불굴의 수호자. 위기 시 전 능력 +30%, 사기 면역. 평시엔 평범."),
    "R_world_bridge": ("World Bridge", "세계의 다리",
        "Cultural mediator. Cross-cultural +50%, prejudice immune. Stranger everywhere.",
        "문화 중재자. 이문화 소통 +50%, 편견 면역. 어디서도 이방인."),
    "R_seed_keeper": ("Seed-Keeper", "씨앗 지기",
        "Civilization preserver. Technology preservation 100%, teaching 2x. Preservation first.",
        "문명의 보존자. 기술 보존 100%, 교육 2배. 혁신보다 보존 우선."),
    "R_hearthfire": ("Hearthfire", "난롯불",
        "Community parent. Child growth +30%, cohesion +20%. May exclude outsiders.",
        "공동체의 부모. 아이 성장 +30%, 결속 +20%. 외부인에게 배타적 가능."),
    "R_calm_harbor": ("Calm Harbor", "고요한 항구",
        "Crisis anchor. Calms nearby panic, blocks stress spread. Cannot share emotions.",
        "위기의 닻. 주변 패닉 진정, 스트레스 전파 차단. 감정 공유 불가."),
    "R_dawn_singer": ("Dawn Singer", "새벽을 노래하는 자",
        "Inspiration source. Art masterwork 5x, can found culture. Reality adaptation -40%.",
        "영감의 원천. 예술 걸작 5배, 문화 창시 가능. 현실 적응 -40%."),
    "R_iron_promise": ("Iron Promise", "철의 약속",
        "Absolute loyalist. Unconditional obedience to sworn lord, even if tyrant.",
        "절대 충성자. 맹세한 주군에 무조건 복종. 주군이 폭군이어도 따름."),
    "R_gentle_thunder": ("Gentle Thunder", "온화한 천둥",
        "Benevolent authority. Intimidation +30% AND likability +30% simultaneously.",
        "자비로운 강자. 위압감 +30% AND 호감 +30%. 결단 시 모두가 따름."),
    "R_wellspring": ("Wellspring", "샘물",
        "Infinite empathy. Trauma healing +50%. Accumulates others' trauma, mental health risk.",
        "무한 공감. 타인 트라우마 치유 +50%. 본인 트라우마 축적, 정신 위험."),
    "R_first_fire": ("First Fire", "최초의 불꽃",
        "Pure innovator. Invention 3x, genuine discovery. Always treated as heretic.",
        "순수한 혁신가. 발명 3배, 진정한 발견. 기존 체제에서 항상 이단 취급."),
    # ── Corpus ──
    "B_titan": ("Titan", "거인",
        "Intimidation +50%, combat damage +40%, construction +30%. Stealth impossible, food 2x.",
        "위압감 +50%, 전투 데미지 +40%, 건설 +30%. 은밀 행동 불가, 음식 소비 2배."),
    "B_wraith": ("Wraith", "망령",
        "Movement +40%, evasion +50%, assassination aptitude. Stamina -30%, weak in prolonged fights.",
        "이동속도 +40%, 회피 +50%, 암살 적성. 체력 -30%, 장기전 불리."),
    "B_mountain_body": ("Mountain Body", "산 같은 몸",
        "Injury severity -50%, labor endurance 2x. Agility -20%, precision work disadvantage.",
        "부상 심각도 -50%, 노동 지속 2배. 민첩 -20%, 정밀 작업 불리."),
    "B_phoenix_blood": ("Phoenix Blood", "불사조의 피",
        "Healing speed 3x, epidemic immunity. High pain sensitivity.",
        "치유 속도 3배, 전염병 면역. 고통 감수성 높음."),
    "B_glass_cannon": ("Glass Cannon", "유리대포",
        "Maximum attack power, minimum defense. One-hit kill or one-hit down.",
        "공격력 최상, 방어력 최하. 한 방 제압 or 한 방 쓰러짐."),
    "B_iron_lungs": ("Iron Lungs", "철의 폐",
        "Fatigue accumulation -60%. Optimal for long-distance travel and pursuit.",
        "피로 축적 -60%. 장거리 이동/추격 최적."),
    "B_paper_skin": ("Paper Skin", "종이 피부",
        "All damage 2x, disease vulnerable. Danger avoidance instinct +30%.",
        "모든 피해 2배, 질병 취약. 위험 회피 생존 본능 +30%."),
    "B_perfect_form": ("Perfect Form", "완벽한 신체",
        "First impression +40%, social bonus +25%. Target of envy and danger.",
        "첫인상 +40%, 사교 보너스 +25%. 시기 대상, 표적 위험."),
    "B_withered": ("Withered", "시든",
        "Physical labor and combat impossible. Intellectual focus learning +20%.",
        "육체 노동/전투 불가. 지적 활동 집중 학습 +20%."),
    "B_cat_eyes": ("Cat Eyes", "고양이 눈",
        "Night activity, ambush detection +40%. Vulnerable to bright light.",
        "야간 활동, 매복 탐지 +40%. 강한 빛에 약함."),
    "B_slow_healer": ("Slow Healer", "더딘 치유",
        "Healing speed 1/3. Extreme medical dependency. Cautiousness +20%.",
        "치유 속도 1/3. 의료 의존도 극대. 조심성 +20%."),
    "B_undying": ("Undying", "죽지 않는",
        "Lethal wound survival +50%. Risk of becoming reckless.",
        "치명상 생존 +50%. 무모해질 위험."),
    # ── Nous ──
    "N_polymath": ("Polymath", "박학다식",
        "All knowledge skill learning +40%. Expertise ceiling -30% (jack of all trades).",
        "모든 지식 기술 학습 +40%. 전문성 한계(대가 레벨 -30%)."),
    "N_silver_voice": ("Silver Voice", "은빛 목소리",
        "Persuasion, diplomacy, teaching +50%. Risk of manipulation.",
        "설득·외교·교육 +50%. 조종 위험."),
    "N_architects_eye": ("Architect's Eye", "건축가의 눈",
        "Construction quality +2 tiers, fortress design optimal. Dissatisfied with others' work.",
        "건설 품질 +2단계, 요새 설계 최상. 타인 작업에 불만."),
    "N_beast_tongue": ("Beast Tongue", "짐승의 말",
        "Animal training +50%, wilderness survival +40%. Human social adaptation -20%.",
        "동물 조련 +50%, 야생 생존 +40%. 인간 사회 적응 -20%."),
    "N_war_savant": ("War Savant", "전쟁의 천재",
        "Combat learning 3x speed, tactics +40%. Restless in peace, violence tendency.",
        "전투 학습 3배속, 전술 +40%. 평화 시 불안, 폭력 성향."),
    "N_inner_eye": ("Inner Eye", "내면의 눈",
        "Stress recovery 2x, emotion regulation optimal. Empathy is separate.",
        "스트레스 회복 2배, 감정 조절 최상. 공감은 별개."),
    "N_natures_child": ("Nature's Child", "자연의 아이",
        "Weather prediction, gathering +40%, farming +30%. Unfit for urban life.",
        "날씨 예측, 채집 +40%, 농업 +30%. 도시 생활 부적응."),
    "N_muse_touched": ("Muse-Touched", "뮤즈에 닿은",
        "Art quality top tier, frequent inspiration. Reality focus -30%.",
        "예술 품질 최상, 영감 빈번. 현실 집중 -30%."),
    "N_dim": ("Dim", "어둑한",
        "Learning -50%, complex skills impossible. Simple labor persistence +20%.",
        "학습 -50%, 복잡한 기술 불가. 단순 노동 끈기 +20%."),
    "N_feral_mind": ("Feral Mind", "야수의 정신",
        "Physical skills +40%, instinctive combat. Language, social, organization extremely difficult.",
        "신체 기술 +40%, 본능적 전투. 언어·사교·조직 극난."),
    # ── Awakened ──
    "W_scarred_soul": ("Scarred Soul", "상처 입은 영혼",
        "Base stress +20%, fear resistance +30%. Deep relationships difficult, nightmares.",
        "스트레스 기본 +20%, 공포 저항 +30%. 깊은 관계 어려움, 악몽."),
    "W_battle_forged": ("Battle-Forged", "전장에서 벼려진",
        "Panic immune, morale +20%, ignores injuries. Hypervigilance in peacetime.",
        "패닉 면역, 사기 +20%, 부상 무시. 평시 과경계."),
    "W_widows_frost": ("Widow's Frost", "미망인의 서리",
        "Emotional dulling, new relationships -50%. Lonely resilience, comforting others +30%.",
        "감정 둔화, 새 관계 -50%. 고독한 강인함, 타인 위로 +30%."),
    "W_twice_born": ("Twice-Born", "두 번 태어난",
        "Death fear immune, risk-taking +30%. Meaning-seeking urgency surges.",
        "죽음 공포 면역, 위험 감수 +30%. 의미 욕구 급등."),
    "W_oath_breaker": ("Oath-Breaker", "맹세를 깨뜨린",
        "Trust -30%, freedom +20%. Guilt or liberation depending on personality.",
        "신뢰도 -30%, 자유도 +20%. 죄책감 or 해방감(성격별)."),
    "W_kinslayer": ("Kinslayer", "동족을 벤",
        "Social stigma -50%, fear aura. Revered in some cultures.",
        "사회적 낙인(-50%), 공포 아우라. 일부 문화에서 경외."),
    "W_exile_risen": ("Exile-Risen", "추방에서 일어선",
        "Adaptability +40%, self-reliance +30%. Vengefulness or transcendence.",
        "적응력 +40%, 자립심 +30%. 복수심 or 초월."),
    "W_first_kill": ("First Kill", "첫 번째 죽임",
        "Personality fork: guilt (stress) or awakening (combat +20%).",
        "성격 분기: 죄책감(스트레스) or 각성(전투 +20%)."),
    "W_old_wolf": ("Old Wolf", "늙은 늑대",
        "Wisdom +30%, combat experience offsets physical decline. Successor training 2x.",
        "지혜 +30%, 전투 경험 유지(체력하락 상쇄). 후계자 양성 2배."),
    "W_broken_faith": ("Broken Faith", "부서진 믿음",
        "Previous faith removed, skepticism. Can found new religion.",
        "기존 신앙 제거, 회의주의. 새 종교 창시 가능."),
    "W_touched_by_gods": ("Touched by Gods", "신에게 닿은",
        "Faith maximized, prophet potential. Revered or deemed insane.",
        "신앙심 극대, 예언자 가능. 경외 or 광인 취급."),
    "W_famine_survivor": ("Famine Survivor", "기근 생존자",
        "Food hoarding compulsion, food management +30%. Sharing difficulty.",
        "음식 비축 강박, 식량 관리 +30%. 나눠주기 어려움."),
    "W_plague_walker": ("Plague-Walker", "역병을 걸은 자",
        "Immune to that disease, medicine +20%. Loneliness resistance.",
        "해당 질병 면역, 의학 +20%. 고독 저항."),
    "W_crown_weight": ("Weight of the Crown", "왕관의 무게",
        "Governance experience +30%, charisma +15%. Loneliness, paranoia, stress.",
        "통치 경험 +30%, 카리스마 +15%. 고독, 편집증, 스트레스."),
    "W_mothers_fury": ("Mother's/Father's Fury", "부모의 분노",
        "Combat 2x for endangered children. Loss of reason.",
        "자녀 관련 전투 시 2배, 이성 상실."),
    "W_dreaming_prophet": ("Dreaming Prophet", "꿈꾸는 예언자",
        "Prophecy events, follower recruitment. Reality/fantasy distinction blurs.",
        "예언 이벤트, 추종자 모집. 현실/환상 구분 어려움."),
    "W_chain_breaker": ("Chain-Breaker", "사슬을 끊는 자",
        "Freedom struggle leader, charisma +20%. Distrusts all leaders.",
        "자유 투쟁 리더, 카리스마 +20%. 모든 지도자 불신."),
    "W_wanderers_return": ("Wanderer's Return", "방랑자의 귀환",
        "Multicultural knowledge, trade +30%, multilingual. Never fully belongs.",
        "다문화 지식, 교역 +30%, 다언어. 완전한 소속감은 없음."),
    # ── Bloodline: Positive ──
    "L_giants_marrow": ("Giant's Marrow", "거인의 골수",
        "Height +15%, strength +20%, intimidation +25%. Food 1.5x, lifespan -5%.",
        "키 +15%, 근력 +20%, 위압감 +25%. 식량 1.5배, 수명 -5%."),
    "L_hawks_gaze": ("Hawk's Gaze", "매의 시선",
        "Vision +40%, hunting and scouting +30%, archery +20%. Close-range precision -15%.",
        "시야 +40%, 사냥·정찰 +30%, 궁술 +20%. 근거리 정밀 -15%."),
    "L_winter_blood": ("Winter Blood", "겨울의 피",
        "Cold resistance +50%, endurance +20%. Heat -30%.",
        "추위 저항 +50%, 지구력 +20%. 더위 -30%."),
    "L_summer_veins": ("Summer Veins", "여름의 핏줄",
        "Heat resistance +50%, agility +15%. Cold -30%.",
        "더위 저항 +50%, 민첩 +15%. 추위 -30%."),
    "L_iron_liver": ("Iron Liver", "철의 간",
        "Poison and alcohol resistance +60%, immunity +20%. Taste -10%.",
        "독·알코올 저항 +60%, 면역 +20%. 미각 둔감(-10%)."),
    "L_mothers_intuition": ("Mother's Intuition", "어머니의 직감",
        "Danger premonition +40%, nurturing +25%, interpersonal +15%. Logic -10%.",
        "위험 예감 +40%, 양육 +25%, 대인지능 +15%. 논리 -10%."),
    "L_war_seed": ("War Seed", "전쟁의 씨앗",
        "Combat learning 2x, adrenaline +25%. Peaceful occupation aptitude -20%.",
        "전투 학습 2배, 아드레날린 +25%. 평화 직업 적성 -20%."),
    "L_silver_tongue_blood": ("Silver-Tongue Blood", "은설의 혈통",
        "Linguistic +20%, persuasion +30%. Cannot endure silence (stress when alone).",
        "언어 +20%, 설득 +30%. 침묵 못 견딤(혼자=스트레스)."),
    "L_deep_roots": ("Deep Roots", "깊은 뿌리",
        "Lifespan +15%, recovery +25%, aging slowdown. Cannot migrate.",
        "수명 +15%, 회복 +25%, 노화 감속. 이동/이주 불가."),
    "L_starlit_mind": ("Starlit Mind", "별빛의 정신",
        "All intelligence +10%, learning +20%. Information overload mental break risk.",
        "모든 지능 +10%, 학습 +20%. 정보 과부하 시 멘탈 브레이크."),
    "L_beast_affinity": ("Beast Affinity", "짐승 친화",
        "Animal training +50%, beast attack -40%. Human social -15%.",
        "동물 조련 +50%, 야수 공격 -40%. 인간 사교 -15%."),
    "L_stone_bones": ("Stone Bones", "돌의 뼈",
        "Toughness +25%, fracture -60%. Agility -15%, cannot swim.",
        "인성 +25%, 골절 -60%. 민첩 -15%, 수영 불가."),
    "L_dawn_blessed": ("Dawn-Blessed", "새벽의 축복",
        "Charisma +20%, morale +10%, healing received +25%. Night -15%.",
        "카리스마 +20%, 사기 +10%, 치유 속도 +25%. 밤에 -15%."),
    # ── Bloodline: Negative ──
    "L_thin_blood": ("Thin Blood", "엷은 피",
        "Bleeding lethality 2x, recovery -30%. Danger avoidance +20%.",
        "출혈 치명도 2배, 회복 -30%. 위험 회피 +20%."),
    "L_moon_sickness": ("Moon-Sickness", "달의 병",
        "Periodic mental instability (emotion swing 2x). Art inspiration +30%.",
        "주기적 정신 불안정(감정 변동 2배). 예술 영감 +30%."),
    "L_hollow_bones": ("Hollow Bones", "텅 빈 뼈",
        "Fracture 3x, combat unfit. Agility +20%, running +15%.",
        "골절 3배, 전투 부적합. 민첩 +20%, 달리기 +15%."),
    "L_blood_fury": ("Blood Fury", "피의 광기",
        "Combat berserk (strength +40%, ally identification -30%). Peacetime anger builds.",
        "전투 광폭화(근력 +40%, 아군 식별 -30%). 평시 분노 축적."),
    "L_cursed_womb": ("Cursed Womb", "저주받은 자궁",
        "Birth mortality 3x, infertility +40%. Surviving children all abilities +10%.",
        "출산 사망률 3배, 불임 +40%. 생존 자녀 모든 능력 +10%."),
    "L_short_wick": ("Short Wick", "짧은 심지",
        "Lifespan -20%, accelerated aging. Youth: all abilities +15%.",
        "수명 -20%, 노화 가속. 젊은 시절 모든 능력 +15%."),
    "L_wandering_mind": ("Wandering Mind", "떠도는 정신",
        "Focus -30%, task abandonment. Creativity +25%, unexpected discoveries +20%.",
        "집중 -30%, 작업 이탈. 창의 +25%, 예상 밖 발견 +20%."),
    # ── Bloodline: Neutral ──
    "L_twin_souled": ("Twin-Souled", "쌍둥이 영혼",
        "Twin birth 5x. Extreme bond between twins; severe trauma if one dies.",
        "쌍둥이 출산 5배. 쌍둥이 간 극유대, 한 명 사망 시 severe trauma."),
    "L_old_blood": ("Old Blood", "오래된 피",
        "Bloodline marker. Same carriers +20 relations, other bloodlines -10.",
        "가문 표식. 같은 보유자끼리 +20, 타 가문 -10."),
    "L_echo_face": ("Echo Face", "메아리 얼굴",
        "Ancestral resemblance. Family +15%, easier family recognition.",
        "조상 닮은 외모. 가문 내 +15%, 가문 인식 용이."),
    "L_fey_touched": ("Fey-Touched", "요정에게 닿은",
        "Openness fixed ≥0.8, art +30%. Reality sense -20%.",
        "개방성 고정 ≥0.8, 예술 +30%, 현실감 -20%."),
    "L_ember_heart": ("Ember Heart", "잔불의 심장",
        "Crisis activation (+30%), peacetime lethargy (-15%). Extreme situations only.",
        "위기 시 각성(+30%), 평시 무기력(-15%). 극한 전용."),
    # ── Mastery ──
    "M_anvils_echo": ("Anvil's Echo", "모루의 메아리",
        "Metal quality +3, unique weapon crafting. Cannot acknowledge other smiths.",
        "금속 품질 +3, 고유 무기 제작. 타 대장장이 인정 못함."),
    "M_green_thumb": ("Verdant Touch", "푸른 손길",
        "Harvest +50%, crop failure resist, new crop discovery. War witness = severe stress.",
        "수확 +50%, 흉작 저항, 신작물 발견. 전쟁 목격 severe stress."),
    "M_death_dealer": ("Death Dealer", "죽음의 거래인",
        "Critical hit +30%, enemy morale destruction aura. Killing desensitization.",
        "치명타 +30%, 적 사기 파괴 아우라. 살인 무감각."),
    "M_tongue_of_ages": ("Tongue of Ages", "시대의 혀",
        "Diplomacy +50%, can prevent wars. Loses sense of own sincerity.",
        "외교 +50%, 전쟁 회피 가능. 본인 진심 잊어감."),
    "M_bone_setter": ("Bone-Setter", "뼈를 맞추는 자",
        "Treatment +40%, permanent disability recovery (low chance). Pain desensitization.",
        "치료 +40%, 영구장애 회복(저확률). 고통 둔감."),
    "M_wall_maker": ("Wall-Maker", "벽을 세우는 자",
        "Building durability 2x, fortress design. Ugly building destruction urge.",
        "건물 내구 2배, 요새 설계. 못생긴 건물 파괴 충동."),
    "M_thread_weaver": ("Thread-Weaver", "실을 엮는 자",
        "Textile top quality, special fabrics possible. Refuses rush orders.",
        "직물 최상, 특수 직물 가능. 급한 주문 거부."),
    "M_song_keeper": ("Song-Keeper", "노래 지키는 자",
        "Oral history preservation, festival 2x, culture spread +40%. Silence anxiety.",
        "구전 역사 보존, 축제 2배, 문화 전파 +40%. 침묵 불안."),
    "M_shadow_step": ("Shadow Step", "그림자 걸음",
        "Infiltration and assassination +50%, detection -60%, intel 2x. Daylight anxiety.",
        "잠입·암살 +50%, 발각 -60%, 정보 2배. 대낮 불안."),
    "M_kings_hand": ("King's Hand", "왕의 손",
        "Settlement efficiency +30%, waste -40%. Micromanagement compulsion.",
        "정착지 효율 +30%, 낭비 -40%. 미시관리 강박."),
    "M_fire_tamer": ("Fire-Tamer", "불을 길들이는 자",
        "Food/drink top quality, banquet 2x. Criticizes others' cooking.",
        "음식/주류 최상, 연회 2배. 타인 요리 비판."),
    "M_star_reader": ("Star-Reader", "별을 읽는 자",
        "Weather prediction +50%, navigation +40%. Observation compulsion (insomnia).",
        "날씨 예측 +50%, 항해 +40%. 관측 강박(불면)."),
    "M_horse_whisperer": ("Horse-Whisperer", "말에게 속삭이는 자",
        "Mounted combat +40%, taming ~100%. Animal slaughter = severe trauma.",
        "기마전투 +40%, 조련 ~100%. 동물 도살 severe trauma."),
    "M_law_speaker": ("Law-Speaker", "법을 말하는 자",
        "Trial +40%, dispute resolution 2x, lawmaking. Refuses authority above law.",
        "재판 +40%, 분쟁 2배속, 법 제정. 법 위 존재 불인정."),
    "M_stone_singer": ("Stone-Singer", "돌에게 노래하는 자",
        "Mining +50%, ore vein discovery 2x. Surface discomfort (-10%).",
        "채굴 +50%, 광맥 발견 2배. 지상 불편(-10%)."),
    "M_root_finder": ("Root-Finder", "뿌리를 찾는 자",
        "Herbs +50%, poison/antidote knowledge. Despises cultivated crops.",
        "약초 +50%, 독/해독 지식. 재배 작물 경멸."),
    "M_death_midwife": ("Death's Midwife", "죽음의 산파",
        "End-of-life care, autopsy, poison knowledge. Life detachment (emotional dulling).",
        "임종 케어, 부검, 독 지식. 삶 달관(감정 둔화)."),
    "M_bridge_builder": ("Bridge-Builder", "다리를 놓는 자",
        "Hostile faction mediation, alliance +50%. Neutrality compulsion.",
        "적대 세력 중재, 동맹 +50%. 중립 강박."),
    "M_edge_walker": ("Edge-Walker", "경계를 걷는 자",
        "Intel+combat fusion, solo +40%, escape +50%. Cannot do teamwork.",
        "첩보+전투 융합, 단독 +40%, 탈출 +50%. 팀워크 불가."),
    "M_word_carver": ("Word-Carver", "글자를 새기는 자",
        "Record preservation, histories, literacy reduction. Social -15%.",
        "기록 보존, 역사서, 문맹률 감소. 사교 -15%."),
    # ── Bond ──
    "D_soul_tethered": ("Soul-Tethered", "영혼이 묶인",
        "Near target +15%. Separation = severe stress. Target death = permanent trauma.",
        "대상 근처 +15%, 분리 severe stress. 대상 사망=permanent trauma."),
    "D_blood_oath": ("Blood Oath", "피의 맹세",
        "Mutual combat +25%, morale immune. Betrayal = Kinslayer-level stigma.",
        "상호 전투력 +25%, 사기 면역. 배신=Kinslayer급."),
    "D_eternal_grudge": ("Eternal Grudge", "영원한 원한",
        "Target tracking, revenge obsession, intelligence +40%. Everything else secondary.",
        "대상 추적·복수 집착, 정보 +40%. 복수까지 모든 것 후순위."),
    "D_shepherds_heart": ("Shepherd's Heart", "목자의 마음",
        "Nurturing +40%, personality shaping +25%. Extreme reaction to child endangerment.",
        "양육 +40%, 성격 형성 +25%. 자녀 위험 시 극단적 반응."),
    "D_twice_betrayed": ("Twice-Betrayed", "두 번 배신당한",
        "Betrayal detection +50%, trust 1/3. Cannot reach intimate level.",
        "배신 탐지 +50%, 신뢰 1/3. intimate 레벨 도달 불가."),
    "D_pack_alpha": ("Pack Alpha", "무리의 우두머리",
        "Group morale +15%, command +30%. Cannot show weakness.",
        "집단 사기 +15%, 명령 +30%. 약점 노출 불가."),
    "D_lone_wolf": ("Lone Wolf", "외로운 늑대",
        "Self-sufficiency +30%, solitude immune. Group activity -40%.",
        "자급자족 +30%, 고독 면역. 집단 활동 -40%."),
    "D_unrequited": ("Unrequited", "이루지 못한",
        "Poetry and art +40%. Stress near target, other romance impossible.",
        "시·예술 +40%. 대상 근처 stress, 다른 연애 불가."),
    "D_kingmaker": ("Kingmaker", "왕을 만드는 자",
        "Politics +40%, power transfer manipulation. Never becomes leader.",
        "정치 +40%, 권력 이양 조작. 자신은 절대 리더 안 됨."),
    "D_mirror_bond": ("Mirror Bond", "거울의 유대",
        "Mutual emotion sharing. One's trauma transfers to both.",
        "쌍방 감정 공유. 한 명 trauma=양쪽 전이."),
    "D_debt_of_life": ("Debt of Life", "목숨의 빚",
        "Absolute loyalty to benefactor. Cannot refuse benefactor's commands.",
        "은인에 절대 충성. 은인 명령 거부 불가."),
    "D_bitter_mentor": ("Bitter Mentor", "쓰라린 스승",
        "Teaching +30%, student acceleration. Self-esteem damage, complex emotions.",
        "교육 +30%, 제자 가속. 자존감 손상, 복잡한 감정."),
    "D_orphans_resolve": ("Orphan's Resolve", "고아의 결의",
        "Self-reliance +40%, crisis response +25%. Extreme family values.",
        "자립 +40%, 위기 대응 +25%. 가족 가치관 극단적."),
    "D_last_of_line": ("Last of the Line", "마지막 혈족",
        "Survival instinct +30%. Breeding urge or nihilism (fork).",
        "생존 본능 +30%. 번식 욕구 or 허무주의(분기)."),
    "D_forged_family": ("Forged Family", "만들어진 가족",
        "Non-kin bond +40%, settlement +20%. May disregard blood relatives.",
        "비혈연 결속 +40%, 정착지 +20%. 진짜 혈연 무관심 가능."),
    "D_cursed_lover": ("Cursed Lover", "저주받은 연인",
        "New lover avoidance. Attraction rises (tragic aura).",
        "새 연인 기피. 매력 오히려 상승(비극 아우라)."),
    "D_sworn_enemy": ("Sworn Enemy", "맹세한 적",
        "Target combat +30%, relentless pursuit. Until death.",
        "대상 전투력 +30%, 추적 불굴. 죽을 때까지."),
    "D_foster_bond": ("Foster Bond", "양육의 끈",
        "Foster child intimacy = blood-kin level. Conflict with birth parents.",
        "양육 아이 친밀도=혈연급. 친부모와 갈등."),
    "D_river_between": ("River Between", "사이에 놓인 강",
        "Mediation possible, dual access. Wartime agony. Suspected spy.",
        "중재 가능, 양면 접근. 전쟁 시 고통. 스파이 의심."),
    "D_chain_of_grief": ("Chain of Grief", "슬픔의 사슬",
        "Comforting others +40%, loss empathy. New relationships distant.",
        "타인 위로 +40%, 상실 공감. 새 관계 거리, 감정 억제."),
    # ── Fate ──
    "F_world_shaper": ("World-Shaper", "세계를 빚는 자",
        "Invention 5x, triggers era transitions. Enemy of existing order.",
        "발명 5배, 시대 전환 촉발. 기존 질서의 적."),
    "F_peoples_flame": ("People's Flame", "민중의 불꽃",
        "Rebellion leadership supreme, morale +40%. Risk of tyranny upon ascent.",
        "반란 지도력 최상, 사기 +40%. 권좌 시 폭군화 위험."),
    "F_deathless_name": ("Deathless Name", "죽지 않는 이름",
        "Posthumous memory. Descendant reputation legacy, cultural imprint.",
        "사후 기억. 후손 reputation 유산, 문화 각인."),
    "F_doom_bringer": ("Doom-Bringer", "파멸을 부르는 자",
        "Enemy morale -50%, surrender induction. #1 assassination target.",
        "적 사기 -50%, 항복 유도. 암살 1순위."),
    "F_last_hope": ("Last Hope", "마지막 희망",
        "Crisis +40%, morale immortal. Ordinary in peacetime.",
        "위기 시 +40%, 사기 불멸. 평시 그냥 사람."),
    "F_god_touched": ("God-Touched", "신에게 선택된",
        "Miracle events, prophet/priest supreme. Misunderstanding risk.",
        "기적 이벤트, 예언자/사제 최상. 오해 위험."),
    "F_curse_bearer": ("Curse-Bearer", "저주를 짊어진",
        "Misfortune chain: break free or fall deeper. Narrative crossroads.",
        "불행 연쇄 끊거나 빠지거나. 서사 갈림길."),
    "F_bridge_of_ages": ("Bridge of Ages", "시대의 다리",
        "Technology/culture transfer perfect. Posthumous educational legacy.",
        "기술/문화 전수 완벽. 사후 교육 유산."),
    "F_twin_crowned": ("Twin-Crowned", "이중 왕관",
        "Wide-area governance, diplomacy +50%. Both sides rebellion risk.",
        "광역 통치, 외교 +50%. 양쪽 반란 위험."),
    "F_seasons_child": ("Season's Child", "계절의 아이",
        "All-climate adaptation. Belongs nowhere.",
        "전 기후 적응. 어디에도 소속감 없음."),
    "F_ender_of_lines": ("Ender of Lines", "혈통을 끊는 자",
        "Fear and awe. Without enemies, turns on allies.",
        "공포와 경외. 적 없으면 아군을 적으로."),
    "F_silent_founder": ("Silent Founder", "조용한 시조",
        "Nameless contributor. Functionally optimal without recognition.",
        "이름 없는 공로자. 인정 없이 기능적 최적."),
    "F_prophet_of_ruin": ("Prophet of Ruin", "파멸의 예언자",
        "Prophecy maximized. Self-fulfilling prophecy (disaster induction).",
        "예언 극대. 예언의 자기 실현(재앙 유발)."),
    "F_uncrowned_king": ("Uncrowned King", "왕관 없는 왕",
        "Unofficial influence supreme. Stronger than official leaders.",
        "비공식 영향력 최상. 공식 리더보다 강함."),
    "F_memory_keeper": ("Memory-Keeper", "기억 지기",
        "Perfect history preservation. Living memory of civilization.",
        "완벽한 역사 보존. 살아있는 문명의 기억."),
    # ── Synergy: 2-trait ──
    "Y_frozen_fury": ("Frozen Fury", "얼어붙은 분노",
        "Interrogation resistance 100%, threat immune.",
        "심문 저항 100%, 위협 면역."),
    "Y_burning_glass": ("Burning Glass", "불타는 유리",
        "Emotion contagion 3x. Festival king or panic epicenter.",
        "감정 전염 3배. 축제의 왕 or 패닉 진원지."),
    "Y_iron_sun": ("Iron Sun", "철의 태양",
        "Anti-corruption crusade. Uncompromising reform.",
        "부패 척결 운동. 타협 없는 개혁."),
    "Y_velvet_knife": ("Velvet Knife", "비단의 칼",
        "Perfect double agent. All factions infiltrated.",
        "완벽한 이중 간첩. 모든 파벌 침투."),
    "Y_storm_crown": ("Storm Crown", "폭풍의 왕관",
        "Pre-battle morale +40%, enemy surrender induction.",
        "전투 전 사기 +40%, 적 항복 유도."),
    "Y_silent_forge": ("Silent Forge", "침묵의 대장간",
        "Solo crafting +40% for all types.",
        "홀로 모든 제작 +40%."),
    "Y_broken_mirror": ("Broken Mirror", "깨진 거울",
        "Unpredictable, occasional genius insight. Self-destructive.",
        "예측 불가, 가끔 천재적 통찰. 자멸적."),
    "Y_holy_fire": ("Holy Fire", "성화",
        "Merciful zealotry. Conversion maximized, martyrdom risk.",
        "자비의 광신. 개종 극대, 순교 위험."),
    "Y_wolves_pact": ("Wolves' Pact", "늑대의 서약",
        "Mutual +35%. If one dies, suicidal charge.",
        "상호 +35%. 하나 죽으면 자멸적 돌격."),
    "Y_poisoned_well": ("Poisoned Well", "독이 든 우물",
        "Perfect evil. Detection rate ~0.",
        "완벽한 악. 발각률 ~0."),
    "Y_bleeding_root": ("Bleeding Root", "피 흘리는 뿌리",
        "One with settlement. Settlement destruction = own death.",
        "정착지와 하나. 정착지 파괴=본인 사망."),
    "Y_scarred_diamond": ("Scarred Diamond", "흠집 난 다이아몬드",
        "Broken but hardened. Crisis +35%.",
        "부서졌지만 단단한. 위기 +35%."),
    "Y_autumn_hymn": ("Autumn Hymn", "가을의 찬가",
        "Wandering bard. Culture spread +50%.",
        "방랑 음유시인. 문화 전파 +50%."),
    "Y_night_garden": ("Night Garden", "밤의 정원",
        "Inner world richer than reality. Masterwork max, reality min.",
        "내면이 현실보다 풍요. 걸작 극대, 현실 최하."),
    "Y_living_monument": ("Living Monument", "살아있는 기념비",
        "Massive authority. Physical + principled. Cannot be defied.",
        "거대한 권위. 물리적+원칙. 거역 불가."),
    "Y_ember_prophet": ("Ember Prophet", "불씨의 예언자",
        "Prophecy from suffering. Accuracy extreme, mental critical.",
        "고통 속 예언. 정확도 극대, 정신 치명적."),
    "Y_gentle_cage": ("Gentle Cage", "부드러운 감옥",
        "Care becomes control. Creates dependency, burnout.",
        "돌봄이 통제. 의존성 유발, 번아웃."),
    "Y_blood_architect": ("Blood Architect", "피의 건축가",
        "Civilization built on sacrifice. Human cost ignored.",
        "희생 감수 문명 건설. 인간 비용 무시."),
    "Y_truth_serum": ("Truth Serum", "진실의 약",
        "Irresistible sincerity. Diplomacy supreme.",
        "거부 불가 진심. 외교 최상."),
    "Y_chain_march": ("Chain Breaker's March", "사슬 끊는 자의 행진",
        "Symbol of liberation. Total rejection of existing order.",
        "해방의 상징. 기존 체제 전면 거부."),
    "Y_fading_star": ("Fading Star", "꺼져가는 별",
        "Old wanderer. Legendary storyteller.",
        "늙은 방랑자. 전설적 이야기꾼."),
    "Y_red_garden": ("Red Garden", "붉은 정원",
        "Fascinating but lethal. All relationships tragic.",
        "매혹적이지만 치명적. 모든 관계 비극."),
    "Y_crown_of_thorns": ("Crown of Thorns", "가시면류관",
        "Throne's agony. Governance +30%, mental accelerated deterioration.",
        "왕좌의 고통. 통치 +30%, 정신 가속 악화."),
    "Y_titans_mercy": ("Titan's Mercy", "거인의 자비",
        "Mercy of the strong. Enemy's respect.",
        "힘센 자의 자비. 적의 존경."),
    "Y_hollow_saint": ("Hollow Saint", "텅 빈 성인",
        "Perfect saint act. Zero empathy but caring. Undetectable.",
        "완벽한 성인 연기. 공감 0이지만 돌봄. 발각 불가."),
    "Y_mirror_war": ("Mirror War", "거울의 전쟁",
        "Fated rivals. Infinite mutual growth.",
        "운명적 라이벌. 무한 성장."),
    "Y_winters_bloom": ("Winter's Bloom", "겨울에 피는 꽃",
        "Strongest in extreme + crisis combined.",
        "극한+위기에서 최강."),
    "Y_golden_chains": ("Golden Chains", "황금 사슬",
        "Oath + debt. No freedom, absolute trust.",
        "맹세+빚. 자유 없음, 절대 신뢰."),
    "Y_dream_forge": ("Dream Forge", "꿈의 대장간",
        "Imagination made real. Masterwork of masterworks.",
        "상상을 현실로. 걸작 중 걸작."),
    "Y_plague_saint": ("Plague Saint", "역병의 성인",
        "Survived plague, still cares for others. Medicine +40%.",
        "역병 겪고도 남을 돌봄. 의료 +40%."),
    # ── Synergy: 3-trait ──
    "Y_god_killer": ("God-Killer", "신살자",
        "Ultimate warrior + worst human. Civilization's catastrophe.",
        "최강 전사+최악 인간. 문명의 재앙."),
    "Y_eternal_flame": ("Eternal Flame", "영원한 불꽃",
        "Perfect leader. Golden age catalyst. No successor possible.",
        "완벽한 지도자. 황금기 촉발. 후계자 불가."),
    "Y_three_faced": ("Three-Faced", "세 얼굴",
        "History's shadow. Three identities simultaneously.",
        "역사의 흑막. 세 정체 동시 운용."),
    "Y_pain_weaver": ("Pain-Weaver", "고통을 엮는 자",
        "Pain becomes art. Cultural history eternal.",
        "고통을 예술로. 문화사 영원."),
    "Y_ashes_dawn": ("From Ashes, Dawn", "잿더미의 새벽",
        "From worst to best. Era's turning point.",
        "최악에서 최선. 시대의 전환점."),
    "Y_unmoving_storm": ("Unmoving Storm", "움직이지 않는 폭풍",
        "Absolute power machine. Inevitably tragic ending.",
        "절대 권력 기계. 반드시 비극적 결말."),
    "Y_bleeding_compass": ("Bleeding Compass", "피 흘리는 나침반",
        "Wounded wanderer connecting worlds.",
        "상처 입은 방랑자가 세계를 잇는."),
    "Y_moons_cradle": ("Moon's Cradle", "달의 요람",
        "Madness + genius fusion. Masterwork and catastrophe alternate.",
        "광기+천재 융합. 걸작과 재앙 번갈아."),
    "Y_blood_remembers": ("The Blood Remembers", "피는 기억한다",
        "Bloodline + culture + memory perfectly preserved.",
        "혈통+문화+기억 완벽 보존."),
    "Y_dawn_of_war": ("Dawn of War", "전쟁의 새벽",
        "Born for war. Does not know peace.",
        "전쟁 위해 태어난 존재. 평화를 모름."),
}


# ═══════════════════════════════════════════════════════════════
#  OUTPUT GENERATION
# ═══════════════════════════════════════════════════════════════
def main():
    all_traits = (ARCHETYPE_SINGLE + ARCHETYPE_DUAL + ARCHETYPE_TRIPLE
                  + ARCHETYPE_VALUE + SHADOW + RADIANCE + CORPUS + NOUS
                  + AWAKENED + BLOODLINE + MASTERY + BOND + FATE
                  + SYNERGY)

    # Merge effects into trait dicts
    efx_count = 0
    for t in all_traits:
        if t["id"] in EFFECTS:
            t["effects"] = EFFECTS[t["id"]]
            efx_count += 1
    if efx_count:
        print(f"Merged effects for {efx_count}/{len(all_traits)} traits")

    # Validate
    missing = [t["id"] for t in all_traits if t["id"] not in L]
    if missing:
        print(f"WARNING: Missing localization for: {missing}")

    # Count
    counts = {}
    for t in all_traits:
        counts[t["category"]] = counts.get(t["category"], 0) + 1
    print(f"Total traits: {len(all_traits)}")
    for cat, cnt in sorted(counts.items()):
        print(f"  {cat}: {cnt}")

    # Write trait_defs_v3.json
    out_path = "data/personality/trait_defs_v3.json"
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(all_traits, f, indent=2, ensure_ascii=False)
        f.write("\n")
    print(f"\nWrote {len(all_traits)} traits to {out_path}")

    # Merge localization into existing traits.json
    for lang in ["en", "ko"]:
        loc_path = f"localization/{lang}/traits.json"
        existing = {}
        if os.path.exists(loc_path):
            with open(loc_path, "r", encoding="utf-8") as f:
                existing = json.load(f)

        added = 0
        for t in all_traits:
            tid = t["id"]
            if tid not in L:
                continue
            en_name, ko_name, en_desc, ko_desc = L[tid]
            name = en_name if lang == "en" else ko_name
            desc = en_desc if lang == "en" else ko_desc
            nk = t["name_key"]
            dk = t["desc_key"]
            if nk not in existing:
                added += 1
            existing[nk] = name
            existing[dk] = desc

        with open(loc_path, "w", encoding="utf-8") as f:
            json.dump(existing, f, indent=2, ensure_ascii=False)
            f.write("\n")
        print(f"Wrote {len(existing)} keys to {loc_path} (+{added} new)")


if __name__ == "__main__":
    main()
