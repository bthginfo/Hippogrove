use std::f32::consts::TAU;

use bevy::{pbr::NotShadowCaster, prelude::*};

use crate::{
    art::{Art, block, mound, part},
    game::{DAY_LENGTH, Game, SideQuest},
    play::CameraRig,
    region::{Biome, Region, Tile},
    terrain,
    view::villager,
};

#[derive(Component)]
pub(crate) enum Animal {
    Bird { center: Vec3, flock: usize },
    Beetle { center: Vec3, number: usize },
}

#[derive(Component)]
pub(crate) struct Wing(f32, usize);

#[derive(Component)]
pub(crate) struct Neighbor {
    center: Tile,
    number: usize,
}

#[derive(Component)]
pub(crate) struct QuestSign(SideQuest);

pub(crate) fn sign_position(region: &Region) -> Vec3 {
    terrain::on_ground(region, region.depot.world() + Vec3::new(-0.87, 0., -0.26)) + Vec3::Y * 1.43
}

fn signs(parent: &mut ChildSpawnerCommands, art: &Art, region: &Region) {
    if region.sandbox {
        return;
    }
    for phase in [SideQuest::Available, SideQuest::Ready] {
        let color = if phase == SideQuest::Available {
            &art.gold
        } else {
            &art.valid
        };
        parent
            .spawn((
                QuestSign(phase),
                Transform::from_translation(sign_position(region))
                    .with_rotation(Quat::from_rotation_y(0.46)),
                Visibility::Hidden,
            ))
            .with_children(|sign| {
                part(
                    sign,
                    &art.cylinder,
                    &art.dark_wood,
                    Vec3::ZERO,
                    Vec3::new(0.65, 0.09, 0.65),
                    Quat::from_rotation_x(1.57),
                );
                if phase == SideQuest::Available {
                    block(
                        sign,
                        art,
                        color,
                        Vec3::new(0., 0.057, 0.059),
                        Vec3::new(0.078, 0.23, 0.045),
                    );
                } else {
                    for (x, y, w, h) in [
                        (0., 0.169, 0.20, 0.065),
                        (-0.096, 0.107, 0.065, 0.12),
                        (0.091, 0.092, 0.065, 0.138),
                        (0.045, 0.019, 0.124, 0.068),
                        (0., -0.029, 0.071, 0.092),
                    ] {
                        block(
                            sign,
                            art,
                            color,
                            Vec3::new(x, y, 0.059),
                            Vec3::new(w, h, 0.043),
                        );
                    }
                }
                block(
                    sign,
                    art,
                    color,
                    Vec3::new(0., -0.128, 0.059),
                    Vec3::new(0.080, 0.065, 0.049),
                );
            });
    }
}

fn fauna_piece(
    parent: &mut ChildSpawnerCommands,
    art: &Art,
    color: &Handle<StandardMaterial>,
    position: Vec3,
    size: Vec3,
) {
    let piece = mound(parent, art, color, position, size);
    parent.commands().entity(piece).insert(NotShadowCaster);
}

pub(crate) fn spawn(parent: &mut ChildSpawnerCommands, art: &Art, region: &Region) {
    signs(parent, art, region);
    for (district, anchor) in region.districts.iter().enumerate() {
        for index in 0..3 {
            let number = district * 3 + index;
            parent
                .spawn((
                    Neighbor {
                        center: anchor.center,
                        number,
                    },
                    Transform::from_translation(anchor.center.world())
                        .with_scale(Vec3::splat(0.86)),
                    Visibility::Inherited,
                ))
                .with_children(|actor| {
                    villager(actor, art, false, number + 1);
                    for side in [-1., 1.] {
                        fauna_piece(
                            actor,
                            art,
                            &art.boot,
                            Vec3::new(side * 0.13, 0.19, 0.044),
                            Vec3::new(0.23, 0.40, 0.30),
                        );
                    }
                });
        }
    }
    let shores = [
        region.depot.offset(2, -2),
        region.bridge(),
        region.districts[0].center,
    ];
    for (index, home) in shores.into_iter().enumerate() {
        for member in 0..2 {
            let flock = index * 2 + member;
            parent
                .spawn((
                    Animal::Bird {
                        center: home.world(),
                        flock,
                    },
                    Transform::from_translation(home.world() + Vec3::Y * 3.),
                    Visibility::Hidden,
                ))
                .with_children(|bird| {
                    let plumage =
                        if matches!(region.kind, Biome::Fjord | Biome::Aegean | Biome::Lake) {
                            &art.white
                        } else {
                            &art.warm_stone
                        };
                    fauna_piece(bird, art, plumage, Vec3::ZERO, Vec3::new(0.22, 0.18, 0.43));
                    fauna_piece(
                        bird,
                        art,
                        &art.ridge,
                        Vec3::Z * 0.24,
                        Vec3::new(0.082, 0.066, 0.15),
                    );
                    for side in [-1., 1.] {
                        bird.spawn((
                            Wing(side, flock),
                            Transform::from_xyz(side * 0.08, 0.018, 0.),
                            Visibility::Inherited,
                        ))
                        .with_children(|wing| {
                            fauna_piece(
                                wing,
                                art,
                                plumage,
                                Vec3::X * (side * 0.16),
                                Vec3::new(0.35, 0.066, 0.20),
                            );
                            fauna_piece(
                                wing,
                                art,
                                &art.dark_wood,
                                Vec3::new(side * 0.32, 0., -0.045),
                                Vec3::new(0.18, 0.051, 0.131),
                            );
                        });
                    }
                });
        }
    }
    for (index, tile) in region
        .start
        .neighbors()
        .into_iter()
        .chain(region.depot.neighbors())
        .enumerate()
        .filter(|&(_, tile)| {
            !region.water(tile) && !region.road(tile) && !region.fixed_structure(tile)
        })
    {
        parent
            .spawn((
                Animal::Beetle {
                    center: tile.world(),
                    number: index,
                },
                Transform::from_translation(tile.world()),
                Visibility::Hidden,
            ))
            .with_children(|bug| {
                fauna_piece(
                    bug,
                    art,
                    &art.eye,
                    Vec3::new(0., 0.068, 0.045),
                    Vec3::new(0.17, 0.112, 0.25),
                );
                for side in [-1., 1.] {
                    fauna_piece(
                        bug,
                        art,
                        &art.red,
                        Vec3::new(side * 0.041, 0.104, 0.),
                        Vec3::new(0.091, 0.101, 0.193),
                    );
                    fauna_piece(
                        bug,
                        art,
                        &art.eye,
                        Vec3::new(side * 0.055, 0.145, 0.028),
                        Vec3::splat(0.036),
                    );
                }
            });
    }
}

pub(crate) fn update(
    game: Res<Game>,
    camera: Res<CameraRig>,
    mut actors: Query<
        (&Animal, &mut Transform, &mut Visibility),
        (Without<Wing>, Without<Neighbor>, Without<QuestSign>),
    >,
    mut wings: Query<(&Wing, &mut Transform), (Without<Animal>, Without<Neighbor>)>,
    mut neighbors: Query<(&Neighbor, &mut Transform), (Without<Animal>, Without<Wing>)>,
    mut signs: Query<(&QuestSign, &mut Visibility), Without<Animal>>,
) {
    let region = game.region();
    let city = game.city();
    let clock = city.weather_time;
    let quest = city.side_quest(region);
    for (sign, mut visible) in &mut signs {
        *visible = if sign.0 == quest && camera.rendered_height < 46. {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (resident, mut pose) in &mut neighbors {
        let local = (clock + resident.number as f32 * 2.8).rem_euclid(DAY_LENGTH);
        let stages = [0., 34., 111., 150.];
        let stage = stages
            .partition_point(|&start| start <= local)
            .saturating_sub(1);
        // The town's north-south entrances meet this paved lane.
        let number = (resident.number % 3) as f32;
        let offsets = [
            -2.7 + number * 0.67,
            0.7 + number * 0.81,
            0.04 + number * 0.71,
            -1.8 + number * 0.52,
        ];
        let previous = offsets[(stage + 3) % 4];
        let destination = offsets[stage];
        let progress = terrain::smooth((local - stages[stage]) / 6.4);
        let walking = progress < 0.995;
        let base = resident.center.offset(0, 2).world();
        let next = base
            + Vec3::new(
                previous.lerp(destination, progress) * 1.22,
                0.,
                number * 0.27 - 0.31,
            );
        let next = if city.walkable(region, Tile::from_world(next)) {
            next
        } else {
            base
        };
        pose.translation = terrain::on_ground(region, next)
            + Vec3::Y
                * if walking {
                    (local * 11.).sin().abs() * 0.07
                } else {
                    0.
                };
        let heading = if !walking {
            0.53 + number * 0.62
        } else {
            (destination - previous).signum() * TAU / 4.
        };
        pose.rotation = Quat::from_rotation_y(heading);
    }
    let night = city.day_phase() == crate::game::DayPhase::Night;
    for (animal, mut pose, mut visible) in &mut actors {
        let (center, distance) = match animal {
            Animal::Bird { center, .. } => (center, 31.),
            Animal::Beetle { center, .. } => (center, 17.),
        };
        if night
            || center.distance(camera.target) > distance
            || camera.rendered_height > distance + 3.
        {
            *visible = Visibility::Hidden;
            continue;
        }
        *visible = Visibility::Inherited;
        match *animal {
            Animal::Bird { center, flock } => {
                let angle = clock * 0.32 + (flock % 2) as f32 * 0.43 + flock as f32 * 0.54;
                let course = Vec3::new(angle.cos() * 5.2, 0., angle.sin() * 3.9);
                let bank = (angle * 1.17).sin() * 0.15;
                pose.translation = terrain::on_ground(region, center + course)
                    + Vec3::Y * (3.18 + (flock % 2) as f32 * 0.32 + (angle * 2.).sin() * 0.48);
                pose.rotation = Quat::from_rotation_y((-angle.sin()).atan2(angle.cos()))
                    * Quat::from_rotation_z(bank);
            }
            Animal::Beetle { center, number } => {
                let angle = clock * 0.57 + number as f32;
                let spot = center + Vec3::new(angle.cos() * 0.59, 0., angle.sin() * 0.34);
                *visible = if city.building_at(Tile::from_world(spot)).is_none() {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                pose.translation = terrain::on_ground(region, spot);
                pose.rotation = Quat::from_rotation_y((-angle.sin()).atan2(angle.cos()));
            }
        }
    }
    for (wing, mut pose) in &mut wings {
        let flap = (clock * 13.1 + wing.1 as f32 * 1.39).sin();
        let glide = 0.4 + (clock * 0.47 + wing.1 as f32).sin().abs() * 0.6;
        pose.rotation = Quat::from_rotation_z(wing.0 * (0.13 + flap * glide * 0.61));
    }
}
