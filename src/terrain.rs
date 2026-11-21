use bevy::prelude::*;

use crate::region::{Biome, Region, Tile};

pub const WATER: f32 = -0.32;
pub const DECK: f32 = 0.18;
const BRIDGE_RISE: f32 = 0.37;

pub fn smooth(t: f32) -> f32 {
    let t = t.clamp(0., 1.);
    t * t * (3. - 2. * t)
}

pub fn hash(x: i16, z: i16, salt: u32) -> u32 {
    let mut n = (i32::from(x) as u32).wrapping_mul(0x85ebca6b)
        ^ (i32::from(z) as u32).wrapping_mul(0xc2b2ae35)
        ^ salt.wrapping_mul(0x9e3779b9);
    n ^= n >> 16;
    n = n.wrapping_mul(0x7feb352d);
    n ^= n >> 15;
    n = n.wrapping_mul(0x846ca68b);
    n ^ (n >> 16)
}

pub fn noise(at: Vec2, size: f32, salt: u32) -> f32 {
    let at = at / size;
    let x = at.x.floor() as i16;
    let z = at.y.floor() as i16;
    let tx = smooth(at.x - x as f32);
    let tz = smooth(at.y - z as f32);
    let value = |x, z| (hash(x, z, salt) & 0xffff) as f32 / 65535.;
    value(x, z)
        .lerp(value(x + 1, z), tx)
        .lerp(value(x, z + 1).lerp(value(x + 1, z + 1), tx), tz)
}

/// Signed horizontal distance to the nearest shoreline in gameplay-grid units.
/// The rendered shore and the gameplay grid use the same water boundaries.
pub fn shore(region: &Region, point: Vec2) -> f32 {
    if region.tutorial {
        return region.island_shore(point);
    }
    let left = point.x.floor() as i16;
    let t = smooth(point.x - f32::from(left));
    let near = f32::from(region.river_at(left)).lerp(f32::from(region.river_at(left + 1)), t) - 0.5;
    let width = f32::from(region.river_span(left)).lerp(f32::from(region.river_span(left + 1)), t);
    let channel = (near - point.y)
        .max(point.y - near - width)
        .min(region.basin_shore(point));
    if region.island_world() {
        channel.min(region.island_shore(point))
    } else {
        channel
    }
}

fn relief(region: &Region, point: Vec2) -> f32 {
    let scale = match region.kind {
        Biome::Valley => 0.89,
        Biome::Fjord => 1.87,
        Biome::Terraces => 1.29,
        Biome::Lake => 1.22,
        Biome::Aegean => 1.16,
    };
    let mut height: f32 = 0.;
    for &(anchor, radius) in &region.highlands {
        let radius = f32::from(radius);
        let anchor = Vec2::new(f32::from(anchor.0), f32::from(anchor.1));
        let offset = point - anchor;
        let distance = offset.length_squared();
        if distance >= radius * radius {
            continue;
        }
        let progress = 1. - distance.sqrt() / radius;
        let crooked = 0.67 + 0.43 * noise(point, 2.05, 54) + offset.x * 0.043;
        let steps = if region.kind == Biome::Terraces {
            ((progress * 4.).floor() * 0.21 + progress * 0.16).max(0.)
        } else {
            progress.powf(0.69)
        };
        height = height.max(scale * radius * steps * crooked);
    }
    let original = region.original_bounds();
    let distance = (original.0.local() - point)
        .max(point - original.1.local())
        .max_element();
    let headroom = 0.94 + smooth(distance / 6.7) * 42.;
    for &ridge in &region.ridges {
        let (width, along) = ridge.section(point);
        if width <= 0. {
            continue;
        }
        let ledges = noise(point, 2.7, 59);
        let crests = 0.74 + (0.18 + along * 28.).sin().abs() * 0.18 + ledges * 0.15;
        let slope = width.powf(1.62);
        let ridge = if matches!(region.kind, Biome::Terraces | Biome::Lake) {
            (slope * 7.).floor() / 9. + slope * 0.22
        } else {
            slope
        } * ridge.3
            * crests;
        height = height.max(ridge.min(headroom));
    }
    if !region.island_world() {
        let outside = (f32::from(region.bounds.0.1) - point.y)
            .max(f32::from(region.bounds.0.0) - point.x)
            .max(point.x - f32::from(region.bounds.1.0))
            .max(0.);
        height = height.max(outside * scale * (0.49 + noise(point, 9., 65) * 0.54));
    }
    height
}

pub fn surface_height(region: &Region, local: Vec2, coast: f32) -> f32 {
    if coast <= 0. {
        return WATER - 0.85 * smooth(-coast / 1.95);
    }
    let roll = (noise(local, 6.3, 42) - 0.5) * 0.19 + noise(local, 15., 43) * 0.19;
    let bank = smooth(coast / 1.03);
    WATER + (-WATER + roll + relief(region, local)) * bank
}

pub fn height(region: &Region, world: Vec3) -> f32 {
    let local = Tile::coordinates(world);
    surface_height(region, local, shore(region, local)).max(WATER)
}

pub fn on_ground(region: &Region, world: Vec3) -> Vec3 {
    world.with_y(height(region, world))
}

pub fn bridge_height(region: &Region, column: i16, z: f32) -> f32 {
    let bank = f32::from(region.river_at(column)) - 0.5;
    let progress = ((z - bank) / f32::from(region.river_span(column))).clamp(0., 1.);
    DECK + BRIDGE_RISE * (progress * std::f32::consts::PI).sin()
}

pub fn actor_height(region: &Region, world: Vec3, bridge_fixed: bool) -> f32 {
    let ground = height(region, world);
    let tile = Tile::from_world(world);
    if region.ford(tile) && !bridge_fixed {
        return ground.max(0.08);
    }
    if bridge_fixed && region.crossing(tile) {
        let coordinates = Tile::coordinates(world);
        let bank = f32::from(region.river_at(tile.0)) - 0.5;
        let end = bank + f32::from(region.river_span(tile.0));
        let approach = (bank - coordinates.y).max(coordinates.y - end);
        let edge = (coordinates.x - f32::from(tile.0)).abs();
        let width = if region.kind == Biome::Aegean {
            0.49
        } else {
            0.48
        };
        if approach < 0.43 && edge <= width {
            return ground.max(ground.lerp(
                bridge_height(region, tile.0, coordinates.y),
                1. - smooth(approach / 0.43),
            ));
        }
    }
    ground
}

/// Sweep a downward camera ray to the rendered height field, then refine its first contact.
pub fn ray_contact(region: &Region, ray: Ray3d, bridge_fixed: bool) -> Option<Vec3> {
    if ray.direction.y >= -0.03 {
        return None;
    }
    let enter = ((49. - ray.origin.y) / ray.direction.y).max(0.);
    let exit = (WATER - 0.06 - ray.origin.y) / ray.direction.y;
    if !exit.is_finite() || exit <= enter {
        return None;
    }
    let above = |t| {
        let point = ray.get_point(t);
        point.y > actor_height(region, point, bridge_fixed)
    };
    let mut previous = enter;
    for index in 1..=36 {
        let t = enter.lerp(exit, index as f32 / 36.);
        if !above(t) {
            let mut a = previous;
            let mut b = t;
            for _ in 0..11 {
                let middle = (a + b) * 0.5;
                if above(middle) {
                    a = middle;
                } else {
                    b = middle;
                }
            }
            return Some(ray.get_point(b));
        }
        previous = t;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rendered_shore_and_walkable_tiles_match_and_picking_hits_the_surface() {
        for region in Region::campaign() {
            for tile in region.bounds.tiles() {
                let point = Tile::coordinates(tile.world());
                assert_eq!(
                    shore(&region, point) < 0.,
                    region.water(tile),
                    "{} {tile:?}",
                    region.name
                );
            }
            let p = region.start.world();
            let ray = Ray3d::new(p + Vec3::Y * 4., Dir3::NEG_Y);
            let hit = ray_contact(&region, ray, false).unwrap();
            assert!((hit.y - height(&region, p)).abs() < 0.0005);
            let bridge = region.bridge().world();
            let walking = actor_height(&region, bridge, true);
            assert!(walking > DECK);
            assert!(actor_height(&region, bridge, false) < 0.);
            let ray = Ray3d::new(bridge + Vec3::Y * 4., Dir3::NEG_Y);
            assert!((ray_contact(&region, ray, true).unwrap().y - walking).abs() < 0.0005);
        }
    }
}

/// One deterministic placement is shared by picking, saving and the batched landscape mesh.
pub struct WildLand {
    pub key: u32,
    pub position: Vec3,
    pub grove: f32,
    pub kind: Option<crate::region::Material>,
    pub flowers: bool,
}
pub fn wild_land(region: &Region, tile: Tile) -> Option<WildLand> {
    use crate::region::{Biome, Material};
    if region.water(tile)
        || region.road(tile)
        || region.fixed_structure(tile)
        || region.source_at(tile).is_some()
        || region
            .villages
            .iter()
            .any(|v| v.center == tile || v.houses.contains(&tile))
    {
        return None;
    }
    let key = hash(tile.0, tile.1, 171);
    let where_ = tile.local()
        + Vec2::new(
            ((key & 15) as f32 - 7.) * 0.051,
            (((key >> 4) & 15) as f32 - 7.) * 0.051,
        );
    let point = Tile::world_point(where_);
    let coast = shore(region, where_);
    if coast < 0.24 {
        return None;
    }
    let p = point.with_y(surface_height(region, where_, coast).max(WATER));
    let clearing = tile.within(region.start, 5)
        || tile.within(region.frontier, 4)
        || tile.within(region.rival_shrine, 4)
        || region
            .districts
            .iter()
            .any(|district| tile.within(district.center, 5));
    let stand = region
        .areas
        .iter()
        .any(|area| area.material == Material::Wood && tile.within(area.center, 12));
    let grove =
        smooth((noise(where_, 8.3, 154) * 0.79 + noise(where_, 3., 241) * 0.21 - 0.25) / 0.57);
    let high = match region.kind {
        Biome::Fjord => 13.,
        Biome::Terraces => 19.,
        _ => 18.,
    };
    let growth = (grove * 0.39 + if stand { 0.23 } else { 0.035 })
        * (1. - smooth((p.y - high * 0.58) / high));
    let kind = if !clearing && p.y <= high && coast > 0.81 && (key % 1000) as f32 / 1000. < growth {
        Some(Material::Wood)
    } else if !clearing
        && (region.rocky(tile) || coast < 0.9 || grove < 0.34)
        && key % 100 < if region.rocky(tile) { 35 } else { 22 }
    {
        Some(Material::Stone)
    } else {
        None
    };
    Some(WildLand {
        key,
        position: p,
        grove,
        kind,
        flowers: (clearing || coast < 1.6) && !region.rocky(tile) && key % 100 < 21,
    })
}
