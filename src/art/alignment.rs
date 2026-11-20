use bevy::prelude::*;

use crate::{game::Game, growth::Tone};

use super::{Art, block, mound, part};

#[derive(Component)]
pub(crate) struct CreatureDecor(Tone);

pub fn companion_decor(parent: &mut ChildSpawnerCommands, art: &Art) {
    parent
        .spawn((
            CreatureDecor(Tone::Gentle),
            Transform::default(),
            Visibility::Hidden,
        ))
        .with_children(|crown| {
            for i in 0..7 {
                let theta = (i as f32 - 3.) * 0.28;
                mound(
                    crown,
                    art,
                    &art.leaf[i % 3],
                    Vec3::new(theta.sin() * 0.46, 1.62, 0.10 + 0.29 * theta.cos()),
                    Vec3::new(0.17, 0.12, 0.17),
                );
            }
            mound(
                crown,
                art,
                &art.gold,
                Vec3::new(0., 1.68, 0.38),
                Vec3::splat(0.14),
            );
            for side in [-1., 1.] {
                mound(
                    crown,
                    art,
                    &art.berry,
                    Vec3::new(side * 0.22, 1.66, 0.34),
                    Vec3::splat(0.094),
                );
            }
        });
    parent
        .spawn((
            CreatureDecor(Tone::Stern),
            Transform::default(),
            Visibility::Hidden,
        ))
        .with_children(|armor| {
            for side in [-1., 1.] {
                part(
                    armor,
                    &art.cone,
                    &art.dark_wood,
                    Vec3::new(side * 0.34, 1.77, 0.14),
                    Vec3::new(0.20, 0.40, 0.22),
                    Quat::from_rotation_z(-side * 0.40),
                );
                mound(
                    armor,
                    art,
                    &art.ridge,
                    Vec3::new(side * 0.31, 0.92, 0.),
                    Vec3::new(0.26, 0.24, 0.49),
                );
                block(
                    armor,
                    art,
                    &art.red,
                    Vec3::new(side * 0.31, 0.91, 0.24),
                    Vec3::new(0.12, 0.08, 0.07),
                );
            }
        });
}

pub(crate) fn style_creature(
    game: Res<Game>,
    mut decorations: Query<(&CreatureDecor, &mut Visibility)>,
) {
    let tone = game.campaign.gifts.creature.nature.tone();
    for (part, mut visible) in &mut decorations {
        *visible = if part.0 == tone {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

pub fn city_decor(parent: &mut ChildSpawnerCommands, art: &Art, ground: Vec3, tone: Tone) {
    parent
        .spawn((Transform::from_translation(ground), Visibility::Inherited))
        .with_children(|decoration| {
            let foot = Vec3::new(0., 0.14, 0.);
            if tone == Tone::Gentle {
                block(
                    decoration,
                    art,
                    &art.plank,
                    foot,
                    Vec3::new(0.58, 0.27, 0.50),
                );
                for side in [-1., 0., 1.] {
                    mound(
                        decoration,
                        art,
                        &art.leaf[1],
                        Vec3::new(side * 0.18, 0.31, 0.),
                        Vec3::new(0.26, 0.28, 0.31),
                    );
                    mound(
                        decoration,
                        art,
                        &art.berry,
                        Vec3::new(side * 0.19, 0.46, 0.02),
                        Vec3::splat(0.14),
                    );
                }
                block(
                    decoration,
                    art,
                    &art.timber,
                    Vec3::new(0.0, 0.75, 0.14),
                    Vec3::new(0.085, 1.36, 0.082),
                );
                mound(
                    decoration,
                    art,
                    &art.gold,
                    Vec3::new(0., 1.48, 0.14),
                    Vec3::new(0.29, 0.34, 0.29),
                );
            } else {
                block(
                    decoration,
                    art,
                    &art.ridge,
                    foot,
                    Vec3::new(0.68, 0.28, 0.56),
                );
                block(
                    decoration,
                    art,
                    &art.dark_wood,
                    Vec3::Y * 0.89,
                    Vec3::new(0.28, 1.56, 0.28),
                );
                part(
                    decoration,
                    &art.cone,
                    &art.ridge,
                    Vec3::new(0., 1.81, 0.),
                    Vec3::new(0.45, 0.46, 0.45),
                    Quat::IDENTITY,
                );
                block(
                    decoration,
                    art,
                    &art.red,
                    Vec3::new(0., 1.22, 0.15),
                    Vec3::new(0.53, 0.76, 0.06),
                );
                block(
                    decoration,
                    art,
                    &art.gold,
                    Vec3::new(0., 1.26, 0.19),
                    Vec3::new(0.11, 0.31, 0.03),
                );
            }
        });
}
