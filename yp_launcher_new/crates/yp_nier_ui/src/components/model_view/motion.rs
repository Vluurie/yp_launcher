use crate::{Rect, motion::ease_out};
pub(super) struct Motion;
impl Motion {
    pub fn native() -> &'static Self {
        &Self
    }
    pub fn viewport(&self, open: bool, elapsed: f32) -> Rect {
        let t = (elapsed * 3.).clamp(0., 1.);
        let blend = if open {
            1. - (1. - t).powf(2.2)
        } else {
            (1. - t).powf(2.2)
        };
        Rect::new(
            615. * (1. - blend),
            286. * (1. - blend),
            408. + 1192. * blend,
            408. + 492. * blend,
        )
    }
    pub fn title(&self, open: bool, elapsed: f32) -> (f32, f32) {
        let t = if open {
            ease_out(elapsed * 2.)
        } else {
            1. - ease_out(elapsed * 6.)
        };
        (-210. + 260. * t, t)
    }
    pub fn footer(&self, open: bool, elapsed: f32) -> (f32, f32) {
        let t = if open {
            ease_out(elapsed * 6.)
        } else {
            1. - ease_out(elapsed * 6.)
        };
        (210. * (1. - t), t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn both_clips_connect_the_detail_mask_to_the_full_view() {
        let m = Motion::native();
        let near = |a: Rect, b: Rect| {
            [a.x - b.x, a.y - b.y, a.w - b.w, a.h - b.h]
                .into_iter()
                .all(|d| d.abs() < 0.06)
        };
        assert!(near(
            m.viewport(true, 0.),
            Rect::new(615., 286., 408., 408.)
        ));
        assert!(near(m.viewport(true, 1.), Rect::new(0., 0., 1600., 900.)));
        assert!(near(m.viewport(false, 0.), m.viewport(true, 1.)));
        assert!(near(m.viewport(false, 1.), m.viewport(true, 0.)));
        let middle = m.viewport(true, 10. / 60.);
        assert!(middle.w > 1250. && middle.w < 1400.);
        assert!(middle.h > 760. && middle.h < 820.);
        assert!(m.title(true, 0.).0 < 0. && m.title(true, 0.5).0 > 49.);
        assert_eq!(m.footer(false, 1.).1, 0.);
    }
}
