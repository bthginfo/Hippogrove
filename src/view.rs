use bevy::{
    core_pipeline::{fxaa::Fxaa, tonemapping::Tonemapping},
    pbr::DistanceFog,
    prelude::*,
    render::camera::ScalingMode,
};

use crate::{
    art::{self, Art, block, mound, part},
    game::{Building, Crop, Game, MAX_ROAD_RUN, MAX_WALKERS, Miracle, Quest, Walker},
    modes::Mode,
    play::{CameraRig, Panel, PawGesture, Play, Target, Tool},
    region::{AREA_COUNT, Biome, CELL, Material, Tile},
    sound, terrain,
};

#[derive(Component)]
pub struct WorldCamera;

#[derive(Resource, Default)]
pub struct Board {
    active: Option<(Mode, usize)>,
    root: Option<Entity>,
    bridge: Option<(bool, Entity)>,
    boundary: Option<([bool; 5], Entity)>,
    boundary_anchors: usize,
    boundary_camps: [bool; AREA_COUNT],
    roads: Option<(usize, Entity)>,
    buildings: usize,
    walkers: usize,
    seen_magic: u64,
}

#[derive(Component)]
pub struct SourceView {
    index: usize,
    grown: bool,
    yaw: f32,
}

#[derive(Component)]
pub struct GuardView(usize);

#[derive(Component)]
pub enum Banner {
    Rival(Option<usize>),
    Camp(usize),
}

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
    facing: Quat,
    holding: bool,
    gathering: f32,
}

#[derive(Component)]
pub struct Limb {
    walker: usize,
    side: f32,
    arm: bool,
    origin: Vec3,
}

#[derive(Component)]
pub struct GateWing {
    tile: Tile,
    side: f32,
    opening: f32,
}

#[derive(Component)]
pub enum CarryView {
    Hand(Material),
    Actor(usize, Material),
}

#[derive(Component)]
pub struct Cloud;

#[derive(Component)]
pub struct Paw;

#[derive(Component)]
pub struct PawDigit(Vec3);

#[derive(Component)]
pub struct GroundRing(Option<(Mode, usize, Tile, i16, bool)>);

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
    Debris,
}

#[derive(Clone, Copy)]
struct HarvestMotion {
    age: f32,
    depleted: bool,
    regrowing: bool,
}

#[derive(Resource, Default)]
pub struct HarvestEffects {
    active: Option<(Mode, usize)>,
    stock: Vec<u16>,
    motion: Vec<Option<HarvestMotion>>,
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
            far: 790.,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_translation(view.target + CameraRig::arm(view.height))
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
                    Paw,
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
                    let rest = Vec3::new(x, -0.38, 0.20);
                    let digit = mound(digits, &art, &art.boot, rest, Vec3::new(0.16, 0.27, 0.19));
                    digits.commands().entity(digit).insert(PawDigit(rest));
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
    mut view: ResMut<CameraRig>,
    game: Res<Game>,
    mut camera: Single<(&mut Transform, &mut Projection), With<WorldCamera>>,
) {
    let (pose, projection) = &mut *camera;
    let delta = time.delta_secs().min(0.12);
    let jump = std::mem::take(&mut view.jump);
    let follow = if jump { 1. } else { 1. - (-13. * delta).exp() };
    view.cover = if jump {
        1.
    } else {
        (view.cover - delta * 3.8).max(0.)
    };
    let elevation = terrain::height(game.region(), view.target).max(0.)
        * terrain::smooth((72. - view.height) / 47.);
    let arm = CameraRig::arm(view.height);
    let desired =
        Transform::from_translation(view.target.with_y(elevation) + arm).looking_to(-arm, Vec3::Y);
    pose.translation = pose.translation.lerp(desired.translation, follow);
    pose.rotation = pose.rotation.slerp(desired.rotation, follow);
    if let Projection::Orthographic(ortho) = &mut **projection {
        let ScalingMode::FixedVertical {
            ref mut viewport_height,
        } = ortho.scaling_mode
        else {
            return;
        };
        *viewport_height += (view.height - *viewport_height) * follow;
        view.rendered_height = *viewport_height;
    }
}

fn paw_pulse(play: &Play, now: f32) -> (Option<PawGesture>, f32) {
    let Some((gesture, started)) = play.paw_gesture else {
        return (None, 0.);
    };
    let t = ((now - started) / 0.48).clamp(0., 1.);
    (
        (t < 1.).then_some(gesture),
        (t * std::f32::consts::PI).sin(),
    )
}

pub fn animate_paw(
    time: Res<Time>,
    play: Res<Play>,
    game: Res<Game>,
    mut paw: Single<&mut Transform, (With<Paw>, Without<PawDigit>)>,
    mut digits: Query<(&PawDigit, &mut Transform), Without<Paw>>,
) {
    let (gesture, pulse) = paw_pulse(&play, time.elapsed_secs());
    let holding: f32 = if game.city().held.is_some() { 1. } else { 0. };
    let (reach, curl) = match gesture {
        Some(PawGesture::Grab) => (pulse * 0.22, holding.max(pulse * 0.75)),
        Some(PawGesture::Release) => (pulse * 0.17, pulse * 0.35),
        Some(PawGesture::Cast) => (-pulse * 0.13, 0.15),
        Some(PawGesture::Point) => (pulse * 0.14, holding),
        None => (0., holding),
    };
    paw.translation = Vec3::new(0., -reach, reach * 0.49);
    paw.rotation = Quat::from_rotation_x(-0.38 - reach * 1.8 + curl * 0.32)
        * Quat::from_rotation_z(-0.20 + pulse * 0.1);
    for (digit, mut pose) in &mut digits {
        pose.translation =
            digit.0 + Vec3::new(-digit.0.x * curl * 0.14, curl * 0.034, curl * 0.061);
        pose.rotation = Quat::from_rotation_x(curl * 0.64);
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

pub(crate) fn villager(parent: &mut ChildSpawnerCommands, art: &Art, guard: bool, variant: usize) {
    let coat = if guard {
        &art.blue
    } else {
        [&art.door, &art.cobalt, &art.red, &art.clay][variant % 4]
    };
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
        &art.plaster,
        Vec3::new(0., 0.84, 0.088),
        Vec3::new(0.41, 0.14, 0.34),
    );
    mound(
        parent,
        art,
        &art.dark_wood,
        Vec3::new(0., 0.49, 0.),
        Vec3::new(0.58, 0.12, 0.41),
    );
    block(
        parent,
        art,
        &art.ridge,
        Vec3::new(0.02, 0.50, 0.207),
        Vec3::new(0.084, 0.07, 0.036),
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
    mound(
        parent,
        art,
        &art.skin,
        Vec3::new(0., 0.965, 0.273),
        Vec3::new(0.096, 0.079, 0.087),
    );
    let hat = if guard {
        &art.blue
    } else {
        [&art.plank, &art.timber, &art.warm_stone][variant % 3]
    };
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
    if !guard && variant % 3 == 1 {
        mound(
            parent,
            art,
            &art.dark_wood,
            Vec3::new(0.11, 1.02, -0.13),
            Vec3::new(0.33, 0.31, 0.26),
        );
    }
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
                facing: Quat::from_rotation_y(0.44),
                holding: walker.bundle.is_some(),
                gathering: 0.,
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
                villager(actor, art, walker.guard, index);
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
    mut forests: Query<(&art::ForestPatch, &mut Mesh3d)>,
) {
    let current = (game.mode, game.campaign.active);
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
                crate::life::spawn(parent, &art, region);
                for (index, source) in region.sources.iter().enumerate() {
                    let p = grounded(source.tile);
                    let yaw =
                        (terrain::hash(source.tile.0, source.tile.1, 115) % 69) as f32 * 0.092;
                    if index < region.original_sources {
                        parent
                            .spawn((
                                SourceView {
                                    index,
                                    grown: true,
                                    yaw: 0.,
                                },
                                Transform::from_translation(p),
                                Visibility::Inherited,
                            ))
                            .with_children(|child| {
                                contact_shadow(child, &art, 1.15);
                                art::resources(child, &art, source.material);
                            });
                    } else {
                        parent.spawn((
                            SourceView {
                                index,
                                grown: true,
                                yaw,
                            },
                            Mesh3d(
                                art.deposits[region.kind as usize][source.material.index()].clone(),
                            ),
                            MeshMaterial3d(art.ground.clone()),
                            Transform::from_translation(p)
                                .with_rotation(Quat::from_rotation_y(yaw)),
                            Visibility::Inherited,
                        ));
                    }
                    parent.spawn((
                        SourceView {
                            index,
                            grown: false,
                            yaw,
                        },
                        Mesh3d(
                            art.deposits[region.kind as usize]
                                [source.material.index() + Material::ALL.len()]
                            .clone(),
                        ),
                        MeshMaterial3d(art.ground.clone()),
                        Transform::from_translation(p).with_rotation(Quat::from_rotation_y(yaw)),
                        Visibility::Hidden,
                    ));
                }
                for (index, area) in region.areas.iter().enumerate() {
                    parent
                        .spawn((
                            Transform::from_translation(grounded(area.center)),
                            Visibility::Inherited,
                        ))
                        .with_children(|camp| {
                            let flag = art::camp(camp, &art, area.material);
                            camp.commands().entity(flag).insert(Banner::Camp(index));
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
                        Banner::Rival(Some(index)),
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
                            p.commands().entity(flag).insert(Banner::Rival(None));
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
                            villager(p, &art, true, i);
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
        *previous != flags
            || board.boundary_anchors != city.count(Building::Waystone)
            || board.boundary_camps != city.camps
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
        board.boundary_camps = city.camps;
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
    if board.buildings != city.buildings.len()
        || board.roads.map_or(0, |(count, _)| count) != city.roads.len()
    {
        let previous_roads = board.roads.map_or(0, |(count, _)| count);
        for (patch, mut mesh) in &mut forests {
            let changed = city
                .roads
                .iter()
                .skip(previous_roads)
                .any(|&tile| patch.0.contains(tile))
                || city.buildings.iter().skip(board.buildings).any(|site| {
                    site.kind
                        .tiles(site.tile)
                        .any(|tile| patch.0.contains(tile))
                });
            if changed {
                mesh.0 = meshes.add(art::forest_mesh(region, *patch, city));
            }
        }
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
                if site.kind.gate() {
                    for side in [-1., 1.] {
                        parent
                            .spawn((
                                GateWing {
                                    tile: site.tile,
                                    side,
                                    opening: 0.,
                                },
                                Transform::from_translation(Vec3::new(side * 0.73, 0.04, 0.)),
                                Visibility::Inherited,
                            ))
                            .with_children(|wing| art::gate_leaf(wing, &art, side));
                    }
                }
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
                                    let p = Vec3::new(x, 0.16, z);
                                    let yaw = (i * 19 + j * 11) as f32 * 0.29;
                                    if region.kind == Biome::Valley {
                                        art::place_scene(
                                            plants,
                                            &art.waldwinkel.crop,
                                            p,
                                            0.60,
                                            yaw,
                                        );
                                    } else {
                                        part(
                                            plants,
                                            &art.deposits[region.kind as usize]
                                                [Material::Food.index()],
                                            &art.ground,
                                            p,
                                            Vec3::splat(0.46),
                                            Quat::from_rotation_y(yaw),
                                        );
                                    }
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
    harvest: Res<HarvestEffects>,
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
    mut flags: Query<(&Banner, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    let region = game.region();
    let city = game.city();
    for (mut visible, source, guard, crop, marker, carry, upgraded, pose) in &mut visuals {
        let show = if let Some(source) = source {
            let material = region.sources[source.index].material;
            let active = harvest.motion.get(source.index).and_then(|&motion| motion);
            let stock = city.nodes[source.index];
            let dropping = active.is_some_and(|motion| motion.depleted && stock == 0);
            let age = active.map_or(1., |motion| motion.age);
            let stretched =
                dropping && material == Material::Wood && source.index >= region.original_sources;
            if let Some(mut pose) = pose {
                let pulse = (age * 34.).sin() * (1. - age / 0.47).max(0.);
                let tilt = if stretched {
                    terrain::smooth((age - 0.06) / 0.60) * 1.48
                } else if stock == 0 && dropping {
                    terrain::smooth(age * 1.8) * 0.35
                } else {
                    pulse
                        * if material == Material::Stone {
                            0.06
                        } else {
                            0.16
                        }
                };
                let returning = active.is_some_and(|motion| motion.regrowing);
                let settle = if returning {
                    0.28 + terrain::smooth(age / 0.78) * 0.72
                } else if dropping && source.grown && !stretched {
                    1. - terrain::smooth(age / 0.7) * 0.45
                } else {
                    1.
                };
                pose.scale = Vec3::splat(settle * (1. - pulse.abs() * 0.05));
                pose.rotation = Quat::from_rotation_y(source.yaw) * Quat::from_rotation_z(tilt);
            }
            if source.grown {
                stock > 0 || (dropping && age < if stretched { 0.91 } else { 0.64 })
            } else {
                stock == 0 && (!dropping || age >= 0.42)
            }
        } else if let Some(guard) = guard {
            if let Some(mut pose) = pose {
                let location = terrain::on_ground(region, city.guard_world(region, guard.0));
                let travel = location - pose.translation;
                if travel.xz().length_squared() > 0.000002 {
                    let heading = Quat::from_rotation_y(travel.x.atan2(travel.z));
                    pose.rotation = pose.rotation.slerp(heading, 0.22);
                }
                pose.translation = location;
            }
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
        surface.0 = match *flag {
            Banner::Camp(index) if city.camps[index] => art.gold.clone(),
            Banner::Camp(_) => art.blue.clone(),
            Banner::Rival(district) => {
                let outcome = district.map_or(city.outcome, |i| city.districts[i].outcome);
                match outcome {
                    Some(crate::game::Outcome::Alliance) => art.valid.clone(),
                    Some(crate::game::Outcome::Victory) => art.amber.clone(),
                    None => art.blue.clone(),
                }
            }
        };
    }
}

pub fn animate(
    time: Res<Time>,
    game: Res<Game>,
    mut actors: Query<(&mut WalkerView, &mut Transform), Without<Limb>>,
    mut limbs: Query<(&Limb, &mut Transform), Without<WalkerView>>,
) {
    let delta = time.delta_secs().clamp(0.0001, 0.10);
    let city = game.city();
    let region = game.region();
    let mut motion = [(0., 0.); MAX_WALKERS];
    for (mut meta, mut pose) in &mut actors {
        let Some(walker) = game.city().walkers.get(meta.index) else {
            continue;
        };
        let goal = walker.world().with_y(0.);
        let offset = goal - pose.translation.with_y(0.);
        let blend = 1. - (-18. * delta).exp();
        let travel = offset * blend;
        let pace = (travel.length() / delta / 4.8).clamp(0., 1.);
        meta.speed += (pace - meta.speed) * (1. - (-10. * delta).exp());
        let stride = if walker.hippo { 1.44 } else { 1.20 };
        meta.phase = (meta.phase + travel.length() * std::f32::consts::TAU / stride)
            .rem_euclid(std::f32::consts::TAU);
        if travel.length_squared() > 0.00002 {
            let facing = Quat::from_rotation_y(travel.x.atan2(travel.z));
            meta.facing = meta.facing.slerp(facing, 1. - (-12. * delta).exp());
        }
        let holding = walker.bundle.is_some();
        if holding && !meta.holding {
            meta.gathering = 0.48;
        }
        meta.holding = holding;
        meta.gathering = (meta.gathering - delta).max(0.);
        let bow = (meta.gathering / 0.48 * std::f32::consts::PI).sin() * 0.20;
        pose.rotation = meta.facing * Quat::from_rotation_x(bow);
        let bob = meta.phase.sin().abs() * 0.040 * meta.speed;
        let next = pose.translation + travel;
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

pub fn gates(
    time: Res<Time>,
    walkers: Query<&Transform, (With<WalkerView>, Without<GateWing>)>,
    mut wings: Query<(&mut GateWing, &mut Transform), Without<WalkerView>>,
) {
    let blend = 1. - (-14. * time.delta_secs()).exp();
    for (mut gate, mut pose) in &mut wings {
        let p = gate.tile.world();
        let approach = walkers
            .iter()
            .any(|walker| walker.translation.with_y(0.).distance_squared(p) < 9.5);
        gate.opening = gate.opening.lerp(if approach { 1. } else { 0. }, blend);
        pose.rotation = Quat::from_rotation_y(gate.side * gate.opening * 1.64);
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
    let (gesture, pulse) = paw_pulse(&play, time.elapsed_secs());
    let bounce = match gesture {
        Some(PawGesture::Grab | PawGesture::Release) => -0.41 * pulse,
        Some(PawGesture::Cast) => 0.21 * pulse,
        Some(PawGesture::Point) => -0.15 * pulse,
        None => 0.,
    };
    let destination = p + Vec3::new(
        0.18,
        1.53 + time.elapsed_secs().sin() * 0.057 + bounce,
        -0.23,
    );
    let speed = if cloud_pose.translation.distance_squared(destination) > 150. {
        1.
    } else {
        1. - (-17. * time.delta_secs().min(0.12)).exp()
    };
    cloud_pose.translation = cloud_pose.translation.lerp(destination, speed);
    cloud_pose.rotation = Quat::from_rotation_y(0.51);
    cloud_pose.scale = Vec3::splat(1.17 + pulse * 0.035);
    **cloud_visible = if open || play.story.is_some_and(|story| story.show_paw(region)) {
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
    let signature = (
        game.mode,
        game.campaign.active,
        play.aim,
        cells,
        city.bridge_fixed,
    );
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
        let rotation = city.build_rotation(region, kind, play.aim, play.rotation);
        let rotation = Quat::from_rotation_y(f32::from(rotation) * std::f32::consts::FRAC_PI_2);
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
                } else if kind.gate() {
                    Vec3::new(1.73, 2.42, 0.42)
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

pub fn harvest(
    mut commands: Commands,
    time: Res<Time>,
    game: Res<Game>,
    view: Res<CameraRig>,
    art: Res<Art>,
    board: Res<Board>,
    mut effects: ResMut<HarvestEffects>,
    mut sounds: EventWriter<sound::Harvest>,
) {
    let region = game.region();
    let city = game.city();
    let key = (game.mode, game.campaign.active);
    if effects.active != Some(key) {
        effects.active = Some(key);
        effects.stock.clone_from(&city.nodes);
        effects.motion = vec![None; city.nodes.len()];
        return;
    }
    let delta = time.delta_secs().min(0.12);
    for motion in &mut effects.motion {
        if let Some(value) = motion {
            value.age += delta;
            if value.age > 0.96 {
                *motion = None;
            }
        }
    }
    let mut budget = 30;
    for (index, (&count, source)) in city.nodes.iter().zip(&region.sources).enumerate() {
        let was = std::mem::replace(&mut effects.stock[index], count);
        if count > 0 && was == 0 {
            effects.motion[index] = Some(HarvestMotion {
                age: 0.,
                depleted: false,
                regrowing: true,
            });
        }
        if count >= was {
            continue;
        }
        effects.motion[index] = Some(HarvestMotion {
            age: 0.,
            depleted: count == 0,
            regrowing: false,
        });
        let Some(root) = board.root else {
            continue;
        };
        let point = terrain::on_ground(region, source.tile.world());
        let distance = point.distance(view.target);
        if distance > view.height * 1.65 {
            continue;
        }
        let hearing = 1. - terrain::smooth((distance - 5.) / 19.);
        if hearing > 0.02 {
            sounds.write(sound::Harvest(source.material, hearing));
        }
        for i in 0..budget.min(if count == 0 { 12 } else { 6 }) {
            let angle =
                (i as f32 * 2.399 + (index % 5) as f32 * 0.77).rem_euclid(std::f32::consts::TAU);
            let away = Vec3::new(angle.cos(), 0., angle.sin());
            let strength = 0.56 + (i % 4) as f32 * 0.31;
            let (mesh, material, height, size) = match source.material {
                Material::Wood if i % 3 == 0 => (&art.lump, &art.leaf[1], 1.22, 0.13),
                Material::Wood => (&art.cube, &art.plank, 0.46, 0.095),
                Material::Stone if i % 3 == 0 => (&art.lump, &art.plaster, 0.29, 0.125),
                Material::Stone => (&art.lump, &art.stone, 0.37, 0.17),
                Material::Food if i % 3 == 0 => (&art.lump, &art.leaf[2], 0.59, 0.11),
                Material::Food => {
                    let fruit = match region.kind {
                        Biome::Fjord => &art.cobalt,
                        Biome::Terraces | Biome::Lake => &art.gold,
                        Biome::Aegean => &art.leaf[1],
                        _ => &art.berry,
                    };
                    (&art.lump, fruit, 0.72, 0.13)
                }
            };
            commands.spawn((
                ChildOf(root),
                Effect {
                    age: 0.,
                    length: 0.70 + (i % 5) as f32 * 0.056,
                    velocity: away * (strength * 1.29) + Vec3::Y * (1.45 + strength),
                    scale: size,
                    phase: EffectPhase::Debris,
                },
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(point + away * 0.30 + Vec3::Y * height)
                    .with_scale(Vec3::splat(size)),
                bevy::pbr::NotShadowCaster,
            ));
            budget -= 1;
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
            EffectPhase::Debris => {
                pose.translation += effect.velocity * delta - Vec3::Y * 4.9 * delta * delta;
                effect.velocity.y -= 9.8 * delta;
                let ground = contact(pose.translation) + 0.045;
                if pose.translation.y <= ground {
                    pose.translation.y = ground;
                    effect.velocity = effect.velocity.with_y(effect.velocity.y.abs() * 0.29) * 0.73;
                }
                pose.rotate_local_x(delta * 7.);
                pose.rotate_local_z(delta * 4.);
                pose.scale = Vec3::splat(effect.scale * (1. - effect.age / effect.length).sqrt());
            }
            EffectPhase::Flash => {}
        }
    }
}
