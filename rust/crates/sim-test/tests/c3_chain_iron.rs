//! C3-b iron chain under the D-C2-1 preparation contract: seed 0 plus seven founder knowledge
//! grants, then only ordinary player commands. Iron outcrop revealed → mined → charcoal →
//! bloomery → first bloom (chapter 8) → smithy → refined bar → iron axe forged, held and used
//! (chapter 9) → mud-brick house upgraded to a cut-stone house. Forging and iron tools are
//! discovered naturally; targets are retargeted deterministically and natural cancellations are
//! printed with their recorded cause.
mod spine_support;

use sim_core::colony::building_site::Tile;
use sim_core::colony::content::installed_or_bundled;
use sim_core::colony::knowledge::{self, Knowledge, Source};
use sim_core::colony::ore::MetalWitness;
use sim_core::colony::{
    BillMode, BlueprintKind, Command, ItemKind, Job, JobKind, JobStage, NodeKind, RecipeKind, Tool,
};
use sim_core::components::{Agent, Child, Elder, Position};
use sim_engine::SimEngine;
use sim_systems::commands::{camp_reaches, submit};
use sim_systems::runtime::jobs::{invariants, items::chebyshev};
use spine_support::{anchor, camp, command, complete};

const SEED: u64 = 0;
// Seed 0 keeps the C2 chain's world: one iron-ore cluster 46 tiles from camp (C3-a census) on
// reachable ground, and no bog iron, so the ore rule (14) is the one exercised.
const GRANTS: [&str; 7] = [
    "prospecting",
    "mining",
    "charcoal",
    "iron_smelting",
    "mud_brick",
    "mud_brick_house",
    "stone_house",
];
const REISSUES: u32 = 6;

fn item(name: &str) -> ItemKind {
    ItemKind::from_name(name).expect(name)
}
fn blueprint(name: &str) -> BlueprintKind {
    BlueprintKind::from_name(name).expect(name)
}

/// The single initializer shared by the run and the full replay.
fn initial() -> (SimEngine, Vec<(u64, hecs::Entity)>) {
    let mut e = sim_test::colony_engine(SEED);
    let mut founders: Vec<_> = e
        .world
        .query::<(&Agent, &Knowledge)>()
        .without::<&Child>()
        .without::<&Elder>()
        .iter()
        .map(|(entity, (a, _))| (a.id, entity))
        .collect();
    founders.sort();
    founders.truncate(3);
    for &(_, entity) in &founders {
        let mut k = (*e.world.get::<&Knowledge>(entity).expect("knowledge")).clone();
        for name in GRANTS {
            k.set(knowledge::id(name).expect(name), 800, Source::Seeded, 0);
        }
        e.world.insert_one(entity, k).expect("grant");
    }
    (e, founders)
}

fn is_iron(d: &sim_core::colony::ore::OreDeposit) -> bool {
    installed_or_bundled().outcrop(d.kind).is_some_and(|def| {
        def.yield_item
            .definition()
            .tags
            .iter()
            .any(|t| t == "metal_iron")
    })
}

struct Run {
    e: SimEngine,
    out: Option<std::path::PathBuf>,
    receipt: serde_json::Value,
    subjects: serde_json::Value,
    lost: u64,
    cancels: std::collections::BTreeMap<sim_core::colony::CancelReason, u64>,
}

impl Run {
    /// Tick until `ready` or the display-day deadline. A craft cancelled mid-work (its inputs are
    /// consumed) is printed with the recorded cancel reasons and, if given, reissued ≤6 times.
    fn wait(
        &mut self,
        label: &str,
        deadline_day: u64,
        ready: impl Fn(&SimEngine) -> bool,
        reissue: Option<&dyn Fn(&mut Run)>,
    ) {
        let mut reissued = 0;
        while self.e.current_tick() / 1_440 + 1 <= deadline_day && !ready(&self.e) {
            self.e.tick();
            if self.e.current_tick().is_multiple_of(1_440) {
                invariants::check(&self.e.world, &self.e.resources).expect("ledger and jobs");
            }
            let tallies = &self.e.resources.colony.tallies;
            if tallies.crafts_lost > self.lost {
                let causes: Vec<_> = tallies
                    .cancels
                    .iter()
                    .filter(|(r, n)| self.cancels.get(r).copied().unwrap_or(0) < **n)
                    .map(|(r, _)| format!("{r:?}"))
                    .collect();
                println!(
                    "C3B_CANCEL step={label} tick={} crafts_lost={} causes={causes:?}",
                    self.e.current_tick(),
                    tallies.crafts_lost
                );
                self.lost = tallies.crafts_lost;
                if let Some(again) = reissue {
                    assert!(
                        reissued < REISSUES,
                        "{label}: more than {REISSUES} reissues"
                    );
                    reissued += 1;
                    println!("C3B_REISSUE step={label} n={reissued}");
                    again(self);
                }
            }
            self.cancels
                .clone_from(&self.e.resources.colony.tallies.cancels);
        }
        if !ready(&self.e) {
            self.write(&format!("FAILED-{label}"));
            let c = &self.e.resources.colony;
            let bills: Vec<_> = c
                .workshops
                .values()
                .flat_map(|s| {
                    s.bills
                        .iter()
                        .map(move |b| (s.id, b.recipe.name(), b.mode, b.done, b.suspended))
                })
                .collect();
            let made: Vec<_> = [
                "iron_ore",
                "charcoal",
                "iron_bloom",
                "wrought_iron",
                "iron_axe",
            ]
            .iter()
            .map(|n| {
                (
                    n,
                    c.item_ledger.created_of(item(n)),
                    c.ground_total(item(n)),
                )
            })
            .collect();
            panic!(
                "{label} not reached by display day {deadline_day} (tick {}): crafts_lost={} cancels={:?} made(created, on ground)={made:?} bills={bills:?} sites={:?}",
                self.e.current_tick(),
                c.tallies.crafts_lost,
                c.tallies.cancels,
                c.buildings.sites.values().map(|s| (&s.template, s.status)).collect::<Vec<_>>()
            );
        }
        println!(
            "C3B_STEP {label} tick={} day={}",
            self.e.current_tick(),
            self.e.current_tick() / 1_440 + 1
        );
    }
    /// Save `<label>.wsave` plus the capture manifest: the stage's subject tile and the
    /// chronicle entries of the ore, ingot and tool events recorded so far.
    fn write(&mut self, label: &str) {
        let Some(dir) = self.out.clone() else { return };
        let bytes = sim_systems::save::save(&mut self.e).expect("scene save");
        std::fs::write(dir.join(format!("{label}.wsave")), bytes).expect("save bytes");
        self.receipt["steps"][label] = serde_json::json!(self.e.current_tick());
        let entries = self.e.resources.chronicle.entries();
        let ore_entry = self.e.resources.ore.witnesses.iter().find_map(|(&i, w)| {
            let def = installed_or_bundled().outcrop(w.kind)?;
            let iron = def
                .yield_item
                .definition()
                .tags
                .iter()
                .any(|t| t == "metal_iron");
            iron.then_some(i64::from(i))
        });
        // The iron lines: the first bloom's FirstIngot, the iron axe's upgrade and first use.
        let iron_line = |code: u8| {
            entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| entry.kind as u8 == code)
                .find(|(_, entry)| {
                    let code = if code == 67 {
                        u32::try_from(entry.value).ok()
                    } else {
                        Some(entry.cause)
                    };
                    code.and_then(|c| ItemKind::checked(c).ok())
                        .is_some_and(|k| k.definition().tags.iter().any(|t| t == "metal_iron"))
                })
                .map(|(i, _)| i as i64)
        };
        let facts = sim_bridge::ffi::collect_ui_facts(&self.e.world, &self.e.resources);
        let mut manifest = self.receipt.clone();
        manifest["tick"] = serde_json::json!(self.e.current_tick());
        manifest["facts"] = serde_json::json!({
            "first_iron_bloom": facts.first_iron_bloom,
            "first_iron_tool_used": facts.first_iron_tool_used,
        });
        manifest["ui"] = serde_json::json!({
            "subject": self.subjects.get(label),
            "subjects": self.subjects,
            "ore_entry": ore_entry,
            "ingot_entry": iron_line(67),
            "tool_upgraded_entry": iron_line(68),
            "tool_used_entry": iron_line(69),
            "chapter": if label.starts_with("07") { Some("iron_village") } else { None },
        });
        let text = serde_json::to_vec_pretty(&manifest).expect("manifest");
        std::fs::write(dir.join(format!("{label}.json")), text).expect("manifest file");
    }
    fn subject(&mut self, label: &str, tile: Tile) {
        self.subjects[label] = serde_json::json!([tile.0, tile.1]);
    }
    fn send(&mut self, cmd: Command) {
        command(&mut self.e, cmd);
    }
    fn target(&mut self, what: &str, chosen: serde_json::Value, reason: &str) {
        println!("C3B_TARGET {what} {chosen} ({reason})");
        self.receipt["targets"][what] = serde_json::json!({"chosen": chosen, "reason": reason});
    }
    fn shop(&self, kind: &str, within: Option<u32>) -> u32 {
        let kind = blueprint(kind);
        let c = &self.e.resources.colony;
        let inside = |s: &&sim_core::colony::Workshop| {
            within.is_none_or(|site| {
                let site = &c.buildings.sites[&site];
                let t = s.top_left;
                t.0 >= site.anchor.0
                    && t.1 >= site.anchor.1
                    && t.0 < site.anchor.0 + site.footprint.0
                    && t.1 < site.anchor.1 + site.footprint.1
            })
        };
        c.workshops
            .values()
            .filter(|s| s.kind == kind)
            .find(inside)
            .expect("workshop")
            .id
    }
    fn bill(&mut self, shop: u32, recipe: &str, n: u16) -> u32 {
        let id = self.e.resources.colony.workshops[&shop].bills.len();
        self.send(Command::AddBill {
            workshop: shop,
            recipe: RecipeKind::from_name(recipe).expect(recipe),
            mode: BillMode::Make(n),
        });
        self.e.resources.colony.workshops[&shop].bills[id].id
    }
    fn template(&mut self, id: &str) -> u32 {
        let at = anchor(&self.e, id);
        let site = self.e.resources.colony.buildings.next_site_id;
        self.send(Command::BuildTemplate {
            id: id.into(),
            anchor: at,
            rotation: 0,
            station: None,
        });
        site
    }
    fn chop_trees(&mut self, n: usize) {
        let c = camp(&self.e);
        let mut trees: Vec<_> = self
            .e
            .resources
            .colony
            .trees
            .keys()
            .copied()
            .filter(|&t| {
                camp_reaches(&self.e.resources, t)
                    && !self.e.resources.colony.designations.contains_key(&t)
            })
            .collect();
        trees.sort_by_key(|&t| (chebyshev(t, c), t.1, t.0));
        trees.truncate(n);
        self.send(Command::DesignateChopTiles { tiles: trees });
    }
    /// Quarry boulders and dig clay banks nearest camp (ties by tile).
    fn gather(&mut self, kind: NodeKind, n: usize) {
        let c = camp(&self.e);
        let mut nodes: Vec<_> = self
            .e
            .resources
            .colony
            .nodes
            .iter()
            .filter(|(t, node)| {
                node.kind == kind
                    && camp_reaches(&self.e.resources, **t)
                    && !self.e.resources.colony.designations.contains_key(*t)
            })
            .map(|(t, _)| *t)
            .collect();
        nodes.sort_by_key(|&t| (chebyshev(t, c), t.1, t.0));
        for t in nodes.into_iter().take(n) {
            let (x0, y0, x1, y1) = (t.0, t.1, t.0, t.1);
            self.send(if kind == NodeKind::ClayBank {
                Command::DesignateClay { x0, y0, x1, y1 }
            } else {
                Command::DesignateQuarry { x0, y0, x1, y1 }
            });
        }
    }
}

fn knows(e: &SimEngine, name: &str) -> bool {
    knowledge::id(name).is_some_and(|id| {
        sim_systems::runtime::knowledge::anyone_practises(&e.world, knowledge::bit(id))
    })
}
fn stocked_or_held(e: &SimEngine, kind: ItemKind) -> bool {
    let c = &e.resources.colony;
    c.ground_items
        .iter()
        .any(|(t, s)| s.kind == kind && c.zone_of_tile.contains_key(t))
        || e.world
            .query::<&Tool>()
            .iter()
            .any(|(_, t)| t.item.kind == kind)
}
fn tool_used(e: &SimEngine, kind: ItemKind) -> bool {
    e.world
        .query::<&Tool>()
        .iter()
        .any(|(_, t)| t.item.kind == kind && t.uses > 0)
}
fn working_at(e: &SimEngine, shops: &[u32]) -> bool {
    e.world.query::<&Job>().iter().any(|(_, j)| {
        matches!(j.kind, JobKind::Craft { workshop, .. } if shops.contains(&workshop))
            && matches!(j.stage, JobStage::Working { .. })
    })
}

/// Replay `log` on `e` at the recorded ticks, then run to `end`.
fn replay(mut e: SimEngine, log: &[sim_core::colony::CommandRecord], end: u64) -> SimEngine {
    for record in log {
        // Steward orders are regenerated by the simulation itself; only player orders replay.
        if matches!(record.command, Command::Steward { .. }) {
            continue;
        }
        while e.current_tick() < record.tick {
            e.tick();
        }
        submit(&mut e, record.command.clone());
    }
    while e.current_tick() < end {
        e.tick();
    }
    e
}

#[test]
fn c3_chain_iron() {
    let out = std::env::var_os("WORLDSIM_C3_SCENE_OUT").map(|d| {
        let dir = std::path::PathBuf::from(d).join(format!("chain-{}", std::process::id()));
        std::fs::create_dir(&dir).expect("fresh run directory");
        dir
    });
    let (e, founders) = initial();
    let mut r = Run {
        e,
        out,
        receipt: serde_json::json!({"seed": SEED, "grants": GRANTS, "founders": founders.iter().map(|f| f.0).collect::<Vec<_>>(), "injected_items": 0, "steps": {}, "targets": {}}),
        subjects: serde_json::json!({}),
        lost: 0,
        cancels: Default::default(),
    };
    // The steward is switched on, as a player does: defence and food orders keep founders alive.
    r.send(Command::SetSteward {
        on: true,
        scope: Default::default(),
    });
    let c = camp(&r.e);
    r.send(Command::AddStockpile {
        x0: c.0.saturating_sub(6),
        y0: c.1 + 8,
        x1: c.0 + 6,
        y1: c.1 + 14,
    });
    r.chop_trees(48);
    r.gather(NodeKind::Boulder, 2);
    r.gather(NodeKind::ClayBank, 2);
    // 00: retarget to the nearest iron deposit tile with a reachable passable neighbour.
    sim_systems::runtime::nav::ensure(&mut r.e.resources);
    let approach = |e: &SimEngine, t: Tile| {
        (t.1.saturating_sub(1)..=t.1 + 1)
            .flat_map(|y| (t.0.saturating_sub(1)..=t.0 + 1).map(move |x| (x, y)))
            .filter(|n| e.resources.nav.passable(*n) && e.resources.nav.reachable(c, *n))
            .min_by_key(|n| (chebyshev(*n, c), n.1, n.0))
    };
    let mut iron: Vec<Tile> =
        r.e.resources
            .ore
            .deposits
            .iter()
            .filter(|(_, d)| is_iron(d))
            .map(|(t, _)| *t)
            .collect();
    iron.sort_by_key(|&t| (chebyshev(t, c), t.1, t.0));
    let ore_tile = *iron
        .iter()
        .find(|&&t| approach(&r.e, t).is_some())
        .expect("a reachable iron deposit");
    let target = approach(&r.e, ore_tile).expect("approach");
    r.target(
        "iron_outcrop",
        serde_json::json!({"tile": ore_tile, "approach": target, "distance": chebyshev(ore_tile, c)}),
        "nearest iron-tagged deposit tile to camp with a reachable passable neighbour; ties by (y, x)",
    );
    r.receipt["outcrop"] = serde_json::json!(ore_tile);
    for stage in ["00-iron-outcrop-found", "01-iron-mining"] {
        r.subject(stage, ore_tile);
    }
    let walker = founders[0].0;
    r.send(Command::SetDrafted {
        agent: walker,
        drafted: true,
    });
    r.send(Command::DraftMove {
        agent: walker,
        x: target.0,
        y: target.1,
    });
    r.wait(
        "00-iron-outcrop-found",
        15,
        |e| e.resources.ore.deposits[&ore_tile].revealed,
        None,
    );
    r.send(Command::SetDrafted {
        agent: walker,
        drafted: false,
    });
    assert!(r
        .e
        .resources
        .chronicle
        .entries()
        .iter()
        .any(|entry| entry.kind as u8 == 66));
    r.write("00-iron-outcrop-found");
    // 01: the ordinary mine designation; the bloomery is ordered at once so the steward's ore
    // rule finds its site instead of placing a second one.
    r.send(Command::DesignateMine {
        x0: ore_tile.0,
        y0: ore_tile.1,
        x1: ore_tile.0,
        y1: ore_tile.1,
    });
    r.wait(
        "01-iron-mining",
        20,
        |e| {
            e.world.query::<&Job>().iter().any(|(_, j)| {
                matches!(j.kind, JobKind::Mine { .. })
                    && matches!(j.stage, JobStage::Working { .. })
            })
        },
        None,
    );
    r.write("01-iron-mining");
    let ore = item("iron_ore");
    let charcoal = item("charcoal");
    r.wait(
        "clamp materials",
        40,
        |e| {
            e.resources.colony.ground_total(item("clay")) >= 4
                && e.resources.colony.ground_total(item("stone")) >= 2
        },
        None,
    );
    let clamp = r.template("charcoal_clamp");
    r.wait("charcoal clamp built", 50, |e| complete(e, clamp), None);
    let clamp = r.shop("charcoal_clamp", None);
    r.bill(clamp, "burn_charcoal", 4);
    r.gather(NodeKind::ClayBank, 2);
    let bloomery = r.template("bloomery");
    r.wait("bloomery built", 60, |e| complete(e, bloomery), None);
    let bloomery_anchor = r.e.resources.colony.buildings.sites[&bloomery].anchor;
    for stage in ["02-bloomery-working", "03-first-iron-bloom"] {
        r.subject(stage, bloomery_anchor);
    }
    let shop = r.shop("bloomery", Some(bloomery));
    r.wait(
        "six iron ore mined",
        60,
        |e| e.resources.colony.item_ledger.created_of(ore) >= 6,
        None,
    );
    // Two of each: the first iron axe may go to a colonist without an axe; the second can
    // replace an older axe (ToolUpgraded). Blooms are smelted from the ore lying at the mine.
    r.bill(shop, "smelt_bloom", 2);
    let bloom = item("iron_bloom");
    let more_charcoal = |r: &mut Run| {
        r.bill(clamp, "burn_charcoal", 2);
    };
    r.wait(
        "02-bloomery-working",
        60,
        |e| e.resources.colony.item_ledger.crafted_of(bloom) == 0 && working_at(e, &[shop]),
        Some(&more_charcoal),
    );
    r.write("02-bloomery-working");
    let mid_bytes = sim_systems::save::save(&mut r.e).expect("mid save");
    let mut mid = sim_systems::save::load(&mid_bytes).expect("mid load");
    assert_eq!(
        sim_systems::save::save(&mut mid).expect("mid resave"),
        mid_bytes,
        "mid roundtrip"
    );
    let mid_index = r.e.resources.colony.command_log.len();
    r.wait(
        "03-first-iron-bloom",
        70,
        |e| e.resources.colony.item_ledger.crafted_of(bloom) >= 1,
        Some(&more_charcoal),
    );
    let (first_bloom, _) =
        r.e.resources
            .metal
            .witnesses
            .iter()
            .find(|(_, w)| matches!(w, MetalWitness::FirstIngot { item, inputs, .. } if *item == bloom && inputs.iter().any(|i| !i.miners.is_empty())))
            .expect("first bloom witness with miners");
    let causes =
        sim_bridge::ffi::ore::metal_causes(&r.e.resources, *first_bloom).expect("bloom causes");
    assert_eq!(causes.knowledge, ["KNOWLEDGE_IRON_SMELTING"], "{causes:?}");
    assert_eq!(causes.place, "BUILDING_BLOOMERY");
    assert_eq!(causes.inputs, [charcoal.code()]);
    assert!(sim_bridge::ffi::collect_ui_facts(&r.e.world, &r.e.resources).first_iron_bloom);
    r.write("03-first-iron-bloom");
    // Forging is discovered naturally from the stocked bloom; the smithy follows.
    r.wait("forging discovered", 90, |e| knows(e, "forging"), None);
    r.chop_trees(24);
    r.gather(NodeKind::Boulder, 2);
    let smithy = r.template("forge");
    r.wait("smithy built", 100, |e| complete(e, smithy), None);
    let smithy_anchor = r.e.resources.colony.buildings.sites[&smithy].anchor;
    r.subject("04-smithy-working", smithy_anchor);
    // The smithy's default station is the forge hearth; forging recipes run there too.
    let hearth = r.shop("forge", Some(smithy));
    r.bill(clamp, "burn_charcoal", 2);
    r.bill(hearth, "refine_bloom", 2);
    r.wait(
        "04-smithy-working",
        100,
        |e| working_at(e, &[hearth]),
        Some(&more_charcoal),
    );
    r.write("04-smithy-working");
    let bar = item("wrought_iron");
    r.wait(
        "refined bar",
        110,
        |e| e.resources.colony.item_ledger.crafted_of(bar) >= 1,
        Some(&more_charcoal),
    );
    // Iron tools are discovered naturally from the stocked bar; the axe is forged in the smithy.
    r.wait(
        "iron tools discovered",
        120,
        |e| knows(e, "iron_tools"),
        None,
    );
    r.bill(hearth, "forge_iron_axe", 2);
    let axe = item("iron_axe");
    r.wait(
        "iron axe forged",
        125,
        |e| e.resources.colony.item_ledger.crafted_of(axe) >= 1,
        None,
    );
    let (tool_line, _) =
        r.e.resources
            .metal
            .witnesses
            .iter()
            .find(|(_, w)| matches!(w, MetalWitness::FirstIngot { item, .. } if *item == bar))
            .expect("first bar line");
    println!("C3B_BAR_LINE {tool_line}");
    r.wait(
        "iron axe stockpiled",
        125,
        |e| stocked_or_held(e, axe),
        None,
    );
    r.chop_trees(16);
    // 05: an iron axe replaces a colonist's older axe through the ordinary upgrade pickup.
    let upgraded = |e: &SimEngine| {
        e.resources
            .chronicle
            .entries()
            .iter()
            .find(|entry| entry.kind as u8 == 68 && entry.cause == axe.code())
            .and_then(|entry| entry.subject)
    };
    r.wait("05-iron-axe-in-hand", 60, |e| upgraded(e).is_some(), None);
    let agent = upgraded(&r.e).expect("upgrading colonist");
    let tile =
        r.e.world
            .query::<(&Agent, &Tool, &Position)>()
            .iter()
            .find(|(_, (a, t, _))| a.id == agent && t.item.kind == axe)
            .map(|(_, (_, _, p))| (p.x, p.y))
            .expect("the upgrading colonist holds the iron axe");
    r.receipt["holder"] = serde_json::json!(agent);
    r.subject("05-iron-axe-in-hand", tile);
    r.write("05-iron-axe-in-hand");
    r.wait("iron axe used", 135, |e| tool_used(e, axe), None);
    let (used, _) =
        r.e.resources
            .metal
            .witnesses
            .iter()
            .find(|(_, w)| matches!(w, MetalWitness::FirstToolUsed { item, .. } if *item == axe))
            .expect("first iron axe use");
    let causes = sim_bridge::ffi::ore::metal_causes(&r.e.resources, *used).expect("axe causes");
    assert_eq!(
        causes.knowledge,
        ["KNOWLEDGE_FORGING", "KNOWLEDGE_IRON_TOOLS"]
    );
    assert_eq!(causes.place, "BUILDING_FORGE");
    assert!(sim_bridge::ffi::collect_ui_facts(&r.e.world, &r.e.resources).first_iron_tool_used);
    // 06: mud bricks from dug clay and gathered wild flax, a mud-brick house, then cut stone
    // from the mason bench and the ordinary upgrade.
    r.gather(NodeKind::ClayBank, 8);
    r.gather(NodeKind::Boulder, 10);
    r.chop_trees(16);
    r.wait(
        "pot forming discovered",
        160,
        |e| knows(e, "pot_forming"),
        None,
    );
    let flax = installed_or_bundled()
        .plant_by_name("wild_flax")
        .expect("wild flax");
    let ripe = |e: &SimEngine| {
        let mut tiles: Vec<Tile> = e
            .resources
            .food
            .plants
            .iter()
            .filter(|(t, p)| {
                p.species == flax
                    && p.units_left >= 4
                    && camp_reaches(&e.resources, **t)
                    && sim_systems::runtime::food::plants::harvestable(&e.resources, **t)
                    && !e.resources.colony.designations.contains_key(*t)
            })
            .map(|(&t, _)| t)
            .collect();
        tiles.sort_by_key(|&t| (chebyshev(t, c), t.1, t.0));
        tiles.truncate(6);
        tiles
    };
    r.wait("ripe wild flax", 170, |e| ripe(e).len() >= 5, None);
    let plants = ripe(&r.e);
    r.target(
        "wild_flax",
        serde_json::json!(plants),
        "six nearest ripe reachable wild flax plants with four stalks; ties by (y, x)",
    );
    for (x, y) in plants {
        r.send(Command::DesignateHarvest {
            x0: x,
            y0: y,
            x1: x,
            y1: y,
        });
    }
    let bench = blueprint("pottery_bench");
    let tile = spine_support::ui_fixture::free_tile(&r.e);
    r.send(Command::PlaceBlueprint {
        kind: bench,
        x: tile.0,
        y: tile.1,
    });
    r.wait(
        "pottery bench",
        175,
        |e| {
            e.resources
                .colony
                .workshops
                .values()
                .any(|s| s.kind == bench)
        },
        None,
    );
    let pottery = r.shop("pottery_bench", None);
    r.bill(pottery, "make_mud_brick", 17);
    let brick = item("mud_brick");
    let more_bricks = |r: &mut Run| {
        r.bill(pottery, "make_mud_brick", 1);
    };
    r.wait(
        "33 mud bricks",
        200,
        |e| e.resources.colony.item_ledger.crafted_of(brick) >= 33,
        Some(&more_bricks),
    );
    let house = r.template("mud_brick_house");
    r.wait("mud-brick house built", 215, |e| complete(e, house), None);
    let mason = blueprint("mason_bench");
    let tile = spine_support::ui_fixture::free_tile(&r.e);
    r.send(Command::PlaceBlueprint {
        kind: mason,
        x: tile.0,
        y: tile.1,
    });
    r.wait(
        "mason bench",
        220,
        |e| {
            e.resources
                .colony
                .workshops
                .values()
                .any(|s| s.kind == mason)
        },
        None,
    );
    let mason = r.shop("mason_bench", None);
    r.bill(mason, "dress_stone", 33);
    let cut = item("cut_stone");
    let more_stone = |r: &mut Run| {
        r.bill(mason, "dress_stone", 1);
    };
    r.wait(
        "33 cut stones",
        240,
        |e| e.resources.colony.item_ledger.crafted_of(cut) >= 33,
        Some(&more_stone),
    );
    r.send(Command::UpgradeBuilding { site: house });
    let house_anchor = r.e.resources.colony.buildings.sites[&house].anchor;
    for stage in ["06-cut-stone-house", "07-chapter-nine"] {
        r.subject(stage, house_anchor);
    }
    r.wait(
        "06-cut-stone-house",
        260,
        |e| {
            complete(e, house)
                && e.resources.colony.buildings.sites[&house].template == "cut_stone_house"
        },
        None,
    );
    r.write("06-cut-stone-house");
    assert!(sim_bridge::ffi::collect_ui_facts(&r.e.world, &r.e.resources).first_iron_tool_used);
    r.write("07-chapter-nine");
    // Replays: from the mid-chain save, and from the initializer.
    let end = r.e.current_tick();
    let final_bytes = sim_systems::save::save(&mut r.e).expect("final save");
    let log = r.e.resources.colony.command_log.clone();
    let mut from_mid = replay(mid, &log[mid_index..], end);
    assert_eq!(
        sim_systems::save::save(&mut from_mid).expect("mid replay"),
        final_bytes,
        "mid-chain replay"
    );
    let mut full = replay(initial().0, &log, end);
    assert_eq!(
        sim_systems::save::save(&mut full).expect("full replay"),
        final_bytes,
        "full replay"
    );
    if let Some(dir) = &r.out {
        r.receipt["final_tick"] = serde_json::json!(end);
        r.receipt["commands"] = serde_json::json!(log.len());
        std::fs::write(
            dir.join("chain.json"),
            serde_json::to_vec_pretty(&r.receipt).expect("receipt"),
        )
        .expect("receipt file");
    }
}
