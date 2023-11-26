use std::fs::File;
use std::io::{BufRead, BufReader};
use cube::shapes::vec_tex::VecTex;
use crate::shapes::vec3::{Vec3, Triangle};
use crate::shapes::vec_tex::TriangleTex;

#[derive(Clone, Debug, Default)]
pub struct Polygon {
    pub triangle: Triangle,
    pub triangle_tex: TriangleTex,
    pub color: u32,
    pub fill: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub polygon_list: Vec<Polygon>,
    pub texture: Option<Vec<u32>>,
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
        let mut vt = Vec::new();
        let mut mesh = Mesh::default();

        for line in reader.lines() {
            let line = line.unwrap();
            let line: Vec<&str> = line.split(' ').collect();

            match line[0] {
                "v" => {
                    let x = line[1].parse::<f32>().unwrap();
                    let y = line[2].parse::<f32>().unwrap();
                    let z = line[3].parse::<f32>().unwrap();

                    v.push(Vec3{x, y, z});
                },
                "vt" => {
                    let x = line[1].parse::<f32>().unwrap();
                    let y = line[2].parse::<f32>().unwrap();

                    vt.push(VecTex{x, y})
                }
                "f" => {
                    let parse = |s: &str| -> (usize, Option<usize>) {
                        let s: Vec<&str> = s.split('/').collect();
                        let a = s[0].parse::<usize>().unwrap();
                        let at = s.get(1).map(|&a| a.parse::<usize>().unwrap());

                        (a, at)
                    };

                    let (a, at) = parse(line[1]);
                    let (b, bt) = parse(line[2]);
                    let (c, ct) = parse(line[3]);

                    let triangle_tex = if let Some(at) = at {
                        TriangleTex{a: vt[at - 1], b: vt[bt.unwrap() - 1], c: vt[ct.unwrap() - 1]}
                    } else { TriangleTex::default()};

                    mesh.polygon_list.push(Polygon{
                        triangle: Triangle { a: v[a - 1], b: v[b - 1], c: v[c - 1] },
                        triangle_tex,
                        color: 0xff_ff_ff_ff,
                        fill: true })
                },
                _ => (),
            }
        }

        mesh
    }
}
