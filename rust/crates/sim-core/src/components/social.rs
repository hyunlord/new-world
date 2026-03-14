// TODO(v3.1): Migrate relationship storage from Vec to sparse capped BTreeMap.
use crate::enums::{AttachmentType, RelationType};
use crate::ids::EntityId;
use serde::{Deserialize, Serialize};

/// A single social relationship edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipEdge {
    pub target: EntityId,
    /// Affinity score (0.0..=100.0) — maps to RelationType thresholds
    pub affinity: f64,
    /// Trust level (0.0..=1.0)
    pub trust: f64,
    /// Familiarity / interaction count (0.0..=1.0)
    pub familiarity: f64,
    pub relation_type: RelationType,
    /// Tick of last interaction
    pub last_interaction_tick: u64,
    /// Is a bridge tie (connects otherwise disconnected groups, Burt 2004)
    pub is_bridge: bool,
}

impl RelationshipEdge {
    pub fn new(target: EntityId) -> Self {
        Self {
            target,
            affinity: 0.0,
            trust: 0.0,
            familiarity: 0.0,
            relation_type: RelationType::Stranger,
            last_interaction_tick: 0,
            is_bridge: false,
        }
    }

    /// Update relation_type from affinity (Granovetter thresholds from GameConfig)
    pub fn update_type(&mut self) {
        self.relation_type = if self.affinity >= 85.0 {
            RelationType::Intimate
        } else if self.affinity >= 60.0 {
            RelationType::CloseFriend
        } else if self.affinity >= 30.0 {
            RelationType::Friend
        } else if self.affinity >= 5.0 {
            RelationType::Acquaintance
        } else {
            RelationType::Stranger
        };
    }
}

/// Social network component — all relationship edges for one entity
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Social {
    pub edges: Vec<RelationshipEdge>,
    /// Spouse entity ID (if any)
    pub spouse: Option<EntityId>,
    /// Parent entity IDs
    pub parents: Vec<EntityId>,
    /// Children entity IDs
    pub children: Vec<EntityId>,
    /// Social class
    pub social_class: crate::enums::SocialClass,
    /// Faction / settlement group affiliation
    pub faction_id: Option<String>,
    /// Reputation: local score (0.0..=1.0)
    pub reputation_local: f64,
    /// Reputation: regional score (0.0..=1.0)
    pub reputation_regional: f64,
    /// Reputation tags (e.g., "generous", "thief")
    pub reputation_tags: Vec<String>,
    /// Granted social titles (e.g., "TITLE_ELDER", "TITLE_CHIEF")
    #[serde(default)]
    pub titles: Vec<String>,
    /// Social capital (derived from network structure)
    pub social_capital: f64,
    /// Attachment type (Bowlby 1969 — determined during infancy)
    #[serde(default)]
    pub attachment_type: Option<AttachmentType>,
}

impl Social {
    pub fn find_edge(&self, target: EntityId) -> Option<&RelationshipEdge> {
        self.edges.iter().find(|e| e.target == target)
    }

    pub fn find_edge_mut(&mut self, target: EntityId) -> Option<&mut RelationshipEdge> {
        self.edges.iter_mut().find(|e| e.target == target)
    }

    pub fn get_or_create_edge(&mut self, target: EntityId) -> &mut RelationshipEdge {
        if let Some(pos) = self.edges.iter().position(|e| e.target == target) {
            &mut self.edges[pos]
        } else {
            self.edges.push(RelationshipEdge::new(target));
            self.edges.last_mut().unwrap()
        }
    }

    /// Returns the direct kinship coefficient for immediate relations.
    pub fn kinship_r_direct(&self, target: EntityId) -> f64 {
        if self.parents.contains(&target) || self.children.contains(&target) {
            0.5
        } else {
            0.0
        }
    }

    pub fn has_title(&self, title_id: &str) -> bool {
        self.titles.iter().any(|title| title == title_id)
    }

    pub fn grant_title(&mut self, title_id: &str) {
        if self.has_title(title_id) {
            return;
        }
        self.titles.push(title_id.to_string());
    }

    pub fn revoke_title(&mut self, title_id: &str) {
        self.titles.retain(|title| title != title_id);
    }
}

/// Hamilton's coefficient of relatedness between two agents.
pub fn kinship_r(
    subject: &Social,
    target_id: EntityId,
    all_socials: &[(EntityId, &Social)],
) -> f64 {
    let direct = subject.kinship_r_direct(target_id);
    if direct > 0.0 {
        return direct;
    }

    if subject.spouse == Some(target_id) {
        return 0.0;
    }

    let target_social = all_socials
        .iter()
        .find(|(id, _)| *id == target_id)
        .map(|(_, social)| *social);

    if let Some(target) = target_social {
        let shared_parents = subject
            .parents
            .iter()
            .filter(|parent| target.parents.contains(parent))
            .count();
        if shared_parents > 0 {
            return if shared_parents == 2 { 0.5 } else { 0.25 };
        }

        for parent_id in &subject.parents {
            if let Some((_, parent_social)) = all_socials.iter().find(|(id, _)| id == parent_id) {
                if parent_social.parents.contains(&target_id) {
                    return 0.25;
                }
            }
        }

        for child_id in &subject.children {
            if let Some((_, child_social)) = all_socials.iter().find(|(id, _)| id == child_id) {
                if child_social.children.contains(&target_id) {
                    return 0.25;
                }
            }
        }
    }

    0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::EntityId;

    fn make_social(parents: Vec<EntityId>, children: Vec<EntityId>) -> Social {
        Social {
            parents,
            children,
            ..Social::default()
        }
    }

    #[test]
    fn kinship_parent_child() {
        let parent = EntityId(1);
        let child = EntityId(2);
        let subject = make_social(vec![parent], vec![]);
        let all = vec![(child, make_social(vec![parent], vec![]))];
        let all_refs: Vec<(EntityId, &Social)> = all.iter().map(|(id, s)| (*id, s)).collect();
        assert_eq!(subject.kinship_r_direct(parent), 0.5);
        assert_eq!(kinship_r(&subject, parent, &all_refs), 0.5);
    }

    #[test]
    fn kinship_unrelated() {
        let subject = Social::default();
        let target = Social::default();
        let all = vec![(EntityId(3), target)];
        let all_refs: Vec<(EntityId, &Social)> = all.iter().map(|(id, s)| (*id, s)).collect();
        assert_eq!(kinship_r(&subject, EntityId(3), &all_refs), 0.0);
    }

    #[test]
    fn kinship_sibling() {
        let common_parent = EntityId(5);
        let subject = make_social(vec![common_parent], vec![]);
        let sibling = make_social(vec![common_parent], vec![]);
        let all = vec![(EntityId(10), sibling)];
        let all_refs: Vec<(EntityId, &Social)> = all.iter().map(|(id, s)| (*id, s)).collect();
        assert_eq!(kinship_r(&subject, EntityId(10), &all_refs), 0.25);
    }

    #[test]
    fn kinship_grandparent() {
        let grandparent = EntityId(7);
        let parent = EntityId(8);
        let child = EntityId(9);
        let subject = make_social(vec![parent], vec![child]);
        let mut all = vec![(parent, make_social(vec![grandparent], vec![child]))];
        all.push((grandparent, Social::default()));
        let all_refs: Vec<(EntityId, &Social)> = all.iter().map(|(id, s)| (*id, s)).collect();
        assert_eq!(kinship_r(&subject, grandparent, &all_refs), 0.25);
    }
}
