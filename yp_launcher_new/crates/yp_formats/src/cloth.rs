#![forbid(unsafe_code)]

mod bxm;

use bxm::Node;
use glam::Vec3;

#[derive(Debug, Clone, PartialEq)]
pub struct ClothGroup {
    pub(crate) bones: Vec<ClothBone>,
    pub(crate) colliders: Vec<Collider>,
    pub(crate) winds: Vec<Wind>,
    pub(crate) gravity: Vec3,
    pub(crate) gravity_bone: Option<i16>,
    pub(crate) world_gravity: bool,
    pub(crate) wind: Vec3,
    pub(crate) wind_bone: Option<i16>,
    pub(crate) damping: f32,
    pub(crate) stretch: f32,
    pub(crate) spring: f32,
    pub(crate) thickness: f32,
    pub(crate) original_rate: f32,
    pub(crate) first_bundle: f32,
    pub(crate) bundles: usize,
    pub(crate) lengths: usize,
    pub(crate) no_stretch: bool,
    pub(crate) expand: f32,
    pub(crate) hit_rate: f32,
    pub(crate) center_collision: bool,
    pub(crate) late_add: bool,
    pub(crate) fixed_axis: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ClothBone {
    pub bone: i16,
    pub up: Option<i16>,
    pub down: Option<i16>,
    pub side: Option<i16>,
    pub poly: Option<i16>,
    pub fixed: Option<i16>,
    pub side_anchor: bool,
    pub reverse_poly: bool,
    pub copy_fixed: bool,
    pub offset: Vec3,
    pub angle: f32,
    pub original_rate: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Collider {
    pub bones: [i16; 2],
    pub offsets: [Vec3; 2],
    pub weight: f32,
    pub radius: f32,
    pub capsule: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Wind {
    pub bone: Option<i16>,
    pub start: Vec3,
    pub end: Vec3,
    pub radius: f32,
    pub power: f32,
    pub kind: u8,
    pub swing: f32,
    pub speed: f32,
    pub timer: f32,
    pub delay: f32,
    pub rot_max: f32,
    pub rot_speed: f32,
    pub rot_type: u8,
}

fn entries<'a>(
    list: Node<'a>,
    count: usize,
    limit: usize,
    tag: &str,
) -> Option<impl Iterator<Item = Node<'a>>> {
    let children = list.children();
    if children.len() != count || count > limit || list.children().any(|n| n.name() != Some(tag)) {
        return None;
    }
    Some(children)
}

impl ClothGroup {
    pub fn parse(parameters: &[u8], collision: Option<&[u8]>, wind: Option<&[u8]>) -> Option<Self> {
        let root = Node::root(parameters, "CLOTH")?;
        let h = root.child("CLOTH_HEADER")?;
        let bones = entries(
            root.child("CLOTH_WK_LIST")?,
            h.count("m_Num")?,
            4096,
            "CLOTH_WK",
        )?
        .map(|n| {
            let (side, side_anchor) = n.reference("noSide", 0x8000)?;
            let (poly, reverse_poly) = n.reference("noPoly", 0x4000)?;
            let (fixed, copy_fixed) = n.reference("noFix", 0x2000)?;
            Some(ClothBone {
                bone: n.bone("no")??,
                up: n.bone("noUp")?,
                down: n.bone("noDown")?,
                side,
                poly,
                fixed,
                side_anchor,
                reverse_poly,
                copy_fixed,
                offset: n.vector("offset", Vec3::new(0., -0.1, 0.))?,
                angle: n
                    .scalar("rotLimit", std::f32::consts::PI)?
                    .clamp(0., std::f32::consts::PI),
                original_rate: n.scalar("m_OriginalRate", 0.)?.clamp(0., 1.),
            })
        })
        .collect::<Option<Vec<_>>>()?;
        if bones.is_empty() {
            return None;
        }
        let mut colliders = Vec::new();
        if let Some(bytes) = collision {
            let r = Node::root(bytes, "CLOTH_AT")?;
            for n in entries(
                r.child("CLOTH_AT_WK_LIST")?,
                r.count("CLOTH_AT_NUM")?,
                4096,
                "CLOTH_AT_WK",
            )? {
                let first = n.bone("p1")??;
                colliders.push(Collider {
                    bones: [first, n.bone("p2")?.unwrap_or(first)],
                    offsets: [
                        n.vector("offset1", Vec3::ZERO)?,
                        n.vector("offset2", Vec3::ZERO)?,
                    ],
                    weight: n.scalar("weight", 0.)?.clamp(0., 1.),
                    radius: n.scalar("radius", 0.)?.max(0.),
                    capsule: n.scalar("capsule", 0.)? != 0.,
                });
            }
        }
        let mut winds = Vec::new();
        if let Some(bytes) = wind {
            let r = Node::root(bytes, "CLOTH_WIND")?;
            for n in entries(
                r.child("CLOTH_WIND_WK_LIST")?,
                r.count("CLOTH_WIND_NUM")?,
                1024,
                "CLOTH_WIND_WK",
            )? {
                winds.push(Wind {
                    bone: n.bone("PartsNo")?,
                    start: n.vector("Offset", Vec3::ZERO)?,
                    end: n.vector("Offset2", Vec3::ZERO)?,
                    radius: n.scalar("Radius", 0.)?.max(0.),
                    power: n.scalar("Power", 0.)?,
                    kind: n.scalar("WindType", 0.)? as u8,
                    swing: n.scalar("SwingRate", 0.)?,
                    speed: n.scalar("SwingSpd", 0.)?,
                    timer: n.scalar("Timer", 0.)?.max(0.),
                    delay: n.scalar("DelayMax", 0.)?.max(0.),
                    rot_max: n.scalar("RotMax", 0.)?,
                    rot_speed: n.scalar("RotSpd", 0.)?,
                    rot_type: u8::try_from(n.integer("RotType", 0)?).ok()?,
                });
            }
        }
        Some(Self {
            bones,
            colliders,
            winds,
            gravity: h.vector("m_GravityVec", Vec3::ZERO)?,
            gravity_bone: h.bone("m_GravityPartsNo")?,
            world_gravity: h.integer("m_GravityPartsNo", -2)? <= -2,
            wind: h.vector("m_WindVec", Vec3::ZERO)?,
            wind_bone: h.bone("m_WindPartsNo")?,
            damping: h.scalar("m_SpdRate", 0.9)?.clamp(0., 1.),
            stretch: h.scalar("m_Stretchy", 0.3)?.clamp(0., 1.),
            spring: h.scalar("m_LimitSpringRate", 0.)?.clamp(0., 1.),
            thickness: h.scalar("m_Thick", 0.)?.max(0.),
            original_rate: h.scalar("m_OriginalRate", 0.)?.clamp(0., 1.),
            first_bundle: h.scalar("m_FirstBundleRate", 0.)?.clamp(0., 1.),
            bundles: u16::try_from(h.integer("m_BundleNum", 1)?).ok()? as usize,
            lengths: u16::try_from(h.integer("m_BundleNum2", 1)?).ok()? as usize,
            no_stretch: h.integer("m_bNoStretchy", 0)? != 0,
            expand: h.scalar("m_ExpandMax", 1.)?.max(0.),
            hit_rate: h.scalar("m_HitAdjustRate", 1.)?.clamp(0., 1.),
            center_collision: h.integer("m_bAtCenter", 0)? != 0,
            late_add: h.integer("m_bLateAddMode", 0)? != 0,
            fixed_axis: match h.integer("m_FixAxis", 0)? {
                0 => None,
                axis @ 1..=3 => Some((axis - 1) as usize),
                _ => return None,
            },
        })
    }
}
