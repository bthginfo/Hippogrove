use super::{Art, block, mound, parcel, part};
use crate::{
    civic::Civic,
    region::{Biome, Material},
};
use bevy::prelude::*;

type Surface = Handle<StandardMaterial>;

fn box_at(
    parent: &mut ChildSpawnerCommands,
    art: &Art,
    mat: &Surface,
    at: [f32; 3],
    size: [f32; 3],
) {
    block(parent, art, mat, Vec3::from(at), Vec3::from(size));
}

fn round(
    parent: &mut ChildSpawnerCommands,
    art: &Art,
    mat: &Surface,
    at: [f32; 3],
    diameter: f32,
    height: f32,
) {
    part(
        parent,
        &art.cylinder,
        mat,
        Vec3::from(at),
        Vec3::new(diameter, height, diameter),
        Quat::IDENTITY,
    );
}

fn wall(art: &Art, biome: Biome) -> &Surface {
    match biome {
        Biome::Fjord => &art.timber,
        Biome::Lake => &art.warm_stone,
        Biome::Aegean => &art.limestone,
        _ => &art.plaster,
    }
}

fn trim(art: &Art, biome: Biome) -> &Surface {
    match biome {
        Biome::Aegean => &art.cobalt,
        Biome::Terraces => &art.red,
        _ => &art.door,
    }
}

fn column(parent: &mut ChildSpawnerCommands, art: &Art, biome: Biome, x: f32, z: f32, h: f32) {
    if biome == Biome::Aegean {
        round(parent, art, &art.limestone, [x, h * 0.5 + 0.17, z], 0.21, h);
        for y in [0.16, h + 0.13] {
            box_at(parent, art, &art.white, [x, y, z], [0.29, 0.13, 0.29]);
        }
    } else {
        let mat = if biome == Biome::Lake {
            &art.warm_stone
        } else {
            &art.timber
        };
        box_at(parent, art, mat, [x, h * 0.5 + 0.14, z], [0.16, h, 0.16]);
        box_at(
            parent,
            art,
            trim(art, biome),
            [x, h + 0.09, z],
            [0.23, 0.16, 0.23],
        );
    }
}

fn roof(
    parent: &mut ChildSpawnerCommands,
    art: &Art,
    biome: Biome,
    at: [f32; 3],
    mut size: [f32; 3],
    sod: bool,
) {
    let mesh = if biome == Biome::Terraces {
        &art.curved_roof
    } else {
        &art.roof
    };
    if biome == Biome::Fjord {
        size[1] *= 1.24;
    }
    let mat = if sod && biome == Biome::Fjord {
        &art.leaf[2]
    } else {
        &art.roof_color[biome as usize]
    };
    part(
        parent,
        mesh,
        mat,
        Vec3::from(at),
        Vec3::from(size),
        Quat::IDENTITY,
    );
    let [x, y, z] = at;
    let [width, rise, length] = size;
    box_at(
        parent,
        art,
        &art.ridge,
        [x, y + rise * 0.83, z],
        [0.14, 0.13, length + 0.09],
    );
    if biome == Biome::Fjord {
        // Crossed end timbers give the longhouse its silhouette.
        for side in [-1., 1.] {
            part(
                parent,
                &art.cube,
                &art.plank,
                Vec3::new(x, y + rise * 0.73, z + side * length * 0.47),
                Vec3::new(0.1, 0.70, 0.13),
                Quat::from_rotation_z(side * 0.69),
            );
            part(
                parent,
                &art.cube,
                &art.plank,
                Vec3::new(x, y + rise * 0.73, z + side * length * 0.47),
                Vec3::new(0.1, 0.70, 0.13),
                Quat::from_rotation_z(-side * 0.69),
            );
        }
    } else if biome == Biome::Aegean {
        for side in [-1., 1.] {
            box_at(
                parent,
                art,
                &art.limestone,
                [x + side * width * 0.45, y - 0.01, z],
                [0.19, 0.13, length],
            );
        }
    }
}

fn lamp(parent: &mut ChildSpawnerCommands, art: &Art, x: f32, y: f32, z: f32) {
    box_at(
        parent,
        art,
        &art.dark_wood,
        [x, y - 0.17, z],
        [0.07, 0.34, 0.07],
    );
    box_at(parent, art, &art.amber, [x, y, z], [0.16, 0.19, 0.14]);
    box_at(
        parent,
        art,
        &art.timber,
        [x, y + 0.12, z],
        [0.23, 0.06, 0.19],
    );
}

fn pot(parent: &mut ChildSpawnerCommands, art: &Art, x: f32, z: f32, y: f32, flowers: bool) {
    round(parent, art, &art.clay, [x, y + 0.16, z], 0.34, 0.29);
    round(parent, art, &art.ridge, [x, y + 0.31, z], 0.39, 0.075);
    let mat = if flowers { &art.berry } else { &art.leaf[1] };
    mound(
        parent,
        art,
        mat,
        Vec3::new(x, y + 0.41, z),
        Vec3::new(0.36, 0.28, 0.32),
    );
}

fn pennant(parent: &mut ChildSpawnerCommands, art: &Art, biome: Biome, x: f32, y: f32, z: f32) {
    box_at(
        parent,
        art,
        &art.plank,
        [x, y + 0.36, z],
        [0.60, 0.07, 0.08],
    );
    box_at(parent, art, trim(art, biome), [x, y, z], [0.46, 0.66, 0.06]);
    part(
        parent,
        &art.cube,
        &art.gold,
        Vec3::new(x, y + 0.06, z + 0.052),
        Vec3::new(0.16, 0.16, 0.04),
        Quat::from_rotation_z(0.79),
    );
}

fn shell(
    parent: &mut ChildSpawnerCommands,
    art: &Art,
    biome: Biome,
    center: [f32; 2],
    size: [f32; 3],
    sod: bool,
) {
    let [x, z] = center;
    let [w, h, d] = size;
    box_at(
        parent,
        art,
        &art.warm_stone,
        [x, 0.11, z],
        [w + 0.14, 0.22, d + 0.16],
    );
    box_at(
        parent,
        art,
        wall(art, biome),
        [x, h / 2. + 0.17, z],
        [w, h, d],
    );
    box_at(
        parent,
        art,
        trim(art, biome),
        [x, 0.25, z + d / 2.],
        [w, 0.13, 0.07],
    );
    roof(
        parent,
        art,
        biome,
        [x, h + 0.20, z],
        [w + 0.38, 1.03, d + 0.32],
        sod,
    );
    box_at(
        parent,
        art,
        &art.dark_wood,
        [x, 0.64, z + d / 2. + 0.055],
        [0.43, 0.92, 0.12],
    );
    box_at(
        parent,
        art,
        trim(art, biome),
        [x, 0.62, z + d / 2. + 0.117],
        [0.31, 0.78, 0.035],
    );
    for sign in [-1., 1.] {
        let wx = x + sign * w * 0.33;
        box_at(
            parent,
            art,
            &art.dark_wood,
            [wx, 0.98, z + d / 2. + 0.045],
            [0.40, 0.42, 0.09],
        );
        box_at(
            parent,
            art,
            &art.amber,
            [wx, 0.98, z + d / 2. + 0.095],
            [0.26, 0.29, 0.05],
        );
        box_at(
            parent,
            art,
            &art.timber,
            [wx, 0.98, z + d / 2. + 0.13],
            [0.045, 0.29, 0.045],
        );
    }
}

fn canopy(
    parent: &mut ChildSpawnerCommands,
    art: &Art,
    biome: Biome,
    center: [f32; 2],
    size: [f32; 3],
) {
    let [x, z] = center;
    let [w, h, d] = size;
    for dx in [-0.44, 0.44] {
        for dz in [-0.43, 0.43] {
            column(parent, art, biome, x + dx * w, z + dz * d, h - 0.16);
        }
    }
    roof(
        parent,
        art,
        biome,
        [x, h, z],
        [w + 0.22, 0.66, d + 0.17],
        false,
    );
}

fn exhibit(parent: &mut ChildSpawnerCommands, art: &Art, mat: Material, x: f32, z: f32) {
    parcel(parent, art, mat, Vec3::new(x, 0.28, z), 0.85);
}

fn shields(parent: &mut ChildSpawnerCommands, art: &Art, biome: Biome, x: f32, y: f32, z: f32) {
    for dx in [-0.42, 0.42] {
        part(
            parent,
            &art.cylinder,
            trim(art, biome),
            Vec3::new(x + dx, y, z),
            Vec3::new(0.43, 0.075, 0.43),
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        );
        mound(
            parent,
            art,
            &art.gold,
            Vec3::new(x + dx, y, z + 0.064),
            Vec3::splat(0.14),
        );
    }
}

fn brazier(parent: &mut ChildSpawnerCommands, art: &Art, x: f32, y: f32, z: f32) {
    round(parent, art, &art.dark_wood, [x, y, z], 0.54, 0.16);
    mound(
        parent,
        art,
        &art.amber,
        Vec3::new(x, y + 0.23, z),
        Vec3::new(0.31, 0.49, 0.31),
    );
    mound(
        parent,
        art,
        &art.ridge,
        Vec3::new(x + 0.12, y + 0.14, z + 0.075),
        Vec3::new(0.19, 0.36, 0.23),
    );
}

fn steps(parent: &mut ChildSpawnerCommands, art: &Art, x: f32, z: f32, w: f32) {
    for n in 0..3 {
        let i = n as f32;
        box_at(
            parent,
            art,
            &art.limestone,
            [x, 0.065 + i * 0.056, z - i * 0.16],
            [w, i * 0.112 + 0.13, 0.20],
        );
    }
}

pub fn draw(parent: &mut ChildSpawnerCommands, art: &Art, biome: Biome, kind: Civic) {
    let width = f32::from(kind.spec().size) * 1.48;
    box_at(
        parent,
        art,
        &art.warm_stone,
        [0., 0.065, 0.],
        [width, 0.13, width * 0.96],
    );
    match kind {
        Civic::Lodge => {
            shell(parent, art, biome, [0., -0.17], [2.58, 1.52, 2.10], true);
            if biome == Biome::Aegean {
                for x in [-1.12, 1.12] {
                    column(parent, art, biome, x, 1.15, 1.16);
                }
                box_at(
                    parent,
                    art,
                    &art.limestone,
                    [0., 1.35, 1.15],
                    [2.47, 0.15, 0.47],
                );
            }
            for x in [-0.92, 0.92] {
                pot(parent, art, x, 1.13, 0.12, true);
            }
            steps(parent, art, 0., 1.42, 0.58);
        }
        Civic::Courtyard => {
            shell(parent, art, biome, [0., -1.18], [3.64, 1.70, 1.66], true);
            canopy(parent, art, biome, [-1.37, 0.52], [1.15, 1.45, 1.90]);
            canopy(parent, art, biome, [1.37, 0.52], [1.15, 1.45, 1.90]);
            round(parent, art, &art.limestone, [0., 0.23, 0.66], 0.75, 0.22);
            round(parent, art, &art.ripple, [0., 0.36, 0.66], 0.57, 0.026);
            for x in [-1.72, 1.72] {
                pot(parent, art, x, 1.64, 0.13, true);
            }
        }
        Civic::Nursery => {
            shell(parent, art, biome, [0., -0.62], [2.27, 1.27, 1.45], false);
            pennant(parent, art, biome, -0.99, 1.27, 0.27);
            for x in [-0.64, 0.34] {
                box_at(parent, art, &art.plank, [x, 0.32, 1.05], [0.71, 0.15, 0.31]);
                mound(
                    parent,
                    art,
                    &art.berry,
                    Vec3::new(x, 0.47, 1.05),
                    Vec3::splat(0.22),
                );
            }
            pot(parent, art, 1.03, 1.07, 0.12, true);
        }
        Civic::Infirmary => {
            shell(parent, art, biome, [0., -0.23], [2.35, 1.58, 1.93], false);
            box_at(
                parent,
                art,
                trim(art, biome),
                [0., 1.38, 1.02],
                [1.45, 0.12, 0.69],
            );
            for x in [-1.02, 1.02] {
                pot(parent, art, x, 1.21, 0.12, false);
            }
            mound(
                parent,
                art,
                &art.leaf[1],
                Vec3::new(0., 1.68, 0.83),
                Vec3::new(0.39, 0.25, 0.15),
            );
        }
        Civic::Bath => {
            for (y, d, mat) in [
                (0.20, 2.3, &art.warm_stone),
                (0.35, 2.05, &art.limestone),
                (0.43, 1.74, &art.ripple),
            ] {
                round(parent, art, mat, [0., y, 0.45], d, 0.14);
            }
            canopy(parent, art, biome, [0., -1.32], [3.28, 1.66, 1.22]);
            for x in [-1.77, 1.77] {
                box_at(parent, art, &art.plank, [x, 0.34, 0.59], [0.42, 0.27, 1.32]);
                pot(parent, art, x, 1.57, 0.12, true);
            }
        }
        Civic::Sawmill => {
            canopy(parent, art, biome, [0., -0.57], [2.51, 1.75, 1.35]);
            box_at(
                parent,
                art,
                &art.timber,
                [0.06, 0.52, 0.52],
                [2.01, 0.56, 0.70],
            );
            part(
                parent,
                &art.cylinder,
                &art.stone,
                Vec3::new(0.26, 0.94, 0.69),
                Vec3::new(0.68, 0.105, 0.68),
                Quat::from_rotation_x(1.57),
            );
            exhibit(parent, art, Material::Wood, -0.93, 0.94);
            box_at(
                parent,
                art,
                &art.plank,
                [0.86, 0.87, 0.46],
                [0.87, 0.11, 0.31],
            );
        }
        Civic::Masonry => {
            canopy(parent, art, biome, [0.47, -0.48], [1.78, 1.69, 1.54]);
            for (i, x) in [-1.03, -0.40, 0.22].into_iter().enumerate() {
                for level in 0..i + 1 {
                    box_at(
                        parent,
                        art,
                        &art.limestone,
                        [x, 0.30 + level as f32 * 0.24, 0.73],
                        [0.57, 0.22, 0.57],
                    );
                }
            }
            exhibit(parent, art, Material::Stone, 0.98, 0.82);
            pot(parent, art, -1.08, -0.95, 0.12, false);
        }
        Civic::Bakery => {
            shell(parent, art, biome, [0.28, -0.50], [1.92, 1.32, 1.41], false);
            mound(
                parent,
                art,
                &art.clay,
                Vec3::new(-0.96, 0.48, 0.33),
                Vec3::new(0.91, 1.01, 0.91),
            );
            box_at(
                parent,
                art,
                &art.dark_wood,
                [-0.95, 0.40, 0.78],
                [0.47, 0.38, 0.15],
            );
            box_at(
                parent,
                art,
                &art.amber,
                [-0.95, 0.44, 0.87],
                [0.23, 0.19, 0.04],
            );
            box_at(
                parent,
                art,
                &art.plank,
                [0.29, 0.43, 0.92],
                [1.33, 0.48, 0.59],
            );
            for x in [-0.10, 0.28, 0.70] {
                mound(
                    parent,
                    art,
                    &art.ridge,
                    Vec3::new(x, 0.75, 0.91),
                    Vec3::new(0.27, 0.20, 0.37),
                );
            }
        }
        Civic::Forge => {
            canopy(parent, art, biome, [0., -0.51], [2.43, 1.91, 1.35]);
            box_at(
                parent,
                art,
                &art.stone,
                [-0.83, 1.08, -0.69],
                [0.60, 1.94, 0.69],
            );
            box_at(
                parent,
                art,
                &art.amber,
                [-0.83, 0.51, -0.33],
                [0.36, 0.35, 0.05],
            );
            box_at(
                parent,
                art,
                &art.timber,
                [0.55, 0.44, 0.82],
                [0.55, 0.58, 0.57],
            );
            box_at(
                parent,
                art,
                &art.stone,
                [0.55, 0.88, 0.82],
                [0.82, 0.21, 0.30],
            );
            box_at(
                parent,
                art,
                &art.warm_stone,
                [-0.83, 2.10, -0.69],
                [0.71, 0.15, 0.78],
            );
            for (i, offset) in [0., 0.27, 0.49].into_iter().enumerate() {
                mound(
                    parent,
                    art,
                    &art.plaster,
                    Vec3::new(-0.81 + offset, 2.31 + offset, -0.69),
                    Vec3::splat(0.18 + i as f32 * 0.06),
                );
            }
        }
        Civic::Port => {
            canopy(parent, art, biome, [0., -0.63], [3.40, 1.9, 1.55]);
            for i in -4..=4 {
                box_at(
                    parent,
                    art,
                    &art.plank,
                    [i as f32 * 0.40, 0.24, 1.06],
                    [0.37, 0.14, 1.76],
                );
            }
            for x in [-1.63, 1.63] {
                column(parent, art, biome, x, 1.53, 0.44);
                exhibit(parent, art, Material::Wood, x * 0.77, 0.54);
            }
            pennant(parent, art, biome, 1.46, 1.31, -0.21);
            exhibit(parent, art, Material::Food, 0.13, 0.43);
        }
        Civic::Garden => {
            for x in [-0.97, 0.97] {
                box_at(parent, art, &art.clay, [x, 0.19, 0.25], [0.62, 0.13, 1.68]);
                for z in [-0.39, 0.07, 0.53, 0.99] {
                    pot(parent, art, x, z, 0.24, true);
                }
            }
            canopy(parent, art, biome, [0., -0.73], [1.23, 1.29, 0.60]);
            for x in [-0.43, 0.43] {
                lamp(parent, art, x, 0.49, 1.20);
            }
        }
        Civic::Tavern => {
            shell(parent, art, biome, [0., -0.53], [2.43, 1.55, 1.51], true);
            box_at(
                parent,
                art,
                &art.plank,
                [0., 0.55, 0.88],
                [1.45, 0.18, 0.72],
            );
            for x in [-0.65, 0.65] {
                box_at(
                    parent,
                    art,
                    &art.timber,
                    [x, 0.32, 0.87],
                    [0.15, 0.51, 0.52],
                );
            }
            pot(parent, art, 0.19, 0.88, 0.63, false);
            pennant(parent, art, biome, -1.0, 1.23, 0.29);
            lamp(parent, art, 1.00, 1.23, 0.39);
        }
        Civic::Hall => {
            shell(parent, art, biome, [0., -0.56], [3.66, 1.92, 2.4], true);
            canopy(parent, art, biome, [0., 0.96], [2.55, 1.57, 0.94]);
            for x in [-1.73, 1.73] {
                pennant(parent, art, biome, x, 1.57, 0.72);
                brazier(parent, art, x, 0.37, 1.57);
            }
            steps(parent, art, 0., 1.80, 1.84);
        }
        Civic::Theater => {
            round(parent, art, &art.limestone, [0., 0.19, 0.44], 3.10, 0.19);
            round(parent, art, trim(art, biome), [0., 0.29, 0.45], 1.52, 0.05);
            for tier in 0..3 {
                let radius = 1.26 + tier as f32 * 0.32;
                for step in -4..=4 {
                    let angle = step as f32 * 0.28;
                    let p = Vec3::new(
                        angle.sin() * radius,
                        0.32 + tier as f32 * 0.15,
                        0.55 - angle.cos() * radius,
                    );
                    part(
                        parent,
                        &art.cube,
                        &art.limestone,
                        p,
                        Vec3::new(0.47 + tier as f32 * 0.09, 0.19, 0.34),
                        Quat::from_rotation_y(-angle),
                    );
                }
            }
            for x in [-1.30, -0.46, 0.46, 1.30] {
                column(parent, art, biome, x, -1.47, 1.29);
            }
            box_at(
                parent,
                art,
                trim(art, biome),
                [0., 1.56, -1.47],
                [2.95, 0.20, 0.30],
            );
            for x in [-1.58, 1.58] {
                brazier(parent, art, x, 0.35, 1.34);
            }
        }
        Civic::Archive => {
            shell(parent, art, biome, [0., -0.48], [2.55, 1.63, 1.37], false);
            canopy(parent, art, biome, [0., 0.79], [2.10, 1.35, 0.82]);
            box_at(
                parent,
                art,
                &art.timber,
                [0., 0.70, 0.54],
                [1.39, 0.76, 0.45],
            );
            for x in [-0.53, -0.26, 0., 0.26, 0.53] {
                box_at(parent, art, &art.white, [x, 0.90, 0.80], [0.11, 0.39, 0.19]);
            }
        }
        Civic::Oracle => {
            steps(parent, art, 0., 0.65, 1.16);
            canopy(parent, art, biome, [0., -0.35], [1.24, 1.53, 0.89]);
            round(parent, art, &art.limestone, [0., 0.31, 0.02], 0.55, 0.37);
            mound(
                parent,
                art,
                &art.gold,
                Vec3::new(0., 0.79, 0.02),
                Vec3::new(0.36, 0.59, 0.37),
            );
            for x in [-0.59, 0.59] {
                lamp(parent, art, x, 0.49, 0.55);
            }
        }
        Civic::Temple => {
            for (y, width, depth) in [(0.16, 4.17, 3.88), (0.33, 3.92, 3.65), (0.48, 3.70, 3.38)] {
                box_at(
                    parent,
                    art,
                    &art.limestone,
                    [0., y, -0.10],
                    [width, 0.18, depth],
                );
            }
            for x in [-1.47, -0.48, 0.48, 1.47] {
                for z in [-1.41, 1.41] {
                    column(parent, art, biome, x, z, 2.02);
                }
            }
            box_at(
                parent,
                art,
                wall(art, biome),
                [0., 1.20, -0.25],
                [2.07, 1.42, 1.74],
            );
            box_at(
                parent,
                art,
                trim(art, biome),
                [0., 2.21, 0.],
                [3.63, 0.23, 3.29],
            );
            roof(
                parent,
                art,
                biome,
                [0., 2.35, 0.],
                [3.72, 1.12, 3.36],
                false,
            );
            steps(parent, art, 0., 2.00, 2.12);
            for x in [-1.22, 1.22] {
                brazier(parent, art, x, 0.68, 1.71);
            }
            pennant(parent, art, biome, 0., 1.81, 0.67);
        }
        Civic::Palisade => {
            for x in [-0.45, 0., 0.45] {
                box_at(parent, art, &art.timber, [x, 0.56, 0.], [0.27, 0.90, 0.23]);
                part(
                    parent,
                    &art.cone,
                    &art.plank,
                    Vec3::new(x, 1.09, 0.),
                    Vec3::new(0.32, 0.30, 0.33),
                    Quat::IDENTITY,
                );
            }
            box_at(
                parent,
                art,
                &art.dark_wood,
                [0., 0.51, 0.14],
                [1.36, 0.13, 0.14],
            );
        }
        Civic::Beacon => {
            round(parent, art, &art.stone, [0., 0.62, 0.], 0.83, 1.07);
            round(parent, art, &art.limestone, [0., 1.19, 0.], 1.30, 0.17);
            for x in [-0.44, 0.44] {
                column(parent, art, biome, x, 0.19, 1.46);
            }
            brazier(parent, art, 0., 1.48, 0.);
            pennant(parent, art, biome, 0., 0.80, 0.47);
        }
        Civic::Barracks => {
            shell(
                parent,
                art,
                biome,
                [0., -0.26],
                [2.36, 1.61, 1.83],
                biome == Biome::Fjord,
            );
            for x in [-1.04, 1.04] {
                column(parent, art, biome, x, 1.18, 1.17);
            }
            shields(parent, art, biome, 0., 1.38, 0.70);
            pennant(parent, art, biome, 1.09, 1.78, 0.53);
            steps(parent, art, 0., 1.30, 0.54);
        }
        Civic::Bastion => {
            for (x, h) in [(-1.13, 2.08), (1.15, 2.62)] {
                box_at(
                    parent,
                    art,
                    &art.limestone,
                    [x, h * 0.5 + 0.13, 0.],
                    [1.52, h, 2.73],
                );
                box_at(
                    parent,
                    art,
                    trim(art, biome),
                    [x, h * 0.75, 0.],
                    [1.56, 0.20, 2.76],
                );
                for dx in [-0.59, 0., 0.59] {
                    box_at(
                        parent,
                        art,
                        &art.limestone,
                        [x + dx, h + 0.30, 1.26],
                        [0.27, 0.40, 0.33],
                    );
                }
                shields(parent, art, biome, x, 0.96, 1.43);
            }
            box_at(
                parent,
                art,
                &art.stone,
                [0., 0.67, -1.11],
                [2.22, 1.11, 0.64],
            );
            pennant(parent, art, biome, 1.16, 2.79, 1.38);
            for x in [-1.71, 1.72] {
                brazier(parent, art, x, 0.36, 1.68);
            }
            steps(parent, art, 0., 1.77, 0.76);
        }
    }
}
