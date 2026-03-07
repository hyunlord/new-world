/// A player-facing notification generated from simulation events.
#[derive(Clone, Debug)]
pub struct SimNotification {
    /// Absolute simulation tick when the notification was produced.
    pub tick: u64,
    /// Urgency tier for UI presentation.
    pub tier: NotificationTier,
    /// Stable kind identifier (for cooldown and UI routing).
    pub kind: String,
    /// Importance score in the 0.0..=1.0 range.
    pub importance: f64,
    /// Raw primary entity ID.
    pub primary_entity: u32,
    /// Optional raw secondary entity ID.
    pub secondary_entity: Option<u32>,
    /// Locale key reserved for future UI localization.
    pub message_key: String,
    /// Preformatted English fallback text for the current prototype.
    pub message_fallback: String,
    /// Notification anchor X position in world space.
    pub position_x: f64,
    /// Notification anchor Y position in world space.
    pub position_y: f64,
}

/// UI urgency tiers for story notifications.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationTier {
    /// T1: crisis notification.
    Crisis = 0,
    /// T2: drama notification.
    Drama = 1,
    /// T3: milestone notification.
    Milestone = 2,
    /// T4: ambient log-only notification.
    Ambient = 3,
}

impl NotificationTier {
    /// Returns the numeric tier value expected by the Godot bridge.
    pub fn as_i64(self) -> i64 {
        self as i64
    }
}

#[cfg(test)]
mod tests {
    use super::NotificationTier;

    #[test]
    fn notification_tier_values_are_stable() {
        assert_eq!(NotificationTier::Crisis.as_i64(), 0);
        assert_eq!(NotificationTier::Drama.as_i64(), 1);
        assert_eq!(NotificationTier::Milestone.as_i64(), 2);
        assert_eq!(NotificationTier::Ambient.as_i64(), 3);
    }
}
