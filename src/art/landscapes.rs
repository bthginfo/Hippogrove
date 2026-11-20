use std::collections::BTreeSet;

use bevy::{
    asset::RenderAssetUsages,
    pbr::NotShadowCaster,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
    window::PrimaryWindow,
};

use super::{Art, Geometry, nature, place_scene, rgb, vertex_color};
use crate::{
    game::{Game, Settlement},
    play::CameraRig,
    region::{Biome, Bounds, CELL, Material, Region, Tile},
    terrain::{self, WATER, hash, noise, smooth},
};

const CHUNK: i16 = 16;
const DETAIL: i16 = 2;

#[derive(Component)]
pub struct LandscapeProp(pub Tile);

#[derive(Component, Clone, Copy)]
pub struct ForestPatch(pub Bounds);

#[derive(Component)]
pub struct MeadowPatch(Bounds);

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

    fn open_water(&self) -> Color {
        rgb(self.water[0]).mix(&rgb(self.water[1]), 0.31)
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
    let render_bounds = region.render_bounds();
    let bounds = render_bounds.padded(8);
    let sea_bounds = region.island_world().then_some(render_bounds);
    let occupied = occupied_land(city, region);
    if region.island_world() {
        // Continue the sea beyond the detailed chunks in the campaign overview.
        let center = (region.bounds.0.world() + region.bounds.1.world()) * 0.5;
        let mut sea = Geometry::default();
        sea.flat_box(
            center.with_y(WATER - 0.028),
            440.,
            440.,
            palette.open_water(),
        );
        parent.spawn((
            Mesh3d(meshes.add(sea.render_mesh())),
            MeshMaterial3d(art.water.clone()),
            Transform::default(),
            NotShadowCaster,
        ));
    }
    let Tile(left, north) = bounds.0;
    let Tile(right, south) = bounds.1.offset(1, 1);
    for z in (north..south).step_by(CHUNK as usize) {
        for x in (left..right).step_by(CHUNK as usize) {
            let end = Tile((x + CHUNK).min(right), (z + CHUNK).min(south));
            let (ground, water, foam) = chunk(region, &palette, Tile(x, z), end, sea_bounds);
            let patch_bounds = Bounds(Tile(x, z), end.offset(-1, -1));
            let land = !ground.positions.is_empty();
            for (mesh, material) in [
                (land.then(|| ground.terrain_mesh()), &art.terrain_surface),
                (water, &art.water),
                (
                    (!foam.positions.is_empty()).then(|| foam.render_mesh()),
                    &art.foam,
                ),
            ] {
                let Some(mesh) = mesh else { continue };
                parent.spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(material.clone()),
                    Transform::default(),
                    NotShadowCaster,
                ));
            }
            if land {
                parent.spawn((
                    MeadowPatch(patch_bounds),
                    Transform::default(),
                    Visibility::Hidden,
                    NotShadowCaster,
                ));
            }
            let patch = ForestPatch(patch_bounds);
            let forest = forest_geometry(region, patch, &occupied);
            if !forest.positions.is_empty() {
                parent.spawn((
                    patch,
                    Mesh3d(meshes.add(forest.render_mesh())),
                    MeshMaterial3d(art.ground.clone()),
                    Transform::default(),
                ));
            }
        }
    }
    landmarks(parent, art, region, &occupied);
}

pub fn meadow_detail(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    art: Res<Art>,
    game: Res<Game>,
    view: Res<CameraRig>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut patches: Query<(Entity, &MeadowPatch, Option<&Mesh3d>, &mut Visibility)>,
) {
    let close = view.rendered_height < 56.;
    let center = Tile::coordinates(view.target);
    let aspect = window.width() / window.height().max(1.);
    let radius = view.rendered_height / CELL * (0.5 * aspect.hypot(1.65)) + 6.;
    let mut pending = Vec::new();
    for (entity, patch, mesh, mut visible) in &mut patches {
        let Bounds(first, last) = patch.0;
        let nearest = center.clamp(first.local(), last.local());
        let distance = center.distance_squared(nearest);
        let show = close && distance <= radius * radius;
        let ready = mesh.is_some();
        visible.set_if_neq(if show && ready {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        if show && !ready {
            pending.push((distance, entity, patch.0));
        }
    }
    if pending.is_empty() {
        return;
    }
    pending.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
    let region = game.region();
    let palette = Palette::for_biome(region.kind);
    for (_, entity, bounds) in pending.into_iter().take(2) {
        let mut geometry = Geometry::default();
        for tile in bounds.tiles() {
            let key = hash(tile.0, tile.1, 223);
            if key % 3 == 0
                && !region.water(tile)
                && !region.road(tile)
                && !region.rocky(tile)
                && !region.fixed_structure(tile)
            {
                meadow(&mut geometry, region, &palette, tile, key);
            }
        }
        if geometry.positions.is_empty() {
            commands.entity(entity).remove::<MeadowPatch>();
        } else {
            commands.entity(entity).insert((
                Mesh3d(meshes.add(geometry.terrain_mesh())),
                MeshMaterial3d(art.terrain_surface.clone()),
                Visibility::Inherited,
            ));
        }
    }
}

fn occupied_land(city: &Settlement, region: &Region) -> BTreeSet<Tile> {
    city.buildings
        .iter()
        .flat_map(|site| site.kind.tiles(site.tile))
        .chain(city.roads.iter().copied())
        .chain(city.opening_farm(region))
        .collect()
}

pub fn forest_mesh(region: &Region, patch: ForestPatch, city: &Settlement) -> Mesh {
    forest_geometry(region, patch, &occupied_land(city, region)).render_mesh()
}

fn forest_geometry(region: &Region, patch: ForestPatch, occupied: &BTreeSet<Tile>) -> Geometry {
    let mut mesh = Geometry::default();
    let palette = Palette::for_biome(region.kind);
    let cliff = rgb(palette.cliff);
    for tile in patch.0.tiles() {
        let Tile(x, z) = tile;
        if region.water(tile)
            || region.road(tile)
            || region.fixed_structure(tile)
            || region.source_at(tile).is_some()
            || occupied.contains(&tile)
        {
            continue;
        }
        let key = hash(x, z, 171);
        let where_ = tile.local()
            + Vec2::new(
                ((key & 15) as f32 - 7.) * 0.051,
                (((key >> 4) & 15) as f32 - 7.) * 0.051,
            );
        let point = Tile::world_point(where_);
        let coast = terrain::shore(region, where_);
        if coast < 0.24 {
            continue;
        }
        let p = point.with_y(terrain::surface_height(region, where_, coast).max(WATER));
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
        if !clearing && p.y <= high && coast > 0.81 && (key % 1000) as f32 / 1000. < growth {
            let scale = 0.53 + ((key >> 16) % 39) as f32 * 0.021;
            nature::tree(&mut mesh, p - Vec3::Y * 0.055, scale, region.kind, key >> 9);
        } else if !clearing
            && (region.rocky(tile) || coast < 0.9 || grove < 0.34)
            && (key % 100) < if region.rocky(tile) { 35 } else { 22 }
        {
            let scale = if p.y > 2.0 { 0.93 } else { 0.54 } + ((key >> 8) % 19) as f32 * 0.036;
            let size = Vec3::new(scale, scale * (0.67 + grove * 0.31), scale * 0.84);
            nature::stone(&mut mesh, p - Vec3::Y * 0.07, size, cliff, key >> 12);
            if key % 3 == 0 {
                let small = Tile::world_point(where_) + Vec3::new(scale * 0.78, 0., scale * 0.31);
                nature::stone(
                    &mut mesh,
                    terrain::on_ground(region, small),
                    size * 0.48,
                    cliff,
                    key >> 10,
                );
            }
        } else if (clearing || coast < 1.6) && !region.rocky(tile) && key % 100 < 21 {
            wildflowers(&mut mesh, region, &palette, p, key);
        }
    }
    mesh
}

fn wildflowers(ground: &mut Geometry, region: &Region, palette: &Palette, center: Vec3, key: u32) {
    let leaves = rgb(palette.grass[0]).mix(&rgb(0x4a693b), 0.46);
    let petal = rgb(palette.flower);
    let turn = Quat::from_rotation_y((key & 31) as f32 * 0.203);
    for (index, position) in [(-0.23, 0.10), (0.11, -0.18), (0.22, 0.16)]
        .into_iter()
        .enumerate()
    {
        let base = terrain::on_ground(
            region,
            center + turn * Vec3::new(position.0, 0., position.1),
        );
        let stem = base + Vec3::Y * (0.20 + index as f32 * 0.07);
        for side in [-1., 1.] {
            let tip = base + turn * Vec3::new(side * 0.18, 0.13, -0.011);
            let bend = base + turn * Vec3::new(side * 0.065, 0.098, 0.055);
            ground.tri(base, bend, tip, leaves);
            ground.tri(tip, bend, base, leaves);
        }
        ground.tri(base + Vec3::X * 0.018, stem, base - Vec3::X * 0.018, leaves);
        ground.tri(base - Vec3::X * 0.018, stem, base + Vec3::X * 0.018, leaves);
        for ring in 0..5 {
            let angle = ring as f32 * std::f32::consts::TAU / 5.;
            let reach = Vec3::new(angle.cos(), 0.34, angle.sin());
            let across = Vec3::Y.cross(reach) * 0.034;
            let tip = stem + reach * 0.095;
            ground.quad(stem, tip - across, tip + reach * 0.025, tip + across, petal);
        }
        ground.flat_box(stem + Vec3::Y * 0.014, 0.022, 0.022, rgb(0xe9b666));
    }
}

fn chunk(
    region: &Region,
    palette: &Palette,
    start: Tile,
    end: Tile,
    sea_bounds: Option<Bounds>,
) -> (Geometry, Option<Mesh>, Geometry) {
    let mut ground = Geometry::default();
    let mut stream = Vec::new();
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
    let mut land_colors = vec![None; samples.len()];
    for z in 0..length as usize {
        for x in 0..width as usize {
            let i = z * stride + x;
            let [a, b, c, d] = [i, i + stride, i + stride + 1, i + 1];
            let triangles = if (x + z) % 2 == 0 {
                [[a, b, c], [a, c, d]]
            } else {
                [[b, c, d], [b, d, a]]
            };
            for face in triangles {
                let corners = face.map(|index| samples[index]);
                let land = corners.iter().any(|s| s.coast > 0.);
                let sea = corners.iter().any(|s| s.coast <= 0.);
                if land {
                    ground.tri_linear(
                        corners.map(|s| s.position),
                        face.map(|i| {
                            *land_colors[i].get_or_insert_with(|| {
                                vertex_color(land_color(region, palette, samples[i]))
                            })
                        }),
                    );
                }
                if sea {
                    stream.extend(face.map(|index| index as u16));
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
                    ripple(&mut foam, region, tile, key, sea_bounds);
                }
            } else if region.road(tile) {
                stones(&mut ground, region, palette, tile, key);
            }
        }
    }
    (
        ground,
        water_mesh(&samples, stream, palette, sea_bounds),
        foam,
    )
}

fn water_mesh(
    samples: &[Sample],
    mut triangles: Vec<u16>,
    palette: &Palette,
    bounds: Option<Bounds>,
) -> Option<Mesh> {
    if triangles.is_empty() {
        return None;
    }
    // Water is flat, so adjacent triangles can share vertices and upward normals.
    let mut vertex_indices = vec![None; samples.len()];
    let mut positions = Vec::new();
    let mut colors = Vec::new();
    for index in &mut triangles {
        let sample_index = usize::from(*index);
        *index = *vertex_indices[sample_index].get_or_insert_with(|| {
            let next = positions.len() as u16;
            let sample = samples[sample_index];
            positions.push(sample.position.with_y(WATER).to_array());
            colors.push(vertex_color(sea_color(palette, sample, bounds)));
            next
        });
    }
    let normals = vec![Vec3::Y.to_array(); positions.len()];
    Some(
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
        .with_inserted_indices(Indices::U16(triangles)),
    )
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

fn sea_color(palette: &Palette, s: Sample, bounds: Option<Bounds>) -> Color {
    let detail = bounds.map_or(1., |bounds| sea_detail(bounds, s.local));
    if detail == 0. {
        return palette.open_water();
    }
    let depth = -s.coast;
    let run = noise(Vec2::new(s.local.x * 0.67, s.local.y * 1.61), 0.98, 80);
    let broad = noise(s.local, 9.5, 86);
    let deep = rgb(palette.water[0]).mix(&rgb(palette.water[1]), 0.11 + broad * 0.37 + run * 0.14);
    let shallows = rgb(palette.water[1]).mix(&rgb(palette.water[2]), 0.28 + run * 0.51);
    let depth_fade = smooth((depth + broad * 0.88 - 0.45) / 3.7);
    let local_color = shallows.mix(&deep, depth_fade).mix(
        &rgb(0xc4e8ca),
        smooth((run - 0.54) / 0.46) * (1. - depth_fade) * 0.38,
    );
    palette.open_water().mix(&local_color, detail)
}

fn sea_detail(bounds: Bounds, point: Vec2) -> f32 {
    let edge = (point - bounds.0.local())
        .min(bounds.1.local() - point)
        .min_element();
    smooth((edge + 6.5) / 8.5)
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

fn ripple(foam: &mut Geometry, region: &Region, tile: Tile, key: u32, bounds: Option<Bounds>) {
    let p = tile.world().with_y(WATER + 0.013);
    let angle = (key % 31) as f32 * 0.033 - 0.53;
    let length = 0.43 + (key % 19) as f32 * 0.054;
    let along = Vec3::new(angle.cos(), 0., angle.sin()) * length * 0.5;
    let cross = along.cross(Vec3::Y) * 0.13;
    let depth = -terrain::shore(region, Tile::coordinates(p));
    let fade = bounds.map_or(1., |bounds| {
        sea_detail(bounds, tile.local()) * (1. - smooth((depth - 4.) / 17.))
    });
    if depth > 0.29 && fade > 0.025 {
        foam.quad(
            p - along,
            p + cross,
            p + along,
            p - cross,
            rgb(0xc7f4e9).with_alpha((0.18 + 0.12 * (key % 3) as f32) * fade),
        );
    }
}

fn stones(ground: &mut Geometry, region: &Region, palette: &Palette, tile: Tile, key: u32) {
    let base = rgb(palette.path);
    let vertical = region.road(tile.offset(0, 1)) || region.road(tile.offset(0, -1));
    let paved = region.kind == Biome::Aegean;
    let count = if paved { 8 } else { 3 };
    for i in 0..count {
        let column = if paved { i / 2 } else { i };
        let offset = if paved {
            (column as f32 - 1.5) * 0.42
        } else {
            (i as f32 - 1.) * 0.48
        };
        let irregular = ((hash(tile.0, tile.1, i + 31) % 21) as f32 - 10.) * 0.011;
        let shift = irregular
            + if paved {
                (i % 2) as f32 * 0.45 - 0.19
            } else {
                0.
            };
        let center = tile.world()
            + if vertical {
                Vec3::new(shift, 0., offset)
            } else {
                Vec3::new(offset, 0., shift)
            };
        let shade = hash(tile.0, tile.1, i + 223);
        let color = base.mix(&rgb(palette.cliff), 0.09 + (shade & 7) as f32 * 0.087);
        let size = if paved {
            Vec2::new(0.201, 0.218)
        } else {
            Vec2::new(0.21, 0.252)
        };
        let size = if vertical { size.yx() } else { size };
        ground.flagstone(region, center, size, key ^ shade, color);
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
        let close = tile.within(region.start, 23) || tile.within(region.rival_shrine, 20);
        let bank_prop = key % if close { 13 } else { 47 } == 0;
        let tree = matches!(region.kind, Biome::Fjord | Biome::Valley)
            && close
            && !tile.within(region.start, 7)
            && key % 81 == 0;
        let detail = key % if close { 19 } else { 193 } == 0;
        if !(bank_prop || tree || detail) {
            continue;
        }
        let near_water = tile.neighbors().into_iter().any(|t| region.water(t));
        let bank = near_water && bank_prop;
        let legacy_tree = tree && !near_water;
        if !(bank || legacy_tree || detail) {
            continue;
        }
        let p = terrain::on_ground(region, tile.world());
        let yaw = (key % 127) as f32 * 0.049;
        parent
            .spawn((
                LandscapeProp(tile),
                Transform::from_translation(p).with_rotation(Quat::from_rotation_y(yaw)),
                Visibility::Inherited,
            ))
            .with_children(|parent| {
                let p = Vec3::ZERO;
                if bank {
                    place_scene(parent, &art.waldwinkel.rock, p - Vec3::Y * 0.12, 0.72, 0.);
                } else if legacy_tree {
                    place_scene(parent, &art.waldwinkel.conifer, p, 0.6, 0.);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_faces_share_the_flat_grid_vertices() {
        let region = Region::practice();
        let palette = Palette::for_biome(region.kind);
        let start = Tile(-12, -12);
        let (land, water, _) = chunk(&region, &palette, start, start.offset(CHUNK, CHUNK), None);
        assert!(land.positions.is_empty());
        let water = water.unwrap();
        let sides = (CHUNK * DETAIL) as usize;
        assert_eq!(water.count_vertices(), (sides + 1).pow(2));
        assert_eq!(water.indices().unwrap().len(), 6 * sides.pow(2));
        let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(normals)) =
            water.attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            panic!("The water mesh has planar normals");
        };
        assert!(normals.iter().all(|normal| *normal == Vec3::Y.to_array()));
    }
}
