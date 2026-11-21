use super::*;
use crate::game::WorkStage;

#[derive(Component)]
pub(crate) struct Yard;
#[derive(Component)]
pub(crate) struct OriginalDepot;
#[derive(Component)]
pub(crate) struct Millwheel;

pub(crate) fn stage(
    mut commands: Commands,
    game: Res<Game>,
    art: Res<Art>,
    assets: Res<AssetServer>,
    board: Res<Board>,
    mut last: Local<Option<(Entity, WorkStage)>>,
    yards: Query<Entity, With<Yard>>,
    mut depot: Query<&mut Visibility, With<OriginalDepot>>,
) {
    let Some(root) = board.root else {
        return;
    };
    let r = game.region();
    if r.tutorial || board.active != Some((game.mode, game.campaign.active)) {
        return;
    }
    let stage = game.city().work.stage;
    if *last == Some((root, stage)) {
        return;
    }
    *last = Some((root, stage));
    for entity in &yards {
        commands.entity(entity).despawn();
    }
    let completed = matches!(stage, WorkStage::Opening | WorkStage::Running);
    for mut v in &mut depot {
        *v = if completed {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
    let at = terrain::on_ground(r, r.depot.world());
    commands.entity(root).with_children(|p| {
        p.spawn((Yard, Transform::from_translation(at), Visibility::Inherited))
            .with_children(|p| {
                if completed {
                    let stem = ["valley", "fjord", "terraces", "lake", "aegean"][r.kind as usize];
                    let scene = assets.load(format!("embedded://models/works/{stem}.glb#Scene0"));
                    art::place_scene(p, &scene, Vec3::ZERO, 1.12, 0.28);
                    return;
                }
                let (x, z) = (-0.66, 0.42);
                block(
                    p,
                    &art,
                    &art.dark_wood,
                    Vec3::new(x, 0.66, z),
                    Vec3::new(0.11, 1.32, 0.10),
                );
                block(
                    p,
                    &art,
                    &art.roof_color[r.kind as usize],
                    Vec3::new(x + 0.22, 1.18, z),
                    Vec3::new(0.53, 0.34, 0.07),
                );
                block(
                    p,
                    &art,
                    &art.gold,
                    Vec3::new(x + 0.24, 1.21, z + 0.045),
                    Vec3::new(0.17, 0.09, 0.03),
                );
                if stage != WorkStage::Offered {
                    for (n, mat) in [&art.stone, &art.plank, &art.timber]
                        .into_iter()
                        .enumerate()
                    {
                        block(
                            p,
                            &art,
                            mat,
                            Vec3::new(0.39, 0.26 + n as f32 * 0.10, -0.35),
                            Vec3::new(0.72, 0.10, 0.29),
                        );
                    }
                }
                if stage == WorkStage::Raising {
                    for dx in [-0.49, 0.49] {
                        block(
                            p,
                            &art,
                            &art.timber,
                            Vec3::new(dx, 0.88, -0.52),
                            Vec3::new(0.13, 1.76, 0.13),
                        );
                    }
                    block(
                        p,
                        &art,
                        &art.plank,
                        Vec3::new(0., 1.71, -0.52),
                        Vec3::new(1.35, 0.13, 0.17),
                    );
                    block(
                        p,
                        &art,
                        &art.gold,
                        Vec3::new(0.14, 1.28, -0.52),
                        Vec3::new(0.025, 0.75, 0.025),
                    );
                }
            });
    });
}
pub(crate) fn find_wheels(mut commands: Commands, names: Query<(Entity, &Name), Added<Name>>) {
    for (e, n) in &names {
        if n.as_str() == "WerkRad" {
            commands.entity(e).insert(Millwheel);
        }
    }
}
pub(crate) fn turn_wheels(
    game: Res<Game>,
    play: Res<Play>,
    time: Res<Time>,
    mut wheels: Query<&mut Transform, With<Millwheel>>,
) {
    if play.paused
        || wheels.is_empty()
        || game.city().work.stage != WorkStage::Running
        || !game.city().work_partner(game.region())
    {
        return;
    }
    for mut pose in &mut wheels {
        pose.rotate_local_x(time.delta_secs().min(0.18) * 0.75);
    }
}
