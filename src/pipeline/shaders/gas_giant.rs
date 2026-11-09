use super::{FragmentInput, FragmentShader};
use crate::buffer::Color;
use crate::math::Vec3;

/// Gigante gaseoso procedural: cuatro capas cromáticas sin texturas.
pub struct GasGiantShader {
    pub body_center: Vec3,
    pub light_pos: Vec3,
    pub time: f32,
    pub rotation_rad: f32,
}

impl GasGiantShader {
    pub fn new(body_center: Vec3, light_pos: Vec3) -> Self {
        Self {
            body_center,
            light_pos,
            time: 0.0,
            rotation_rad: 0.0,
        }
    }
}

impl FragmentShader for GasGiantShader {
    fn shade(&self, input: &FragmentInput<'_>) -> Color {
        let n_world = (input.world_pos - self.body_center).normalize();
        let c = self.rotation_rad.cos();
        let s = self.rotation_rad.sin();
        let n = Vec3::new(
            c * n_world.x - s * n_world.z,
            n_world.y,
            s * n_world.x + c * n_world.z,
        );

        // Capa 1: bandas latitudinales anchas.
        let latitude_warp = 0.09 * (n.x * 5.0 + n.z * 3.0 + self.time * 0.18).sin();
        let bands = ((n.y + latitude_warp) * 18.0).sin() * 0.5 + 0.5;

        // Capa 2: turbulencia fina que retuerce los colores de las bandas.
        let turbulence = ((n.x * 13.0 + n.z * 9.0 - self.time * 0.35).sin()
            * (n.y * 21.0 + n.x * 4.0 + self.time * 0.22).cos())
            * 0.5
            + 0.5;

        // Capa 3: tormenta ovalada brillante, localizada analíticamente.
        let storm_center = Vec3::new(0.45, -0.18, 0.87).normalize();
        let dx = (n.x - storm_center.x) * 1.7;
        let dy = (n.y - storm_center.y) * 4.5;
        let dz = (n.z - storm_center.z) * 1.7;
        let storm = (-(dx * dx + dy * dy + dz * dz) * 7.0).exp();

        // Capa 4: bruma polar azul y borde atmosférico Fresnel.
        let polar_haze = n.y.abs().powf(3.2);
        let view = (Vec3::new(0.0, 4.0, 28.0) - input.world_pos).normalize();
        let fresnel = (1.0 - n_world.dot(view).max(0.0)).powf(2.5);

        let dark = Vec3::new(0.10, 0.20, 0.40);
        let cyan = Vec3::new(0.20, 0.72, 0.82);
        let cream = Vec3::new(0.86, 0.78, 0.58);
        let band_color = dark * (1.0 - bands) + cyan * bands;
        let mut color = band_color * (0.78 + turbulence * 0.30);
        color += cream * storm * 0.85;
        color = color * (1.0 - polar_haze * 0.38) + Vec3::new(0.32, 0.42, 0.82) * polar_haze * 0.55;
        color += Vec3::new(0.18, 0.55, 0.85) * fresnel * 0.42;

        let light = (self.light_pos - input.world_pos).normalize();
        let diffuse = 0.18 + 0.82 * n_world.dot(light).max(0.0);
        Color::from_f32(color.x * diffuse, color.y * diffuse, color.z * diffuse, 1.0)
    }
}

/// Shader procedural de los anillos: franjas radiales, vetas y transparencia simulada por color.
pub struct PlanetRingShader {
    pub center: Vec3,
    pub time: f32,
}

impl FragmentShader for PlanetRingShader {
    fn shade(&self, input: &FragmentInput<'_>) -> Color {
        let p = input.world_pos - self.center;
        let radius = (p.x * p.x + p.y * p.y + p.z * p.z).sqrt();
        let angle = p.z.atan2(p.x);
        let fine_bands = (radius * 12.0).sin() * 0.5 + 0.5;
        let gaps = (radius * 31.0 + angle * 2.0).sin().abs();
        let dust = (angle * 17.0 - self.time * 0.12).sin() * 0.5 + 0.5;
        let brightness = 0.28 + fine_bands * 0.38 + gaps * 0.16 + dust * 0.10;
        Color::from_f32(brightness * 0.72, brightness * 0.82, brightness, 1.0)
    }
}
