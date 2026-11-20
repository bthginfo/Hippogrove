use bevy::{
    pbr::{
        CascadeShadowConfigBuilder, DirectionalLightShadowMap, DistanceFog, FogFalloff,
        NotShadowCaster,
    },
    prelude::*,
};

use crate::{
    art::{Art, rgb},
    game::{DAY_LENGTH, Game},
    play::{CameraRig, Panel, Play},
    region::{Biome, Weather},
    terrain,
    view::WorldCamera,
};

const HANDHELD: bool = cfg!(any(target_os = "android", target_os = "ios"));

#[derive(Component)]
pub(crate) struct Sun;

#[derive(Component)]
pub(crate) struct Particle {
    offset: Vec2,
    phase: f32,
    variation: f32,
}

#[derive(Resource, Default)]
pub(crate) struct Atmosphere {
    biome: Option<Biome>,
    sky: Option<Sky>,
    clock: f32,
    texture: Option<(Weather, Handle<StandardMaterial>)>,
    weather: Option<Weather>,
    presence: f32,
    snow: Handle<StandardMaterial>,
    rain: Handle<StandardMaterial>,
    mist: Handle<StandardMaterial>,
    dust: Handle<StandardMaterial>,
    leaf: Handle<StandardMaterial>,
}

#[derive(Clone, Copy)]
struct Sky {
    sun: Color,
    lux: f32,
    fill: Color,
    fill_amount: f32,
    horizon: Color,
    haze: f32,
    yaw: f32,
    tilt: f32,
}

impl Sky {
    fn target(biome: Biome, weather: Weather, clock: f32) -> Self {
        let (sun, fill, horizon, lux, yaw, tilt) = match biome {
            Biome::Valley => (0xffe1b4, 0xd8ebe2, 0xb7c9ab, 3410., 0.40, -0.79),
            Biome::Fjord => (0xffe8c6, 0xc2dcf0, 0x99b3c5, 3080., 0.29, -0.63),
            Biome::Terraces => (0xffe5bd, 0xd1e6d6, 0xadc5b4, 3310., 0.80, -0.88),
            Biome::Lake => (0xffdcb1, 0xe6e4cd, 0xdac7a4, 3690., 0.72, -0.87),
            Biome::Aegean => (0xffe6c3, 0xc9e4f2, 0x88c1dc, 3910., 0.56, -0.82),
        };
        let mut sky = Self {
            sun: rgb(sun),
            lux,
            fill: rgb(fill),
            fill_amount: 780.,
            horizon: rgb(horizon),
            haze: 0.31,
            yaw,
            tilt,
        };
        let (filter, mix, lux, fill, haze) = match weather {
            Weather::Clear => (0xffffff, 0., 1., 1., 0.18),
            Weather::Showers => (0x9bbed0, 0.68, 0.54, 1.19, 0.49),
            Weather::Snow => (0xb3cbe8, 0.61, 0.59, 1.19, 0.57),
            Weather::Mist => (0xc3ddcc, 0.55, 0.59, 1.24, 0.72),
            Weather::Drought => (0xe8ac67, 0.39, 1.04, 0.97, 0.33),
            Weather::Breeze => (0xffdab5, 0.29, 0.99, 1.03, 0.19),
        };
        sky.sun = sky.sun.mix(&rgb(filter), mix);
        sky.fill = sky.fill.mix(&rgb(filter), mix * 0.49);
        sky.horizon = sky.horizon.mix(&rgb(filter), mix * 0.79);
        sky.lux *= lux;
        sky.fill_amount *= fill;
        sky.haze = haze;
        let time = clock.rem_euclid(DAY_LENGTH);
        let dawn = 1. - terrain::smooth(time / 34.);
        let dusk =
            terrain::smooth((time - 103.) / 29.) * (1. - terrain::smooth((time - 141.) / 12.));
        let moon =
            terrain::smooth((time - 143.) / 13.) * (1. - terrain::smooth((time - 178.) / 14.));
        let evening = (dusk + dawn * 0.57).clamp(0., 1.);
        sky.sun = sky
            .sun
            .mix(&rgb(0xffbf86), evening * 0.58)
            .mix(&rgb(0xa1b5ed), moon * 0.94);
        sky.fill = sky.fill.mix(&rgb(0x9bb1e8), moon * 0.86);
        sky.horizon = sky
            .horizon
            .mix(&rgb(0xedbc98), dusk * 0.43)
            .mix(&rgb(0x526887), moon * 0.90);
        sky.lux *= 1. - 0.72 * moon - 0.13 * dusk;
        sky.fill_amount *= 1. - 0.21 * moon;
        sky.yaw += time.min(150.) * 0.0035 - 0.16;
        sky.tilt += 0.31 * evening + 0.22 * moon;
        sky.haze += 0.10 * dusk + 0.09 * moon;
        sky
    }

    fn approach(self, target: Self, t: f32) -> Self {
        Self {
            sun: self.sun.mix(&target.sun, t),
            lux: self.lux.lerp(target.lux, t),
            fill: self.fill.mix(&target.fill, t),
            fill_amount: self.fill_amount.lerp(target.fill_amount, t),
            horizon: self.horizon.mix(&target.horizon, t),
            haze: self.haze.lerp(target.haze, t),
            yaw: self.yaw.lerp(target.yaw, t),
            tilt: self.tilt.lerp(target.tilt, t),
        }
    }
}

pub(crate) fn setup(commands: &mut Commands, materials: &mut Assets<StandardMaterial>, art: &Art) {
    let mut material = |color, alpha| {
        materials.add(StandardMaterial {
            base_color: rgb(color).with_alpha(alpha),
            unlit: true,
            alpha_mode: if alpha < 1. {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            },
            ..default()
        })
    };
    let state = Atmosphere {
        snow: material(0xf1f8fc, 1.),
        rain: material(0xa7d5ea, 0.69),
        mist: material(0xdfeadd, 0.17),
        dust: material(0xe5bb85, 0.44),
        leaf: material(0xd6b45d, 1.),
        ..default()
    };
    for i in 0..if HANDHELD { 52 } else { 76 } {
        let h = terrain::hash(i, 11, 221);
        let x = (h & 1023) as f32 / 1023. - 0.5;
        let z = ((h >> 10) & 1023) as f32 / 1023. - 0.5;
        let variation = (h >> 20) as f32 / 4095.;
        commands.spawn((
            Particle {
                offset: Vec2::new(x, z),
                phase: (f32::from(i) * 0.618_034).fract(),
                variation,
            },
            Mesh3d(art.lump.clone()),
            MeshMaterial3d(state.snow.clone()),
            Transform::default(),
            Visibility::Hidden,
            NotShadowCaster,
        ));
    }
    commands.insert_resource(state);
    commands.insert_resource(DirectionalLightShadowMap {
        size: if HANDHELD { 1024 } else { 2048 },
    });
    commands.insert_resource(AmbientLight::default());
    commands.spawn((
        Name::new("Sonnenlicht"),
        Sun,
        DirectionalLight {
            shadows_enabled: true,
            shadow_normal_bias: if HANDHELD { 1.47 } else { 1.52 },
            ..default()
        },
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: if HANDHELD { 59. } else { 72. },
            ..default()
        }
        .build(),
        Transform::default(),
    ));
}

pub(crate) fn update(
    time: Res<Time>,
    game: Res<Game>,
    play: Res<Play>,
    view: Res<CameraRig>,
    art: Res<Art>,
    mut state: ResMut<Atmosphere>,
    mut ambient: ResMut<AmbientLight>,
    mut background: ResMut<ClearColor>,
    mut sun: Single<(&mut DirectionalLight, &mut Transform), With<Sun>>,
    mut fog: Single<&mut DistanceFog, With<WorldCamera>>,
    mut particles: Query<
        (
            &Particle,
            &mut Transform,
            &mut Mesh3d,
            &mut MeshMaterial3d<StandardMaterial>,
            &mut Visibility,
        ),
        Without<Sun>,
    >,
) {
    let city = game.city();
    let region = game.region();
    let weather = if city.rainfall > 0. {
        Weather::Showers
    } else {
        region.weather(city.weather_time)
    };
    let target = Sky::target(region.kind, weather, city.weather_time);
    let delta = time.delta_secs().min(0.075);
    let easing = 1. - (-1.65 * delta).exp();
    let changed = state.biome != Some(region.kind);
    let sky = match state.sky {
        Some(old) if !changed => old.approach(target, easing),
        _ => target,
    };
    state.biome = Some(region.kind);
    state.sky = Some(sky);
    if !play.paused && play.panel == Panel::Closed {
        state.clock = (state.clock + delta).rem_euclid(4096.);
    }
    ambient.color = sky.fill;
    ambient.brightness = sky.fill_amount;
    background.0 = sky.horizon;
    sun.0.color = sky.sun;
    sun.0.illuminance = sky.lux;
    sun.1.rotation = Quat::from_euler(EulerRot::YXZ, sky.yaw, sky.tilt, 0.);
    let height = view.rendered_height;
    let distance = CameraRig::arm(height).length();
    let density = 1.09 - 0.52 * sky.haze;
    fog.color = sky.horizon.with_alpha(sky.haze);
    fog.falloff = FogFalloff::Linear {
        start: distance * density,
        end: distance + height * (1.23 + 1.07 * density),
    };

    if changed || (weather != Weather::Clear && state.weather != Some(weather)) {
        state.weather = Some(weather);
        state.presence = if changed { 1. } else { 0. };
    }
    let active = weather != Weather::Clear && height < 39.;
    let desired = if active { 1. } else { 0. };
    state.presence = state.presence.lerp(desired, (delta * 3.6).min(1.));
    let current = state.weather.unwrap_or(Weather::Clear);
    if state
        .texture
        .as_ref()
        .is_none_or(|(previous, _)| *previous != current)
    {
        let texture = match current {
            Weather::Showers => &state.rain,
            Weather::Mist => &state.mist,
            Weather::Drought => &state.dust,
            Weather::Breeze => &state.leaf,
            _ => &state.snow,
        };
        state.texture = Some((current, texture.clone()));
    }
    let texture = &state.texture.as_ref().unwrap().1;
    let span = (21. + view.height * 0.87).min(53.);
    let t = state.clock;
    for (seed, mut pose, mut mesh, mut surface, mut visible) in &mut particles {
        let ratio = match current {
            Weather::Clear => 0.,
            Weather::Mist => 0.29,
            Weather::Drought => 0.45,
            Weather::Breeze => 0.57,
            Weather::Snow => 0.91,
            Weather::Showers => 1.,
        };
        if state.presence < 0.04 || seed.variation > ratio {
            visible.set_if_neq(Visibility::Hidden);
            continue;
        }
        visible.set_if_neq(Visibility::Visible);
        let (wind, rate) = match current {
            Weather::Showers => (1.32, 0.72),
            Weather::Snow => (0.77, 0.109),
            Weather::Mist => (0.48, 0.052),
            Weather::Drought => (1.63, 0.21),
            Weather::Breeze => (2.25, 0.19),
            Weather::Clear => unreachable!(),
        };
        let age = (t * rate * (0.76 + seed.variation * 0.63) + seed.phase).fract();
        let pulse = (t * (0.81 + seed.variation) + seed.phase * 29.).sin();
        let offset = (seed.offset + Vec2::new(t * wind / span + 0.5, t * wind * 0.23 / span + 0.5))
            .rem_euclid(Vec2::ONE)
            - Vec2::splat(0.5);
        let p = view.target + Vec3::new(offset.x * span, 0., offset.y * span + pulse * 0.23);
        let ground = terrain::actor_height(region, p, city.bridge_fixed);
        let splash = current == Weather::Showers && age > 0.88;
        let (height, shape) = match current {
            Weather::Showers if splash => (0.063, Vec3::splat((age - 0.88) * 1.65 + 0.029)),
            Weather::Showers => (
                7.6 * (1. - (age / 0.88).powf(1.38)),
                Vec3::new(0.034, 0.53, 0.038),
            ),
            Weather::Snow => (7.3 * (1. - age), Vec3::splat(0.082 + seed.variation * 0.08)),
            Weather::Mist => (0.26 + seed.variation * 1.75, Vec3::new(2.7, 0.16, 0.83)),
            Weather::Drought => (0.24 + (pulse + 1.) * 0.19, Vec3::new(0.27, 0.081, 0.09)),
            Weather::Breeze => (
                0.7 + 2.4 * (1. - age) + pulse * 0.28,
                Vec3::new(0.24, 0.043, 0.085),
            ),
            Weather::Clear => unreachable!(),
        };
        pose.translation = p.with_y(ground + height);
        pose.scale = shape * state.presence;
        pose.rotation = if current == Weather::Showers {
            Quat::from_rotation_z(if splash { 0. } else { 0.18 })
        } else {
            Quat::from_euler(EulerRot::YXZ, pulse + seed.phase * 6., pulse * 0.48, 0.)
        };
        let desired_mesh = if splash { &art.ring } else { &art.lump };
        if mesh.0 != *desired_mesh {
            mesh.0 = desired_mesh.clone();
        }
        if surface.0 != *texture {
            surface.0 = texture.clone();
        }
    }
}
