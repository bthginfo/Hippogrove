use bevy::prelude::*;

use crate::region::Biome;

use super::{Art, block, mound, part};

fn rod(
    parent: &mut ChildSpawnerCommands,
    art: &Art,
    material: &Handle<StandardMaterial>,
    from: Vec3,
    to: Vec3,
    width: f32,
) {
    let axis = to - from;
    part(
        parent,
        &art.cylinder,
        material,
        (from + to) * 0.5,
        Vec3::new(width, axis.length(), width),
        Quat::from_rotation_arc(Vec3::Y, axis.normalize()),
    );
}

pub fn faction_color(art: &Art, biome: Biome) -> &Handle<StandardMaterial> {
    match biome {
        Biome::Valley => &art.leaf[2],
        Biome::Fjord => &art.blue,
        Biome::Terraces => &art.cobalt,
        Biome::Lake => &art.amber,
        Biome::Aegean => &art.plaster,
    }
}

pub fn leader(parent: &mut ChildSpawnerCommands, art: &Art, biome: Biome) {
    let color = faction_color(art, biome);
    for side in [-1., 1.] {
        mound(
            parent,
            art,
            color,
            Vec3::new(side * 0.20, 0.80, -0.12),
            Vec3::new(0.37, 0.31, 0.46),
        );
        mound(
            parent,
            art,
            &art.boot,
            Vec3::new(side * 0.14, 0.17, 0.035),
            Vec3::new(0.23, 0.38, 0.28),
        );
    }
    block(
        parent,
        art,
        color,
        Vec3::new(0., 0.58, 0.185),
        Vec3::new(0.43, 0.46, 0.12),
    );
    block(
        parent,
        art,
        &art.gold,
        Vec3::new(0., 0.70, 0.26),
        Vec3::new(0.11, 0.12, 0.033),
    );
    match biome {
        Biome::Valley => {
            for side in [-1., 1.] {
                let horn = Vec3::new(side * 0.20, 1.31, 0.06);
                let tip = horn + Vec3::new(side * 0.19, 0.32, -0.081);
                rod(parent, art, &art.timber, horn, tip, 0.09);
                rod(
                    parent,
                    art,
                    &art.timber,
                    horn.lerp(tip, 0.55),
                    tip + Vec3::new(-side * 0.12, 0.07, 0.10),
                    0.064,
                );
                mound(parent, art, &art.leaf[1], horn, Vec3::new(0.34, 0.17, 0.31));
            }
        }
        Biome::Fjord => {
            mound(
                parent,
                art,
                &art.white,
                Vec3::new(0., 0.81, -0.10),
                Vec3::new(0.79, 0.26, 0.48),
            );
            mound(
                parent,
                art,
                &art.ridge,
                Vec3::Y * 1.34,
                Vec3::new(0.65, 0.33, 0.58),
            );
            part(
                parent,
                &art.cylinder,
                &art.gold,
                Vec3::new(-0.49, 0.59, 0.28),
                Vec3::new(0.54, 0.10, 0.54),
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            );
            mound(
                parent,
                art,
                &art.blue,
                Vec3::new(-0.49, 0.59, 0.36),
                Vec3::new(0.39, 0.39, 0.11),
            );
        }
        Biome::Terraces => {
            part(
                parent,
                &art.cone,
                &art.cobalt,
                Vec3::Y * 1.45,
                Vec3::new(0.83, 0.35, 0.76),
                Quat::IDENTITY,
            );
            mound(parent, art, &art.gold, Vec3::Y * 1.64, Vec3::splat(0.12));
            for i in 0..5 {
                let tilt = (i as f32 - 2.) * 0.24;
                part(
                    parent,
                    &art.lump,
                    &art.plaster,
                    Vec3::new(0.47 + tilt * 0.25, 0.89, 0.23),
                    Vec3::new(0.15, 0.29, 0.055),
                    Quat::from_rotation_z(tilt),
                );
            }
        }
        Biome::Lake => {
            mound(
                parent,
                art,
                &art.clay,
                Vec3::Y * 1.33,
                Vec3::new(0.72, 0.26, 0.64),
            );
            for i in 0..6 {
                mound(
                    parent,
                    art,
                    &art.gold,
                    Vec3::new((i as f32 - 2.5) * 0.081, 0.83, 0.24),
                    Vec3::splat(0.091),
                );
            }
            rod(
                parent,
                art,
                &art.dark_wood,
                Vec3::new(0.48, 0., 0.21),
                Vec3::new(0.48, 1.42, 0.21),
                0.077,
            );
            mound(
                parent,
                art,
                &art.cobalt,
                Vec3::new(0.48, 1.47, 0.21),
                Vec3::splat(0.23),
            );
        }
        Biome::Aegean => {
            for i in 0..6 {
                let angle = (i as f32 - 2.5) * 0.32;
                mound(
                    parent,
                    art,
                    &art.gold,
                    Vec3::new(angle.sin() * 0.34, 1.32, 0.12 + 0.23 * angle.cos()),
                    Vec3::new(0.13, 0.11, 0.10),
                );
            }
            part(
                parent,
                &art.cylinder,
                &art.cobalt,
                Vec3::new(-0.48, 0.57, 0.28),
                Vec3::new(0.60, 0.08, 0.60),
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            );
            mound(
                parent,
                art,
                &art.gold,
                Vec3::new(-0.48, 0.57, 0.32),
                Vec3::new(0.31, 0.31, 0.09),
            );
        }
    }
}
