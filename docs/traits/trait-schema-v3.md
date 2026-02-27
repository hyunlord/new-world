# WorldSim Trait System v3 — Schema Reference
> Canonical source for trait data structures, operations, and system targets.

## 2. Trait Top-Level Structure
```yaml
trait:
  id: String                    # unique ID (A_incorruptible, S_hollow_crown, ...)
  name:
    en: String                  # English name (The Incorruptible)
    ko: String                  # Korean name (청렴결백)
  category: String              # archetype | shadow | radiance | corpus | nous | awakened | bloodline | mastery | bond | fate | synergy
  rarity: String                # common | uncommon | rare | epic | legendary

  acquisition:
    type: String                # personality | physical | cognitive | event | genetic | mastery | relationship | composite | player
    conditions: []              # Condition object array
    require_all: bool           # true=AND, false=OR

  effects: []                   # Effect object array

  loss_conditions:
    type: String                # permanent | decay | betrayal | transform | threshold
    detail: String
    transform_to: String?

  incompatible_with: []         # trait IDs that cannot coexist

  display:
    border: String              # silver | crimson | gold | bronze | azure | purple | maroon | emerald | rose | iridescent
    glow: bool

  meta: {}                      # free extension field
```

## 3. Condition Structure
```yaml
condition:
  source: String    # hexaco | body | gardner | value | event | genetic | skill | relationship | derived | age | memory | self | leader | target | settlement | world
  key: String       # specific key (H, strength, logical, MARTIAL, is_alone, ...)
  op: String        # gte | lte | eq | neq | gt | lt | has | not_has | count_gte | between
  value: Variant    # comparison value
```

## 4. Effect Structure
```yaml
effect:
  system: String              # target system (see System Registry)
  target: String | String[]   # variable within system
  op: String                  # operation (see Op Registry)
  value: Variant              # value
  condition:                  # optional structured condition (null = always active)
    source: String
    key: String
    op: String
    value: Variant
  priority: int?              # conflict resolution (higher wins, default 0)
  tags: String[]?             # classification tags
  meta: {}?                   # extension field
```

## 5. Op Registry (full canonical list)
| Op | Meaning | Value Type | Example |
|----|---------|-----------|---------|
| `set` | force-fix a value | any | `emotion.guilt → 0.0` |
| `add` | absolute addition | float | `stress.base_load + 0.15` |
| `mult` | multiplier | float | `combat.damage × 1.4` |
| `min` | floor guarantee (cannot go below) | float | `emotion.anger ≥ 0.3` |
| `max` | ceiling cap (cannot exceed) | float | `relationship.intimacy ≤ 70` |
| `disable` | fully disable a system/coping/category | bool | `coping.social_support = disabled` |
| `enable` | unlock a hidden action or system | bool/obj | `action.prophesy = enabled` |
| `block` | remove from behavior candidate pool | bool | `behavior -= ["forgive", "console"]` |
| `inject` | insert into behavior candidate pool | string/obj | `behavior.queue += "seek_revenge"` |
| `override` | replace an entire decision/judgment | string | `combat.flee = never` |
| `on_event` | fire event when condition met | obj | `on: hp<0.5 → enter_berserk` |
| `tag` | attach a reputation/status tag | string | `reputation += "untarnished"` |
| `immune` | full immunity to specific source | bool/list | `stress.source.guilt = immune` |
| `replace` | replace entire value structure | obj | `break_types = {berserk: 0.7, ...}` |

## 6. System Registry
| System | Description | Example Targets |
|--------|-------------|-----------------|
| `stress` | Lazarus stress model | `accumulation_rate`, `allostatic_load`, `mental_break_threshold`, `break_types`, `source_immunity`, `coping` |
| `emotion` | Plutchik emotions | `joy`, `trust`, `fear`, `surprise`, `sadness`, `disgust`, `anger`, `anticipation`, `guilt`, `contempt`, `decay_rate`, `intensity_mult`, `volatility` |
| `need` | Maslow+ERG needs | `hunger`, `thirst`, `sleep`, `warmth`, `safety`, `belonging`, `intimacy`, `recognition`, `autonomy`, `competence`, `meaning`, `transcendence` |
| `relationship` | Social relations | `intimacy_gain`, `intimacy_ceil`, `first_impression`, `betrayal_cooldown`, `trust`, `mode`, `bond_impact` |
| `combat` | Combat | `damage_mult`, `morale_floor`, `crit_chance`, `flee_threshold`, `kill_stress`, `panic` |
| `behavior` | Behavior tree | `inject`, `block`, `priority_override`, `decision_weight` |
| `skill` | Skill learning | `learning_rate`, `category_mult`, `ceiling`, `decay_rate` |
| `memory` | Memory | `trauma_intensity`, `positive_intensity`, `compression_resist`, `kill_trauma`, `distortion` |
| `derived` | Derived stats | `charisma`, `intimidation`, `allure`, `trustworthiness`, `creativity`, `wisdom` |
| `body` | Physical | `strength`, `agility`, `endurance`, `toughness`, `recuperation`, `disease_resistance` |
| `aging` | Aging | `decline_start`, `decline_rate`, `lifespan_mult`, `chronic_chance` |
| `fertility` | Reproduction | `fertility_mult`, `twin_chance`, `child_stat_bonus` |
| `reputation` | Reputation | `tags`, `spread_speed`, `decay_rate`, `visibility` |
| `values` | Value system | `(33 value IDs)`, `drift_rate` |
| `event` | Events | `festival_mult`, `discovery_mult`, `crisis_response` |
| `aura` | Area effects | `emotion_type`, `radius`, `intensity`, `target_filter` |
| `genetics` | Genetics | `inheritance`, `mutation_rate`, `carrier_status` |
| `magic` | Magic (future) | `mana_affinity`, `spell_learning`, `resistance` |
| `religion` | Religion (future) | `faith`, `piety`, `heresy_resistance` |
| `mount` | Mounted (future) | `ride_skill`, `bond`, `charge_damage` |
| `crafting` | Crafting | `quality_bonus`, `unique_recipes`, `material_efficiency` |

## 7. Worked Example
```yaml
trait:
  id: A_incorruptible
  name:
    en: The Incorruptible
    ko: 청렴결백
  category: archetype
  rarity: rare

  acquisition:
    type: personality
    conditions:
      - source: hexaco
        key: H
        op: gte
        value: 0.88
    require_all: true

  effects:
    - system: behavior
      target: [accept_bribe, steal, fraud, embezzle]
      op: block
      value: true
      tags: [integrity]

    - system: behavior
      target: trade_pricing
      op: override
      value: fair_price_only

    - system: derived
      target: trustworthiness
      op: add
      value: 0.25

    - system: reputation
      target: tags
      op: tag
      value: untarnished

    - system: behavior
      target: [compromise, bluff, deceive]
      op: block
      value: true

    - system: behavior
      target: diplomacy_filter
      op: set
      value: reject_if_unfair

    - system: stress
      target: corruption_exposure
      op: set
      value: 0.1
      condition:
        source: leader
        key: tags
        op: has
        value: corrupt

  loss_conditions:
    type: permanent
    detail: No explicit loss path.
    transform_to: null

  incompatible_with: [A_serpent_tongue]

  display:
    border: silver
    glow: false

  meta: {}
```

## 8. Deprecated Ops Reference
| Deprecated | Replacement | Migration Note |
|-----------|------------|----------------|
| floor | min | Same semantics |
| ceil | max | Same semantics |
| lock | disable | Same semantics |
| unlock | enable | Same semantics |
| remove | block | behavior system only |
| trigger | on_event | Same semantics |
| immunity | immune | Same semantics |
| scale | mult + condition | Decompose into mult with structured condition |
| curve | mult + condition | Decompose into mult with structured condition |
