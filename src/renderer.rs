use std::cmp::Ordering;
use std::collections::VecDeque;
use minifb::clamp;
use crate::window::Window;
use std::f32::consts::PI;
use std::fs::File;
use std::io::{BufRead, BufReader};

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
}
impl Vec2 {
    pub fn add(&self, v: &Vec2) -> Self {
        Self {
            x: self.x + v.x,
            y: self.y + v.y,
        }
    }
    pub fn sub(&self, v: &Vec2) -> Self {
        Self {
            x: self.x - v.x,
            y: self.y - v.y,
        }
    }
    pub fn dot(&self, v: &Vec2) -> isize {
        self.x * v.x + self.y * v.y
    }
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
                    let a = line[1].parse::<usize>().unwrap();
                    let b = line[2].parse::<usize>().unwrap();
                    let c = line[3].parse::<usize>().unwrap();
                    mesh.polygon_list.push(Polygon{triangle: Triangle { a: v[a-1], b: v[b-1], c: v[c-1]}, color: 0xff_ff_ff_ff, fill: true})
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
    let mut error = dx + dy;
    let mut e2;

    loop {
        if (start.x as usize) < window.width && (start.y as usize) < window.height {
            window.buffer[start.x as usize + start.y as usize * window.width] = color;
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
    start.add(&Vec2{x: (d1.x as f32 * t).floor() as isize, y: (d1.y as f32 * t).floor() as isize})
}

impl Renderer {
    pub fn new(fov: f32) -> Self {
        Renderer {
            camera: Camera{
                location: Vec3{x: 0., y: 0., z: 0.},
                fov,
                pitch: 0.,
                yaw: 0.,
            }
        }
    }

    pub fn clear_screen(&self, window: &mut Window, color: u32) {
        let buffer: Vec<u32> = vec![color; window.width * window.height];
        window.buffer = buffer;
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

        return Vec2{x: x as isize, y: y as isize};
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
        let n = p1.cross(&p2);
        // Dot product
        if n.dot(&c) > 0. { return; }

        // Get projections of the corners
        let  pa = self.project(window, triangle.a);
        let pb = self.project(window, triangle.b);
        let pc = self.project(window, triangle.c);
        let t = Triangle2D{a: pa, b: pb, c: pc};
        // Clipping
        let triangle_list = self.clip_against_screen(t, window.width, window.height);
        for t in triangle_list {
            let mut pa = t.a;
            let pb = t.b;
            let pc = t.c;

            // Draw the triangle
            self.draw_line(window, pa, pb, color);
            self.draw_line(window, pa, pc, color);
            self.draw_line(window, pb, pc, color);

            if !fill { return; }

            // Use the bresenham line algorithm to go draw a line from c to each pixel between a and b
            let dx = (pb.x - pa.x).abs();
            let sx = if pa.x < pb.x { 1 } else { -1 };
            let dy = -(pb.y - pa.y).abs();
            let sy = if pa.y < pb.y { 1 } else { -1 };
            let mut error = dx + dy;
            let mut e2;

            loop {
                if (pa.x as usize) < window.width && (pa.y as usize) < window.height {
                    self.draw_line(window, pc, Vec2 { x: pa.x, y: pa.y }, color);
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
                    0 => self.clip_against_line(t, Vec2{x: 0, y: 0}, Vec2{x: 1, y: 0}), // Left
                    1 => self.clip_against_line(t, Vec2{x: width as isize, y: 0}, Vec2{x: -1, y: 0}), // Right
                    2 => self.clip_against_line(t, Vec2{x: 0, y: 0}, Vec2{x: 0, y: 1}), // Top
                    3 => self.clip_against_line(t, Vec2{x: 0, y: height as isize}, Vec2{x: 0, y: -1}), // Bottom
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
}