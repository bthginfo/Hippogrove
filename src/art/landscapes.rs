use std::collections::BTreeSet;

use bevy::{pbr::NotShadowCaster, prelude::*};

use super::{Art, Geometry, broadleaf, mound, part, place_scene, rgb};
use crate::{
    game::Settlement,
    region::{Biome, Region, Tile},
    terrain::{self, WATER, hash, noise, smooth},
};

const CHUNK: i16 = 16;
const DETAIL: i16 = 2;

#[derive(Component)]
pub struct LandscapeProp(pub Tile);

struct Palette {
    grass: [u32; 2],
    shore: u32,
    cliff: u32,
    path: u32,
    water: [u32; 3],
    flower: u32,
}

impl Palette {
    fn for_biome(biome: Biome) -> Self {
        match biome {
            Biome::Valley => Self {
                grass: [0x6b914a, 0xb7bd68],
                shore: 0xc0ab7d,
                cliff: 0x9d9481,
                path: 0xd9bd92,
                water: [0x236972, 0x389591, 0x8bc6a5],
                flower: 0xffe49e,
            },
            Biome::Fjord => Self {
                grass: [0x728d79, 0xb0b899],
                shore: 0xabaaa2,
                cliff: 0x8b99ad,
                path: 0xcdc1ac,
                water: [0x275470, 0x428091, 0x87b8b2],
                flower: 0xd1cbea,
            },
            Biome::Terraces => Self {
                grass: [0x699345, 0xbaaf59],
                shore: 0xb7a47b,
                cliff: 0x7e8459,
                path: 0xd3b78d,
                water: [0x295f69, 0x408d83, 0x98c2a3],
                flower: 0xffb19b,
            },
            Biome::Lake => Self {
                grass: [0xa59553, 0xdac890],
                shore: 0xe2c894,
                cliff: 0xc29064,
                path: 0xe9cf9f,
                water: [0x256876, 0x38988f, 0x91c6a0],
                flower: 0xf5b073,
            },
            Biome::Aegean => Self {
                grass: [0x809747, 0xc0c174],
                shore: 0xe4d1a4,
                cliff: 0xc4b496,
                path: 0xebd5ad,
                water: [0x15568f, 0x169eb4, 0x6fd0ae],
                flower: 0xe68eb0,
            },
        }
    }
}

#[derive(Clone, Copy)]
struct Sample {
    local: Vec2,
    position: Vec3,
    coast: f32,
}

pub fn landscape(
    parent: &mut ChildSpawnerCommands,
    meshes: &mut Assets<Mesh>,
    art: &Art,
    region: &Region,
    city: &Settlement,
) {
    let palette = Palette::for_biome(region.kind);
    let bounds = region.bounds.padded(8);
    let Tile(left, north) = bounds.0;
    let Tile(right, south) = bounds.1.offset(1, 1);
    for z in (north..south).step_by(CHUNK as usize) {
        for x in (left..right).step_by(CHUNK as usize) {
            let end = Tile((x + CHUNK).min(right), (z + CHUNK).min(south));
            let (ground, water, foam) = chunk(region, &palette, Tile(x, z), end);
            for (geometry, material) in [
                (ground, &art.ground),
                (water, &art.water),
                (foam, &art.foam),
            ] {
                if geometry.positions.is_empty() {
                    continue;
                }
                parent.spawn((
                    Mesh3d(meshes.add(geometry.mesh())),
                    MeshMaterial3d(material.clone()),
                    Transform::default(),
                    NotShadowCaster,
                ));
            }
        }
    }
    let occupied: BTreeSet<_> = city
        .buildings
        .iter()
        .flat_map(|site| site.kind.tiles(site.tile))
        .collect();
    landmarks(parent, art, region, &occupied);
}

fn chunk(
    region: &Region,
    palette: &Palette,
    start: Tile,
    end: Tile,
) -> (Geometry, Geometry, Geometry) {
    let mut ground = Geometry::default();
    let mut water = Geometry::default();
    let mut foam = Geometry::default();
    let width = (end.0 - start.0) * DETAIL;
    let length = (end.1 - start.1) * DETAIL;
    let stride = (width + 1) as usize;
    let mut samples = Vec::with_capacity(stride * (length + 1) as usize);
    for z in 0..=length {
        for x in 0..=width {
            let gx = start.0 * DETAIL + x;
            let gz = start.1 * DETAIL + z;
            let jitter = |salt| ((hash(gx, gz, salt) & 31) as f32 / 31. - 0.5) * 0.15;
            let local = Vec2::new(f32::from(gx), f32::from(gz)) / f32::from(DETAIL)
                - Vec2::splat(0.5)
                + Vec2::new(jitter(91), jitter(92));
            let coast = terrain::shore(region, local);
            let position =
                Tile::world_point(local).with_y(terrain::surface_height(region, local, coast));
            samples.push(Sample {
                local,
                position,
                coast,
            });
        }
    }
    for z in 0..length as usize {
        for x in 0..width as usize {
            let i = z * stride + x;
            let [a, b, c, d] = [
                samples[i],
                samples[i + stride],
                samples[i + stride + 1],
                samples[i + 1],
            ];
            let triangles = if (x + z) % 2 == 0 {
                [[a, b, c], [a, c, d]]
            } else {
                [[b, c, d], [b, d, a]]
            };
            for corners in triangles {
                let land = corners.iter().any(|s| s.coast > 0.);
                let sea = corners.iter().any(|s| s.coast <= 0.);
                if land {
                    ground.tri_colors(
                        corners.map(|s| s.position),
                        corners.map(|s| land_color(region, palette, s)),
                    );
                }
                if sea {
                    water.tri_colors(
                        corners.map(|s| s.position.with_y(WATER)),
                        corners.map(|s| sea_color(palette, s)),
                    );
                    if land {
                        shoreline(&mut foam, corners);
                    }
                }
            }
        }
    }
    for z in start.1..end.1 {
        for x in start.0..end.0 {
            let tile = Tile(x, z);
            let key = hash(x, z, 223);
            if region.water(tile) {
                if key % 4 == 0 {
                    ripple(&mut foam, region, tile, key);
                }
            } else if region.road(tile) {
                stones(&mut ground, region, palette, tile, key);
            } else if key % 3 == 0 && !region.rocky(tile) && !region.fixed_structure(tile) {
                meadow(&mut ground, region, palette, tile, key);
            }
        }
    }
    (ground, water, foam)
}

fn land_color(region: &Region, palette: &Palette, s: Sample) -> Color {
    let broad = noise(s.local, 8.4, 43);
    let patches = noise(s.local, 2.55, 49);
    let grain = noise(s.local, 0.71, 57);
    let meadow = rgb(palette.grass[0]).mix(
        &rgb(palette.grass[1]),
        broad * 0.55 + patches * 0.32 + grain * 0.13,
    );
    let shore_width = if region.kind == Biome::Aegean {
        1.27
    } else {
        0.63
    };
    let bank = (1. - smooth(s.coast / shore_width)) * (0.72 + grain * 0.28);
    let rock = smooth((s.position.y - 0.65) / 2.4);
    let mut color = meadow
        .mix(&rgb(palette.cliff), rock * 0.93)
        .mix(&rgb(palette.shore), bank);
    let snow = if region.kind == Biome::Fjord {
        smooth((s.position.y - 3.3) / 4.5)
            .max(smooth((broad * 0.7 + patches * 0.3 - 0.53) / 0.31) * 0.76)
    } else {
        0.
    };
    color = color.mix(&rgb(0xe5ecee), snow);
    let tile = Tile(s.local.x.round() as i16, s.local.y.round() as i16);
    let center = Vec2::new(f32::from(tile.0), f32::from(tile.1));
    let mut road_distance = f32::INFINITY;
    if region.road(tile) {
        for next in tile
            .neighbors()
            .into_iter()
            .filter(|&tile| region.road(tile))
        {
            let vector = Vec2::new(f32::from(next.0), f32::from(next.1)) - center;
            let from = s.local - center;
            road_distance =
                road_distance.min((from - vector * from.dot(vector).clamp(0., 1.)).length());
        }
        road_distance = road_distance.min(s.local.distance(center));
    }
    color.mix(
        &rgb(palette.path),
        (1. - smooth((road_distance - 0.14) / 0.43)) * (0.63 + grain * 0.19),
    )
}

fn sea_color(palette: &Palette, s: Sample) -> Color {
    let depth = -s.coast;
    let run = noise(Vec2::new(s.local.x * 0.67, s.local.y * 1.61), 0.98, 80);
    let broad = noise(s.local, 9.5, 86);
    let deep = rgb(palette.water[0]).mix(&rgb(palette.water[1]), 0.11 + broad * 0.37 + run * 0.14);
    let shallows = rgb(palette.water[1]).mix(&rgb(palette.water[2]), 0.28 + run * 0.51);
    let depth_fade = smooth((depth + broad * 0.88 - 0.45) / 3.7);
    shallows.mix(&deep, depth_fade).mix(
        &rgb(0xc4e8ca),
        smooth((run - 0.54) / 0.46) * (1. - depth_fade) * 0.38,
    )
}

fn shoreline(foam: &mut Geometry, corners: [Sample; 3]) {
    let mut tips = [Vec3::ZERO; 2];
    let mut count = 0;
    let mut deep = corners[0];
    for (a, b) in corners
        .into_iter()
        .zip([corners[1], corners[2], corners[0]])
    {
        if a.coast < deep.coast {
            deep = a;
        }
        if (a.coast <= 0.) != (b.coast <= 0.) {
            let factor = (WATER - a.position.y) / (b.position.y - a.position.y);
            tips[count] = a.position.lerp(b.position, factor).with_y(WATER + 0.011);
            count += 1;
        }
    }
    if count != 2 || tips[0].distance_squared(tips[1]) < 0.0004 {
        return;
    }
    let [a, b] = tips;
    let midpoint = (a + b) * 0.5;
    let tangent = (b - a).normalize();
    let axis = Vec3::new(tangent.z, 0., -tangent.x);
    let outward = axis * (deep.position - midpoint).dot(axis).signum();
    let broken = noise(Tile::coordinates(midpoint), 0.74, 55);
    let color = rgb(0xdcf4df).with_alpha(0.45 + broken * 0.35);
    let x = a.lerp(b, broken * 0.21);
    let y = b.lerp(a, broken * 0.15);
    let width = 0.038 + broken * 0.06;
    foam.quad(
        x + outward * width,
        y + outward * width,
        y + outward * 0.022,
        x + outward * 0.022,
        color,
    );
    if broken > 0.53 {
        let x = a.lerp(b, 0.24) + outward * 0.26;
        let y = a.lerp(b, 0.74) + outward * 0.23;
        foam.quad(
            x + outward * 0.05,
            y + outward * 0.05,
            y,
            x,
            color.with_alpha(0.24),
        );
    }
}

fn ripple(foam: &mut Geometry, region: &Region, tile: Tile, key: u32) {
    let p = tile.world().with_y(WATER + 0.013);
    let angle = (key % 31) as f32 * 0.033 - 0.53;
    let length = 0.43 + (key % 19) as f32 * 0.054;
    let along = Vec3::new(angle.cos(), 0., angle.sin()) * length * 0.5;
    let cross = along.cross(Vec3::Y) * 0.13;
    let depth = -terrain::shore(region, Tile::coordinates(p));
    if depth > 0.29 {
        foam.quad(
            p - along,
            p + cross,
            p + along,
            p - cross,
            rgb(0xc7f4e9).with_alpha(0.18 + 0.12 * (key % 3) as f32),
        );
    }
}

fn stones(ground: &mut Geometry, region: &Region, palette: &Palette, tile: Tile, key: u32) {
    let base = rgb(palette.path);
    let rotation = Quat::from_rotation_y((key % 37) as f32 * 0.017 - 0.31);
    let vertical = region.road(tile.offset(0, 1)) || region.road(tile.offset(0, -1));
    for i in 0..3 {
        let offset = (i as f32 - 1.) * 0.48;
        let shift = (hash(tile.0, tile.1, i + 31) % 21) as f32 * 0.019 - 0.19;
        let center = tile.world()
            + if vertical {
                Vec3::new(shift, 0., offset)
            } else {
                Vec3::new(offset, 0., shift)
            };
        let color = base.mix(
            &rgb(palette.cliff),
            0.04 + ((key >> (i * 3)) & 7) as f32 * 0.075,
        );
        let around = [
            (-0.22, -0.13),
            (-0.28, 0.05),
            (-0.16, 0.21),
            (0.18, 0.18),
            (0.27, 0.01),
            (0.17, -0.15),
        ];
        let center = terrain::on_ground(region, center) + Vec3::Y * 0.045;
        let border = around.map(|(x, z)| {
            terrain::on_ground(region, center + rotation * Vec3::new(x, 0., z)) + Vec3::Y * 0.028
        });
        for (a, b) in border.into_iter().zip(border.into_iter().cycle().skip(1)) {
            ground.tri(center, a, b, color);
        }
    }
}

fn meadow(ground: &mut Geometry, region: &Region, palette: &Palette, tile: Tile, key: u32) {
    let local = Tile::coordinates(tile.world());
    let near = terrain::shore(region, local);
    if near < 0.55 {
        return;
    }
    let color = rgb(palette.grass[0]).mix(&rgb(0xe0cd8b), 0.37 + noise(local, 4.3, 62) * 0.33);
    for i in 0..2 {
        let h = hash(tile.0, tile.1, i + 52);
        let point = terrain::on_ground(
            region,
            tile.world()
                + Vec3::new(
                    ((h & 31) as f32 - 15.) * 0.042,
                    0.,
                    (((h >> 7) & 31) as f32 - 15.) * 0.042,
                ),
        );
        for side in [-1., 0., 1.] {
            let tip = point + Vec3::new(side * 0.119, 0.10 + (h % 13) as f32 * 0.009, 0.025);
            let base = point + Vec3::X * side * 0.042;
            ground.tri(base - Vec3::X * 0.035, base + Vec3::X * 0.035, tip, color);
            ground.tri(base + Vec3::X * 0.035, base - Vec3::X * 0.035, tip, color);
        }
        if key % 12 == 0 {
            let bloom = point + Vec3::new(0.04, 0.17, 0.019);
            let color = rgb(palette.flower);
            ground.tri(
                bloom + Vec3::new(-0.07, 0., 0.),
                bloom + Vec3::new(0.05, 0., 0.05),
                bloom + Vec3::new(0.04, 0.03, -0.06),
                color,
            );
        }
    }
}

fn landmarks(
    parent: &mut ChildSpawnerCommands,
    art: &Art,
    region: &Region,
    occupied: &BTreeSet<Tile>,
) {
    for tile in region.bounds.padded(5).tiles() {
        let Tile(x, z) = tile;
        if region.water(tile)
            || region.rocky(tile)
            || region.road(tile)
            || region.fixed_structure(tile)
            || region.source_at(tile).is_some()
            || occupied.contains(&tile)
        {
            continue;
        }
        let key = hash(x, z, 93);
        let p = terrain::on_ground(region, tile.world());
        let near_water = tile.neighbors().into_iter().any(|t| region.water(t));
        let village = tile.within(region.start, 8)
            || tile.within(region.frontier, 9)
            || tile.within(region.rival_shrine, 7)
            || region
                .districts
                .iter()
                .any(|district| tile.within(district.center, 9));
        let close = tile.within(region.start, 23) || tile.within(region.rival_shrine, 20);
        let yaw = (key % 127) as f32 * 0.049;
        let bank = near_water && key % if close { 13 } else { 26 } == 0;
        let orchard = region.kind == Biome::Aegean
            && tile.within(region.start, 8)
            && !tile.within(region.start, 4)
            && key % 13 == 0;
        let tree = !near_water && ((!village && key % if close { 11 } else { 23 } == 0) || orchard);
        let detail = key % if close { 19 } else { 83 } == 0;
        if !(bank || tree || detail) {
            continue;
        }
        parent
            .spawn((
                LandscapeProp(tile),
                Transform::from_translation(p).with_rotation(Quat::from_rotation_y(yaw)),
                Visibility::Inherited,
            ))
            .with_children(|parent| {
                let p = Vec3::ZERO;
                if bank {
                    place_scene(
                        parent,
                        &art.waldwinkel.rock,
                        p - Vec3::Y * 0.12,
                        0.55 + (key % 19) as f32 * 0.026,
                        0.,
                    );
                } else if tree {
                    let scale = if orchard {
                        0.71
                    } else {
                        0.65 + (key % 43) as f32 * 0.017
                    };
                    if region.kind == Biome::Fjord || (region.kind == Biome::Valley && key % 3 != 0)
                    {
                        place_scene(parent, &art.waldwinkel.conifer, p, scale * 0.80, 0.);
                        if region.kind == Biome::Fjord {
                            for (tier, size) in [(3.06, 0.81), (2.17, 0.92), (1.37, 1.04)] {
                                mound(
                                    parent,
                                    art,
                                    &art.white,
                                    p + Vec3::Y * tier * scale,
                                    Vec3::new(size, 0.16, size) * scale,
                                );
                            }
                        }
                    } else if region.kind == Biome::Aegean && key % 3 == 1 {
                        part(
                            parent,
                            &art.cone,
                            &art.evergreen[0],
                            p + Vec3::Y * 1.43 * scale,
                            Vec3::new(0.81, 2.87, 0.70) * scale,
                            Quat::from_rotation_y(yaw),
                        );
                        mound(
                            parent,
                            art,
                            &art.evergreen[1],
                            p + Vec3::Y * 1.32 * scale,
                            Vec3::new(0.79, 1.90, 0.66) * scale,
                        );
                    } else {
                        broadleaf(parent, art, p, scale, region.kind);
                    }
                } else {
                    let model = match region.kind {
                        Biome::Aegean | Biome::Lake if near_water => &art.waldwinkel.shell,
                        Biome::Terraces | Biome::Lake => &art.waldwinkel.reed,
                        _ => &art.waldwinkel.mushroom,
                    };
                    place_scene(parent, model, p, 0.51 + (key % 11) as f32 * 0.036, 0.);
                }
            });
    }
}
