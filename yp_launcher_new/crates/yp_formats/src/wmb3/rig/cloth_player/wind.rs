use crate::cloth::Wind;
use glam::{Affine3A, Quat, Vec3};

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Emitter {
    bone: Option<usize>,
    wind: Wind,
    angle: f32,
    rotation: f32,
    time: f32,
    delay: f32,
    random: u32,
}

pub(super) enum Force {
    Uniform(Vec3),
    Radial {
        origin: Vec3,
        direction: Vec3,
        radius: f32,
        power: f32,
    },
}

impl Force {
    pub fn at(&self, position: Vec3) -> Vec3 {
        match *self {
            Self::Uniform(force) => force,
            Self::Radial {
                origin,
                direction,
                radius,
                power,
            } => {
                let offset = position - origin;
                let distance = offset.length();
                if distance > 0. && distance <= radius && offset.dot(direction) >= 0. {
                    offset * ((radius - distance) * power / distance)
                } else {
                    Vec3::ZERO
                }
            }
        }
    }
}

impl Emitter {
    pub fn new(bone: Option<usize>, wind: Wind) -> Self {
        Self {
            bone,
            wind,
            angle: 0.,
            rotation: 0.,
            time: 0.,
            delay: 0.,
            random: 0,
        }
    }

    pub fn reset(&mut self) {
        self.angle = 0.;
        self.rotation = 0.;
        self.time = 0.;
        self.delay = 0.;
        self.random = 0;
    }

    pub fn advance(&mut self, root: Affine3A, world: &[Affine3A]) -> Option<Force> {
        if self.delay > 0. {
            self.delay -= 1.;
            return Some(Force::Uniform(Vec3::ZERO));
        }
        let w = &self.wind;
        self.angle = (self.angle + w.speed) % std::f32::consts::TAU;
        let power = ((1. + self.angle.sin() * w.swing) * w.power).clamp(0., 1.);
        let rotation = Quat::from_rotation_y(self.rotation.sin() * w.rot_max);
        if w.rot_type == 0 {
            self.rotation = (self.rotation + w.rot_speed) % std::f32::consts::TAU;
        }
        let transform = self.bone.map_or(root, |i| world[i]);
        let start = transform.transform_point3(rotation * w.start);
        let travel = transform.transform_vector3(rotation * (w.end - w.start));
        self.time += 1.;
        if self.time > w.timer {
            self.time = 0.;
            self.delay = w.delay;
            match w.rot_type {
                1 => self.rotation = (self.rotation + w.rot_speed) % std::f32::consts::TAU,
                2 => {
                    self.random = self.random.wrapping_mul(1664525).wrapping_add(1013904223);
                    let sample = (self.random >> 8) as f32 / 16777216.;
                    self.rotation = (1. - 2. * sample) * std::f32::consts::PI;
                }
                _ => {}
            }
        }
        if !start.is_finite() || !travel.length_squared().is_finite() || !power.is_finite() {
            return None;
        }
        if w.kind == 1 {
            return Some(Force::Uniform(travel * power));
        }
        let offset = if w.timer > 0. {
            travel * (self.time / w.timer)
        } else {
            Vec3::ZERO
        };
        let direction = if w.timer > 0. {
            offset.try_normalize().unwrap_or(Vec3::Y)
        } else {
            Vec3::Z
        };
        Some(Force::Radial {
            origin: start + offset,
            direction,
            radius: w.radius,
            power,
        })
    }
}
