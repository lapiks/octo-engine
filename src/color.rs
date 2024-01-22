use std::ops::{Add, Mul, Sub, AddAssign, SubAssign, MulAssign, Div, DivAssign};

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self {
            r, g, b
        }
    }

    pub fn white() -> Color {
        Color::new(1.0, 1.0, 1.0)
    }

    pub fn black() -> Color {
        Color::new(0.0, 0.0, 0.0)
    }

    pub fn red() -> Color {
        Color::new(1.0, 0.0, 0.0)
    }

    pub fn green() -> Color {
        Color::new(0.0, 1.0, 0.0)
    }

    pub fn blue() -> Color {
        Color::new(0.0, 0.0, 1.0)
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::black()
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Color) -> bool {
            (self.r - other.r).abs() < 1.0e-3 &&
            (self.g - other.g).abs() < 1.0e-3 &&
            (self.b - other.b).abs() < 1.0e-3
    }
}

impl Add for Color {
    type Output = Color;

    fn add(self, rhs: Self) -> Self::Output {
        Color {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b
        }
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}

impl Sub for Color {
    type Output = Color;

    fn sub(self, rhs: Self) -> Self::Output {
        Color {
            r: self.r - rhs.r,
            g: self.g - rhs.g,
            b: self.b - rhs.b
        }
    }
}

impl SubAssign for Color {
    fn sub_assign(&mut self, rhs: Self) {
        self.r -= rhs.r;
        self.g -= rhs.g;
        self.b -= rhs.b;
    }
}

impl Mul for Color {
    type Output = Color;

    fn mul(self, rhs: Self) -> Self::Output {
        Color {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b
        }
    }
}

impl MulAssign for Color {

    fn mul_assign(&mut self, rhs: Self) {
        self.r *= rhs.r;
        self.g *= rhs.g;
        self.b *= rhs.b;
    }
}

impl Mul<f64> for Color {
    type Output = Color;

    fn mul(self, rhs: f64) -> Self::Output {
        Color {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs
        }
    }
}

impl MulAssign<f64> for Color {
    fn mul_assign(&mut self, rhs: f64) {
        self.r *= rhs;
        self.g *= rhs;
        self.b *= rhs;
    }
}

impl Div for Color {
    type Output = Color;

    fn div(self, rhs: Self) -> Self::Output {
        Color {
            r: self.r / rhs.r,
            g: self.g / rhs.g,
            b: self.b / rhs.b
        }
    }
}

impl DivAssign for Color {

    fn div_assign(&mut self, rhs: Self) {
        self.r /= rhs.r;
        self.g /= rhs.g;
        self.b /= rhs.b;
    }
}

impl Div<f64> for Color {
    type Output = Color;

    fn div(self, rhs: f64) -> Self::Output {
        Color {
            r: self.r / rhs,
            g: self.g / rhs,
            b: self.b / rhs
        }
    }
}

impl DivAssign<f64> for Color {
    fn div_assign(&mut self, rhs: f64) {
        self.r /= rhs;
        self.g /= rhs;
        self.b /= rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_color() {
        let c = Color::new(-0.5, 0.4, 1.7);
        assert_eq!(c.r, -0.5);
        assert_eq!(c.g, 0.4);
        assert_eq!(c.b, 1.7);
    }

    #[test]
    fn adding_colors() {
        let c1 = Color::new(0.9, 0.6, 0.75);
        let c2 = Color::new(0.7, 0.1, 0.25);
        let c3 = c1 + c2;
        assert_eq!(c3, Color::new(1.6, 0.7, 1.0));
    }

    #[test]
    fn add_assign_colors() {
        let mut c1 = Color::new(0.9, 0.6, 0.75);
        let c2 = Color::new(0.7, 0.1, 0.25);
        c1 += c2;
        assert_eq!(c1, Color::new(1.6, 0.7, 1.0));
    }

    #[test]
    fn substracting_colors() {
        let c1 = Color::new(0.9, 0.6, 0.75);
        let c2 = Color::new(0.7, 0.1, 0.25);
        let c3 = c1 - c2;
        assert_eq!(c3, Color::new(0.2, 0.5, 0.5));
    }

    #[test]
    fn sub_assign_colors() {
        let mut c1 = Color::new(0.9, 0.6, 0.75);
        let c2 = Color::new(0.7, 0.1, 0.25);
        c1 -= c2;
        assert_eq!(c1, Color::new(0.2, 0.5, 0.5));
    }

    #[test]
    fn multiplying_color_by_a_scalar() {
        let c = Color::new(0.2, 0.3, 0.4);
        let c2 = c * 2.0;
        assert_eq!(c2, Color::new(0.4, 0.6, 0.8));
    }

    #[test]
    fn mul_assign_color_by_a_scalar() {
        let mut c = Color::new(0.2, 0.3, 0.4);
        c *= 2.0;
        assert_eq!(c, Color::new(0.4, 0.6, 0.8));
    }

    #[test]
    fn multiplying_colors() {
        let c1 = Color::new(1.0, 0.2, 0.4);
        let c2 = Color::new(0.9, 1.0, 0.1);
        let c3 = c1 * c2;
        assert_eq!(c3, Color::new(0.9, 0.2, 0.04));
    }

    #[test]
    fn mul_assign_colors() {
        let mut c1 = Color::new(1.0, 0.2, 0.4);
        let c2 = Color::new(0.9, 1.0, 0.1);
        c1 *= c2;
        assert_eq!(c1, Color::new(0.9, 0.2, 0.04));
    }
}