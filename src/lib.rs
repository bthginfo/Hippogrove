mod art;
mod atmosphere;
mod civic;
mod context;
mod controls;
mod game;
mod growth;
mod hud;
mod life;
mod modes;
mod play;
mod practice;
mod quests;
mod region;
mod rivals;
mod save;
mod sound;
mod story;
mod terrain;
mod view;
mod wheel;

use bevy::{
    asset::{AssetMetaCheck, io::embedded::EmbeddedAssetRegistry},
    audio::{AudioLoader, AudioPlugin, AudioSource},
    prelude::*,
    render::{
        ExtractSchedule, RenderApp,
        view::screenshot::{Screenshot, save_to_disk},
    },
    window::PresentMode,
};

use crate::{
    game::Game,
    modes::Mode,
    play::{CameraRig, Panel, Play, Target},
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
    let mut file = SaveFile::for_device();
    let saved = file.load().and_then(Game::resume);
    if saved.is_none() {
        file.preserve_unreadable();
    }
    let game = saved.unwrap_or_else(|| Game::start(Mode::Tutorial));
    let mut play = Play::new(&game);
    play.show_main_menu(&game);
    play.mix = file.load_mix();
    let view = CameraRig::new(&game);
    let mut app = App::new();
    let plugins = DefaultPlugins
        .build()
        .disable::<AudioPlugin>()
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
        .init_asset::<AudioSource>()
        .init_asset_loader::<AudioLoader>()
        .init_non_send_resource::<sound::Playback>()
        .insert_resource(ClearColor(art::rgb(0xc9d1a7)))
        .insert_resource(Time::<Fixed>::from_hz(24.))
        .insert_resource(play)
        .insert_resource(view)
        .insert_resource(game)
        .insert_resource(file)
        .init_resource::<controls::Controls>()
        .init_resource::<hud::Hud>()
        .init_resource::<Target>()
        .init_resource::<view::Board>()
        .init_resource::<view::RenderReadiness>()
        .init_resource::<view::HarvestEffects>()
        .add_event::<sound::Harvest>()
        .add_systems(Startup, (view::setup, hud::setup, sound::setup))
        .add_systems(FixedUpdate, simulate)
        .add_systems(
            Update,
            (
                controls::collect,
                story::update.run_if(world_visible),
                view::move_camera,
                refresh_target,
                view::sync_board,
                view::harvest,
                sound::update,
                (
                    view::props,
                    view::animate,
                    view::animate_rival,
                    view::highlight_guide,
                    view::festival::stage,
                    view::visits::animate,
                    view::highlight_carry,
                    art::style_creature,
                    art::meadow_detail,
                    life::update,
                    view::gates.after(view::animate),
                    view::aim,
                    view::animate_paw.after(view::aim),
                    atmosphere::update,
                    view::magic,
                ),
                view::ready_to_show,
                hud::layout,
                (hud::update, hud::update_meter),
                (hud::pad_reticle, hud::world_labels),
                hud::camera_fade,
                save::checkpoint,
                screenshot,
            )
                .chain(),
        );
    app.sub_app_mut(RenderApp)
        .add_systems(ExtractSchedule, view::report_render_readiness);
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
        synchronous_pipeline_compilation: true,
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
        embed!("models/villages/valley.glb"),
        embed!("models/villages/fjord.glb"),
        embed!("models/villages/terraces.glb"),
        embed!("models/villages/lake.glb"),
        embed!("models/villages/aegean.glb"),
        embed!("ui/atlas.png"),
        embed!("ui/welten.png"),
        embed!("ui/buildings/valley.png"),
        embed!("ui/buildings/fjord.png"),
        embed!("ui/buildings/terraces.png"),
        embed!("ui/buildings/lake.png"),
        embed!("ui/buildings/aegean.png"),
        embed!("models/waldwinkel/wood_tree.glb"),
        embed!("models/waldwinkel/resource_rock.glb"),
        embed!("models/waldwinkel/find_mushroom.glb"),
        embed!("models/waldwinkel/find_marsh_herb.glb"),
        embed!("models/waldwinkel/find_shell.glb"),
        embed!("models/waldwinkel/crop_strawberry.glb"),
        embed!("models/waldwinkel/furniture_harbor_lantern.glb"),
        embed!("models/waldwinkel/watch_fern.glb"),
        embed!("audio/grove.ogg"),
        embed!("audio/cove.ogg"),
        embed!("audio/fjord.ogg"),
        embed!("audio/terraces.ogg"),
        embed!("audio/lake.ogg"),
        embed!("audio/aegean.ogg"),
        embed!("audio/axe_cut.wav"),
        embed!("audio/stone_break.wav"),
        embed!("audio/harvest_sound.wav"),
        embed!("audio/coast.wav"),
        embed!("audio/rain_wood.wav"),
        embed!("audio/fernwind.wav"),
        embed!("audio/card_hit.wav"),
        embed!("audio/card_shield.wav"),
        embed!("audio/paw_lift.wav"),
        embed!("audio/paw_land.wav"),
        embed!("audio/hippo_care.wav"),
        embed!("audio/chapter_chime.wav"),
        embed!("audio/rival_omen.wav"),
        embed!("audio/ui_focus.wav"),
        embed!("audio/ui_choose.wav"),
        embed!("audio/ui_back.wav"),
        embed!("audio/ui_blocked.wav"),
        embed!("audio/quest_accepted.wav"),
    ];
    for (name, data) in files {
        assets.insert_asset(Path::new("assets").join(name), Path::new(name), data);
    }
}

fn world_visible(board: Res<view::Board>, game: Res<Game>) -> bool {
    !board.loading(&game)
}

fn simulate(
    time: Res<Time<Fixed>>,
    play: Res<Play>,
    board: Res<view::Board>,
    mut game: ResMut<Game>,
) {
    if !play.paused && play.panel == Panel::Closed && !board.loading(&game) {
        game.step(time.delta_secs());
    }
}

fn refresh_target(game: Res<Game>, play: Res<Play>, time: Res<Time>, mut target: ResMut<Target>) {
    target.refresh(&play, &game, time.elapsed_secs());
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
