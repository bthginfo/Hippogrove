use bevy::prelude::*;

use super::{Geometry, rgb};
use crate::region::{Biome, Material};

pub(super) fn tree(geo: &mut Geometry, at: Vec3, scale: f32, biome: Biome, seed: u32) {
    let turn = (seed % 61) as f32 * 0.104;
    let grain = (seed % 29) as f32 / 28.;
    let bark = rgb(if biome == Biome::Fjord {
        0x6e5748
    } else {
        0x8e694b
    });
    match biome {
        Biome::Valley | Biome::Fjord if biome == Biome::Fjord || seed % 5 < 4 => {
            tier(geo, at, scale, [0., 1.38, 0.15, 0.084], turn, bark);
            let shade = if biome == Biome::Fjord {
                0x536e62
            } else {
                0x497440
            };
            let light = if biome == Biome::Fjord {
                0x80967f
            } else {
                0x95a858
            };
            for (index, (rise, width, height)) in
                [(0.69, 0.79, 1.13), (1.30, 0.62, 1.08), (1.92, 0.48, 1.22)]
                    .into_iter()
                    .enumerate()
            {
                let color = rgb(shade).mix(&rgb(light), grain * 0.51 + index as f32 * 0.12);
                tier(
                    geo,
                    at,
                    scale,
                    [rise, rise + height, width, 0.025],
                    turn + index as f32 * 0.4,
                    color,
                );
                if biome == Biome::Fjord {
                    let snow = rgb(0xdde8e9).mix(&rgb(0xa8bec5), grain * 0.35);
                    tier(
                        geo,
                        at,
                        scale,
                        [
                            rise + height * 0.47,
                            rise + height * 1.03,
                            width * 0.56,
                            0.014,
                        ],
                        turn + index as f32 * 0.4,
                        snow,
                    );
                }
            }
        }
        Biome::Aegean if seed % 5 == 0 => {
            let green = rgb(0x41694e).mix(&rgb(0x72935e), grain * 0.74);
            tier(geo, at, scale, [0., 0.89, 0.13, 0.07], turn, bark);
            tier(geo, at, scale, [0.27, 1.05, 0.23, 0.34], turn, green);
            tier(geo, at, scale, [1.05, 2.04, 0.34, 0.25], turn, green);
            tier(geo, at, scale, [2.04, 3.19, 0.25, 0.018], turn, green);
        }
        Biome::Terraces if seed % 7 > 2 => bamboo(geo, at, scale, turn),
        _ => {
            let (shade, light, spread, crown_height) = match biome {
                Biome::Aegean => (0x547650, 0xc0c688, 0.88, 0.53),
                Biome::Lake => (0x5d7946, 0xc0bb67, 1.16, 0.35),
                Biome::Terraces if seed % 6 < 2 => (0x99483c, 0xda8c56, 0.91, 0.55),
                Biome::Terraces => (0x45754f, 0x91b27b, 0.85, 0.58),
                _ => (0x527b43, 0xb2be68, 0.88, 0.68),
            };
            let to_world = |point| at + Quat::from_rotation_y(turn) * point * scale;
            let fork = to_world(Vec3::new(-0.12, 0.82, 0.04));
            bough(geo, at, fork, 0.21 * scale, 0.12 * scale, bark);
            for (index, (x, y, z, width)) in [
                (-0.56, 1.43, 0.17, 0.67),
                (0.13, 1.68, -0.20, 0.78),
                (0.61, 1.56, 0.18, 0.66),
            ]
            .into_iter()
            .enumerate()
            {
                let crown = to_world(Vec3::new(x * spread, y + grain * 0.20, z));
                bough(geo, fork, crown, 0.117 * scale, 0.043 * scale, bark);
                let color = rgb(shade).mix(&rgb(light), 0.14 + grain * 0.35 + index as f32 * 0.17);
                stone(
                    geo,
                    crown,
                    Vec3::new(width * spread, crown_height, width * spread) * scale,
                    color,
                    seed + index as u32,
                );
            }
        }
    }
}

fn bough(geo: &mut Geometry, root: Vec3, tip: Vec3, base: f32, end: f32, bark: Color) {
    let axis = (tip - root).normalize();
    let tangent = axis.cross(Vec3::Z).normalize();
    let across = axis.cross(tangent);
    let edge = |index: u8| {
        let angle = f32::from(index) * std::f32::consts::TAU / 5.;
        tangent * angle.cos() + across * angle.sin()
    };
    for face in 0..5 {
        let a = edge(face);
        let b = edge(face + 1);
        let shade = bark.mix(&rgb(0x4d433a), f32::from(face % 3) * 0.15);
        geo.quad(
            root + base * a,
            root + base * b,
            tip + end * b,
            tip + end * a,
            shade,
        );
    }
}

fn bamboo(geo: &mut Geometry, at: Vec3, scale: f32, turn: f32) {
    let rotation = Quat::from_rotation_y(turn);
    let world = |local| at + rotation * local * scale;
    for (index, (x, z, height)) in [(-0.19, 0.12, 2.40), (0.11, -0.17, 2.70), (0.28, 0.19, 2.13)]
        .into_iter()
        .enumerate()
    {
        let root = world(Vec3::new(x, 0., z));
        let tip = world(Vec3::new(x * 1.92, height, z * 1.25));
        bough(geo, root, tip, 0.067 * scale, 0.026 * scale, rgb(0x9b9e59));
        for row in 0..3 {
            let start = root.lerp(tip, 0.51 + row as f32 * 0.19);
            let reach = if (index + row) % 2 == 0 { 1. } else { -1. };
            let far = start + rotation * Vec3::new(reach * 0.57, 0.12, 0.31) * scale;
            let middle = start.lerp(far, 0.43) + Vec3::Y * 0.125 * scale;
            let base = start.lerp(far, 0.38) - Vec3::Y * 0.064 * scale;
            geo.tri(start, middle, far, rgb(0x819b57));
            geo.tri(start, far, middle, rgb(0x819b57));
            geo.tri(start, far, base, rgb(0x486c44));
            geo.tri(start, base, far, rgb(0x486c44));
        }
    }
}

pub(super) fn stone(geo: &mut Geometry, at: Vec3, size: Vec3, color: Color, seed: u32) {
    let turn = (seed % 29) as f32 * 0.22;
    let height = 0.70 + (seed % 23) as f32 * 0.012;
    let shape = |j: usize, radius: f32, y: f32, lift: f32| {
        let angle = turn + j as f32 * std::f32::consts::TAU / 6.;
        let rough = 0.81 + ((seed as usize + j * 81) % 19) as f32 * 0.017;
        at + Vec3::new(
            angle.sin() * radius * rough + lift,
            y * height,
            angle.cos() * radius * rough,
        ) * size
    };
    for i in 0..6 {
        let j = (i + 1) % 6;
        let a = shape(i, 0.77, -0.19, 0.);
        let b = shape(j, 0.77, -0.19, 0.);
        let c = shape(i, 1.0, 0.32, 0.);
        let d = shape(j, 1.0, 0.32, 0.);
        let e = shape(i, 0.58, 0.89, 0.08);
        let f = shape(j, 0.58, 0.89, 0.08);
        let dim = color.mix(&rgb(0x70666c), (i % 3) as f32 * 0.08);
        geo.quad(a, b, d, c, dim);
        geo.tri(c, d, f, color);
        geo.tri(c, f, e, dim);
        geo.tri(
            e,
            f,
            at + Vec3::new(0.04, height * 0.94, 0.) * size,
            color.mix(&rgb(0xf5e4c8), 0.14),
        );
    }
}

fn tier(geo: &mut Geometry, at: Vec3, scale: f32, profile: [f32; 4], turn: f32, color: Color) {
    let [y0, y1, r0, r1] = profile;
    let p = |i, r, y| {
        let angle = turn + i as f32 * std::f32::consts::TAU / 7.;
        at + Vec3::new(angle.sin() * r, y, angle.cos() * r) * scale
    };
    for i in 0..7 {
        let shade = color.mix(&rgb(0x3c5947), (i % 3) as f32 * 0.09);
        geo.quad(
            p(i, r0, y0),
            p(i + 1, r0, y0),
            p(i + 1, r1, y1),
            p(i, r1, y1),
            shade,
        );
        if r1 > 0.042 {
            geo.tri(
                p(i, r1, y1),
                p(i + 1, r1, y1),
                at + Vec3::Y * y1 * scale,
                color,
            );
        }
        geo.tri(
            p(i, r0, y0),
            at + Vec3::Y * y0 * scale,
            p(i + 1, r0, y0),
            shade,
        );
    }
}

pub(super) fn deposit(biome: Biome, material: Material, grown: bool) -> Mesh {
    let mut geo = Geometry::default();
    match material {
        Material::Wood if grown => {
            let [main, sapling] = if biome == Biome::Aegean {
                [25, 65]
            } else {
                [27, 68]
            };
            tree(&mut geo, Vec3::ZERO, 1.12, biome, main);
            tree(&mut geo, Vec3::new(-0.48, 0., 0.40), 0.42, biome, sapling);
        }
        Material::Wood => {
            tier(
                &mut geo,
                Vec3::ZERO,
                1.,
                [-0.03, 0.22, 0.29, 0.23],
                0.,
                rgb(0x956844),
            );
            tier(
                &mut geo,
                Vec3::ZERO,
                1.,
                [0.225, 0.239, 0.202, 0.181],
                0.,
                rgb(0xd6ab72),
            );
        }
        Material::Stone => {
            let shade = if biome == Biome::Aegean {
                0xc7b69e
            } else if biome == Biome::Lake {
                0xc89267
            } else {
                0x849393
            };
            let size = if grown { 0.91 } else { 0.32 };
            for (i, (x, z, width)) in [(0., 0., 0.74), (-0.48, 0.31, 0.44), (0.49, 0.19, 0.31)]
                .into_iter()
                .enumerate()
            {
                stone(
                    &mut geo,
                    Vec3::new(x, 0., z),
                    Vec3::new(width, size, width),
                    rgb(shade),
                    i as u32 * 13,
                );
            }
        }
        Material::Food if biome == Biome::Terraces => {
            for (i, (x, z)) in [
                (-0.42, 0.23),
                (-0.25, -0.23),
                (0.22, 0.19),
                (0.04, -0.15),
                (0.38, -0.12),
            ]
            .into_iter()
            .enumerate()
            {
                let height = if grown { 0.53 + i as f32 * 0.081 } else { 0.25 };
                let point = Vec3::new(x, 0., z);
                tier(
                    &mut geo,
                    point,
                    1.,
                    [0., height, 0.044, 0.023],
                    i as f32,
                    rgb(0x7e9c4d),
                );
                if grown {
                    stone(
                        &mut geo,
                        point + Vec3::Y * height,
                        Vec3::new(0.12, 0.22, 0.071),
                        rgb(0xd9bc6f),
                        i as u32 * 17,
                    );
                }
            }
        }
        Material::Food => {
            for (i, (x, y, z, size)) in [
                (0., 0.10, 0., 0.59),
                (-0.43, 0.04, 0.16, 0.38),
                (0.39, 0.07, 0.25, 0.35),
            ]
            .into_iter()
            .enumerate()
            {
                stone(
                    &mut geo,
                    Vec3::new(x, y, z),
                    Vec3::new(size, if grown { size } else { size * 0.57 }, size),
                    rgb(0x658a4b),
                    28 + i as u32 * 13,
                );
            }
            if grown {
                for (i, (x, y, z)) in [
                    (-0.30, 0.51, 0.32),
                    (-0.57, 0.28, 0.30),
                    (0.09, 0.64, 0.31),
                    (0.39, 0.37, 0.41),
                    (0.16, 0.42, 0.54),
                ]
                .into_iter()
                .enumerate()
                {
                    let berry = match biome {
                        Biome::Lake => 0xd79a46,
                        Biome::Aegean => 0x9baf57,
                        Biome::Fjord => 0x666eac,
                        _ => 0xcb5251,
                    };
                    stone(
                        &mut geo,
                        Vec3::new(x, y, z),
                        Vec3::splat(0.116),
                        rgb(berry),
                        i as u32,
                    );
                }
            }
        }
    }
    geo.mesh()
}
