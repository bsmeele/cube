#[derive(Copy, Clone, Debug, Default)]
pub struct TriangleTex {
    pub a: VecTex,
    pub b: VecTex,
    pub c: VecTex,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct VecTex { // Used for the location on the screen, meaning positive y is down
    pub x: f32,
    pub y: f32,
}
