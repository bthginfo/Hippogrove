use bevy::prelude::*;

use super::{Geometry, rgb};
use crate::region::Biome;

pub(crate) struct People {
    residents: [[Costume; 3]; 5],
    watch: [Costume; 5],
    baskets: [Handle<Mesh>; 5],
    material: Handle<StandardMaterial>,
}

pub(crate) struct Costume {
    body: Handle<Mesh>,
    arm: Handle<Mesh>,
    leg: Handle<Mesh>,
}

impl People {
    pub fn load(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            residents: Biome::ALL.map(|biome| std::array::from_fn(|i| costume(meshes, biome, i))),
            watch: Biome::ALL.map(|biome| costume(meshes, biome, 3)),
            baskets: Biome::ALL.map(|biome| meshes.add(basket_mesh(Dye::on(biome, 0)))),
            material: materials.add(StandardMaterial {
                perceptual_roughness: 0.92,
                ..default()
            }),
        }
    }

    pub fn get(&self, biome: Biome, variant: usize, guard: bool) -> &Costume {
        let index = biome as usize;
        if guard {
            &self.watch[index]
        } else {
            &self.residents[index][variant % 3]
        }
    }

    pub fn limb_origin(side: f32, arm: bool) -> Vec3 {
        if arm {
            Vec3::new(side * 0.252, 0.815, 0.015)
        } else {
            Vec3::new(side * 0.113, 0.416, -0.01)
        }
    }

    fn part(
        &self,
        parent: &mut ChildSpawnerCommands,
        mesh: &Handle<Mesh>,
        transform: Transform,
    ) -> Entity {
        parent
            .spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(self.material.clone()),
                transform,
            ))
            .id()
    }

    pub fn body(&self, parent: &mut ChildSpawnerCommands, costume: &Costume) {
        let body = self.part(parent, &costume.body, Transform::default());
        parent
            .commands()
            .entity(body)
            .insert(Name::new("Gläubiger"));
    }

    pub fn limb(&self, parent: &mut ChildSpawnerCommands, costume: &Costume, arm: bool) {
        let mesh = if arm { &costume.arm } else { &costume.leg };
        self.part(parent, mesh, Transform::default());
    }

    pub fn basket(&self, parent: &mut ChildSpawnerCommands, biome: Biome) -> Entity {
        let basket = self.part(parent, &self.baskets[biome as usize], Transform::default());
        parent
            .commands()
            .entity(basket)
            .insert(Name::new("Sammelkorb"));
        basket
    }

    pub fn standing(
        &self,
        parent: &mut ChildSpawnerCommands,
        biome: Biome,
        guard: bool,
        variant: usize,
    ) {
        let person = self.get(biome, variant, guard);
        self.body(parent, person);
        for side in [-1., 1.] {
            for (arm, mesh) in [(false, &person.leg), (true, &person.arm)] {
                self.part(
                    parent,
                    mesh,
                    Transform::from_translation(Self::limb_origin(side, arm)),
                );
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Dye {
    cloth: Color,
    shaded: Color,
    edge: Color,
    metal: Color,
    binding: Color,
    hair: Color,
    skin: Color,
    boot: Color,
}

impl Dye {
    fn on(biome: Biome, outfit: usize) -> Self {
        let (clothes, lining, trim, metal, leather, skin) = match biome {
            Biome::Valley => (
                [0x71805b, 0x8c6655, 0xa28453, 0x546e54],
                0x405648,
                0xe5c79c,
                0xba8b56,
                0x5f473b,
                [0xc88e67, 0xe0af80, 0x9c684f, 0xd49b73],
            ),
            Biome::Fjord => (
                [0x5f7e89, 0x887467, 0x655a7b, 0x486b80],
                0x3b4d63,
                0xe2d4bd,
                0xbf9767,
                0x50473e,
                [0xdbab86, 0xb67c58, 0xe7bc9c, 0xc5926e],
            ),
            Biome::Terraces => (
                [0x577c6b, 0x9f604b, 0x657a8b, 0x6a493d],
                0x3f5750,
                0xddc48f,
                0xbca66f,
                0x534236,
                [0xba855a, 0xd1a378, 0x9e6d53, 0xb88964],
            ),
            Biome::Lake => (
                [0xd3b486, 0xa96a48, 0x79948a, 0xba9466],
                0x875b4d,
                0x477d85,
                0xd4a763,
                0x674735,
                [0x9b6550, 0xb8805e, 0x7f5142, 0xa56b51],
            ),
            Biome::Aegean => (
                [0xe0c49d, 0xa76148, 0x628790, 0xb5865c],
                0x76634d,
                0x3f6876,
                0xbd9657,
                0x654639,
                [0xc08a60, 0xd6a781, 0xa67155, 0xc18c74],
            ),
        };
        let hair = match (biome, outfit) {
            (Biome::Fjord | Biome::Valley, 1) => 0x946445,
            (Biome::Fjord, 2) => 0x6d5544,
            (Biome::Lake, _) => 0x332e2d,
            (_, 2) => 0x4e3d36,
            _ => 0x40332d,
        };
        Self {
            cloth: rgb(clothes[outfit]),
            shaded: rgb(lining),
            edge: rgb(trim),
            metal: rgb(metal),
            binding: rgb(leather),
            hair: rgb(hair),
            skin: rgb(skin[outfit]),
            boot: rgb(leather).mix(&rgb(0x302a27), 0.33),
        }
    }
}

const BORDER: [(f32, f32); 8] = [
    (-0.68, 1.),
    (0.68, 1.),
    (1., 0.60),
    (1., -0.60),
    (0.68, -1.),
    (-0.68, -1.),
    (-1., -0.60),
    (-1., 0.60),
];

#[derive(Clone, Copy)]
struct Ring {
    at: Vec3,
    half: Vec2,
}

fn at(x: f32, y: f32, z: f32, width: f32, depth: f32) -> Ring {
    Ring {
        at: Vec3::new(x, y, z),
        half: Vec2::new(width, depth),
    }
}

fn bands(mesh: &mut Geometry, rings: &[Ring], cloth: Color, shadow: Color) {
    let rim =
        |ring: Ring| BORDER.map(|(x, z)| ring.at + Vec3::new(x * ring.half.x, 0., z * ring.half.y));
    for pair in rings.windows(2) {
        let [low, high] = [rim(pair[0]), rim(pair[1])];
        for (side, mix) in [0.02, 0.16, 0.26, 0.38, 0.40, 0.27, 0.19, 0.08]
            .into_iter()
            .enumerate()
        {
            let next = (side + 1) % 8;
            mesh.quad(
                low[side],
                low[next],
                high[next],
                high[side],
                cloth.mix(&shadow, mix),
            );
        }
    }
    for (ring, outward) in [(rings[0], false), (*rings.last().unwrap(), true)] {
        let vertices = rim(ring);
        for (i, &p) in vertices.iter().enumerate() {
            let next = vertices[(i + 1) % 8];
            if outward {
                mesh.tri(ring.at, p, next, cloth);
            } else {
                mesh.tri(ring.at, next, p, shadow);
            }
        }
    }
}

fn patch(mesh: &mut Geometry, center: Vec3, half: Vec2, shade: Color) {
    let x = Vec3::X * half.x;
    let y = Vec3::Y * half.y;
    mesh.quad(
        center - x - y,
        center + x - y,
        center + x + y,
        center - x + y,
        shade,
    );
}

fn stitch(mesh: &mut Geometry, first: Vec3, last: Vec3, thickness: f32, shade: Color) {
    let raised = Vec3::Z.cross((last - first).normalize_or_zero()) * thickness;
    mesh.quad(
        first - raised,
        last - raised,
        last + raised,
        first + raised,
        shade,
    );
}

fn emblem(mesh: &mut Geometry, center: Vec3, horizontal: f32, vertical: f32, shade: Color) {
    let x = Vec3::X * horizontal;
    let y = Vec3::Y * vertical;
    mesh.quad(center - x, center - y, center + x, center + y, shade);
}

fn face(mesh: &mut Geometry, dye: Dye, outfit: usize) {
    let jaw = if outfit == 2 { 0.127 } else { 0.143 };
    let head = [
        at(0., 0.952, 0.025, 0.068, 0.069),
        at(0., 0.995, 0.057, jaw, 0.110),
        at(0., 1.083, 0.036, 0.174, 0.156),
        at(0., 1.205, 0.008, 0.172, 0.145),
        at(0., 1.271, -0.008, 0.127, 0.115),
    ];
    bands(mesh, &head, dye.skin, dye.binding);
    let front = Vec3::new(0., 1.077, 0.182);
    mesh.tri(
        front + Vec3::new(-0.032, -0.015, 0.003),
        front + Vec3::new(0.022, -0.019, 0.003),
        front + Vec3::new(0.004, -0.015, 0.052),
        dye.skin.mix(&dye.binding, 0.22),
    );
    mesh.tri(
        front + Vec3::new(0.022, -0.019, 0.003),
        front + Vec3::new(0., 0.061, 0.003),
        front + Vec3::new(0.004, -0.015, 0.052),
        dye.skin,
    );
    mesh.tri(
        front + Vec3::new(0., 0.061, 0.003),
        front + Vec3::new(-0.032, -0.015, 0.003),
        front + Vec3::new(0.004, -0.015, 0.052),
        dye.skin.mix(&dye.binding, 0.08),
    );
    for side in [-1., 1.] {
        patch(
            mesh,
            Vec3::new(side * 0.076, 1.124, 0.188),
            Vec2::new(0.027, 0.012),
            dye.hair,
        );
        stitch(
            mesh,
            Vec3::new(side * 0.045, 1.149, 0.186),
            Vec3::new(side * 0.103, 1.143, 0.177),
            0.008,
            dye.hair,
        );
        bands(
            mesh,
            &[
                at(side * 0.171, 1.065, 0.013, 0.022, 0.043),
                at(side * 0.176, 1.125, 0.016, 0.025, 0.035),
                at(side * 0.163, 1.144, 0.006, 0.014, 0.025),
            ],
            dye.skin,
            dye.binding,
        );
    }
    if outfit == 1 {
        patch(
            mesh,
            Vec3::new(0., 1.023, 0.164),
            Vec2::new(0.054, 0.019),
            dye.hair,
        );
        patch(
            mesh,
            Vec3::new(-0.024, 1.035, 0.174),
            Vec2::new(0.033, 0.009),
            dye.hair,
        );
    } else {
        patch(
            mesh,
            Vec3::new(0., 1.031, 0.183),
            Vec2::new(0.034, 0.007),
            dye.skin.mix(&dye.binding, 0.56),
        );
    }
    bands(
        mesh,
        &[
            at(0., 1.156, -0.048, 0.169, 0.094),
            at(0., 1.243, -0.022, 0.179, 0.157),
            at(0., 1.306, -0.025, 0.136, 0.136),
            at(0., 1.340, -0.060, 0.063, 0.081),
        ],
        dye.hair,
        dye.boot,
    );
}

fn headwear(mesh: &mut Geometry, biome: Biome, dye: Dye, outfit: usize) {
    let guard = outfit == 3;
    match biome {
        Biome::Valley if outfit != 2 => {
            bands(
                mesh,
                &[
                    at(0., 1.235, 0., 0.209, 0.187),
                    at(-0.021, 1.278, 0.031, 0.212, 0.203),
                    at(-0.021, 1.371, -0.036, 0.156, 0.149),
                    at(-0.071, 1.416, -0.06, 0.053, 0.068),
                ],
                dye.cloth,
                dye.shaded,
            );
            patch(
                mesh,
                Vec3::new(-0.043, 1.267, 0.229),
                Vec2::new(0.137, 0.021),
                dye.edge,
            );
        }
        Biome::Fjord => {
            bands(
                mesh,
                &[
                    at(0., 1.190, 0.003, 0.209, 0.181),
                    at(0., 1.257, 0.01, 0.213, 0.192),
                ],
                dye.edge,
                dye.binding,
            );
            bands(
                mesh,
                &[
                    at(0., 1.250, -0.012, 0.191, 0.180),
                    at(-0.026, 1.351, -0.036, 0.149, 0.135),
                    at(-0.046, 1.420, -0.047, 0.065, 0.077),
                ],
                if guard { dye.metal } else { dye.cloth },
                dye.shaded,
            );
        }
        Biome::Terraces if outfit != 2 => {
            bands(
                mesh,
                &[
                    at(0., 1.255, 0.018, 0.304, 0.259),
                    at(0., 1.278, 0.018, 0.306, 0.264),
                    at(-0.017, 1.405, -0.024, 0.072, 0.063),
                    at(-0.017, 1.433, -0.024, 0.026, 0.023),
                ],
                dye.metal.mix(&dye.edge, 0.32),
                dye.shaded,
            );
            patch(
                mesh,
                Vec3::new(0.002, 1.270, 0.283),
                Vec2::new(0.183, 0.016),
                dye.binding,
            );
        }
        Biome::Lake => {
            bands(
                mesh,
                &[
                    at(0., 1.232, 0., 0.178, 0.164),
                    at(-0.017, 1.281, 0.013, 0.218, 0.178),
                    at(0.027, 1.354, -0.022, 0.174, 0.135),
                    at(0.032, 1.403, -0.064, 0.091, 0.077),
                ],
                if outfit == 1 { dye.edge } else { rgb(0xd9bd93) },
                dye.binding,
            );
            stitch(
                mesh,
                Vec3::new(-0.137, 1.253, 0.174),
                Vec3::new(0.115, 1.332, 0.147),
                0.023,
                dye.edge,
            );
            emblem(
                mesh,
                Vec3::new(0.018, 1.303, 0.196),
                0.034,
                0.034,
                dye.metal,
            );
        }
        Biome::Aegean if guard => {
            bands(
                mesh,
                &[
                    at(0., 1.218, 0.008, 0.196, 0.184),
                    at(0., 1.369, -0.011, 0.165, 0.165),
                    at(0., 1.421, -0.06, 0.052, 0.088),
                ],
                dye.metal,
                dye.binding,
            );
            for side in [-1., 1.] {
                bands(
                    mesh,
                    &[
                        at(side * 0.164, 1.065, 0.089, 0.023, 0.053),
                        at(side * 0.166, 1.240, 0.048, 0.037, 0.103),
                    ],
                    dye.metal,
                    dye.shaded,
                );
            }
            bands(
                mesh,
                &[
                    at(0., 1.384, -0.047, 0.037, 0.142),
                    at(0., 1.527, -0.085, 0.024, 0.084),
                ],
                rgb(0x975547),
                dye.binding,
            );
        }
        _ => {
            bands(
                mesh,
                &[
                    at(0., 1.208, 0.001, 0.181, 0.165),
                    at(0., 1.24, 0.008, 0.184, 0.159),
                ],
                dye.edge,
                dye.shaded,
            );
            bands(
                mesh,
                &[
                    at(0.034, 1.260, -0.109, 0.086, 0.089),
                    at(0.063, 1.342, -0.128, 0.105, 0.079),
                    at(0.050, 1.388, -0.111, 0.045, 0.049),
                ],
                dye.hair,
                dye.binding,
            );
        }
    }
}

fn clothing(mesh: &mut Geometry, biome: Biome, dye: Dye, outfit: usize) {
    let long = matches!(biome, Biome::Terraces | Biome::Lake);
    let hem = if long { 0.321 } else { 0.374 };
    bands(
        mesh,
        &[
            at(0., hem, -0.009, 0.237, 0.159),
            at(0., 0.529, 0.0, 0.195, 0.146),
            at(0., 0.728, -0.015, 0.223, 0.168),
            at(0., 0.853, -0.020, 0.257, 0.163),
            at(0., 0.948, -0.004, 0.090, 0.111),
        ],
        dye.cloth,
        dye.shaded,
    );
    bands(
        mesh,
        &[
            at(0., hem + 0.012, -0.008, 0.240, 0.163),
            at(0., hem + 0.057, -0.006, 0.232, 0.162),
        ],
        dye.edge,
        dye.shaded,
    );
    bands(
        mesh,
        &[
            at(0., 0.506, 0.007, 0.200, 0.151),
            at(0., 0.570, 0.002, 0.206, 0.157),
        ],
        dye.binding,
        dye.shaded,
    );
    emblem(
        mesh,
        Vec3::new(0.043, 0.537, 0.163),
        0.041,
        0.031,
        dye.metal,
    );

    if outfit == 1 {
        patch(
            mesh,
            Vec3::new(0., hem + 0.101, 0.158),
            Vec2::new(0.110, 0.076),
            dye.shaded,
        );
        for x in [-0.091, 0.091] {
            stitch(
                mesh,
                Vec3::new(x, hem + 0.040, 0.162),
                Vec3::new(x, hem + 0.166, 0.162),
                0.006,
                dye.edge,
            );
        }
        patch(
            mesh,
            Vec3::new(0.042, hem + 0.099, 0.166),
            Vec2::new(0.035, 0.022),
            dye.binding,
        );
    } else if outfit == 2 {
        bands(
            mesh,
            &[
                at(0.181, 0.470, 0.068, 0.062, 0.047),
                at(0.201, 0.534, 0.069, 0.081, 0.058),
                at(0.198, 0.560, 0.057, 0.064, 0.052),
            ],
            dye.binding,
            dye.shaded,
        );
        emblem(
            mesh,
            Vec3::new(0.205, 0.539, 0.126),
            0.027,
            0.020,
            dye.metal,
        );
    } else if outfit == 3 {
        for x in [-0.12, -0.04, 0.04, 0.12] {
            patch(
                mesh,
                Vec3::new(x, hem + 0.105, 0.163),
                Vec2::new(0.026, 0.058),
                dye.metal.mix(&dye.shaded, 0.42),
            );
        }
    }
    match biome {
        Biome::Valley => {
            for (x, color) in [
                (-0.114, dye.shaded),
                (-0.052, dye.edge),
                (0.116, dye.shaded),
            ] {
                stitch(
                    mesh,
                    Vec3::new(x, hem + 0.072, 0.165),
                    Vec3::new(x * 0.70, 0.496, 0.154),
                    0.021,
                    color,
                );
            }
            stitch(
                mesh,
                Vec3::new(-0.166, 0.883, 0.136),
                Vec3::new(0.121, 0.587, 0.162),
                0.046,
                dye.shaded,
            );
            for (x, y) in [(0.056, 0.717), (0.092, 0.663)] {
                emblem(mesh, Vec3::new(x, y, 0.17), 0.022, 0.030, dye.edge);
            }
        }
        Biome::Fjord => {
            bands(
                mesh,
                &[
                    at(0., 0.771, -0.015, 0.270, 0.177),
                    at(-0.006, 0.861, -0.018, 0.284, 0.175),
                    at(0., 0.950, -0.013, 0.095, 0.100),
                ],
                dye.edge,
                dye.binding,
            );
            for x in [-0.151, -0.031, 0.096] {
                mesh.tri(
                    Vec3::new(x, 0.779, 0.159),
                    Vec3::new(x + 0.081, 0.779, 0.161),
                    Vec3::new(x + 0.042, 0.722, 0.159),
                    dye.edge,
                );
            }
            emblem(
                mesh,
                Vec3::new(0.097, 0.749, 0.184),
                0.041,
                0.045,
                dye.metal,
            );
        }
        Biome::Terraces => {
            stitch(
                mesh,
                Vec3::new(-0.155, 0.891, 0.158),
                Vec3::new(0.098, 0.603, 0.163),
                0.032,
                dye.edge,
            );
            stitch(
                mesh,
                Vec3::new(0.139, 0.887, 0.159),
                Vec3::new(0.002, 0.736, 0.167),
                0.021,
                dye.edge,
            );
            for x in [-0.113, 0., 0.113] {
                emblem(
                    mesh,
                    Vec3::new(x, hem + 0.044, 0.164),
                    0.025,
                    0.017,
                    dye.shaded,
                );
            }
        }
        Biome::Lake => {
            bands(
                mesh,
                &[
                    at(0., 0.770, -0.003, 0.197, 0.170),
                    at(0., 0.824, -0.002, 0.224, 0.175),
                    at(0., 0.921, -0.003, 0.109, 0.125),
                ],
                dye.edge,
                dye.shaded,
            );
            for x in [-0.094, -0.047, 0., 0.047, 0.094] {
                emblem(
                    mesh,
                    Vec3::new(x, 0.802 + x.abs() * 0.23, 0.178),
                    0.017,
                    0.017,
                    dye.metal,
                );
            }
            for x in [-0.128, -0.014, 0.094] {
                stitch(
                    mesh,
                    Vec3::new(x - 0.02, hem + 0.062, 0.166),
                    Vec3::new(x + 0.03, 0.498, 0.155),
                    0.017,
                    dye.cloth.mix(&dye.shaded, 0.37),
                );
            }
        }
        Biome::Aegean => {
            stitch(
                mesh,
                Vec3::new(-0.174, 0.855, 0.156),
                Vec3::new(0.155, 0.594, 0.166),
                0.041,
                dye.edge,
            );
            emblem(
                mesh,
                Vec3::new(-0.167, 0.847, 0.174),
                0.052,
                0.052,
                dye.metal,
            );
            for x in [-0.136, -0.060, 0.016, 0.091] {
                let corner = Vec3::new(x + 0.032, hem + 0.044, 0.167);
                stitch(mesh, corner - Vec3::X * 0.032, corner, 0.005, dye.cloth);
                stitch(mesh, corner, corner - Vec3::Y * 0.022, 0.005, dye.cloth);
            }
        }
    }
}

fn shield(mesh: &mut Geometry, biome: Biome, dye: Dye) {
    // Shields share the walking root. Their faceted fronts stay legible against the tunic.
    let center = Vec3::new(-0.340, 0.559, 0.188);
    let (width, height, sides) = match biome {
        Biome::Fjord => (0.281, 0.281, 12),
        Biome::Valley => (0.201, 0.294, 8),
        Biome::Terraces => (0.180, 0.289, 6),
        Biome::Lake => (0.160, 0.321, 10),
        Biome::Aegean => (0.251, 0.277, 10),
    };
    for i in 0..sides {
        let p = |j: usize, radius: f32, depth: f32| {
            let angle = j as f32 * std::f32::consts::TAU / sides as f32;
            center
                + Vec3::new(
                    angle.cos() * width * radius,
                    angle.sin() * height * radius,
                    depth,
                )
        };
        let outer = p(i, 1., 0.);
        let next = p(i + 1, 1., 0.);
        let inner = p(i, 0.87, 0.039);
        let inner_next = p(i + 1, 0.87, 0.039);
        mesh.quad(outer, next, inner_next, inner, dye.metal);
        mesh.tri(
            inner,
            inner_next,
            center + Vec3::Z * 0.066,
            if i % 3 == 0 {
                dye.edge
            } else {
                dye.shaded.mix(&dye.cloth, 0.40)
            },
        );
        mesh.tri(next, outer, center - Vec3::Z * 0.029, dye.binding);
    }
    emblem(mesh, center + Vec3::Z * 0.071, 0.074, 0.086, dye.metal);
    emblem(mesh, center + Vec3::Z * 0.074, 0.041, 0.039, dye.binding);
}

fn costume(meshes: &mut Assets<Mesh>, biome: Biome, outfit: usize) -> Costume {
    let dye = Dye::on(biome, outfit);
    let mut body = Geometry::default();
    clothing(&mut body, biome, dye, outfit);
    face(&mut body, dye, outfit);
    headwear(&mut body, biome, dye, outfit);
    if outfit == 3 {
        shield(&mut body, biome, dye);
    }

    let mut arm = Geometry::default();
    bands(
        &mut arm,
        &[
            at(0., -0.188, 0.028, 0.058, 0.051),
            at(0., -0.080, 0.015, 0.076, 0.075),
            at(0., 0.033, 0., 0.087, 0.087),
            at(0., 0.060, -0.017, 0.054, 0.066),
        ],
        dye.cloth,
        dye.shaded,
    );
    bands(
        &mut arm,
        &[
            at(0., -0.214, 0.033, 0.058, 0.052),
            at(0., -0.172, 0.026, 0.061, 0.055),
        ],
        dye.edge,
        dye.binding,
    );
    bands(
        &mut arm,
        &[
            at(0., -0.342, 0.054, 0.041, 0.040),
            at(0., -0.292, 0.043, 0.058, 0.052),
            at(0., -0.207, 0.029, 0.045, 0.044),
        ],
        dye.skin,
        dye.binding,
    );

    let mut leg = Geometry::default();
    let sandals = matches!(biome, Biome::Lake | Biome::Aegean) && outfit < 3;
    bands(
        &mut leg,
        &[
            at(0., -0.355, 0.007, 0.055, 0.065),
            at(0., -0.187, -0.012, 0.072, 0.066),
            at(0., -0.033, -0.01, 0.084, 0.078),
        ],
        if sandals {
            dye.skin
        } else {
            dye.cloth.mix(&dye.binding, 0.52)
        },
        dye.shaded,
    );
    if sandals {
        bands(
            &mut leg,
            &[
                at(0., -0.408, 0.045, 0.076, 0.121),
                at(0., -0.380, 0.040, 0.078, 0.122),
                at(0., -0.365, 0.036, 0.072, 0.115),
            ],
            dye.boot,
            dye.binding,
        );
        bands(
            &mut leg,
            &[
                at(0., -0.369, 0.028, 0.060, 0.103),
                at(0., -0.341, 0.030, 0.048, 0.049),
            ],
            dye.skin,
            dye.binding,
        );
        for y in [-0.340, -0.301] {
            bands(
                &mut leg,
                &[
                    at(0., y, 0.007, 0.064, 0.069),
                    at(0., y + 0.016, 0.007, 0.064, 0.069),
                ],
                dye.binding,
                dye.shaded,
            );
        }
    } else {
        bands(
            &mut leg,
            &[
                at(0., -0.408, 0.045, 0.079, 0.127),
                at(0., -0.348, 0.042, 0.076, 0.125),
                at(0., -0.304, 0.003, 0.063, 0.085),
                at(0., -0.220, -0.007, 0.069, 0.065),
            ],
            dye.boot,
            dye.binding,
        );
        patch(
            &mut leg,
            Vec3::new(0., -0.277, 0.071),
            Vec2::new(0.039, 0.024),
            dye.edge,
        );
    }

    Costume {
        body: meshes.add(body.render_mesh()),
        arm: meshes.add(arm.render_mesh()),
        leg: meshes.add(leg.render_mesh()),
    }
}

fn basket_mesh(dye: Dye) -> Mesh {
    let mut basket = Geometry::default();
    let wicker = dye.binding.mix(&rgb(0xb99260), 0.61);
    bands(
        &mut basket,
        &[
            at(0., 0.482, -0.223, 0.118, 0.104),
            at(0., 0.785, -0.225, 0.165, 0.127),
            at(0., 0.823, -0.226, 0.158, 0.127),
        ],
        wicker,
        dye.binding,
    );
    for y in [0.534, 0.610, 0.686, 0.772] {
        let width = 0.115 + (y - 0.48) * 0.16;
        bands(
            &mut basket,
            &[
                at(0., y, -0.225, width + 0.006, 0.127),
                at(0., y + 0.018, -0.225, width + 0.008, 0.129),
            ],
            dye.edge.mix(&wicker, 0.52),
            dye.binding,
        );
    }
    basket.render_mesh()
}
