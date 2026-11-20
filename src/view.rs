use bevy::{
    core_pipeline::{fxaa::Fxaa, tonemapping::Tonemapping},
    pbr::DistanceFog,
    prelude::*,
    render::camera::ScalingMode,
};

use crate::{
    art::{self, Art, block, mound, part},
    game::{Building, Crop, Game, MAX_ROAD_RUN, MAX_WALKERS, Miracle, Quest, Walker},
    play::{CameraRig, Panel, Play, Target, Tool},
    region::{Biome, CELL, Material, Tile},
    terrain,
};

#[derive(Component)]
pub struct WorldCamera;

#[derive(Resource, Default)]
pub struct Board {
    active: Option<usize>,
    root: Option<Entity>,
    bridge: Option<(bool, Entity)>,
    boundary: Option<([bool; 5], Entity)>,
    boundary_anchors: usize,
    roads: Option<(usize, Entity)>,
    buildings: usize,
    walkers: usize,
    seen_magic: u64,
}

#[derive(Component)]
pub struct SourceView(usize);

#[derive(Component)]
pub struct GuardView(usize);

#[derive(Component)]
pub struct RivalFlag(Option<usize>);

#[derive(Component)]
pub struct UpgradeView(usize);

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
pub struct GroundRing(Option<(usize, Tile, i16, bool)>);

#[derive(Clone, Copy, Component)]
pub enum Blueprint {
    Base,
    Roof,
    Corners,
    Obstruction,
    Road(usize),
}

#[derive(Component)]
pub struct Effect {
    age: f32,
    length: f32,
    velocity: Vec3,
    scale: f32,
    phase: EffectPhase,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum EffectPhase {
    Rising,
    Falling,
    Splash,
    Flash,
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
            far: 410.,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_translation(view.target + Vec3::new(13., 22., 26.))
            .looking_at(view.target, Vec3::Y),
        Msaa::Off,
        Fxaa::default(),
        Tonemapping::None,
        DistanceFog::default(),
    ));
    crate::atmosphere::setup(&mut commands, &mut materials, &art);

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
        GroundRing(None),
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
    for index in 0..usize::from(MAX_ROAD_RUN) + 2 {
        commands.spawn((
            Blueprint::Road(index),
            Mesh3d(art.plan_corners.clone()),
            MeshMaterial3d(art.valid.clone()),
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
    let desired = view.target + Vec3::new(13., 22., 26.) * (view.height / 22.).max(1.);
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
    walker: &Walker,
    ground: f32,
) {
    let hippo = walker.hippo;
    let scale = actor_scale(walker);
    parent
        .spawn((
            WalkerView {
                index,
                speed: 0.,
                phase: 0.,
            },
            Transform::from_translation(walker.world().with_y(ground))
                .with_scale(Vec3::splat(scale))
                .with_rotation(Quat::from_rotation_y(0.44)),
            Visibility::Visible,
        ))
        .with_children(|actor| {
            contact_shadow(actor, art, if hippo { 1.26 } else { 0.82 });
            if hippo {
                actor.spawn((SceneRoot(art.hippo_model.clone()), Transform::default()));
            } else {
                villager(actor, art, walker.guard);
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

fn actor_scale(walker: &Walker) -> f32 {
    if walker.hippo {
        1.46
    } else if walker.guard {
        1.0
    } else if walker.childhood > 0. {
        0.62 + 0.26 * (1. - (walker.childhood / 46.).min(1.))
    } else {
        0.90
    }
}

pub fn sync_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    art: Res<Art>,
    game: Res<Game>,
    mut board: ResMut<Board>,
    scenery: Query<(Entity, &art::LandscapeProp)>,
) {
    let current = game.campaign.active;
    let region = game.region();
    let city = game.city();
    if board.active != Some(current) {
        if let Some(root) = board.root {
            commands.entity(root).despawn();
        }
        let grounded = |tile: Tile| terrain::on_ground(region, tile.world());
        let root = commands
            .spawn((
                Name::new(region.name.clone()),
                Transform::default(),
                Visibility::Visible,
            ))
            .with_children(|parent| {
                art::landscape(parent, &mut meshes, &art, region, city);
                for (i, source) in region.sources.iter().enumerate() {
                    parent
                        .spawn((
                            SourceView(i),
                            Transform::from_translation(grounded(source.tile)),
                            Visibility::Visible,
                        ))
                        .with_children(|p| {
                            contact_shadow(p, &art, 1.15);
                            art::resources(p, &art, source.material);
                        });
                }
                for p in [region.start, region.rival_shrine]
                    .into_iter()
                    .chain(region.districts.iter().map(|district| district.center))
                {
                    parent
                        .spawn((
                            Transform::from_translation(grounded(p)),
                            Visibility::Visible,
                        ))
                        .with_children(|p| {
                            if region.kind == Biome::Aegean {
                                art::building(p, &art, region.kind, Building::Shrine);
                            } else {
                                art::shrine(p, &art, false);
                            }
                        });
                }
                for (index, district) in region.districts.iter().enumerate() {
                    for tile in region.district_houses(index) {
                        parent
                            .spawn((
                                Transform::from_translation(grounded(tile))
                                    .with_scale(Vec3::splat(0.92)),
                                Visibility::Visible,
                            ))
                            .with_children(|p| {
                                art::building(p, &art, region.kind, Building::House)
                            });
                    }
                    let p = grounded(district.center);
                    parent.spawn((
                        RivalFlag(Some(index)),
                        Mesh3d(art.ring.clone()),
                        MeshMaterial3d(art.blue.clone()),
                        Transform::from_translation(p + Vec3::Y * 0.055)
                            .with_scale(Vec3::splat(1.55)),
                    ));
                }
                parent
                    .spawn((
                        Transform::from_translation(grounded(region.depot)),
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
                            Transform::from_translation(grounded(tile))
                                .with_scale(Vec3::splat(0.91)),
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
                            p.commands().entity(flag).insert(RivalFlag(None));
                        });
                }
                for (i, &tile) in region.guards.iter().enumerate() {
                    parent
                        .spawn((
                            GuardView(i),
                            Transform::from_translation(grounded(tile)),
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
                    Transform::from_translation(grounded(region.frontier) + Vec3::Y * 0.055)
                        .with_scale(Vec3::splat(0.68)),
                    Visibility::Hidden,
                ));
            })
            .id();
        *board = Board {
            active: Some(current),
            root: Some(root),
            ..default()
        };
    }

    let root = board.root.unwrap();
    let flags = [
        city.count(Building::Shrine) > 0,
        city.outcome.is_some(),
        city.city_rank() == 4,
        city.districts[0].outcome.is_some(),
        city.districts[1].outcome.is_some(),
    ];
    if board.boundary.as_ref().is_none_or(|(previous, _)| {
        *previous != flags || board.boundary_anchors != city.count(Building::Waystone)
    }) {
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
        board.boundary_anchors = city.count(Building::Waystone);
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
            .with_children(|parent| {
                art::bridge(parent, &art, region, region.cross_x, city.bridge_fixed);
                for column in region.crossings {
                    art::bridge(parent, &art, region, column, city.bridge_fixed);
                }
            })
            .id();
        board.bridge = Some((city.bridge_fixed, entity));
    }
    if !city.roads.is_empty()
        && board
            .roads
            .is_none_or(|(count, _)| count != city.roads.len())
    {
        if let Some((_, previous)) = board.roads {
            commands.entity(previous).despawn();
        }
        let entity = commands
            .spawn((
                ChildOf(root),
                Mesh3d(meshes.add(art::paved_roads(city, region))),
                MeshMaterial3d(art.ground.clone()),
                Transform::default(),
                bevy::pbr::NotShadowCaster,
            ))
            .id();
        board.roads = Some((city.roads.len(), entity));
    }
    for (index, site) in city.buildings.iter().enumerate().skip(board.buildings) {
        for (entity, spot) in &scenery {
            if site.kind.tiles(site.tile).any(|tile| tile == spot.0) {
                commands.entity(entity).try_despawn();
            }
        }
        commands
            .spawn((
                ChildOf(root),
                Transform::from_translation(terrain::on_ground(
                    region,
                    site.kind.center(site.tile),
                ))
                .with_rotation(Quat::from_rotation_y(
                    f32::from(site.rotation) * std::f32::consts::FRAC_PI_2,
                )),
                Visibility::Visible,
            ))
            .with_children(|parent| {
                art::building(parent, &art, region.kind, site.kind);
                if site.kind.upgrade_price().is_some() {
                    parent
                        .spawn((UpgradeView(index), Transform::default(), Visibility::Hidden))
                        .with_children(|detail| {
                            art::upgrade_detail(detail, &art, site.kind);
                        });
                }
                if site.kind == Building::Field {
                    parent
                        .spawn((CropView(index), Transform::default(), Visibility::Hidden))
                        .with_children(|plants| {
                            for (i, x) in [-0.96, -0.32, 0.32, 0.96].into_iter().enumerate() {
                                for (j, z) in [-0.87, -0.28, 0.28, 0.87].into_iter().enumerate() {
                                    art::place_scene(
                                        plants,
                                        &art.waldwinkel.crop,
                                        Vec3::new(x, 0.16, z),
                                        0.60,
                                        (i * 19 + j * 11) as f32 * 0.29,
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
            spawn_walker(
                parent,
                &art,
                index,
                walker,
                terrain::actor_height(region, walker.world(), city.bridge_fixed),
            )
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
            Option<&UpgradeView>,
            Option<&mut Transform>,
        ),
        Or<(
            With<SourceView>,
            With<GuardView>,
            With<CropView>,
            With<PlotMarker>,
            With<CarryView>,
            With<UpgradeView>,
        )>,
    >,
    mut flags: Query<(&RivalFlag, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    let region = game.region();
    let city = game.city();
    for (mut visible, source, guard, crop, marker, carry, upgraded, pose) in &mut visuals {
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
        } else if let Some(view) = upgraded {
            city.buildings.get(view.0).is_some_and(|site| site.upgraded)
        } else {
            false
        };
        *visible = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (flag, mut surface) in &mut flags {
        let outcome = flag.0.map_or(city.outcome, |i| city.districts[i].outcome);
        surface.0 = match outcome {
            Some(crate::game::Outcome::Alliance) => art.valid.clone(),
            Some(crate::game::Outcome::Victory) => art.amber.clone(),
            None => art.blue.clone(),
        }
    }
}

pub fn animate(
    time: Res<Time>,
    game: Res<Game>,
    mut actors: Query<(&mut WalkerView, &mut Transform), Without<Limb>>,
    mut limbs: Query<(&Limb, &mut Transform), Without<WalkerView>>,
) {
    let delta = time.delta_secs().max(0.0001);
    let city = game.city();
    let region = game.region();
    let mut motion = [(0., 0.); MAX_WALKERS];
    for (mut meta, mut pose) in &mut actors {
        let Some(walker) = game.city().walkers.get(meta.index) else {
            continue;
        };
        let goal = walker.world().with_y(0.);
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
        let next = pose.translation + offset * blend;
        let ground = terrain::actor_height(region, next, city.bridge_fixed);
        pose.translation = next.with_y(ground + bob);
        pose.scale = Vec3::splat(actor_scale(walker));
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut cloud: Single<
        (&mut Transform, &mut Visibility),
        (With<Cloud>, Without<GroundRing>, Without<Blueprint>),
    >,
    mut ring: Single<
        (
            &mut GroundRing,
            &mut Mesh3d,
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
    let region = game.region();
    let city = game.city();
    let open = play.panel == Panel::Closed && region.bounds.contains(play.aim);
    let p = play.aim.world().with_y(terrain::actor_height(
        region,
        play.aim.world(),
        city.bridge_fixed,
    ));
    let (cloud_pose, cloud_visible) = &mut *cloud;
    cloud_pose.translation = p + Vec3::new(0.18, 1.56 + time.elapsed_secs().sin() * 0.065, -0.23);
    cloud_pose.rotation = Quat::from_rotation_y(0.51);
    cloud_pose.scale = Vec3::splat(1.17);
    **cloud_visible = if open {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    let (ring_shape, ring_mesh, ring_visible, ring_surface) = &mut *ring;
    let cells = if play.tool == Tool::Miracles {
        play.miracle().radius(game.campaign.gifts)
    } else {
        0
    };
    let signature = (game.campaign.active, play.aim, cells, city.bridge_fixed);
    if open && play.tool != Tool::Build && ring_shape.0 != Some(signature) {
        let radius = if cells == 0 {
            0.74
        } else {
            f32::from(cells) * CELL
        };
        ring_mesh.0 = meshes.add(art::surface_ring(region, p, radius, city.bridge_fixed));
        ring_shape.0 = Some(signature);
    }
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
        if let Blueprint::Road(index) = *layer {
            let tile = if index == 0 {
                play.road_anchor
            } else {
                target.road_preview.get(index - 1).copied()
            };
            *visible = if let Some(tile) =
                tile.filter(|_| open && play.tool == Tool::Plan && play.option == 0)
            {
                pose.translation = terrain::on_ground(region, tile.world()) + Vec3::Y * 0.082;
                pose.scale = Vec3::splat(CELL * 0.87);
                surface.0 = if index == 0 {
                    art.gold.clone()
                } else {
                    art.valid.clone()
                };
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            continue;
        }
        let kind = play.building();
        let enabled = match layer {
            Blueprint::Roof => kind.size() >= 2 && kind != Building::Field,
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
        let center = terrain::on_ground(region, kind.center(play.aim));
        let rotation = Quat::from_rotation_y(play.rotation as f32 * std::f32::consts::FRAC_PI_2);
        pose.rotation = rotation;
        surface.0 = if target.legal {
            art.ghost.clone()
        } else {
            art.blocked.clone()
        };
        match layer {
            Blueprint::Base => {
                let size = if kind == Building::Tower {
                    Vec3::new(1.47, 2.86, 1.47)
                } else if kind.size() == 1 {
                    Vec3::new(1.35, 1.54, 1.32)
                } else if kind == Building::Field {
                    Vec3::new(3.25, 0.18, 3.21)
                } else {
                    let span = f32::from(kind.size()) * CELL;
                    Vec3::new(span * 0.86, 1.45, span * 0.80)
                };
                pose.translation = center + Vec3::Y * size.y * 0.5;
                pose.scale = size;
            }
            Blueprint::Roof => {
                pose.translation = center + Vec3::Y * 1.46;
                pose.scale = Vec3::new(
                    1.50 * f32::from(kind.size()),
                    1.14,
                    1.37 * f32::from(kind.size()),
                );
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
            Blueprint::Road(_) => unreachable!(),
        }
    }
}

pub fn magic(
    mut commands: Commands,
    time: Res<Time>,
    play: Res<Play>,
    game: Res<Game>,
    art: Res<Art>,
    mut board: ResMut<Board>,
    mut effects: Query<(Entity, &mut Effect, &mut Transform, &mut Mesh3d)>,
) {
    let city = game.city();
    let region = game.region();
    let contact = |point| terrain::actor_height(region, point, city.bridge_fixed);
    if let Some((spell, tile, revision)) = play.last_magic {
        if revision != board.seen_magic {
            board.seen_magic = revision;
            let mat = match spell {
                Miracle::Rain => &art.ripple,
                Miracle::Wind => &art.white,
                Miracle::Ward => &art.valid,
                Miracle::Bloom => &art.berry,
                Miracle::Mend => &art.gold,
                Miracle::Thunder => &art.amber,
            };
            let radius = f32::from(spell.radius(game.campaign.gifts)) * CELL;
            let center = tile.world().with_y(contact(tile.world()));
            if let Some(root) = board.root {
                if spell == Miracle::Thunder {
                    for (x, z, h, angle) in [
                        (0.18, 0., 5.8, 0.28),
                        (-0.31, 0.05, 4.1, -0.44),
                        (0.25, -0.11, 2.1, 0.38),
                    ] {
                        commands.spawn((
                            ChildOf(root),
                            Effect {
                                age: 0.,
                                length: 0.26,
                                velocity: Vec3::ZERO,
                                scale: 0.17,
                                phase: EffectPhase::Flash,
                            },
                            Mesh3d(art.cube.clone()),
                            MeshMaterial3d(art.gold.clone()),
                            Transform::from_translation(center + Vec3::new(x, h, z))
                                .with_scale(Vec3::new(0.17, 2.35, 0.17))
                                .with_rotation(Quat::from_rotation_z(angle)),
                            bevy::pbr::NotShadowCaster,
                        ));
                    }
                }
                for i in 0..if spell == Miracle::Rain { 34 } else { 20 } {
                    let angle = i as f32 * 2.39996;
                    let outward = Vec3::new(angle.cos(), 0., angle.sin());
                    let frac = (i as f32 * 0.618).fract();
                    let start = center + outward * radius * frac.sqrt();
                    let height = if spell == Miracle::Rain {
                        1.6 + frac * 3.4
                    } else {
                        0.25 + frac
                    };
                    let start = start.with_y(contact(start) + height);
                    let velocity = match spell {
                        Miracle::Rain => Vec3::new(0.1, -1.9, 0.05),
                        Miracle::Mend => -outward * 0.5 + Vec3::Y * 1.45,
                        Miracle::Bloom => outward * 0.41 + Vec3::Y * 0.84,
                        Miracle::Thunder => outward * 2.2 + Vec3::Y * 0.55,
                        _ => outward * (1.4 + frac) + Vec3::Y * 0.33,
                    };
                    let length = if spell == Miracle::Rain {
                        2.0
                    } else {
                        0.7 + frac * 0.34
                    };
                    commands.spawn((
                        ChildOf(root),
                        Effect {
                            age: 0.,
                            length,
                            velocity,
                            scale: if spell == Miracle::Rain { 0.085 } else { 0.25 },
                            phase: if spell == Miracle::Rain {
                                EffectPhase::Falling
                            } else {
                                EffectPhase::Rising
                            },
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
    let delta = time.delta_secs().min(0.09);
    for (entity, mut effect, mut pose, mut mesh) in &mut effects {
        effect.age += delta;
        if effect.age >= effect.length {
            commands.entity(entity).despawn();
            continue;
        }
        match effect.phase {
            EffectPhase::Falling => {
                pose.translation += effect.velocity * delta - Vec3::Y * 4.9 * delta * delta;
                effect.velocity.y -= 9.8 * delta;
                let ground = contact(pose.translation);
                if pose.translation.y <= ground + 0.049 {
                    pose.translation.y = ground + 0.051;
                    pose.scale = Vec3::splat(0.08);
                    effect.phase = EffectPhase::Splash;
                    effect.length = 0.24;
                    effect.age = 0.;
                    mesh.0 = art.ring.clone();
                } else {
                    pose.scale =
                        Vec3::new(effect.scale, effect.velocity.y.abs() * 0.089, effect.scale);
                }
            }
            EffectPhase::Splash => {
                pose.scale = Vec3::splat(0.08 + effect.age * 1.14);
            }
            EffectPhase::Rising => {
                pose.translation += effect.velocity * delta;
                pose.translation.y = pose.translation.y.max(contact(pose.translation) + 0.06);
                let scale = effect.scale * (1. - effect.age / effect.length).sqrt();
                pose.scale = Vec3::new(scale, scale * 0.41, scale);
            }
            EffectPhase::Flash => {}
        }
    }
}
