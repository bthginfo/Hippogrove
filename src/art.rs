use bevy::{asset::RenderAssetUsages, prelude::*, render::mesh::PrimitiveTopology};

use crate::{
    civic::Civic,
    game::{Building, Settlement},
    region::{Biome, CELL, Material, Region, Tile},
};

mod settlement_models;

pub fn rgb(hex: u32) -> Color {
    Color::srgb_u8((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

fn paint(assets: &mut Assets<StandardMaterial>, color: u32) -> Handle<StandardMaterial> {
    assets.add(StandardMaterial {
        base_color: rgb(color),
        perceptual_roughness: 0.9,
        ..default()
    })
}

fn glow(assets: &mut Assets<StandardMaterial>, color: u32, alpha: f32) -> Handle<StandardMaterial> {
    assets.add(StandardMaterial {
        base_color: rgb(color).with_alpha(alpha),
        unlit: true,
        alpha_mode: if alpha < 1. {
            AlphaMode::Blend
        } else {
            AlphaMode::Opaque
        },
        ..default()
    })
}

#[derive(Resource)]
pub struct Art {
    pub cube: Handle<Mesh>,
    pub lump: Handle<Mesh>,
    pub cylinder: Handle<Mesh>,
    cone: Handle<Mesh>,
    pub roof: Handle<Mesh>,
    pub curved_roof: Handle<Mesh>,
    pub ring: Handle<Mesh>,
    pub plan_corners: Handle<Mesh>,
    pub plan_cross: Handle<Mesh>,
    pub grass: Handle<StandardMaterial>,
    pub leaf: [Handle<StandardMaterial>; 3],
    pub evergreen: [Handle<StandardMaterial>; 3],
    pub moss: Handle<StandardMaterial>,
    pub clay: Handle<StandardMaterial>,
    pub timber: Handle<StandardMaterial>,
    pub plank: Handle<StandardMaterial>,
    pub dark_wood: Handle<StandardMaterial>,
    pub plaster: Handle<StandardMaterial>,
    pub stone: Handle<StandardMaterial>,
    pub warm_stone: Handle<StandardMaterial>,
    pub door: Handle<StandardMaterial>,
    pub roof_color: [Handle<StandardMaterial>; 5],
    pub ridge: Handle<StandardMaterial>,
    pub amber: Handle<StandardMaterial>,
    pub white: Handle<StandardMaterial>,
    pub purple: Handle<StandardMaterial>,
    pub boot: Handle<StandardMaterial>,
    pub skin: Handle<StandardMaterial>,
    pub eye: Handle<StandardMaterial>,
    pub berry: Handle<StandardMaterial>,
    pub gold: Handle<StandardMaterial>,
    pub ripple: Handle<StandardMaterial>,
    pub blue: Handle<StandardMaterial>,
    pub red: Handle<StandardMaterial>,
    pub limestone: Handle<StandardMaterial>,
    pub cobalt: Handle<StandardMaterial>,
    pub valid: Handle<StandardMaterial>,
    pub invalid: Handle<StandardMaterial>,
    pub ghost: Handle<StandardMaterial>,
    pub blocked: Handle<StandardMaterial>,
    pub safe_zone: Handle<StandardMaterial>,
    pub shadow: Handle<StandardMaterial>,
    pub ground: Handle<StandardMaterial>,
    pub hippo_model: Handle<Scene>,
}

impl Art {
    pub fn load(
        meshes: &mut Assets<Mesh>,
        mats: &mut Assets<StandardMaterial>,
        assets: &AssetServer,
    ) -> Self {
        Self {
            cube: meshes.add(Cuboid::from_size(Vec3::ONE)),
            lump: meshes.add(Sphere::new(0.5).mesh().ico(1).unwrap()),
            cylinder: meshes.add(Cylinder::new(0.5, 1.).mesh().resolution(9)),
            cone: meshes.add(Cone::new(0.5, 1.).mesh().resolution(8)),
            roof: meshes.add(roof_mesh(false)),
            curved_roof: meshes.add(roof_mesh(true)),
            ring: meshes.add(ring_mesh()),
            plan_corners: meshes.add(plan_corners()),
            plan_cross: meshes.add(plan_cross()),
            grass: paint(mats, 0x91a251),
            leaf: [
                paint(mats, 0x9ca74c),
                paint(mats, 0xadb359),
                paint(mats, 0x728540),
            ],
            evergreen: [
                paint(mats, 0x596d45),
                paint(mats, 0x697e48),
                paint(mats, 0x81914e),
            ],
            moss: paint(mats, 0xa9ad62),
            clay: paint(mats, 0xa57856),
            timber: paint(mats, 0x8c5736),
            plank: paint(mats, 0xc59762),
            dark_wood: paint(mats, 0x5f4835),
            plaster: paint(mats, 0xe6cba0),
            stone: paint(mats, 0x8e8782),
            warm_stone: paint(mats, 0xbca48a),
            door: paint(mats, 0x4e7d72),
            roof_color: [
                paint(mats, 0xbd6b42),
                paint(mats, 0x485969),
                paint(mats, 0x51796c),
                paint(mats, 0xc69c57),
                paint(mats, 0xce764d),
            ],
            ridge: paint(mats, 0xd18d54),
            amber: glow(mats, 0xffd38a, 1.),
            white: paint(mats, 0xffedc9),
            purple: paint(mats, 0xa790b9),
            boot: paint(mats, 0x7f6a93),
            skin: paint(mats, 0xebbb93),
            eye: paint(mats, 0x382c35),
            berry: paint(mats, 0xb95743),
            gold: glow(mats, 0xf5dc97, 1.),
            ripple: glow(mats, 0x92decc, 0.62),
            blue: paint(mats, 0x526988),
            red: paint(mats, 0xa75548),
            limestone: paint(mats, 0xecdec5),
            cobalt: paint(mats, 0x3d74a7),
            valid: glow(mats, 0xc2ebbd, 1.),
            invalid: glow(mats, 0xffb29b, 1.),
            ghost: glow(mats, 0x8fc4a4, 0.44),
            blocked: glow(mats, 0xce7361, 0.42),
            safe_zone: glow(mats, 0xe9ca81, 0.85),
            shadow: glow(mats, 0x3d4b3a, 0.19),
            ground: mats.add(StandardMaterial {
                base_color: Color::WHITE,
                perceptual_roughness: 0.96,
                ..default()
            }),
            hippo_model: assets.load("embedded://models/hippo.glb#Scene0"),
        }
    }
}

pub fn part(
    parent: &mut ChildSpawnerCommands,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    position: Vec3,
    size: Vec3,
    rotation: Quat,
) -> Entity {
    parent
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform {
                translation: position,
                scale: size,
                rotation,
            },
        ))
        .id()
}

pub fn block(
    parent: &mut ChildSpawnerCommands,
    art: &Art,
    mat: &Handle<StandardMaterial>,
    position: Vec3,
    size: Vec3,
) -> Entity {
    part(parent, &art.cube, mat, position, size, Quat::IDENTITY)
}

pub fn mound(
    parent: &mut ChildSpawnerCommands,
    art: &Art,
    mat: &Handle<StandardMaterial>,
    position: Vec3,
    size: Vec3,
) -> Entity {
    part(parent, &art.lump, mat, position, size, Quat::IDENTITY)
}

#[derive(Default)]
pub struct Geometry {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
}

impl Geometry {
    pub fn tri(&mut self, a: Vec3, b: Vec3, c: Vec3, color: Color) {
        self.tri_colors([a, b, c], [color; 3]);
    }

    fn tri_colors(&mut self, points: [Vec3; 3], colors: [Color; 3]) {
        let [a, b, c] = points;
        let normal = (b - a).cross(c - a).normalize_or_zero();
        for (p, color) in points.into_iter().zip(colors) {
            let lin = color.to_linear();
            self.positions.push(p.to_array());
            self.normals.push(normal.to_array());
            self.colors.push([lin.red, lin.green, lin.blue, lin.alpha]);
        }
    }

    pub fn quad(&mut self, a: Vec3, b: Vec3, c: Vec3, d: Vec3, color: Color) {
        self.tri(a, b, c, color);
        self.tri(a, c, d, color);
    }

    pub fn flat_box(&mut self, center: Vec3, half_x: f32, half_z: f32, color: Color) {
        self.quad(
            center + Vec3::new(-half_x, 0., -half_z),
            center + Vec3::new(-half_x, 0., half_z),
            center + Vec3::new(half_x, 0., half_z),
            center + Vec3::new(half_x, 0., -half_z),
            color,
        );
    }

    pub fn mesh(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
    }
}

fn roof_mesh(curved: bool) -> Mesh {
    let roofline: &[(f32, f32)] = if curved {
        &[
            (-0.5, 0.14),
            (-0.39, 0.09),
            (-0.2, 0.27),
            (0., 0.56),
            (0.2, 0.27),
            (0.39, 0.09),
            (0.5, 0.14),
        ]
    } else {
        &[(-0.5, 0.), (0., 0.55), (0.5, 0.)]
    };
    let mut geometry = Geometry::default();
    for pair in roofline.windows(2) {
        let [(left, bottom), (right, top)] = [pair[0], pair[1]];
        geometry.quad(
            Vec3::new(left, bottom, -0.5),
            Vec3::new(left, bottom, 0.5),
            Vec3::new(right, top, 0.5),
            Vec3::new(right, top, -0.5),
            Color::WHITE,
        );
        geometry.tri(
            Vec3::new(left, bottom, 0.5),
            Vec3::new(left, -0.04, 0.5),
            Vec3::new(right, top, 0.5),
            rgb(0xa99f8c),
        );
        geometry.tri(
            Vec3::new(right, top, -0.5),
            Vec3::new(left, -0.04, -0.5),
            Vec3::new(left, bottom, -0.5),
            rgb(0xafa48e),
        );
        geometry.tri(
            Vec3::new(left, -0.04, 0.5),
            Vec3::new(right, -0.04, 0.5),
            Vec3::new(right, top, 0.5),
            rgb(0xa99f8c),
        );
        geometry.tri(
            Vec3::new(right, top, -0.5),
            Vec3::new(right, -0.04, -0.5),
            Vec3::new(left, -0.04, -0.5),
            rgb(0xafa48e),
        );
    }
    geometry.mesh()
}

fn ring_mesh() -> Mesh {
    let mut geometry = Geometry::default();
    for index in 0..48 {
        let angle = |i: usize| i as f32 * std::f32::consts::TAU / 48.;
        let point = |i, radius| Vec3::new(angle(i).sin() * radius, 0., angle(i).cos() * radius);
        geometry.quad(
            point(index, 0.93),
            point(index, 1.),
            point(index + 1, 1.),
            point(index + 1, 0.93),
            Color::WHITE,
        );
    }
    geometry.mesh()
}

fn plan_corners() -> Mesh {
    let mut geometry = Geometry::default();
    for sx in [-1., 1.] {
        for sz in [-1., 1.] {
            geometry.flat_box(
                Vec3::new(sx * 0.40, 0., sz * 0.49),
                0.10,
                0.014,
                Color::WHITE,
            );
            geometry.flat_box(
                Vec3::new(sx * 0.49, 0., sz * 0.40),
                0.014,
                0.10,
                Color::WHITE,
            );
        }
    }
    geometry.mesh()
}

fn plan_cross() -> Mesh {
    let mut geometry = Geometry::default();
    for sign in [-1., 1.] {
        let start = Vec3::new(-0.20, 0., -sign * 0.20);
        let end = -start;
        let width = Vec3::new(sign * 0.024, 0., -0.024);
        geometry.quad(
            start - width,
            end - width,
            end + width,
            start + width,
            Color::WHITE,
        );
        geometry.quad(
            start + width,
            end + width,
            end - width,
            start - width,
            Color::WHITE,
        );
    }
    geometry.mesh()
}

fn hash(x: i16, z: i16, salt: u32) -> u32 {
    let n = (x as u32).wrapping_mul(1973) ^ (z as u32).wrapping_mul(9277) ^ salt;
    let n = (n ^ (n >> 16)).wrapping_mul(0x45d9f3b);
    n ^ (n >> 16)
}

fn relief(region: &Region, x: i16, z: i16) -> f32 {
    let tile = Tile(x, z);
    if [
        tile,
        tile.offset(-1, 0),
        tile.offset(0, -1),
        tile.offset(-1, -1),
    ]
    .into_iter()
    .any(|p| region.water(p))
    {
        return 0.;
    }
    let point = Vec2::new(f32::from(x) - 0.5, f32::from(z) - 0.5);
    let wave = (point.x * 0.118 + point.y * 0.68).sin() * (point.y * 0.22 - point.x * 0.39).cos();
    let scale = match region.kind {
        Biome::Valley => 0.96,
        Biome::Fjord => 1.92,
        Biome::Terraces => 1.36,
        Biome::Lake => 1.24,
        Biome::Aegean => 1.12,
    };
    let mut height: f32 = 0.;
    for &(center, radius) in &region.highlands {
        let radius = f32::from(radius);
        let offset = point - Vec2::new(f32::from(center.0), f32::from(center.1));
        let distance = offset.length() / radius;
        if distance < 1. {
            let slope = 1. - distance;
            let turbulence = 0.82 + wave * 0.12 + offset.x * 0.042;
            let profile = if region.kind == Biome::Terraces {
                (slope * 4.).floor() * 0.205 + slope * 0.18
            } else {
                slope.sqrt() * slope.powf(0.38)
            };
            height = height.max(scale * radius * profile * turbulence);
        }
    }
    let outside = (region.bounds.0.1 - z)
        .max(region.bounds.0.0 - x)
        .max(x - region.bounds.1.0);
    if outside > 0 && region.kind != Biome::Aegean {
        let roll = ((point.x * 0.32 + point.y * 0.45).sin()
            + (point.x * 0.11 - point.y * 0.27).cos())
            * 0.28;
        height = height.max(f32::from(outside) * scale * (0.68 + roll));
    }
    height
}

pub fn landscape(region: &Region) -> Mesh {
    let (land, path, water): (&[u32], &[u32], &[u32]) = match region.kind {
        Biome::Valley => (
            &[0xaab96d, 0x99aa61, 0xb0b974, 0xa3b569],
            &[0xcbb383, 0xd0b889, 0xc5aa78],
            &[0x54a39f, 0x56a9a7, 0x519f9f, 0x5aada8],
        ),
        Biome::Fjord => (
            &[0x939b74, 0x9fa280, 0x889578, 0xabb39c],
            &[0xbab3a2, 0xc3b8a6, 0xb6ad9e],
            &[0x4c8195, 0x5890a1, 0x487b8a, 0x6ba2ac],
        ),
        Biome::Terraces => (
            &[0xa2ae5d, 0xaab459, 0x98a654, 0xa8b46a],
            &[0xc2a874, 0xd3ba84, 0xc7af82],
            &[0x4f9b90, 0x5fa99a, 0x70b4a1, 0x4b978c],
        ),
        Biome::Lake => (
            &[0xc3a36f, 0xcbab77, 0xc1a477, 0xc9ae7d],
            &[0xd9b88c, 0xccb083, 0xc6a97f],
            &[0x53a8a5, 0x60b3ad, 0x489b9d, 0x6cbab0],
        ),
        Biome::Aegean => (
            &[0xb9b47a, 0xa9b66a, 0xc7bc89, 0xa4ae70],
            &[0xe8d6b3, 0xdbc5a5, 0xe7cfaa],
            &[0x2c94b3, 0x389eb4, 0x2d88a9, 0x4aaab8],
        ),
    };
    let mut ground = Geometry::default();
    let half = CELL * 0.5;
    for tile in region.bounds.padded(7).tiles() {
        let Tile(x, z) = tile;
        let wet = region.water(tile);
        let road = !wet && region.road(tile);
        let meadow = if wet {
            water
        } else if road {
            path
        } else {
            land
        };
        let center = tile.world() + Vec3::Y * if wet { -0.26 } else { 0. };
        let beach = region.kind == Biome::Aegean
            && !wet
            && tile
                .neighbors()
                .into_iter()
                .any(|neighbor| region.water(neighbor));
        let corner = |dx: i16, dz: i16| {
            let dx_offset = (hash(x + dx, z + dz, 90) % 21) as f32 * 0.018 - 0.18;
            let dz_offset = (hash(x + dx, z + dz, 47) % 21) as f32 * 0.018 - 0.18;
            center
                + Vec3::new(
                    f32::from(2 * dx - 1) * half + dx_offset,
                    if wet {
                        0.
                    } else {
                        relief(region, x + dx, z + dz)
                    },
                    f32::from(2 * dz - 1) * half + dz_offset,
                )
        };
        let corners = [corner(0, 0), corner(0, 1), corner(1, 1), corner(1, 0)];
        if !wet && !road {
            let grass = |i: usize, dx: i16, dz: i16| {
                let corner = tile.offset(dx, dz);
                let seed = hash(corner.0.div_euclid(6), corner.1.div_euclid(5), 21);
                let color = rgb(land[seed as usize % land.len()]);
                let neighbor = rgb(land[(seed as usize + 1) % land.len()]);
                let mixing =
                    ((corners[i].x * 0.14).sin() * (corners[i].z * 0.13).cos() + 1.) * 0.35;
                let base = if beach {
                    rgb(0xdac79c)
                } else {
                    color.mix(&neighbor, mixing)
                };
                let y = corners[i].y;
                let cliff = match region.kind {
                    Biome::Valley => rgb(0x9b9279),
                    Biome::Fjord => rgb(0x8a98a9),
                    Biome::Terraces => rgb(0x7f9854),
                    Biome::Lake => rgb(0xce9267),
                    Biome::Aegean => rgb(0xd2bfa5),
                };
                let color = base.mix(&cliff, ((y - 0.55) / 2.2).clamp(0., 0.96));
                if region.kind == Biome::Fjord {
                    color.mix(&rgb(0xe6e8e4), ((y - 3.8) / 2.5).clamp(0., 0.95))
                } else {
                    color
                }
            };
            let [a, b, c, d] = corners;
            let [ca, cb, cc, cd] = [
                grass(0, 0, 0),
                grass(1, 0, 1),
                grass(2, 1, 1),
                grass(3, 1, 0),
            ];
            if (x + z).rem_euclid(2) == 0 {
                ground.tri_colors([a, b, c], [ca, cb, cc]);
                ground.tri_colors([a, c, d], [ca, cc, cd]);
            } else {
                ground.tri_colors([b, c, d], [cb, cc, cd]);
                ground.tri_colors([b, d, a], [cb, cd, ca]);
            }
        } else {
            let seed = hash(x, z, 28) as usize;
            ground.quad(
                corners[0],
                corners[1],
                corners[2],
                corners[3],
                rgb(meadow[seed % meadow.len()]),
            );
        }
        if !wet {
            for (adjacent, left, right) in [
                (tile.offset(-1, 0), 0, 1),
                (tile.offset(0, 1), 1, 2),
                (tile.offset(1, 0), 2, 3),
                (tile.offset(0, -1), 3, 0),
            ] {
                if region.water(adjacent) {
                    let a = corners[left];
                    let b = corners[right];
                    let bank = if region.kind == Biome::Aegean {
                        0xd5bba1
                    } else {
                        0x9a876b
                    };
                    ground.quad(b, a, a - Vec3::Y * 0.27, b - Vec3::Y * 0.27, rgb(bank));
                }
            }
        }
        if road {
            for (ox, oz, width) in [
                (-0.36, -0.30, 0.20),
                (0.23, -0.08, 0.24),
                (-0.14, 0.43, 0.26),
            ] {
                ground.flat_box(
                    center + Vec3::new(ox, 0.018, oz),
                    width,
                    0.13,
                    rgb(0xd1b98f),
                );
            }
        }
    }
    ground.mesh()
}

fn conifer(parent: &mut ChildSpawnerCommands, art: &Art, p: Vec3, scale: f32, snow: bool) {
    part(
        parent,
        &art.cylinder,
        &art.timber,
        p + Vec3::Y * scale * 0.48,
        Vec3::new(0.35, 0.97, 0.35) * scale,
        Quat::IDENTITY,
    );
    for (index, (height, width)) in [(1.24, 1.70), (1.90, 1.37), (2.57, 0.97)]
        .into_iter()
        .enumerate()
    {
        part(
            parent,
            &art.cone,
            &art.evergreen[index],
            p + Vec3::Y * height * scale,
            Vec3::new(width, 1.32, width) * scale,
            Quat::from_rotation_y(index as f32 * 0.38),
        );
    }
    if snow {
        part(
            parent,
            &art.cone,
            &art.white,
            p + Vec3::Y * 2.82 * scale,
            Vec3::new(0.68, 0.95, 0.68) * scale,
            Quat::from_rotation_y(0.64),
        );
    }
}

fn broadleaf(parent: &mut ChildSpawnerCommands, art: &Art, p: Vec3, scale: f32, biome: Biome) {
    part(
        parent,
        &art.cylinder,
        &art.timber,
        p + Vec3::Y * scale * 0.93,
        Vec3::new(0.28, 1.86, 0.28) * scale,
        Quat::from_rotation_z(0.08),
    );
    for (i, (x, z, y)) in [(0., 0., 2.25), (-0.61, 0.13, 1.98), (0.60, -0.12, 2.12)]
        .into_iter()
        .enumerate()
    {
        let mat = if biome == Biome::Terraces && i != 0 {
            &art.red
        } else {
            &art.leaf[i]
        };
        let spread = if biome == Biome::Lake {
            Vec3::new(1.9, 0.63, 1.45)
        } else if biome == Biome::Aegean {
            Vec3::new(1.49, 0.82, 1.24)
        } else {
            Vec3::new(1.55, 1.54, 1.35)
        };
        mound(
            parent,
            art,
            mat,
            p + Vec3::new(x, y, z) * scale,
            spread * scale,
        );
    }
    if biome == Biome::Valley {
        for (i, (x, z, y)) in [(-0.69, 0.40, 1.59), (0.62, 0.30, 1.59), (0.13, 0.62, 1.72)]
            .into_iter()
            .enumerate()
        {
            mound(
                parent,
                art,
                &art.leaf[i],
                p + Vec3::new(x, y, z) * scale,
                Vec3::new(0.57, 1.51, 0.66) * scale,
            );
        }
    }
}

pub fn scenery(parent: &mut ChildSpawnerCommands, art: &Art, region: &Region) {
    for tile in region.bounds.padded(5).tiles() {
        let Tile(x, z) = tile;
        let p = tile.world() + Vec3::Y * relief(region, x, z);
        let choice = hash(x, z, 93);
        let near_water = tile.neighbors().into_iter().any(|q| region.water(q));
        if region.water(tile) {
            let divisor = if region.kind == Biome::Aegean && !near_water {
                53
            } else {
                11
            };
            if choice % divisor == 0 && !region.bridge_cell(tile) {
                block(
                    parent,
                    art,
                    &art.ripple,
                    p + Vec3::Y * -0.245,
                    Vec3::new(0.70, 0.01, 0.035),
                );
                if choice % 3 == 0 {
                    let (surface, y, size) = match region.kind {
                        Biome::Fjord => (&art.white, -0.20, Vec3::new(0.72, 0.14, 0.44)),
                        Biome::Terraces => (&art.leaf[0], -0.24, Vec3::new(0.46, 0.07, 0.38)),
                        Biome::Aegean => (&art.ripple, -0.25, Vec3::new(0.88, 0.035, 0.12)),
                        _ => (&art.leaf[2], -0.24, Vec3::new(0.27, 0.06, 0.23)),
                    };
                    mound(parent, art, surface, p + Vec3::Y * y, size);
                    if region.kind == Biome::Terraces {
                        mound(
                            parent,
                            art,
                            &art.berry,
                            p + Vec3::Y * -0.18,
                            Vec3::new(0.16, 0.16, 0.16),
                        );
                    }
                }
            }
            continue;
        }
        if region.rocky(tile) {
            continue;
        }
        if near_water && !region.road(tile) && choice % 4 == 0 {
            mound(
                parent,
                art,
                if region.kind == Biome::Aegean {
                    &art.limestone
                } else {
                    &art.warm_stone
                },
                p + Vec3::Y * 0.11,
                Vec3::new(1.15, 0.45, 0.69),
            );
        }
        let playable = tile.within(region.start, 8)
            || tile.within(region.frontier, 9)
            || tile.within(region.rival_shrine, 7)
            || region
                .districts
                .iter()
                .any(|district| tile.within(district.center, 9));
        let inner = tile.within(region.start, 24);
        if !playable
            && !near_water
            && !region.road(tile)
            && choice % if inner { 6 } else { 23 } == 0
        {
            let scale = 0.58 + (choice % 39) as f32 * 0.023;
            if region.kind == Biome::Aegean && hash(x, z, 57) % 4 == 0 {
                part(
                    parent,
                    &art.cone,
                    &art.evergreen[0],
                    p + Vec3::Y * 1.32 * scale,
                    Vec3::new(0.65, 2.64, 0.63) * scale,
                    Quat::IDENTITY,
                );
                mound(
                    parent,
                    art,
                    &art.evergreen[1],
                    p + Vec3::new(0.07, 1.01 * scale, 0.04),
                    Vec3::new(0.72, 1.69, 0.61) * scale,
                );
            } else if region.kind == Biome::Lake
                || (region.kind != Biome::Fjord && hash(x, z, 69) % 5 != 0)
            {
                broadleaf(parent, art, p, scale, region.kind);
            } else {
                conifer(parent, art, p, scale, region.kind == Biome::Fjord);
            }
        } else if !playable && !region.road(tile) && choice % if inner { 9 } else { 29 } == 0 {
            mound(
                parent,
                art,
                &art.stone,
                p + Vec3::Y * 0.23,
                Vec3::new(1.39, 0.9, 0.94),
            );
            if region.kind == Biome::Fjord {
                mound(
                    parent,
                    art,
                    &art.white,
                    p + Vec3::Y * 0.54,
                    Vec3::new(0.98, 0.31, 0.77),
                );
            }
        } else if !region.road(tile)
            && !region.fixed_structure(tile)
            && region.source_at(tile).is_none()
            && choice % if inner { 7 } else { 37 } == 0
        {
            let plant = p + Vec3::new(0.55, 0.10, 0.60);
            let size = if near_water { 0.53 } else { 0.18 };
            part(
                parent,
                &art.cone,
                &art.grass,
                plant,
                Vec3::new(0.17, size, 0.13),
                Quat::from_rotation_z(0.2),
            );
            if choice % 3 == 0 {
                mound(
                    parent,
                    art,
                    &art.amber,
                    plant + Vec3::Y * size * 0.43,
                    Vec3::splat(0.085),
                );
            }
        }
    }
}

pub fn resources(parent: &mut ChildSpawnerCommands, art: &Art, material: Material) {
    match material {
        Material::Wood => {
            for (x, y, z) in [
                (-0.22, 0.18, 0.03),
                (0.19, 0.18, -0.13),
                (0.05, 0.45, -0.03),
            ] {
                part(
                    parent,
                    &art.cylinder,
                    &art.timber,
                    Vec3::new(x, y, z),
                    Vec3::new(0.31, 1.17, 0.31),
                    Quat::from_rotation_x(1.57),
                );
                part(
                    parent,
                    &art.cylinder,
                    &art.plank,
                    Vec3::new(x, y, z + 0.595),
                    Vec3::new(0.27, 0.04, 0.27),
                    Quat::from_rotation_x(1.57),
                );
            }
        }
        Material::Stone => {
            for (x, y, z, scale) in [
                (0.05, 0.31, 0., 1.),
                (-0.36, 0.14, 0.32, 0.55),
                (0.40, 0.16, 0.35, 0.49),
            ] {
                mound(
                    parent,
                    art,
                    &art.stone,
                    Vec3::new(x, y, z),
                    Vec3::new(0.79, 0.67, 0.77) * scale,
                );
            }
        }
        Material::Food => {
            mound(
                parent,
                art,
                &art.leaf[0],
                Vec3::new(0., 0.4, 0.),
                Vec3::new(1.13, 0.89, 1.05),
            );
            mound(
                parent,
                art,
                &art.leaf[2],
                Vec3::new(-0.29, 0.24, 0.38),
                Vec3::splat(0.57),
            );
            for (x, y, z) in [
                (-0.24, 0.59, 0.37),
                (0.16, 0.58, 0.32),
                (0.38, 0.44, 0.29),
                (-0.39, 0.34, 0.56),
                (0., 0.30, 0.46),
            ] {
                mound(
                    parent,
                    art,
                    &art.berry,
                    Vec3::new(x, y, z),
                    Vec3::splat(0.17),
                );
            }
        }
    }
}

pub fn parcel(
    parent: &mut ChildSpawnerCommands,
    art: &Art,
    material: Material,
    at: Vec3,
    scale: f32,
) {
    match material {
        Material::Wood => {
            part(
                parent,
                &art.cylinder,
                &art.timber,
                at,
                Vec3::new(0.29, 0.85, 0.29) * scale,
                Quat::from_rotation_z(1.57),
            );
            part(
                parent,
                &art.cylinder,
                &art.plank,
                at + Vec3::X * 0.43 * scale,
                Vec3::new(0.25, 0.045, 0.25) * scale,
                Quat::from_rotation_z(1.57),
            );
        }
        Material::Stone => {
            mound(
                parent,
                art,
                &art.stone,
                at,
                Vec3::new(0.68, 0.38, 0.40) * scale,
            );
        }
        Material::Food => {
            block(
                parent,
                art,
                &art.plank,
                at,
                Vec3::new(0.52, 0.24, 0.36) * scale,
            );
            mound(
                parent,
                art,
                &art.berry,
                at + Vec3::Y * 0.14 * scale,
                Vec3::new(0.46, 0.20, 0.34) * scale,
            );
        }
    }
}

pub fn shrine(parent: &mut ChildSpawnerCommands, art: &Art, frontier: bool) {
    block(
        parent,
        art,
        &art.stone,
        Vec3::Y * 0.12,
        Vec3::new(1.36, 0.24, 1.24),
    );
    block(
        parent,
        art,
        &art.warm_stone,
        Vec3::Y * 0.29,
        Vec3::new(1.1, 0.18, 1.0),
    );
    if frontier {
        part(
            parent,
            &art.ring,
            &art.gold,
            Vec3::Y * 0.033,
            Vec3::splat(1.02),
            Quat::IDENTITY,
        );
    }
    for x in [-0.44, 0.44] {
        block(
            parent,
            art,
            &art.timber,
            Vec3::new(x, 0.84, 0.),
            Vec3::new(0.12, 1.04, 0.16),
        );
    }
    block(
        parent,
        art,
        &art.dark_wood,
        Vec3::new(0., 0.82, -0.24),
        Vec3::new(0.91, 1., 0.13),
    );
    part(
        parent,
        &art.roof,
        &art.roof_color[0],
        Vec3::Y * 1.32,
        Vec3::new(1.22, 0.76, 0.78),
        Quat::IDENTITY,
    );
    block(
        parent,
        art,
        &art.amber,
        Vec3::new(0., 0.67, 0.15),
        Vec3::new(0.20, 0.39, 0.15),
    );
    mound(
        parent,
        art,
        &art.amber,
        Vec3::new(0., 0.94, 0.16),
        Vec3::splat(0.31),
    );
}

pub fn building(parent: &mut ChildSpawnerCommands, art: &Art, biome: Biome, kind: Building) {
    if let Building::Civic(kind) = kind {
        settlement_models::draw(parent, art, biome, kind);
        return;
    }
    if biome == Biome::Aegean && kind == Building::Shrine {
        settlement_models::draw(parent, art, biome, Civic::Oracle);
        return;
    }
    if kind == Building::Shrine {
        shrine(parent, art, true);
        return;
    }
    if kind == Building::Waystone {
        mound(
            parent,
            art,
            &art.warm_stone,
            Vec3::new(0., 0.19, 0.),
            Vec3::new(1.32, 0.38, 1.18),
        );
        part(
            parent,
            &art.cylinder,
            &art.stone,
            Vec3::Y * 0.62,
            Vec3::new(0.60, 0.93, 0.60),
            Quat::from_rotation_z(0.16),
        );
        mound(
            parent,
            art,
            &art.gold,
            Vec3::new(-0.05, 1.15, 0.),
            Vec3::splat(0.32),
        );
        return;
    }
    if kind == Building::Field {
        block(
            parent,
            art,
            &art.clay,
            Vec3::Y * 0.05,
            Vec3::new(2.95, 0.17, 2.72),
        );
        for x in [-0.96, -0.32, 0.32, 0.96] {
            block(
                parent,
                art,
                &art.dark_wood,
                Vec3::new(x, 0.15, 0.),
                Vec3::new(0.39, 0.13, 2.43),
            );
        }
        for x in [-1.47, 1.47] {
            block(
                parent,
                art,
                &art.plank,
                Vec3::new(x, 0.25, 0.),
                Vec3::new(0.13, 0.24, 2.86),
            );
        }
        return;
    }
    if matches!(kind, Building::Well | Building::Market | Building::Tower) {
        civic_building(parent, art, biome, kind);
        return;
    }
    let roof_color = &art.roof_color[biome as usize];
    let h = if kind == Building::Store { 1.27 } else { 1.59 };
    let width = if biome == Biome::Fjord { 2.55 } else { 2.36 };
    let length = if kind == Building::Workshop {
        2.66
    } else {
        2.18
    };
    block(
        parent,
        art,
        &art.warm_stone,
        Vec3::Y * 0.15,
        Vec3::new(width + 0.17, 0.3, length + 0.24),
    );
    if biome == Biome::Lake && kind == Building::House {
        part(
            parent,
            &art.cylinder,
            &art.plaster,
            Vec3::Y * 0.91,
            Vec3::new(2.35, 1.47, 2.35),
            Quat::IDENTITY,
        );
        part(
            parent,
            &art.cone,
            roof_color,
            Vec3::Y * 2.12,
            Vec3::new(2.89, 1.21, 2.89),
            Quat::IDENTITY,
        );
        block(
            parent,
            art,
            &art.door,
            Vec3::new(0., 0.77, 1.14),
            Vec3::new(0.57, 0.95, 0.13),
        );
        block(
            parent,
            art,
            &art.amber,
            Vec3::new(0.61, 1.17, 0.98),
            Vec3::new(0.27, 0.32, 0.07),
        );
        return;
    }

    let wall = if biome == Biome::Aegean {
        &art.limestone
    } else if kind == Building::Store || biome == Biome::Fjord {
        &art.timber
    } else {
        &art.plaster
    };
    block(
        parent,
        art,
        wall,
        Vec3::Y * (0.26 + h * 0.5),
        Vec3::new(width, h, length),
    );
    let roof_height = if biome == Biome::Fjord { 1.75 } else { 1.26 };
    let roof_shape = if biome == Biome::Terraces {
        &art.curved_roof
    } else {
        &art.roof
    };
    let roof_y = 0.23 + h;
    part(
        parent,
        roof_shape,
        roof_color,
        Vec3::Y * roof_y,
        Vec3::new(width + 0.65, roof_height, length + 0.61),
        Quat::IDENTITY,
    );
    block(
        parent,
        art,
        &art.ridge,
        Vec3::Y * (roof_y + roof_height * 0.55),
        Vec3::new(0.11, 0.12, length + 0.69),
    );
    for x in [-width * 0.47, width * 0.47] {
        for z in [-length * 0.49, length * 0.49] {
            block(
                parent,
                art,
                &art.dark_wood,
                Vec3::new(x, h * 0.5 + 0.24, z),
                Vec3::new(0.15, h, 0.15),
            );
        }
        block(
            parent,
            art,
            &art.timber,
            Vec3::new(x, 1.16, 0.),
            Vec3::new(0.13, 0.12, length + 0.12),
        );
    }
    let face = length * 0.5 + 0.045;
    block(
        parent,
        art,
        &art.timber,
        Vec3::new(0., h + 0.19, face),
        Vec3::new(width, 0.14, 0.13),
    );
    block(
        parent,
        art,
        &art.timber,
        Vec3::new(0., 0.45, face + 0.22),
        Vec3::new(0.93, 0.19, 0.51),
    );
    if kind == Building::Store || kind == Building::Granary {
        block(
            parent,
            art,
            &art.dark_wood,
            Vec3::new(0., 0.86, face),
            Vec3::new(1.39, 1.16, 0.095),
        );
        parcel(
            parent,
            art,
            Material::Wood,
            Vec3::new(0., 0.58, face + 0.23),
            1.0,
        );
        parcel(
            parent,
            art,
            Material::Wood,
            Vec3::new(-0.12, 0.84, face + 0.19),
            0.82,
        );
    } else {
        block(
            parent,
            art,
            &art.dark_wood,
            Vec3::new(0., 0.84, face),
            Vec3::new(0.69, 1.13, 0.14),
        );
        block(
            parent,
            art,
            &art.door,
            Vec3::new(0., 0.83, face + 0.076),
            Vec3::new(0.54, 1.04, 0.063),
        );
        mound(
            parent,
            art,
            &art.amber,
            Vec3::new(0.18, 0.79, face + 0.115),
            Vec3::splat(0.072),
        );
        for x in [-0.74, 0.74] {
            block(
                parent,
                art,
                &art.timber,
                Vec3::new(x, 1.19, face),
                Vec3::new(0.58, 0.58, 0.12),
            );
            block(
                parent,
                art,
                &art.amber,
                Vec3::new(x, 1.19, face + 0.067),
                Vec3::new(0.43, 0.40, 0.05),
            );
            block(
                parent,
                art,
                &art.timber,
                Vec3::new(x, 1.19, face + 0.101),
                Vec3::new(0.044, 0.43, 0.04),
            );
        }
    }
    if kind != Building::Store {
        block(
            parent,
            art,
            &art.stone,
            Vec3::new(width * 0.26, h + roof_height * 0.50, -0.34),
            Vec3::new(0.29, 0.85, 0.39),
        );
        block(
            parent,
            art,
            &art.warm_stone,
            Vec3::new(width * 0.26, h + roof_height * 0.50 + 0.45, -0.34),
            Vec3::new(0.37, 0.13, 0.46),
        );
    }
}

fn civic_building(parent: &mut ChildSpawnerCommands, art: &Art, kind_biome: Biome, kind: Building) {
    let roof = &art.roof_color[kind_biome as usize];
    match kind {
        Building::Well => {
            for (height, width, thickness, mat) in [
                (0.12, 1.40, 0.24, &art.warm_stone),
                (0.38, 1.19, 0.42, &art.stone),
                (0.61, 1.40, 0.14, &art.warm_stone),
                (0.687, 0.94, 0.014, &art.ripple),
            ] {
                part(
                    parent,
                    &art.cylinder,
                    mat,
                    Vec3::Y * height,
                    Vec3::new(width, thickness, width),
                    Quat::IDENTITY,
                );
            }
            for x in [-0.54, 0.54] {
                block(
                    parent,
                    art,
                    &art.timber,
                    Vec3::new(x, 0.97, -0.12),
                    Vec3::new(0.14, 1.79, 0.15),
                );
            }
            part(
                parent,
                &art.roof,
                roof,
                Vec3::new(0., 1.66, -0.09),
                Vec3::new(1.50, 0.49, 1.23),
                Quat::IDENTITY,
            );
            block(
                parent,
                art,
                &art.dark_wood,
                Vec3::new(0.10, 1.10, -0.04),
                Vec3::new(0.045, 0.80, 0.045),
            );
            part(
                parent,
                &art.cylinder,
                &art.timber,
                Vec3::new(0.10, 0.75, -0.04),
                Vec3::splat(0.25),
                Quat::IDENTITY,
            );
        }
        Building::Market => {
            block(
                parent,
                art,
                &art.warm_stone,
                Vec3::Y * 0.10,
                Vec3::new(2.99, 0.20, 2.64),
            );
            for x in [-1.26, 1.26] {
                for z in [-0.84, 0.72] {
                    block(
                        parent,
                        art,
                        &art.timber,
                        Vec3::new(x, 0.94, z),
                        Vec3::new(0.14, 1.88, 0.14),
                    );
                }
            }
            for n in 0..7 {
                part(
                    parent,
                    &art.cube,
                    if n % 2 == 0 { roof } else { &art.plaster },
                    Vec3::new(n as f32 * 0.42 - 1.26, 1.88, -0.04),
                    Vec3::new(0.43, 0.09, 2.15),
                    Quat::from_rotation_x(0.16),
                );
            }
            block(
                parent,
                art,
                &art.timber,
                Vec3::new(0., 0.52, 0.47),
                Vec3::new(2.34, 0.70, 0.66),
            );
            for (i, material) in Material::ALL.into_iter().enumerate() {
                parcel(
                    parent,
                    art,
                    material,
                    Vec3::new(-0.76 + i as f32 * 0.73, 0.91, 0.46),
                    0.9,
                );
            }
        }
        Building::Tower => {
            for (height, width, thickness, mat) in [
                (0.15, 1.45, 0.30, &art.warm_stone),
                (0.95, 0.96, 1.65, &art.stone),
                (1.88, 1.47, 0.24, &art.timber),
                (2.38, 1.22, 0.75, &art.plaster),
            ] {
                block(
                    parent,
                    art,
                    mat,
                    Vec3::Y * height,
                    Vec3::new(width, thickness, width),
                );
            }
            part(
                parent,
                &art.roof,
                roof,
                Vec3::Y * 2.78,
                Vec3::new(1.56, 0.63, 1.49),
                Quat::IDENTITY,
            );
            for x in [-0.59, 0.59] {
                block(
                    parent,
                    art,
                    &art.dark_wood,
                    Vec3::new(x, 2.38, 0.60),
                    Vec3::new(0.10, 0.76, 0.10),
                );
            }
            block(
                parent,
                art,
                &art.amber,
                Vec3::new(0., 2.43, 0.62),
                Vec3::new(0.29, 0.35, 0.07),
            );
        }
        _ => unreachable!(),
    }
}

pub fn upgrade_detail(parent: &mut ChildSpawnerCommands, art: &Art, kind: Building) {
    if kind == Building::Field {
        block(
            parent,
            art,
            &art.ripple,
            Vec3::new(0., 0.12, -1.28),
            Vec3::new(2.70, 0.14, 0.21),
        );
        for x in [-1.4, 1.4] {
            for z in [-1.2, 1.2] {
                block(
                    parent,
                    art,
                    &art.timber,
                    Vec3::new(x, 0.36, z),
                    Vec3::new(0.11, 0.68, 0.11),
                );
            }
        }
    } else {
        let size = f32::from(kind.size());
        let offset = size * 0.55;
        block(
            parent,
            art,
            &art.timber,
            Vec3::new(offset, 0.72, offset),
            Vec3::new(0.09, 1.44, 0.09),
        );
        block(
            parent,
            art,
            &art.amber,
            Vec3::new(offset - 0.19, 1.19, offset),
            Vec3::new(0.42, 0.49, 0.067),
        );
        mound(
            parent,
            art,
            &art.gold,
            Vec3::new(offset, 1.49, offset),
            Vec3::splat(0.12),
        );
    }
}

pub fn paved_roads(city: &Settlement, region: &Region) -> Mesh {
    let mut ground = Geometry::default();
    let dirt = rgb(0xbba480);
    let pavers = [rgb(0xd9c5a3), rgb(0xdccaa7), rgb(0xc7b395), rgb(0xc9b69c)];
    for &tile in &city.roads {
        let p = tile.world() + Vec3::Y * 0.031;
        ground.flat_box(p, 0.49, 0.49, dirt);
        for next in tile
            .neighbors()
            .into_iter()
            .filter(|&next| city.road_at(region, next))
        {
            let delta = (next.world() - tile.world()).normalize();
            ground.flat_box(
                p + delta * 0.56,
                0.39 - delta.x.abs() * 0.08,
                0.39 - delta.z.abs() * 0.08,
                dirt,
            );
        }
        for (i, (x, z)) in [(-0.24, -0.23), (0.25, -0.24), (-0.22, 0.23), (0.22, 0.24)]
            .into_iter()
            .enumerate()
        {
            let seed = hash(tile.0, tile.1, i as u32);
            ground.flat_box(
                p + Vec3::new(x, 0.014, z),
                0.19 + (seed % 4) as f32 * 0.013,
                0.17,
                pavers[(seed % 4) as usize],
            );
        }
    }
    ground.mesh()
}

pub fn bridge(
    parent: &mut ChildSpawnerCommands,
    art: &Art,
    region: &Region,
    column: i16,
    fixed: bool,
) {
    let start = Tile(column, region.river_at(column)).world();
    if region.kind == Biome::Aegean {
        let steps = region.river_span(column) * 3 + 2;
        for i in 0..steps {
            if !fixed && (3..steps - 3).contains(&i) {
                continue;
            }
            let z = start.z - CELL / 2. + f32::from(i) * CELL / 3.;
            let at = Vec3::new(start.x, 0.085, z);
            block(
                parent,
                art,
                if i % 3 == 0 {
                    &art.white
                } else {
                    &art.limestone
                },
                at,
                Vec3::new(1.59, 0.17, 0.54),
            );
            for x in [-0.75, 0.75] {
                if i % 4 == 0 {
                    block(
                        parent,
                        art,
                        &art.limestone,
                        at + Vec3::new(x, 0.16, 0.),
                        Vec3::new(0.19, 0.62, 0.27),
                    );
                    block(
                        parent,
                        art,
                        &art.cobalt,
                        at + Vec3::new(x, 0.43, 0.),
                        Vec3::new(0.23, 0.09, 0.30),
                    );
                }
                if fixed {
                    block(
                        parent,
                        art,
                        &art.limestone,
                        at + Vec3::new(x, 0.31, 0.),
                        Vec3::new(0.07, 0.14, 0.56),
                    );
                }
            }
        }
        return;
    }
    let boards = region.river_span(column) * 6 + 3;
    for index in 0..boards {
        if !fixed && (5..boards - 5).contains(&index) {
            continue;
        }
        let z = start.z - CELL * 0.5 + f32::from(index) * CELL / 6.;
        let pos = Vec3::new(start.x, 0.09, z);
        block(
            parent,
            art,
            if index % 3 == 0 {
                &art.plank
            } else {
                &art.timber
            },
            pos,
            Vec3::new(1.57, 0.15, 0.25),
        );
        if index % 6 == 0 {
            for side in [-1., 1.] {
                block(
                    parent,
                    art,
                    &art.dark_wood,
                    pos + Vec3::new(side * 0.79, 0.29, 0.),
                    Vec3::new(0.14, 0.76, 0.15),
                );
            }
        }
        if fixed {
            for side in [-1., 1.] {
                block(
                    parent,
                    art,
                    &art.plank,
                    pos + Vec3::new(side * 0.79, 0.47, 0.),
                    Vec3::new(0.075, 0.075, 0.29),
                );
            }
        }
    }
}

pub fn influence_mesh(city: &Settlement, region: &Region) -> Mesh {
    let mut mesh = Geometry::default();
    for tile in region.bounds.tiles() {
        if !city.influence(region, tile) {
            continue;
        }
        for next in tile.neighbors() {
            if city.influence(region, next) {
                continue;
            }
            let direction = (next.world() - tile.world()).normalize_or_zero();
            let base = tile.world() + direction * CELL * 0.5 + Vec3::Y * 0.054;
            let axis = Vec3::new(direction.z, 0., direction.x);
            for offset in [-0.47, 0., 0.47] {
                mesh.flat_box(
                    base + axis * offset,
                    if direction.x != 0. { 0.046 } else { 0.15 },
                    if direction.z != 0. { 0.046 } else { 0.15 },
                    Color::WHITE,
                );
            }
        }
    }
    mesh.mesh()
}
