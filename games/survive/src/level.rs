#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LevelDef {
    pub(crate) name: String,
    pub(crate) mob_definitions: Vec<String>,
    pub(crate) waves: Vec<WaveDef>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct WaveDef {
    /// Duration in seconds
    pub(crate) duration: u32,
    pub(crate) spawn_rate: f32,
    pub(crate) mobs: Vec<MobSpawnDef>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct MobSpawnDef {
    pub(crate) name: String,
    pub(crate) weight: u32,
    pub(crate) min: u32,
    pub(crate) max: u32,
}
