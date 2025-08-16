use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tuning {
    /// Anti-aliasing level (None = off, Some(x) = samples count).
    #[serde(default)]
    pub anti_aliasing: AAMode,
    /// Vertical Sync (reduces screen tearing, may add input delay).
    #[serde(default = "default_true")]
    pub vsync: bool,
    /// Low power mode (reduces CPU/GPU usage, possible frame drops and more latency).
    #[serde(default)]
    pub low_power: bool,
    /// Max Frames Per Second (None = unlimited).
    #[serde(default)]
    pub fps_limit: Option<f32>,
    /// Pipelined rendering (better GPU use, may increase input delay).
    #[serde(default = "default_true")]
    pub pipelined: bool,
    /// Multi-threading (faster CPU tasks; wasm only supports single thread).
    #[serde(default = "default_true")]
    pub multithreading: bool,
    /// Show performance diagnostics (FPS, frame time, etc.).
    #[serde(default)]
    pub diagnostic: bool,
    /// Asynchronous asset loading (reduces time to load file, assets may show up later).
    #[serde(default = "default_true")]
    pub async_assets: bool,
    /// Use compressed textures (reduces VRAM use, may lower quality).
    #[serde(default)]
    pub texture_compression: bool,
    /// Enable frustum culling (skips rendering objects outside camera view).
    #[serde(default = "default_true")]
    pub frustum_culling: bool,
    #[cfg_attr(not(target_arch = "wasm32"), serde(default = "default_true"))]
    pub smooth_scaling: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum AAMode {
    None,
    #[cfg_attr(target_arch = "wasm32", default)]
    Msaa2,
    #[cfg_attr(not(target_arch = "wasm32"), default)]
    Msaa4,
    Msaa8,
    Fxaa,
    Taa
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum PowerPref {
    Full,
    Save,
    #[default]
    Auto,
}

fn default_true() -> bool { true }

impl Default for Tuning {
    fn default() -> Self {
        Self {
            anti_aliasing: Default::default(),
            vsync: true,
            low_power: false,
            fps_limit: None,
            pipelined: true,
            multithreading: true,
            diagnostic: false,
            async_assets: true,
            texture_compression: false,
            frustum_culling: true,
            smooth_scaling: false,
        }
    }
}
