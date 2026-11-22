use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Motion {
    pub fast: Duration,
    pub hover: Duration,
    pub base: Duration,
    pub expand: Duration,
    pub cross: Duration,
    pub toast: Duration,
    pub sweep: Duration,
    pub page: Duration,
    pub tooltip_delay: Duration,
    pub pulse: Duration,
    pub spin: Duration,
    pub ring_inner: Duration,
    pub ring_outer: Duration,
    pub cursor_blink: Duration,
    pub copy_feedback: Duration,
    pub toast_min: Duration,
    pub toast_max: Duration,
    pub toast_per_char: Duration,
    pub toast_hover_grace: Duration,
}

impl Default for Motion {
    fn default() -> Self {
        Self {
            fast: Duration::from_millis(100),
            hover: Duration::from_millis(120),
            base: Duration::from_millis(150),
            expand: Duration::from_millis(180),
            cross: Duration::from_millis(220),
            toast: Duration::from_millis(250),
            sweep: Duration::from_millis(260),
            page: Duration::from_millis(300),
            tooltip_delay: Duration::from_millis(350),
            pulse: Duration::from_millis(900),
            spin: Duration::from_millis(900),
            ring_inner: Duration::from_secs(3),
            ring_outer: Duration::from_secs(2),
            cursor_blink: Duration::from_millis(530),
            copy_feedback: Duration::from_secs(2),
            toast_min: Duration::from_secs(4),
            toast_max: Duration::from_secs(12),
            toast_per_char: Duration::from_millis(55),
            toast_hover_grace: Duration::from_secs(2),
        }
    }
}

impl Motion {
    pub fn toast_lifetime(&self, text: &str) -> Duration {
        let chars = u32::try_from(text.chars().count()).unwrap_or(u32::MAX);
        self.toast_per_char
            .saturating_mul(chars)
            .clamp(self.toast_min, self.toast_max)
    }
}

pub mod easing {
    const FILL: [f32; 27] = [
        0., 0.19589533, 0.3602655, 0.4962539, 0.60700387, 0.6956588, 0.7653773, 0.8192874,
        0.86056304, 0.8923476, 0.91776913, 0.9399863, 0.95011824, 0.9590601, 0.9668727, 0.9736324,
        0.97943085, 0.984329, 0.98840314, 0.99171436, 0.9943542, 0.9963836, 0.99787897, 0.9989166,
        0.9995575, 0.9998932, 1.,
    ];

    const SMOOTH: [f32; 11] = [
        0., 0.02798505, 0.10399023, 0.21599145, 0.35199508, 0.49999237, 0.64798963, 0.7839933,
        0.8959896, 0.9719997, 1.,
    ];

    const RISE: [f32; 11] = [
        0.,
        0.018997483,
        0.07199206,
        0.15298696,
        0.25598535,
        0.37499046,
        0.50399023,
        0.63698786,
        0.7679866,
        0.89098954,
        1.,
    ];

    const FADE: [f32; 31] = [
        0.,
        0.0032501717,
        0.012726024,
        0.027985046,
        0.048584726,
        0.074067295,
        0.103990234,
        0.13791104,
        0.17540246,
        0.21599145,
        0.2592508,
        0.30473793,
        0.3519951,
        0.40057984,
        0.45006484,
        0.49999237,
        0.5499199,
        0.5994049,
        0.64798963,
        0.6952468,
        0.740734,
        0.7839933,
        0.8245823,
        0.8620737,
        0.8959945,
        0.92591745,
        0.95140004,
        0.9719997,
        0.98725873,
        0.99673456,
        1.,
    ];

    const SLIDE: [f32; 31] = [
        0., 0.07335014, 0.1434196, 0.21025406, 0.27393, 0.334493, 0.39200428, 0.4465095, 0.498085,
        0.5467613, 0.5925994, 0.6356603, 0.6760052, 0.71367972, 0.74874494, 0.7812619, 0.8112612,
        0.8388342, 0.8640116, 0.88685435, 0.90740826, 0.92576486, 0.9419394, 0.955993, 0.96800183,
        0.9780117, 0.98608377, 0.9922637, 0.99659724, 0.99916075, 1.,
    ];

    const MASK_WIPE: [f32; 19] = [
        0.,
        0.120408945,
        0.23181506,
        0.33447775,
        0.42865643,
        0.5146105,
        0.59258413,
        0.6628367,
        0.7256428,
        0.78124666,
        0.8298924,
        0.87187,
        0.907393,
        0.9367666,
        0.9602045,
        0.97799647,
        0.99038684,
        0.9976348,
        1.,
    ];

    fn sample(samples: &[f32], t: f32) -> f32 {
        if samples.len() < 2 {
            return t;
        }
        let t = t.clamp(0., 1.);
        let last = samples.len() - 1;
        let position = t * last as f32;
        let index = (position.floor() as usize).min(last - 1);
        let fraction = position - index as f32;
        samples[index] + (samples[index + 1] - samples[index]) * fraction
    }

    pub fn nier_fill(t: f32) -> f32 {
        sample(&FILL, t)
    }

    pub fn nier_smooth(t: f32) -> f32 {
        sample(&SMOOTH, t)
    }

    pub fn nier_rise(t: f32) -> f32 {
        sample(&RISE, t)
    }

    pub fn nier_fade(t: f32) -> f32 {
        sample(&FADE, t)
    }

    pub fn nier_slide(t: f32) -> f32 {
        sample(&SLIDE, t)
    }

    pub fn nier_mask_wipe(t: f32) -> f32 {
        sample(&MASK_WIPE, t)
    }

    pub fn ease_out_cubic(t: f32) -> f32 {
        let inverse = 1. - t.clamp(0., 1.);
        1. - inverse * inverse * inverse
    }

    pub fn ping_pong(t: f32) -> f32 {
        let t = t.clamp(0., 1.);
        let folded = if t < 0.5 { t * 2. } else { (1. - t) * 2. };
        gpui::ease_in_out(folded)
    }
}
