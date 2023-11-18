extern crate minifb;

use minifb::{clamp, Key};
use std::time::{Duration, Instant};

mod renderer;
mod window;
mod shapes;

use renderer::{Renderer, Camera};
use shapes::mesh::{Mesh, Polygon, Triangle};
use shapes::vec3::Vec3;
use window::Window;

const SCALE: usize = 1;
const WIDTH: usize = 800/SCALE;
const HEIGHT: usize = 600/SCALE;
const FRAME_RATE: u64 = 60;

#[allow(dead_code)]
mod colors {
    pub const BLACK: u32 = 0x00_00_00_00;
    pub const WHITE: u32 = 0x00_ff_ff_ff;
    pub const RED: u32 = 0x00_ff_00_00;
    pub const GREEN: u32 = 0x00_00_ff_00;
    pub const BLUE: u32 = 0x00_00_00_ff;
    pub const YELLOW: u32 = 0x00_ff_ff_00;
    pub const PURPLE: u32 = 0x00_ff_00_ff;
    pub const AQUA: u32 = 0x00_00_ff_ff;
    pub const ORANGE: u32 = 0x00_ff_9e_00;
}

fn main() {
    let mut window = Window::new(SCALE, WIDTH, HEIGHT);

    let frame_duration = Duration::from_secs(1) / FRAME_RATE as u32;
    let mut last_frame_time = Instant::now();

    let mut renderer = Renderer::new(90.);

    let _axis = Mesh{
        polygon_list: vec![
            // X-axis
            Polygon{triangle: Triangle{a: Vec3{x: 1000., y: 0., z: 50.}, b: Vec3{x: 0., y: 0., z: 50.}, c: Vec3{x: -1000., y: 0., z: 50.}}, color: colors::RED, fill: false},
            // Y-axis
            Polygon{triangle: Triangle{a: Vec3{x: 0., y: 1000., z: 50.}, b: Vec3{x: 0., y: 0., z: 50.}, c: Vec3{x: 0., y: -1000., z: 50.}}, color: colors::GREEN, fill: false},
            // Z-axis
            Polygon{triangle: Triangle{a: Vec3{x: 0., y: 0., z: 1050.}, b: Vec3{x: 0., y: 0., z: 50.}, c: Vec3{x: 0., y: 0., z: -950.}}, color: colors::BLUE, fill: false},
        ],
    };

    let model_select = "mountains";
    #[allow(unused_mut)]
    let mut model = match model_select {
        "cube" => Mesh{
            polygon_list: vec![
                // Front face
                Polygon{triangle: Triangle{a: Vec3{x: -1., y: -1., z: 4.}, b: Vec3{x: 1., y: 1., z: 4.}, c: Vec3{x: 1., y: -1., z: 4.}}, color: colors::BLUE, fill: true},
                Polygon{triangle: Triangle{a: Vec3{x: -1., y: -1., z: 4.}, b: Vec3{x: -1., y: 1., z: 4.}, c: Vec3{x: 1., y: 1., z: 4.}}, color: colors::BLUE, fill: true},

                // Right face
                Polygon{triangle: Triangle{a: Vec3{x: 1., y: -1., z: 4.}, b: Vec3{x: 1., y: 1., z: 6.}, c: Vec3{x: 1., y: -1., z: 6.}}, color: colors::RED, fill: true},
                Polygon{triangle: Triangle{a: Vec3{x: 1., y: -1., z: 4.}, b: Vec3{x: 1., y: 1., z: 4.}, c: Vec3{x: 1., y: 1., z: 6.}}, color: colors::RED, fill: true},

                // Back face
                Polygon{triangle: Triangle{a: Vec3{x: -1., y: -1., z: 6.}, b: Vec3{x: 1., y: -1., z: 6.}, c: Vec3{x: 1., y: 1., z: 6.}}, color: colors::GREEN, fill: true},
                Polygon{triangle: Triangle{a: Vec3{x: -1., y: -1., z: 6.}, b: Vec3{x: 1., y: 1., z: 6.}, c: Vec3{x: -1., y: 1., z: 6.}}, color: colors::GREEN, fill: true},

                // Left face
                Polygon{triangle: Triangle{a: Vec3{x: -1., y: -1., z: 4.}, b: Vec3{x: -1., y: -1., z: 6.}, c: Vec3{x: -1., y: 1., z: 6.}}, color: colors::ORANGE, fill: true},
                Polygon{triangle: Triangle{a: Vec3{x: -1., y: -1., z: 4.}, b: Vec3{x: -1., y: 1., z: 6.}, c: Vec3{x: -1., y: 1., z: 4.}}, color: colors::ORANGE, fill: true},

                // Top face
                Polygon{triangle: Triangle{a: Vec3{x: -1., y: 1., z: 4.}, b: Vec3{x: 1., y: 1., z: 6.}, c: Vec3{x: 1., y: 1., z: 4.}}, color: colors::YELLOW, fill: true},
                Polygon{triangle: Triangle{a: Vec3{x: -1., y: 1., z: 4.}, b: Vec3{x: -1., y: 1., z: 6.}, c: Vec3{x: 1., y: 1., z: 6.}}, color: colors::YELLOW, fill: true},

                // Bottom face
                Polygon{triangle: Triangle{a: Vec3{x: -1., y: -1., z: 4.}, b: Vec3{x: 1., y: -1., z: 4.}, c: Vec3{x: 1., y: -1., z: 6.}}, color: colors::WHITE, fill: true},
                Polygon{triangle: Triangle{a: Vec3{x: -1., y: -1., z: 4.}, b: Vec3{x: 1., y: -1., z: 6.}, c: Vec3{x: -1., y: -1., z: 6.}}, color: colors::WHITE, fill: true},
            ],
        },
        "ship" => Mesh::from_object_file("VideoShip.obj"),
        "teapot" => Mesh::from_object_file("teapot.obj"),
        "mountains" => Mesh::from_object_file("mountains.obj"),
        "skull" => {
            let mut skull = Mesh::from_object_file("skull.obj");
            renderer.rotate_mesh(&mut skull, Vec3{x: 0., y: 180., z: 0.});
            renderer.translate_mesh(&mut skull, Vec3{x: 0., y: 0., z: 5.});
            skull
        }
        "squirtle" => {
            let mut squirtle = Mesh::from_object_file("squirtle.obj");
            renderer.translate_mesh(&mut squirtle, Vec3{x: -120., y: -80., z: 50.});
            renderer.rotate_mesh(&mut squirtle, Vec3{x: 90., y: 180., z: 0.});
            squirtle
        }
        _ => panic!()
    };

    while window.handle.is_open() && !window.handle.is_key_down(Key::Escape) {
        let current_time = Instant::now();
        let delta_time = current_time - last_frame_time;

        if delta_time < frame_duration {
            // Sleep until it's time for the next frame
            std::thread::sleep(frame_duration - delta_time);
            continue;
        }

        // ---------- Camera control ----------
        let mut look_dir = Vec3{x: 0., y: 0., z: 1.};
        let angles = Vec3{x: renderer.camera.pitch, y: renderer.camera.yaw, z: 0.};
        renderer.rotate(&mut look_dir, angles);
        let mut side_dir = Vec3{x: 0., y: 0., z: 1.};
        let angles = Vec3{x: 0., y: renderer.camera.yaw + 90., z: 0.};
        renderer.rotate(&mut side_dir, angles);

        if window.handle.is_key_down(Key::A) {
            renderer.camera.location.x += side_dir.x;
            renderer.camera.location.z += side_dir.z;
        }
        if window.handle.is_key_down(Key::W) {
            renderer.camera.location.x += look_dir.x;
            renderer.camera.location.y += look_dir.y;
            renderer.camera.location.z += look_dir.z;
        }
        if window.handle.is_key_down(Key::S) {
            renderer.camera.location.x -= look_dir.x;
            renderer.camera.location.y -= look_dir.y;
            renderer.camera.location.z -= look_dir.z;
        }
        if window.handle.is_key_down(Key::D) {
            renderer.camera.location.x -= side_dir.x;
            renderer.camera.location.z -= side_dir.z;
        }
        if window.handle.is_key_down(Key::Space) { renderer.camera.location.y += 1.; }
        if window.handle.is_key_down(Key::LeftShift) { renderer.camera.location.y -= 1.; }
        if window.handle.is_key_down(Key::Minus) { renderer.camera.fov -= 1.; }
        if window.handle.is_key_down(Key::Equal) { renderer.camera.fov += 1.; }
        if window.handle.is_key_down(Key::Left) { renderer.camera.yaw += 5.; }
        if window.handle.is_key_down(Key::Right) { renderer.camera.yaw -= 5.; }
        if window.handle.is_key_down(Key::Up) { renderer.camera.pitch = clamp(-90., renderer.camera.pitch + 5., 90.); }
        if window.handle.is_key_down(Key::Down) { renderer.camera.pitch = clamp(-90., renderer.camera.pitch - 5., 90.); }

        if window.handle.is_key_down(Key::R) { renderer.camera = Camera::default(); }

        // ---------- Simulate ----------
        // renderer.rotate_mesh(&mut model, Vec3{x: 0.03 * delta_time.as_millis() as f32, y: 0.045 * delta_time.as_millis() as f32, z: 0.06 * delta_time.as_millis() as f32});


        // ---------- Render ----------
        renderer.clear_screen(&mut window, colors::BLACK);

        // renderer.depth_sort_mesh(&mut model);
        renderer.draw_mesh(&mut window, &model);

        // ---------- Update ----------
        window.handle.update_with_buffer(&window.buffer, window.width, window.height).unwrap();

        last_frame_time = current_time;
    }
}
