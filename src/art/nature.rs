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
    let trunk = |geo: &mut Geometry| tier(geo, at, scale, [0., 1.38, 0.15, 0.084], turn, bark);
    trunk(geo);
    match biome {
        Biome::Valley | Biome::Fjord if biome == Biome::Fjord || seed % 5 < 4 => {
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
                [(0.79, 0.68, 1.04), (1.34, 0.57, 0.99), (1.90, 0.44, 1.08)]
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
            tier(geo, at, scale, [0.34, 1.27, 0.24, 0.38], turn, green);
            tier(geo, at, scale, [1.27, 3.15, 0.38, 0.019], turn, green);
        }
        _ => {
            let (shade, light, spread) = match biome {
                Biome::Aegean => (0x577847, 0xa3ae60, 0.85),
                Biome::Lake => (0x5a8148, 0xa4a65d, 1.03),
                Biome::Terraces if seed % 6 < 2 => (0xad4937, 0xd97941, 0.74),
                Biome::Terraces => (0x507650, 0x8fa45b, 0.75),
                _ => (0x618c47, 0xafb85c, 0.76),
            };
            for (index, (x, y, z, width)) in [
                (0., 1.48, 0., 0.78),
                (-0.43, 1.27, 0.12, 0.67),
                (0.46, 1.34, -0.14, 0.64),
            ]
            .into_iter()
            .enumerate()
            {
                let offset = Quat::from_rotation_y(turn) * Vec3::new(x, y, z) * scale;
                let color = rgb(shade).mix(&rgb(light), 0.22 + grain * 0.40 + index as f32 * 0.13);
                stone(
                    geo,
                    at + offset,
                    Vec3::new(width * spread, 0.59, width * spread) * scale,
                    color,
                    seed + index as u32,
                );
            }
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
            tree(&mut geo, Vec3::ZERO, 1.12, biome, 27);
            tree(&mut geo, Vec3::new(-0.48, 0., 0.40), 0.42, biome, 68);
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
