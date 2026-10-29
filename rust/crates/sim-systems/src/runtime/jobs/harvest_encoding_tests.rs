//! Save-wire compatibility for harvest additions to existing job/order enums.

use sim_core::colony::{Command, Designation, JobKind, ReserveKey, WorkType};

#[test]
fn harvest_appends_without_changing_existing_discriminants() {
    let tile = (7, 9);
    let jobs = [
        JobKind::Chop { tree: tile },
        JobKind::Haul {
            from: tile,
            to: tile,
        },
        JobKind::Deliver {
            from: tile,
            site: tile,
        },
        JobKind::Build { site: tile },
        JobKind::Quarry { node: tile },
        JobKind::Supply {
            from: tile,
            workshop: 1,
            to: tile,
        },
        JobKind::Gather {
            spot: tile,
            to: tile,
        },
        JobKind::Dig { tile },
        JobKind::Deconstruct { tile },
        JobKind::Clear {
            from: tile,
            to: tile,
        },
        JobKind::DigClay { node: tile },
        JobKind::Teach { order: 1, at: tile },
        JobKind::Learn { order: 1, at: tile },
        JobKind::Hunt { animal: 1 },
        JobKind::Equip { from: tile },
        JobKind::Trade {
            from: tile,
            to: tile,
        },
        JobKind::Craft {
            workshop: 1,
            bill: 1,
            at: tile,
        },
        JobKind::Harvest { plant: tile },
    ];
    for (index, job) in jobs.iter().enumerate() {
        let bytes = bincode::serialize(job).unwrap();
        assert_eq!(&bytes[..4], &u32::try_from(index).unwrap().to_le_bytes());
        assert_eq!(bincode::deserialize::<JobKind>(&bytes).unwrap(), *job);
    }
    let marks = [
        Designation::Chop,
        Designation::Quarry,
        Designation::Dig,
        Designation::Deconstruct,
        Designation::ClayPit,
        Designation::Hunt,
        Designation::Harvest,
    ];
    for (index, mark) in marks.iter().enumerate() {
        assert_eq!(
            bincode::serialize(mark).unwrap(),
            u32::try_from(index).unwrap().to_le_bytes()
        );
        assert_eq!(usize::from(mark.code()), index);
    }
    assert_eq!(
        &bincode::serialize(&ReserveKey::Animal(1)).unwrap()[..4],
        &8_u32.to_le_bytes()
    );
    assert_eq!(
        &bincode::serialize(&ReserveKey::Plant(tile)).unwrap()[..4],
        &9_u32.to_le_bytes()
    );
    assert_eq!(
        &bincode::serialize(&Command::SetStewardDefence { on: true }).unwrap()[..4],
        &36_u32.to_le_bytes()
    );
    assert_eq!(
        &bincode::serialize(&Command::DesignateHarvest {
            x0: 1,
            y0: 2,
            x1: 3,
            y1: 4
        })
        .unwrap()[..4],
        &37_u32.to_le_bytes()
    );
    let harvest = JobKind::Harvest { plant: tile };
    assert_eq!(
        &bincode::serialize(&Command::DeployFoodPot { tile }).unwrap()[..4],
        &38_u32.to_le_bytes()
    );
    assert_eq!(harvest.code(), 18);
    assert_eq!(harvest.work(), WorkType::Haul);
    assert_eq!(harvest.source(), tile);
    assert_eq!(harvest.dest(), None);
}
