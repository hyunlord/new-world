//! C2-b metal chain on seed 0 (no tin outcrop): six scene-only knowledge grants to three
//! founders, then only ordinary commands. reveal → mine → charcoal → smelt → mould → cast →
//! copper axe used → tin by card trade → alloy → bronze axe used (chapter 7 fact).
mod spine_support;

use sim_core::colony::building_site::Tile;
use sim_core::colony::content::installed_or_bundled;
use sim_core::colony::knowledge::{self, Knowledge, Source};
use sim_core::colony::{
    BillMode, BlueprintKind, Command, ItemKind, Job, JobKind, JobStage, NodeKind, RecipeKind, Tool,
};
use sim_core::components::{Agent, Child, Elder};
use sim_engine::SimEngine;
use sim_systems::commands::{camp_reaches, submit};
use sim_systems::runtime::jobs::{invariants, items::chebyshev};
use spine_support::{anchor, camp, command, complete};

const GRANTS: [&str; 6] = [
    "prospecting",
    "mining",
    "charcoal",
    "copper_smelting",
    "mould_casting",
    "tin_sourcing",
];
const DEADLINE_DAYS: u64 = 120;

fn item(name: &str) -> ItemKind {
    ItemKind::from_name(name).expect(name)
}

/// The single initializer shared by the run and the full replay: seed 0 plus six grants.
fn initial() -> (SimEngine, Vec<(u64, hecs::Entity)>, Tile) {
    let mut e = sim_test::colony_engine(0);
    let catalog = installed_or_bundled();
    let yields = |d: &sim_core::colony::ore::OreDeposit, tag: &str| {
        catalog
            .outcrop(d.kind)
            .is_some_and(|def| def.yield_item.definition().tags.iter().any(|t| t == tag))
    };
    assert!(
        !e.resources
            .ore
            .deposits
            .values()
            .any(|d| yields(d, "metal_tin")),
        "seed0 has no tin"
    );
    let c = camp(&e);
    let copper = e
        .resources
        .ore
        .deposits
        .iter()
        .filter(|(t, d)| yields(d, "metal_copper") && chebyshev(**t, c) <= 60)
        .map(|(t, _)| *t)
        .min_by_key(|&t| (chebyshev(t, c), t.1, t.0))
        .expect("copper within 60 tiles");
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
    (e, founders, copper)
}

struct Run {
    e: SimEngine,
    out: Option<std::path::PathBuf>,
    receipt: serde_json::Value,
}

impl Run {
    fn wait(&mut self, label: &str, ready: impl Fn(&SimEngine) -> bool) {
        let deadline = DEADLINE_DAYS * 1_440;
        while self.e.current_tick() < deadline && !ready(&self.e) {
            self.e.tick();
            if self.e.current_tick().is_multiple_of(1_440) {
                invariants::check(&self.e.world, &self.e.resources).expect("ledger and jobs");
            }
        }
        if !ready(&self.e) {
            self.write(&format!("FAILED-{label}"));
            panic!(
                "{label} not reached by day {DEADLINE_DAYS} (tick {})",
                self.e.current_tick()
            );
        }
        println!(
            "C2B_STEP {label} tick={} day={}",
            self.e.current_tick(),
            self.e.current_tick() / 1_440
        );
    }
    fn write(&mut self, label: &str) {
        let Some(dir) = self.out.clone() else { return };
        let bytes = sim_systems::save::save(&mut self.e).expect("scene save");
        std::fs::write(dir.join(format!("{label}.wsave")), bytes).expect("save bytes");
        self.receipt["steps"][label] = serde_json::json!(self.e.current_tick());
        let event_index = |code: u8| {
            self.e
                .resources
                .chronicle
                .entries()
                .iter()
                .enumerate()
                .filter(|(_, entry)| entry.kind as u8 == code)
                .map(|(index, _)| index as i64)
                .max()
        };
        let smelter_anchor = self
            .e
            .resources
            .colony
            .buildings
            .sites
            .values()
            .find(|site| site.template == "smelter")
            .map(|site| site.anchor);
        // The native driver's manifest: exact tick plus the chain facts recorded so far.
        let mut manifest = self.receipt.clone();
        manifest["tick"] = serde_json::json!(self.e.current_tick());
        manifest["ui"] = serde_json::json!({
            "smelter_anchor": smelter_anchor,
            "ore_entry": event_index(66),
            "ingot_entry": event_index(67),
            "tool_upgraded_entry": event_index(68),
            "tool_used_entry": event_index(69),
        });
        let text = serde_json::to_vec_pretty(&manifest).expect("manifest");
        std::fs::write(dir.join(format!("{label}.json")), text).expect("manifest file");
    }
    fn send(&mut self, cmd: Command) {
        command(&mut self.e, cmd);
    }
    /// Wait for `n` crafts of `bill`.
    fn feed(&mut self, shop: u32, bill: u32, n: u32) {
        self.wait("bill crafts", |e| done(e, shop, bill, n));
    }
    fn shop(&self, kind: &str) -> u32 {
        let kind = BlueprintKind::from_name(kind).expect(kind);
        self.e
            .resources
            .colony
            .workshops
            .values()
            .find(|s| s.kind == kind)
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
}

fn done(e: &SimEngine, shop: u32, bill: u32, n: u32) -> bool {
    e.resources.colony.workshops[&shop]
        .bills
        .iter()
        .any(|b| b.id == bill && b.done >= n)
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

/// Replay `log[from..]` on `e` at the recorded ticks, then run to `end`.
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
fn c2_chain_bronze() {
    let out = std::env::var_os("WORLDSIM_C2_SCENE_OUT").map(|d| {
        let dir = std::path::PathBuf::from(d).join(format!("chain-{}", std::process::id()));
        std::fs::create_dir(&dir).expect("fresh run directory");
        dir
    });
    let (e, founders, copper) = initial();
    let mut r = Run {
        e,
        out,
        receipt: serde_json::json!({"seed": 0, "grants": GRANTS, "founders": founders.iter().map(|f| f.0).collect::<Vec<_>>(), "copper": copper, "injected_items": 0, "steps": {}}),
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
    let mut nodes: Vec<_> =
        r.e.resources
            .colony
            .nodes
            .iter()
            .filter(|(t, _)| camp_reaches(&r.e.resources, **t))
            .map(|(t, n)| (*t, n.kind))
            .collect();
    nodes.sort_by_key(|&(t, _)| (chebyshev(t, c), t.1, t.0));
    for (tile, kind) in nodes
        .iter()
        .filter(|n| n.1 == NodeKind::Boulder)
        .take(2)
        .chain(nodes.iter().filter(|n| n.1 == NodeKind::ClayBank).take(2))
        .copied()
        .collect::<Vec<_>>()
    {
        let cmd = if kind == NodeKind::ClayBank {
            Command::DesignateClay {
                x0: tile.0,
                y0: tile.1,
                x1: tile.0,
                y1: tile.1,
            }
        } else {
            Command::DesignateQuarry {
                x0: tile.0,
                y0: tile.1,
                x1: tile.0,
                y1: tile.1,
            }
        };
        r.send(cmd);
    }
    // 00: a granted founder walks beside the deposit by ordinary draft orders.
    sim_systems::runtime::nav::ensure(&mut r.e.resources);
    let walker = founders[0].0;
    let target = (copper.1.saturating_sub(1)..=copper.1 + 1)
        .flat_map(|y| (copper.0.saturating_sub(1)..=copper.0 + 1).map(move |x| (x, y)))
        .filter(|t| r.e.resources.nav.passable(*t) && r.e.resources.nav.reachable(c, *t))
        .min_by_key(|t| (chebyshev(*t, c), t.1, t.0))
        .expect("reachable approach");
    r.send(Command::SetDrafted {
        agent: walker,
        drafted: true,
    });
    r.send(Command::DraftMove {
        agent: walker,
        x: target.0,
        y: target.1,
    });
    r.wait("00-outcrop-found", |e| {
        e.resources.ore.deposits[&copper].revealed
    });
    r.send(Command::SetDrafted {
        agent: walker,
        drafted: false,
    });
    r.write("00-outcrop-found");
    // 01: ordinary mine designation and job.
    r.send(Command::DesignateMine {
        x0: copper.0,
        y0: copper.1,
        x1: copper.0,
        y1: copper.1,
    });
    r.wait("01-mining", |e| {
        e.world.query::<&Job>().iter().any(|(_, j)| {
            matches!(j.kind, JobKind::Mine { .. }) && matches!(j.stage, JobStage::Working { .. })
        })
    });
    r.write("01-mining");
    let ore = item("copper_ore");
    r.wait("eight copper ore mined", |e| {
        e.resources.colony.item_ledger.created_of(ore) >= 8
    });
    // Charcoal clamp and smelter from gathered clay, stone and logs.
    r.wait("clamp materials", |e| {
        e.resources.colony.ground_total(item("clay")) >= 4
            && e.resources.colony.ground_total(item("stone")) >= 4
    });
    let clamp = r.template("charcoal_clamp");
    r.wait("charcoal clamp built", |e| complete(e, clamp));
    let clamp = r.shop("charcoal_clamp");
    r.bill(clamp, "burn_charcoal", 4);
    r.wait("smelter materials", |e| {
        e.resources.colony.ground_total(item("clay")) >= 6
            && e.resources.colony.ground_total(ItemKind::Log) >= 23
    });
    let smelter = r.template("smelter");
    r.wait("smelter built", |e| complete(e, smelter));
    let smelter = r.shop("smelter");
    let smelt = r.bill(smelter, "smelt_copper", 4);
    let ingot = item("copper_ingot");
    r.wait("02-smelter-working", |e| {
        e.resources.colony.item_ledger.crafted_of(ingot) == 0
            && e.world.query::<&Job>().iter().any(|(_, j)| {
                matches!(j.kind, JobKind::Craft { workshop, .. } if workshop == smelter)
                    && matches!(j.stage, JobStage::Working { .. })
            })
    });
    r.write("02-smelter-working");
    let mid_bytes = sim_systems::save::save(&mut r.e).expect("mid save");
    let mut mid = sim_systems::save::load(&mid_bytes).expect("mid load");
    assert_eq!(
        sim_systems::save::save(&mut mid).expect("mid resave"),
        mid_bytes,
        "mid roundtrip"
    );
    let mid_index = r.e.resources.colony.command_log.len();
    r.wait("03-first-copper-ingot", |e| {
        e.resources.colony.item_ledger.crafted_of(ingot) >= 1
    });
    assert!(r.e.resources.metal.witnesses.values().any(|w| matches!(w, sim_core::colony::ore::MetalWitness::FirstIngot { item, inputs, .. } if *item == ingot && inputs.iter().any(|i| !i.miners.is_empty()))));
    r.write("03-first-copper-ingot");
    r.feed(smelter, smelt, 4);
    // Mould and casting: the pottery bench needs naturally discovered pot forming.
    r.wait("pot forming discovered", |e| knows(e, "pot_forming"));
    let tile = spine_support::ui_fixture::free_tile(&r.e);
    r.send(Command::PlaceBlueprint {
        kind: BlueprintKind::from_name("pottery_bench").expect("bench"),
        x: tile.0,
        y: tile.1,
    });
    let bench_kind = BlueprintKind::from_name("pottery_bench").expect("bench");
    r.wait("pottery bench", |e| {
        e.resources
            .colony
            .workshops
            .values()
            .any(|s| s.kind == bench_kind)
    });
    let bench = r.shop("pottery_bench");
    let mould = r.bill(bench, "make_mould", 2);
    let casting = r.template("casting_bench");
    r.wait("casting bench built", |e| complete(e, casting));
    let casting = r.shop("casting_bench");
    r.wait("two moulds", |e| done(e, bench, mould, 2));
    // The steward's axe rule (or this bill, whichever runs first) casts the copper axe.
    r.bill(casting, "cast_copper_axe", 1);
    let copper_axe = item("copper_axe");
    r.wait("copper axe cast", |e| {
        e.resources.colony.item_ledger.crafted_of(copper_axe) >= 1
    });
    r.wait("copper axe stockpiled", |e| stocked_or_held(e, copper_axe));
    r.chop_trees(16);
    r.wait("04-copper-axe-used", |e| tool_used(e, copper_axe));
    r.write("04-copper-axe-used");
    // The player keeps copper for bronze: remove copper-axe bills (a steward veto), smelt more
    // copper and make the bronze mould.
    let copper_bills: Vec<u32> = r.e.resources.colony.workshops[&casting]
        .bills
        .iter()
        .filter(|b| b.recipe == RecipeKind::from_name("cast_copper_axe").expect("recipe"))
        .map(|b| b.id)
        .collect();
    for bill in copper_bills {
        r.send(Command::RemoveBill {
            workshop: casting,
            bill,
        });
    }
    let more = r.bill(smelter, "smelt_copper", 4);
    r.bill(clamp, "burn_charcoal", 4);
    r.bill(bench, "make_mould", 1);
    r.feed(smelter, more, 4);
    // 05: the alloy is discovered naturally, then tin comes from a real waiting caravan.
    r.wait("bronze alloy discovered", |e| knows(e, "bronze_alloy"));
    r.wait("tin caravan waiting", |e| {
        sim_systems::runtime::ore::tin_caravan(&e.resources).is_some()
    });
    assert!(sim_systems::runtime::ore::tin_shortage(
        &r.e.world,
        &r.e.resources
    ));
    let caravan_id = sim_systems::runtime::ore::tin_caravan(&r.e.resources).expect("caravan");
    let caravan =
        r.e.resources
            .colony
            .neighbours
            .caravans
            .iter()
            .find(|c| c.id == caravan_id)
            .expect("caravan")
            .clone();
    let tin_tagged = |k: ItemKind| k.definition().tags.iter().any(|t| t == "metal_tin");
    let (index, goods) = caravan
        .goods
        .iter()
        .enumerate()
        .filter(|(_, g)| tin_tagged(g.kind))
        .min_by_key(|(_, g)| {
            (
                !g.kind.definition().tags.iter().any(|t| t == "ingot"),
                g.kind.code(),
            )
        })
        .expect("tin goods");
    let take = vec![(
        u16::try_from(index).expect("index"),
        if goods.kind.definition().tags.iter().any(|t| t == "ingot") {
            1
        } else {
            2
        },
    )];
    let rule = sim_systems::runtime::neighbours::rule(&r.e.resources).expect("neighbour rule");
    let mut give = Vec::new();
    let mut stacks: Vec<_> =
        r.e.resources
            .colony
            .ground_items
            .iter()
            .filter(|(t, s)| {
                r.e.resources.colony.zone_of_tile.contains_key(*t)
                    && (s.kind == ItemKind::Log || s.kind == item("stone"))
            })
            .map(|(t, s)| (*t, s.count))
            .collect();
    stacks.sort();
    for (tile, count) in stacks {
        for n in 1..=count {
            let mut trial = give.clone();
            trial.push((tile, n));
            if let Some((g, t)) = sim_systems::runtime::neighbours::trade::values_of(
                &r.e.resources,
                &caravan,
                &trial,
                &take,
            ) {
                if sim_systems::runtime::neighbours::trade::balanced(&rule, g, t) {
                    give = trial;
                    break;
                }
            }
        }
        if !give.is_empty()
            && sim_systems::runtime::neighbours::trade::values_of(
                &r.e.resources,
                &caravan,
                &give,
                &take,
            )
            .is_some_and(|(g, t)| sim_systems::runtime::neighbours::trade::balanced(&rule, g, t))
        {
            break;
        }
        give.push((tile, count));
    }
    let trade = Command::TradeFromCard {
        card: "tin_shortage".into(),
        caravan: caravan_id,
        give: give.iter().map(|&((x, y), n)| (x, y, n)).collect(),
        take,
    };
    r.receipt["trade"] = serde_json::json!(format!("{trade:?}"));
    if let Command::TradeFromCard {
        caravan,
        give,
        take,
        ..
    } = &trade
    {
        r.receipt["trade_terms"] =
            serde_json::json!({"caravan": caravan, "give": give, "take": take});
    }
    r.write("05-tin-trade-card");
    r.send(trade);
    assert!(r
        .e
        .resources
        .metal
        .imports
        .iter()
        .any(|i| i.card.as_deref() == Some("tin_shortage")));
    r.write("trade-committed");
    // 06: alloy and bronze axe, held through the ordinary upgrade pickup.
    if !tin_tagged(item("tin_ingot")) || caravan.goods[index].kind != item("tin_ingot") {
        let tin = r.bill(smelter, "smelt_tin", 1);
        r.feed(smelter, tin, 1);
    }
    let alloy = r.bill(smelter, "alloy_bronze", 1);
    r.feed(smelter, alloy, 1);
    let bronze = item("bronze_ingot");
    let first_bronze = r.e.resources.metal.witnesses.iter().find(|(_, w)| matches!(w, sim_core::colony::ore::MetalWitness::FirstIngot { item, .. } if *item == bronze)).map(|(i, _)| *i).expect("first bronze line");
    let traded =
        r.e.resources
            .metal
            .imports
            .iter()
            .find(|i| i.card.is_some())
            .expect("card import")
            .traded;
    let cause = r.e.resources.chronicle.cause_of(first_bronze);
    assert!(
        matches!(cause, Some(sim_core::chronicle::CauseRef::Entry(i)) if i == traded || r.e.resources.chronicle.cause_of(i) == Some(sim_core::chronicle::CauseRef::Entry(traded))),
        "bronze line cites the card trade: {cause:?}"
    );
    // Earlier moulds went into the steward's copper axes; the player forms one for bronze.
    r.bill(bench, "make_mould", 1);
    r.bill(casting, "cast_bronze_axe", 1);
    let bronze_axe = item("bronze_axe");
    r.wait("bronze axe cast", |e| {
        e.resources.colony.item_ledger.crafted_of(bronze_axe) >= 1
    });
    r.wait("bronze axe stockpiled", |e| stocked_or_held(e, bronze_axe));
    r.chop_trees(16);
    r.wait("06-first-bronze-axe", |e| {
        e.world
            .query::<&Tool>()
            .iter()
            .any(|(_, t)| t.item.kind == bronze_axe)
    });
    r.write("06-first-bronze-axe");
    r.wait("07-chapter-seven", |e| tool_used(e, bronze_axe));
    assert!(r.e.resources.metal.witnesses.values().any(|w| matches!(w, sim_core::colony::ore::MetalWitness::FirstToolUsed { item, .. } if *item == bronze_axe)));
    r.write("07-chapter-seven");
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
