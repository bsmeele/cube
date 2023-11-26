#[derive(Copy, Clone, Debug, Default)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Vec3 { // Used for the location in the world
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
#[allow(dead_code)]
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
