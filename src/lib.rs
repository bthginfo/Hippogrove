mod art;
mod atmosphere;
mod civic;
mod controls;
mod game;
mod hud;
mod play;
mod region;
mod save;
mod terrain;
mod view;

use bevy::{
    asset::{AssetMetaCheck, io::embedded::EmbeddedAssetRegistry},
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
    window::PresentMode,
};

use crate::{
    game::Game,
    play::{CameraRig, Panel, Play, Target},
    region::Region,
    save::SaveFile,
};

#[bevy_main]
pub fn main() {
    #[cfg(target_os = "ios")]
    {
        if let Some(bundle) = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(ToOwned::to_owned))
        {
            let _ = std::env::set_current_dir(bundle);
        }
        controls::setup_native();
    }
    let file = SaveFile::for_device();
    let regions = Region::campaign();
    let saved = file.load().and_then(|campaign| campaign.restore(&regions));
    if saved.is_none() {
        file.preserve_unreadable();
    }
    let game = Game::new(regions, saved);
    let mut app = App::new();
    let plugins = DefaultPlugins
        .set(AssetPlugin {
            meta_check: AssetMetaCheck::Never,
            ..default()
        })
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Hippogrove".into(),
                #[cfg(not(target_os = "ios"))]
                resolution: (1360_f32, 765_f32).into(),
                #[cfg(target_os = "ios")]
                mode: bevy::window::WindowMode::BorderlessFullscreen(
                    bevy::window::MonitorSelection::Primary,
                ),
                #[cfg(target_os = "ios")]
                prefers_status_bar_hidden: true,
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        });
    #[cfg(target_os = "android")]
    let plugins = plugins
        .set(android_renderer())
        .add_after::<bevy::render::RenderPlugin>(AndroidMeshPreparation);
    app.add_plugins(plugins)
        .insert_resource(ClearColor(art::rgb(0xc9d1a7)))
        .insert_resource(Time::<Fixed>::from_hz(24.))
        .insert_resource(Play::new(&game))
        .insert_resource(CameraRig::new(&game))
        .insert_resource(game)
        .insert_resource(file)
        .init_resource::<controls::Controls>()
        .init_resource::<hud::Hud>()
        .init_resource::<Target>()
        .init_resource::<view::Board>()
        .add_systems(Startup, (view::setup, hud::setup))
        .add_systems(FixedUpdate, simulate)
        .add_systems(
            Update,
            (
                controls::collect,
                view::move_camera,
                refresh_target,
                view::sync_board,
                (
                    view::props,
                    view::animate,
                    view::aim,
                    atmosphere::update,
                    view::magic,
                ),
                hud::layout,
                hud::update,
                save::checkpoint,
                screenshot,
            )
                .chain(),
        );
    embed_assets(&app);
    app.run();
}

#[cfg(target_os = "android")]
fn android_renderer() -> bevy::render::RenderPlugin {
    use bevy::render::settings::{WgpuFeatures, WgpuSettings, WgpuSettingsPriority};

    // Some Android Vulkan drivers report bindless features that cannot validate the PBR shaders.
    bevy::render::RenderPlugin {
        render_creation: WgpuSettings {
            priority: WgpuSettingsPriority::Compatibility,
            features: WgpuFeatures::empty(),
            ..default()
        }
        .into(),
        ..default()
    }
}

#[cfg(target_os = "android")]
struct AndroidMeshPreparation;

#[cfg(target_os = "android")]
impl Plugin for AndroidMeshPreparation {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        use bevy::render::{
            RenderApp,
            batching::gpu_preprocessing::{GpuPreprocessingMode, GpuPreprocessingSupport},
        };

        // Bevy 0.16's GPU mesh layout needs more buffers than the mobile compatibility profile.
        // Select the same CPU path for both mesh preparation and render phase batching.
        if let Some(renderer) = app.get_sub_app_mut(RenderApp) {
            renderer.insert_resource(GpuPreprocessingSupport {
                max_supported_mode: GpuPreprocessingMode::None,
            });
        }
    }
}

fn embed_assets(app: &App) {
    use std::path::Path;

    let assets = app.world().resource::<EmbeddedAssetRegistry>();
    macro_rules! embed {
        ($file:literal) => {
            ($file, include_bytes!(concat!("../assets/", $file)) as &[u8])
        };
    }
    let files = [
        embed!("fonts/DMSans-Bold.ttf"),
        embed!("fonts/DMSans-Medium.ttf"),
        embed!("fonts/DMSerifText-Regular.ttf"),
        embed!("models/hippo.glb"),
        embed!("ui/atlas.png"),
        embed!("models/waldwinkel/wood_tree.glb"),
        embed!("models/waldwinkel/resource_rock.glb"),
        embed!("models/waldwinkel/find_mushroom.glb"),
        embed!("models/waldwinkel/find_marsh_herb.glb"),
        embed!("models/waldwinkel/find_shell.glb"),
        embed!("models/waldwinkel/crop_strawberry.glb"),
        embed!("models/waldwinkel/furniture_copper_lamp.glb"),
        embed!("models/waldwinkel/furniture_fern.glb"),
    ];
    for (name, data) in files {
        assets.insert_asset(Path::new("assets").join(name), Path::new(name), data);
    }
}

fn simulate(time: Res<Time<Fixed>>, play: Res<Play>, mut game: ResMut<Game>) {
    if !play.paused && play.panel == Panel::Closed {
        game.step(time.delta_secs());
    }
}

fn refresh_target(game: Res<Game>, play: Res<Play>, mut target: ResMut<Target>) {
    target.refresh(&play, &game);
}

fn screenshot(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::F12) {
        if let Err(error) = std::fs::create_dir_all("build") {
            warn!("Screenshot-Ordner: {error}");
            return;
        }
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis());
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(format!("build/hippogrove-{stamp}.png")));
    }
}
