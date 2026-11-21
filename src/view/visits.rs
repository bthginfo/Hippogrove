use super::*;
use crate::game::{Allegiance, Settlement};
use crate::region::Region;

#[derive(Component)]
pub(crate) struct Envoy;
#[derive(Component)]
pub(crate) struct PilgrimParty;
#[derive(Component)]
pub(crate) struct Host(usize);

pub(super) fn populate(
    parent: &mut ChildSpawnerCommands,
    art: &Art,
    region: &Region,
    city: &Settlement,
) {
    for (i, v) in region.villages.iter().enumerate() {
        let base = terrain::on_ground(region, v.center.world());
        for &tile in &v.houses {
            if city.building_at(tile).is_none() {
                parent
                    .spawn((
                        Transform::from_translation(terrain::on_ground(region, tile.world()))
                            .with_scale(Vec3::splat(0.84)),
                        Visibility::Inherited,
                    ))
                    .with_children(|p| art::building(p, art, region.kind, Building::House));
            }
        }
        // A meeting pole, a local host and the requested goods make each plaza recognizable at ground level.
        parent
            .spawn((Transform::from_translation(base), Visibility::Inherited))
            .with_children(|p| {
                block(
                    p,
                    art,
                    &art.timber,
                    Vec3::new(0.7, 1.5, 0.5),
                    Vec3::new(0.15, 3., 0.15),
                );
                let flag = block(
                    p,
                    art,
                    &art.limestone,
                    Vec3::new(1.29, 2.45, 0.5),
                    Vec3::new(1.15, 0.72, 0.08),
                );
                p.commands().entity(flag).insert(Banner::Village(i));
                art::parcel(p, art, v.need, Vec3::new(0.8, 0.22, 1.15), 0.9);
            });
        let spot = v
            .center
            .neighbors()
            .into_iter()
            .find(|&t| city.walkable(region, t))
            .unwrap_or(v.center);
        parent
            .spawn((
                Host(i),
                Transform::from_translation(
                    terrain::on_ground(region, spot.world()) + Vec3::new(0.22, 0., 0.18),
                ),
                Visibility::Inherited,
            ))
            .with_children(|p| {
                contact_shadow(p, art, 0.48);
                villager(p, art, region.kind, false, i + 2);
            });
    }
    if !region.tutorial {
        parent
            .spawn((PilgrimParty, Transform::default(), Visibility::Hidden))
            .with_children(|p| {
                for x in [-0.48, 0.48] {
                    p.spawn((
                        Transform::from_xyz(x, 0., x * 0.4).with_scale(Vec3::splat(0.85)),
                        Visibility::Inherited,
                    ))
                    .with_children(|q| {
                        villager(q, art, region.kind, false, 3);
                    });
                }
                block(
                    p,
                    art,
                    &art.timber,
                    Vec3::new(0., 1.15, 0.),
                    Vec3::new(0.08, 2.3, 0.08),
                );
                block(
                    p,
                    art,
                    &art.gold,
                    Vec3::new(0.32, 2.02, 0.),
                    Vec3::new(0.65, 0.42, 0.08),
                );
            });
    }
    if !region.sandbox && !region.tutorial {
        let pos = city.contest.rival_tile.unwrap_or(region.rival_lair());
        parent
            .spawn((
                Envoy,
                Transform::from_translation(terrain::on_ground(region, pos.world())),
                Visibility::Inherited,
            ))
            .with_children(|p| {
                contact_shadow(p, art, 0.65);
                villager(p, art, region.kind, false, 2);
                block(
                    p,
                    art,
                    &art.timber,
                    Vec3::new(0.43, 1.2, 0.04),
                    Vec3::new(0.09, 2.4, 0.09),
                );
                block(
                    p,
                    art,
                    &art.purple,
                    Vec3::new(0.72, 2.02, 0.04),
                    Vec3::new(0.64, 0.50, 0.06),
                );
            });
    }
}

pub(crate) fn animate(
    game: Res<Game>,
    time: Res<Time>,
    mut hosts: Query<(&Host, &mut Transform), (Without<Envoy>, Without<PilgrimParty>)>,
    mut envoys: Query<(&mut Transform, &mut Visibility), (With<Envoy>, Without<PilgrimParty>)>,
    mut pilgrims: Query<(&mut Transform, &mut Visibility), (With<PilgrimParty>, Without<Envoy>)>,
) {
    let region = game.region();
    let city = game.city();
    for (mut pose, mut visible) in &mut pilgrims {
        if let Some(p) = &city.projects.pilgrims {
            let goal = p.tile.world();
            let delta = (goal - pose.translation).with_y(0.);
            let next = if *visible == Visibility::Hidden {
                goal
            } else {
                pose.translation
                    .lerp(goal, 1. - (-9. * time.delta_secs().min(0.16)).exp())
            };
            pose.translation = next.with_y(
                terrain::actor_height(region, next, city.bridge_fixed)
                    + (time.elapsed_secs() * 7.).sin().abs() * 0.06,
            );
            if delta.length_squared() > 0.02 {
                pose.rotation = Quat::from_rotation_y(delta.x.atan2(delta.z));
            }
            *visible = Visibility::Inherited;
        } else {
            *visible = Visibility::Hidden;
        }
    }
    for (host, mut transform) in &mut hosts {
        let i = host.0;
        let Some(v) = region.villages.get(i) else {
            continue;
        };
        let guest = city.walkers.iter().find(|w| city.visiting(w, region, i));
        let target = guest.map(|w| w.world()).unwrap_or_else(|| v.center.world());
        let delta = target - transform.translation;
        let beat = (time.elapsed_secs() * 3. + i as f32).sin();
        let bow = if guest.is_some() {
            0.13 * (beat + 1.)
        } else {
            0.
        };
        transform.rotation =
            Quat::from_rotation_y(delta.x.atan2(delta.z)) * Quat::from_rotation_x(bow);
    }
    for (mut pose, mut vis) in &mut envoys {
        *vis = if city.contest.pact || city.rival_resolve == 0 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        let target = city
            .contest
            .rival_tile
            .unwrap_or(region.rival_lair())
            .world();
        let delta = (target - pose.translation).with_y(0.);
        let p = pose
            .translation
            .lerp(target, 1. - (-7. * time.delta_secs().min(0.16)).exp());
        let y = terrain::actor_height(region, p, city.bridge_fixed);
        pose.translation = p.with_y(
            y + if delta.length_squared() > 0.05 {
                (time.elapsed_secs() * 9.).sin().abs() * 0.08
            } else {
                0.
            },
        );
        if delta.length_squared() > 0.005 {
            pose.rotation = Quat::from_rotation_y(delta.x.atan2(delta.z));
        }
    }
}

pub(super) fn color(art: &Art, side: Allegiance) -> Handle<StandardMaterial> {
    match side {
        Allegiance::Ours => art.valid.clone(),
        Allegiance::Neutral => art.limestone.clone(),
        Allegiance::Rival => art.purple.clone(),
    }
}
