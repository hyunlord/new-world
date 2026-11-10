//! C1-b: one fixed-source, ordinary-command livestock chain, with physical prerequisites.
//! Evidence is external and append-only. No intermediate items, traits or knowledge are injected.
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use sim_core::colony::building_site::Tile;
use sim_core::colony::content::{installed_or_bundled, AnimalKind};
use sim_core::colony::knowledge::{self, Source};
use sim_core::colony::livestock::{current_age, LivestockEvent};
use sim_core::colony::livestock_demographics::wild_demographics;
use sim_core::colony::people::{Drafted, Fleeing};
use sim_core::colony::{
    BillMode, BlueprintKind, Command, ItemKind, Job, JobKind, JobStage, NodeKind, RecipeKind,
    Skills, WorkPriorities, WorkType,
};
use sim_core::components::{Agent, AgentId, BodyHealth, Elder, Hunger, Sleep, Temperament, Thirst};
use sim_core::material::TerrainType;
use sim_engine::SimEngine;
use sim_systems::commands::{check_blueprint, submit};
use sim_systems::runtime::jobs::{invariants, items::chebyshev};
use sim_systems::runtime::knowledge::{knowledge_of, people};
use sim_systems::runtime::livestock::pens::pen_layout;

fn item(name: &str) -> ItemKind {
    ItemKind::from_name(name).expect("authored chain item")
}
fn node(name: &str) -> u8 {
    knowledge::defs()
        .id_of(name)
        .expect("authored chain knowledge")
}
fn camp(e: &SimEngine) -> Tile {
    sim_systems::worldgen::colony::hearth(&e.resources).expect("ordinary camp")
}
fn fresh(path: &Path, bytes: &[u8]) {
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .expect("fresh external evidence file")
        .write_all(bytes)
        .expect("evidence bytes");
}
fn document(path: &Path, value: Value) {
    fresh(
        path,
        &serde_json::to_vec_pretty(&value).expect("evidence JSON"),
    );
}
fn output() -> PathBuf {
    let base = std::env::var_os("WORLDSIM_C1_SCENE_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("worldsim-c1-scene"));
    assert!(base.is_absolute(), "external absolute scene path");
    std::fs::create_dir_all(&base).expect("external scene root");
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("repository root");
    assert!(
        !base
            .canonicalize()
            .expect("external root identity")
            .starts_with(repository.canonicalize().expect("repository identity")),
        "raw scene evidence stays outside repository"
    );
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("artifact clock")
        .as_nanos();
    let out = base.join(format!("chain-{}-{stamp}", std::process::id()));
    std::fs::create_dir(&out).expect("fresh process/run directory");
    println!("C1_CHAIN_OUT {}", out.display());
    out
}

/// This is the only fixture mutation. The identical initializer is used for replay.
fn initialize(seed: u64) -> (SimEngine, Value) {
    let mut e = sim_test::colony_engine(seed);
    let initial: BTreeMap<_, _> = people(&e.world, &e.resources)
        .into_iter()
        .map(|p| (p.id, (p.entity, knowledge_of(&e.world, p.entity))))
        .collect();
    let founders: Vec<_> = people(&e.world, &e.resources)
        .into_iter()
        .filter(|p| {
            p.adult
                && e.world
                    .get::<&BodyHealth>(p.entity)
                    .is_ok_and(|h| !h.is_dead())
        })
        .take(2)
        .map(|p| p.id)
        .collect();
    assert_eq!(
        founders.len(),
        2,
        "two actual adult founders; never change age/Child"
    );
    let taming = node("animal_taming");
    for id in &founders {
        let (entity, before) = &initial[id];
        assert!(
            before.get(taming).is_none(),
            "taming is not normally seeded"
        );
        let mut allowed = before.clone();
        allowed.set(taming, 1_000, Source::Seeded, 0);
        e.world
            .insert_one(*entity, allowed)
            .expect("only allowed taming fixture");
    }
    for (&id, &(entity, ref before)) in &initial {
        let after = knowledge_of(&e.world, entity);
        let retained: Vec<_> = after
            .entries()
            .iter()
            .copied()
            .filter(|k| k.id() != taming)
            .collect();
        assert_eq!(
            retained,
            before.entries(),
            "all other knowledge unchanged for {id}"
        );
        if founders.contains(&id) {
            assert_eq!(after.proficiency(taming), 1_000);
            assert_eq!(after.entries().len(), before.entries().len() + 1);
        } else {
            assert_eq!(&after, before, "nonrecipient unchanged");
        }
    }
    let proof = json!({"founders": founders, "allowed_node": taming, "only_change": "animal_taming=1000",
        "before": initial.iter().map(|(id, (_, k))| json!({"id":id,"knowledge":k})).collect::<Vec<_>>(),
        "all_other_knowledge_exact": true});
    (e, proof)
}

fn source_rows(e: &SimEngine, goat: AnimalKind) -> Vec<Value> {
    let policy = goat.definition().tameable.as_ref().expect("goat policy");
    e.resources.colony.wildlife.animals.values().filter(|a| a.kind == goat).map(|a| {
        let profile = wild_demographics(e.resources.world_seed, a, policy, e.resources.current_tick, e.resources.ticks_per_day);
        let initial = wild_demographics(e.resources.world_seed, a, policy, a.born_tick, e.resources.ticks_per_day);
        let expiry = a.born_tick.saturating_add(u64::from(policy.kid_age_days.saturating_sub(initial.age_days)) * e.resources.ticks_per_day);
        json!({"id":a.id,"born_tick":a.born_tick,"pos":a.pos,"age":profile.age_days,"initial_age":initial.age_days,"female":profile.sex.female,
            "last_age_tick":profile.last_age_tick,"kid_expiry_tick":expiry,"distance":chebyshev(a.pos,camp(e)),
            "reachable":e.resources.nav.reachable(camp(e),a.pos),"kid":profile.age_days<policy.kid_age_days})
    }).collect()
}

/// Admission precedes every full-chain outcome. Never select another seed after a chain failure.
fn admit(out: &Path) -> (SimEngine, [u32; 2], usize, Value) {
    let goat = installed_or_bundled()
        .animal_by_name("wild_goat")
        .expect("goat");
    let policy = goat.definition().tameable.as_ref().expect("goat policy");
    for seed in 0..64 {
        let (mut e, initialization) = initialize(seed);
        let centre = camp(&e);
        let region = e.resources.nav.region_of(centre);
        let climate = sim_core::variety::WorldVariety::draw(seed).climate;
        let mut habitat = 0;
        for y in 0..e.resources.tile_grid.height {
            for x in 0..e.resources.tile_grid.width {
                let t = (x, y);
                let i = e.resources.tile_grid.idx(x, y);
                if chebyshev(t, centre) <= 40
                    && e.resources.nav.passable(t)
                    && e.resources.nav.reaches(region, t)
                    && !e.resources.colony.trees.contains_key(&t)
                    && e.resources.tile_grid.terrain_type[i] == TerrainType::Plain
                    && e.resources.tile_grid.elevation[i] >= 160
                    && goat.definition().origin.admits(
                        climate,
                        TerrainType::Plain,
                        e.resources.tile_grid.elevation[i],
                    )
                {
                    habitat += 1;
                }
            }
        }
        assert!(
            submit(
                &mut e,
                Command::SetSteward {
                    on: false,
                    scope: Default::default()
                }
            )
            .accepted
        );
        for _ in 0..12 {
            e.tick();
        }
        let rows = source_rows(&e, goat);
        let mut kids: Vec<_> = e
            .resources
            .colony
            .wildlife
            .animals
            .values()
            .filter(|a| a.kind == goat)
            .filter_map(|a| {
                let p = wild_demographics(
                    seed,
                    a,
                    policy,
                    e.resources.current_tick,
                    e.resources.ticks_per_day,
                );
                (p.age_days < policy.kid_age_days
                    && chebyshev(a.pos, centre) <= policy.capture_range_tiles
                    && e.resources.nav.reachable(centre, a.pos))
                .then_some((p.age_days, a.id, p.sex.female))
            })
            .collect();
        // Longest remaining window first, then immutable ID; no admission age-slack threshold.
        kids.sort_by_key(|&(age, id, _)| (age, id));
        let pair = kids.first().and_then(|first| {
            kids.iter()
                .find(|a| a.2 != first.2)
                .map(|other| [first.1, other.1])
        });
        let clay = sim_systems::runtime::knowledge::discover::clay_bank_near(&e.resources, 24);
        let accepted = habitat >= 20 && pair.is_some() && clay;
        document(
            &out.join(format!("seed-admission-{seed:02}.json")),
            json!({"seed":seed,"tick":e.current_tick(),
            "highland_open_reachable_within40":habitat,"clay_bank_near24":clay,"sources":rows,"pair":pair,
            "accepted":accepted,"initializer":initialization,"predicate":"first ascending0..63: habitat>=20, actual opposite-sex reachable kids within capture range, ClayBankNear24"}),
        );
        println!("C1_ADMISSION seed={seed} habitat={habitat} clay={clay} pair={pair:?} accepted={accepted}");
        if accepted {
            let pair = pair.expect("admitted pair");
            // Preserve run1 admission and seed; D15 permits ordinary natural-source succession.
            assert_eq!(
                (seed, pair),
                (5, [10, 11]),
                "fixed first-admission source changed; stop rather than select another world"
            );
            document(
                &out.join("source-binding.json"),
                json!({"seed":seed,"pair":pair,
                "first_admission_evidence":"/Users/rexxa/work/c1-data/scene/chain-41172-1794161240877036000/seed-admission-05.json",
                "binding":"initial admission counterexample retained; actual capture sources follow prospective D15 and sequential D17"}),
            );
            return (e, pair, habitat, initialization);
        }
    }
    panic!("no predeclared source admission in seeds0..63; preserve evidence, do not tune");
}

struct Scene {
    e: SimEngine,
    twin: Option<SimEngine>,
    out: PathBuf,
    initial_pair: [u32; 2],
    capture_sources: Vec<u32>,
    hunting: bool,
    habitat: usize,
    initialization: Value,
    pen: Option<u32>,
    pottery_tile: Option<Tile>,
    clay_ordered: bool,
    forming_ordered: bool,
    next_firing: u64,
}
impl Scene {
    fn send(&mut self, command: Command) {
        let actual = submit(&mut self.e, command.clone());
        if let Some(twin) = self.twin.as_mut() {
            assert_eq!(submit(twin, command.clone()), actual, "twin command");
        }
        assert!(actual.accepted, "ordinary command {command:?}: {actual:?}");
    }
    fn limit(&self) -> u64 {
        60 * self.e.resources.ticks_per_day
    }
    fn known(&self, name: &str) -> bool {
        sim_systems::runtime::knowledge::anyone_practises(&self.e.world, knowledge::bit(node(name)))
    }
    fn candidates(&self) -> Vec<Tile> {
        let c = camp(&self.e);
        let mut tiles = Vec::new();
        for y in 2..self.e.resources.tile_grid.height.saturating_sub(6) {
            for x in 2..self.e.resources.tile_grid.width.saturating_sub(6) {
                if chebyshev(c, (x, y)) <= 24 {
                    tiles.push((x, y));
                }
            }
        }
        tiles.sort_by_key(|&t| (chebyshev(c, t), t.1, t.0));
        tiles
    }
    fn place_shop(&mut self, name: &str) -> Tile {
        let kind = BlueprintKind::from_name(name).expect("authored workshop");
        let (x, y) = self
            .candidates()
            .into_iter()
            .find(|&(x, y)| check_blueprint(&self.e.resources, kind, x, y).is_ok())
            .expect("physical workshop footprint");
        self.send(Command::PlaceBlueprint { kind, x, y });
        (x, y)
    }
    fn shop(&self, name: &str, tile: Option<Tile>) -> Option<u32> {
        let kind = BlueprintKind::from_name(name).expect("workshop kind");
        self.e
            .resources
            .colony
            .workshops
            .values()
            .find(|s| s.kind == kind && tile.is_none_or(|t| t == s.top_left))
            .map(|s| s.id)
    }
    fn bill(&mut self, workshop: u32, name: &str, count: u16) {
        self.send(Command::AddBill {
            workshop,
            recipe: RecipeKind::from_name(name).expect("physical recipe"),
            mode: BillMode::Make(count),
        });
    }
    fn build_bench(&mut self) -> u32 {
        let tile = self.place_shop("wood_bench");
        self.until("actual wood bench", |e| {
            e.resources
                .colony
                .workshops
                .values()
                .any(|shop| shop.top_left == tile)
        });
        let id = self
            .shop("wood_bench", Some(tile))
            .expect("built wood bench");
        let automatic: Vec<_> = self.e.resources.colony.workshops[&id]
            .bills
            .iter()
            .filter(|bill| bill.auto)
            .map(|bill| bill.id)
            .collect();
        for bill in automatic {
            self.send(Command::RemoveBill { workshop: id, bill });
        }
        id
    }
    /// Ordinary teaching joins real prerequisites on the actual curious adults, never grants them.
    fn lessons(&mut self) {
        let mut adults: Vec<_> = people(&self.e.world, &self.e.resources)
            .into_iter()
            .filter(|p| p.adult)
            .collect();
        let rule = sim_systems::runtime::knowledge::rule(&self.e.resources)
            .expect("normal knowledge rule");
        adults.sort_by_key(|p| (std::cmp::Reverse(p.ns), p.id));
        let top = (adults.len() as u64 * u64::from(rule.discovery_top_permille))
            .div_ceil(1000)
            .max(1) as usize;
        let curious: Vec<_> = adults
            .iter()
            .enumerate()
            .filter(|(i, p)| *i < top || p.ns >= rule.discovery_ns)
            .map(|(_, p)| *p)
            .collect();
        // One outstanding lesson per learner avoids script-created overlapping sessions.
        for learner in curious {
            if self
                .e
                .resources
                .colony
                .teach_orders
                .iter()
                .any(|o| o.learner == learner.id)
            {
                continue;
            }
            for name in [
                "foraging",
                "timber_wall",
                "digging",
                "fire_keeping",
                "cordage",
                "clay_digging",
                "clay_prep",
                "pot_forming",
                "pot_firing",
                "animal_taming",
            ] {
                let id = node(name);
                if knowledge_of(&self.e.world, learner.entity).practises(id) {
                    continue;
                }
                let teacher = people(&self.e.world, &self.e.resources)
                    .into_iter()
                    .filter(|p| p.adult && p.id != learner.id)
                    .filter(|p| knowledge_of(&self.e.world, p.entity).practises(id))
                    .max_by_key(|p| {
                        (
                            knowledge_of(&self.e.world, p.entity).proficiency(id),
                            std::cmp::Reverse(p.id),
                        )
                    });
                if let Some(teacher) = teacher {
                    self.send(Command::OrderTeach {
                        teacher: teacher.id,
                        learner: learner.id,
                        knowledge: id,
                    });
                    break;
                }
            }
        }
    }
    /// Pottery proceeds concurrently with the physical capture preparation and biological waiting.
    fn pottery(&mut self) {
        if !self.clay_ordered && self.known("clay_digging") {
            let mut banks: Vec<_> = self
                .e
                .resources
                .colony
                .nodes
                .iter()
                .filter(|(t, n)| {
                    n.kind == NodeKind::ClayBank
                        && sim_systems::commands::camp_reaches(&self.e.resources, **t)
                })
                .map(|(&t, _)| t)
                .collect();
            banks.sort_by_key(|&t| (chebyshev(t, camp(&self.e)), t.1, t.0));
            assert!(
                !banks.is_empty(),
                "admitted clay must be physically obtainable"
            );
            for (x, y) in banks.into_iter().take(2) {
                self.send(Command::DesignateClay {
                    x0: x,
                    y0: y,
                    x1: x,
                    y1: y,
                });
            }
            self.clay_ordered = true;
        }
        if self.pottery_tile.is_none() && self.known("pot_forming") {
            self.pottery_tile = Some(self.place_shop("pottery_bench"));
        }
        if !self.forming_ordered {
            if let Some(shop) = self
                .pottery_tile
                .and_then(|t| self.shop("pottery_bench", Some(t)))
            {
                self.bill(shop, "form_pot", 24);
                self.forming_ordered = true;
            }
        }
        if self.known("pot_firing")
            && self.e.current_tick() >= self.next_firing
            && self.e.resources.colony.ground_total(item("raw_pot")) > 0
        {
            let hearth = self
                .shop("hearth_fire", None)
                .expect("actual worldgen firing hearth");
            let fire = RecipeKind::from_name("fire_pot").expect("fire pot");
            if !self.e.resources.colony.workshops[&hearth]
                .bills
                .iter()
                .any(|b| b.recipe == fire && b.remaining() != Some(0))
            {
                self.bill(hearth, "fire_pot", 1);
                self.next_firing = self.e.current_tick() + 4 * self.e.resources.ticks_per_day;
            }
        }
    }
    fn tick(&mut self) {
        assert!(
            self.e.current_tick() < self.limit(),
            "absolute60day deadline"
        );
        if self.hunting && self.e.resources.livestock.animals.len() < 2 {
            let goat = installed_or_bundled()
                .animal_by_name("wild_goat")
                .expect("goat");
            let policy = goat.definition().tameable.as_ref().expect("goat policy");
            let mature: Vec<_> = self
                .e
                .resources
                .colony
                .wildlife
                .animals
                .values()
                .filter(|a| {
                    a.kind == goat && !self.e.resources.colony.wildlife.marks.contains(&a.id)
                })
                .filter(|a| {
                    wild_demographics(
                        self.e.resources.world_seed,
                        a,
                        policy,
                        self.e.current_tick(),
                        self.e.resources.ticks_per_day,
                    )
                    .age_days
                        >= policy.kid_age_days
                })
                .map(|a| a.id)
                .collect();
            for animal in mature {
                self.send(Command::DesignateHunt { animal });
            }
        }
        if self.e.current_tick().is_multiple_of(240) {
            self.lessons();
            self.pottery();
        }
        self.e.tick();
        if let Some(twin) = self.twin.as_mut() {
            twin.tick();
        }
        invariants::check(&self.e.world, &self.e.resources)
            .expect("real ledger/reservation invariants");
        if self
            .e
            .current_tick()
            .is_multiple_of(self.e.resources.ticks_per_day)
        {
            self.equal();
            let day = self.e.current_tick() / self.e.resources.ticks_per_day;
            document(
                &self.out.join(format!("progress-day-{day:02}.json")),
                self.manifest("in-progress"),
            );
            println!(
                "C1_CHAIN_DAY {day} livestock={} knowledge={:?}",
                self.e.resources.livestock.animals.len(),
                self.e.resources.colony.teach_orders
            );
        }
    }
    fn until(&mut self, label: &str, ready: impl Fn(&SimEngine) -> bool) {
        println!("C1_WAIT {label} tick={}", self.e.current_tick());
        while self.e.current_tick() < self.limit() && !ready(&self.e) {
            if !self.capture_sources.is_empty() {
                let expired = self.capture_sources.iter().any(|id| {
                    let Some(a) = self.e.resources.colony.wildlife.animals.get(id) else {
                        return false; // A successfully captured frozen target is no longer wild.
                    };
                    let policy = a
                        .kind
                        .definition()
                        .tameable
                        .as_ref()
                        .expect("source policy");
                    wild_demographics(
                        self.e.resources.world_seed,
                        a,
                        policy,
                        self.e.current_tick(),
                        self.e.resources.ticks_per_day,
                    )
                    .age_days
                        >= policy.kid_age_days
                });
                if expired {
                    let bytes =
                        sim_systems::save::save(&mut self.e).expect("actual failed world save");
                    fresh(&self.out.join("expired-preparation.wsave"), &bytes);
                    document(
                        &self.out.join("expired-preparation.json"),
                        self.manifest(label),
                    );
                    panic!(
                        "fixed source expired during {label}; evidence={}",
                        self.out.display()
                    );
                }
            }
            self.tick();
        }
        if !ready(&self.e) {
            let bytes = sim_systems::save::save(&mut self.e).expect("actual deadline world save");
            fresh(&self.out.join("failed-deadline.wsave"), &bytes);
            document(&self.out.join("failed-deadline.json"), self.manifest(label));
        }
        assert!(
            ready(&self.e),
            "{label} failed at absolute60days; evidence={}",
            self.out.display()
        );
    }
    fn equal(&mut self) {
        if let Some(twin) = self.twin.as_mut() {
            invariants::check(&twin.world, &twin.resources).expect("loaded twin invariants");
            assert_eq!(
                sim_test::probe::fingerprint(&self.e),
                sim_test::probe::fingerprint(twin)
            );
            assert_eq!(
                sim_test::probe::colony::colony_fingerprint(&self.e),
                sim_test::probe::colony::colony_fingerprint(twin)
            );
            assert_eq!(
                sim_systems::save::save(&mut self.e).expect("original bytes"),
                sim_systems::save::save(twin).expect("twin bytes"),
                "future command twins byte exact"
            );
        }
    }
    fn manifest(&self, label: &str) -> Value {
        let goat = installed_or_bundled()
            .animal_by_name("wild_goat")
            .expect("goat");
        let event_index = |code| {
            self.e
                .resources
                .livestock
                .witnesses
                .keys()
                .copied()
                .filter(|&index| {
                    self.e.resources.chronicle.entries()[index as usize].kind as u8 == code
                })
                .max()
        };
        let animals: Vec<_> = self.e.resources.livestock.animals.values().collect();
        let grass: Vec<_> = self
            .e
            .resources
            .livestock
            .grass
            .iter()
            .map(|(tile, state)| json!({"tile":tile,"state":state}))
            .collect();
        let anchor = self.pen.and_then(|id| {
            self.e
                .resources
                .colony
                .buildings
                .sites
                .get(&id)
                .map(|site| site.anchor)
        });
        json!({"label":label,"seed":self.e.resources.world_seed,"tick":self.e.current_tick(),
            "display_day":sim_systems::runtime::resource_regen::day_index(&self.e.resources),"absolute_deadline":self.limit(),
            "habitat_tiles":self.habitat,"initial_admission_pair":self.initial_pair,"actual_capture_pair":self.capture_sources,
            "source_policy":"D15 mature-only hunting/natural replacement; D17 bind each current kid sequentially by stable ID, second opposite-sex; born_tick is appearance, not biological birth", "source_profiles":source_rows(&self.e,goat),
            "initializer":self.initialization,"steward":{"enabled":self.e.resources.steward.enabled,
                "scope":self.e.resources.steward.scope,"defence_enabled":self.e.resources.steward.defence.enabled},"pen":self.pen,"selected_site":self.pen,
            "livestock":{"animals":animals,"grass":grass,"witnesses":self.e.resources.livestock.witnesses,
                "first_capture":self.e.resources.livestock.first_capture,"first_kid_id":self.e.resources.livestock.first_kid_id,
                "capture_attempts":self.e.resources.livestock.capture_attempts},
            "ui":{"pen_anchor":anchor,"first_kid":self.e.resources.livestock.first_kid_id,
                "capture_entry":event_index(61),"birth_entry":event_index(62),"milk_entry":event_index(63),"slaughter_entry":event_index(64),
                "livestock":animals.iter().map(|a|json!({"id":a.id,"species":a.species.code(),
                    "age":current_age(a,self.e.resources.current_tick,self.e.resources.ticks_per_day),"female":a.sex.female,"pen_site":a.pen_site})).collect::<Vec<_>>()},
            "ledger":self.e.resources.colony.item_ledger,"command_log":self.e.resources.colony.command_log,
            "workshops":self.e.resources.colony.workshops,
            "ground_items":self.e.resources.colony.ground_items.iter().map(|(tile,stack)|json!({"tile":tile,"stack":stack})).collect::<Vec<_>>(),
            "chronicle":self.e.resources.chronicle.entries(),"knowledge":people(&self.e.world,&self.e.resources).into_iter()
                .map(|p|json!({"id":p.id,"ns":p.ns,"adult":p.adult,"knowledge":knowledge_of(&self.e.world,p.entity),
                    "position":p.pos,"hunger":self.e.world.get::<&Hunger>(p.entity).ok().map(|v|v.value),
                    "thirst":self.e.world.get::<&Thirst>(p.entity).ok().map(|v|v.value),
                    "drafted":self.e.world.get::<&Drafted>(p.entity).ok().map(|v|*v),
                    "sleep":self.e.world.get::<&Sleep>(p.entity).ok().map(|v|*v),
                    "skills":self.e.world.get::<&Skills>(p.entity).ok().map(|s|*s),
                    "priorities":self.e.world.get::<&WorkPriorities>(p.entity).ok().map(|v|*v),
                    "job":self.e.world.get::<&Job>(p.entity).ok().map(|v|*v)})).collect::<Vec<_>>(),
            "invariants":"passed","twins":self.twin.is_some()})
    }
    fn checkpoint(&mut self, label: &str) {
        invariants::check(&self.e.world, &self.e.resources)
            .expect("checkpoint physical invariants");
        for &index in self.e.resources.livestock.witnesses.keys() {
            assert_eq!(
                self.e.resources.chronicle.cause_of(index),
                Some(sim_core::chronicle::CauseRef::Livestock { witness: index })
            );
        }
        self.equal();
        let bytes = sim_systems::save::save(&mut self.e).expect("real checkpoint save");
        fresh(&self.out.join(format!("{label}.wsave")), &bytes);
        document(
            &self.out.join(format!("{label}.json")),
            self.manifest(label),
        );
        println!("C1_STEP {label} tick={}", self.e.current_tick());
    }
    fn twin(&mut self) {
        let bytes = sim_systems::save::save(&mut self.e).expect("mid-second-capture save");
        let mut loaded = sim_systems::save::load(&bytes).expect("mid-second-capture load");
        assert_eq!(
            sim_systems::save::save(&mut loaded).expect("immediate loaded bytes"),
            bytes
        );
        fresh(&self.out.join("mid-second-capture.wsave"), &bytes);
        self.twin = Some(loaded);
        self.equal();
    }
}

fn travel_ready(e: &SimEngine, entity: hecs::Entity) -> bool {
    use sim_systems::runtime::agent_actions as needs;
    if e.world.get::<&Drafted>(entity).is_ok()
        || e.world.get::<&Fleeing>(entity).is_ok()
        || sim_systems::runtime::mood::broken(&e.world, &e.resources, entity)
        || e.world.get::<&Agent>(entity).is_ok_and(|agent| {
            e.resources
                .combat_pairs
                .iter()
                .any(|&(a, b)| a == agent.id || b == agent.id)
        })
        || e.world.get::<&WorkPriorities>(entity).map_or_else(
            |_| {
                WorkPriorities::defaults(e.world.get::<&Elder>(entity).is_ok())
                    .level(WorkType::Hunt)
            },
            |priorities| priorities.level(WorkType::Hunt),
        ) == 0
    {
        return false;
    }
    let hunger = e.world.get::<&Hunger>(entity).ok();
    let thirst = e.world.get::<&Thirst>(entity).ok();
    let sleep = e.world.get::<&Sleep>(entity).ok();
    let temperament = e.world.get::<&Temperament>(entity).ok();
    let day_tick = (e.resources.ticks_per_day != 0).then(|| {
        e.resources
            .current_tick
            .saturating_add(e.resources.rules.clock_offset_ticks)
            % e.resources.ticks_per_day
    });
    let threshold = needs::person_night_rest(
        e.resources.rules.agent.night_rest,
        e.resources.rules.agent.temperament,
        temperament.as_deref(),
        day_tick,
    )
    .map_or(needs::FATIGUE_THRESHOLD, |rule| rule.fatigue_threshold);
    needs::actionable_need_with_fatigue_threshold(
        hunger.as_deref(),
        thirst.as_deref(),
        sleep.as_deref(),
        needs::NeedAvailability {
            food: true,
            water: true,
            sleep: true,
        },
        threshold,
    )
    .is_none()
}

fn prioritize(s: &mut Scene, agent: AgentId, work: WorkType, level: u8) -> u8 {
    let person = people(&s.e.world, &s.e.resources)
        .into_iter()
        .find(|p| p.id == agent)
        .expect("actual priority recipient");
    let current = s.e.world.get::<&WorkPriorities>(person.entity).map_or_else(
        |_| WorkPriorities::defaults(s.e.world.get::<&Elder>(person.entity).is_ok()),
        |p| *p,
    );
    if current.level(work) != level {
        s.send(Command::SetWorkPriority { agent, work, level });
    }
    current.level(work)
}

#[test]
fn c1_chain_goat() {
    let out = output();
    let (e, pair, habitat, initialization) = admit(&out);
    let mut s = Scene {
        e,
        twin: None,
        out,
        initial_pair: pair,
        capture_sources: Vec::new(),
        hunting: false,
        habitat,
        initialization,
        pen: None,
        pottery_tile: None,
        clay_ordered: false,
        forming_ordered: false,
        next_firing: 0,
    };
    // D16: ordinary live-game defence and housing support; Food livestock orders stay off.
    s.send(Command::SetSteward {
        on: true,
        scope: sim_core::colony::steward::StewardScope::Housing,
    });
    let adults: Vec<_> = people(&s.e.world, &s.e.resources)
        .into_iter()
        .filter(|p| p.adult)
        .collect();
    let builder = adults
        .iter()
        .filter(|p| knowledge_of(&s.e.world, p.entity).practises(node("firewood_splitting")))
        .min_by_key(|p| (p.ns, p.id))
        .expect("ordinary firewood/wood-bench practitioner")
        .id;
    println!("C1_ROLE bench_builder={builder}; curious adults remain free for discovery");
    for p in people(&s.e.world, &s.e.resources)
        .into_iter()
        .filter(|p| p.adult)
    {
        prioritize(&mut s, p.id, WorkType::Craft, 3); // Teaching must remain possible.
        prioritize(
            &mut s,
            p.id,
            WorkType::Build,
            if p.id == builder { 1 } else { 0 },
        );
        prioritize(
            &mut s,
            p.id,
            WorkType::Split,
            if p.id == builder { 1 } else { 0 },
        );
    }
    let mut trees: Vec<_> =
        s.e.resources
            .colony
            .trees
            .keys()
            .copied()
            .filter(|&t| sim_systems::commands::camp_reaches(&s.e.resources, t))
            .collect();
    trees.sort_by_key(|&t| (chebyshev(t, camp(&s.e)), t.1, t.0));
    trees.truncate(32);
    assert!(!trees.is_empty());
    s.send(Command::DesignateChopTiles { tiles: trees });
    let stone = s
        .e
        .resources
        .colony
        .nodes
        .iter()
        .filter(|(t, n)| {
            n.kind == NodeKind::Boulder && sim_systems::commands::camp_reaches(&s.e.resources, **t)
        })
        .map(|(&t, _)| t)
        .min_by_key(|&t| (chebyshev(t, camp(&s.e)), t.1, t.0))
        .expect("real reachable boulder");
    s.send(Command::DesignateQuarry {
        x0: stone.0,
        y0: stone.1,
        x1: stone.0,
        y1: stone.1,
    });
    // Additional ordinary storage is placed off the camp centre, leaving a pen footprint free.
    let c = camp(&s.e);
    s.send(Command::AddStockpile {
        x0: c.0.saturating_sub(6),
        y0: c.1.saturating_add(8),
        x1: c.0.saturating_add(6),
        y1: c.1.saturating_add(14),
    });
    s.until("real bench materials", |e| {
        e.resources.colony.ground_total(ItemKind::Log) >= 16
            && e.resources.colony.ground_total(ItemKind::Stone) >= 4
    });
    let bench = s.build_bench();
    s.build_bench();
    s.until("two real builds by one worker", |e| {
        e.world
            .query::<&Skills>()
            .iter()
            .any(|(_, k)| k.completed[4] >= 2)
    });
    s.hunting = true;
    for person in people(&s.e.world, &s.e.resources)
        .into_iter()
        .filter(|p| p.adult)
    {
        if knowledge_of(&s.e.world, person.entity).practises(node("tracking")) {
            prioritize(&mut s, person.id, WorkType::Hunt, 1);
        }
    }
    // Consecutive crafts on one bill stay in one Job; separate real bills give
    // the same worker two completed jobs for the authored cordage prerequisite.
    for count in 1..=2 {
        s.bill(bench, "split_firewood", 1);
        let split_bill = s.e.resources.colony.workshops[&bench]
            .bills
            .last()
            .expect("actual finite split bill")
            .id;
        s.until(
            "one worker physically finishes separate split crafts",
            |e| {
                e.resources.colony.workshops[&bench]
                    .bills
                    .iter()
                    .any(|b| b.id == split_bill && b.done == 1)
                    && people(&e.world, &e.resources)
                        .into_iter()
                        .find(|p| p.id == builder)
                        .and_then(|p| {
                            e.world
                                .get::<&Skills>(p.entity)
                                .ok()
                                .map(|k| k.completed[7])
                        })
                        .is_some_and(|done| done >= count)
            },
        );
    }
    for p in people(&s.e.world, &s.e.resources)
        .into_iter()
        .filter(|p| p.adult)
    {
        prioritize(&mut s, p.id, WorkType::Build, 2);
        prioritize(&mut s, p.id, WorkType::Split, 3);
        prioritize(&mut s, p.id, WorkType::Haul, 1);
    }
    s.bill(bench, "split_firewood", 8); // Finite real fuel, rather than both benches' standing auto-bills.
    let flax_species = installed_or_bundled()
        .plant_by_name("wild_flax")
        .expect("natural flax");
    let ripe = |e: &SimEngine| {
        let mut tiles: Vec<_> = e
            .resources
            .food
            .plants
            .iter()
            .filter(|(t, p)| {
                p.species == flax_species
                    && p.units_left >= 4
                    && sim_systems::commands::camp_reaches(&e.resources, **t)
                    && e.resources.nav.reachable(camp(e), **t)
                    && sim_systems::runtime::food::plants::harvestable(&e.resources, **t)
                    && !e.resources.colony.designations.contains_key(*t)
            })
            .map(|(&t, _)| t)
            .collect();
        tiles.sort_by_key(|&t| (chebyshev(t, camp(e)), t.1, t.0));
        tiles.truncate(4);
        tiles
    };
    s.until("naturally ripe first-summer flax", |e| !ripe(e).is_empty());
    s.checkpoint("flax-ready");
    // Independent real plants let ordinary free haulers harvest concurrently.
    for (x, y) in ripe(&s.e) {
        let r = &s.e.resources;
        let spec = sim_core::colony::ItemSpec::raw(item("flax_stalks"), None);
        let passable = r.nav.passable((x, y));
        let region = (r.nav.is_built() && passable)
            .then(|| r.nav.region_of((x, y)))
            .filter(|region| *region != 0);
        println!("C1_FLAX_SOURCE tile={x},{y} region={region:?} passable={passable}");
        let mut available: Vec<_> = (0..r.tile_grid.height)
            .flat_map(|iy| (0..r.tile_grid.width).map(move |ix| (ix, iy)))
            .filter(|&t| {
                region.is_none_or(|region| r.nav.region_of(t) == region)
                    && sim_systems::runtime::jobs::items::room_on(r, t, spec) > 0
            })
            .collect();
        available.sort_by_key(|&t| (chebyshev(t, (x, y)), t.1, t.0));
        let mut left = r.food.plants[&(x, y)].units_left;
        let radius = available
            .into_iter()
            .find_map(|t| {
                left = left.saturating_sub(sim_systems::runtime::jobs::items::room_on(r, t, spec));
                (left == 0).then_some(chebyshev(t, (x, y)))
            })
            .expect("real local output capacity")
            .max(1);
        let (x0, y0, x1, y1) = (
            x.saturating_sub(radius),
            y.saturating_sub(radius),
            x.saturating_add(radius).min(r.tile_grid.width - 1),
            y.saturating_add(radius).min(r.tile_grid.height - 1),
        );
        let fresh_cells = (y0..=y1).any(|iy| {
            (x0..=x1).any(|ix| {
                let t = (ix, iy);
                !r.colony.zone_of_tile.contains_key(&t)
                    && sim_systems::runtime::jobs::items::can_hold_items(r, t)
            })
        });
        if fresh_cells {
            s.send(Command::AddStockpile { x0, y0, x1, y1 });
        }
        s.send(Command::DesignateHarvest {
            x0: x,
            y0: y,
            x1: x,
            y1: y,
        });
    }
    document(
        &s.out.join("flax-orders.json"),
        s.manifest("physical flax orders"),
    );
    s.until(
        "four actual stockpiled stalks at the collection site",
        |e| {
            sim_systems::runtime::knowledge::discover::stockpiled(&e.resources, item("flax_stalks"))
                >= 4
        },
    );
    document(
        &s.out.join("flax-stockpiled.json"),
        s.manifest("physical flax stockpile"),
    );
    s.until("natural cordage discovery", |e| {
        sim_systems::runtime::knowledge::anyone_practises(&e.world, knowledge::bit(node("cordage")))
    });
    let cord_worker = people(&s.e.world, &s.e.resources)
        .into_iter()
        .find(|p| p.adult && knowledge_of(&s.e.world, p.entity).practises(node("cordage")))
        .expect("actual cord practitioner")
        .id;
    prioritize(&mut s, cord_worker, WorkType::Craft, 1);
    prioritize(&mut s, cord_worker, WorkType::Haul, 3);
    prioritize(&mut s, cord_worker, WorkType::Split, 4);
    s.bill(bench, "twist_cord", 2);
    s.until("two physical cords and natural fence discovery", |e| {
        e.resources.colony.ground_total(item("cord")) >= 2
            && sim_systems::runtime::knowledge::anyone_practises(
                &e.world,
                knowledge::bit(node("fence")),
            )
    });
    let anchor = s
        .candidates()
        .into_iter()
        .find(|&(x, y)| {
            let r = &s.e.resources;
            sim_systems::runtime::building_templates::commands::preview(
                &s.e.world,
                r,
                "pen",
                (x, y),
                0,
                None,
            )
            .is_ok()
                && (y + 1..y + 4).all(|iy| {
                    (x + 1..x + 4).all(|ix| {
                        let t = (ix, iy);
                        let i = r.tile_grid.idx(ix, iy);
                        matches!(
                            r.tile_grid.terrain_type[i],
                            TerrainType::Plain | TerrainType::Forest | TerrainType::Tundra
                        ) && r.tile_grid.outdoor[i]
                            && r.tile_grid.wall_material[i].is_none()
                            && r.tile_grid.floor_material[i].is_none()
                            && r.tile_grid.ground_resource[i].is_none()
                            && !r.tile_grid.is_dug(i)
                            && r.nav.passable(t)
                            && !r.colony.trees.contains_key(&t)
                            && !r.colony.nodes.contains_key(&t)
                            && !r.colony.zone_of_tile.contains_key(&t)
                            && !sim_systems::runtime::jobs::items::under_building(r, t)
                            && !r.cultivation.fields.contains_key(&t)
                    })
                })
        })
        .expect("real pen footprint with nine natural forage cells");
    s.send(Command::BuildTemplate {
        id: "pen".into(),
        anchor,
        rotation: 0,
        station: None,
    });
    let pen =
        s.e.resources
            .colony
            .buildings
            .sites
            .values()
            .find(|p| p.template == "pen" && p.anchor == anchor)
            .expect("accepted pen")
            .id;
    s.pen = Some(pen);
    for person in people(&s.e.world, &s.e.resources)
        .into_iter()
        .filter(|p| p.adult)
    {
        prioritize(&mut s, person.id, WorkType::Build, 1);
        prioritize(&mut s, person.id, WorkType::Haul, 2);
        prioritize(&mut s, person.id, WorkType::Craft, 3);
        prioritize(&mut s, person.id, WorkType::Split, 4);
    }
    s.checkpoint("00-pen-placed");
    s.until("physically intact pen", |e| {
        pen_layout(&e.resources, pen).is_ok()
    });
    let layout = pen_layout(&s.e.resources, pen).expect("actual enclosure");
    assert_eq!((layout.capacity, layout.interior.len()), (6, 9));
    for person in people(&s.e.world, &s.e.resources)
        .into_iter()
        .filter(|p| p.adult)
    {
        if knowledge_of(&s.e.world, person.entity).practises(node("animal_taming")) {
            prioritize(&mut s, person.id, WorkType::Hunt, 1);
            prioritize(&mut s, person.id, WorkType::Build, 3);
        }
    }
    let current_source = |e: &SimEngine, first_sex: Option<bool>| {
        let goat = installed_or_bundled()
            .animal_by_name("wild_goat")
            .expect("goat");
        let mut kids: Vec<_> = source_rows(e, goat)
            .into_iter()
            .filter(|row| {
                row["kid"].as_bool() == Some(true)
                    && row["reachable"].as_bool() == Some(true)
                    && first_sex.is_none_or(|sex| row["female"].as_bool() != Some(sex))
            })
            .map(|row| u32::try_from(row["id"].as_u64().expect("actual ID")).expect("wild ID"))
            .collect();
        kids.sort_unstable();
        kids.first().copied()
    };
    let mut first_sex = None;
    for index in 0..2 {
        s.until("current natural kid for sequential capture", |e| {
            current_source(e, first_sex).is_some()
        });
        let wild = current_source(&s.e, first_sex).expect("current natural kid");
        s.capture_sources.push(wild); // Freeze before dispatch; never replace a failed target.
        if index == 1 {
            document(
                &s.out.join("actual-capture-pair.json"),
                s.manifest("D17 sequential pair frozen before second capture"),
            );
            document(
                &s.out.join("capture-order.json"),
                json!({"bound_pair":s.capture_sources,"capture_order":s.capture_sources,
                    "rule":"D17 stable first current kid; after success stable first current opposite-sex kid"}),
            );
        }
        let source =
            s.e.resources
                .colony
                .wildlife
                .animals
                .get(&wild)
                .expect("admitted natural source still exists")
                .clone();
        let policy = source
            .kind
            .definition()
            .tameable
            .as_ref()
            .expect("tameable");
        let p = wild_demographics(
            s.e.resources.world_seed,
            &source,
            policy,
            s.e.resources.current_tick,
            s.e.resources.ticks_per_day,
        );
        if index == 0 {
            first_sex = Some(p.sex.female);
        } else {
            assert_ne!(first_sex, Some(p.sex.female));
        }
        document(
            &s.out.join(format!("capture-source-{index}.json")),
            json!({"source":wild,"appearance_tick":source.born_tick,"appearance_is_not_biological_birth":true,"profile_age":p.age_days,"female":p.sex.female,"tick":s.e.current_tick(),"all_sources":source_rows(&s.e,source.kind)}),
        );
        assert!(
            p.age_days < policy.kid_age_days,
            "accepted source expired during ordinary preparation; do not replace seed/age"
        );
        let consumed = s.e.resources.colony.item_ledger.consumed_of(item("cord"));
        let capture_owner = |e: &SimEngine| {
            people(&e.world, &e.resources).into_iter().find(|person| {
                e.world.get::<&Job>(person.entity).is_ok_and(
                    |job| matches!(job.kind, JobKind::Capture { animal, .. } if animal == wild),
                )
            })
        };
        let captured = |e: &SimEngine| {
            e.resources
                .livestock
                .witnesses
                .values()
                .any(|w| matches!(w.event,LivestockEvent::Captured{wild:id,..}if id==wild))
        };
        let mut staged_tamer = None;
        let mut last_operator = None;
        let mut locked_operator = None;
        let mut seen_capture = false;
        loop {
            if captured(&s.e) {
                break;
            }
            if let Some(owner) = capture_owner(&s.e) {
                seen_capture = true;
                last_operator = Some(owner.id);
                if index == 1 && s.twin.is_none() && s.e.world.get::<&Job>(owner.entity)
                    .is_ok_and(|job| matches!(job.stage, JobStage::Working { ticks_left } if ticks_left > 0))
                {
                    s.twin();
                    document(&s.out.join("mid-second-capture.json"), s.manifest("actual second capture Working"));
                }
            } else if seen_capture || !s.e.resources.livestock.capture_orders.contains_key(&wild) {
                if let Some((agent, craft, haul)) = staged_tamer.take() {
                    prioritize(&mut s, agent, WorkType::Haul, haul);
                    prioritize(&mut s, agent, WorkType::Craft, craft);
                }
                if locked_operator.is_none()
                    && s.e
                        .resources
                        .livestock
                        .capture_attempts
                        .get(&wild)
                        .is_some_and(|attempt| attempt.completed_attempts > 0)
                {
                    locked_operator = last_operator;
                }
                seen_capture = false;
                s.until("same source natural capture cooldown", |e| {
                    capture_owner(e).is_some()
                        || e.resources.livestock.capture_ready(
                            wild,
                            u32::try_from(sim_systems::runtime::resource_regen::day_index(
                                &e.resources,
                            ))
                            .unwrap_or(u32::MAX),
                        )
                });
                if capture_owner(&s.e).is_some() {
                    continue;
                }
                let ready = |e: &SimEngine, person: &sim_systems::runtime::knowledge::Person| {
                    person.adult
                        && locked_operator.is_none_or(|id| person.id == id)
                        && travel_ready(e, person.entity)
                        && knowledge_of(&e.world, person.entity)
                            .practises(node(&policy.capture_knowledge))
                        && policy.capture_tool.is_none_or(|tool| {
                            sim_systems::runtime::jobs::hunt::nearest_stack(
                                &e.resources,
                                person.pos,
                                u32::MAX,
                                &[tool],
                                true,
                            )
                            .is_some()
                        })
                };
                s.until(
                    "current ready adult capture practitioner and stockpile tool",
                    |e| {
                        capture_owner(e).is_some()
                            || people(&e.world, &e.resources)
                                .iter()
                                .any(|person| ready(e, person))
                    },
                );
                if capture_owner(&s.e).is_some() {
                    continue;
                }
                let tamer = people(&s.e.world, &s.e.resources)
                    .into_iter()
                    .filter(|person| ready(&s.e, person))
                    .min_by_key(|person| person.id)
                    .expect("current adult capture practitioner and stockpile tool");
                let capture_at = |e: &SimEngine, at: Tile| {
                    let r = &e.resources;
                    sim_systems::runtime::jobs::livestock::capture_candidate(
                        r,
                        knowledge_of(&e.world, tamer.entity).practice(),
                        r.nav.region_of(at),
                        at,
                        r.rules.colony.expect("colony rules").job_scan_radius,
                        r.livestock.capture_orders.contains_key(&wild),
                    )
                    .is_some_and(|offer| offer.wild == wild && offer.from_tool.is_some())
                };
                loop {
                    s.until(
                        "ordinary needs recovery and free stockpile tool before movement",
                        |e| {
                            capture_owner(e).is_some()
                                || people(&e.world, &e.resources)
                                    .iter()
                                    .any(|person| person.id == tamer.id && ready(e, person))
                        },
                    );
                    if capture_owner(&s.e).is_some() {
                        break;
                    }
                    let from = people(&s.e.world, &s.e.resources)
                        .into_iter()
                        .find(|person| person.id == tamer.id)
                        .expect("same living tamer")
                        .pos;
                    // Admission is relative to the worker, not source-to-tool distance.
                    let r = &s.e.resources;
                    let at = (0..r.tile_grid.height)
                        .flat_map(|y| (0..r.tile_grid.width).map(move |x| (x, y)))
                        .filter(|&tile| {
                            r.nav.passable(tile)
                                && r.nav.reachable(from, tile)
                                && capture_at(&s.e, tile)
                        })
                        .min_by_key(|&tile| (chebyshev(from, tile), tile.1, tile.0))
                        .expect("reachable staging tile admits the same source and real tool");
                    s.send(Command::SetDrafted {
                        agent: tamer.id,
                        drafted: true,
                    });
                    s.send(Command::DraftMove {
                        agent: tamer.id,
                        x: at.0,
                        y: at.1,
                    });
                    s.until("ordinary walk or critical-need release", |e| {
                        people(&e.world, &e.resources).into_iter().any(|person| {
                            capture_owner(e).is_some()
                                || person.id == tamer.id
                                    && (e.resources.nav.arrived(person.pos, at)
                                        || e.world.get::<&Drafted>(person.entity).is_err())
                        })
                    });
                    if let Some(owner) = capture_owner(&s.e) {
                        if owner.id != tamer.id && s.e.world.get::<&Drafted>(tamer.entity).is_ok() {
                            s.send(Command::SetDrafted {
                                agent: tamer.id,
                                drafted: false,
                            });
                        }
                        break;
                    }
                    let person = people(&s.e.world, &s.e.resources)
                        .into_iter()
                        .find(|person| person.id == tamer.id)
                        .expect("same living tamer");
                    s.send(Command::SetDrafted {
                        agent: tamer.id,
                        drafted: false,
                    });
                    if s.e.resources.nav.arrived(person.pos, at)
                        && travel_ready(&s.e, tamer.entity)
                        && capture_at(&s.e, person.pos)
                    {
                        // Ordinary assignment: keep stock-foraging from taking the staged tamer.
                        let haul = prioritize(&mut s, tamer.id, WorkType::Haul, 0);
                        // Lessons precede work rounds; keep their progress while the tamer is busy.
                        let craft = prioritize(&mut s, tamer.id, WorkType::Craft, 0);
                        staged_tamer = Some((tamer.id, craft, haul));
                        break;
                    }
                    // Let real needs resolve after any interrupted or unfinished excursion.
                }

                if !s.e.resources.livestock.capture_orders.contains_key(&wild) {
                    s.send(Command::DesignateCapture { animal: wild });
                }
            }
            let checkpoint = index == 1 && s.twin.is_none();
            s.until("same bound source capture lifecycle", |e| {
                captured(e) || match capture_owner(e) {
                    None => seen_capture,
                    Some(owner) => !seen_capture || checkpoint && e.world.get::<&Job>(owner.entity)
                        .is_ok_and(|job| matches!(job.stage, JobStage::Working { ticks_left } if ticks_left > 0)),
                }
            });
        }
        if let Some((agent, craft, haul)) = staged_tamer {
            prioritize(&mut s, agent, WorkType::Haul, haul);
            prioritize(&mut s, agent, WorkType::Craft, craft);
        }
        assert!(
            index == 0 || s.twin.is_some(),
            "second actual capture Working checkpoint"
        );
        let witness =
            s.e.resources
                .livestock
                .witnesses
                .values()
                .find(|w| matches!(w.event,LivestockEvent::Captured{wild:id,..}if id==wild))
                .expect("capture witness");
        let live = &s.e.resources.livestock.animals[&witness.livestock];
        let now = wild_demographics(
            s.e.resources.world_seed,
            &source,
            policy,
            s.e.resources.current_tick,
            s.e.resources.ticks_per_day,
        );
        assert_eq!(
            (
                current_age(
                    live,
                    s.e.resources.current_tick,
                    s.e.resources.ticks_per_day
                ),
                live.sex
            ),
            (now.age_days, now.sex)
        );
        assert!(!s.e.resources.colony.wildlife.animals.contains_key(&wild));
        assert_eq!(
            s.e.resources.colony.item_ledger.consumed_of(item("cord")),
            consumed + 1
        );
        if index == 0 {
            s.checkpoint("01-capture");
        }
    }
    for person in people(&s.e.world, &s.e.resources)
        .into_iter()
        .filter(|p| p.adult)
    {
        prioritize(&mut s, person.id, WorkType::Build, 2);
        prioritize(&mut s, person.id, WorkType::Haul, 1);
        prioritize(&mut s, person.id, WorkType::Craft, 3);
        prioritize(&mut s, person.id, WorkType::Split, 3);
    }
    s.bill(bench, "split_firewood", 16);
    let parents: Vec<_> = s.e.resources.livestock.animals.keys().copied().collect();
    assert_eq!(parents.len(), 2);
    s.until("two opposite-sex same-pen adults", |e| {
        let animals: Vec<_> = parents
            .iter()
            .filter_map(|id| e.resources.livestock.animals.get(id))
            .collect();
        animals.len() == 2
            && animals.iter().all(|a| a.pen_site == pen)
            && animals[0].species == animals[1].species
            && animals[0].sex != animals[1].sex
            && animals.iter().all(|a| {
                current_age(a, e.resources.current_tick, e.resources.ticks_per_day)
                    >= a.species
                        .definition()
                        .tameable
                        .as_ref()
                        .expect("policy")
                        .kid_age_days
            })
    });
    s.checkpoint("02-two-adults");
    s.until("ordinary birth by absolute60days", |e| {
        e.resources.livestock.first_kid_id.is_some()
    });
    let kid =
        s.e.resources
            .livestock
            .first_kid_id
            .expect("actual newborn");
    let born = s.e.resources.livestock.animals[&kid].creation_entry;
    assert!(matches!(
        s.e.resources.livestock.witnesses[&born].event,
        LivestockEvent::Born { .. }
    ));
    s.checkpoint("03-kid");
    let after_birth = s.e.current_tick();
    let milk_before = s.e.resources.colony.item_ledger.created_of(item("milk"));
    s.until("fresh real milk commit after birth", |e| {
        e.resources.colony.item_ledger.created_of(item("milk")) > milk_before
            && e.resources.livestock.witnesses.iter().any(|(entry, w)| {
                matches!(w.event, LivestockEvent::Milk { .. })
                    && e.resources.chronicle.entries()[*entry as usize].tick >= after_birth
            })
    });
    s.checkpoint("04-milk");
    let species = s.e.resources.livestock.animals[&kid].species;
    let products = &species
        .definition()
        .tameable
        .as_ref()
        .expect("policy")
        .products;
    let meat = products.meat;
    let hide = products.hide;
    let before_meat = s.e.resources.colony.item_ledger.created_of(meat.0);
    let before_hide = s.e.resources.colony.item_ledger.created_of(hide.0);
    s.send(Command::DesignateSlaughter { animal: kid });
    s.until("ordinary atomic meat+hide slaughter", |e| {
        !e.resources.livestock.animals.contains_key(&kid)
            && e.resources
                .livestock
                .witnesses
                .values()
                .any(|w| w.livestock == kid && matches!(w.event, LivestockEvent::Slaughter { .. }))
    });
    assert_eq!(
        s.e.resources.colony.item_ledger.created_of(meat.0),
        before_meat + u64::from(meat.1)
    );
    assert_eq!(
        s.e.resources.colony.item_ledger.created_of(hide.0),
        before_hide + u64::from(hide.1)
    );
    assert_eq!(
        s.e.resources.livestock.first_kid_id,
        Some(kid),
        "card identity remains pinned after removal"
    );
    s.checkpoint("05-slaughter");
    // Replay the complete actual command stream at its original ticks, from the same sole initializer.
    let log = s.e.resources.colony.command_log.clone();
    let final_tick = s.e.current_tick();
    let (mut replay, _) = initialize(s.e.resources.world_seed);
    replay.resources.steward.replay = true;
    let mut next = 0;
    for tick in 0..=final_tick {
        while next < log.len() && log[next].tick == tick {
            let command = log[next].command.clone();
            if matches!(command, Command::Steward { .. }) {
                replay.resources.steward.pending.push((tick, command));
            } else {
                assert_eq!(
                    submit(&mut replay, command),
                    log[next].result,
                    "full replay command {next}"
                );
            }
            next += 1;
        }
        if tick < final_tick {
            replay.tick();
        }
    }
    assert_eq!(next, log.len());
    assert!(replay.resources.steward.pending.is_empty());
    replay.resources.steward.replay = false;
    assert_eq!(replay.resources.colony.command_log, log);
    invariants::check(&replay.world, &replay.resources).expect("full replay physical invariants");
    assert_eq!(
        sim_systems::save::save(&mut replay).expect("replay save"),
        sim_systems::save::save(&mut s.e).expect("original final save"),
        "full ordinary replay byte exact"
    );
    document(
        &s.out.join("complete.json"),
        json!({"complete":true,"seed":s.e.resources.world_seed,"tick":final_tick,
        "deadline":s.limit(),"checkpoints":6,"mid_second_capture_roundtrip":true,"future_command_twins":true,
        "full_command_replay":true,"command_count":log.len(),"first_kid":kid}),
    );
    println!(
        "C1_CHAIN_COMPLETE seed={} tick={final_tick} evidence={}",
        s.e.resources.world_seed,
        s.out.display()
    );
}
