use bevy::{
    core_pipeline::{fxaa::Fxaa, tonemapping::Tonemapping},
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    render::camera::ScalingMode,
};

use crate::{
    art::{self, Art, block, mound, part},
    game::{Building, Crop, Game, Miracle, Quest},
    play::{CameraRig, Panel, Play, Target, Tool},
    region::{Biome, CELL, Material},
};

#[derive(Component)]
pub struct WorldCamera;

#[derive(Resource, Default)]
pub struct Board {
    active: Option<usize>,
    root: Option<Entity>,
    bridge: Option<(bool, Entity)>,
    boundary: Option<((bool, bool), Entity)>,
    buildings: usize,
    walkers: usize,
    seen_magic: u64,
}

#[derive(Component)]
pub struct SourceView(usize);

#[derive(Component)]
pub struct GuardView(usize);

#[derive(Component)]
pub struct RivalFlag;

#[derive(Component)]
pub struct PlotMarker;

#[derive(Component)]
pub struct CropView(usize);

#[derive(Component)]
pub struct WalkerView {
    index: usize,
    speed: f32,
    phase: f32,
}

#[derive(Component)]
pub struct Limb {
    walker: usize,
    side: f32,
    arm: bool,
    origin: Vec3,
}

#[derive(Component)]
pub enum CarryView {
    Hand(Material),
    Actor(usize, Material),
}

#[derive(Component)]
pub struct Cloud;

#[derive(Component)]
pub struct GroundRing;

#[derive(Clone, Copy, Component)]
pub enum Blueprint {
    Base,
    Roof,
    Corners,
    Obstruction,
}

#[derive(Component)]
pub struct WeatherParticle {
    origin: Vec3,
    drift: f32,
}

#[derive(Component)]
pub struct Effect {
    pub age: f32,
    pub length: f32,
    pub drift: Vec3,
    pub scale: f32,
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
    view: Res<CameraRig>,
) {
    let art = Art::load(&mut meshes, &mut materials, &assets);
    commands.spawn((
        Name::new("Spielkamera"),
        WorldCamera,
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: view.height,
            },
            near: 0.01,
            far: 130.,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_translation(view.target + Vec3::new(13., 22., 26.))
            .looking_at(view.target, Vec3::Y),
        Msaa::Off,
        Fxaa::default(),
        Tonemapping::None,
    ));
    commands.insert_resource(AmbientLight {
        color: art::rgb(0xffebca),
        brightness: 850.,
        ..default()
    });
    commands.spawn((
        Name::new("Weiches Sonnenlicht"),
        DirectionalLight {
            color: art::rgb(0xffefd8),
            illuminance: 2350.,
            shadows_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 54.,
            ..default()
        }
        .build(),
        Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, -0.85, -0.93, 0.)),
    ));

    commands
        .spawn((Cloud, Transform::default(), Visibility::Visible))
        .with_children(|parent| {
            for (x, y, z, s) in [
                (-0.38, 0.22, 0., 0.62),
                (0.02, 0.37, -0.02, 0.73),
                (0.40, 0.18, 0.02, 0.59),
                (0.10, 0.09, 0.20, 0.65),
            ] {
                mound(
                    parent,
                    &art,
                    &art.white,
                    Vec3::new(x, y, z),
                    Vec3::new(s, s * 0.81, s * 0.87),
                );
            }
            let paw = parent
                .spawn((
                    Transform::from_rotation(
                        Quat::from_rotation_x(-0.38) * Quat::from_rotation_z(-0.20),
                    ),
                    Visibility::Visible,
                ))
                .id();
            parent.commands().entity(paw).with_children(|digits| {
                mound(
                    digits,
                    &art,
                    &art.purple,
                    Vec3::new(0.08, -0.18, 0.15),
                    Vec3::new(0.55, 0.47, 0.24),
                );
                for x in [-0.12, 0.04, 0.2] {
                    mound(
                        digits,
                        &art,
                        &art.boot,
                        Vec3::new(x, -0.38, 0.20),
                        Vec3::new(0.16, 0.27, 0.19),
                    );
                }
            });
            for material in Material::ALL {
                parent
                    .spawn((
                        CarryView::Hand(material),
                        Transform::default(),
                        Visibility::Hidden,
                    ))
                    .with_children(|carry| {
                        art::parcel(carry, &art, material, Vec3::new(0.08, -0.64, 0.31), 0.79)
                    });
            }
        });
    commands.spawn((
        GroundRing,
        Mesh3d(art.ring.clone()),
        MeshMaterial3d(art.gold.clone()),
        Transform::default(),
        Visibility::Visible,
    ));
    for (layer, mesh) in [
        (Blueprint::Base, &art.cube),
        (Blueprint::Roof, &art.roof),
        (Blueprint::Corners, &art.plan_corners),
        (Blueprint::Obstruction, &art.plan_cross),
    ] {
        commands.spawn((
            layer,
            Mesh3d(mesh.clone()),
            MeshMaterial3d(art.ghost.clone()),
            Transform::default(),
            Visibility::Hidden,
            bevy::pbr::NotShadowCaster,
        ));
    }
    commands.insert_resource(art);
}

pub fn move_camera(
    time: Res<Time>,
    view: Res<CameraRig>,
    mut camera: Single<(&mut Transform, &mut Projection), With<WorldCamera>>,
) {
    let (pose, projection) = &mut *camera;
    let follow = 1. - (-13. * time.delta_secs()).exp();
    let desired = view.target + Vec3::new(13., 22., 26.);
    pose.translation = pose.translation.lerp(desired, follow);
    pose.look_to(Vec3::new(-13., -22., -26.), Vec3::Y);
    if let Projection::Orthographic(ortho) = &mut **projection {
        let ScalingMode::FixedVertical {
            ref mut viewport_height,
        } = ortho.scaling_mode
        else {
            return;
        };
        *viewport_height += (view.height - *viewport_height) * follow;
    }
}

fn contact_shadow(parent: &mut ChildSpawnerCommands, art: &Art, size: f32) {
    mound(
        parent,
        art,
        &art.shadow,
        Vec3::new(0.05, 0.025, -0.05),
        Vec3::new(size, 0.025, size * 0.78),
    );
}

fn villager(parent: &mut ChildSpawnerCommands, art: &Art, guard: bool) {
    let coat = if guard { &art.blue } else { &art.door };
    mound(
        parent,
        art,
        coat,
        Vec3::Y * 0.62,
        Vec3::new(0.58, 0.61, 0.41),
    );
    mound(
        parent,
        art,
        &art.skin,
        Vec3::new(0., 0.99, 0.06),
        Vec3::new(0.56, 0.51, 0.43),
    );
    for x in [-0.12, 0.12] {
        mound(
            parent,
            art,
            &art.eye,
            Vec3::new(x, 1.01, 0.247),
            Vec3::splat(0.055),
        );
    }
    let hat = if guard { &art.blue } else { &art.plank };
    part(
        parent,
        &art.cylinder,
        hat,
        Vec3::Y * 1.19,
        Vec3::new(0.76, 0.087, 0.67),
        Quat::IDENTITY,
    );
    mound(
        parent,
        art,
        hat,
        Vec3::Y * 1.27,
        Vec3::new(0.48, 0.23, 0.44),
    );
    if guard {
        block(
            parent,
            art,
            &art.timber,
            Vec3::new(0.35, 0.65, 0.15),
            Vec3::new(0.063, 1.32, 0.063),
        );
        block(
            parent,
            art,
            &art.blue,
            Vec3::new(0.48, 1.21, 0.15),
            Vec3::new(0.26, 0.29, 0.05),
        );
        for x in [-0.15, 0.15] {
            mound(
                parent,
                art,
                &art.dark_wood,
                Vec3::new(x, 0.20, 0.045),
                Vec3::new(0.24, 0.42, 0.30),
            );
        }
    }
}

fn spawn_walker(
    parent: &mut ChildSpawnerCommands,
    art: &Art,
    index: usize,
    hippo: bool,
    point: Vec3,
) {
    let scale = if hippo { 1.46 } else { 0.90 };
    parent
        .spawn((
            WalkerView {
                index,
                speed: 0.,
                phase: 0.,
            },
            Transform::from_translation(point)
                .with_scale(Vec3::splat(scale))
                .with_rotation(Quat::from_rotation_y(0.44)),
            Visibility::Visible,
        ))
        .with_children(|actor| {
            contact_shadow(actor, art, if hippo { 1.26 } else { 0.82 });
            if hippo {
                actor.spawn((SceneRoot(art.hippo_model.clone()), Transform::default()));
            } else {
                villager(actor, art, false);
            }
            for side in [-1., 1.] {
                let leg_origin = Vec3::new(side * if hippo { 0.18 } else { 0.13 }, 0.40, 0.);
                let arm_origin = Vec3::new(
                    side * if hippo { 0.34 } else { 0.25 },
                    if hippo { 0.92 } else { 0.75 },
                    0.065,
                );
                actor
                    .spawn((
                        Limb {
                            walker: index,
                            side,
                            arm: false,
                            origin: leg_origin,
                        },
                        Transform::from_translation(leg_origin),
                        Visibility::Visible,
                    ))
                    .with_children(|leg| {
                        let mat = if hippo { &art.purple } else { &art.dark_wood };
                        mound(
                            leg,
                            art,
                            mat,
                            Vec3::new(0., -0.18, 0.),
                            Vec3::new(0.24, 0.41, 0.25),
                        );
                        let boot = if hippo { &art.boot } else { &art.timber };
                        mound(
                            leg,
                            art,
                            boot,
                            Vec3::new(0., -0.31, 0.084),
                            Vec3::new(0.28, 0.17, 0.36),
                        );
                    });
                actor
                    .spawn((
                        Limb {
                            walker: index,
                            side,
                            arm: true,
                            origin: arm_origin,
                        },
                        Transform::from_translation(arm_origin),
                        Visibility::Visible,
                    ))
                    .with_children(|arm| {
                        let mat = if hippo { &art.purple } else { &art.skin };
                        mound(
                            arm,
                            art,
                            mat,
                            Vec3::new(0., -0.16, 0.),
                            Vec3::new(0.22, 0.40, 0.22),
                        );
                        mound(
                            arm,
                            art,
                            mat,
                            Vec3::new(0., -0.30, 0.02),
                            Vec3::new(0.19, 0.18, 0.20),
                        );
                    });
            }
            for material in Material::ALL {
                actor
                    .spawn((
                        CarryView::Actor(index, material),
                        Transform::default(),
                        Visibility::Hidden,
                    ))
                    .with_children(|carry| {
                        art::parcel(
                            carry,
                            art,
                            material,
                            Vec3::new(0., 0.77, 0.40),
                            if hippo { 0.86 } else { 0.61 },
                        )
                    });
            }
        });
}

pub fn sync_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    art: Res<Art>,
    game: Res<Game>,
    mut board: ResMut<Board>,
) {
    let current = game.campaign.active;
    let region = game.region();
    let city = game.city();
    if board.active != Some(current) {
        if let Some(root) = board.root {
            commands.entity(root).despawn();
        }
        let ground = meshes.add(art::landscape(region));
        let root = commands
            .spawn((
                Name::new(region.name.clone()),
                Transform::default(),
                Visibility::Visible,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Mesh3d(ground),
                    MeshMaterial3d(art.ground.clone()),
                    Transform::default(),
                ));
                block(
                    parent,
                    &art,
                    &art.clay,
                    Vec3::new(0., -0.64, 0.),
                    Vec3::new(78., 0.57, 67.),
                );
                art::scenery(parent, &art, region);
                for (i, source) in region.sources.iter().enumerate() {
                    parent
                        .spawn((
                            SourceView(i),
                            Transform::from_translation(source.tile.world()),
                            Visibility::Visible,
                        ))
                        .with_children(|p| {
                            contact_shadow(p, &art, 1.15);
                            art::resources(p, &art, source.material);
                        });
                }
                for p in [region.start, region.rival_shrine] {
                    parent
                        .spawn((Transform::from_translation(p.world()), Visibility::Visible))
                        .with_children(|p| art::shrine(p, &art, false));
                }
                parent
                    .spawn((
                        Transform::from_translation(region.depot.world()),
                        Visibility::Visible,
                    ))
                    .with_children(|p| {
                        block(
                            p,
                            &art,
                            &art.plank,
                            Vec3::Y * 0.10,
                            Vec3::new(1.47, 0.19, 1.35),
                        );
                        for x in [-0.39, 0.39] {
                            block(
                                p,
                                &art,
                                &art.gold,
                                Vec3::new(x, 0.21, 0.57),
                                Vec3::new(0.31, 0.045, 0.08),
                            );
                        }
                        art::parcel(p, &art, Material::Food, Vec3::new(0., 0.25, 0.), 0.8);
                    });
                for tile in region.rival_buildings() {
                    parent
                        .spawn((
                            Transform::from_translation(tile.world()).with_scale(Vec3::splat(0.91)),
                            Visibility::Visible,
                        ))
                        .with_children(|p| {
                            art::building(p, &art, region.kind, Building::House);
                            block(
                                p,
                                &art,
                                &art.timber,
                                Vec3::new(1.39, 1.06, 0.99),
                                Vec3::new(0.08, 2.1, 0.08),
                            );
                            let flag = block(
                                p,
                                &art,
                                &art.blue,
                                Vec3::new(1.75, 1.73, 0.99),
                                Vec3::new(0.68, 0.54, 0.056),
                            );
                            p.commands().entity(flag).insert(RivalFlag);
                        });
                }
                for (i, tile) in region.guards.iter().enumerate() {
                    parent
                        .spawn((
                            GuardView(i),
                            Transform::from_translation(tile.world()),
                            Visibility::Visible,
                        ))
                        .with_children(|p| {
                            contact_shadow(p, &art, 0.76);
                            villager(p, &art, true);
                        });
                }
                parent.spawn((
                    PlotMarker,
                    Mesh3d(art.ring.clone()),
                    MeshMaterial3d(art.gold.clone()),
                    Transform::from_translation(region.frontier.world() + Vec3::Y * 0.055)
                        .with_scale(Vec3::splat(0.68)),
                    Visibility::Hidden,
                ));
                if region.kind == Biome::Fjord {
                    for i in 0..44 {
                        let p = region.start.world()
                            + Vec3::new(
                                (i * 17 % 62) as f32 * 0.51 - 15.,
                                0.,
                                (i * 23 % 58) as f32 * 0.54 - 16.,
                            );
                        let flake = mound(parent, &art, &art.white, p, Vec3::splat(0.078));
                        parent.commands().entity(flake).insert((
                            WeatherParticle {
                                origin: p,
                                drift: (i % 9) as f32,
                            },
                            bevy::pbr::NotShadowCaster,
                        ));
                    }
                }
            })
            .id();
        *board = Board {
            active: Some(current),
            root: Some(root),
            ..default()
        };
    }

    let root = board.root.unwrap();
    let flags = (city.count(Building::Shrine) > 0, city.outcome.is_some());
    if board
        .boundary
        .as_ref()
        .is_none_or(|(previous, _)| *previous != flags)
    {
        if let Some((_, entity)) = board.boundary.take() {
            commands.entity(entity).despawn();
        }
        let perimeter = meshes.add(art::influence_mesh(city, region));
        let entity = commands
            .spawn((
                ChildOf(root),
                Mesh3d(perimeter),
                MeshMaterial3d(art.safe_zone.clone()),
                Transform::default(),
                bevy::pbr::NotShadowCaster,
            ))
            .id();
        board.boundary = Some((flags, entity));
    }
    if board
        .bridge
        .as_ref()
        .is_none_or(|(fixed, _)| *fixed != city.bridge_fixed)
    {
        if let Some((_, entity)) = board.bridge.take() {
            commands.entity(entity).despawn();
        }
        let entity = commands
            .spawn((ChildOf(root), Transform::default(), Visibility::Visible))
            .with_children(|parent| art::bridge(parent, &art, region, city.bridge_fixed))
            .id();
        board.bridge = Some((city.bridge_fixed, entity));
    }
    for (index, site) in city.buildings.iter().enumerate().skip(board.buildings) {
        commands
            .spawn((
                ChildOf(root),
                Transform::from_translation(site.kind.center(site.tile)).with_rotation(
                    Quat::from_rotation_y(f32::from(site.rotation) * std::f32::consts::FRAC_PI_2),
                ),
                Visibility::Visible,
            ))
            .with_children(|parent| {
                art::building(parent, &art, region.kind, site.kind);
                if site.kind == Building::Field {
                    parent
                        .spawn((CropView(index), Transform::default(), Visibility::Hidden))
                        .with_children(|plants| {
                            for x in [-0.96, -0.32, 0.32, 0.96] {
                                for z in [-0.91, -0.30, 0.30, 0.91] {
                                    let point = Vec3::new(x, 0.28, z);
                                    mound(
                                        plants,
                                        &art,
                                        &art.leaf[2],
                                        point,
                                        Vec3::new(0.32, 0.38, 0.29),
                                    );
                                    mound(
                                        plants,
                                        &art,
                                        &art.moss,
                                        point + Vec3::Y * 0.15,
                                        Vec3::new(0.24, 0.21, 0.22),
                                    );
                                }
                            }
                        });
                }
            });
    }
    board.buildings = city.buildings.len();
    for (index, walker) in city.walkers.iter().enumerate().skip(board.walkers) {
        commands.entity(root).with_children(|parent| {
            spawn_walker(parent, &art, index, walker.hippo, walker.world())
        });
    }
    board.walkers = city.walkers.len();
}

pub fn props(
    game: Res<Game>,
    art: Res<Art>,
    mut visuals: Query<
        (
            &mut Visibility,
            Option<&SourceView>,
            Option<&GuardView>,
            Option<&CropView>,
            Option<&PlotMarker>,
            Option<&CarryView>,
            Option<&mut Transform>,
        ),
        Or<(
            With<SourceView>,
            With<GuardView>,
            With<CropView>,
            With<PlotMarker>,
            With<CarryView>,
        )>,
    >,
    mut flags: Query<&mut MeshMaterial3d<StandardMaterial>, With<RivalFlag>>,
) {
    let region = game.region();
    let city = game.city();
    for (mut visible, source, guard, crop, marker, carry, pose) in &mut visuals {
        let show = if let Some(source) = source {
            city.nodes.get(source.0).is_some_and(|&amount| amount > 0)
        } else if let Some(guard) = guard {
            city.guards.get(guard.0).is_some_and(|&spirit| spirit > 0)
        } else if let Some(crop) = crop {
            let scale = match city.buildings.get(crop.0).map(|site| site.crop) {
                Some(Crop::Growing(remaining)) => (1. - remaining / 11.).clamp(0.24, 0.98),
                Some(Crop::Ripe) => 1.,
                _ => 0.,
            };
            if let Some(mut p) = pose {
                p.scale.y = scale.max(0.02);
            }
            scale > 0.
        } else if marker.is_some() {
            city.quest == Quest::Bridge && city.count(Building::Shrine) == 0
        } else if let Some(carry) = carry {
            let (bundle, material) = match carry {
                CarryView::Hand(material) => (city.held, *material),
                CarryView::Actor(index, material) => (
                    city.walkers.get(*index).and_then(|actor| actor.bundle),
                    *material,
                ),
            };
            bundle.is_some_and(|bundle| region.sources[bundle.source].material == material)
        } else {
            false
        };
        *visible = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for mut flag in &mut flags {
        flag.0 = if city.outcome.is_some() {
            art.door.clone()
        } else {
            art.blue.clone()
        };
    }
}

pub fn animate(
    time: Res<Time>,
    game: Res<Game>,
    mut actors: Query<(&mut WalkerView, &mut Transform), Without<Limb>>,
    mut limbs: Query<(&Limb, &mut Transform), Without<WalkerView>>,
) {
    let delta = time.delta_secs().max(0.0001);
    let mut motion = [(0., 0.); 7];
    for (mut meta, mut pose) in &mut actors {
        let Some(walker) = game.city().walkers.get(meta.index) else {
            continue;
        };
        let goal = walker.world();
        let offset = goal - pose.translation.with_y(0.);
        let blend = 1. - (-16. * delta).exp();
        let speed = (offset.length() * blend / delta / 2.8).clamp(0., 1.);
        meta.speed += (speed - meta.speed) * blend;
        meta.phase += delta * (6.9 * meta.speed + 0.6);
        if offset.length_squared() > 0.0006 {
            let facing = Quat::from_rotation_y(offset.x.atan2(offset.z));
            pose.rotation = pose.rotation.slerp(facing, (delta * 10.).min(1.));
        }
        let bob = meta.phase.sin().abs() * 0.055 * meta.speed;
        pose.translation = (pose.translation + offset * blend).with_y(bob);
        if let Some(state) = motion.get_mut(meta.index) {
            *state = (meta.phase, meta.speed);
        }
    }
    for (limb, mut pose) in &mut limbs {
        let Some(walker) = game.city().walkers.get(limb.walker) else {
            continue;
        };
        let (phase, speed) = motion[limb.walker];
        let swing = phase.sin() * limb.side * speed;
        let carrying = limb.arm && walker.bundle.is_some();
        let angle = if carrying {
            -0.92
        } else if limb.arm {
            -0.46 * swing
        } else {
            0.57 * swing
        };
        pose.rotation = Quat::from_rotation_x(angle);
        pose.translation = limb.origin;
        if !limb.arm {
            pose.translation.y += (swing * 0.11).max(0.);
        }
    }
}

pub fn aim(
    time: Res<Time>,
    game: Res<Game>,
    play: Res<Play>,
    target: Res<Target>,
    art: Res<Art>,
    mut cloud: Single<
        (&mut Transform, &mut Visibility),
        (With<Cloud>, Without<GroundRing>, Without<Blueprint>),
    >,
    mut ring: Single<
        (
            &mut Transform,
            &mut Visibility,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        (With<GroundRing>, Without<Cloud>, Without<Blueprint>),
    >,
    mut previews: Query<
        (
            &Blueprint,
            &mut Transform,
            &mut Visibility,
            &mut Mesh3d,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        (Without<Cloud>, Without<GroundRing>),
    >,
) {
    let open = play.panel == Panel::Closed && play.aim.inside_map();
    let p = play.aim.world();
    let (cloud_pose, cloud_visible) = &mut *cloud;
    cloud_pose.translation = p + Vec3::new(0.18, 1.56 + time.elapsed_secs().sin() * 0.065, -0.23);
    cloud_pose.rotation = Quat::from_rotation_y(0.51);
    cloud_pose.scale = Vec3::splat(1.17);
    **cloud_visible = if open {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    let (ring_pose, ring_visible, ring_surface) = &mut *ring;
    ring_pose.translation = p + Vec3::Y * 0.067;
    ring_pose.scale = Vec3::splat(if play.tool == Tool::Miracles {
        f32::from(play.miracle().radius(game.campaign.gifts)) * CELL
    } else {
        0.74
    });
    ring_surface.0 = if target.legal {
        art.gold.clone()
    } else {
        art.invalid.clone()
    };
    **ring_visible = if open && play.tool != Tool::Build {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for (layer, mut pose, mut visible, mut mesh, mut surface) in &mut previews {
        let kind = play.building();
        let enabled = match layer {
            Blueprint::Roof => !matches!(kind, Building::Field | Building::Shrine),
            Blueprint::Obstruction => !target.legal,
            _ => true,
        };
        *visible = if open && play.tool == Tool::Build && enabled {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visible == Visibility::Hidden {
            continue;
        }
        let center = kind.center(play.aim);
        let rotation = Quat::from_rotation_y(play.rotation as f32 * std::f32::consts::FRAC_PI_2);
        pose.rotation = rotation;
        surface.0 = if target.legal {
            art.ghost.clone()
        } else {
            art.blocked.clone()
        };
        match layer {
            Blueprint::Base => {
                let size = if kind == Building::Shrine {
                    Vec3::new(1.35, 1.54, 1.32)
                } else if kind == Building::Field {
                    Vec3::new(3.25, 0.18, 3.21)
                } else {
                    Vec3::new(2.84, 1.45, 2.64)
                };
                pose.translation = center + Vec3::Y * size.y * 0.5;
                pose.scale = size;
            }
            Blueprint::Roof => {
                pose.translation = center + Vec3::Y * 1.46;
                pose.scale = Vec3::new(3.0, 1.14, 2.75);
                mesh.0 = if game.region().kind == Biome::Terraces {
                    art.curved_roof.clone()
                } else {
                    art.roof.clone()
                };
            }
            Blueprint::Corners | Blueprint::Obstruction => {
                pose.translation = center + Vec3::Y * 0.095;
                pose.scale = Vec3::splat(CELL * f32::from(kind.size()));
                surface.0 = if target.legal {
                    art.valid.clone()
                } else {
                    art.invalid.clone()
                };
            }
        }
    }
}

pub fn weather(
    time: Res<Time>,
    mut ambient: ResMut<AmbientLight>,
    game: Res<Game>,
    mut flakes: Query<(&WeatherParticle, &mut Transform)>,
) {
    ambient.brightness = if game.city().rainfall > 0. {
        740.
    } else {
        850.
    };
    for (flake, mut pose) in &mut flakes {
        let t = game.city().weather_time;
        let h = 6.5 - (t * 0.77 + flake.drift * 0.73).rem_euclid(6.5);
        pose.translation = flake.origin
            + Vec3::new(
                (t * 0.60 + flake.drift).sin() * 0.7,
                h,
                (t * 0.3).cos() * 0.61,
            );
        pose.rotate_y(time.delta_secs() * 0.26);
    }
}

pub fn magic(
    mut commands: Commands,
    time: Res<Time>,
    play: Res<Play>,
    game: Res<Game>,
    art: Res<Art>,
    mut board: ResMut<Board>,
    mut effects: Query<(Entity, &mut Effect, &mut Transform)>,
) {
    if let Some((spell, tile, revision)) = play.last_magic {
        if revision != board.seen_magic {
            board.seen_magic = revision;
            let mat = match spell {
                Miracle::Rain => &art.ripple,
                Miracle::Wind => &art.white,
                Miracle::Ward => &art.valid,
            };
            let radius = f32::from(spell.radius(game.campaign.gifts)) * CELL;
            if let Some(root) = board.root {
                for i in 0..if spell == Miracle::Rain { 34 } else { 20 } {
                    let angle = i as f32 * 2.39996;
                    let outward = Vec3::new(angle.cos(), 0., angle.sin());
                    let frac = (i as f32 * 0.618).fract();
                    let start = tile.world()
                        + outward * radius * frac.sqrt()
                        + Vec3::Y
                            * if spell == Miracle::Rain {
                                1.5 + frac * 3.4
                            } else {
                                0.25 + frac
                            };
                    let drift = if spell == Miracle::Rain {
                        Vec3::new(0.1, -1.9, 0.05)
                    } else {
                        outward * (1.4 + frac) + Vec3::Y * 0.33
                    };
                    let length = if spell == Miracle::Rain {
                        0.67 + frac * 1.2
                    } else {
                        0.7 + frac * 0.34
                    };
                    commands.spawn((
                        ChildOf(root),
                        Effect {
                            age: 0.,
                            length,
                            drift,
                            scale: if spell == Miracle::Rain { 0.085 } else { 0.25 },
                        },
                        Mesh3d(art.lump.clone()),
                        MeshMaterial3d(mat.clone()),
                        Transform::from_translation(start),
                        bevy::pbr::NotShadowCaster,
                    ));
                }
            }
        }
    }
    for (entity, mut effect, mut pose) in &mut effects {
        effect.age += time.delta_secs();
        if effect.age >= effect.length || pose.translation.y < 0.04 {
            commands.entity(entity).despawn();
            continue;
        }
        pose.translation += effect.drift * time.delta_secs();
        let scale = effect.scale * (1. - effect.age / effect.length).sqrt();
        pose.scale = Vec3::new(
            scale,
            scale * if effect.drift.y < 0. { 3.8 } else { 0.41 },
            scale,
        );
    }
}
