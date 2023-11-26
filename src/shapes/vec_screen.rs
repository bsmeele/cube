use minifb::clamp;

#[derive(Copy, Clone, Debug, Default)]
pub struct TriangleScreen {
    pub a: VecScreen,
    pub b: VecScreen,
    pub c: VecScreen,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct VecScreen { // Used for the location on the screen, meaning positive y is down
    pub x: isize,
    pub y: isize,
    pub depth: f32,
}
#[allow(dead_code)]
impl VecScreen {
    pub fn add(&self, v: &Self) -> Self {
        Self {
            x: self.x + v.x,
            y: self.y + v.y,
            depth: self.depth + v.depth,
        }
    }
    pub fn sub(&self, v: &Self) -> Self {
        Self {
            x: self.x - v.x,
            y: self.y - v.y,
            depth: self.depth - v.depth,
        }
    }
    pub fn dot(&self, v: &Self) -> isize {
        self.x * v.x + self.y * v.y
    }

    pub fn length(&self) -> f32 { (self.dot(&self) as f32).sqrt() }

    pub fn clamp_screen(&self, width: isize, height: isize) -> Self {
        Self{
            x: clamp(0, self.x, width),
            y: clamp(0, self.y, height),
            depth: self.depth,
        }
    }
}