//! Read-only factual previews after the real tick owner prepares the place cache.
use super::*;

impl RoutineTick {
    pub(in crate::runtime) const fn preview_ready(&self) -> bool {
        self.checked
    }

    /// Cache-only setup by the real tick owner, before creating a shadow view.
    pub(in crate::runtime) fn prepare_preview_places(&mut self, resources: &SimResources) {
        if !self.checked {
            self.places.refresh(resources);
            self.checked = true;
        }
    }

    /// An eligible place for one explicit kind; does not reserve it or draw a kind.
    pub(in crate::runtime) fn preview_kind(
        &self,
        id: AgentId,
        key: Option<u64>,
        member: Option<SettlementId>,
        temperament: Option<&Temperament>,
        kind: ActivityKind,
    ) -> Option<Tile> {
        if !self.checked
            || self.interrupted.contains(&id)
            || !self.can_enter(id, key, member, temperament)
        {
            return None;
        }
        let sid = member?;
        let lists = self.places.places().get(&sid)?;
        self.cursors
            .borrow_mut()
            .entry((sid, kind.index()))
            .or_default()
            .choose(&lists[kind.index()], &self.occupancy)
    }

    pub(in crate::runtime) fn invitation_pending(&self, id: AgentId) -> bool {
        self.invitations.contains(&id)
    }
    /// Whether any nonzero life-stage weight has an admissible physical place.
    pub(in crate::runtime) fn weighted_place_available(
        &self,
        id: AgentId,
        key: Option<u64>,
        member: Option<SettlementId>,
        child: bool,
        elder: bool,
        temperament: Option<&Temperament>,
    ) -> bool {
        let neutral = Temperament::from_axes([500; 4]);
        let weights = self
            .rule
            .adjusted_weights(child, elder, temperament.unwrap_or(&neutral));
        KINDS.iter().enumerate().any(|(index, kind)| {
            weights[index] > 0
                && self
                    .preview_kind(id, key, member, temperament, *kind)
                    .is_some()
        })
    }
}
