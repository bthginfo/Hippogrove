use bevy::{
    audio::{AudioSink, AudioSinkPlayback, Volume},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::{
    game::{DayPhase, Game, Miracle},
    play::{CameraRig, Panel, Play},
    region::{Biome, Material, Weather},
    terrain,
};

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
    Drop,
    Build,
    Order,
    Cast(Miracle),
    Reward,
}

#[derive(Event)]
pub struct Harvest(pub Material, pub f32);

#[derive(Component)]
pub(crate) enum Bed {
    Music,
    Wind,
    Coast,
    Rain,
}

#[derive(Resource)]
pub struct Sound {
    wood: Handle<AudioSource>,
    stone: Handle<AudioSource>,
    food: Handle<AudioSource>,
    wind: Handle<AudioSource>,
    light: Handle<AudioSource>,
    last_revision: u64,
    last_harvest: f32,
    variant: usize,
}

fn source(assets: &AssetServer, file: &str) -> Handle<AudioSource> {
    assets.load(format!("embedded://audio/{file}"))
}

pub fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    for (bed, file) in [
        (Bed::Music, "grove.ogg"),
        (Bed::Wind, "fernwind.wav"),
        (Bed::Coast, "coast.wav"),
        (Bed::Rain, "rain_wood.wav"),
    ] {
        commands.spawn((
            bed,
            AudioPlayer::new(source(&assets, file)),
            PlaybackSettings::LOOP.with_volume(Volume::SILENT),
        ));
    }
    commands.insert_resource(Sound {
        wood: source(&assets, "axe_cut.wav"),
        stone: source(&assets, "stone_break.wav"),
        food: source(&assets, "harvest_sound.wav"),
        wind: source(&assets, "card_hit.wav"),
        light: source(&assets, "card_shield.wav"),
        last_revision: 0,
        last_harvest: -1.,
        variant: 0,
    });
}

fn play_cue(commands: &mut Commands, sound: &mut Sound, cue: Cue, attenuation: f32) {
    let (clip, level, pitch) = match cue {
        Cue::Harvest(Material::Wood) => (&sound.wood, 0.34, 0.94),
        Cue::Harvest(Material::Stone) => (&sound.stone, 0.31, 0.96),
        Cue::Harvest(Material::Food) => (&sound.food, 0.30, 1.08),
        Cue::Gather => (&sound.food, 0.18, 1.25),
        Cue::Drop => (&sound.wood, 0.24, 0.73),
        Cue::Build => (&sound.stone, 0.35, 0.74),
        Cue::Order => (&sound.food, 0.14, 0.89),
        Cue::Cast(Miracle::Wind) => (&sound.wind, 0.42, 1.13),
        Cue::Cast(Miracle::Thunder) => (&sound.wind, 0.52, 0.64),
        Cue::Cast(Miracle::Rain) => (&sound.light, 0.22, 0.82),
        Cue::Cast(Miracle::Bloom) => (&sound.light, 0.27, 1.23),
        Cue::Cast(Miracle::Mend) => (&sound.light, 0.27, 1.03),
        Cue::Cast(Miracle::Ward) => (&sound.light, 0.32, 0.72),
        Cue::Reward => (&sound.light, 0.24, 1.15),
    };
    let variation = [1., 0.94, 1.05, 0.97, 1.09][sound.variant];
    sound.variant = (sound.variant + 1) % 5;
    commands.spawn((
        AudioPlayer::new(clip.clone()),
        PlaybackSettings::DESPAWN
            .with_volume(Volume::Linear(level * attenuation))
            .with_speed(pitch * variation),
    ));
}

pub fn update(
    mut commands: Commands,
    time: Res<Time>,
    game: Res<Game>,
    play: Res<Play>,
    view: Res<CameraRig>,
    mut state: ResMut<Sound>,
    mut beds: Query<(&Bed, &mut AudioSink)>,
    mut harvests: EventReader<Harvest>,
) {
    let region = game.region();
    let city = game.city();
    let rain = city.rainfall > 0. || region.weather(city.weather_time) == Weather::Showers;
    let coast = if matches!(region.kind, Biome::Aegean | Biome::Fjord | Biome::Lake) {
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
    let music = if play.story.is_some() { 0.31 } else { 0.44 };
    let ambient = quiet
        * if city.day_phase() == DayPhase::Night {
            0.64
        } else {
            1.
        };
    let fade = 1. - (-2.3 * time.delta_secs().min(0.13)).exp();
    for (bed, mut sink) in &mut beds {
        let (bus, level) = match bed {
            Bed::Music => (Bus::Music, music),
            Bed::Wind => (Bus::Ambient, ambient * (1. - coast * 0.72) * 0.20),
            Bed::Coast => (Bus::Ambient, ambient * coast * 0.27),
            Bed::Rain => (Bus::Ambient, if rain { ambient * 0.28 } else { 0. }),
        };
        let wanted = if play.mix.enabled(bus) && !play.paused {
            level
        } else {
            0.
        };
        let volume = sink.volume().to_linear().lerp(wanted, fade);
        if (sink.volume().to_linear() - wanted).abs() > 0.0003 {
            sink.set_volume(Volume::Linear(volume));
        }
    }
    let enabled = play.mix.enabled(Bus::Effects) && !play.paused && !beds.is_empty();
    if let Some((cue, revision)) = play.last_sound {
        if state.last_revision != revision {
            state.last_revision = revision;
            if enabled {
                play_cue(&mut commands, &mut state, cue, 1.);
            }
        }
    }
    let nearest = harvests.read().max_by(|a, b| a.1.total_cmp(&b.1));
    if let Some(&Harvest(material, level)) = nearest {
        let now = time.elapsed_secs();
        if enabled && now - state.last_harvest >= 0.22 {
            state.last_harvest = now;
            play_cue(&mut commands, &mut state, Cue::Harvest(material), level);
        }
    }
}
