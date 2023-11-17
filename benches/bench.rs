use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cube::{renderer::{Renderer, Mesh, Vec3}, window::Window};

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

pub fn benchmark(c: &mut Criterion) {
    c.bench_function("benchmarking", |b| {
        // if you have setup code that shouldn't be benchmarked itself
        // (like constructing a large array, or initializing data),
        // you can put that here

        // this code, inside `iter` is actually measured
        b.iter(|| {
            // a black box disables rust's optimization
            let mut window = Window::new(SCALE, WIDTH, HEIGHT);

            let mut renderer = Renderer::new(90.);

            let mut model = Mesh::from_object_file("skull.obj");
            renderer.rotate_mesh(&mut model, Vec3{x: 0., y: 180., z: 0.});
            renderer.translate_mesh(&mut model, Vec3{x: 0., y: 0., z: 5.});

            for _ in 0..100 {
                renderer.clear_screen(&mut window, colors::BLACK);
                renderer.rotate_mesh(&mut model, Vec3{x: 0.03, y: 0.045, z: 0.06});
                renderer.draw_mesh(&mut window, &model);
            }
        })
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);