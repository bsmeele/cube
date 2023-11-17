use std::cmp::Ordering;
use std::collections::VecDeque;
use minifb::clamp;
use crate::window::Window;
use std::f32::consts::PI;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::mem::swap;

pub struct Renderer {
    pub camera: Camera,
}

pub struct Camera {
    pub location: Vec3,
    pub fov: f32,           // In degrees
    pub pitch: f32,         // In degrees
    pub yaw: f32,           // In degrees
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            location: Vec3::default(),
            fov: 90.,
            pitch: 0.,
            yaw: 0.,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
struct Vec2 { // Used for the location on the screen, meaning positive y is down
x: isize,
    y: isize,
    depth: f32,
}
impl Vec2 {
    pub fn add(&self, v: &Vec2) -> Self {
        Self {
            x: self.x + v.x,
            y: self.y + v.y,
            depth: self.depth + v.depth,
        }
    }
    pub fn sub(&self, v: &Vec2) -> Self {
        Self {
            x: self.x - v.x,
            y: self.y - v.y,
            depth: self.depth - v.depth,
        }
    }
    pub fn dot(&self, v: &Vec2) -> isize {
        self.x * v.x + self.y * v.y
    }

    pub fn length(&self) -> f32 { (self.dot(&self) as f32).sqrt() }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Vec3 { // Used for the location in the world
pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vec3 {
    pub fn add(&self, v: &Vec3) -> Self {
        Self {
            x: self.x + v.x,
            y: self.y + v.y,
            z: self.z + v.z,
        }
    }
    pub fn sub(&self, v: &Vec3) -> Self {
        Self {
            x: self.x - v.x,
            y: self.y - v.y,
            z: self.z - v.z,
        }
    }
    pub fn scale(&self, s: f32) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }
    pub fn dot(&self, v: &Vec3) -> f32 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }
    pub fn cross(&self, v: &Vec3) -> Self {
        Self {
            x: self.y * v.z - self.z * v.y,
            y: self.z * v.x - self.x * v.z,
            z: self.x * v.y - self.y * v.x
        }
    }
    pub fn length(&self) -> f32 { self.dot(&self).sqrt() }
    pub fn normalise(&self) -> Self {
        let l = self.length();
        self.scale(1./l)
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
}

#[derive(Copy, Clone, Debug, Default)]
struct Triangle2D {
    pub a: Vec2,
    pub b: Vec2,
    pub c: Vec2,
}

#[derive(Clone, Debug, Default)]
pub struct Polygon {
    pub triangle: Triangle,
    pub color: u32,
    pub fill: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub polygon_list: Vec<Polygon>,
}
impl Mesh {
    pub fn from_object_file(file_name: &str) -> Self {
        let file_path = String::from("objects/") + file_name;
        let file = match File::open(file_path) {
            Ok(f) => f,
            Err(e) => panic!("Could not open file: {}", e),
        };
        let reader = BufReader::new(file);

        let mut v = Vec::new();
        let mut mesh = Mesh::default();

        for line in reader.lines() {
            let line = line.unwrap();
            let line: Vec<&str> = line.split(' ').collect();

            match line[0] {
                "v" => {
                    let x = line[1].parse::<f32>().unwrap();
                    let y = line[2].parse::<f32>().unwrap();
                    let z = line[3].parse::<f32>().unwrap();

                    v.push(Vec3 { x, y, z });
                },
                "f" => {
                    let a = line[1].split('/').next().unwrap().parse::<usize>().unwrap();
                    let b = line[2].split('/').next().unwrap().parse::<usize>().unwrap();
                    let c = line[3].split('/').next().unwrap().parse::<usize>().unwrap();

                    mesh.polygon_list.push(Polygon { triangle: Triangle { a: v[a - 1], b: v[b - 1], c: v[c - 1] }, color: 0xff_ff_ff_ff, fill: true })
                },
                _ => (),
            }
        }

        mesh
    }
}

// Draws a line between two points based on the bressenham algorithm
fn bresenham_line(window: &mut Window, start: Vec2, end: Vec2, color: u32) {
    let mut start = start;
    let dx = (end.x - start.x).abs();
    let sx = if start.x < end.x { 1 } else { -1 };
    let dy = -(end.y - start.y).abs();
    let sy = if start.y < end.y { 1 } else { -1 };
    let dz = (end.depth - start.depth) / (dx.max(dy) as f32);
    let mut error = dx + dy;
    let mut e2;

    loop {
        if (start.x as usize) < window.width && (start.y as usize) < window.height {
            if start.depth < window.depth_buffer[start.x as usize + start.y as usize * window.width]
            {
                window.buffer[start.x as usize + start.y as usize * window.width] = color;
                window.depth_buffer[start.x as usize + start.y as usize * window.width] = start.depth;
            }
            start.depth += dz;
        }
        if start.x == end.x && start.y == end.y { break; }
        e2 = 2 * error;
        if e2 >= dy {
            if start.x == end.x { break; }
            error += dy;
            start.x += sx;
        }
        if e2 <= dx {
            if start.y == end.y { break; }
            error += dx;
            start.y += sy;
        }
    }
}

fn line_intersect(start: &Vec2, end: &Vec2, line_p: &Vec2, line_n: &Vec2) -> Vec2 {
    let d1 = end.sub(&start);
    let d2 = line_p.sub(&start);
    let t = line_n.dot(&d2) as f32 / line_n.dot(&d1) as f32;
    start.add(&Vec2{x: (d1.x as f32 * t).floor() as isize, y: (d1.y as f32 * t).floor() as isize, depth: d1.depth * t})
}

fn line_intersect_plane(start: &Vec3, end: &Vec3, plane_p: &Vec3, plane_n: &Vec3) -> Vec3 {
    let d1 = end.sub(&start);
    let d2 = plane_p.sub(&start);
    let t = plane_n.dot(&d2) / plane_n.dot(&d1);
    start.add(&d1.scale(t))
}

impl Renderer {
    pub fn new(fov: f32) -> Self {
        Renderer {
            camera: Camera{
                location: Vec3{x: 0., y: 0., z: 0.},
                fov,
                pitch: 0.,
                yaw: 0.,
            },
        }
    }

    pub fn clear_screen(&self, window: &mut Window, color: u32) {
        window.buffer = vec![color; window.width * window.height];
        window.depth_buffer = vec![f32::MAX; window.width * window.height];
    }

    fn draw_line(&self, window: &mut Window, start: Vec2, end: Vec2, color: u32) {
        let mut start = start;
        start.x = clamp(0, start.x, window.width as isize);
        start.y = clamp(0, start.y, window.height as isize);

        let mut end = end;
        end.x = clamp(0, end.x, window.width as isize);
        end.y = clamp(0, end.y, window.height as isize);

        bresenham_line(window, start, end, color);
    }

    // Projects a 3D point on the 2D screen
    fn project(&self, window: &Window, point: Vec3) -> Vec2 {
        let mut point = point;

        // Translate towards camera
        point.x -= self.camera.location.x;
        point.y -= self.camera.location.y;
        point.z -= self.camera.location.z;

        // Rotate around camera yaw and pitch
        self.rotate(&mut point, Vec3{x: 0., y: -self.camera.yaw, z: 0.});
        self.rotate(&mut point, Vec3{x: -self.camera.pitch, y: 0., z: 0.});

        // Calculate projection on the camera
        let tmp = 1. / (point.z * (self.camera.fov * PI / 360.).tan());
        let mut x = point.x * tmp;
        let mut y = -point.y * tmp;

        // Calculate the equivalent projection for the screen
        x *= window.height as f32 / 2.;
        y *= window.height as f32 / 2.;
        x += window.width as f32 / 2.;
        y += window.height as f32 / 2.;

        return Vec2{x: x as isize, y: y as isize, depth: point.z};
    }

    // Rotates a point around (0, 0, 0), angles in degrees
    pub fn rotate(&self, point: &mut Vec3, angle: Vec3) {
        // Calculate rotation angles in radiants
        let dx = angle.x * PI / 180.;
        let dy = angle.y * PI / 180.;
        let dz = angle.z * PI / 180.;

        // Calculate rotation around X-axis
        let mut tmp = dx.cos() * point.y + dx.sin() * point.z;
        point.z = dx.cos() * point.z - dx.sin() * point.y;
        point.y = tmp;

        // Calculate rotation around Y-axis
        tmp = dy.cos() * point.z + dy.sin() * point.x;
        point.x = dy.cos() * point.x - dy.sin() * point.z;
        point.z = tmp;

        // Calculate rotation around Z-axis
        tmp = dz.cos() * point.x + dz.sin() * point.y;
        point.y = dz.cos() * point.y - dz.sin() * point.x;
        point.x = tmp;
    }

    fn draw_triangle(&self, window: &mut Window, triangle: &Triangle, color: u32, fill: bool) {
        // Check whether triangle faces camera
        // Get ray from triangle to camera
        let c = triangle.a.sub(&self.camera.location);
        // Get triangle normal
        let p1 = triangle.b.sub(&triangle.a);
        let p2 = triangle.c.sub(&triangle.a);
        let n = p1.cross(&p2).normalise();
        // Dot product
        if n.dot(&c) > 0. { return; }

        // Lighting
        let light_direction = Vec3{x: 0., y: -1., z: 1.}.normalise();
        let color_scale = (-light_direction.dot(&n)).max(0.1);
        let red = ((((0x00_ff_00_00 & color) >> 16) as f32) * color_scale) as u32;
        let green = ((((0x00_00_ff_00 & color) >> 8) as f32) * color_scale) as u32;
        let blue = (((0x00_00_00_ff & color) as f32) * color_scale) as u32;
        let color =  0x00_00_00_00 | red << 16 | green << 8 | blue;

        // CLip against camera near plane
        let mut plane_n = Vec3{x: 0., y: 0., z: 1.};
        let angles = Vec3{x: self.camera.pitch, y: self.camera.yaw, z: 0.};
        self.rotate(&mut plane_n, angles);
        let plane_p = self.camera.location.add(&plane_n.scale(0.1));
        let (n, clipped) = self.clip_against_plane(triangle.clone(), plane_p, plane_n);

        for i in 0..n {
            let triangle = clipped[i];

            // Get projections of the corners
            let pa = self.project(window, triangle.a);
            let pb = self.project(window, triangle.b);
            let pc = self.project(window, triangle.c);
            let t = Triangle2D { a: pa, b: pb, c: pc };

            // Clipping
            let triangle_list = self.clip_against_screen(t, window.width, window.height);

            for t in triangle_list {
                if fill {
                    // Use the bresenham line algorithm to go draw a line from c to each pixel between a and b
                    // self.bressenham_fill(window, &t, color);
                    // self.scanline_fill(window, &t, color);
                    self.triangle_fill(window, &t, color);

                    // self.draw_line(window, t.a, t.b, 0x_00_ff_00_00);
                    // self.draw_line(window, t.a, t.c, 0x_00_ff_00_00);
                    // self.draw_line(window, t.b, t.c, 0x_00_ff_00_00);
                } else {
                    // Draw the triangle
                    self.draw_line(window, t.a, t.b, color);
                    self.draw_line(window, t.a, t.c, color);
                    self.draw_line(window, t.b, t.c, color);
                }
            }
        }
    }

    fn bressenham_fill(&self, window: &mut Window, triangle: &Triangle2D, color: u32) {
        let mut pa = Vec2{
            x: clamp(0, triangle.a.x, window.width as isize),
            y: clamp(0, triangle.a.y, window.height as isize),
            depth: triangle.a.depth,
        };
        let pb = Vec2{
            x: clamp(0, triangle.b.x, window.width as isize),
            y: clamp(0, triangle.b.y, window.height as isize),
            depth: triangle.a.depth,
        };
        let pc = Vec2{
            x: clamp(0, triangle.c.x, window.width as isize),
            y: clamp(0, triangle.c.y, window.height as isize),
            depth: triangle.a.depth,
        };

        let dx = (pb.x - pa.x).abs();
        let sx = if pa.x < pb.x { 1 } else { -1 };
        let dy = -(pb.y - pa.y).abs();
        let sy = if pa.y < pb.y { 1 } else { -1 };
        let dz = (pb.depth - pa.depth) / (dx.max(dy) as f32);
        let mut error = dx + dy;
        let mut e2;

        loop {
            if (pa.x as usize) < window.width && (pa.y as usize) < window.height {
                self.draw_line(window, pc, pa.clone(), color);
                pa.depth += dz;
            }
            if pa.x == pb.x && pa.y == pb.y { break; }
            e2 = 2 * error;
            if e2 >= dy {
                if pa.x == pb.x { break; }
                error += dy;
                pa.x += sx;
            }
            if e2 <= dx {
                if pa.y == pb.y { break; }
                error += dx;
                pa.y += sy;
            }
        }
    }

    fn scanline_fill(&self, window: &mut Window, triangle: &Triangle2D, color: u32) {
        // Note: No depth buffer implemented
        let mut pa = Vec2{
            x: clamp(0, triangle.a.x, window.width as isize),
            y: clamp(0, triangle.a.y, window.height as isize),
            depth: triangle.a.depth,
        };
        let pb = Vec2{
            x: clamp(0, triangle.b.x, window.width as isize),
            y: clamp(0, triangle.b.y, window.height as isize),
            depth: triangle.a.depth,
        };
        let pc = Vec2{
            x: clamp(0, triangle.c.x, window.width as isize),
            y: clamp(0, triangle.c.y, window.height as isize),
            depth: triangle.a.depth,
        };

        let min_x = pa.x.min(pb.x).min(pc.x);
        let max_x = pa.x.max(pb.x).max(pc.x);
        let min_y = pa.y.min(pb.y).min(pc.y);
        let max_y = pa.y.max(pb.y).max(pc.y);

        // Calculate area
        // let area = 0.5 * (x3 * -y2 + y1 * (x3 - x2) + x1 * (y2 - y3) + x2 * y3) as f32;

        // Iterate over each pixel in the grid
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                // Calculate the barycentric coordinates of the pixel
                let a = pc.y - pa.y;
                let b = y - triangle.a.y;
                let c = pc.x - pa.x;
                let d = pb.y - pa.y;
                let e = pb.x - pa.x;

                let w1 = (pa.x * a + b * c - x * a) as f32 / (d * c - e * a) as f32;
                let w2 = (b as f32 - w1 * d as f32) / a as f32;
                let w3 = 1. - w1 - w2;

                // let w1 = (0.5 * (-y2 * x3 + y * (-x2 + x3) + x * (y2 - y3) + x2 * y3) as f32) / area;
                // let w2 = (0.5 * (y1 * x3 + y * (x1 - x3) + x * (y3 - y1) + x1 * -y3) as f32) / area;
                // let w3 = 1. - w1 - w2;

                // Check if the pixel is inside the triangle
                if w1 >= 0. && w2 >= 0. && w3 >= 0. && x >= 0 && x < window.width as isize && y >= 0 && y < window.height as isize  {
                    // Fill the pixel with a character
                    window.buffer[x as usize + y as usize * window.width] = color;
                }
            }
        }
    }

    fn triangle_fill(&self, window: &mut Window, triangle: &Triangle2D, color: u32) {
        let mut pa = Vec2{
            x: clamp(0, triangle.a.x, window.width as isize),
            y: clamp(0, triangle.a.y, window.height as isize),
            depth: triangle.a.depth,
        };
        let mut pb = Vec2{
            x: clamp(0, triangle.b.x, window.width as isize),
            y: clamp(0, triangle.b.y, window.height as isize),
            depth: triangle.b.depth,
        };
        let mut pc = Vec2{
            x: clamp(0, triangle.c.x, window.width as isize),
            y: clamp(0, triangle.c.y, window.height as isize),
            depth: triangle.c.depth,
        };

        if pb.y < pa.y {
            swap(&mut pa, &mut pb);
        }
        if pc.y < pa.y {
            swap(&mut pc, &mut pa);
        }
        if pc.y < pb.y {
            swap(&mut pc, &mut pb);
        }

        let dxab = pb.x - pa.x;
        let dxbc = pc.x - pb.x;
        let dxac = pc.x - pa.x;

        let dyab = pb.y - pa.y;
        let dybc = pc.y - pb.y;
        let dyac = pc.y - pa.y;

        let dzab = pb.depth - pa.depth;
        let dzbc = pc.depth - pb.depth;
        let dzac = pc.depth - pa.depth;

        let dxab = if dyab != 0 { dxab as f32 / dyab as f32 } else { 0. };
        let dxbc = if dybc != 0 { dxbc as f32 / dybc as f32 } else { 0. };
        let dxac = if dyac != 0 { dxac as f32 / dyac as f32 } else { 0. };

        let dzab = if dyab != 0 { dzab as f32 / dyab as f32 } else { 0. };
        let dzbc = if dybc != 0 { dzbc as f32 / dybc as f32 } else { 0. };
        let dzac = if dyac != 0 { dzac as f32 / dyac as f32 } else { 0. };

        let mut xac = pa.x as f32;
        let mut xabc = pa.x as f32;

        let mut zac = pa.depth;
        let mut zabc = pa.depth;

        for y in pa.y..pb.y {
            let (x1, x2, z1, z2) = if xac < xabc {
                (xac.floor() as usize, xabc.floor() as usize, zac, zabc)
            } else {
                (xabc.floor() as usize, xac.floor() as usize, zabc, zac)
            };
            let dz = (z2 - z1) / ((x2 - x1) as f32);
            let mut z = z1;
            for x in x1..x2 {
                if z < window.depth_buffer[x + y as usize * window.width] {
                    window.buffer[x + y as usize * window.width] = color;
                    window.depth_buffer[x + y as usize * window.width] = z
                }
                z += dz
            }
            xac += dxac;
            xabc += dxab;
            zac += dzac;
            zabc += dzab;
        }
        xabc = pb.x as f32;
        zabc = pb.depth;
        for y in pb.y..pc.y {
            let (x1, x2, z1, z2) = if xac < xabc {
                (xac.floor() as usize, xabc.floor() as usize, zac, zabc)
            } else {
                (xabc.floor() as usize, xac.floor() as usize, zabc, zac)
            };
            let dz = (z2 - z1) / ((x2 - x1) as f32);
            let mut z = z1;
            for x in x1..x2 {
                if z < window.depth_buffer[x + y as usize * window.width] {
                    window.buffer[x + y as usize * window.width] = color;
                    window.depth_buffer[x + y as usize * window.width] = z
                }
                z += dz
            }
            xac += dxac;
            xabc += dxbc;
        }
    }

    fn clip_against_screen(&self, triangle: Triangle2D, width: usize, height: usize) -> VecDeque<Triangle2D> {
        let mut triangle_list = VecDeque::new();
        triangle_list.push_back(triangle);
        let mut num_triangles = 1;

        for p in 0..4 { // For each border of the screen
            while num_triangles > 0 {
                let t = triangle_list.pop_front().unwrap();
                num_triangles -= 1;

                let (num, clipped) = match p {
                    0 => self.clip_against_line(t, Vec2{x: 0, y: 0, depth: 0.}, Vec2{x: 1, y: 0, depth: 0.}), // Left
                    1 => self.clip_against_line(t, Vec2{x: width as isize, y: 0, depth: 0.}, Vec2{x: -1, y: 0, depth: 0.}), // Right
                    2 => self.clip_against_line(t, Vec2{x: 0, y: 0, depth: 0.}, Vec2{x: 0, y: 1, depth: 0.}), // Top
                    3 => self.clip_against_line(t, Vec2{x: 0, y: height as isize, depth: 0.}, Vec2{x: 0, y: -1, depth: 0.}), // Bottom
                    _ => panic!("Unreachable"),
                };

                for w in 0..num {
                    triangle_list.push_back(clipped[w]);
                }
            }
            num_triangles = triangle_list.len();
        }

        triangle_list
    }

    fn clip_against_line(&self, triangle: Triangle2D, line_p: Vec2, line_n: Vec2) -> (usize, [Triangle2D; 2]) {
        let mut clipped = [Triangle2D::default(); 2];

        // determine inside/outside points
        let mut num_outside = 0;
        let mut num_inside = 0;
        let mut outside = [Vec2::default(); 3];
        let mut inside = [Vec2::default(); 3];
        if triangle.a.sub(&line_p).dot(&line_n) < 0 {
            outside[num_outside] = triangle.a;
            num_outside += 1;
        } else {
            inside[num_inside] = triangle.a;
            num_inside += 1;
        }
        if triangle.b.sub(&line_p).dot(&line_n) < 0 {
            outside[num_outside] = triangle.b;
            num_outside += 1;
        } else {
            inside[num_inside] = triangle.b;
            num_inside += 1;
        }
        if triangle.c.sub(&line_p).dot(&line_n) < 0 {
            outside[num_outside] = triangle.c;
            num_outside += 1;
        } else {
            inside[num_inside] = triangle.c;
            num_inside += 1;
        }

        match num_outside {
            0 => { // No clipping needed, returning triangle
                clipped[0] = triangle;
                (1, clipped)
            },
            1 => { // Clipping into two triangles
                // calculate intersection points
                let p1 = line_intersect(&inside[0], &outside[0], &line_p, &line_n);
                let p2 = line_intersect(&inside[1], &outside[0], &line_p, &line_n);

                // construct clipped triangles
                clipped[0] = Triangle2D{a: inside[0], b: inside[1], c: p1};
                clipped[1] = Triangle2D{a: inside[1], b: p1, c: p2};

                (2, clipped)
            },
            2 => { // Clipping into one triangle
                // calculate intersection points
                let p1 = line_intersect(&inside[0], &outside[0], &line_p, &line_n);
                let p2 = line_intersect(&inside[0], &outside[1], &line_p, &line_n);

                // construct clipped triangles
                clipped[0] = Triangle2D{a: inside[0], b: p1, c: p2};

                (1, clipped)
            },
            3 => (0, clipped), // Triangle completely clipped
            _ => panic!("Unreachable")
        }
    }

    fn clip_against_plane(&self, triangle: Triangle, line_p: Vec3, line_n: Vec3) -> (usize, [Triangle; 2]) {
        let mut clipped = [Triangle::default(); 2];

        // determine inside/outside points
        let mut num_outside = 0;
        let mut num_inside = 0;
        let mut outside = [Vec3::default(); 3];
        let mut inside = [Vec3::default(); 3];
        if triangle.a.sub(&line_p).dot(&line_n) < 0. {
            outside[num_outside] = triangle.a;
            num_outside += 1;
        } else {
            inside[num_inside] = triangle.a;
            num_inside += 1;
        }
        if triangle.b.sub(&line_p).dot(&line_n) < 0. {
            outside[num_outside] = triangle.b;
            num_outside += 1;
        } else {
            inside[num_inside] = triangle.b;
            num_inside += 1;
        }
        if triangle.c.sub(&line_p).dot(&line_n) < 0. {
            outside[num_outside] = triangle.c;
            num_outside += 1;
        } else {
            inside[num_inside] = triangle.c;
            num_inside += 1;
        }

        match num_outside {
            0 => { // No clipping needed, returning triangle
                clipped[0] = triangle;
                (1, clipped)
            },
            1 => {
                // Calculate intersection points
                let p1 = line_intersect_plane(&inside[0], &outside[0], &line_p, &line_n);
                let p2 = line_intersect_plane(&inside[1], &outside[0], &line_p, &line_n);

                // Construct clipped triangles
                clipped[0] = Triangle{a: inside[0], b: inside[1], c: p1};
                clipped[1] = Triangle{a: inside[1], b: p1, c: p2};

                (2, clipped)
            }, // Clipping into two triangles
            2 => {
                // Calculate intersection points
                let p1 = line_intersect_plane(&inside[0], &outside[0], &line_p, &line_n);
                let p2 = line_intersect_plane(&inside[0], &outside[1], &line_p, &line_n);

                // Construct clipped triangles
                clipped[0] = Triangle{a: inside[0], b: p1, c: p2};

                (1, clipped)

            }, // Clipping into one triangle
            3 => (0, clipped), // Triangle completely clipped
            _ => panic!("Unreachable"),
        }
    }

    // Rotates a triangle around a centroid
    fn rotate_triangle(&self, triangle: &mut Triangle, centroid: Vec3, angle: Vec3) {
        // Translate to (0, 0, 0);
        triangle.a.x -= centroid.x;
        triangle.a.y -= centroid.y;
        triangle.a.z -= centroid.z;

        triangle.b.x -= centroid.x;
        triangle.b.y -= centroid.y;
        triangle.b.z -= centroid.z;

        triangle.c.x -= centroid.x;
        triangle.c.y -= centroid.y;
        triangle.c.z -= centroid.z;

        // Rotate
        self.rotate(&mut triangle.a, angle);
        self.rotate(&mut triangle.b, angle);
        self.rotate(&mut triangle.c, angle);

        // Translate back to original position
        triangle.a.x += centroid.x;
        triangle.a.y += centroid.y;
        triangle.a.z += centroid.z;

        triangle.b.x += centroid.x;
        triangle.b.y += centroid.y;
        triangle.b.z += centroid.z;

        triangle.c.x += centroid.x;
        triangle.c.y += centroid.y;
        triangle.c.z += centroid.z;
    }

    pub fn draw_mesh(&self, window: &mut Window, mesh: &Mesh) {
        for p in &mesh.polygon_list {
            self.draw_triangle(window, &p.triangle, p.color, p.fill);
        }
    }

    // Rotate a mesh around a centroid
    pub fn rotate_mesh(&self, mesh: &mut Mesh, angle: Vec3) {
        let mut centroid = Vec3::default();
        for p in &mesh.polygon_list {
            centroid.x += p.triangle.a.x; centroid.x += p.triangle.b.x; centroid.x += p.triangle.c.x;
            centroid.y += p.triangle.a.y; centroid.y += p.triangle.b.y; centroid.y += p.triangle.c.y;
            centroid.z += p.triangle.a.z; centroid.z += p.triangle.b.z; centroid.z += p.triangle.c.z;
        }
        centroid.x /= mesh.polygon_list.len() as f32 * 3.;
        centroid.y /= mesh.polygon_list.len() as f32 * 3.;
        centroid.z /= mesh.polygon_list.len() as f32 * 3.;

        for p in &mut mesh.polygon_list {
            self.rotate_triangle(&mut p.triangle, centroid, angle);
        }
    }

    // Sort polygons based on the distance to the camera. Furthest polygons first
    pub fn depth_sort_mesh(&self, mesh: &mut Mesh) {
        mesh.polygon_list.sort_by(|a, b| self.compare_depth(a, b));
    }

    fn compare_depth(&self, p1: &Polygon, p2: &Polygon) -> Ordering {
        let d1 = self.calc_depth(p1);
        let d2 = self.calc_depth(p2);
        d2.partial_cmp(&d1).unwrap()
    }

    // Calculate the squared distance to the camera
    fn calc_depth(&self, p: &Polygon) -> f32 {
        let tmp = Vec3 {
            x: p.triangle.a.x + p.triangle.b.x + p.triangle.c.x - 3. * self.camera.location.x,
            y: p.triangle.a.y + p.triangle.b.y + p.triangle.c.y - 3. * self.camera.location.y,
            z: p.triangle.a.z + p.triangle.b.z + p.triangle.c.z - 3. * self.camera.location.z,
        };
        tmp.x * tmp.x + tmp.y * tmp.y + tmp.z * tmp.z
    }

    pub fn translate_mesh(&self, mesh: &mut Mesh, translate: Vec3) {
        for p in &mut mesh.polygon_list {
            p.triangle.a = p.triangle.a.add(&translate);
            p.triangle.b = p.triangle.b.add(&translate);
            p.triangle.c = p.triangle.c.add(&translate);
        }
    }
}