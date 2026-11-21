use bevy::prelude::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ReadingSize {
    Compact,
    #[default]
    Auto,
    Large,
}

impl ReadingSize {
    pub const ALL: [Self; 3] = [Self::Compact, Self::Auto, Self::Large];

    pub fn name(self) -> &'static str {
        match self {
            Self::Compact => "Kompakt",
            Self::Auto => "Automatisch",
            Self::Large => "Groß",
        }
    }

    pub fn scale(self, safe: Vec2, gamepad: bool, modal: bool) -> f32 {
        if modal && safe.y < 352. {
            return (safe.y / 352.).clamp(0.64, 1.);
        }
        // Work in logical pixels. This keeps the same reading distance at 1× and 2× DPI.
        let canvas = match self {
            Self::Compact => Vec2::new(1400., 820.),
            Self::Auto if !gamepad => Vec2::new(1200., 720.),
            Self::Auto => Vec2::new(1030., 640.),
            Self::Large => Vec2::new(920., 590.),
        };
        (safe / canvas).min_element().clamp(1., 3.)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reading_distance_scales_tablets_without_clipping_phone_menus() {
        let tablet = Vec2::new(1280., 784.);
        let auto = ReadingSize::Auto;
        assert!(auto.scale(tablet, true, true) > auto.scale(tablet, false, true));
        assert!(ReadingSize::Large.scale(tablet, true, true) > auto.scale(tablet, true, true));
        for mode in ReadingSize::ALL {
            let double = mode.scale(2. * tablet, true, true);
            let single = mode.scale(tablet, true, true);
            assert!(tablet.y / single >= 590.);
            assert!((single * 2. - double).abs() < 0.001 || single == 1.);
            assert!((mode.scale(Vec2::new(512., 288.), true, true) - 288. / 352.).abs() < 0.001);
        }
    }
}
