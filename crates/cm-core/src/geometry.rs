use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

/// A 2D point. Coordinates are f64 in the project's unit system.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn distance_to(self, other: Point2D) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl Add<Vector2D> for Point2D {
    type Output = Point2D;
    fn add(self, v: Vector2D) -> Point2D {
        Point2D {
            x: self.x + v.x,
            y: self.y + v.y,
        }
    }
}

impl Sub for Point2D {
    type Output = Vector2D;
    fn sub(self, other: Point2D) -> Vector2D {
        Vector2D {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

/// A 2D vector.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector2D {
    pub x: f64,
    pub y: f64,
}

impl Vector2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

/// An axis-aligned rectangle, the fundamental shape for cabinet panels.
/// Origin is at the bottom-left corner.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub origin: Point2D,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn new(origin: Point2D, width: f64, height: f64) -> Self {
        Self {
            origin,
            width,
            height,
        }
    }

    pub fn from_dimensions(width: f64, height: f64) -> Self {
        Self {
            origin: Point2D::origin(),
            width,
            height,
        }
    }

    pub fn area(&self) -> f64 {
        self.width * self.height
    }

    pub fn min_x(&self) -> f64 {
        self.origin.x
    }

    pub fn min_y(&self) -> f64 {
        self.origin.y
    }

    pub fn max_x(&self) -> f64 {
        self.origin.x + self.width
    }

    pub fn max_y(&self) -> f64 {
        self.origin.y + self.height
    }

    pub fn center(&self) -> Point2D {
        Point2D {
            x: self.origin.x + self.width / 2.0,
            y: self.origin.y + self.height / 2.0,
        }
    }

    /// The four corners: bottom-left, bottom-right, top-right, top-left.
    pub fn corners(&self) -> [Point2D; 4] {
        [
            self.origin,
            Point2D::new(self.max_x(), self.min_y()),
            Point2D::new(self.max_x(), self.max_y()),
            Point2D::new(self.min_x(), self.max_y()),
        ]
    }

    /// Check if this rect fits inside another rect (for nesting).
    pub fn fits_inside(&self, other: &Rect) -> bool {
        self.width <= other.width && self.height <= other.height
    }

    /// Check if this rect fits inside another rect when rotated 90 degrees.
    pub fn fits_inside_rotated(&self, other: &Rect) -> bool {
        self.height <= other.width && self.width <= other.height
    }
}

/// A line segment between two points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineSegment {
    pub start: Point2D,
    pub end: Point2D,
}

impl LineSegment {
    pub fn new(start: Point2D, end: Point2D) -> Self {
        Self { start, end }
    }

    pub fn length(&self) -> f64 {
        self.start.distance_to(self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_distance() {
        let a = Point2D::new(0.0, 0.0);
        let b = Point2D::new(3.0, 4.0);
        assert!((a.distance_to(b) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_rect_area() {
        let r = Rect::from_dimensions(36.0, 30.0);
        assert!((r.area() - 1080.0).abs() < 1e-10);
    }

    #[test]
    fn test_rect_corners() {
        let r = Rect::new(Point2D::new(1.0, 2.0), 10.0, 5.0);
        let c = r.corners();
        assert_eq!(c[0], Point2D::new(1.0, 2.0));
        assert_eq!(c[1], Point2D::new(11.0, 2.0));
        assert_eq!(c[2], Point2D::new(11.0, 7.0));
        assert_eq!(c[3], Point2D::new(1.0, 7.0));
    }

    #[test]
    fn test_rect_fits_inside() {
        let small = Rect::from_dimensions(10.0, 5.0);
        let big = Rect::from_dimensions(48.0, 96.0);
        assert!(small.fits_inside(&big));
        assert!(!big.fits_inside(&small));
    }

    #[test]
    fn test_rect_fits_inside_rotated() {
        let panel = Rect::from_dimensions(30.0, 12.0);
        let bed = Rect::from_dimensions(18.0, 9.5);
        assert!(!panel.fits_inside(&bed));
        // 12 x 30 rotated: 12 <= 18, 30 <= 9.5? No.
        assert!(!panel.fits_inside_rotated(&bed));

        let small = Rect::from_dimensions(8.0, 15.0);
        // 15 x 8: 15 <= 18 yes, 8 <= 9.5 yes
        assert!(small.fits_inside_rotated(&bed));
    }
}
