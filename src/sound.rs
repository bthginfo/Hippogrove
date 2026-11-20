use std::{
    io::Cursor,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use bevy::prelude::*;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use serde::{Deserialize, Serialize};

use crate::{
    game::{DayPhase, Game, Miracle},
    play::{CameraRig, Panel, Play},
    region::{Biome, Material, Weather},
    terrain,
};

#[cfg(target_os = "ios")]
#[link(name = "AVFoundation", kind = "framework")]
unsafe extern "C" {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Bus {
    Music,
    Ambient,
    Effects,
}

impl Bus {
    pub const ALL: [Self; 3] = [Self::Music, Self::Ambient, Self::Effects];

    pub fn name(self) -> &'static str {
        match self {
            Self::Music => "Musik",
            Self::Ambient => "Umgebung",
            Self::Effects => "Effekte",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Mix {
    music: bool,
    ambient: bool,
    effects: bool,
}

impl Default for Mix {
    fn default() -> Self {
        Self {
            music: true,
            ambient: true,
            effects: true,
        }
    }
}

impl Mix {
    pub fn enabled(self, bus: Bus) -> bool {
        match bus {
            Bus::Music => self.music,
            Bus::Ambient => self.ambient,
            Bus::Effects => self.effects,
        }
    }

    pub fn toggle(&mut self, bus: Bus) {
        let enabled = match bus {
            Bus::Music => &mut self.music,
            Bus::Ambient => &mut self.ambient,
            Bus::Effects => &mut self.effects,
        };
        *enabled = !*enabled;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Cue {
    Harvest(Material),
    Gather,
    Lift,
    Drop,
    Pet,
    Omen(Biome),
    Build,
    Order,
    Cast(Miracle),
    Reward,
    Focus,
    Choose,
    Back,
    Blocked,
    Quest,
    Test,
}

#[derive(Event)]
pub struct Harvest(pub Material, pub f32);

#[derive(Default)]
pub struct Playback {
    output: Option<Output>,
    retry_at: f64,
    warned: bool,
}

const MUSIC_TRACKS: usize = 1 + Biome::ALL.len();
const BED_COUNT: usize = MUSIC_TRACKS + 3;

struct Output {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    beds: [Option<Sink>; BED_COUNT],
    effects: Vec<Sink>,
    callbacks: Arc<AtomicUsize>,
    seen_callbacks: usize,
    last_callback: f64,
}

impl Playback {
    fn reconnect(&mut self) {
        self.output = None;
        self.retry_at = 0.;
    }

    fn refresh(&mut self, now: f64, paused: bool) {
        if paused {
            self.reconnect();
            return;
        }
        if let Some(output) = &mut self.output {
            let callbacks = output.callbacks.load(Ordering::Relaxed);
            if callbacks != output.seen_callbacks {
                output.last_callback = now;
                output.seen_callbacks = callbacks;
            }
            if now - output.last_callback > 4. {
                self.reconnect();
            }
            return;
        }
        if now < self.retry_at {
            return;
        }
        match OutputStream::try_default() {
            Ok((stream, handle)) => {
                let callbacks = Arc::new(AtomicUsize::new(0));
                let rendered = Arc::clone(&callbacks);
                let keep_alive = rodio::source::Zero::<f32>::new(1, 48_000).periodic_access(
                    Duration::from_millis(250),
                    move |_| {
                        rendered.fetch_add(1, Ordering::Relaxed);
                    },
                );
                if handle.play_raw(keep_alive).is_err() {
                    self.retry_at = now + 2.;
                    return;
                }
                self.output = Some(Output {
                    _stream: stream,
                    handle,
                    beds: Default::default(),
                    effects: Vec::new(),
                    callbacks,
                    seen_callbacks: 0,
                    last_callback: now,
                });
                self.warned = false;
            }
            Err(error) => {
                self.retry_at = now + 2.;
                if !self.warned {
                    self.warned = true;
                    warn!("Tonausgabe noch nicht verfügbar: {error}");
                }
            }
        }
    }
}

impl Output {
    fn track(&self, clip: &AudioSource, volume: f32, pitch: f32, looping: bool) -> Option<Sink> {
        let data = Cursor::new(clip.clone());
        let sink = Sink::try_new(&self.handle).ok()?;
        sink.set_volume(volume);
        sink.set_speed(pitch);
        if looping {
            sink.append(Decoder::new_looped(data).ok()?);
        } else {
            sink.append(Decoder::new(data).ok()?);
        }
        Some(sink)
    }
}

#[derive(Resource)]
pub struct Sound {
    beds: [Handle<AudioSource>; BED_COUNT],
    wood: Handle<AudioSource>,
    stone: Handle<AudioSource>,
    food: Handle<AudioSource>,
    wind: Handle<AudioSource>,
    light: Handle<AudioSource>,
    lift: Handle<AudioSource>,
    land: Handle<AudioSource>,
    care: Handle<AudioSource>,
    chime: Handle<AudioSource>,
    omen: Handle<AudioSource>,
    focus: Handle<AudioSource>,
    choose: Handle<AudioSource>,
    back: Handle<AudioSource>,
    blocked: Handle<AudioSource>,
    quest: Handle<AudioSource>,
    last_harvest: f32,
    variant: usize,
    growth: Option<(crate::modes::Mode, usize)>,
    alert: Option<(crate::modes::Mode, usize)>,
    connected: bool,
    test_pending: Option<f64>,
    test_feedback: f64,
}

impl Sound {
    pub fn status(&self, mix: Mix, now: f64) -> &'static str {
        if !self.connected {
            "Audiogerät wird gesucht …"
        } else if now < self.test_feedback {
            "Testton über die Medienlautstärke abgespielt."
        } else if Bus::ALL.into_iter().all(|bus| !mix.enabled(bus)) {
            "Alle Tonkanäle sind ausgeschaltet."
        } else {
            "Tonausgabe bereit."
        }
    }
}

fn source(assets: &AssetServer, file: &str) -> Handle<AudioSource> {
    assets.load(format!("embedded://audio/{file}"))
}

pub fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(Sound {
        beds: [
            "cove.ogg",
            "grove.ogg",
            "fjord.ogg",
            "terraces.ogg",
            "lake.ogg",
            "aegean.ogg",
            "fernwind.wav",
            "coast.wav",
            "rain_wood.wav",
        ]
        .map(|file| source(&assets, file)),
        wood: source(&assets, "axe_cut.wav"),
        stone: source(&assets, "stone_break.wav"),
        food: source(&assets, "harvest_sound.wav"),
        wind: source(&assets, "card_hit.wav"),
        light: source(&assets, "card_shield.wav"),
        lift: source(&assets, "paw_lift.wav"),
        land: source(&assets, "paw_land.wav"),
        care: source(&assets, "hippo_care.wav"),
        chime: source(&assets, "chapter_chime.wav"),
        omen: source(&assets, "rival_omen.wav"),
        focus: source(&assets, "ui_focus.wav"),
        choose: source(&assets, "ui_choose.wav"),
        back: source(&assets, "ui_back.wav"),
        blocked: source(&assets, "ui_blocked.wav"),
        quest: source(&assets, "quest_accepted.wav"),
        last_harvest: -1.,
        variant: 0,
        growth: None,
        alert: None,
        connected: false,
        test_pending: None,
        test_feedback: 0.,
    });
}

fn play_cue(
    output: &mut Output,
    clips: &Assets<AudioSource>,
    sound: &mut Sound,
    cue: Cue,
    attenuation: f32,
) -> bool {
    let (clip, level, pitch) = match cue {
        Cue::Harvest(Material::Wood) => (&sound.wood, 0.34, 0.94),
        Cue::Harvest(Material::Stone) => (&sound.stone, 0.31, 0.96),
        Cue::Harvest(Material::Food) => (&sound.food, 0.30, 1.08),
        Cue::Gather => (&sound.food, 0.18, 1.25),
        Cue::Lift => (&sound.lift, 0.27, 1.),
        Cue::Drop => (&sound.land, 0.29, 1.),
        Cue::Pet => (&sound.care, 0.41, 1.),
        Cue::Omen(biome) => (
            &sound.omen,
            0.34,
            [1.06, 0.83, 1.27, 0.75, 1.48][biome as usize],
        ),
        Cue::Build => (&sound.stone, 0.35, 0.74),
        Cue::Order => (&sound.food, 0.14, 0.89),
        Cue::Cast(Miracle::Wind) => (&sound.wind, 0.42, 1.13),
        Cue::Cast(Miracle::Thunder) => (&sound.wind, 0.52, 0.64),
        Cue::Cast(Miracle::Rain) => (&sound.light, 0.22, 0.82),
        Cue::Cast(Miracle::Bloom) => (&sound.light, 0.27, 1.23),
        Cue::Cast(Miracle::Mend) => (&sound.light, 0.27, 1.03),
        Cue::Cast(Miracle::Ward) => (&sound.light, 0.32, 0.72),
        Cue::Reward => (&sound.chime, 0.32, 1.),
        Cue::Focus => (&sound.focus, 0.16, 1.),
        Cue::Choose => (&sound.choose, 0.25, 1.),
        Cue::Back => (&sound.back, 0.22, 1.),
        Cue::Blocked => (&sound.blocked, 0.27, 1.),
        Cue::Quest => (&sound.quest, 0.37, 1.),
        Cue::Test => (&sound.chime, 0.72, 1.),
    };
    let Some(clip) = clips.get(clip) else {
        return false;
    };
    let variation = [1., 0.94, 1.05, 0.97, 1.09][sound.variant];
    let Some(sink) = output.track(clip, level * attenuation * 1.35, pitch * variation, false)
    else {
        return false;
    };
    sound.variant = (sound.variant + 1) % 5;
    if output.effects.len() >= 12 {
        output.effects.remove(0);
    }
    output.effects.push(sink);
    true
}

pub fn update(
    time: Res<Time>,
    realtime: Res<Time<Real>>,
    game: Res<Game>,
    mut play: ResMut<Play>,
    view: Res<CameraRig>,
    mut state: ResMut<Sound>,
    clips: Res<Assets<AudioSource>>,
    mut playback: NonSendMut<Playback>,
    mut harvests: EventReader<Harvest>,
) {
    let now = realtime.elapsed_secs_f64();
    let mut cues = if play.sound_cues.is_empty() {
        Vec::new()
    } else {
        std::mem::take(&mut play.sound_cues)
    };
    if cues.iter().any(|cue| matches!(cue, Cue::Test)) {
        playback.retry_at = 0.;
        state.test_pending = Some(now + 6.);
    }
    playback.refresh(now, play.paused);
    state.connected = playback
        .output
        .as_ref()
        .is_some_and(|out| out.seen_callbacks > 0);
    let Some(output) = &mut playback.output else {
        harvests.clear();
        return;
    };
    output.effects.retain(|sink| !sink.empty());
    if let Some(deadline) = state.test_pending {
        if now >= deadline {
            state.test_pending = None;
        } else if state.connected && play_cue(output, &clips, &mut state, Cue::Test, 1.) {
            state.test_pending = None;
            state.test_feedback = now + 3.;
        }
    }
    let region = game.region();
    let city = game.city();
    let rain = city.rainfall > 0. || region.weather(city.weather_time) == Weather::Showers;
    let coast = if region.island_world() || matches!(region.kind, Biome::Fjord | Biome::Lake) {
        let shore = terrain::shore(region, crate::region::Tile::coordinates(view.target));
        1. - terrain::smooth(shore / 8.)
    } else {
        0.
    };
    let quiet = if play.panel == Panel::Closed {
        1.
    } else {
        0.55
    };
    let alert = (city.rival_active(region)
        && city.next_raid(region).is_some_and(|seconds| seconds <= 10.))
    .then_some((game.mode, game.campaign.active));
    let score = if play.at_title() || region.tutorial {
        0
    } else {
        1 + region.kind as usize
    };
    let music = if play.story.is_some() {
        0.31
    } else if alert.is_some() {
        0.35
    } else {
        0.44
    };
    let ambient = quiet
        * if city.day_phase() == DayPhase::Night {
            0.64
        } else {
            1.
        };
    let fade = 1. - (-2.3 * time.delta_secs().min(0.13)).exp();
    let music_levels =
        (0..MUSIC_TRACKS).map(|track| (Bus::Music, if track == score { music } else { 0. }));
    let ambience = [
        ambient * (1. - coast * 0.72) * 0.20,
        ambient * coast * 0.27,
        if rain { ambient * 0.28 } else { 0. },
    ];
    let levels = music_levels.chain(ambience.map(|gain| (Bus::Ambient, gain)));
    for (index, (bus, level)) in levels.enumerate() {
        let wanted = if play.mix.enabled(bus) {
            level * 1.82
        } else {
            0.
        };
        if output.beds[index].is_none() && wanted > 0. {
            output.beds[index] = clips
                .get(&state.beds[index])
                .and_then(|clip| output.track(clip, 0., 1., true));
        }
        if let Some(sink) = &output.beds[index] {
            let volume = sink.volume();
            if (volume - wanted).abs() > 0.0003 {
                let blend = if bus == Bus::Music && play.mix.enabled(bus) {
                    1. - (-1.05 * time.delta_secs().min(0.13)).exp()
                } else {
                    fade
                };
                sink.set_volume(volume.lerp(wanted, blend));
            } else if wanted == 0. {
                output.beds[index] = None;
            }
        }
    }
    let enabled = play.mix.enabled(Bus::Effects);
    if alert.is_some() && alert != state.alert && enabled {
        play_cue(output, &clips, &mut state, Cue::Omen(region.kind), 1.);
    }
    state.alert = alert;
    let stage = game.campaign.gifts.creature.stage();
    if state
        .growth
        .replace((game.mode, stage))
        .is_some_and(|(mode, earlier)| mode == game.mode && earlier < stage)
        && enabled
    {
        play_cue(output, &clips, &mut state, Cue::Reward, 0.77);
    }
    let focus_only = cues.iter().all(|cue| matches!(cue, Cue::Focus));
    for cue in cues.drain(..) {
        if enabled && !matches!(cue, Cue::Test) && (focus_only || !matches!(cue, Cue::Focus)) {
            play_cue(output, &clips, &mut state, cue, 1.);
        }
    }
    if cues.capacity() > 0 {
        play.sound_cues = cues;
    }
    let nearest = harvests.read().max_by(|a, b| a.1.total_cmp(&b.1));
    if let Some(&Harvest(material, level)) = nearest {
        let now = time.elapsed_secs();
        if enabled && now - state.last_harvest >= 0.22 {
            state.last_harvest = now;
            play_cue(output, &clips, &mut state, Cue::Harvest(material), level);
        }
    }
}
