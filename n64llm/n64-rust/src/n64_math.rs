// n64_math.rs
// Basic math functions needed for N64 development

pub const PI: f32 = 3.14159265358979323846;

pub fn deg_to_rad(degrees: f32) -> f32 {
    degrees * (PI / 180.0)
}

pub fn rad_to_deg(radians: f32) -> f32 {
    radians * (180.0 / PI)
}

pub fn sin_approx(x: f32) -> f32 {
    let x2 = x * x;
    x * (1.0 - x2 / 6.0 * (1.0 - x2 / 20.0))
}

pub fn cos_approx(x: f32) -> f32 {
    let x2 = x * x;
    1.0 - x2 / 2.0 * (1.0 - x2 / 12.0)
}

pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }
    
    pub fn dot(&self, other: &Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }
    
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }
    
    pub fn length(&self) -> f32 {
        // Call sqrt directly since we’re in the same module.
        sqrt(self.length_squared())
    }
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }
    
    pub fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    
    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
    
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
    
    pub fn length(&self) -> f32 {
        sqrt(self.length_squared())
    }
}

// One implementation of sqrt using Newton's method.
pub fn sqrt(x: f32) -> f32 {
    let mut z = x / 2.0;
    let mut prev_z = 0.0;
    while abs_custom(z - prev_z) > 0.0001 {
        prev_z = z;
        z = z - (z * z - x) / (2.0 * z);
    }
    z
}

// Custom absolute value function.
pub fn abs(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

pub fn abs_custom(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

// Approximate e^x using a few terms of its Taylor series. This is
// sufficient for the softmax used in the toy transformer implementation
// and avoids pulling in a heavy math library.
pub fn exp_approx(x: f32) -> f32 {
    let mut term = 1.0f32;
    let mut sum = 1.0f32;
    // Compute 4 additional terms of the series
    for i in 1..5 {
        term *= x / (i as f32);
        sum += term;
    }
    sum
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deg_rad_roundtrip() {
        let d = 90.0f32;
        let r = deg_to_rad(d);
        assert!((rad_to_deg(r) - d).abs() < 0.01);
    }

    #[test]
    fn vec2_length() {
        let v = Vec2::new(3.0, 4.0);
        assert!((v.length() - 5.0).abs() < 0.01);
    }
}
