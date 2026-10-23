//! M5 bridge collectors (`sim_bridge::ffi::story`): the storyteller and the
//! firewood line, animals, the snapshot's M5 columns, a person's mood and
//! draft, the drafted list, the chronicle's index and "→ 왜?" links, the year
//! summary and the teach suggestion — content and order, headless.

use sim_bridge::ffi::{
    chronicle::collect_chronicle_since, collect_agent_m5, collect_agent_m5_columns,
    collect_agent_snapshot, collect_animals, collect_chronicle_why, collect_drafted, collect_story,
    collect_teach_suggestion, collect_year_summary, submit_story, submit_undraft_all, StoryCommand,
};
use sim_core::chronicle::ChronicleKind;
use sim_core::colony::knowledge::{defs, Knowledge};
use sim_core::colony::RejectReason;
use sim_core::components::{Agent, Child, Temperament};
use sim_engine::SimEngine;
use sim_systems::runtime::wildlife;

fn hearth(engine: &SimEngine) -> (u32, u32) {
    sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth")
}

/// Adults by `AgentId`.
fn adults(engine: &SimEngine) -> Vec<u64> {
    let mut v: Vec<u64> = engine
        .world
        .query::<&Agent>()
        .without::<&Child>()
        .iter()
        .map(|(_, a)| a.id)
        .collect();
    v.sort_unstable();
    v
}

#[test]
fn every_m5_read_is_empty_in_the_baseline() {
    let engine = sim_test::baseline_engine(3);
    let story = collect_story(&engine.world, &engine.resources);
    assert!(!story.on && story.active.is_empty());
    assert_eq!((story.wolves, story.wolves_x, story.wolves_y), (0, -1, -1));
    assert!(collect_animals(&engine.resources).id.is_empty());
    let rows = collect_agent_snapshot(&engine.world);
    let m5 = collect_agent_m5_columns(&engine.world, &engine.resources);
    assert!(!rows.is_empty());
    assert_eq!(m5.len(), rows.len(), "one M5 row per snapshot row");
    assert!(
        m5.iter().all(|r| *r == (0, 0, 0, 0)),
        "no top, mood, draft or illness"
    );
    assert!(collect_drafted(&engine.world, &engine.resources)
        .id
        .is_empty());
    let id = i64::try_from(rows[0].agent_id).expect("id");
    let v = collect_agent_m5(&engine.world, &engine.resources, id);
    assert!(v.found);
    assert_eq!((v.mood, v.band, v.top, v.draft_order), (-1, 0, 0, -1));
}

#[test]
fn wolves_show_in_the_story_and_the_animals_with_their_mean_tile() {
    let mut engine = sim_test::colony_engine(4);
    for _ in 0..3 {
        engine.tick();
    }
    let deer = collect_animals(&engine.resources);
    assert!(
        deer.id.len() >= 6,
        "the herd is placed on the first steps: {}",
        deer.id.len()
    );
    assert!(deer.kind.iter().all(|k| *k == 1), "deer only");
    assert!(deer.id.windows(2).all(|w| w[0] < w[1]), "by AnimalId");
    let h = hearth(&engine);
    let at = (h.0 + 30, h.1);
    wildlife::spawn_pack(&mut engine.resources, at, 3, 1);
    let story = collect_story(&engine.world, &engine.resources);
    assert!(story.on);
    assert_eq!((story.wolves, story.pack_size), (3, 3));
    assert_eq!(
        (story.wolves_x, story.wolves_y),
        (at.0 as i32, at.1 as i32),
        "three wolves on one tile"
    );
    assert!(
        story.firewood_day_need > 0,
        "the colony burns wood at night"
    );
    assert!(story.dress_order, "the dressing order is on by default");
    let all = collect_animals(&engine.resources);
    assert_eq!(
        all.kind.iter().filter(|k| **k == 0).count(),
        3,
        "three wolves"
    );
    // A hunt zone over the first deer marks it huntable.
    let (x, y) = (deer.x[0], deer.y[0]);
    let r = submit_story(
        &mut engine,
        StoryCommand::AddHuntZone(x - 1, y - 1, x + 1, y + 1),
    );
    assert!(r.accepted, "{r:?}");
    let zoned = collect_animals(&engine.resources);
    assert_eq!(zoned.zones, vec![x - 1, y - 1, x + 1, y + 1]);
    let i = zoned
        .id
        .iter()
        .position(|v| *v == deer.id[0])
        .expect("still there");
    assert_eq!(zoned.huntable[i], 1);
    let mark = submit_story(&mut engine, StoryCommand::DesignateHunt(deer.id[1]));
    assert!(mark.accepted, "{mark:?}");
    let marked = collect_animals(&engine.resources);
    let j = marked
        .id
        .iter()
        .position(|v| *v == deer.id[1])
        .expect("deer 2");
    assert_eq!((marked.marked[j], marked.huntable[j]), (1, 1));
}

#[test]
fn a_draft_shows_in_the_columns_the_person_and_the_drafted_list() {
    let mut engine = sim_test::colony_engine(4);
    // Past the first MoodSystem run (minute 7 of the hour, D18), so every adult has a mood.
    for _ in 0..8 {
        engine.tick();
    }
    let folk = adults(&engine);
    let (a, b) = (folk[0] as i64, folk[1] as i64);
    assert!(submit_story(&mut engine, StoryCommand::Draft(a, true)).accepted);
    assert!(submit_story(&mut engine, StoryCommand::Draft(b, true)).accepted);
    engine.tick();
    let rows = collect_agent_snapshot(&engine.world);
    let m5 = collect_agent_m5_columns(&engine.world, &engine.resources);
    assert_eq!(m5.len(), rows.len());
    for (row, cols) in rows.iter().zip(&m5) {
        let drafted = row.agent_id as i64 == a || row.agent_id as i64 == b;
        assert_eq!(
            cols.2,
            u8::from(drafted),
            "drafted column of {}",
            row.agent_id
        );
        assert!(cols.1 >= 1, "every adult has a mood band");
    }
    let list = collect_drafted(&engine.world, &engine.resources);
    assert_eq!(list.id, vec![a, b], "by AgentId");
    assert_eq!(list.order, vec![0, 0], "holding");
    let h = hearth(&engine);
    let mv = submit_story(
        &mut engine,
        StoryCommand::DraftMove(a, h.0 as i32, h.1 as i32 + 2),
    );
    assert!(mv.accepted, "{mv:?}");
    let v = collect_agent_m5(&engine.world, &engine.resources, a);
    assert!(v.drafted);
    assert_eq!(
        (v.draft_order, v.draft_x, v.draft_y),
        (1, i64::from(h.0), i64::from(h.1) + 2)
    );
    assert!(v.mood >= 0 && v.target >= 0, "an adult's mood and target");
    assert!(!v.modifiers.is_empty(), "something moves the mood");
    assert!(
        v.modifiers.windows(2).all(|w| w[0].2.abs() >= w[1].2.abs()),
        "largest first: {:?}",
        v.modifiers
    );
    let hit = submit_story(&mut engine, StoryCommand::DraftAttackPerson(a, b));
    assert!(hit.accepted, "{hit:?}");
    assert_eq!(
        collect_agent_m5(&engine.world, &engine.resources, a).draft_order,
        3
    );
    let me = submit_story(&mut engine, StoryCommand::DraftAttackPerson(a, a));
    assert_eq!(me.reason, Some(RejectReason::Unknown), "not oneself");
    let all = submit_undraft_all(&mut engine);
    assert!(all.accepted);
    assert_eq!(all.count, 2);
    assert!(collect_drafted(&engine.world, &engine.resources)
        .id
        .is_empty());
    let none = submit_undraft_all(&mut engine);
    assert_eq!(none.reason, Some(RejectReason::Nothing));
}

#[test]
fn chronicle_rows_carry_their_index_and_the_why_link() {
    let mut engine = sim_test::colony_engine(1);
    let day = 1_440u64;
    // Run until a pressure has ended (its PressureEnded links to its PressureBegan).
    let mut ended = None;
    for _ in 0..(30 * day) {
        engine.tick();
        ended = engine
            .resources
            .chronicle
            .entries()
            .iter()
            .position(|e| e.kind == ChronicleKind::PressureEnded);
        if ended.is_some() {
            break;
        }
    }
    let ended = ended.expect("a pressure ends within 30 days") as i64;
    let page = collect_chronicle_since(&engine.resources.chronicle, 0, 512);
    assert_eq!(
        page.index,
        (0..page.tick.len() as i64).collect::<Vec<_>>(),
        "indices in order"
    );
    let i = usize::try_from(ended).expect("index");
    assert_eq!(page.link[i], 0, "PressureEnded links to an entry");
    let began = page.link_entry[i];
    assert_eq!(
        engine.resources.chronicle.entries()[began as usize].kind,
        ChronicleKind::PressureBegan
    );
    let why = collect_chronicle_why(&engine.resources.chronicle, ended);
    assert!(why.found);
    assert_eq!(why.link, vec![0, 0], "ended → began");
    assert_eq!(why.code, vec![ended, began]);
    assert_eq!(why.kind[1], i32::from(ChronicleKind::PressureBegan.as_u8()));
    assert!(!collect_chronicle_why(&engine.resources.chronicle, 1 << 40).found);
}

#[test]
fn the_year_summary_counts_the_years_chronicle() {
    let mut engine = sim_test::colony_engine(1);
    let year = engine.resources.rules.calendar.ticks_per_year();
    for _ in 0..year + 10 {
        engine.tick();
    }
    let s = collect_year_summary(&engine.resources, 0);
    assert!(s.found && s.complete);
    let entries = engine.resources.chronicle.entries();
    let in_year = |k: ChronicleKind| {
        entries
            .iter()
            .filter(|e| e.kind == k && e.tick < year)
            .count() as u32
    };
    let began = in_year(ChronicleKind::PressureBegan);
    assert!(began >= 1, "the storyteller acted in year 0");
    assert_eq!(s.pressures.iter().sum::<u32>(), began);
    assert_eq!(s.births, in_year(ChronicleKind::Born));
    assert_eq!(s.quarrels, in_year(ChronicleKind::Quarrel));
    assert_eq!(s.deaths.iter().sum::<u32>(), in_year(ChronicleKind::Died));
    let next = collect_year_summary(&engine.resources, 1);
    assert!(next.found && !next.complete);
    assert!(!collect_year_summary(&engine.resources, 5).found);
}

#[test]
fn the_teach_suggestion_is_the_most_novelty_seeking_non_practitioner() {
    let engine = sim_test::colony_engine(2);
    let node = defs().id_of("tracking").expect("tracking");
    let folk = adults(&engine);
    let teacher = folk[0];
    let got = collect_teach_suggestion(&engine.world, teacher as i64, i64::from(node));
    // Expected by hand: adults other than the teacher who do not practise it, highest ns.
    let mut best: Option<(u16, u64)> = None;
    for (_, (a, k, t)) in engine
        .world
        .query::<(&Agent, Option<&Knowledge>, Option<&Temperament>)>()
        .without::<&Child>()
        .iter()
    {
        if a.id == teacher || k.is_some_and(|k| k.practises(node)) {
            continue;
        }
        let ns = t.map_or(0, |t| t.ns);
        if best.is_none_or(|(b, _)| ns > b) {
            best = Some((ns, a.id));
        }
    }
    let (ns, _) = best.expect("a candidate");
    assert!(got >= 0, "a learner is suggested");
    let chosen = engine
        .world
        .query::<(&Agent, Option<&Temperament>)>()
        .iter()
        .find(|(_, (a, _))| a.id as i64 == got)
        .map(|(_, (_, t))| t.map_or(0, |t| t.ns))
        .expect("the suggestion is a person");
    assert_eq!(chosen, ns, "the highest novelty seeking");
    assert_ne!(got, teacher as i64);
    assert_eq!(
        collect_teach_suggestion(&engine.world, teacher as i64, 250),
        -1
    );
}
