use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    pub parent_id: String,
    pub missing_id: String,
    pub reason: String,
}

pub fn apply_patches() -> Vec<Patch> {
    vec![
        Patch {
            parent_id: "FlashShifter.StardewValleyExpandedCP".to_string(),
            missing_id: "Esca.FarmTypeManager".to_string(),
            reason: "SVE 内子组件 [FTM] 依赖 FTM，但 manifest 顶层未声明".to_string(),
        },
    ]
}

pub fn get_missing_dependency(mod_unique_id: &str) -> Option<Patch> {
    apply_patches()
        .into_iter()
        .find(|p| p.parent_id.to_lowercase() == mod_unique_id.to_lowercase())
}
