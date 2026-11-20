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

fn eyes(parent: &mut ChildSpawnerCommands, art: &Art, center: Vec3, spread: f32) {
    for side in [-1., 1.] {
        let point = center + Vec3::X * side * spread;
        mound(parent, art, &art.gold, point, Vec3::splat(0.115));
        mound(
            parent,
            art,
            &art.eye,
            point + Vec3::new(side * 0.012, 0.004, 0.046),
            Vec3::splat(0.057),
        );
    }
}

fn deer(parent: &mut ChildSpawnerCommands, art: &Art) {
    let hide = &art.timber;
    mound(
        parent,
        art,
        hide,
        Vec3::new(0., 0.87, 0.),
        Vec3::new(0.78, 0.74, 1.62),
    );
    mound(
        parent,
        art,
        &art.leaf[2],
        Vec3::new(0., 1.15, -0.14),
        Vec3::new(0.74, 0.32, 1.12),
    );
    rod(
        parent,
        art,
        hide,
        Vec3::new(0., 0.85, 0.45),
        Vec3::new(0., 1.52, 0.65),
        0.38,
    );
    mound(
        parent,
        art,
        hide,
        Vec3::new(0., 1.54, 0.78),
        Vec3::new(0.47, 0.41, 0.66),
    );
    mound(
        parent,
        art,
        &art.warm_stone,
        Vec3::new(0., 1.44, 1.02),
        Vec3::new(0.32, 0.25, 0.39),
    );
    eyes(parent, art, Vec3::new(0., 1.58, 0.95), 0.182);
    for side in [-1., 1.] {
        for z in [-0.50, 0.46] {
            let thigh = Vec3::new(side * 0.25, 0.70, z);
            let foot = Vec3::new(side * 0.27, 0.13, z + 0.073);
            rod(parent, art, hide, thigh, foot, 0.15);
            block(
                parent,
                art,
                &art.dark_wood,
                foot,
                Vec3::new(0.17, 0.16, 0.25),
            );
        }
        let base = Vec3::new(side * 0.12, 1.65, 0.7);
        let fork = Vec3::new(side * 0.37, 1.93, 0.57);
        rod(parent, art, &art.plank, base, fork, 0.11);
        rod(
            parent,
            art,
            &art.plank,
            fork,
            Vec3::new(side * 0.59, 2.15, 0.51),
            0.077,
        );
        rod(
            parent,
            art,
            &art.plank,
            fork,
            Vec3::new(side * 0.43, 2.26, 0.74),
            0.066,
        );
        mound(parent, art, &art.leaf[1], fork, Vec3::new(0.21, 0.15, 0.20));
        mound(
            parent,
            art,
            &art.berry,
            fork + Vec3::new(side * 0.08, 0.04, 0.08),
            Vec3::splat(0.076),
        );
    }
}

fn wolf(parent: &mut ChildSpawnerCommands, art: &Art) {
    mound(
        parent,
        art,
        &art.ridge,
        Vec3::new(0., 0.67, -0.05),
        Vec3::new(0.87, 0.73, 1.89),
    );
    mound(
        parent,
        art,
        &art.stone,
        Vec3::new(0., 0.91, 0.53),
        Vec3::new(0.83, 0.80, 0.99),
    );
    mound(
        parent,
        art,
        &art.ridge,
        Vec3::new(0., 1.10, 0.83),
        Vec3::new(0.61, 0.49, 0.61),
    );
    mound(
        parent,
        art,
        &art.warm_stone,
        Vec3::new(0., 1.00, 1.15),
        Vec3::new(0.44, 0.29, 0.65),
    );
    mound(
        parent,
        art,
        &art.dark_wood,
        Vec3::new(0., 1.05, 1.43),
        Vec3::new(0.21, 0.16, 0.17),
    );
    eyes(parent, art, Vec3::new(0., 1.16, 1.04), 0.234);
    for side in [-1., 1.] {
        for z in [-0.60, 0.54] {
            mound(
                parent,
                art,
                &art.ridge,
                Vec3::new(side * 0.30, 0.31, z),
                Vec3::new(0.27, 0.59, 0.32),
            );
            mound(
                parent,
                art,
                &art.warm_stone,
                Vec3::new(side * 0.30, 0.095, z + 0.09),
                Vec3::new(0.26, 0.15, 0.36),
            );
        }
        part(
            parent,
            &art.cone,
            &art.ridge,
            Vec3::new(side * 0.24, 1.42, 0.78),
            Vec3::new(0.24, 0.38, 0.23),
            Quat::from_rotation_z(-side * 0.19),
        );
        block(
            parent,
            art,
            &art.blue,
            Vec3::new(side * 0.36, 0.94, 0.24),
            Vec3::new(0.044, 0.17, 0.14),
        );
    }
    rod(
        parent,
        art,
        &art.stone,
        Vec3::new(0., 0.66, -0.83),
        Vec3::new(0.26, 0.97, -1.47),
        0.24,
    );
}

fn dragon(parent: &mut ChildSpawnerCommands, art: &Art) {
    for i in 0..11 {
        let t = i as f32 / 10.;
        let path = Vec3::new(
            (t * 6.5).sin() * (1. - t) * 0.83,
            0.34 + t * 0.90,
            t * 2.48 - 1.30,
        );
        let width = 0.25 + t * 0.37;
        mound(
            parent,
            art,
            &art.cobalt,
            path,
            Vec3::new(width, width, 0.49 + t * 0.14),
        );
        mound(
            parent,
            art,
            &art.plaster,
            path - Vec3::Y * width * 0.31,
            Vec3::new(width * 0.75, width * 0.63, 0.40),
        );
        if i > 2 && i < 10 {
            part(
                parent,
                &art.cone,
                &art.gold,
                path + Vec3::Y * width * 0.48,
                Vec3::new(0.13, 0.20, 0.13),
                Quat::IDENTITY,
            );
        }
    }
    mound(
        parent,
        art,
        &art.cobalt,
        Vec3::new(0., 1.27, 1.26),
        Vec3::new(0.75, 0.57, 0.81),
    );
    mound(
        parent,
        art,
        &art.leaf[1],
        Vec3::new(0., 1.14, 1.58),
        Vec3::new(0.62, 0.27, 0.60),
    );
    eyes(parent, art, Vec3::new(0., 1.39, 1.51), 0.279);
    for side in [-1., 1.] {
        let horn = Vec3::new(side * 0.24, 1.57, 1.14);
        rod(
            parent,
            art,
            &art.gold,
            horn,
            horn + Vec3::new(side * 0.13, 0.25, -0.21),
            0.11,
        );
        let whisker = Vec3::new(side * 0.28, 1.19, 1.70);
        let elbow = Vec3::new(side * 0.67, 1.31, 1.45);
        rod(parent, art, &art.plaster, whisker, elbow, 0.064);
        rod(
            parent,
            art,
            &art.plaster,
            elbow,
            Vec3::new(side * 0.81, 1.48, 1.08),
            0.043,
        );
        rod(
            parent,
            art,
            &art.cobalt,
            Vec3::new(side * 0.21, 0.98, 0.65),
            Vec3::new(side * 0.53, 0.68, 0.85),
            0.15,
        );
    }
}

fn serpent(parent: &mut ChildSpawnerCommands, art: &Art) {
    for i in 0..11 {
        let t = i as f32 / 10.;
        let path = Vec3::new(
            (t * 7.8).cos() * (1. - t) * 0.72,
            0.33 + 0.43 * t * t,
            t * 2.64 - 1.31,
        );
        let width = 0.17 + 0.53 * t;
        mound(
            parent,
            art,
            if i % 3 == 0 { &art.blue } else { &art.cobalt },
            path,
            Vec3::new(width, width * 0.91, 0.56),
        );
        if i > 2 && i < 9 {
            part(
                parent,
                &art.cone,
                &art.leaf[2],
                path + Vec3::Y * (0.16 + t * 0.23),
                Vec3::new(0.18, 0.40, 0.23),
                Quat::from_rotation_x(0.26),
            );
        }
    }
    mound(
        parent,
        art,
        &art.cobalt,
        Vec3::new(0., 0.83, 1.45),
        Vec3::new(0.90, 0.44, 0.87),
    );
    mound(
        parent,
        art,
        &art.blue,
        Vec3::new(0., 0.71, 1.77),
        Vec3::new(0.70, 0.26, 0.82),
    );
    eyes(parent, art, Vec3::new(0., 1.01, 1.66), 0.30);
    for side in [-1., 1.] {
        part(
            parent,
            &art.cone,
            &art.leaf[1],
            Vec3::new(side * 0.43, 0.78, 1.23),
            Vec3::new(0.25, 0.56, 0.36),
            Quat::from_rotation_z(-side * 1.02),
        );
    }
}

fn griffin(parent: &mut ChildSpawnerCommands, art: &Art) {
    mound(
        parent,
        art,
        &art.amber,
        Vec3::new(0., 0.61, -0.18),
        Vec3::new(0.87, 0.70, 1.55),
    );
    mound(
        parent,
        art,
        &art.warm_stone,
        Vec3::new(0., 0.86, 0.48),
        Vec3::new(0.83, 0.89, 0.89),
    );
    mound(
        parent,
        art,
        &art.white,
        Vec3::new(0., 1.25, 0.71),
        Vec3::new(0.65, 0.55, 0.72),
    );
    part(
        parent,
        &art.cone,
        &art.gold,
        Vec3::new(0., 1.14, 1.13),
        Vec3::new(0.37, 0.58, 0.41),
        Quat::from_rotation_x(1.94),
    );
    eyes(parent, art, Vec3::new(0., 1.32, 0.96), 0.240);
    for side in [-1., 1.] {
        for z in [-0.55, 0.51] {
            mound(
                parent,
                art,
                &art.amber,
                Vec3::new(side * 0.31, 0.30, z),
                Vec3::new(0.27, 0.57, 0.35),
            );
            mound(
                parent,
                art,
                &art.gold,
                Vec3::new(side * 0.31, 0.10, z + 0.10),
                Vec3::new(0.29, 0.15, 0.42),
            );
        }
        for i in 0..6 {
            let fraction = i as f32 / 5.;
            part(
                parent,
                &art.lump,
                if i < 3 { &art.plaster } else { &art.gold },
                Vec3::new(
                    side * (0.58 + 0.72 * fraction),
                    0.99 + 0.52 * fraction,
                    -0.18 - 0.45 * fraction,
                ),
                Vec3::new(0.72, 0.15, 0.48),
                Quat::from_rotation_z(side * 0.63),
            );
        }
        part(
            parent,
            &art.cone,
            &art.amber,
            Vec3::new(side * 0.22, 1.56, 0.60),
            Vec3::new(0.15, 0.36, 0.16),
            Quat::from_rotation_z(-side * 0.22),
        );
    }
    rod(
        parent,
        art,
        &art.amber,
        Vec3::new(0., 0.64, -0.87),
        Vec3::new(0.16, 1.04, -1.36),
        0.13,
    );
}

pub fn creature(parent: &mut ChildSpawnerCommands, art: &Art, biome: Biome) {
    match biome {
        Biome::Valley => deer(parent, art),
        Biome::Fjord => wolf(parent, art),
        Biome::Terraces => dragon(parent, art),
        Biome::Lake => serpent(parent, art),
        Biome::Aegean => griffin(parent, art),
    }
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
