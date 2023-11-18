use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::shapes::vec2::Vec2;
use crate::shapes::vec3::Vec3;


#[derive(Copy, Clone, Debug, Default)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Triangle2D {
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
