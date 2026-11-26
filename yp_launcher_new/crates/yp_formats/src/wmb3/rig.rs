mod cloth_player;

pub use cloth_player::ClothPlayer;

use super::{Parsed, read_f32, read_u16, read_u32, validate_table};
use crate::mot::{FRAMES_PER_SECOND, Motion};
use glam::{Affine3A, EulerRot, Mat3A, Quat, Vec3};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Weights {
    pub bones: [u16; 4],
    pub weights: [u8; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Transform {
    position: Vec3,
    rotation: Vec3,
    scale: Vec3,
}

impl Transform {
    const IDENTITY: Self = Self {
        position: Vec3::ZERO,
        rotation: Vec3::ZERO,
        scale: Vec3::ONE,
    };

    fn set(&mut self, channel: u8, value: f32) {
        match channel {
            0..=2 => self.position[channel as usize] = value,
            3..=5 => self.rotation[usize::from(channel - 3)] = value,
            7..=9 => self.scale[usize::from(channel - 7)] = value,
            _ => {}
        }
    }

    fn orientation(&self) -> Quat {
        Quat::from_euler(
            EulerRot::ZYX,
            self.rotation.z,
            self.rotation.y,
            self.rotation.x,
        )
    }

    fn blend(self, other: Self, t: f32) -> Affine3A {
        Affine3A::from_scale_rotation_translation(
            self.scale.lerp(other.scale, t),
            self.orientation().slerp(other.orientation(), t),
            self.position.lerp(other.position, t),
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Skeleton {
    bones: Vec<Bone>,
    order: Vec<usize>,
    indices: HashMap<i16, usize>,
}

#[derive(Clone, Debug, PartialEq)]
struct Bone {
    parent: Option<usize>,
    rest: Transform,
    inverse_bind: Affine3A,
}

#[derive(Clone, Copy, Debug)]
struct SkinMatrix {
    positions: Affine3A,
    normals: Mat3A,
}

impl SkinMatrix {
    fn new(positions: Affine3A) -> Self {
        let basis = positions.matrix3;
        let normals = if basis.determinant().abs() > 1.0e-8 {
            basis.inverse().transpose()
        } else {
            basis
        };
        Self { positions, normals }
    }
}

pub struct Pose {
    root: SkinMatrix,
    bones: Vec<SkinMatrix>,
}

impl Pose {
    pub fn vertex(
        &self,
        weights: Option<&Weights>,
        position: [f32; 3],
        normal: [f32; 3],
    ) -> ([f32; 3], [f32; 3]) {
        let position = Vec3::from_array(position);
        let normal = Vec3::from_array(normal);
        let mut p = Vec3::ZERO;
        let mut n = Vec3::ZERO;
        let mut sum = 0u16;
        if let Some(weights) = weights {
            for (&id, &amount) in weights.bones.iter().zip(&weights.weights) {
                if amount == 0 {
                    continue;
                }
                let Some(matrix) = self.bones.get(usize::from(id)) else {
                    continue;
                };
                let weight = f32::from(amount);
                p += weight * matrix.positions.transform_point3(position);
                n += weight * (matrix.normals * normal);
                sum += u16::from(amount);
            }
        }
        if sum == 0 {
            p = self.root.positions.transform_point3(position);
            n = self.root.normals * normal;
        } else {
            p /= f32::from(sum);
        }
        (p.to_array(), n.normalize_or_zero().to_array())
    }
}

impl Skeleton {
    pub(super) fn read(bytes: &[u8]) -> Option<Self> {
        let offset = read_u32(bytes, 40).ok()? as usize;
        let count = read_u32(bytes, 44).ok()? as usize;
        validate_table(bytes, offset, count, 88, 4096).ok()?;
        let vec3 = |at| -> Option<Vec3> {
            let v = Vec3::new(
                read_f32(bytes, at).ok()?,
                read_f32(bytes, at + 4).ok()?,
                read_f32(bytes, at + 8).ok()?,
            );
            v.is_finite().then_some(v)
        };
        let mut bones = Vec::with_capacity(count);
        let mut indices = HashMap::with_capacity(count);
        let mut order = Vec::with_capacity(count);
        let mut children = vec![Vec::new(); count];
        for i in 0..count {
            let at = offset + i * 88;
            let id = read_u16(bytes, at).ok()? as i16;
            let parent = read_u16(bytes, at + 2).ok()?;
            let parent = (parent != u16::MAX).then_some(usize::from(parent));
            if let Some(p) = parent {
                children.get_mut(p)?.push(i);
            } else {
                order.push(i);
            }
            indices.insert(id, i);
            bones.push(Bone {
                parent,
                rest: Transform {
                    position: vec3(at + if parent.is_none() { 40 } else { 4 })?,
                    rotation: vec3(at + 16)?,
                    scale: vec3(at + 28)?,
                },
                inverse_bind: Affine3A::IDENTITY,
            });
        }
        let mut world = vec![Affine3A::IDENTITY; count];
        let mut i = 0;
        while i < order.len() {
            let index = order[i];
            let bone = &mut bones[index];
            let local = bone.rest.blend(bone.rest, 0.);
            let bind = bone.parent.map_or(local, |p| world[p] * local);
            if !bind.is_finite()
                || !bind.matrix3.determinant().is_finite()
                || bind.matrix3.determinant().abs() < 1.0e-10
            {
                return None;
            }
            world[index] = bind;
            bone.inverse_bind = bind.inverse();
            if !bone.inverse_bind.is_finite() {
                return None;
            }
            order.extend(&children[index]);
            i += 1;
        }
        (order.len() == count).then_some(Self {
            bones,
            order,
            indices,
        })
    }

    pub fn accepts(&self, motion: &Motion) -> bool {
        let mut has_root = false;
        let mut has_bones = false;
        for track in &motion.tracks {
            if self.indices.contains_key(&track.bone) {
                return true;
            }
            has_root |= track.bone == -1;
            has_bones |= track.bone >= 0;
        }
        has_root && !has_bones
    }

    pub fn pose(&self, motion: &Motion, seconds: f32) -> Pose {
        self.try_pose(motion, seconds).unwrap_or_else(|| Pose {
            root: SkinMatrix::new(Affine3A::IDENTITY),
            bones: Vec::new(),
        })
    }

    pub fn try_pose(&self, motion: &Motion, seconds: f32) -> Option<Pose> {
        if motion.frames == 0 || !(seconds * FRAMES_PER_SECOND).is_finite() {
            return None;
        }
        let (root, world) = self.world(Some(motion), seconds);
        self.skin(root, world)
    }

    fn world(&self, motion: Option<&Motion>, seconds: f32) -> (Affine3A, Vec<Affine3A>) {
        let frame = motion.map_or(0., |clip| {
            (seconds.max(0.) * FRAMES_PER_SECOND).rem_euclid(f32::from(clip.frames))
        });
        let first = frame.floor() as u16;
        let next = motion.map_or(0, |clip| (first + 1) % clip.frames);
        let mix = frame.fract();
        let mut transforms: Vec<_> = self.bones.iter().map(|b| [b.rest; 2]).collect();
        let mut root = [Transform::IDENTITY; 2];
        for track in motion.into_iter().flat_map(|clip| &clip.tracks) {
            let pair = if track.bone == -1 {
                &mut root
            } else if let Some(&index) = self.indices.get(&track.bone) {
                &mut transforms[index]
            } else {
                continue;
            };
            pair[0].set(track.channel, track.sample(first));
            pair[1].set(track.channel, track.sample(next));
        }
        let root = root[0].blend(root[1], mix);
        let mut world = vec![root; self.bones.len()];
        for &index in &self.order {
            let bone = &self.bones[index];
            let [a, b] = transforms[index];
            world[index] = bone.parent.map_or(root, |p| world[p]) * a.blend(b, mix);
        }
        (root, world)
    }

    fn skin(&self, root: Affine3A, world: Vec<Affine3A>) -> Option<Pose> {
        let checked = |transform: Affine3A| {
            let matrix = SkinMatrix::new(transform);
            (matrix.positions.is_finite() && matrix.normals.is_finite()).then_some(matrix)
        };
        let bones = world
            .into_iter()
            .zip(&self.bones)
            .map(|(w, bone)| checked(w * bone.inverse_bind))
            .collect::<Option<_>>()?;
        Some(Pose {
            root: checked(root)?,
            bones,
        })
    }
}

pub(super) fn palettes(bytes: &[u8]) -> Parsed<Vec<Vec<u16>>> {
    let map_offset = read_u32(bytes, 88)? as usize;
    let maps = read_u32(bytes, 92)? as usize;
    let sets_offset = read_u32(bytes, 96)? as usize;
    let sets = read_u32(bytes, 100)? as usize;
    validate_table(bytes, map_offset, maps, 4, 65536)?;
    validate_table(bytes, sets_offset, sets, 8, 65536)?;
    (0..sets)
        .map(|i| {
            let at = sets_offset + i * 8;
            let start = read_u32(bytes, at)? as usize;
            let count = read_u32(bytes, at + 4)? as usize;
            validate_table(bytes, start, count, 2, 256)?;
            (0..count)
                .map(|j| {
                    let map = usize::from(read_u16(bytes, start + j * 2)?);
                    if map >= maps {
                        return Ok(u16::MAX);
                    }
                    Ok(u16::try_from(read_u32(bytes, map_offset + map * 4)?).unwrap_or(u16::MAX))
                })
                .collect()
        })
        .collect()
}
