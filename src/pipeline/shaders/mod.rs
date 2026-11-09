pub mod alien_energy;
pub mod debug;
pub mod diffuse;
pub mod gas_giant;
pub mod normal_mapped;
pub mod plasma;
pub mod procedural_rock;
pub mod ship;
pub mod sky;
pub mod toon;

use crate::buffer::Color;
use crate::material::Material;
use crate::math::{Vec2, Vec3};

pub use alien_energy::AlienEnergyShader;
pub use debug::{DepthDebugShader, NormalsDebugShader};
pub use diffuse::{DiffuseShader, EmissiveShader, TexturedDiffuseShader};
pub use gas_giant::{GasGiantShader, PlanetRingShader};
pub use normal_mapped::NormalMappedShader;
pub use plasma::PlasmaShader;
pub use procedural_rock::ProceduralRockShader;
pub use ship::ShipShader;
pub use sky::SkyShader;
pub use toon::ToonShader;

#[derive(Debug, Clone)]
pub struct FragmentInput<'a> {
    pub screen_x: usize,
    pub screen_y: usize,
    pub depth_ndc: f32,
    pub world_pos: Vec3,
    pub world_normal: Vec3,
    pub world_tangent: Vec3,
    pub uv: Vec2,
    pub barycentric: Vec3,
    pub material: &'a Material,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderMode {
    Normal,
    Toon,
    DepthDebug,
    NormalsDebug,
}

impl ShaderMode {
    pub fn cycle(self) -> Self {
        match self {
            ShaderMode::Normal => ShaderMode::Toon,
            ShaderMode::Toon => ShaderMode::DepthDebug,
            ShaderMode::DepthDebug => ShaderMode::NormalsDebug,
            ShaderMode::NormalsDebug => ShaderMode::Normal,
        }
    }
}

pub trait FragmentShader: Sync {
    fn shade(&self, input: &FragmentInput<'_>) -> Color;
}
