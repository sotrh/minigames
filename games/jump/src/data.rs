use framework::glam::{Vec2, vec2};

#[derive(Debug, Default)]
pub struct Inputs {
    pub(crate) left: f32,
    pub(crate) right: f32,
}

#[derive(Debug)]
pub struct Player {
    pub position: Vec2,
    pub size: Vec2,
    pub velocity: Vec2,
}

#[derive(Debug)]
pub struct Platform {
    pub position: Vec2,
    pub size: Vec2,
    pub breakable: bool,
    pub bounciness: f32,
}

impl Platform {
    pub fn simple_platform(position: Vec2) -> Self {
        Self {
            position,
            size: vec2(100.0, 10.0),
            breakable: false,
            bounciness: 1.0,
        }
    }

    pub fn bouncy_platform(position: Vec2) -> Self {
        Self {
            bounciness: 2.0,
            ..Self::simple_platform(position)
        }
    }

    pub fn breakable_platform(position: Vec2) -> Self {
        Self {
            breakable: true,
            ..Self::simple_platform(position)
        }
    }
}