use super::*;

#[derive(Component)]
pub(crate) struct Lantern(usize);

#[derive(Default)]
pub(crate) struct FestivalView {
    group: Option<Entity>,
    location: Option<(Entity, Tile, crate::game::Rite)>,
    last_burst: Option<f32>,
}

pub(crate) fn stage(
    mut commands: Commands,
    board: Res<Board>,
    art: Res<Art>,
    game: Res<Game>,
    play: Res<Play>,
    time: Res<Time>,
    mut state: Local<FestivalView>,
    mut lamps: Query<(
        &Lantern,
        &mut Transform,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let city = game.city();
    let region = game.region();
    let circle = city.gatherings.circle.as_ref();
    let location = board
        .root
        .zip(circle)
        .map(|(root, c)| (root, c.center, c.kind));
    if location != state.location {
        if let Some(old) = state.group.take() {
            commands.entity(old).try_despawn();
        }
        state.location = location;
        if let (Some((root, _, _)), Some(c)) = (location, circle) {
            let fabric = match region.kind {
                Biome::Valley => &art.leaf[2],
                Biome::Fjord => &art.blue,
                Biome::Terraces => &art.red,
                Biome::Lake => &art.amber,
                Biome::Aegean => &art.cobalt,
            };
            let group = commands
                .spawn((ChildOf(root), Transform::default(), Visibility::Inherited))
                .with_children(|p| {
                    for &(i, spot) in &c.guests {
                        let base = terrain::on_ground(region, spot.world());
                        part(
                            p,
                            &art.ring,
                            &art.gold,
                            base + Vec3::Y * 0.09,
                            Vec3::splat(0.72),
                            Quat::IDENTITY,
                        );
                        let dir = (spot.world() - c.center.world()).normalize_or_zero();
                        let foot = base + dir * 0.79;
                        block(
                            p,
                            &art,
                            &art.timber,
                            foot + Vec3::Y * 0.76,
                            Vec3::new(0.1, 1.52, 0.1),
                        );
                        let turn = Quat::from_rotation_y(dir.x.atan2(dir.z));
                        let pos = |x, y, z| foot + turn * Vec3::new(x, y, z);
                        part(
                            p,
                            &art.cube,
                            &art.gold,
                            pos(0., 1.44, 0.),
                            Vec3::new(0.82, 0.08, 0.1),
                            turn,
                        );
                        part(
                            p,
                            &art.cube,
                            fabric,
                            pos(0., 1.12, 0.02),
                            Vec3::new(0.62, 0.58, 0.07),
                            turn,
                        );
                        match region.kind {
                            Biome::Fjord => {
                                for side in [-1., 1.] {
                                    part(
                                        p,
                                        &art.cube,
                                        &art.white,
                                        pos(side * 0.09, 1.15, 0.07),
                                        Vec3::new(0.08, 0.36, 0.05),
                                        turn * Quat::from_rotation_z(side * 0.53),
                                    );
                                }
                            }
                            Biome::Terraces => {
                                part(
                                    p,
                                    &art.roof,
                                    &art.clay,
                                    pos(0., 1.63, 0.),
                                    Vec3::new(0.8, 0.27, 0.46),
                                    turn,
                                );
                            }
                            Biome::Lake => {
                                for side in [-1., 0., 1.] {
                                    part(
                                        p,
                                        &art.lump,
                                        &art.leaf[0],
                                        pos(side * 0.15, 1.63, 0.),
                                        Vec3::new(0.14, 0.47, 0.14),
                                        turn * Quat::from_rotation_z(-side * 0.49),
                                    );
                                }
                            }
                            Biome::Valley => {
                                for side in [-1., 1.] {
                                    part(
                                        p,
                                        &art.lump,
                                        &art.leaf[1],
                                        pos(side * 0.19, 1.5, 0.),
                                        Vec3::new(0.34, 0.17, 0.17),
                                        turn * Quat::from_rotation_z(side * 0.35),
                                    );
                                }
                            }
                            Biome::Aegean => {
                                for x in [-0.18, 0., 0.18] {
                                    block(
                                        p,
                                        &art,
                                        &art.white,
                                        pos(x, 0.93, 0.07),
                                        Vec3::new(0.085, 0.18, 0.07),
                                    );
                                }
                            }
                        }
                        p.spawn((
                            Lantern(i),
                            Mesh3d(art.lump.clone()),
                            MeshMaterial3d(art.amber.clone()),
                            Transform::from_translation(foot + Vec3::Y * 1.96),
                        ));
                    }
                })
                .id();
            state.group = Some(group);
        }
    }
    if let Some(c) = circle {
        for (lamp, mut transform, mut color) in &mut lamps {
            if let Some((_, spot)) = c.guests.iter().find(|(i, _)| *i == lamp.0) {
                let w = &city.walkers[lamp.0];
                let arrived = w.tile() == *spot && w.route.is_empty() && w.bundle.is_none();
                let t = time.elapsed_secs() * 3. + lamp.0 as f32;
                transform.scale = Vec3::splat(if arrived { 0.63 + t.sin() * 0.08 } else { 0.26 });
                transform.rotation = Quat::from_rotation_y(t * 0.5);
                color.0 = if arrived {
                    art.gold.clone()
                } else {
                    art.blue.clone()
                };
            }
        }
    }
    if let Some((center, awe, at, kind)) = play.celebration
        && state.last_burst != Some(at)
        && time.elapsed_secs() - at < 1.
    {
        state.last_burst = Some(at);
        let Some(root) = board.root else {
            return;
        };
        let mat = if awe {
            &art.purple
        } else {
            match kind {
                crate::game::Rite::Harvest => &art.leaf[1],
                crate::game::Rite::Stone => &art.ripple,
                _ => &art.amber,
            }
        };
        for i in 0..32 {
            let angle = i as f32 * 2.39996;
            let dir = Vec3::new(angle.cos(), 0., angle.sin());
            let r = 0.8 + (i % 5) as f32 * 0.55;
            let p = terrain::on_ground(region, center.world() + dir * r) + Vec3::Y * 0.6;
            commands.spawn((
                ChildOf(root),
                Effect {
                    age: 0.,
                    length: 1.55 + (i % 3) as f32 * 0.19,
                    velocity: dir * 1.1 + Vec3::Y * (1.1 + (i % 4) as f32 * 0.48),
                    scale: 0.22,
                    phase: EffectPhase::Rising,
                },
                Mesh3d(art.lump.clone()),
                MeshMaterial3d(if i % 3 == 0 {
                    art.gold.clone()
                } else {
                    mat.clone()
                }),
                Transform::from_translation(p).with_scale(Vec3::splat(0.22)),
                bevy::pbr::NotShadowCaster,
            ));
        }
    }
}
