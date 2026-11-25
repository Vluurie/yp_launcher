use crate::Rect;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Transform {
    pub axes: [[f32; 3]; 3],
    pub position: [f32; 3],
}
impl Transform {
    pub fn at(x: f32, y: f32) -> Self {
        Self {
            axes: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
            position: [x, y, 0.],
        }
    }
    fn vector(self, p: [f32; 3]) -> [f32; 3] {
        std::array::from_fn(|i| {
            self.axes[0][i] * p[0] + self.axes[1][i] * p[1] + self.axes[2][i] * p[2]
        })
    }
    pub fn point(self, x: f32, y: f32) -> [f32; 2] {
        [
            self.axes[0][0] * x + self.axes[1][0] * y + self.position[0],
            self.axes[0][1] * x + self.axes[1][1] * y + self.position[1],
        ]
    }
    pub fn scale(self) -> [f32; 3] {
        self.axes.map(|a| a[0].hypot(a[1]).hypot(a[2]))
    }
    pub fn normalized(mut self) -> Self {
        let scale = self.scale();
        for (axis, n) in self.axes.iter_mut().zip(scale) {
            if n > 0. {
                for v in axis {
                    *v /= n;
                }
            }
        }
        self
    }
    pub fn child(mut self, v: [f32; 16], ignore: [bool; 4]) -> Self {
        let scale = self.scale();
        if ignore[1] {
            self.axes = Self::at(0., 0.).axes;
            if !ignore[2] {
                for (i, n) in scale.into_iter().enumerate() {
                    self.axes[i][i] = n;
                }
            }
        } else if ignore[2] {
            self = self.normalized();
        }
        if ignore[0] {
            self.position = [0.; 3];
        }
        let (sx, cx) = v[3].sin_cos();
        let (sy, cy) = v[4].sin_cos();
        let (sz, cz) = v[5].sin_cos();
        let axes = [
            [cy * cz * v[6], cy * sz * v[6], -sy * v[6]],
            [
                (sx * sy * cz - cx * sz) * v[7],
                (sx * sy * sz + cx * cz) * v[7],
                sx * cy * v[7],
            ],
            [cx * sy * cz + sx * sz, cx * sy * sz - sx * cz, cx * cy],
        ]
        .map(|a| self.vector(a));
        let offset = self.vector([v[0], v[1], v[2]]);
        Self {
            axes,
            position: std::array::from_fn(|i| self.position[i] + offset[i]),
        }
    }
    pub fn bounds(self, r: Rect) -> Rect {
        let ps = [
            self.point(r.x, r.y),
            self.point(r.x + r.w, r.y),
            self.point(r.x, r.y + r.h),
            self.point(r.x + r.w, r.y + r.h),
        ];
        let (mut xmin, mut ymin) = (ps[0][0], ps[0][1]);
        let (mut xmax, mut ymax) = (xmin, ymin);
        for [x, y] in ps {
            xmin = xmin.min(x);
            xmax = xmax.max(x);
            ymin = ymin.min(y);
            ymax = ymax.max(y);
        }
        Rect::new(xmin, ymin, xmax - xmin, ymax - ymin)
    }
    pub fn invert(self, p: [f32; 2]) -> Option<[f32; 2]> {
        let [[a, b, _], [c, d, _], _] = self.axes;
        let det = a * d - b * c;
        if det.abs() < f32::EPSILON {
            return None;
        }
        let [x, y] = [p[0] - self.position[0], p[1] - self.position[1]];
        Some([(d * x - c * y) / det, (a * y - b * x) / det])
    }
    pub fn axis_aligned(self) -> bool {
        self.axes[0][1].abs() < 1e-5
            && self.axes[1][0].abs() < 1e-5
            && self.axes[0][0] > 0.
            && self.axes[1][1] > 0.
    }
}

pub(crate) fn anchored([w, h]: [f32; 2], a: [u8; 2]) -> Rect {
    Rect::new(
        -w * f32::from(a[0].min(2)) * 0.5,
        -h * f32::from(a[1].min(2)) * 0.5,
        w,
        h,
    )
}
