use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::{
        mesh::PrimitiveTopology,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

use crate::{
    civic::Civic,
    game::{Building, Settlement},
    region::{Biome, CELL, Material, Region, Tile},
    terrain::{self, hash},
};

mod landscapes;
mod nature;
mod settlement_models;

pub use landscapes::{ForestPatch, LandscapeProp, forest_mesh, landscape};

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
    pub cloud_puff: Handle<Mesh>,
    pub roof: Handle<Mesh>,
    pub curved_roof: Handle<Mesh>,
    pub masonry: Handle<Mesh>,
    pub ring: Handle<Mesh>,
    pub plan_corners: Handle<Mesh>,
    pub plan_cross: Handle<Mesh>,
    pub leaf: [Handle<StandardMaterial>; 3],
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
    terrain_surface: Handle<StandardMaterial>,
    water: Handle<StandardMaterial>,
    foam: Handle<StandardMaterial>,
    pub waldwinkel: WoodlandKit,
    villages: [VillageArt; 5],
    plots: [Handle<Mesh>; 5],
    pub hippo_model: Handle<Scene>,
    pub deposits: [[Handle<Mesh>; 6]; 5],
}

impl Art {
    pub fn load(
        meshes: &mut Assets<Mesh>,
        mats: &mut Assets<StandardMaterial>,
        images: &mut Assets<Image>,
        assets: &AssetServer,
    ) -> Self {
        Self {
            cube: meshes.add(Cuboid::from_size(Vec3::ONE)),
            lump: meshes.add(Sphere::new(0.5).mesh().ico(1).unwrap()),
            cylinder: meshes.add(Cylinder::new(0.5, 1.).mesh().resolution(9)),
            cone: meshes.add(Cone::new(0.5, 1.).mesh().resolution(8)),
            cloud_puff: meshes.add(Sphere::new(0.5).mesh().ico(2).unwrap()),
            roof: meshes.add(roof_mesh(false)),
            curved_roof: meshes.add(roof_mesh(true)),
            masonry: meshes.add(masonry_mesh()),
            ring: meshes.add(ring_mesh()),
            plan_corners: meshes.add(plan_corners()),
            plan_cross: meshes.add(plan_cross()),
            leaf: [
                paint(mats, 0x9ca74c),
                paint(mats, 0xadb359),
                paint(mats, 0x728540),
            ],
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
            terrain_surface: mats.add(StandardMaterial {
                base_color_texture: Some(images.add(terrain_detail())),
                perceptual_roughness: 0.97,
                ..default()
            }),
            water: mats.add(StandardMaterial {
                perceptual_roughness: 0.44,
                reflectance: 0.29,
                ..default()
            }),
            foam: mats.add(StandardMaterial {
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                ..default()
            }),
            waldwinkel: WoodlandKit::load(assets),
            villages: ["valley", "fjord", "terraces", "lake", "aegean"].map(|biome| {
                let scene = |index| {
                    assets.load(format!(
                        "embedded://models/villages/{biome}.glb#Scene{index}"
                    ))
                };
                VillageArt {
                    house: scene(0),
                    store: scene(1),
                }
            }),
            plots: Biome::ALL.map(|biome| meshes.add(plot_mesh(biome))),
            hippo_model: assets.load("embedded://models/hippo.glb#Scene0"),
            deposits: [
                Biome::Valley,
                Biome::Fjord,
                Biome::Terraces,
                Biome::Lake,
                Biome::Aegean,
            ]
            .map(|biome| {
                std::array::from_fn(|index| {
                    let material = Material::ALL[index % Material::ALL.len()];
                    meshes.add(nature::deposit(
                        biome,
                        material,
                        index < Material::ALL.len(),
                    ))
                })
            }),
        }
    }

    pub fn village_loaded(&self, biome: Biome, assets: &AssetServer) -> bool {
        let village = &self.villages[biome as usize];
        [&self.hippo_model, &village.house, &village.store]
            .into_iter()
            .all(|scene| assets.is_loaded_with_dependencies(scene.id()))
    }
}

struct VillageArt {
    house: Handle<Scene>,
    store: Handle<Scene>,
}

fn terrain_detail() -> Image {
    const SIDE: usize = 64;
    let noise = |x: usize, z: usize, cells: i16| {
        let at = Vec2::new(x as f32, z as f32) * (f32::from(cells) / SIDE as f32);
        let tile = at.floor();
        let [x, z] = [tile.x as i16, tile.y as i16];
        let pigment = |u: i16, v: i16| {
            (hash(u.rem_euclid(cells), v.rem_euclid(cells), 391) & 255) as f32 / 255.
        };
        let blend = at - tile;
        let u = terrain::smooth(blend.x);
        let v = terrain::smooth(blend.y);
        pigment(x, z)
            .lerp(pigment(x + 1, z), u)
            .lerp(pigment(x, z + 1).lerp(pigment(x + 1, z + 1), u), v)
    };
    let mut level = (0..SIDE * SIDE)
        .map(|index| {
            let (x, z) = (index % SIDE, index / SIDE);
            (216. + 25. * noise(x, z, 5) + 14. * noise(x, z, 19)) as u8
        })
        .collect::<Vec<_>>();
    let mut image_bytes = Vec::new();
    let mut width = SIDE;
    loop {
        image_bytes.extend(level.iter().flat_map(|&gray| [gray, gray, gray, 255]));
        if width == 1 {
            break;
        }
        let next_width = width / 2;
        level = (0..next_width * next_width)
            .map(|index| {
                let first = 2 * (index % next_width + index / next_width * width);
                let sum = u16::from(level[first])
                    + u16::from(level[first + 1])
                    + u16::from(level[first + width])
                    + u16::from(level[first + width + 1]);
                (sum / 4) as u8
            })
            .collect();
        width = next_width;
    }
    let mut image = Image::new_uninit(
        Extent3d {
            width: SIDE as u32,
            height: SIDE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.mip_level_count = SIDE.ilog2() + 1;
    image.data = Some(image_bytes);
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    });
    image
}

pub struct WoodlandKit {
    conifer: Handle<Scene>,
    rock: Handle<Scene>,
    mushroom: Handle<Scene>,
    reed: Handle<Scene>,
    shell: Handle<Scene>,
    pub crop: Handle<Scene>,
    harbor_lantern: Handle<Scene>,
    watch_fern: Handle<Scene>,
}

impl WoodlandKit {
    fn load(assets: &AssetServer) -> Self {
        let scene = |stem| assets.load(format!("embedded://models/waldwinkel/{stem}.glb#Scene0"));
        Self {
            conifer: scene("wood_tree"),
            rock: scene("resource_rock"),
            mushroom: scene("find_mushroom"),
            reed: scene("find_marsh_herb"),
            shell: scene("find_shell"),
            crop: scene("crop_strawberry"),
            harbor_lantern: scene("furniture_harbor_lantern"),
            watch_fern: scene("watch_fern"),
        }
    }
}

pub fn place_scene(
    parent: &mut ChildSpawnerCommands,
    model: &Handle<Scene>,
    position: Vec3,
    scale: f32,
    yaw: f32,
) {
    parent.spawn((
        SceneRoot(model.clone()),
        Transform::from_translation(position)
            .with_scale(Vec3::splat(scale))
            .with_rotation(Quat::from_rotation_y(yaw)),
    ));
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

    fn ground_box(&mut self, region: &Region, p: Vec3, half_x: f32, half_z: f32, color: Color) {
        let [a, b, c, d] = [
            (-half_x, -half_z),
            (-half_x, half_z),
            (half_x, half_z),
            (half_x, -half_z),
        ]
        .map(|(x, z)| terrain::on_ground(region, p + Vec3::new(x, 0., z)) + Vec3::Y * p.y);
        self.quad(a, b, c, d, color);
    }

    fn flagstone(&mut self, region: &Region, center: Vec3, size: Vec2, key: u32, color: Color) {
        let bevel = 0.028 + (key & 7) as f32 * 0.003;
        let turn = Quat::from_rotation_y(((key >> 4) & 31) as f32 * 0.012 - 0.185);
        let border = [
            (-0.66, -1.),
            (-1., -0.62),
            (-1., 0.57),
            (-0.63, 1.),
            (0.63, 0.95),
            (1., 0.61),
            (0.96, -0.54),
            (0.67, -1.),
        ];
        let points = border.map(|(x, z)| turn * Vec3::new(x * size.x, 0., z * size.y));
        let foot = points.map(|p| terrain::on_ground(region, center + p) + Vec3::Y * 0.028);
        let top = points
            .map(|p| terrain::on_ground(region, center + p * 0.89) + Vec3::Y * (0.028 + bevel));
        let middle = terrain::on_ground(region, center) + Vec3::Y * (0.043 + bevel);
        for index in 0..top.len() {
            let next = (index + 1) % top.len();
            let rim = color.mix(&rgb(0x7c6552), 0.21 + (index % 3) as f32 * 0.065);
            self.quad(foot[index], foot[next], top[next], top[index], rim);
            self.tri(middle, top[index], top[next], color);
        }
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

    fn render_mesh(self) -> Mesh {
        let mut mesh = self.mesh();
        mesh.asset_usage = RenderAssetUsages::RENDER_WORLD;
        mesh
    }

    fn terrain_mesh(self) -> Mesh {
        let uv = self
            .positions
            .iter()
            .map(|p| [p[0] * 0.37, p[2] * 0.37])
            .collect::<Vec<_>>();
        self.render_mesh()
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
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
        let rows = ((right - left) * 10.).ceil() as usize;
        for row in 0..rows {
            let x0 = left.lerp(right, row as f32 / rows as f32 + 0.024);
            let x1 = left.lerp(right, (row as f32 + 1.) / rows as f32 - 0.018);
            let y0 = bottom.lerp(top, (x0 - left) / (right - left)) + 0.014;
            let y1 = bottom.lerp(top, (x1 - left) / (right - left)) + 0.025;
            let stagger = (row % 2) as f32 * 0.058;
            for column in 0..10 {
                let offset = -0.547 + column as f32 * 0.111 + stagger;
                let z0 = offset.max(-0.499);
                let z1 = (offset + 0.106).min(0.499);
                if z1 <= z0 {
                    continue;
                }
                let shade = rgb([0xd8b9a3, 0xfff5df, 0xeed5bf, 0xe9c6b0, 0xf0e1d3, 0xc8a590]
                    [(row * 13 + column * 7) % 6]);
                let ridge = (z0 + z1) * 0.5;
                let uplift = if curved { 0.012 } else { 0.025 };
                geometry.quad(
                    Vec3::new(x0, y0, z0),
                    Vec3::new(x0, y0 + uplift, ridge),
                    Vec3::new(x1, y1 + uplift, ridge),
                    Vec3::new(x1, y1, z0),
                    shade,
                );
                geometry.tri(
                    Vec3::new(x0, y0 - 0.021, z0),
                    Vec3::new(x0, y0, z0),
                    Vec3::new(x0, y0 + uplift, ridge),
                    shade.mix(&rgb(0x9a715f), 0.39),
                );
                geometry.quad(
                    Vec3::new(x0, y0 + uplift, ridge),
                    Vec3::new(x0, y0, z1),
                    Vec3::new(x1, y1, z1),
                    Vec3::new(x1, y1 + uplift, ridge),
                    shade,
                );
            }
        }
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

fn masonry_mesh() -> Mesh {
    let mut geometry = Geometry::default();
    let bricks = [0xfaf5e8, 0xe9ddca, 0xfff9e9, 0xf5ead7, 0xf2e6d5];
    for course in 0..5 {
        for index in 0..5 {
            let start = -0.619 + index as f32 * 0.25 + (course % 2) as f32 * 0.118;
            let x0 = start.max(-0.499);
            let x1 = (start + 0.242).min(0.499);
            let key = hash(index as i16, course as i16, 808);
            let y0 = -0.494 + course as f32 * 0.2 + (key & 3) as f32 * 0.0017;
            let y1 = y0 + 0.183;
            if x1 - x0 < 0.022 {
                continue;
            }
            let color = rgb(bricks[(course * 7 + index * 3) % bricks.len()]);
            let face = 0.505 + ((key >> 8) & 7) as f32 * 0.0019;
            let edge = color.mix(&rgb(0x96846c), 0.25);
            geometry.quad(
                Vec3::new(x0 + 0.008, y0 + 0.007, face),
                Vec3::new(x1 - 0.008, y0 + 0.007, face),
                Vec3::new(x1 - 0.005, y1, face),
                Vec3::new(x0 + 0.006, y1 - 0.002, face),
                color,
            );
            geometry.quad(
                Vec3::new(x0, y0, 0.50),
                Vec3::new(x1, y0, 0.50),
                Vec3::new(x1 - 0.008, y0 + 0.007, face),
                Vec3::new(x0 + 0.008, y0 + 0.007, face),
                edge,
            );
        }
    }
    geometry.mesh()
}

pub fn stonework(parent: &mut ChildSpawnerCommands, art: &Art, at: Vec3, size: Vec3) {
    for side in 0..4 {
        let angle = side as f32 * std::f32::consts::FRAC_PI_2;
        let size = if side % 2 == 0 { size } else { size.zyx() };
        part(
            parent,
            &art.masonry,
            &art.limestone,
            at,
            size + Vec3::Z * 0.012,
            Quat::from_rotation_y(angle),
        );
    }
}

fn plot_mesh(biome: Biome) -> Mesh {
    let soil = rgb(match biome {
        Biome::Valley => 0xa87952,
        Biome::Fjord => 0x846950,
        Biome::Terraces => 0x94674c,
        Biome::Lake => 0xb98152,
        Biome::Aegean => 0xaa7650,
    });
    let mut ground = Geometry::default();
    ground.flat_box(Vec3::Y * 0.139, 1.47, 1.34, soil.mix(&rgb(0x614b38), 0.24));
    let profile = [
        (-0.22, 0.144),
        (-0.14, 0.183),
        (0.039, 0.219),
        (0.20, 0.144),
    ];
    for (row, x) in [-0.96, -0.32, 0.32, 0.96].into_iter().enumerate() {
        let point = |edge: usize, segment: usize| {
            let (across, height) = profile[edge];
            let variation = (hash(row as i16, segment as i16, 241) % 13) as f32 / 12.;
            let z = -1.18 + segment as f32 * 0.393;
            Vec3::new(
                x + across + variation * 0.075,
                height + variation * 0.009,
                z,
            )
        };
        for segment in 0..6 {
            for edge in 0..profile.len() - 1 {
                let grain = (hash(row as i16, (segment * 4 + edge) as i16, 253) % 7) as f32 / 6.;
                let shade = soil.mix(&rgb(0x5f492f), (2 - edge) as f32 * 0.11 + grain * 0.15);
                ground.quad(
                    point(edge, segment),
                    point(edge, segment + 1),
                    point(edge + 1, segment + 1),
                    point(edge + 1, segment),
                    shade,
                );
            }
        }
    }
    ground.terrain_mesh()
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

pub fn surface_ring(region: &Region, center: Vec3, radius: f32, bridge_fixed: bool) -> Mesh {
    let mut geometry = Geometry::default();
    for index in 0..64 {
        let point = |i, width: f32| {
            let angle = i as f32 * std::f32::consts::TAU / 64.;
            let point = center + Vec3::new(angle.sin(), 0., angle.cos()) * width;
            point.with_y(terrain::actor_height(region, point, bridge_fixed) + 0.065)
        };
        let inner = radius - (radius * 0.09).clamp(0.08, 0.21);
        geometry.quad(
            point(index, inner),
            point(index, radius),
            point(index + 1, radius),
            point(index + 1, inner),
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
            place_scene(parent, &art.waldwinkel.rock, Vec3::ZERO, 0.78, 0.3);
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

pub fn camp(parent: &mut ChildSpawnerCommands, art: &Art, material: Material) -> Entity {
    block(
        parent,
        art,
        &art.warm_stone,
        Vec3::Y * 0.09,
        Vec3::new(1.45, 0.18, 1.30),
    );
    block(
        parent,
        art,
        &art.plank,
        Vec3::new(-0.22, 0.26, 0.18),
        Vec3::new(0.72, 0.29, 0.64),
    );
    parcel(parent, art, material, Vec3::new(0.27, 0.25, -0.22), 0.75);
    block(
        parent,
        art,
        &art.dark_wood,
        Vec3::new(0.38, 0.94, -0.29),
        Vec3::new(0.12, 1.78, 0.13),
    );
    block(
        parent,
        art,
        &art.amber,
        Vec3::new(0.38, 1.67, 0.),
        Vec3::splat(0.16),
    );
    block(
        parent,
        art,
        &art.blue,
        Vec3::new(0.05, 1.29, -0.27),
        Vec3::new(0.62, 0.49, 0.07),
    )
}

pub fn gate_leaf(parent: &mut ChildSpawnerCommands, art: &Art, sign: f32) {
    for i in 0..4 {
        let x = -sign * (0.09 + i as f32 * 0.17);
        block(
            parent,
            art,
            if i % 3 == 0 { &art.plank } else { &art.timber },
            Vec3::new(x, 0.63, 0.),
            Vec3::new(0.16, 1.10 + i as f32 * 0.035, 0.096),
        );
    }
    for y in [0.30, 0.91] {
        block(
            parent,
            art,
            &art.dark_wood,
            Vec3::new(-sign * 0.34, y, 0.059),
            Vec3::new(0.68, 0.075, 0.06),
        );
    }
    block(
        parent,
        art,
        &art.gold,
        Vec3::new(-sign * 0.56, 0.66, 0.079),
        Vec3::splat(0.095),
    );
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
    let village = &art.villages[biome as usize];
    let model = match kind {
        Building::House => Some(&village.house),
        Building::Store => Some(&village.store),
        _ => None,
    };
    if let Some(model) = model {
        place_scene(parent, model, Vec3::ZERO, 1., 0.);
        return;
    }
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
        part(
            parent,
            &art.plots[biome as usize],
            &art.terrain_surface,
            Vec3::ZERO,
            Vec3::ONE,
            Quat::IDENTITY,
        );
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
    let h = 1.59;
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
    let wall = if biome == Biome::Aegean {
        &art.limestone
    } else if biome == Biome::Fjord {
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
    if biome == Biome::Aegean {
        stonework(
            parent,
            art,
            Vec3::Y * (0.26 + h * 0.5),
            Vec3::new(width, h, length),
        );
    }
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
        if biome != Biome::Aegean {
            for z in [-length * 0.49, length * 0.49] {
                block(
                    parent,
                    art,
                    &art.dark_wood,
                    Vec3::new(x, h * 0.5 + 0.24, z),
                    Vec3::new(0.15, h, 0.15),
                );
            }
        }
        block(
            parent,
            art,
            if biome == Biome::Aegean {
                &art.warm_stone
            } else {
                &art.timber
            },
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
    if kind == Building::Granary {
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
            if biome == Biome::Aegean {
                &art.cobalt
            } else {
                &art.door
            },
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
            if biome == Biome::Aegean {
                shutter(parent, art, Vec3::new(x, 1.21, face + 0.03), 0.);
                continue;
            }
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
    if biome == Biome::Aegean && kind == Building::Workshop {
        shutter(
            parent,
            art,
            Vec3::new(width * 0.5 + 0.043, 1.22, -0.49),
            std::f32::consts::FRAC_PI_2,
        );
    }
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

fn shutter(parent: &mut ChildSpawnerCommands, art: &Art, position: Vec3, turn: f32) {
    let rotation = Quat::from_rotation_y(turn);
    let mut piece = |surface: &Handle<StandardMaterial>, at: [f32; 3], size: [f32; 3]| {
        part(
            parent,
            &art.cube,
            surface,
            position + rotation * Vec3::from(at),
            Vec3::from(size),
            rotation,
        );
    };
    piece(&art.warm_stone, [0., 0., 0.], [0.62, 0.56, 0.079]);
    piece(&art.dark_wood, [0., 0.025, 0.042], [0.34, 0.40, 0.034]);
    piece(&art.ridge, [0., 0.071, 0.059], [0.23, 0.27, 0.020]);
    piece(&art.limestone, [0., -0.26, 0.052], [0.65, 0.081, 0.173]);
    for side in [-1., 1.] {
        piece(
            &art.cobalt,
            [side * 0.243, 0.024, 0.075],
            [0.172, 0.397, 0.058],
        );
        piece(
            &art.blue,
            [side * 0.226, -0.086, 0.11],
            [0.16, 0.044, 0.025],
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
        ground.ground_box(region, p, 0.49, 0.49, dirt);
        for next in tile
            .neighbors()
            .into_iter()
            .filter(|&next| city.road_at(region, next))
        {
            let delta = (next.world() - tile.world()).normalize();
            ground.ground_box(
                region,
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
            ground.flagstone(
                region,
                p + Vec3::new(x, 0., z),
                Vec2::new(0.19 + (seed % 4) as f32 * 0.013, 0.19),
                seed,
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
    let deck = |z, depth: f32| {
        let local = Tile::coordinates(Vec3::new(start.x, 0., z));
        Vec3::new(
            start.x,
            terrain::bridge_height(region, column, local.y) - depth * 0.5,
            z,
        )
    };
    if region.kind == Biome::Aegean {
        let steps = region.river_span(column) * 3 + 2;
        for i in 0..steps {
            if !fixed && (3..steps - 3).contains(&i) {
                continue;
            }
            let z = start.z - CELL / 2. + f32::from(i) * CELL / 3.;
            let at = deck(z, 0.17);
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
        let pos = deck(z, 0.15);
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
                mesh.ground_box(
                    region,
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
