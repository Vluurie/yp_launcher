use gpui::{
    Animation, AnimationExt, AnyElement, App, Bounds, ElementId, IntoElement, ParentElement,
    Pixels, RenderOnce, SharedString, Size, Transformation, Window, canvas, div, fill,
    linear_color_stop, linear_gradient, point, prelude::*, px, size, svg,
};

use super::util::bounds_tracker;
use crate::styled_ext::h_flex;
use crate::theme::{ActiveTheme, ResponsiveExt, Sizes, easing};

const RINGS_PATH: &str = "icons/extra/nier_rings_outer.svg";
const BORDER_PATH: &str = "icons/extra/nier_border.svg";
const RING_OVERSCAN: Pixels = px(500.);
const RING_RASTER: Pixels = px(1536.);
const RING_REST_SCALE: f32 = 0.97;
const RING_PULSE: f32 = 1.03;

#[derive(Default)]
struct BackgroundState {
    size: Option<Size<Pixels>>,
}

pub fn grid_offsets(extent: Pixels, spacing: Pixels) -> Vec<Pixels> {
    if spacing <= px(0.) || extent <= px(0.) {
        return Vec::new();
    }
    let count = (extent / spacing).ceil() as usize;
    (0..count).map(|index| spacing * index as f32).collect()
}

fn ring_fit(width: Pixels) -> f32 {
    (width + RING_OVERSCAN) / RING_RASTER
}

pub fn border_tile_count(width: Pixels) -> usize {
    if width <= px(0.) {
        return 0;
    }
    (width / Sizes::NIER_BORDER_TILE_WIDTH).ceil() as usize
}

pub struct NierBackground {
    id: ElementId,
    animate: bool,
    grid: bool,
    rings: bool,
    borders: bool,
    children: Vec<AnyElement>,
}

crate::impl_into_element!(NierBackground);

impl Default for NierBackground {
    fn default() -> Self {
        Self::new()
    }
}

impl NierBackground {
    pub fn new() -> Self {
        Self {
            id: ElementId::Name("yp-nier-background".into()),
            animate: true,
            grid: true,
            rings: true,
            borders: true,
            children: Vec::new(),
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }

    pub fn grid(mut self, grid: bool) -> Self {
        self.grid = grid;
        self
    }

    pub fn rings(mut self, rings: bool) -> Self {
        self.rings = rings;
        self
    }

    pub fn borders(mut self, borders: bool) -> Self {
        self.borders = borders;
        self
    }
}

impl ParentElement for NierBackground {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for NierBackground {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let sizes = window.sizes(cx);
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| BackgroundState::default());
        let area = state
            .read(cx)
            .size
            .unwrap_or(size(sizes.window_width, sizes.window_height));
        let ornament = theme.colors.ornament;
        let ring_color = ornament.opacity(theme.alpha.subtle);
        let grid_color = theme.colors.border_light.opacity(theme.alpha.subtle);
        let gradient_top = theme
            .colors
            .background_secondary
            .opacity(theme.alpha.overlay);
        let gradient_bottom = theme.colors.transparent;

        let tracker = bounds_tracker(move |bounds, window, cx| {
            let changed = state.read(cx).size != Some(bounds.size);
            if changed {
                let state = state.clone();
                window.defer(cx, move |_, cx| {
                    state.update(cx, |state, cx| {
                        state.size = Some(bounds.size);
                        cx.notify();
                    });
                });
            }
        });

        let rings = self.rings.then(|| {
            let fit = ring_fit(area.width);
            let rings = svg()
                .path(SharedString::new_static(RINGS_PATH))
                .absolute()
                .left((area.width - RING_RASTER) / 2.)
                .top((area.height - RING_RASTER) / 2.)
                .size(RING_RASTER)
                .text_color(ring_color);
            if self.animate {
                rings
                    .with_animation(
                        "yp-nier-rings",
                        Animation::new(theme.motion.ring_outer * 2)
                            .repeat()
                            .with_easing(easing::ping_pong),
                        move |this, delta| {
                            let scale = fit * RING_REST_SCALE * (1. + (RING_PULSE - 1.) * delta);
                            this.with_transformation(Transformation::scale(size(scale, scale)))
                        },
                    )
                    .into_any_element()
            } else {
                let scale = fit * RING_REST_SCALE;
                rings
                    .with_transformation(Transformation::scale(size(scale, scale)))
                    .into_any_element()
            }
        });

        let grid = self.grid.then(|| {
            canvas(
                |_, _, _| (),
                move |bounds: Bounds<Pixels>, _, window, _| {
                    let line = Sizes::BORDER_WIDTH;
                    for offset in grid_offsets(bounds.size.width, Sizes::NIER_GRID_SPACING) {
                        window.paint_quad(fill(
                            Bounds::new(
                                point(bounds.left() + offset, bounds.top()),
                                size(line, bounds.size.height),
                            ),
                            grid_color,
                        ));
                    }
                    for offset in grid_offsets(bounds.size.height, Sizes::NIER_GRID_SPACING) {
                        window.paint_quad(fill(
                            Bounds::new(
                                point(bounds.left(), bounds.top() + offset),
                                size(bounds.size.width, line),
                            ),
                            grid_color,
                        ));
                    }
                },
            )
            .absolute()
            .size_full()
        });

        let border_strip = |top: bool| {
            let tiles = border_tile_count(area.width);
            h_flex()
                .absolute()
                .left_0()
                .right_0()
                .h(Sizes::NIER_BORDER_TILE_HEIGHT)
                .overflow_hidden()
                .map(|this| {
                    if top {
                        this.top(Sizes::NIER_BORDER_INSET)
                    } else {
                        this.bottom(Sizes::NIER_BORDER_INSET)
                    }
                })
                .children((0..tiles).map(|_| {
                    svg()
                        .path(SharedString::new_static(BORDER_PATH))
                        .flex_none()
                        .w(Sizes::NIER_BORDER_TILE_WIDTH)
                        .h(Sizes::NIER_BORDER_TILE_HEIGHT)
                        .text_color(ornament)
                }))
        };

        div()
            .id(self.id)
            .relative()
            .size_full()
            .overflow_hidden()
            .bg(theme.colors.background_primary)
            .child(tracker)
            .children(rings)
            .children(grid)
            .child(div().absolute().inset_0().bg(linear_gradient(
                180.,
                linear_color_stop(gradient_top, 0.),
                linear_color_stop(gradient_bottom, 1.),
            )))
            .when(self.borders, |this| {
                this.child(border_strip(true)).child(border_strip(false))
            })
            .children(self.children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_lines_cover_the_extent_without_overshooting() {
        let offsets = grid_offsets(px(35.), px(10.));
        assert_eq!(offsets, vec![px(0.), px(10.), px(20.), px(30.)]);
        assert!(grid_offsets(px(0.), px(10.)).is_empty());
        assert!(grid_offsets(px(100.), px(0.)).is_empty());
    }

    #[test]
    fn the_rings_scale_one_raster_to_the_window_width() {
        assert_eq!(ring_fit(RING_RASTER - RING_OVERSCAN), 1.);
        assert_eq!(ring_fit(px(2572.)), 2.);
        assert!(ring_fit(px(1000.)) < 1.);
    }

    #[test]
    fn border_tiles_fill_the_width() {
        assert_eq!(border_tile_count(px(0.)), 0);
        assert_eq!(border_tile_count(px(64.)), 1);
        assert_eq!(border_tile_count(px(65.)), 2);
        assert_eq!(border_tile_count(px(1284.)), 21);
    }
}
