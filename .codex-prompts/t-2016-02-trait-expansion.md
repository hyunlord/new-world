# T-2016-02: Expanded Trait Definitions + Composite Trait Support

## Objective
Replace `data/personality/trait_definitions.json` with ~68 traits (48 facet + 20 composite), and update `scripts/systems/trait_system.gd` to support composite conditions (AND logic), display filtering, and indexed lookup.

## Files to Modify

### 1. `data/personality/trait_definitions.json` — FULL REPLACEMENT

Replace the entire file with the content below. Key changes from current 14-trait file:
- 48 facet traits: each facet gets high (≥0.85) and low (≤0.15) entries
- 20 composite traits: multi-condition with `"all"` array (AND logic)
- New field `"type": "personality"` on all entries (for future filtering)
- Field `"sentiment"` preserved (NOT renamed to "valence") — matches existing UI code
- Comment entries (objects with only `"comment"` key) for readability — must be skipped during parsing

**Complete new file content:**

```json
{
    "traits": [
        {"comment": "===== H facets ====="},

        {"id": "sincere", "name_kr": "진실한", "name_en": "Sincere",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "H_sincerity", "direction": "high", "threshold": 0.85},
         "effects": {"trust_bonus": 0.1, "deception_weight": 0.3}},

        {"id": "manipulative", "name_kr": "교활한", "name_en": "Manipulative",
         "type": "personality", "sentiment": "negative",
         "condition": {"facet": "H_sincerity", "direction": "low", "threshold": 0.15},
         "effects": {"deception_weight": 1.5, "trust_penalty": -0.15}},

        {"id": "just", "name_kr": "공정한", "name_en": "Just",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "H_fairness", "direction": "high", "threshold": 0.85},
         "effects": {"exploit_weight": 0.2, "reputation_bonus": 0.1}},

        {"id": "corrupt", "name_kr": "부패한", "name_en": "Corrupt",
         "type": "personality", "sentiment": "negative",
         "condition": {"facet": "H_fairness", "direction": "low", "threshold": 0.15},
         "effects": {"exploit_weight": 1.5, "steal_weight": 1.3}},

        {"id": "ascetic", "name_kr": "금욕적", "name_en": "Ascetic",
         "type": "personality", "sentiment": "neutral",
         "condition": {"facet": "H_greed_avoidance", "direction": "high", "threshold": 0.85},
         "effects": {"luxury_need": 0.5, "share_weight": 1.3}},

        {"id": "greedy", "name_kr": "탐욕스러운", "name_en": "Greedy",
         "type": "personality", "sentiment": "negative",
         "condition": {"facet": "H_greed_avoidance", "direction": "low", "threshold": 0.15},
         "effects": {"hoard_weight": 1.5, "share_weight": 0.3, "luxury_need": 1.5}},

        {"id": "humble", "name_kr": "겸손한", "name_en": "Humble",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "H_modesty", "direction": "high", "threshold": 0.85},
         "effects": {"lead_weight": 0.7, "cooperation_bonus": 0.1}},

        {"id": "arrogant", "name_kr": "오만한", "name_en": "Arrogant",
         "type": "personality", "sentiment": "negative",
         "condition": {"facet": "H_modesty", "direction": "low", "threshold": 0.15},
         "effects": {"lead_weight": 1.3, "conflict_chance": 1.3, "cooperation_penalty": -0.1}},

        {"comment": "===== E facets ====="},

        {"id": "fearful", "name_kr": "겁 많은", "name_en": "Fearful",
         "type": "personality", "sentiment": "negative",
         "condition": {"facet": "E_fearfulness", "direction": "high", "threshold": 0.85},
         "effects": {"flee_weight": 1.5, "explore_weight": 0.5, "combat_weight": 0.3}},

        {"id": "fearless", "name_kr": "대담한", "name_en": "Fearless",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "E_fearfulness", "direction": "low", "threshold": 0.15},
         "effects": {"explore_weight": 1.4, "combat_weight": 1.3, "flee_weight": 0.5, "reckless_chance": 1.2}},

        {"id": "anxious", "name_kr": "불안한", "name_en": "Anxious",
         "type": "personality", "sentiment": "negative",
         "condition": {"facet": "E_anxiety", "direction": "high", "threshold": 0.85},
         "effects": {"stress_buildup": 1.4, "caution_weight": 1.3, "decision_speed": 0.8}},

        {"id": "composed", "name_kr": "침착한", "name_en": "Composed",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "E_anxiety", "direction": "low", "threshold": 0.15},
         "effects": {"stress_buildup": 0.6, "crisis_efficiency": 1.3}},

        {"id": "dependent", "name_kr": "의존적", "name_en": "Dependent",
         "type": "personality", "sentiment": "neutral",
         "condition": {"facet": "E_dependence", "direction": "high", "threshold": 0.85},
         "effects": {"loneliness_sensitivity": 1.5, "partner_need": 1.4, "solo_penalty": 1.3}},

        {"id": "self_reliant", "name_kr": "자립적", "name_en": "Self-Reliant",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "E_dependence", "direction": "low", "threshold": 0.15},
         "effects": {"solo_bonus": 1.3, "loneliness_sensitivity": 0.5}},

        {"id": "empathic", "name_kr": "공감적", "name_en": "Empathic",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "E_sentimentality", "direction": "high", "threshold": 0.85},
         "effects": {"care_weight": 1.5, "relationship_speed": 1.2, "grief_intensity": 1.4}},

        {"id": "cold_hearted", "name_kr": "냉담한", "name_en": "Cold-Hearted",
         "type": "personality", "sentiment": "negative",
         "condition": {"facet": "E_sentimentality", "direction": "low", "threshold": 0.15},
         "effects": {"care_weight": 0.5, "grief_intensity": 0.5, "relationship_speed": 0.7}},

        {"comment": "===== X facets ====="},

        {"id": "confident", "name_kr": "자신감 넘치는", "name_en": "Confident",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "X_social_self_esteem", "direction": "high", "threshold": 0.85},
         "effects": {"lead_weight": 1.2, "negotiation_bonus": 0.1, "stress_resistance": 1.2}},

        {"id": "insecure", "name_kr": "자신감 없는", "name_en": "Insecure",
         "type": "personality", "sentiment": "negative",
         "condition": {"facet": "X_social_self_esteem", "direction": "low", "threshold": 0.15},
         "effects": {"lead_weight": 0.5, "stress_buildup": 1.2, "approval_need": 1.4}},

        {"id": "bold_speaker", "name_kr": "당당한", "name_en": "Bold Speaker",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "X_social_boldness", "direction": "high", "threshold": 0.85},
         "effects": {"persuasion_bonus": 0.15, "lead_weight": 1.3, "initiative_weight": 1.3}},

        {"id": "shy", "name_kr": "수줍은", "name_en": "Shy",
         "type": "personality", "sentiment": "neutral",
         "condition": {"facet": "X_social_boldness", "direction": "low", "threshold": 0.15},
         "effects": {"socialize_weight": 0.6, "public_speaking_penalty": 1.3}},

        {"id": "social_butterfly", "name_kr": "사교적인", "name_en": "Social Butterfly",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "X_sociability", "direction": "high", "threshold": 0.85},
         "effects": {"socialize_weight": 1.5, "relationship_speed": 1.3, "network_size": 1.3}},

        {"id": "reclusive", "name_kr": "은둔적", "name_en": "Reclusive",
         "type": "personality", "sentiment": "neutral",
         "condition": {"facet": "X_sociability", "direction": "low", "threshold": 0.15},
         "effects": {"socialize_weight": 0.4, "solitude_bonus": 1.4, "social_need_decay": 0.6}},

        {"id": "energetic", "name_kr": "활력 넘치는", "name_en": "Energetic",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "X_liveliness", "direction": "high", "threshold": 0.85},
         "effects": {"work_speed": 1.15, "fatigue_resistance": 1.2, "morale_bonus": 0.1}},

        {"id": "lethargic", "name_kr": "무기력한", "name_en": "Lethargic",
         "type": "personality", "sentiment": "negative",
         "condition": {"facet": "X_liveliness", "direction": "low", "threshold": 0.15},
         "effects": {"work_speed": 0.85, "fatigue_sensitivity": 1.3, "initiative_weight": 0.7}},

        {"comment": "===== A facets ====="},

        {"id": "forgiving", "name_kr": "관대한", "name_en": "Forgiving",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "A_forgiveness", "direction": "high", "threshold": 0.85},
         "effects": {"forgive_weight": 1.5, "grudge_decay": 1.5, "anger_decay": 1.3}},

        {"id": "vengeful", "name_kr": "복수심 강한", "name_en": "Vengeful",
         "type": "personality", "sentiment": "negative",
         "condition": {"facet": "A_forgiveness", "direction": "low", "threshold": 0.15},
         "effects": {"revenge_weight": 1.5, "grudge_decay": 0.3, "forgive_weight": 0.2}},

        {"id": "gentle", "name_kr": "온화한", "name_en": "Gentle",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "A_gentleness", "direction": "high", "threshold": 0.85},
         "effects": {"conflict_chance": 0.6, "care_weight": 1.2, "intimidation_weight": 0.5}},

        {"id": "harsh", "name_kr": "거친", "name_en": "Harsh",
         "type": "personality", "sentiment": "negative",
         "condition": {"facet": "A_gentleness", "direction": "low", "threshold": 0.15},
         "effects": {"conflict_chance": 1.4, "intimidation_weight": 1.3, "relationship_penalty": -0.1}},

        {"id": "flexible", "name_kr": "유연한", "name_en": "Flexible",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "A_flexibility", "direction": "high", "threshold": 0.85},
         "effects": {"negotiation_bonus": 0.15, "adaptation_speed": 1.3, "compromise_weight": 1.4}},

        {"id": "stubborn", "name_kr": "완고한", "name_en": "Stubborn",
         "type": "personality", "sentiment": "neutral",
         "condition": {"facet": "A_flexibility", "direction": "low", "threshold": 0.15},
         "effects": {"compromise_weight": 0.3, "persistence_bonus": 1.2, "conflict_chance": 1.2}},

        {"id": "patient", "name_kr": "인내심 강한", "name_en": "Patient",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "A_patience", "direction": "high", "threshold": 0.85},
         "effects": {"anger_buildup": 0.5, "long_task_bonus": 1.3, "teaching_bonus": 1.2}},

        {"id": "hot_tempered", "name_kr": "다혈질", "name_en": "Hot-Tempered",
         "type": "personality", "sentiment": "negative",
         "condition": {"facet": "A_patience", "direction": "low", "threshold": 0.15},
         "effects": {"anger_buildup": 1.5, "outburst_chance": 1.4, "conflict_chance": 1.3}},

        {"comment": "===== C facets ====="},

        {"id": "organized", "name_kr": "체계적인", "name_en": "Organized",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "C_organization", "direction": "high", "threshold": 0.85},
         "effects": {"inventory_efficiency": 1.3, "planning_bonus": 1.2, "build_quality": 1.15}},

        {"id": "messy", "name_kr": "어수선한", "name_en": "Messy",
         "type": "personality", "sentiment": "negative",
         "condition": {"facet": "C_organization", "direction": "low", "threshold": 0.15},
         "effects": {"inventory_efficiency": 0.7, "lose_item_chance": 1.3}},

        {"id": "diligent", "name_kr": "근면한", "name_en": "Diligent",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "C_diligence", "direction": "high", "threshold": 0.85},
         "effects": {"work_efficiency": 1.3, "task_persistence": 1.4, "idle_penalty": 1.2}},

        {"id": "lazy", "name_kr": "게으른", "name_en": "Lazy",
         "type": "personality", "sentiment": "negative",
         "condition": {"facet": "C_diligence", "direction": "low", "threshold": 0.15},
         "effects": {"work_efficiency": 0.7, "task_abandon_chance": 1.4, "rest_preference": 1.3}},

        {"id": "perfectionist", "name_kr": "완벽주의", "name_en": "Perfectionist",
         "type": "personality", "sentiment": "neutral",
         "condition": {"facet": "C_perfectionism", "direction": "high", "threshold": 0.85},
         "effects": {"build_quality": 1.3, "work_speed": 0.85, "stress_from_imperfection": 1.3}},

        {"id": "sloppy", "name_kr": "대충하는", "name_en": "Sloppy",
         "type": "personality", "sentiment": "negative",
         "condition": {"facet": "C_perfectionism", "direction": "low", "threshold": 0.15},
         "effects": {"build_quality": 0.7, "work_speed": 1.15, "error_chance": 1.3}},

        {"id": "prudent", "name_kr": "신중한", "name_en": "Prudent",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "C_prudence", "direction": "high", "threshold": 0.85},
         "effects": {"risk_aversion": 1.3, "impulse_resistance": 1.4, "planning_bonus": 1.2}},

        {"id": "impulsive", "name_kr": "충동적", "name_en": "Impulsive",
         "type": "personality", "sentiment": "negative",
         "condition": {"facet": "C_prudence", "direction": "low", "threshold": 0.15},
         "effects": {"impulse_action_chance": 1.5, "risk_aversion": 0.5, "regret_chance": 1.3}},

        {"comment": "===== O facets ====="},

        {"id": "aesthetic", "name_kr": "심미적", "name_en": "Aesthetic",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "O_aesthetic", "direction": "high", "threshold": 0.85},
         "effects": {"art_appreciation": 1.5, "beauty_need": 1.3, "decoration_weight": 1.4}},

        {"id": "philistine", "name_kr": "무미건조한", "name_en": "Philistine",
         "type": "personality", "sentiment": "neutral",
         "condition": {"facet": "O_aesthetic", "direction": "low", "threshold": 0.15},
         "effects": {"art_appreciation": 0.3, "beauty_need": 0.5, "practical_focus": 1.2}},

        {"id": "curious", "name_kr": "호기심 많은", "name_en": "Curious",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "O_inquisitiveness", "direction": "high", "threshold": 0.85},
         "effects": {"research_weight": 1.4, "explore_weight": 1.3, "learning_speed": 1.2}},

        {"id": "incurious", "name_kr": "무관심한", "name_en": "Incurious",
         "type": "personality", "sentiment": "neutral",
         "condition": {"facet": "O_inquisitiveness", "direction": "low", "threshold": 0.15},
         "effects": {"research_weight": 0.6, "explore_weight": 0.6, "routine_bonus": 1.2}},

        {"id": "creative", "name_kr": "창의적", "name_en": "Creative",
         "type": "personality", "sentiment": "positive",
         "condition": {"facet": "O_creativity", "direction": "high", "threshold": 0.85},
         "effects": {"invention_chance": 1.5, "problem_solving": 1.3, "unconventional_solution": 1.4}},

        {"id": "unimaginative", "name_kr": "상상력 없는", "name_en": "Unimaginative",
         "type": "personality", "sentiment": "neutral",
         "condition": {"facet": "O_creativity", "direction": "low", "threshold": 0.15},
         "effects": {"invention_chance": 0.5, "routine_bonus": 1.3}},

        {"id": "unconventional", "name_kr": "비전통적", "name_en": "Unconventional",
         "type": "personality", "sentiment": "neutral",
         "condition": {"facet": "O_unconventionality", "direction": "high", "threshold": 0.85},
         "effects": {"tradition_resistance": 1.4, "new_food_accept": 1.3, "culture_clash_resistance": 1.3}},

        {"id": "conventional", "name_kr": "전통적", "name_en": "Conventional",
         "type": "personality", "sentiment": "neutral",
         "condition": {"facet": "O_unconventionality", "direction": "low", "threshold": 0.15},
         "effects": {"tradition_bonus": 1.3, "change_resistance": 1.3, "routine_bonus": 1.2}},

        {"comment": "===== COMPOSITE TRAITS (multi-axis combinations) ====="},

        {"id": "saint", "name_kr": "성인군자", "name_en": "Saint",
         "type": "personality", "sentiment": "positive",
         "condition": {"all": [
             {"axis": "H", "direction": "high", "threshold": 0.85},
             {"axis": "A", "direction": "high", "threshold": 0.80},
             {"facet": "E_sentimentality", "direction": "high", "threshold": 0.75}
         ]},
         "effects": {"reputation_bonus": 0.3, "share_weight": 1.5, "conflict_chance": 0.3, "trust_bonus": 0.2}},

        {"id": "tyrant", "name_kr": "폭군", "name_en": "Tyrant",
         "type": "personality", "sentiment": "negative",
         "condition": {"all": [
             {"axis": "H", "direction": "low", "threshold": 0.20},
             {"axis": "A", "direction": "low", "threshold": 0.20},
             {"axis": "X", "direction": "high", "threshold": 0.75}
         ]},
         "effects": {"lead_weight": 1.5, "exploit_weight": 1.5, "intimidation_weight": 1.5, "rebellion_trigger": 1.3}},

        {"id": "mastermind", "name_kr": "책사", "name_en": "Mastermind",
         "type": "personality", "sentiment": "neutral",
         "condition": {"all": [
             {"axis": "H", "direction": "low", "threshold": 0.20},
             {"axis": "C", "direction": "high", "threshold": 0.80},
             {"facet": "X_social_boldness", "direction": "high", "threshold": 0.75}
         ]},
         "effects": {"deception_weight": 1.4, "planning_bonus": 1.3, "manipulation_weight": 1.5}},

        {"id": "artisan", "name_kr": "장인", "name_en": "Artisan",
         "type": "personality", "sentiment": "positive",
         "condition": {"all": [
             {"facet": "C_diligence", "direction": "high", "threshold": 0.80},
             {"facet": "C_perfectionism", "direction": "high", "threshold": 0.80},
             {"axis": "O", "direction": "low", "threshold": 0.35}
         ]},
         "effects": {"build_quality": 1.5, "craft_mastery_speed": 1.4, "work_efficiency": 1.2}},

        {"id": "visionary", "name_kr": "선각자", "name_en": "Visionary",
         "type": "personality", "sentiment": "positive",
         "condition": {"all": [
             {"axis": "O", "direction": "high", "threshold": 0.85},
             {"facet": "X_social_boldness", "direction": "high", "threshold": 0.75},
             {"axis": "C", "direction": "high", "threshold": 0.70}
         ]},
         "effects": {"research_weight": 1.5, "lead_weight": 1.3, "invention_chance": 1.5, "persuasion_bonus": 0.15}},

        {"id": "hermit", "name_kr": "은둔자", "name_en": "Hermit",
         "type": "personality", "sentiment": "neutral",
         "condition": {"all": [
             {"axis": "X", "direction": "low", "threshold": 0.20},
             {"facet": "E_dependence", "direction": "low", "threshold": 0.25},
             {"axis": "A", "direction": "low", "threshold": 0.35}
         ]},
         "effects": {"solitude_bonus": 1.5, "social_need_decay": 0.3, "hermitage_weight": 1.5}},

        {"id": "peacemaker", "name_kr": "화해자", "name_en": "Peacemaker",
         "type": "personality", "sentiment": "positive",
         "condition": {"all": [
             {"axis": "A", "direction": "high", "threshold": 0.85},
             {"axis": "X", "direction": "high", "threshold": 0.70},
             {"axis": "H", "direction": "high", "threshold": 0.70}
         ]},
         "effects": {"mediation_weight": 1.5, "conflict_resolution": 1.4, "reputation_bonus": 0.15}},

        {"id": "berserker", "name_kr": "광전사", "name_en": "Berserker",
         "type": "personality", "sentiment": "neutral",
         "condition": {"all": [
             {"facet": "E_fearfulness", "direction": "low", "threshold": 0.15},
             {"facet": "A_patience", "direction": "low", "threshold": 0.20},
             {"facet": "C_prudence", "direction": "low", "threshold": 0.25}
         ]},
         "effects": {"combat_weight": 1.5, "combat_damage": 1.3, "flee_weight": 0.2, "friendly_fire_chance": 1.2}},

        {"id": "schemer", "name_kr": "모사꾼", "name_en": "Schemer",
         "type": "personality", "sentiment": "negative",
         "condition": {"all": [
             {"facet": "H_sincerity", "direction": "low", "threshold": 0.20},
             {"facet": "H_fairness", "direction": "low", "threshold": 0.20},
             {"axis": "C", "direction": "high", "threshold": 0.70}
         ]},
         "effects": {"deception_weight": 1.5, "backstab_weight": 1.4, "detect_deception": 1.3}},

        {"id": "nurturer", "name_kr": "양육자", "name_en": "Nurturer",
         "type": "personality", "sentiment": "positive",
         "condition": {"all": [
             {"facet": "E_sentimentality", "direction": "high", "threshold": 0.80},
             {"facet": "A_gentleness", "direction": "high", "threshold": 0.75},
             {"facet": "A_patience", "direction": "high", "threshold": 0.75}
         ]},
         "effects": {"childcare_quality": 1.4, "teaching_bonus": 1.3, "child_development_boost": 1.2}},

        {"id": "adventurer", "name_kr": "모험가", "name_en": "Adventurer",
         "type": "personality", "sentiment": "positive",
         "condition": {"all": [
             {"facet": "E_fearfulness", "direction": "low", "threshold": 0.25},
             {"facet": "O_inquisitiveness", "direction": "high", "threshold": 0.75},
             {"axis": "X", "direction": "high", "threshold": 0.65}
         ]},
         "effects": {"explore_weight": 1.5, "migration_willingness": 1.4, "new_food_accept": 1.3, "discovery_chance": 1.3}},

        {"id": "stoic", "name_kr": "금욕주의자", "name_en": "Stoic",
         "type": "personality", "sentiment": "neutral",
         "condition": {"all": [
             {"facet": "E_anxiety", "direction": "low", "threshold": 0.20},
             {"facet": "E_sentimentality", "direction": "low", "threshold": 0.25},
             {"axis": "C", "direction": "high", "threshold": 0.70}
         ]},
         "effects": {"stress_resistance": 1.5, "grief_intensity": 0.5, "emotional_display": 0.3, "crisis_efficiency": 1.3}},

        {"id": "demagogue", "name_kr": "선동가", "name_en": "Demagogue",
         "type": "personality", "sentiment": "negative",
         "condition": {"all": [
             {"axis": "X", "direction": "high", "threshold": 0.85},
             {"axis": "H", "direction": "low", "threshold": 0.25},
             {"facet": "E_sentimentality", "direction": "high", "threshold": 0.70}
         ]},
         "effects": {"persuasion_bonus": 0.25, "manipulation_weight": 1.5, "mob_incite_weight": 1.4}},

        {"id": "free_spirit", "name_kr": "자유영혼", "name_en": "Free Spirit",
         "type": "personality", "sentiment": "neutral",
         "condition": {"all": [
             {"axis": "O", "direction": "high", "threshold": 0.80},
             {"axis": "C", "direction": "low", "threshold": 0.25},
             {"facet": "A_flexibility", "direction": "high", "threshold": 0.70}
         ]},
         "effects": {"routine_penalty": 1.3, "creativity_bonus": 1.3, "task_abandon_chance": 1.2, "novelty_seeking": 1.5}},

        {"id": "zealot", "name_kr": "광신자", "name_en": "Zealot",
         "type": "personality", "sentiment": "negative",
         "condition": {"all": [
             {"facet": "O_unconventionality", "direction": "low", "threshold": 0.15},
             {"facet": "A_flexibility", "direction": "low", "threshold": 0.20},
             {"axis": "C", "direction": "high", "threshold": 0.75}
         ]},
         "effects": {"tradition_bonus": 1.5, "change_resistance": 1.5, "persecution_weight": 1.3, "devotion_bonus": 1.4}},

        {"id": "renaissance_soul", "name_kr": "다재다능", "name_en": "Renaissance Soul",
         "type": "personality", "sentiment": "positive",
         "condition": {"all": [
             {"axis": "O", "direction": "high", "threshold": 0.80},
             {"facet": "O_inquisitiveness", "direction": "high", "threshold": 0.80},
             {"axis": "C", "direction": "high", "threshold": 0.70}
         ]},
         "effects": {"multi_skill_bonus": 1.3, "learning_speed": 1.4, "research_weight": 1.3, "jack_of_all_trades": 1.3}},

        {"id": "wallflower", "name_kr": "벽꽃", "name_en": "Wallflower",
         "type": "personality", "sentiment": "neutral",
         "condition": {"all": [
             {"axis": "X", "direction": "low", "threshold": 0.20},
             {"facet": "E_anxiety", "direction": "high", "threshold": 0.75},
             {"facet": "X_social_self_esteem", "direction": "low", "threshold": 0.25}
         ]},
         "effects": {"socialize_weight": 0.3, "observation_bonus": 1.3, "invisible_bonus": 1.2}},

        {"id": "natural_leader", "name_kr": "타고난 지도자", "name_en": "Natural Leader",
         "type": "personality", "sentiment": "positive",
         "condition": {"all": [
             {"axis": "X", "direction": "high", "threshold": 0.80},
             {"axis": "C", "direction": "high", "threshold": 0.75},
             {"axis": "H", "direction": "high", "threshold": 0.70}
         ]},
         "effects": {"lead_weight": 1.5, "follower_morale": 1.3, "reputation_bonus": 0.2, "crisis_leadership": 1.4}}
    ]
}
```

### 2. `scripts/systems/trait_system.gd` — FULL REWRITE

Replace the entire file with the following. Key changes:
- `_ensure_loaded()`: builds Dictionary index (`_trait_index`) for O(1) lookup by id; filters out comment entries
- `check_traits()`: delegates to `_evaluate_condition()` which handles both single and composite ("all") conditions
- `filter_display_traits()`: composites suppress overlapping single traits, max 5 displayed
- `get_trait_definition()`: uses indexed Dictionary lookup
- `get_trait_sentiment()`: unchanged (reads "sentiment" field)

```gdscript
extends RefCounted

## Discrete trait emergence system with composite trait support.
## Checks personality extremes and returns active traits + combined effects.
## Supports single conditions (facet/axis threshold) and composite conditions
## ("all" array = AND logic across multiple facet/axis checks).
## Use preload("res://scripts/systems/trait_system.gd") for access.

static var _trait_definitions: Array = []
static var _trait_index: Dictionary = {}  # id -> trait definition Dictionary
static var _loaded: bool = false


static func _ensure_loaded() -> void:
	if _loaded:
		return
	var file = FileAccess.open("res://data/personality/trait_definitions.json", FileAccess.READ)
	if file == null:
		push_warning("[TraitSystem] Cannot load trait_definitions.json")
		_loaded = true
		return
	var json = JSON.new()
	if json.parse(file.get_as_text()) != OK:
		push_warning("[TraitSystem] Invalid trait_definitions.json")
		_loaded = true
		return
	var raw_traits = json.data.get("traits", [])
	# Filter out comment entries and build index
	_trait_definitions = []
	_trait_index = {}
	for i in range(raw_traits.size()):
		var entry = raw_traits[i]
		if entry.has("comment") and not entry.has("id"):
			continue  # Skip comment-only entries
		_trait_definitions.append(entry)
		var tid = entry.get("id", "")
		if tid != "":
			_trait_index[tid] = entry
	_loaded = true


## Check which traits are active for a given PersonalityData.
## Returns Array of trait ID strings (all matching, before display filtering).
static func check_traits(pd: RefCounted) -> Array:
	_ensure_loaded()
	var traits: Array = []
	for i in range(_trait_definitions.size()):
		var tdef = _trait_definitions[i]
		var cond = tdef.get("condition", {})
		if _evaluate_condition(cond, pd):
			traits.append(tdef.get("id", ""))
	return traits


## Evaluate a trait condition against PersonalityData.
## Supports single conditions (facet/axis) and composite conditions ("all" array).
static func _evaluate_condition(condition: Dictionary, pd: RefCounted) -> bool:
	if condition.has("all"):
		# Composite: ALL sub-conditions must pass (AND logic)
		var subs = condition.get("all", [])
		for i in range(subs.size()):
			if not _evaluate_single(subs[i], pd):
				return false
		return true
	else:
		# Single condition
		return _evaluate_single(condition, pd)


## Evaluate a single facet/axis condition.
static func _evaluate_single(cond: Dictionary, pd: RefCounted) -> bool:
	var value: float = 0.5
	if cond.has("facet"):
		value = pd.facets.get(cond.get("facet", ""), 0.5)
	elif cond.has("axis"):
		value = pd.axes.get(cond.get("axis", ""), 0.5)

	var threshold = float(cond.get("threshold", 0.5))
	var direction = cond.get("direction", "")
	if direction == "high":
		return value >= threshold
	elif direction == "low":
		return value <= threshold
	return false


## Filter traits for UI display.
## Composite traits suppress their component single traits.
## Returns at most max_display traits (composites prioritized).
static func filter_display_traits(all_trait_ids: Array, max_display: int = 5) -> Array:
	_ensure_loaded()
	var composites: Array = []
	var singles: Array = []
	for i in range(all_trait_ids.size()):
		var tid = all_trait_ids[i]
		var tdef = _trait_index.get(tid, {})
		var cond = tdef.get("condition", {})
		if cond.has("all"):
			composites.append(tid)
		else:
			singles.append(tid)

	# Build suppression set: single traits overlapping with composite sub-conditions
	var suppressed: Dictionary = {}
	for i in range(composites.size()):
		var cid = composites[i]
		var cdef = _trait_index.get(cid, {})
		var subs = cdef.get("condition", {}).get("all", [])
		for j in range(subs.size()):
			var sub = subs[j]
			# Check each single trait for overlap
			for k in range(singles.size()):
				var sid = singles[k]
				var sdef = _trait_index.get(sid, {})
				var scond = sdef.get("condition", {})
				if _conditions_overlap(sub, scond):
					suppressed[sid] = true

	# Build result: composites first, then non-suppressed singles
	var filtered: Array = composites.duplicate()
	for i in range(singles.size()):
		if not suppressed.has(singles[i]):
			filtered.append(singles[i])

	# Cap at max_display
	if filtered.size() > max_display:
		filtered.resize(max_display)
	return filtered


## Check if a composite sub-condition overlaps with a single trait condition.
## Overlap = same facet or same axis with same direction.
static func _conditions_overlap(sub: Dictionary, single_cond: Dictionary) -> bool:
	if sub.has("facet") and single_cond.has("facet"):
		return sub.get("facet", "") == single_cond.get("facet", "") and sub.get("direction", "") == single_cond.get("direction", "")
	if sub.has("axis") and single_cond.has("axis"):
		return sub.get("axis", "") == single_cond.get("axis", "") and sub.get("direction", "") == single_cond.get("direction", "")
	# Axis sub vs facet single: check if facet belongs to axis
	if sub.has("axis") and single_cond.has("facet"):
		var facet_key = single_cond.get("facet", "")
		var axis_key = sub.get("axis", "")
		if facet_key.begins_with(axis_key + "_") and sub.get("direction", "") == single_cond.get("direction", ""):
			return true
	return false


## Get combined effect multipliers from a list of active trait IDs.
## Effects are combined multiplicatively (multiple traits stack).
## Returns Dictionary of effect_key -> combined_multiplier.
static func get_trait_effects(trait_ids: Array) -> Dictionary:
	_ensure_loaded()
	var combined: Dictionary = {}
	for i in range(trait_ids.size()):
		var tid = trait_ids[i]
		var tdef = _trait_index.get(tid, {})
		if tdef.is_empty():
			continue
		var effects = tdef.get("effects", {})
		var effect_keys = effects.keys()
		for k in range(effect_keys.size()):
			var ek = effect_keys[k]
			var ev = float(effects[ek])
			if combined.has(ek):
				combined[ek] = combined[ek] * ev
			else:
				combined[ek] = ev
	return combined


## Get trait definition by ID (for UI display).
## Returns Dictionary with id, name_kr, name_en, sentiment, etc.
static func get_trait_definition(trait_id: String) -> Dictionary:
	_ensure_loaded()
	return _trait_index.get(trait_id, {})


## Get sentiment for a trait ("positive", "negative", "neutral").
static func get_trait_sentiment(trait_id: String) -> String:
	var tdef = get_trait_definition(trait_id)
	return tdef.get("sentiment", "neutral")
```

## Non-goals
- Do NOT modify personality_data.gd or personality_generator.gd (that's T-2016-01)
- Do NOT modify any UI files (entity_detail_panel.gd etc.)
- Do NOT implement actual gameplay effects from trait effects — data only for now
- Do NOT modify PersonalitySystem or personality_compatibility()
- Trait effect keys are for future Utility AI (Phase C1) — just define the data

## Acceptance Criteria
- [ ] trait_definitions.json has exactly 48 facet traits + 20 composite traits = 68 entries (plus comment entries)
- [ ] All facet traits use threshold 0.85 (high) / 0.15 (low)
- [ ] All composite traits have "all" array with 3 sub-conditions
- [ ] trait_system.gd correctly skips "comment" entries during loading
- [ ] trait_system.gd `check_traits()` supports both single and composite conditions
- [ ] trait_system.gd `filter_display_traits()` suppresses overlapping singles, caps at 5
- [ ] trait_system.gd `get_trait_effects()` uses indexed lookup (O(1) per trait, not O(n))
- [ ] `get_trait_sentiment()` reads "sentiment" field (matches existing UI code)
- [ ] No GDScript parse errors

## Godot 4.6 Notes
- trait_system.gd uses `extends RefCounted`, NO `class_name`
- All functions are `static` — no instance needed
- Use `preload("res://scripts/systems/trait_system.gd")` to reference
- `var x = dict.get(...)` (untyped, no `:=`) to avoid inference errors
