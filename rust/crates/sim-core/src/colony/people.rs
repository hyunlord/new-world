//! M5 components on people: illness (ADR 0027), drafting (ADR 0028), apparel (ADR 0029) and mood (ADR 0030).
use serde::{Deserialize, Serialize};

/// A sick person (M5 illness, ADR 0027): slower work and more sleep need until `until`; never fatal by itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sick {
    /// The tick the illness ends.
    pub until: u64,
}

/// A top a person can wear (M5 apparel, ADR 0029); no `Apparel` component = the loincloth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Top {
    /// A wrap of cured hides.
    HideWrap,
    /// A cloak of wolf fur (droppable 8.5).
    FurCloak,
}

impl Top {
    /// Stable code for the snapshot's apparel column (0 = loincloth).
    pub const fn code(self) -> u8 {
        match self {
            Self::HideWrap => 1,
            Self::FurCloak => 2,
        }
    }

    /// Chance ‰ that a cold hour (the hearth out on a winter night) bites the wearer (goal 3.3: hide wrap ×700,
    /// fur cloak ×500; DECISIONS D15).
    pub fn cold_permille(self) -> u32 {
        self.item()
            .definition()
            .apparel
            .as_ref()
            .map_or(1000, |a| a.cold_permille)
    }

    /// The unique item represented by this legacy saved apparel slot.
    pub fn item(self) -> super::ItemKind {
        super::ItemKind::all()
            .find(|kind| {
                kind.definition()
                    .apparel
                    .as_ref()
                    .is_some_and(|a| a.legacy_top_code == self.code())
            })
            .unwrap_or_else(|| panic!("installed apparel mapping missing for Top {}", self.code()))
    }

    /// The saved apparel slot of a supported item.
    pub fn of_item(kind: super::ItemKind) -> Option<Self> {
        match kind.definition().apparel.as_ref()?.legacy_top_code {
            1 => Some(Self::HideWrap),
            2 => Some(Self::FurCloak),
            _ => None,
        }
    }

    /// Clothing preference from the installed definition.
    pub fn warmth_rank(self) -> u16 {
        self.item()
            .definition()
            .apparel
            .as_ref()
            .map_or(0, |a| a.warmth_rank)
    }

    /// Whether this supported top shields work from a cold snap.
    pub fn cold_snap_work_immune(self) -> bool {
        self.item()
            .definition()
            .apparel
            .as_ref()
            .is_some_and(|a| a.cold_snap_work_immune)
    }
}

/// What a person wears (M5 apparel, ADR 0029): the top and the day it was put on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Apparel {
    /// The top worn.
    pub top: Top,
    /// The display day it was put on.
    pub since_day: u16,
}

/// What a drafted person was told to do (M5 drafting, ADR 0028).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DraftOrder {
    /// Stand where they are.
    Hold,
    /// Walk to a tile and stand there.
    MoveTo((u32, u32)),
    /// Attack an animal or a person.
    Attack(super::wildlife::Fighter),
}

/// A drafted person (M5, ADR 0028): takes only direct orders; the decision system skips them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Drafted {
    /// The tick they were drafted.
    pub since: u64,
    /// The standing order.
    pub order: DraftOrder,
    /// A weapon stack they walk to first (auto-arm), if any.
    pub arming: Option<(u32, u32)>,
}

/// A person running from a wolf (M5.5, ADR 0028 amendment; `FleeSystem`): the decision seam skips them while it
/// lasts, as it skips a drafted person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fleeing {
    /// The tick the flight ends unless a wolf is still near (renewed then).
    pub until: u64,
    /// The wolf it began with (`AnimalId`).
    pub from: u32,
    /// Where they run: a roofed room's tile, the hearth, or a tile away from the wolf.
    pub to: (u32, u32),
    /// The tick the flight began.
    pub since: u64,
}

/// A mood modifier (M5, ADR 0030): what moves a person's mood target, with its weight ‰ of the scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Modifier {
    /// Slept in a roofed bedroom last night (+80).
    SleptRoofed,
    /// Slept outside last night (−120).
    SleptOutside,
    /// Fed: hunger below its seek threshold (+40).
    Fed,
    /// Hungry: hunger at or past its seek threshold (−150).
    Hungry,
    /// Thirsty: thirst at or past its seek threshold (−100).
    Thirsty,
    /// Cold last night (−120).
    Cold,
    /// Quarrelled today (−100).
    Quarrelled,
    /// Taught or learned today (+60).
    Learned,
    /// Discovered something, for 3 days (+150).
    Discovered,
    /// A kin died, for 6 days (−250).
    Grief,
    /// Sick (−80).
    Sick,
    /// Worked a gated craft as its practitioner today (+40).
    Crafted,
    /// Drafted a day or more (−60).
    Drafted,
    /// A pressure is active (−40).
    Pressure,
    /// At least three ingredient groups eaten in the trailing three days (+60).
    FoodVaried,
    /// Exactly one ingredient group eaten in the trailing three days (−40).
    FoodMonotonous,
}

impl Modifier {
    /// Every modifier in code order.
    pub const ALL: [Self; 16] = [
        Self::SleptRoofed,
        Self::SleptOutside,
        Self::Fed,
        Self::Hungry,
        Self::Thirsty,
        Self::Cold,
        Self::Quarrelled,
        Self::Learned,
        Self::Discovered,
        Self::Grief,
        Self::Sick,
        Self::Crafted,
        Self::Drafted,
        Self::Pressure,
        Self::FoodVaried,
        Self::FoodMonotonous,
    ];

    /// Stable code (the chronicle's cause table, the bridge).
    pub const fn code(self) -> u8 {
        self as u8
    }

    /// Stable lowercase name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::SleptRoofed => "slept_roofed",
            Self::SleptOutside => "slept_outside",
            Self::Fed => "fed",
            Self::Hungry => "hungry",
            Self::Thirsty => "thirsty",
            Self::Cold => "cold",
            Self::Quarrelled => "quarrelled",
            Self::Learned => "learned",
            Self::Discovered => "discovered",
            Self::Grief => "grief",
            Self::Sick => "sick",
            Self::Crafted => "crafted",
            Self::Drafted => "drafted",
            Self::Pressure => "pressure",
            Self::FoodVaried => "food_varied",
            Self::FoodMonotonous => "food_monotonous",
        }
    }

    /// The modifier of [`Self::code`].
    pub fn from_code(code: u8) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.code() == code)
    }

    /// Its weight on the mood target (goal 3.4), before temperament.
    pub const fn weight(self) -> i32 {
        match self {
            Self::SleptRoofed => 80,
            Self::SleptOutside => -120,
            Self::Fed => 40,
            Self::Hungry => -150,
            Self::Thirsty => -100,
            Self::Cold => -120,
            Self::Quarrelled => -100,
            Self::Learned => 60,
            Self::Discovered => 150,
            Self::Grief => -250,
            Self::Sick => -80,
            Self::Crafted => 40,
            Self::Drafted => -60,
            Self::Pressure => -40,
            Self::FoodVaried => 60,
            Self::FoodMonotonous => -40,
        }
    }

    /// Whether reward dependence scales it (the social ones), else harm avoidance scales a negative one.
    pub const fn social(self) -> bool {
        matches!(self, Self::Quarrelled | Self::Learned | Self::Grief)
    }

    /// The quarrel cause it maps to (0 hunger, 1 cold, 2 sleep, 3 grief, 4 pressure, 5 relation).
    pub const fn quarrel_cause(self) -> u8 {
        match self {
            Self::Hungry | Self::Thirsty => 0,
            Self::Cold => 1,
            Self::SleptOutside => 2,
            Self::Grief => 3,
            Self::Sick | Self::Drafted | Self::Pressure => 4,
            _ => 5,
        }
    }
}

/// Quarrel cause names by code (`Modifier::quarrel_cause`).
pub const QUARREL_CAUSES: [&str; 6] = ["hunger", "cold", "sleep", "grief", "pressure", "relation"];

/// "No day" in a mood's day fields.
pub const NO_DAY: u16 = u16::MAX;

/// A person's mood (M5, ADR 0030): the value and the facts its modifiers read.
/// Days are display days (`NO_DAY` = never).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mood {
    /// 0..=1000; 500 is even.
    pub value: i16,
    /// Last night's facts (bit 0 slept in a roofed bedroom, 1 slept outside, 2 cold);
    /// bits 3–5 the same for the night in progress.
    pub nights: u8,
    /// The day of the last quarrel.
    pub quarrel_day: u16,
    /// The day the person last taught or learned.
    pub learned_day: u16,
    /// The day the person last worked a gated craft as its practitioner.
    pub crafted_day: u16,
    /// The last day a discovery still cheers (inclusive).
    pub joy_until: u16,
    /// The last day a kin's death still grieves (inclusive).
    pub grief_until: u16,
    /// The day a breakdown ends (the person refuses jobs through it).
    pub breakdown_until: u16,
    /// The chronicle entry of the death grieved for (`u32::MAX` = none).
    pub grief_entry: u32,
}

impl Default for Mood {
    fn default() -> Self {
        Self {
            value: 500,
            nights: 0,
            quarrel_day: NO_DAY,
            learned_day: NO_DAY,
            crafted_day: NO_DAY,
            joy_until: NO_DAY,
            grief_until: NO_DAY,
            breakdown_until: NO_DAY,
            grief_entry: u32::MAX,
        }
    }
}

impl Mood {
    /// Whether `day` falls on or before `until` (a day field that is set).
    pub const fn within(until: u16, day: u16) -> bool {
        until != NO_DAY && day <= until
    }

    /// Whether the person is in a breakdown on `day`.
    pub const fn broken(&self, day: u16) -> bool {
        Self::within(self.breakdown_until, day)
    }
}

/// A person-pair relation (M5, ADR 0030; `ColonyState::bonds`, key `(lower id, higher id)`):
/// the value after the last change, and the day it was set; it drifts back to its kin base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bond {
    /// −1,000..=1,000 when set.
    pub value: i16,
    /// The display day it was set.
    pub day: u16,
    /// The chronicle entry of the pair's last quarrel (`u32::MAX` = none).
    #[serde(default = "no_entry")]
    pub last_quarrel: u32,
}

const fn no_entry() -> u32 {
    u32::MAX
}

/// The cause code (`CauseRef::Modifier`) of a quarrel over the relation itself: no mood modifier, the
/// pair's bad blood (its `entry` is the pair's previous quarrel, if any).
pub const RELATION_CAUSE: u8 = 100;

impl Bond {
    /// The value on `day`, drifting toward `base` by `per_day` a day since it was set.
    pub fn value_on(&self, day: u16, base: i16, per_day: i16) -> i16 {
        let days = i32::from(day.saturating_sub(self.day));
        let step = days * i32::from(per_day);
        let v = i32::from(self.value);
        let b = i32::from(base);
        let moved = if v < b {
            (v + step).min(b)
        } else {
            (v - step).max(b)
        };
        moved as i16
    }
}

/// Mood counters (`ColonyState::mood_tallies`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoodTallies {
    /// Quarrels.
    pub quarrels: u64,
    /// Fights after a quarrel.
    pub fights: u64,
    /// Breakdowns.
    pub breakdowns: u64,
}
