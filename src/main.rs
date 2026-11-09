use raylib::prelude::*;
use space_travel::buffer::{DepthBuffer, Framebuffer};
use space_travel::camera::Camera;
use space_travel::material::Material;
use space_travel::math::{Mat4, Vec2, Vec3};
use space_travel::mesh::{Mesh, Vertex};
use space_travel::pipeline::{
    FragmentShader, GasGiantShader, PlanetRingShader, PlasmaShader, ProceduralRockShader,
    RenderContext, RenderTarget, SkyShader, SoftwareRenderer,
};
use space_travel::procedural::generate_icosphere;
use space_travel::stats::FrameStats;

const WIDTH: usize = 960;
const HEIGHT: usize = 720;

fn create_annulus(inner: f32, outer: f32, segments: usize) -> Mesh {
    let mut vertices = Vec::with_capacity((segments + 1) * 2);
    let mut indices = Vec::with_capacity(segments * 6);
    for i in 0..=segments {
        let angle = std::f32::consts::TAU * i as f32 / segments as f32;
        let (sin, cos) = angle.sin_cos();
        let normal = Vec3::up();
        vertices.push(Vertex::new(
            Vec3::new(cos * inner, 0.0, sin * inner),
            normal,
            Vec2::new(0.0, i as f32 / segments as f32),
        ));
        vertices.push(Vertex::new(
            Vec3::new(cos * outer, 0.0, sin * outer),
            normal,
            Vec2::new(1.0, i as f32 / segments as f32),
        ));
    }
    for i in 0..segments {
        let a = (i * 2) as u32;
        let b = a + 1;
        let c = a + 2;
        let d = a + 3;
        indices.extend_from_slice(&[a, d, b, a, c, d]);
    }
    Mesh::new(vertices, indices)
}

#[allow(clippy::too_many_arguments)]
fn draw_mesh<S: FragmentShader>(
    renderer: &mut SoftwareRenderer,
    mesh: &Mesh,
    shader: &S,
    material: &Material,
    model: &Mat4,
    rotation: &Mat4,
    vp: &Mat4,
    target: &mut RenderTarget<'_>,
    stats: &mut FrameStats,
) {
    let context = RenderContext {
        model_matrix: model,
        rotation_matrix: rotation,
        view_proj_matrix: vp,
        material,
        shader,
    };
    renderer.render_mesh(mesh, &context, target, stats);
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Lab 5 - Shaders procedurales en CPU")
        .build();
    rl.set_target_fps(60);

    let image = unsafe {
        Image::from_raw(raylib::ffi::GenImageColor(
            WIDTH as i32,
            HEIGHT as i32,
            raylib::ffi::Color::BLACK,
        ))
    };
    let mut screen = rl
        .load_texture_from_image(&thread, &image)
        .expect("No se pudo crear la textura de presentación");

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let mut depth = DepthBuffer::new(WIDTH, HEIGHT);
    let mut renderer = SoftwareRenderer::new();
    let camera = Camera::new(
        Vec3::new(0.0, 4.0, 28.0),
        Vec3::zero(),
        Vec3::up(),
        48.0,
        WIDTH as f32 / HEIGHT as f32,
        0.1,
        100.0,
    );
    let sky = SkyShader::new(2026);

    let sphere = generate_icosphere(1.0, 3);
    let moon_mesh = generate_icosphere(1.0, 2);
    let ring_mesh = create_annulus(2.85, 4.0, 112);
    let material = Material::new("ShaderOnly", Vec3::one(), None);

    let sun_pos = Vec3::new(-6.0, 0.0, 0.0);
    let rock_pos = Vec3::new(0.0, 0.0, 0.0);
    let gas_pos = Vec3::new(6.0, 0.0, 0.0);
    let mut plasma = PlasmaShader::new();
    plasma.sun_center = sun_pos;
    plasma.scale = 1.25;
    let mut rock = ProceduralRockShader::new(67, sun_pos);
    rock.body_center = rock_pos;
    let mut gas = GasGiantShader::new(gas_pos, sun_pos);
    let mut moon_shader = ProceduralRockShader::new(173, sun_pos);
    let mut ring_shader = PlanetRingShader {
        center: gas_pos,
        time: 0.0,
    };
    let mut time = 0.0f32;

    while !rl.window_should_close() {
        time += rl.get_frame_time().min(0.1);
        plasma.set_time(time);
        rock.rotation_rad = time * 0.22;
        gas.time = time;
        gas.rotation_rad = time * 0.16;
        ring_shader.time = time;

        sky.render(&mut framebuffer, &camera);
        depth.clear();
        let vp = camera.view_projection_matrix();
        let mut stats = FrameStats::default();
        let mut target = RenderTarget {
            framebuffer: &mut framebuffer,
            depth_buffer: &mut depth,
        };

        let sun_rot = Mat4::rotation_y(time * 0.10);
        let sun_model = Mat4::from_translation(sun_pos) * sun_rot * Mat4::uniform_scaling(2.20);
        draw_mesh(
            &mut renderer,
            &sphere,
            &plasma,
            &material,
            &sun_model,
            &sun_rot,
            &vp,
            &mut target,
            &mut stats,
        );

        let rock_rot = Mat4::rotation_y(time * 0.22);
        let rock_model = Mat4::from_translation(rock_pos) * rock_rot * Mat4::uniform_scaling(2.05);
        draw_mesh(
            &mut renderer,
            &sphere,
            &rock,
            &material,
            &rock_model,
            &rock_rot,
            &vp,
            &mut target,
            &mut stats,
        );

        let gas_rot = Mat4::rotation_y(time * 0.16);
        let gas_model = Mat4::from_translation(gas_pos) * gas_rot * Mat4::uniform_scaling(2.30);
        draw_mesh(
            &mut renderer,
            &sphere,
            &gas,
            &material,
            &gas_model,
            &gas_rot,
            &vp,
            &mut target,
            &mut stats,
        );

        let ring_rotation = Mat4::rotation_z(0.18) * Mat4::rotation_x(0.35);
        let ring_model = Mat4::from_translation(gas_pos) * ring_rotation;
        draw_mesh(
            &mut renderer,
            &ring_mesh,
            &ring_shader,
            &material,
            &ring_model,
            &ring_rotation,
            &vp,
            &mut target,
            &mut stats,
        );

        let moon_pos = gas_pos
            + Vec3::new(
                (time * 0.32).cos() * 3.55,
                1.35 + (time * 0.32).sin() * 0.55,
                (time * 0.32).sin() * 1.15,
            );
        moon_shader.body_center = moon_pos;
        moon_shader.rotation_rad = time * 0.45;
        let moon_rot = Mat4::rotation_y(time * 0.45);
        let moon_model = Mat4::from_translation(moon_pos) * moon_rot * Mat4::uniform_scaling(0.58);
        draw_mesh(
            &mut renderer,
            &moon_mesh,
            &moon_shader,
            &material,
            &moon_model,
            &moon_rot,
            &vp,
            &mut target,
            &mut stats,
        );

        screen
            .update_texture(framebuffer.as_bytes())
            .expect("No se pudo presentar el framebuffer");
        let mut drawing = rl.begin_drawing(&thread);
        drawing.clear_background(raylib::prelude::Color::BLACK);
        drawing.draw_texture(&screen, 0, 0, raylib::prelude::Color::WHITE);
        drawing.draw_text(
            "ESTRELLA DE PLASMA",
            78,
            590,
            20,
            raylib::prelude::Color::GOLD,
        );
        drawing.draw_text(
            "PLANETA ROCOSO",
            385,
            590,
            20,
            raylib::prelude::Color::BEIGE,
        );
        drawing.draw_text(
            "GIGANTE GASEOSO",
            690,
            590,
            20,
            raylib::prelude::Color::SKYBLUE,
        );
        drawing.draw_text(
            "100% SHADERS CPU - SIN TEXTURAS",
            300,
            28,
            20,
            raylib::prelude::Color::RAYWHITE,
        );
    }
}
