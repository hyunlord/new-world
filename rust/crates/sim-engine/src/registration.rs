//! World membership metadata; concrete runtime constructors stay in sim-systems.

/// World whose runtime systems are being registered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldKind {
    /// Frozen V7 world and legacy lattice fixtures.
    Baseline,
    /// The player colony and colony stress fixtures.
    Colony,
}

/// Registration membership, independent of a system's internal rule gates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemWorlds {
    /// Registered only in the frozen baseline.
    Baseline,
    /// Registered only in the colony.
    Colony,
    /// Registered in both worlds, possibly doing no work under some rules.
    Both,
}

impl SystemWorlds {
    /// Whether this registration applies to the requested world.
    pub const fn includes(self, world: WorldKind) -> bool {
        match (self, world) {
            (Self::Both, _)
            | (Self::Baseline, WorldKind::Baseline)
            | (Self::Colony, WorldKind::Colony) => true,
            (Self::Baseline, WorldKind::Colony) | (Self::Colony, WorldKind::Baseline) => false,
        }
    }
}

/// Stable system name and its declared world membership.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemRegistration {
    /// RuntimeSystem::name, also used by saved system lists.
    pub name: &'static str,
    /// Worlds registering this system, not a claim that every branch can fire.
    pub worlds: SystemWorlds,
}

/// Production membership in execution order. Colony food and decision systems are
/// exclusive; shared registrations retain their existing baseline no-op membership.
pub const SYSTEM_REGISTRATIONS: &[SystemRegistration] = &[
    SystemRegistration {
        name: "BuildingStampSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "InfluenceUpdateSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "AgentInfluenceSampleSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "FoodDailySystem",
        worlds: SystemWorlds::Colony,
    },
    SystemRegistration {
        name: "AgentMovementSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "DraftSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "StewardDefenceSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "BillSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "FleeSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "JobAssignSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "AgentDecisionSystem",
        worlds: SystemWorlds::Baseline,
    },
    SystemRegistration {
        name: "ColonyMindSystem",
        worlds: SystemWorlds::Colony,
    },
    SystemRegistration {
        name: "StewardSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "StaleSeekTargetSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "JobExecuteSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "HungerDecaySystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "ThirstDecaySystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "SleepDecaySystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "ConstructionSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "SocialInteractionSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "SocialDecaySystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "MemorySystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "CombatSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "SettlementSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "StarvationSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "ResourceRegenSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "StockpileDepositSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "StockpileConsumeSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "ShelterPlanningSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "MaturationSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "PairingSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "BedAssignmentSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "AgingSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "HearthFeedSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "FireSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "FissionSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "RelationsSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "FertilitySystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "RoomSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "OralTransmissionSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "ObservationSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "KnowledgeDecaySystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "DiscoverySystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "TechAggregateSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "StorytellerSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "AnimalSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "MoodSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "QuarrelSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "NeighbourSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "VisitorSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "DiagnosisSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "RequestsSystem",
        worlds: SystemWorlds::Both,
    },
    SystemRegistration {
        name: "InfluenceVisualizationSystem",
        worlds: SystemWorlds::Both,
    },
];
