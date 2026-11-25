use crate::animation::RandomUnit;
use std::f32::consts::FRAC_1_SQRT_2;

struct Translation([RandomUnit; 2]);

impl Translation {
    fn new(period: f32, amplitude: f32, rng: &mut u32) -> Self {
        Self(std::array::from_fn(|_| {
            RandomUnit::new(period, amplitude, 1, rng)
        }))
    }
    fn sample(&mut self, seconds: f32, rng: &mut u32) -> [f32; 2] {
        std::array::from_fn(|i| self.0[i].sample(seconds, rng))
    }
}

pub(super) struct CornerDrift {
    groups: [[Translation; 6]; 2],
    rng: u32,
}

impl CornerDrift {
    pub(super) fn new(mut seed: u32) -> Self {
        let groups = std::array::from_fn(|_| {
            [(3., 10.), (2., 3.), (3., 3.), (3., 3.), (2., 3.), (3., 3.)]
                .map(|(period, amplitude)| Translation::new(period, amplitude, &mut seed))
        });
        Self { groups, rng: seed }
    }

    pub(super) fn origins(&mut self, seconds: f32, size: [f32; 2]) -> [[[f32; 2]; 5]; 2] {
        std::array::from_fn(|side| {
            let layers = &mut self.groups[side];
            let [mut root_x, mut root_y] = layers[0].sample(seconds, &mut self.rng);
            let opposite = side == 1;
            if opposite {
                root_x += size[0];
                root_y += size[1];
            }
            let k = if opposite {
                -FRAC_1_SQRT_2
            } else {
                FRAC_1_SQRT_2
            };
            let offset = if opposite { 50. } else { -50. };
            let positions = [[0., 0.], [0., 0.], [offset, 0.], [0., 10.], [-offset, 0.]];
            std::array::from_fn(|i| {
                let [dx, dy] = layers[i + 1].sample(seconds, &mut self.rng);
                let [x, y] = [positions[i][0] + dx, positions[i][1] + dy];
                [root_x - (x + y) * k, root_y + (x - y) * k]
            })
        })
    }
}
