use serde::{Deserialize, Serialize};

use crate::settings::SettingKey;

pub const DEFAULT_WIDTH: f64 = 1300.0;
pub const DEFAULT_HEIGHT: f64 = 750.0;
pub const MIN_WIDTH: f64 = 600.0;
pub const MIN_HEIGHT: f64 = 420.0;
pub const MAX_EXTENT: f64 = 10000.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub width: f64,
    pub height: f64,
    #[serde(default)]
    pub x: Option<f64>,
    #[serde(default)]
    pub y: Option<f64>,
    #[serde(default)]
    pub maximized: bool,
    #[serde(default)]
    pub fullscreen: bool,
}

impl Default for WindowGeometry {
    fn default() -> Self {
        Self {
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            x: None,
            y: None,
            maximized: false,
            fullscreen: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisplayArea {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub const WINDOW: SettingKey<Option<WindowGeometry>> = SettingKey::new("window", || None);

impl WindowGeometry {
    pub fn sanitize(self, displays: &[DisplayArea]) -> Self {
        let width = sane_extent(self.width, DEFAULT_WIDTH, MIN_WIDTH);
        let height = sane_extent(self.height, DEFAULT_HEIGHT, MIN_HEIGHT);
        let position = match (self.x, self.y) {
            (Some(x), Some(y)) if reachable(x, y, displays) => (Some(x), Some(y)),
            _ => (None, None),
        };
        Self {
            width,
            height,
            x: position.0,
            y: position.1,
            ..self
        }
    }
}

fn sane_extent(value: f64, fallback: f64, min: f64) -> f64 {
    if value.is_finite() {
        value.clamp(min, MAX_EXTENT)
    } else {
        fallback
    }
}

fn reachable(x: f64, y: f64, displays: &[DisplayArea]) -> bool {
    if !x.is_finite() || !y.is_finite() || y < -64.0 {
        return false;
    }
    if displays.is_empty() {
        return x > -20000.0 && x < 20000.0 && y < 20000.0;
    }
    displays.iter().any(|display| {
        x + 64.0 > display.x
            && x < display.x + display.width - 64.0
            && y + 16.0 > display.y
            && y < display.y + display.height - 32.0
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: DisplayArea = DisplayArea {
        x: 0.0,
        y: 0.0,
        width: 1920.0,
        height: 1080.0,
    };

    #[test]
    fn clamps_size_and_drops_offscreen_position() {
        let geometry = WindowGeometry {
            width: 100.0,
            height: f64::NAN,
            x: Some(5000.0),
            y: Some(10.0),
            ..WindowGeometry::default()
        }
        .sanitize(&[SCREEN]);
        assert_eq!(geometry.width, MIN_WIDTH);
        assert_eq!(geometry.height, DEFAULT_HEIGHT);
        assert_eq!(geometry.x, None);
    }

    #[test]
    fn keeps_visible_position() {
        let geometry = WindowGeometry {
            x: Some(100.0),
            y: Some(100.0),
            ..WindowGeometry::default()
        }
        .sanitize(&[SCREEN]);
        assert_eq!((geometry.x, geometry.y), (Some(100.0), Some(100.0)));
    }

    #[test]
    fn title_bar_above_screen_is_unreachable() {
        let geometry = WindowGeometry {
            x: Some(100.0),
            y: Some(-200.0),
            ..WindowGeometry::default()
        }
        .sanitize(&[]);
        assert_eq!(geometry.y, None);
    }
}
