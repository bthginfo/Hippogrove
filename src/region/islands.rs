use std::sync::OnceLock;

use bevy::prelude::*;
use serde::Deserialize;

use super::{Bounds, Region, Tile};

#[derive(Deserialize)]
struct Map {
    land: Vec<[i16; 5]>,
    #[serde(default)]
    bays: Vec<[i16; 5]>,
}

#[derive(Clone, Debug)]
pub(super) struct Islands {
    land: Vec<Shape>,
    bays: Vec<Shape>,
}

#[derive(Clone, Copy, Debug)]
struct Shape {
    center: Vec2,
    direction: Vec2,
    inverse_radii: Vec2,
    width: f32,
}

impl Shape {
    fn from_data([x, z, width, depth, angle]: [i16; 5]) -> Self {
        let radii = Vec2::new(f32::from(width), f32::from(depth));
        let (sine, cosine) = f32::from(angle).to_radians().sin_cos();
        Self {
            center: Vec2::new(f32::from(x), f32::from(z)),
            direction: Vec2::new(cosine, -sine),
            inverse_radii: radii.recip(),
            width: radii.min_element() / 2.,
        }
    }

    fn shore(self, point: Vec2) -> f32 {
        let local = self.direction.rotate(point - self.center) * self.inverse_radii;
        (1. - local.length_squared()) * self.width
    }
}

impl Islands {
    pub fn for_region(region: &Region) -> Self {
        static MAPS: OnceLock<[[Map; 3]; 5]> = OnceLock::new();
        let maps = MAPS.get_or_init(|| {
            serde_json::from_str(include_str!("../../assets/world/islands.json"))
                .expect("Each chapter has an authored island map")
        });
        let map = &maps[region.kind as usize][region.chapter()];
        let shapes = |data: &[[i16; 5]]| data.iter().copied().map(Shape::from_data).collect();
        Self {
            land: shapes(&map.land),
            bays: shapes(&map.bays),
        }
    }

    pub fn shore(&self, point: Vec2) -> f32 {
        let land = self
            .land
            .iter()
            .map(|shape| shape.shore(point))
            .fold(f32::NEG_INFINITY, f32::max);
        let coast = self
            .bays
            .iter()
            .map(|shape| -shape.shore(point))
            .fold(land, f32::min);
        let variation = crate::terrain::noise(point, 5.5, 874) * 0.68
            + crate::terrain::noise(point, 2.1, 881) * 0.24
            - 0.44;
        coast + variation
    }

    pub fn bounds(&self) -> Bounds {
        let mut low = Vec2::splat(f32::INFINITY);
        let mut high = Vec2::splat(f32::NEG_INFINITY);
        for shape in &self.land {
            let radii = shape.inverse_radii.recip();
            let axis = shape.direction * radii;
            let across = shape.direction.yx() * radii;
            let extent = Vec2::new(axis.length(), across.length()) + Vec2::splat(1.);
            low = low.min(shape.center - extent);
            high = high.max(shape.center + extent);
        }
        Bounds(
            Tile(low.x.floor() as i16, low.y.floor() as i16),
            Tile(high.x.ceil() as i16, high.y.ceil() as i16),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::{
        region::{Biome, WorldLayout},
        terrain,
    };

    #[test]
    fn island_profiles_differ_and_fit_inside_the_rendered_ocean() {
        let mut profiles = HashSet::new();
        for biome in Biome::ALL {
            for chapter in 0..3 {
                let region = Region::themed(biome, chapter, WorldLayout::Islands);
                let bounds = region.render_bounds().padded(2);
                for Tile(x, z) in bounds.tiles().filter(|&Tile(x, z)| {
                    x == bounds.0.0 || x == bounds.1.0 || z == bounds.0.1 || z == bounds.1.1
                }) {
                    assert!(
                        terrain::shore(&region, Tile(x, z).local()) < 0.,
                        "{biome:?} chapter {chapter}"
                    );
                }
                let mask: Vec<_> = (-24..=24)
                    .flat_map(|z| (-31..=31).map(move |x| Tile(x * 2, z * 2)))
                    .map(|tile| region.water(region.layout.tile(tile)))
                    .collect();
                assert!(profiles.insert(mask));
            }
        }
    }
}
