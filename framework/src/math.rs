use core::f32;

use glam::{vec2, Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Box2 {
    pub min: Vec2,
    pub max: Vec2,
}

impl Box2 {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn empty() -> Self {
        Self {
            min: vec2(f32::INFINITY, f32::INFINITY),
            max: vec2(f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    pub fn include_point(&mut self, p: Vec2) -> &mut Self {
        self.min = self.min.min(p);
        self.max = self.max.max(p);
        self
    }

    pub fn include_box(&mut self, other: Self) -> &mut Self {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
        self
    }

    pub fn contains_point(&self, p: &Vec2) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }
}

pub fn intersect_box2_box2(a: &Box2, b: &Box2) -> bool {
    a.max.x >= b.min.x && a.min.x <= b.max.x && a.max.y >= b.min.y && a.min.y <= b.max.y
}

#[cfg(test)]
mod tests {
    use glam::{Vec2, vec2};

    use crate::math::{Box2, intersect_box2_box2};

    #[test]
    fn test_include_point() {
        let mut b = Box2::new(vec2(0.0, 0.0), vec2(1.0, 1.0));
        let p = Vec2::new(2.0, 2.0);

        assert!(!b.contains_point(&p));

        b.include_point(p);
        
        assert_eq!(b, Box2::new(vec2(0.0, 0.0), p));
        assert!(b.contains_point(&p));
    }

    #[test]
    fn test_include_box() {
        let a = Box2::new(vec2(-1.0, -1.0), vec2(0.0, 0.0));
        let b = Box2::new(vec2(0.0, 0.0), vec2(1.0, 1.0));
        let mut c = Box2::empty();

        c.include_box(a);
        c.include_box(b);

        assert!(intersect_box2_box2(&a, &c));
        assert!(intersect_box2_box2(&b, &c));
    }

    #[test]
    fn test_intersect_box2_box2() {
        let a = Box2::new(vec2(-0.5, -0.5), vec2(0.25, 0.25));
        let b = Box2::new(vec2(-0.25, -0.25), vec2(0.5, 0.5));

        assert!(intersect_box2_box2(&a, &b));
    }
}