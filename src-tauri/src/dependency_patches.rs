use crate::mod_name_resolver::resolve_mod_name;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::LazyLock;

use super::mod_installer::MissingDepInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyPatch {
    pub parent_id: String,
    pub missing_id: String,
    pub description: String,
}

static HARDCODED_PATCHES: LazyLock<Vec<DependencyPatch>> = LazyLock::new(|| {
    vec![
        DependencyPatch {
            parent_id: "FlashShifter.StardewValleyExpandedCP".to_string(),
            missing_id: "Esca.FarmTypeManager".to_string(),
            description: "SVE 需要 Farm Type Manager (FTM) 才能正常运行".to_string(),
        },
    ]
});

pub fn apply_final_patches(
    installing_mod_id: &str,
    installed_mod_ids: &HashSet<String>,
    missing_deps: &mut Vec<MissingDepInfo>,
) {
    let already_reported: HashSet<String> = missing_deps.iter().map(|d| d.unique_id.clone()).collect();

    for patch in HARDCODED_PATCHES.iter() {
        if patch.parent_id != installing_mod_id {
            continue;
        }

        if !already_reported.contains(&patch.missing_id) && !installed_mod_ids.contains(&patch.missing_id) {
            let display_name = resolve_mod_name(&patch.missing_id);
            missing_deps.push(MissingDepInfo {
                unique_id: patch.missing_id.clone(),
                display_name,
                minimum_version: None,
                is_required: true,
            });
        }
    }
}
