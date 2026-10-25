//! Sparse content-code gate masks derived once from the same validated catalog.
use std::collections::BTreeMap;
use std::fmt;

use super::{bit, Channel, Gate, KnowledgeDefs, Lore, MAX_NODES};
use crate::colony::content::Catalog;
use crate::colony::{BlueprintKind, Designation, RecipeKind};

/// Practitioner masks for recipes, construction, designations and channels.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Gates {
    recipe: BTreeMap<u32, u32>,
    build: BTreeMap<u32, u32>,
    designation: Vec<u32>,
    apprenticeship: u32,
    oral: u32,
    lore: [u32; 3],
}

impl fmt::Debug for Gates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Keep the historical bundled Debug spelling without max-code allocations.
        f.debug_struct("Gates")
            .field("recipe", &self.recipe.values().copied().collect::<Vec<_>>())
            .field("build", &self.build.values().copied().collect::<Vec<_>>())
            .field("designation", &self.designation)
            .field("apprenticeship", &self.apprenticeship)
            .field("oral", &self.oral)
            .field("lore", &self.lore)
            .finish()
    }
}

/// Validate family membership and derive masks without reading installed state.
pub(crate) fn gates_with_catalog(defs: &KnowledgeDefs, catalog: &Catalog) -> Result<Gates, String> {
    if defs.nodes.len() > MAX_NODES {
        return Err("knowledge gates: more than 32 nodes".into());
    }
    let mut masks = Gates {
        recipe: catalog.recipes().map(|r| (r.identity.code, 0)).collect(),
        build: catalog.blueprints().map(|b| (b.identity.code, 0)).collect(),
        designation: vec![0; 8],
        ..Gates::default()
    };
    for (index, node) in defs.nodes.iter().enumerate() {
        if usize::from(node.id) != index {
            return Err("knowledge gates: node ids must retain authored index order".into());
        }
        let mask = bit(node.id);
        for gate in &node.gates {
            match *gate {
                Gate::Recipe(recipe) => {
                    let slot = masks
                        .recipe
                        .get_mut(&recipe.code())
                        .ok_or_else(|| format!("unknown recipe code {}", recipe.code()))?;
                    *slot |= mask;
                }
                Gate::Blueprint(kind) | Gate::Workshop(kind) => {
                    let slot = masks
                        .build
                        .get_mut(&kind.code())
                        .ok_or_else(|| format!("unknown blueprint code {}", kind.code()))?;
                    *slot |= mask;
                }
                Gate::Designation(d) => {
                    let slot = masks
                        .designation
                        .get_mut(usize::from(d.code()))
                        .ok_or_else(|| format!("unknown designation code {}", d.code()))?;
                    *slot |= mask;
                }
                Gate::Channel(Channel::Apprenticeship) => masks.apprenticeship |= mask,
                Gate::Channel(Channel::Oral) => masks.oral |= mask,
                Gate::Lore(lore) => masks.lore[lore as usize] |= mask,
            }
        }
    }
    Ok(masks)
}

impl Gates {
    /// Required nodes for a registered recipe; foreign-catalog IDs require every bit.
    pub fn recipe(&self, recipe: RecipeKind) -> u32 {
        self.recipe.get(&recipe.code()).copied().unwrap_or(u32::MAX)
    }

    /// Required nodes for a registered blueprint or workshop.
    pub fn build(&self, kind: BlueprintKind) -> u32 {
        self.build.get(&kind.code()).copied().unwrap_or(u32::MAX)
    }

    /// Nodes gating work on a designation.
    pub fn designation(&self, d: Designation) -> u32 {
        self.designation
            .get(usize::from(d.code()))
            .copied()
            .unwrap_or(0)
    }

    /// Required nodes for teaching at full fidelity.
    pub const fn apprenticeship(&self) -> u32 {
        self.apprenticeship
    }

    /// Required nodes for improved evening telling.
    pub const fn oral(&self) -> u32 {
        self.oral
    }

    /// Nodes standing for everyday lore.
    pub const fn lore(&self, lore: Lore) -> u32 {
        self.lore[lore as usize]
    }
}
