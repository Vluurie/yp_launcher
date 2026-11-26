#![forbid(unsafe_code)]

use super::{Pose, Skeleton};
mod contact;
mod wind;

use crate::cloth::ClothGroup;
use crate::mot::{FRAMES_PER_SECOND, Motion};
use glam::{Affine3A, Quat, Vec3};
use std::collections::HashMap;
use wind::Emitter;

const MAX_STEPS: u64 = 12;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClothPlayer {
    groups: Vec<Group>,
    tick: Option<u64>,
    rig_size: usize,
}

#[derive(Debug, Clone, PartialEq)]
struct Point {
    bone: usize,
    up: Option<usize>,
    local_tip: Vec3,
    side: Option<(usize, f32)>,
    poly: Option<usize>,
    side_anchor: bool,
    reverse_poly: bool,
    fixed: Option<(usize, bool)>,
    anchor: Vec3,
    rest_length: f32,
    velocity_scale: f32,
    correction: Vec3,
    angle: f32,
    restore: f32,
    position: Vec3,
    velocity: Vec3,
    rotation: Quat,
}

#[derive(Debug, Clone, PartialEq)]
struct Body {
    bones: [usize; 2],
    offsets: [Vec3; 2],
    weight: f32,
    radius: f32,
    capsule: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct Group {
    points: Vec<Point>,
    order: Vec<usize>,
    colliders: Vec<Body>,
    winds: Vec<Emitter>,
    gravity: (Option<usize>, Vec3),
    wind: (Option<usize>, Vec3),
    damping: f32,
    spring: f32,
    stretch: f32,
    world_gravity: bool,
    first_bundle: f32,
    bundles: usize,
    lengths: usize,
    no_stretch: bool,
    expand: f32,
    hit_rate: f32,
    center_collision: bool,
    late_add: bool,
    fixed_axis: Option<usize>,
}

fn origin(matrix: Affine3A) -> Vec3 {
    matrix.translation.into()
}

impl Group {
    fn bind(group: ClothGroup, skeleton: &Skeleton, bind: &[Affine3A]) -> Option<Self> {
        let bone = |id: i16| skeleton.indices.get(&id).copied();
        let source = &group.bones;
        if source.is_empty() {
            return None;
        }
        let mut indices: HashMap<i16, Vec<usize>> = HashMap::new();
        for (i, p) in source.iter().enumerate() {
            indices.entry(p.bone).or_default().push(i);
        }
        let mut points = Vec::with_capacity(source.len());
        let mut children = vec![Vec::new(); source.len()];
        let mut order = Vec::with_capacity(source.len());
        for (i, p) in source.iter().enumerate() {
            let index = bone(p.bone)?;
            if [p.up, p.down, p.side, p.poly, p.fixed]
                .into_iter()
                .flatten()
                .any(|id| bone(id).is_none())
            {
                return None;
            }
            let world = bind[index];
            let here = origin(world);
            let tip = if let Some(down) = p.down.and_then(bone) {
                skeleton.bones[index]
                    .inverse_bind
                    .transform_point3(origin(bind[down]))
            } else if let Some(up) = p.up.and_then(bone) {
                skeleton.bones[index]
                    .inverse_bind
                    .transform_vector3(here - origin(bind[up]))
            } else {
                p.offset
            };
            let neighbor = |id| {
                indices
                    .get(&id)
                    .and_then(|entries| entries.iter().copied().find(|&other| other != i))
            };
            let up = p.up.and_then(neighbor);
            if p.up == Some(p.bone) && up.is_none() {
                return None;
            }
            if let Some(parent) = up {
                children[parent].push(i);
            } else {
                order.push(i);
            }
            let side = p
                .side
                .and_then(|id| Some((neighbor(id)?, here.distance(origin(bind[bone(id)?])))));
            points.push(Point {
                bone: index,
                up,
                local_tip: tip,
                side,
                poly: p.poly.and_then(neighbor),
                anchor: here,
                rest_length: world.transform_vector3(tip).length(),
                velocity_scale: 1.,
                correction: Vec3::ZERO,
                side_anchor: p.side_anchor,
                reverse_poly: p.reverse_poly,
                fixed: p.fixed.and_then(bone).map(|id| (id, p.copy_fixed)),
                angle: p.angle,
                restore: if p.original_rate > 0. {
                    p.original_rate
                } else {
                    group.original_rate
                },
                position: world.transform_point3(tip),
                velocity: Vec3::ZERO,
                rotation: Quat::IDENTITY,
            });
        }
        let mut next = 0;
        while next < order.len() {
            order.extend(&children[order[next]]);
            next += 1;
        }
        if order.len() != points.len() {
            return None;
        }
        let resolve = |id: Option<i16>| match id {
            Some(id) => Some(Some(bone(id)?)),
            None => Some(None),
        };
        let colliders = group
            .colliders
            .into_iter()
            .map(|c| {
                Some(Body {
                    bones: [bone(c.bones[0])?, bone(c.bones[1])?],
                    offsets: c.offsets,
                    weight: c.weight,
                    radius: c.radius + group.thickness,
                    capsule: c.capsule,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(Self {
            points,
            order,
            colliders,
            winds: group
                .winds
                .into_iter()
                .map(|w| Some(Emitter::new(resolve(w.bone)?, w)))
                .collect::<Option<_>>()?,
            gravity: (resolve(group.gravity_bone)?, group.gravity),
            wind: (resolve(group.wind_bone)?, group.wind),
            damping: group.damping,
            spring: group.spring,
            stretch: group.stretch,
            world_gravity: group.world_gravity,
            first_bundle: group.first_bundle,
            bundles: group.bundles,
            lengths: group.lengths,
            no_stretch: group.no_stretch,
            expand: group.expand,
            hit_rate: group.hit_rate,
            center_collision: group.center_collision,
            late_add: group.late_add,
            fixed_axis: group.fixed_axis,
        })
    }

    fn reset(&mut self, world: &[Affine3A]) {
        for w in &mut self.winds {
            w.reset();
        }
        for p in &mut self.points {
            let pose = world[p.bone];
            p.anchor = origin(pose);
            p.position = pose.transform_point3(p.local_tip);
            p.velocity = Vec3::ZERO;
            p.rotation = Quat::IDENTITY;
        }
    }

    fn collisions(
        &self,
        world: &[Affine3A],
        scale: f32,
    ) -> (Vec<(Vec3, f32)>, Vec<contact::Shape>) {
        let spheres: Vec<_> = self
            .colliders
            .iter()
            .map(|c| {
                let [a, b] =
                    std::array::from_fn(|i| world[c.bones[i]].transform_point3(c.offsets[i]));
                (b.lerp(a, c.weight), c.radius * scale)
            })
            .collect();
        let mut shapes = Vec::with_capacity(spheres.len());
        let mut i = 0;
        while i < spheres.len() {
            let (start, radius) = spheres[i];
            let mut end = start;
            if self.colliders[i].capsule && i + 1 < spheres.len() {
                i += 1;
                end = spheres[i].0;
            }
            shapes.push((start, end, radius));
            i += 1;
        }
        (spheres, shapes)
    }

    fn anchor(&self, i: usize) -> Vec3 {
        let p = &self.points[i];
        p.up.map_or(p.anchor, |up| self.points[up].position)
    }

    fn gather_forces(&mut self, root: Affine3A, world: &[Affine3A], scale: f32) -> Option<()> {
        let space = |bone: Option<usize>| bone.map_or(root, |i| world[i]);
        let gravity = if self.world_gravity {
            self.gravity.1 * scale
        } else {
            space(self.gravity.0).transform_vector3(self.gravity.1)
        };
        let force = gravity + space(self.wind.0).transform_vector3(self.wind.1);
        let winds: Vec<_> = self
            .winds
            .iter_mut()
            .map(|w| w.advance(root, world))
            .collect::<Option<_>>()?;
        for p in &mut self.points {
            let pose = world[p.bone];
            p.anchor = origin(pose);
            p.velocity_scale = 1.;
            p.position += force + p.velocity;
            for wind in &winds {
                p.position += wind.at(p.position);
            }
            if let Some((fixed, false)) = p.fixed {
                p.position = origin(world[fixed]);
            }
            if let Some(axis) = self.fixed_axis {
                if pose.matrix3.determinant().abs() <= 1.0e-10 {
                    return None;
                }
                let mut local = pose.inverse().transform_point3(p.position);
                local[axis] = 0.;
                p.position = pose.transform_point3(local);
            }
            p.position = p
                .position
                .lerp(pose.transform_point3(p.local_tip), p.restore);
            if !p.position.length_squared().is_finite() {
                return None;
            }
        }
        Some(())
    }

    fn project_angles(&mut self, world: &[Affine3A], scale: f32) {
        if self.first_bundle > 0. {
            for i in 0..self.points.len() {
                let anchor = self.anchor(i);
                let p = &mut self.points[i];
                let excess = p.position.distance(anchor) - p.rest_length * scale;
                if excess > 0. {
                    p.position +=
                        (anchor - p.position).normalize_or_zero() * (excess * self.first_bundle);
                }
            }
        }
        for &i in &self.order {
            let anchor = self.anchor(i);
            let p = &self.points[i];
            let parent = p.up.map_or(Quat::IDENTITY, |up| self.points[up].rotation);
            let rest = parent * world[p.bone].transform_vector3(p.local_tip);
            let dir = rest.try_normalize().unwrap_or(Vec3::Y);
            let displacement = p.position - anchor;
            let desired = displacement.try_normalize().unwrap_or(dir);
            let angle = dir.dot(desired).clamp(-1., 1.).acos();
            let mut rotation = Quat::from_rotation_arc(dir, desired);
            let mut velocity_scale = 1.;
            if angle > p.angle && angle > f32::EPSILON {
                let allowed = p.angle + self.spring * (angle - p.angle);
                rotation = Quat::IDENTITY.slerp(rotation, allowed / angle);
                if angle > 1.25 * p.angle {
                    velocity_scale = 0.25;
                }
            }
            let length = p.rest_length * scale;
            let mut distance = displacement.length();
            if distance < length * 0.001 {
                distance = length * 0.1;
            }
            let p = &mut self.points[i];
            p.position = anchor + rotation * dir * distance.min(length * self.expand);
            p.rotation = rotation * parent;
            p.velocity_scale = velocity_scale;
        }
    }

    fn shift_anchor(&mut self, i: usize, delta: Vec3) {
        if let Some(up) = self.points[i].up {
            self.points[up].position += delta;
        } else {
            self.points[i].anchor += delta;
        }
    }

    fn collide(&mut self, spheres: &[(Vec3, f32)], bodies: &[contact::Shape]) {
        if !self.center_collision {
            for i in 0..self.points.len() {
                if let Some(side) = self.points[i].poly {
                    let mut vertices = [self.anchor(i), self.points[i].position, self.anchor(side)];
                    if self.points[i].reverse_poly {
                        vertices.swap(1, 2);
                    }
                    let d = contact::triangle(vertices, spheres, self.hit_rate);
                    self.shift_anchor(i, d);
                    self.points[i].position += d;
                    self.shift_anchor(side, d);
                    let mut vertices = [
                        self.anchor(side),
                        self.points[i].position,
                        self.points[side].position,
                    ];
                    if self.points[i].reverse_poly {
                        vertices.swap(1, 2);
                    }
                    let d = contact::triangle(vertices, spheres, self.hit_rate);
                    self.shift_anchor(side, d);
                    self.points[i].position += d;
                    self.points[side].position += d;
                }
            }
        }
        for i in 0..self.points.len() {
            let normal = (self.points[i].position - self.anchor(i))
                .try_normalize()
                .unwrap_or(Vec3::Y);
            let p = &mut self.points[i];
            p.position += contact::push(p.position, bodies, self.hit_rate, normal);
            if self.center_collision {
                let midpoint = (self.points[i].position + self.anchor(i)) * 0.5;
                let d = contact::push(midpoint, bodies, self.hit_rate, normal);
                self.points[i].position += d;
                if self.points[i].up.is_some() {
                    self.shift_anchor(i, d);
                }
                if let Some((side, _)) = self.points[i].side {
                    let at_anchor = self.points[i].side_anchor;
                    let target = if at_anchor {
                        self.anchor(side)
                    } else {
                        self.points[side].position
                    };
                    let midpoint = (self.points[i].position + target) * 0.5;
                    let d = contact::push(midpoint, bodies, self.hit_rate, normal);
                    self.points[i].position += d;
                    if at_anchor {
                        self.shift_anchor(side, d);
                    } else {
                        self.points[side].position += d;
                    }
                }
                if let Some(side) = self.points[i].poly {
                    let center =
                        (self.anchor(i) + self.anchor(side) + self.points[i].position) / 3.;
                    let d = contact::push(center, bodies, self.hit_rate, normal);
                    self.shift_anchor(i, d);
                    self.shift_anchor(side, d);
                    self.points[i].position += d;
                    let center =
                        (self.anchor(side) + self.points[side].position + self.points[i].position)
                            / 3.;
                    let d = contact::push(center, bodies, self.hit_rate, normal);
                    self.shift_anchor(side, d);
                    self.points[side].position += d;
                    self.points[i].position += d;
                }
            }
        }
    }

    fn constrain(
        &mut self,
        i: usize,
        other: Option<usize>,
        target: Vec3,
        distance: f32,
        stiffness: f32,
    ) {
        let offset = target - self.points[i].position;
        let len = offset.length();
        if len == 0. || (len <= distance && !self.no_stretch) {
            return;
        }
        let correction = offset * ((len - distance) / len * stiffness);
        let share = if self.points[i].up.is_some() { 0.5 } else { 1. };
        if self.late_add {
            self.points[i].correction += share * correction;
            if let Some(j) = other {
                self.points[j].correction -= (1. - share) * correction;
            }
        } else {
            self.points[i].position += share * correction;
            if let Some(j) = other {
                self.points[j].position -= (1. - share) * correction;
            }
        }
    }

    fn step(&mut self, root: Affine3A, world: &[Affine3A]) -> bool {
        let scale = root.matrix3.y_axis.length();
        let previous: Vec<_> = self.points.iter().map(|p| p.position).collect();
        if !scale.is_finite() || self.gather_forces(root, world, scale).is_none() {
            return false;
        }
        self.project_angles(world, scale);
        let (spheres, bodies) = self.collisions(world, scale);
        if spheres
            .iter()
            .any(|(p, r)| !p.is_finite() || !(r * r).is_finite())
        {
            return false;
        }
        for _ in 0..self.bundles {
            self.collide(&spheres, &bodies);
            for _ in 0..self.lengths {
                for i in 0..self.points.len() {
                    if let Some((side, length)) = self.points[i].side {
                        let at_anchor = self.points[i].side_anchor;
                        let other = if at_anchor {
                            self.points[side].up
                        } else {
                            Some(side)
                        };
                        let target = if at_anchor {
                            self.anchor(side)
                        } else {
                            self.points[side].position
                        };
                        self.constrain(i, other, target, length * scale, self.stretch);
                    }
                    self.constrain(
                        i,
                        self.points[i].up,
                        self.anchor(i),
                        self.points[i].rest_length * scale,
                        1.,
                    );
                }
                for p in &mut self.points {
                    if self.late_add {
                        p.position += p.correction;
                        p.correction = Vec3::ZERO;
                    }
                    if let Some((fixed, false)) = p.fixed {
                        p.position = origin(world[fixed]);
                    }
                }
            }
        }
        for &i in &self.order {
            let anchor = self.anchor(i);
            let p = &mut self.points[i];
            if let Some((fixed, copy)) = p.fixed {
                p.position = if copy {
                    world[fixed].transform_point3(p.local_tip)
                } else {
                    origin(world[fixed])
                };
            }
            let rest = p.rotation * world[p.bone].transform_vector3(p.local_tip);
            if let (Some(a), Some(b)) =
                (rest.try_normalize(), (p.position - anchor).try_normalize())
            {
                p.rotation = Quat::from_rotation_arc(a, b) * p.rotation;
            }
            p.velocity = (p.position - previous[i]) * self.damping * p.velocity_scale;
            if !p.position.length_squared().is_finite()
                || !p.velocity.length_squared().is_finite()
                || !p.rotation.is_finite()
            {
                return false;
            }
        }
        true
    }
}

impl ClothPlayer {
    pub fn bind(skeleton: &Skeleton, groups: impl IntoIterator<Item = ClothGroup>) -> Self {
        let mut groups = groups.into_iter().peekable();
        if groups.peek().is_none() {
            return Self::default();
        }
        let (_, bind) = skeleton.world(None, 0.);
        let mut bound = Vec::new();
        let mut work = 0usize;
        for group in groups {
            let per_point = group
                .bundles
                .saturating_mul(
                    group
                        .lengths
                        .saturating_add(group.colliders.len())
                        .saturating_add(1),
                )
                .saturating_add(group.winds.len())
                .saturating_add(1);
            work = work.saturating_add(group.bones.len().saturating_mul(per_point));
            if work > 262_144 {
                return Self::default();
            }
            let Some(group) = Group::bind(group, skeleton, &bind) else {
                return Self::default();
            };
            bound.push(group);
        }
        Self {
            groups: bound,
            tick: None,
            rig_size: skeleton.bones.len(),
        }
    }

    pub fn group_count(&self) -> usize {
        self.groups.len()
    }
    pub fn bone_count(&self) -> usize {
        self.groups.iter().map(|g| g.points.len()).sum()
    }
    pub fn reset(&mut self) {
        self.tick = None;
    }

    pub fn pose(
        &mut self,
        skeleton: &Skeleton,
        motion: Option<&Motion>,
        seconds: f32,
    ) -> Option<Pose> {
        let pose = self.try_pose(skeleton, motion, seconds);
        if pose.is_none() {
            *self = Self::default();
        }
        pose
    }

    fn try_pose(
        &mut self,
        skeleton: &Skeleton,
        motion: Option<&Motion>,
        seconds: f32,
    ) -> Option<Pose> {
        if self.groups.is_empty()
            || self.rig_size != skeleton.bones.len()
            || !(seconds * FRAMES_PER_SECOND).is_finite()
            || motion.is_some_and(|m| m.frames == 0)
        {
            return None;
        }
        let frame = (seconds.max(0.) * FRAMES_PER_SECOND) as u64;
        let continuous = self.tick.filter(|&last| last <= frame);
        let first = continuous.unwrap_or(0).max(frame.saturating_sub(MAX_STEPS));
        if continuous.is_none() {
            let (_, bind) = skeleton.world(motion, first as f32 / FRAMES_PER_SECOND);
            for group in &mut self.groups {
                group.reset(&bind);
            }
        }
        for i in first.saturating_add(1)..=frame {
            let (root, world) = skeleton.world(motion, i as f32 / FRAMES_PER_SECOND);
            if !root.is_finite() || world.iter().any(|m| !m.is_finite()) {
                return None;
            }
            for group in &mut self.groups {
                if !group.step(root, &world) {
                    return None;
                }
            }
        }
        self.tick = Some(frame);
        let (root, mut world) = skeleton.world(motion, seconds);
        let mut relative = vec![None; world.len()];
        for group in &self.groups {
            for p in &group.points {
                if let Some((fixed, true)) = p.fixed {
                    if world[p.bone].matrix3.determinant().abs() < 1.0e-10 {
                        return None;
                    }
                    relative[p.bone] = Some(world[fixed] * world[p.bone].inverse());
                    continue;
                }
                let base = origin(world[p.bone]);
                let anchor = p.up.map_or(base, |up| group.points[up].position);
                relative[p.bone] = Some(Affine3A::from_rotation_translation(
                    p.rotation,
                    anchor - p.rotation * base,
                ));
            }
        }
        for &i in &skeleton.order {
            let delta = relative[i]
                .or_else(|| skeleton.bones[i].parent.and_then(|parent| relative[parent]));
            if let Some(delta) = delta {
                relative[i] = Some(delta);
                world[i] = delta * world[i];
            }
        }
        if !root.is_finite() || world.iter().any(|m| !m.is_finite()) {
            return None;
        }
        skeleton.skin(root, world)
    }
}
