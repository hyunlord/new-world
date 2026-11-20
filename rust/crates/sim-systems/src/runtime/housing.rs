//! HOUSE1 housing helpers (not a system): house history and ownership, persistent households,
//! exclusive owned beds and the derived `HouseView` read (ADR 0072).
//!
//! Boundaries (no per-tick scan): `BuildingTemplateSystem` (p153) calls [`record_completion`] when
//! a Dwelling template physically completes; `BedAssignmentSystem` (p145) calls [`reconcile`]
//! whenever it reassigns beds (20:00 or a shelter change) and then [`bed_claims`]; save load calls
//! [`migrate`] for pre-format26 saves. Colony only: baseline worlds never reach this module.
//! Reads: `Agent`, `Kin`, `Sex`, `Child`, `Life`, `BodyHealth`; settlements; `bed_of`; building
//! sites, rooms, beds, `building_kinds`, `house_growth`; chronicle causes (legacy dates).
//! Writes: `colony.buildings.housing`; chronicle.
//! Events: `HouseholdFormed`, `HouseOwnershipChanged` (never from founding or migration).
use hecs::World;
use sim_core::chronicle::{CauseRef, ChronicleEntry, ChronicleKind};
use sim_core::colony::building_site::{BuildingSite, BuildingSiteId, BuildingStatus, PiecePhase};
use sim_core::colony::building_template::BuildingCategory;
use sim_core::colony::content::installed_or_bundled;
use sim_core::colony::household::{
    pack_ownership_change, BedRef, HouseHistory, HouseId, HouseStatus, HouseView, Household,
    HouseholdId, HousingState, OwnershipReason, ResidenceRule,
};
use sim_core::components::{
    Agent, AgentId, BodyHealth, BuildingKind, Child, Kin, Life, SettlementId, Sex,
};
use sim_engine::SimResources;
use std::collections::{BTreeMap, BTreeSet};

type Tile = (u32, u32);

/// Whether a site's current template is in the Dwelling category (data, never a name).
pub fn is_dwelling(site: &BuildingSite) -> bool {
    installed_or_bundled()
        .building_templates()
        .get(&site.template)
        .is_some_and(|template| template.category == BuildingCategory::Dwelling)
}

fn contains(site: &BuildingSite, tile: Tile) -> bool {
    tile.0 >= site.anchor.0
        && tile.1 >= site.anchor.1
        && tile.0 < site.anchor.0.saturating_add(site.footprint.0)
        && tile.1 < site.anchor.1.saturating_add(site.footprint.1)
}

/// The settlement whose formation tile is nearest (Chebyshev), ties lowest id.
pub fn settlement_of(res: &SimResources, tile: Tile) -> Option<SettlementId> {
    res.settlements
        .iter()
        .min_by_key(|(id, settlement)| {
            let home = settlement.formation_tile;
            (tile.0.abs_diff(home.0).max(tile.1.abs_diff(home.1)), **id)
        })
        .map(|(id, _)| *id)
}

/// Intact, complete Dwelling: offered to claims and counted for ownership.
fn usable(res: &SimResources, house: HouseId) -> bool {
    res.colony
        .buildings
        .sites
        .get(&house.0)
        .is_some_and(|site| is_dwelling(site) && super::building_templates::is_intact(res, site))
}

/// Usable, or a known house rebuilding the same id (upgrade): ownership survives.
fn standing(res: &SimResources, house: HouseId) -> bool {
    usable(res, house)
        || res.colony.buildings.housing.houses.contains_key(&house)
            && res
                .colony
                .buildings
                .sites
                .get(&house.0)
                .is_some_and(|site| site.status == BuildingStatus::Building)
}

/// The known house whose footprint holds `tile` (lowest id on overlap).
pub fn house_at(res: &SimResources, tile: Tile) -> Option<HouseId> {
    res.colony
        .buildings
        .housing
        .houses
        .keys()
        .copied()
        .find(|house| {
            res.colony
                .buildings
                .sites
                .get(&house.0)
                .is_some_and(|site| contains(site, tile))
        })
}

#[derive(Clone, Copy)]
struct Person {
    kin: Kin,
    female: bool,
    adult: bool,
    born: i64,
}

fn people(world: &World) -> BTreeMap<AgentId, Person> {
    world
        .query::<(
            &Agent,
            Option<&Kin>,
            Option<&Sex>,
            Option<&Child>,
            Option<&Life>,
            Option<&BodyHealth>,
        )>()
        .iter()
        .filter(|(_, (.., health))| health.is_none_or(|health| !health.is_dead()))
        .map(|(_, (agent, kin, sex, child, life, _))| {
            (
                agent.id,
                Person {
                    kin: kin.copied().unwrap_or_default(),
                    female: sex.is_some_and(|sex| sex.female),
                    adult: child.is_none(),
                    born: life.map_or(0, |life| life.born_tick),
                },
            )
        })
        .collect()
}

/// Oldest adult (earliest birth), ties lowest id; else the lowest member.
fn head_of(members: &BTreeSet<AgentId>, people: &BTreeMap<AgentId, Person>) -> Option<AgentId> {
    members
        .iter()
        .filter_map(|id| people.get(id).filter(|p| p.adult).map(|p| (p.born, *id)))
        .min()
        .map(|(_, id)| id)
        .or_else(|| members.first().copied())
}

/// Each living settlement member's settlement (lowest settlement id first).
fn home_settlements(res: &SimResources) -> BTreeMap<AgentId, SettlementId> {
    let mut ids: Vec<_> = res.settlements.keys().copied().collect();
    ids.sort_unstable();
    let mut out = BTreeMap::new();
    for sid in ids {
        if let Some(settlement) = res.settlements.get(&sid) {
            let mut members: Vec<_> = settlement.member_agents.iter().copied().collect();
            members.sort_unstable();
            for id in members {
                out.entry(id).or_insert(sid);
            }
        }
    }
    out
}

fn household_settlement(
    household: &Household,
    homes: &BTreeMap<AgentId, SettlementId>,
) -> Option<SettlementId> {
    homes
        .get(&household.head)
        .copied()
        .or_else(|| household.members.iter().find_map(|m| homes.get(m).copied()))
}

/// Households with at least one living adult member: only they claim or receive houses.
fn adult_led(state: &HousingState, people: &BTreeMap<AgentId, Person>) -> BTreeSet<HouseholdId> {
    state
        .households
        .values()
        .filter(|h| {
            h.members
                .iter()
                .any(|m| people.get(m).is_some_and(|p| p.adult))
        })
        .map(|h| h.id)
        .collect()
}

/// Whether `household` owns any house.
fn owns_any(state: &HousingState, household: HouseholdId) -> bool {
    state.ownership.values().any(|owner| *owner == household)
}

/// Who may sleep in `owner`'s beds: its members and its houseless sub-households' members.
fn sleepers_of(state: &HousingState, owner: HouseholdId) -> Vec<AgentId> {
    let mut list: Vec<AgentId> = state
        .households
        .get(&owner)
        .map(|h| h.members.iter().copied().collect())
        .unwrap_or_default();
    for sub in state
        .households
        .values()
        .filter(|h| h.parent == Some(owner) && !owns_any(state, h.id))
    {
        list.extend(sub.members.iter().copied());
    }
    list
}

/// Chronicle entries collected while the state is borrowed, recorded afterwards.
struct Events(Vec<ChronicleEntry>);

impl Events {
    fn formed(&mut self, tick: u64, household: &Household, settlement: Option<SettlementId>) {
        let mut entry = ChronicleEntry::new(tick, ChronicleKind::HouseholdFormed);
        entry.subject = Some(household.head);
        entry.cause = household.id.0;
        entry.value = household.parent.map_or(-1, |parent| i64::from(parent.0));
        entry.settlement = settlement;
        self.0.push(entry);
    }

    fn owner(
        &mut self,
        res: &SimResources,
        house: HouseId,
        from: Option<HouseholdId>,
        to: Option<HouseholdId>,
        reason: OwnershipReason,
    ) {
        let mut entry = ChronicleEntry::new(res.current_tick, ChronicleKind::HouseOwnershipChanged);
        entry.cause = house.0;
        entry.value = pack_ownership_change(from, to, reason);
        entry.settlement = res
            .colony
            .buildings
            .sites
            .get(&house.0)
            .and_then(|site| settlement_of(res, site.anchor));
        self.0.push(entry);
    }

    fn record(self, world: &World, res: &mut SimResources) {
        for entry in self.0 {
            super::chronicle_log::record(world, res, entry);
        }
    }
}

/// Earliest `BuildingComplete` tick caused by each site (the first physical completion).
fn completion_ticks(res: &SimResources) -> BTreeMap<BuildingSiteId, u64> {
    let mut out = BTreeMap::new();
    for (index, cause) in res.chronicle.causes() {
        let CauseRef::Building { site, .. } = cause else {
            continue;
        };
        let Some(entry) = res.chronicle.entry(index) else {
            continue;
        };
        if entry.kind == ChronicleKind::BuildingComplete {
            let tick = out.entry(site).or_insert(entry.tick);
            *tick = (*tick).min(entry.tick);
        }
    }
    out
}

/// Pre-format26 migration: house rows from the completion chronicle, then founding (rule 1).
/// Emits nothing.
pub fn migrate(world: &World, res: &mut SimResources) {
    if res.rules.colony.is_none() {
        return;
    }
    let ticks = completion_ticks(res);
    let mut rows = BTreeMap::new();
    for site in res.colony.buildings.sites.values() {
        if !is_dwelling(site) {
            continue;
        }
        let first = ticks.get(&site.id).copied();
        if first.is_some() || super::building_templates::is_intact(res, site) {
            rows.insert(
                HouseId(site.id),
                HouseHistory {
                    first_completed_tick: first,
                },
            );
        }
    }
    let state = &mut res.colony.buildings.housing;
    *state = HousingState {
        houses: rows,
        ..HousingState::default()
    };
    found(world, res);
}

/// Rule 1: every current family group becomes a household; each household takes the Dwelling
/// holding most of its members' assigned beds (ties lowest house); a house wanted twice goes to
/// the larger household, then the lower id. Emits nothing.
fn found(world: &World, res: &mut SimResources) {
    let people = people(world);
    let tick = res.current_tick;
    let mut ids: Vec<_> = res.settlements.keys().copied().collect();
    ids.sort_unstable();
    let mut groups = Vec::new();
    let mut seen = BTreeSet::new();
    for sid in ids {
        let Some(settlement) = res.settlements.get(&sid) else {
            continue;
        };
        let members: BTreeSet<_> = settlement
            .member_agents
            .iter()
            .copied()
            .filter(|id| people.contains_key(id) && !seen.contains(id))
            .collect();
        seen.extend(members.iter().copied());
        groups.extend(super::agent::households::households(world, &members));
    }
    let houses: Vec<HouseId> = res
        .colony
        .buildings
        .housing
        .houses
        .keys()
        .copied()
        .filter(|house| usable(res, *house))
        .collect();
    let mut wanted: BTreeMap<HouseId, Vec<(usize, HouseholdId)>> = BTreeMap::new();
    let mut slept_in: Vec<(HouseholdId, HouseId)> = Vec::new();
    let mut state = std::mem::take(&mut res.colony.buildings.housing);
    for group in groups {
        let members: BTreeSet<AgentId> = group.into_iter().collect();
        let Some(head) = head_of(&members, &people) else {
            continue;
        };
        let Some(id) = state.allocate() else {
            break;
        };
        let best = houses
            .iter()
            .filter_map(|house| {
                let site = res.colony.buildings.sites.get(&house.0)?;
                let count = members
                    .iter()
                    .filter(|m| res.bed_of.get(m).is_some_and(|bed| contains(site, *bed)))
                    .count();
                (count > 0).then_some((std::cmp::Reverse(count), *house))
            })
            .min();
        if let Some((_, house)) = best {
            wanted.entry(house).or_default().push((members.len(), id));
            slept_in.push((id, house));
        }
        state.households.insert(
            id,
            Household {
                id,
                head,
                members,
                formed_tick: tick,
                parent: None,
            },
        );
    }
    for (house, claims) in wanted {
        if let Some(&(_, owner)) = claims
            .iter()
            .min_by_key(|(size, id)| (std::cmp::Reverse(*size), *id))
        {
            state.ownership.insert(house, owner);
        }
    }
    // A group that sleeps in a house another group won stays there as its sub-household
    // (grown children, widowed elders), exactly as a later neolocal couple without a house.
    for (id, house) in slept_in {
        let Some(&owner) = state.ownership.get(&house) else {
            continue;
        };
        if owner != id && !owns_any(&state, id) {
            if let Some(household) = state.households.get_mut(&id) {
                household.parent = Some(owner);
            }
        }
    }
    state.founded = true;
    state.revision = state.revision.wrapping_add(1);
    res.colony.buildings.housing = state;
}

/// Rule 2 at a template completion: record the first completion; a new Dwelling goes to the
/// largest household of its settlement without a house (ties lowest id), else stays vacant.
pub fn record_completion(world: &World, res: &mut SimResources, id: BuildingSiteId) {
    if res.rules.colony.is_none() {
        return;
    }
    let Some(site) = res.colony.buildings.sites.get(&id) else {
        return;
    };
    if !is_dwelling(site) {
        return;
    }
    let house = HouseId(id);
    let settlement = settlement_of(res, site.anchor);
    let tick = res.current_tick;
    let state = &mut res.colony.buildings.housing;
    state.houses.entry(house).or_insert(HouseHistory {
        first_completed_tick: Some(tick),
    });
    if !state.founded || state.ownership.contains_key(&house) {
        return;
    }
    let homes = home_settlements(res);
    let people = people(world);
    let state = &res.colony.buildings.housing;
    let adults = adult_led(state, &people);
    let owner = state
        .households
        .values()
        .filter(|household| {
            adults.contains(&household.id)
                && household_settlement(household, &homes) == settlement
                && !owns_any(state, household.id)
        })
        .min_by_key(|household| (std::cmp::Reverse(household.members.len()), household.id))
        .map(|household| household.id);
    if let Some(owner) = owner {
        res.colony.buildings.housing.ownership.insert(house, owner);
        // Rule 4 holds at once: anyone else already assigned a bed inside loses it until the next
        // bed assignment (20:00 or a shelter change) instead of keeping an owned bed for hours.
        let allowed: BTreeSet<AgentId> = sleepers_of(&res.colony.buildings.housing, owner)
            .into_iter()
            .collect();
        if let Some(site) = res.colony.buildings.sites.get(&id) {
            let site = site.clone();
            res.bed_of
                .retain(|agent, bed| !contains(&site, *bed) || allowed.contains(agent));
        }
        let state = &mut res.colony.buildings.housing;
        state.revision = state.revision.wrapping_add(1);
        let mut events = Events(Vec::new());
        events.owner(res, house, None, Some(owner), OwnershipReason::Completed);
        events.record(world, res);
    }
}

/// Rules 3–5 and claims at the daily housing boundary.
pub fn reconcile(world: &World, res: &mut SimResources) {
    if res.rules.colony.is_none() {
        return;
    }
    reconcile_with(world, res, installed_or_bundled().yards().residence_rule);
}

/// [`reconcile`] under an explicit residence rule (data-only tests).
pub fn reconcile_with(world: &World, res: &mut SimResources, rule: ResidenceRule) {
    if !res.colony.buildings.housing.founded {
        found(world, res);
        return;
    }
    let people = people(world);
    let homes = home_settlements(res);
    let tick = res.current_tick;
    let mut events = Events(Vec::new());
    let mut state = std::mem::take(&mut res.colony.buildings.housing);
    let before = state.clone();
    // Members who died or left the world.
    for household in state.households.values_mut() {
        household.members.retain(|id| people.contains_key(id));
    }
    orphans(&mut state, &people, res, &mut events);
    dissolve_empty(&mut state, res, &mut events);
    // Newcomers (births, migrants): a minor joins a parent's household, anyone else starts one.
    for (&id, person) in &people {
        if !homes.contains_key(&id) || state.household_of(id).is_some() {
            continue;
        }
        let parent_home = (!person.adult)
            .then(|| {
                [person.kin.mother, person.kin.father]
                    .into_iter()
                    .flatten()
                    .find_map(|parent| state.household_of(parent))
            })
            .flatten();
        if let Some(home) = parent_home.and_then(|h| state.households.get_mut(&h)) {
            home.members.insert(id);
            continue;
        }
        let Some(new) = state.allocate() else {
            break;
        };
        let household = Household {
            id: new,
            head: id,
            members: BTreeSet::from([id]),
            formed_tick: tick,
            parent: None,
        };
        events.formed(tick, &household, homes.get(&id).copied());
        state.households.insert(new, household);
    }
    // Pairing (rule 3): mutual partners in different households.
    for (&a, person) in &people {
        let Some(b) = person.kin.partner.filter(|b| a < *b) else {
            continue;
        };
        let Some(other) = people.get(&b).filter(|p| p.kin.partner == Some(a)) else {
            continue;
        };
        let (Some(ha), Some(hb)) = (state.household_of(a), state.household_of(b)) else {
            continue;
        };
        if ha == hb {
            continue;
        }
        let (wife, husband) = if person.female && !other.female {
            (a, b)
        } else {
            (b, a)
        };
        let minors_of = |state: &HousingState, parent: AgentId| -> Vec<AgentId> {
            let Some(home) = state.household_of(parent) else {
                return Vec::new();
            };
            state
                .households
                .get(&home)
                .map_or_else(Vec::new, |household| {
                    household
                        .members
                        .iter()
                        .copied()
                        .filter(|id| {
                            people.get(id).is_some_and(|p| {
                                !p.adult
                                    && (p.kin.mother == Some(parent)
                                        || p.kin.father == Some(parent))
                            })
                        })
                        .collect()
                })
        };
        let mut movers: BTreeSet<AgentId> = BTreeSet::new();
        let target = match rule {
            ResidenceRule::Neolocal => {
                // The parent is the lower former household that owns a house (they sleep there
                // until they claim one), else the lower former household.
                let parent = [ha.min(hb), ha.max(hb)]
                    .into_iter()
                    .find(|h| owns_any(&state, *h))
                    .unwrap_or(ha.min(hb));
                movers.extend([a, b]);
                movers.extend(minors_of(&state, a));
                movers.extend(minors_of(&state, b));
                let Some(new) = state.allocate() else {
                    break;
                };
                state.households.insert(
                    new,
                    Household {
                        id: new,
                        head: a,
                        members: BTreeSet::new(),
                        formed_tick: tick,
                        parent: Some(parent),
                    },
                );
                new
            }
            ResidenceRule::Patrilocal | ResidenceRule::Matrilocal => {
                let (joiner, stays) = if rule == ResidenceRule::Patrilocal {
                    (wife, husband)
                } else {
                    (husband, wife)
                };
                movers.insert(joiner);
                movers.extend(minors_of(&state, joiner));
                let Some(target) = state.household_of(stays) else {
                    continue;
                };
                target
            }
        };
        for household in state.households.values_mut() {
            if household.id != target {
                household.members.retain(|id| !movers.contains(id));
            }
        }
        if let Some(home) = state.households.get_mut(&target) {
            home.members.extend(movers);
            if rule == ResidenceRule::Neolocal {
                home.head = head_of(&home.members, &people).unwrap_or(a);
                events.formed(tick, home, homes.get(&a).copied());
            }
        }
    }
    dissolve_empty(&mut state, res, &mut events);
    for household in state.households.values_mut() {
        if let Some(head) = head_of(&household.members, &people) {
            household.head = head;
        }
    }
    // Lost houses and households that left the house's settlement (rule 5).
    let owned: Vec<_> = state.ownership.iter().map(|(h, o)| (*h, *o)).collect();
    for (house, owner) in owned {
        let reason = if !standing_with(res, &state, house) {
            Some(OwnershipReason::Lost)
        } else {
            let settlement = res
                .colony
                .buildings
                .sites
                .get(&house.0)
                .and_then(|site| settlement_of(res, site.anchor));
            let present = state.households.get(&owner).is_some_and(|household| {
                household
                    .members
                    .iter()
                    .any(|m| homes.get(m).copied() == settlement && settlement.is_some())
            });
            (!present).then_some(OwnershipReason::Departed)
        };
        if let Some(reason) = reason {
            state.ownership.remove(&house);
            events.owner(res, house, Some(owner), None, reason);
        }
    }
    let adults = adult_led(&state, &people);
    claims(&mut state, res, &homes, &adults, &mut events);
    if state != before {
        state.revision = state.revision.wrapping_add(1);
    }
    res.colony.buildings.housing = state;
    events.record(world, res);
}

fn standing_with(res: &SimResources, state: &HousingState, house: HouseId) -> bool {
    usable(res, house)
        || state.houses.contains_key(&house)
            && res
                .colony
                .buildings
                .sites
                .get(&house.0)
                .is_some_and(|site| site.status == BuildingStatus::Building)
}

/// Rule 5: a household with members but no living adult hands its minors to the closest kin
/// household (a parent's parent, then a sibling; lowest id) and its house to that heir if the heir
/// owns none (else the house becomes vacant). Without kin the minors keep their household.
fn orphans(
    state: &mut HousingState,
    people: &BTreeMap<AgentId, Person>,
    res: &SimResources,
    events: &mut Events,
) {
    let ids: Vec<_> = state.households.keys().copied().collect();
    for id in ids {
        let Some(household) = state.households.get(&id) else {
            continue;
        };
        if household.members.is_empty()
            || household
                .members
                .iter()
                .any(|m| people.get(m).is_some_and(|p| p.adult))
        {
            continue;
        }
        let minors: Vec<_> = household.members.iter().copied().collect();
        let lookup = |agent: Option<AgentId>| -> Option<HouseholdId> {
            state.household_of(agent?).filter(|h| *h != id)
        };
        let grandparents = minors
            .iter()
            .filter_map(|m| people.get(m))
            .flat_map(|p| [p.kin.mother, p.kin.father])
            .flatten()
            .flat_map(|parent| {
                // A dead parent's own parents survive in the chronicle's person record.
                people.get(&parent).map_or_else(
                    || {
                        res.chronicle
                            .person(parent)
                            .map_or([None, None], |p| [p.mother, p.father])
                    },
                    |p| [p.kin.mother, p.kin.father],
                )
            })
            .filter_map(lookup)
            .min();
        let heir = grandparents.or_else(|| {
            let parents: BTreeSet<AgentId> = minors
                .iter()
                .filter_map(|m| people.get(m))
                .flat_map(|p| [p.kin.mother, p.kin.father])
                .flatten()
                .collect();
            people
                .iter()
                .filter(|(sibling, p)| {
                    !minors.contains(*sibling)
                        && [p.kin.mother, p.kin.father]
                            .into_iter()
                            .flatten()
                            .any(|parent| parents.contains(&parent))
                })
                .filter_map(|(sibling, _)| lookup(Some(*sibling)))
                .min()
        });
        let Some(heir) = heir else {
            // No kin household: the minors keep their household, the house becomes vacant.
            for house in state.owned_by(id) {
                state.ownership.remove(&house);
                events.owner(res, house, Some(id), None, OwnershipReason::Vacated);
            }
            continue;
        };
        if let Some(target) = state.households.get_mut(&heir) {
            target.members.extend(minors.iter().copied());
        }
        if let Some(household) = state.households.get_mut(&id) {
            household.members.clear();
        }
        let heir_owns = state.ownership.values().any(|owner| *owner == heir);
        for (index, house) in state.owned_by(id).into_iter().enumerate() {
            if index == 0 && !heir_owns {
                state.ownership.insert(house, heir);
                events.owner(res, house, Some(id), Some(heir), OwnershipReason::Inherited);
            } else {
                state.ownership.remove(&house);
                events.owner(res, house, Some(id), None, OwnershipReason::Vacated);
            }
        }
    }
}

/// Households without living members end; their houses become vacant.
fn dissolve_empty(state: &mut HousingState, res: &SimResources, events: &mut Events) {
    let empty: Vec<_> = state
        .households
        .values()
        .filter(|household| household.members.is_empty())
        .map(|household| household.id)
        .collect();
    for id in empty {
        for house in state.owned_by(id) {
            state.ownership.remove(&house);
            events.owner(res, house, Some(id), None, OwnershipReason::Vacated);
        }
        state.households.remove(&id);
    }
}

/// Households without a house claim the nearest vacant usable Dwelling of their settlement
/// (largest household first, ties lowest id; nearest to the parent's house, else the formation
/// tile; ties lowest house id).
fn claims(
    state: &mut HousingState,
    res: &SimResources,
    homes: &BTreeMap<AgentId, SettlementId>,
    adults: &BTreeSet<HouseholdId>,
    events: &mut Events,
) {
    let mut vacant: Vec<(HouseId, Tile, Option<SettlementId>)> = state
        .houses
        .keys()
        .copied()
        .filter(|house| !state.ownership.contains_key(house) && usable(res, *house))
        .filter_map(|house| {
            let site = res.colony.buildings.sites.get(&house.0)?;
            Some((house, site.anchor, settlement_of(res, site.anchor)))
        })
        .collect();
    if vacant.is_empty() {
        return;
    }
    let mut homeless: Vec<_> = state
        .households
        .values()
        .filter(|household| adults.contains(&household.id) && !owns_any(state, household.id))
        .map(|household| {
            (
                std::cmp::Reverse(household.members.len()),
                household.id,
                household.parent,
                household_settlement(household, homes),
            )
        })
        .collect();
    homeless.sort();
    for (_, id, parent, settlement) in homeless {
        let Some(settlement) = settlement else {
            continue;
        };
        let reference = parent
            .and_then(|parent| state.owned_by(parent).first().copied())
            .and_then(|house| res.colony.buildings.sites.get(&house.0))
            .map(|site| site.anchor)
            .or_else(|| res.settlements.get(&settlement).map(|s| s.formation_tile));
        let Some(reference) = reference else {
            continue;
        };
        let Some(index) = vacant
            .iter()
            .enumerate()
            .filter(|(_, (_, _, owner))| *owner == Some(settlement))
            .min_by_key(|(_, (house, anchor, _))| {
                (
                    anchor
                        .0
                        .abs_diff(reference.0)
                        .max(anchor.1.abs_diff(reference.1)),
                    *house,
                )
            })
            .map(|(index, _)| index)
        else {
            continue;
        };
        let (house, ..) = vacant.remove(index);
        state.ownership.insert(house, id);
        events.owner(res, house, None, Some(id), OwnershipReason::Claimed);
        if vacant.is_empty() {
            break;
        }
    }
}

/// Rule 4 inputs: owned bed tiles → owner, and each owner's sleepers in order (its members by
/// id, then members of its houseless sub-households by household id, then id).
pub(crate) struct BedClaims {
    /// Bed tile inside a standing owned house → owning household.
    pub owner_of: BTreeMap<Tile, HouseholdId>,
    /// Who may sleep in each owner's beds.
    pub sleepers: BTreeMap<HouseholdId, Vec<AgentId>>,
}

/// Exclusive-bed claims for the bed assignment; `None` before founding or outside a colony.
pub(crate) fn bed_claims(res: &SimResources, beds: &[Tile]) -> Option<BedClaims> {
    let state = &res.colony.buildings.housing;
    if res.rules.colony.is_none() || !state.founded {
        return None;
    }
    let mut owner_of = BTreeMap::new();
    for (house, owner) in &state.ownership {
        if !standing(res, *house) {
            continue;
        }
        let Some(site) = res.colony.buildings.sites.get(&house.0) else {
            continue;
        };
        for bed in beds.iter().filter(|bed| contains(site, **bed)) {
            owner_of.entry(*bed).or_insert(*owner);
        }
    }
    let sleepers = owner_of
        .values()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|owner| (owner, sleepers_of(state, owner)))
        .collect();
    Some(BedClaims { owner_of, sleepers })
}

/// Rule 6, in beds: per settlement, the members of adult-led households without a house, except
/// the households the settlement's vacant usable houses will take at the next boundary (largest
/// first, as the claims order them). One settlement's vacancy never hides another's demand.
pub fn housing_demand(world: &World, res: &SimResources) -> u32 {
    let state = &res.colony.buildings.housing;
    if res.rules.colony.is_none() || !state.founded {
        return 0;
    }
    let people = people(world);
    let homes = home_settlements(res);
    let adults = adult_led(state, &people);
    let mut homeless: BTreeMap<SettlementId, Vec<(std::cmp::Reverse<usize>, HouseholdId)>> =
        BTreeMap::new();
    for household in state.households.values() {
        if !adults.contains(&household.id) || owns_any(state, household.id) {
            continue;
        }
        if let Some(sid) = household_settlement(household, &homes) {
            homeless
                .entry(sid)
                .or_default()
                .push((std::cmp::Reverse(household.members.len()), household.id));
        }
    }
    let mut vacant: BTreeMap<SettlementId, usize> = BTreeMap::new();
    for house in state.houses.keys() {
        if state.ownership.contains_key(house) || !usable(res, *house) {
            continue;
        }
        if let Some(sid) = res
            .colony
            .buildings
            .sites
            .get(&house.0)
            .and_then(|site| settlement_of(res, site.anchor))
        {
            *vacant.entry(sid).or_default() += 1;
        }
    }
    let beds: usize = homeless
        .iter_mut()
        .map(|(sid, list)| {
            list.sort();
            list.iter()
                .skip(vacant.get(sid).copied().unwrap_or(0))
                .map(|(size, _)| size.0)
                .sum::<usize>()
        })
        .sum();
    u32::try_from(beds).unwrap_or(u32::MAX)
}

fn row_major(mut cells: Vec<Tile>) -> Vec<Tile> {
    cells.sort_unstable_by_key(|&(x, y)| (y, x));
    cells.dedup();
    cells
}

/// Each living settlement member's current family group key (lowest AgentId of its
/// `households()` group), one `households()` pass per settlement.
fn family_groups(world: &World, res: &SimResources) -> BTreeMap<AgentId, AgentId> {
    let mut ids: Vec<_> = res.settlements.keys().copied().collect();
    ids.sort_unstable();
    let mut out = BTreeMap::new();
    for sid in ids {
        let Some(settlement) = res.settlements.get(&sid) else {
            continue;
        };
        let members: BTreeSet<AgentId> = settlement
            .member_agents
            .iter()
            .copied()
            .filter(|id| !out.contains_key(id))
            .collect();
        for group in super::agent::households::households(world, &members) {
            if let Some(&key) = group.first() {
                for member in group {
                    out.insert(member, key);
                }
            }
        }
    }
    out
}

/// The derived house read model, in house id order. Pure.
pub fn house_views(world: &World, res: &SimResources) -> Vec<HouseView> {
    let catalog = installed_or_bundled();
    let state = &res.colony.buildings.housing;
    if state.houses.is_empty() {
        return Vec::new();
    }
    let groups = family_groups(world, res);
    let hut_beds = res.rules.colony.map_or(0, |rule| rule.hut_beds);
    let space = res.rules.colony.is_some_and(|rule| rule.space.is_some());
    let mut out = Vec::new();
    for (&house, history) in &state.houses {
        let Some(site) = res.colony.buildings.sites.get(&house.0) else {
            continue;
        };
        let template = catalog.building_templates().get(&site.template);
        let status = if super::building_templates::is_intact(res, site) {
            HouseStatus::Complete
        } else if site.status == BuildingStatus::Building {
            HouseStatus::Upgrading
        } else {
            HouseStatus::Damaged
        };
        let settlement = settlement_of(res, site.anchor);
        let formation = settlement
            .and_then(|sid| res.settlements.get(&sid))
            .map(|s| s.formation_tile);
        let reachable =
            |tile: Tile| !space || formation.is_some_and(|from| res.nav.reachable(from, tile));
        let completed_cells = row_major(
            site.pieces
                .iter()
                .filter(|piece| piece.phase == PiecePhase::Complete)
                .flat_map(|piece| {
                    (0..piece.size.1).flat_map(move |dy| {
                        (0..piece.size.0).map(move |dx| (piece.tile.0 + dx, piece.tile.1 + dy))
                    })
                })
                .collect(),
        );
        let inside = |tile: &Tile| {
            tile.0 > site.anchor.0
                && tile.1 > site.anchor.1
                && tile.0 < (site.anchor.0 + site.footprint.0).saturating_sub(1)
                && tile.1 < (site.anchor.1 + site.footprint.1).saturating_sub(1)
        };
        let rooms: Vec<_> = res
            .colony
            .rooms
            .values()
            .filter(|room| !room.tiles.is_empty() && room.tiles.iter().all(inside))
            .collect();
        let roof_cells = if status == HouseStatus::Complete {
            row_major(
                rooms
                    .iter()
                    .filter(|room| room.roofed)
                    .flat_map(|room| room.tiles.iter().copied())
                    .collect(),
            )
        } else {
            Vec::new()
        };
        let mut bed_refs: Vec<BedRef> = res
            .colony
            .beds
            .keys()
            .filter(|tile| contains(site, **tile))
            .map(|&tile| BedRef {
                tile,
                slots: 1,
                reachable: reachable(tile),
            })
            .chain(
                res.building_kinds
                    .iter()
                    .filter(|(tile, kind)| **kind == BuildingKind::Hut && contains(site, **tile))
                    .map(|(&tile, _)| BedRef {
                        tile,
                        slots: hut_beds,
                        reachable: reachable(tile),
                    }),
            )
            .collect();
        bed_refs.sort_unstable_by_key(|bed| (bed.tile.1, bed.tile.0));
        let hearth_refs = row_major(
            res.building_kinds
                .iter()
                .filter(|(tile, kind)| **kind == BuildingKind::Hearth && contains(site, **tile))
                .map(|(&tile, _)| tile)
                .collect(),
        );
        let progress = res.colony.buildings.house_growth.get(&house.0);
        let residents: Vec<AgentId> = res
            .bed_of
            .iter()
            .filter(|(_, bed)| bed_refs.iter().any(|r| r.tile == **bed))
            .map(|(id, _)| *id)
            .collect();
        out.push(HouseView {
            id: house,
            settlement,
            template: site.template.clone(),
            era: template.map(|t| t.era.clone()).unwrap_or_default(),
            tier: template.map_or(0, |t| t.tier),
            yard_stage: progress.map_or(0, |p| p.stage),
            occupied_days: progress.map_or(0, |p| p.occupied_days),
            status,
            first_completed_tick: history.first_completed_tick,
            completed_cells,
            roof_cells,
            room_ids: rooms.iter().map(|room| room.id).collect(),
            capacity: bed_refs
                .iter()
                .filter(|bed| bed.reachable)
                .map(|bed| bed.slots)
                .sum(),
            household_groups: residents
                .iter()
                .map(|id| groups.get(id).copied().unwrap_or(*id))
                .collect(),
            primary_household: {
                let mut counts: BTreeMap<AgentId, usize> = BTreeMap::new();
                for id in &residents {
                    *counts
                        .entry(groups.get(id).copied().unwrap_or(*id))
                        .or_default() += 1;
                }
                counts
                    .into_iter()
                    .min_by_key(|(key, count)| (std::cmp::Reverse(*count), *key))
                    .map(|(key, _)| key)
            },
            resident_ids: residents,
            bed_refs,
            hearth_refs,
            owner_household: state.ownership.get(&house).copied(),
        });
    }
    out
}

#[cfg(test)]
#[path = "housing_tests.rs"]
mod tests;
