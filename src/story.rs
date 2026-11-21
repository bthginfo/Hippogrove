use bevy::prelude::*;

use crate::{
    game::Game,
    play::{CameraRig, Play},
    region::{Biome, Region, Tile},
    terrain,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Sequence {
    Arrival,
    FirstDelivery,
    Shelter,
    Crossing,
    Islands,
}

#[derive(Clone, Copy, Debug)]
pub struct Story {
    pub sequence: Sequence,
    seconds: f32,
}

struct Beat {
    camera: (Tile, f32),
    duration: f32,
    subtitle: &'static str,
}

impl Story {
    pub fn new(sequence: Sequence) -> Self {
        Self {
            sequence,
            seconds: 0.,
        }
    }

    fn beats(self, region: &Region) -> [Beat; 3] {
        let start = region.start;
        let bridge = region.bridge();
        let (shots, lines, timing) = match self.sequence {
            Sequence::FirstDelivery => (
                [
                    (start.offset(0, -1), 22.),
                    (region.opening_farm().unwrap_or(bridge).offset(1, 1), 25.),
                    (start.offset(-3, -4), 27.),
                ],
                [
                    "Miras Familie wartet am Schrein auf ihr erstes Dach.",
                    "Der Sturm hat auch Joras Dach am anderen Ufer beschädigt.",
                    "Nimm Holz auf und entscheide, wem du zuerst hilfst.",
                ],
                [2.4, 2.4, 2.6],
            ),
            Sequence::Arrival => (
                [
                    (start.offset(-10, -11), 49.),
                    (start.offset(-4, -4), 30.),
                    (start.offset(-1, -2), 18.),
                ],
                [
                    match region.kind {
                        Biome::Valley => "Am Weidenfluss wartet ein kleines Dorf auf ein Zeichen.",
                        Biome::Fjord => {
                            "Über dem Fjord leuchtet ein Feuer. Ein Dorf wartet im Schnee."
                        }
                        Biome::Terraces => {
                            "Zwischen Nebel und Terrassen warten die Hüter der Schleusen."
                        }
                        Biome::Lake => {
                            "Am See sammeln sich die Menschen zwischen Akazien und Schilf."
                        }
                        Biome::Aegean => {
                            "An den weißen Ufern warten die Oliveninseln auf den Wind."
                        }
                    },
                    "Da bist du. Die Gläubigen sehen deine Wolkenpfote und das kleine Nilpferd.",
                    region
                        .quests
                        .and_then(|quests| quests.preparation.as_ref())
                        .map(|step| step.hint.as_str())
                        .unwrap_or(if region.kind == Biome::Valley {
                            "Bring Holz zum Sammelplatz. Danach bauen wir das erste Zuhause."
                        } else {
                            "Die Felder brauchen Regen. Wirke dein erstes Wunder und bring die Ernte heim."
                        }),
                ],
                [2.65, 3.0, 3.15],
            ),
            Sequence::Shelter => (
                [
                    (start.offset(0, -1), 19.),
                    (start.offset(2, -2), 21.),
                    (start.offset(2, 0), 23.),
                ],
                [
                    "Unter dem ersten Dach ist es warm.",
                    "Jetzt brauchen wir Nahrung. Leg ein Feld an.",
                    "Ein Regenschauer deiner Wolkenpfote weckt die Saat.",
                ],
                [1.95, 2.1, 2.3],
            ),
            Sequence::Crossing => (
                [
                    (bridge.offset(-4, 3), 23.),
                    (bridge.offset(0, 0), 21.),
                    (region.rival_home().offset(0, 1), 30.),
                ],
                [
                    match region.kind {
                        Biome::Valley => "Die letzten Bretter sitzen. Die Holzbrücke hält.",
                        Biome::Fjord => "Die Fischer haben den Fjordanleger festgebunden.",
                        Biome::Terraces => "Der neue Wehrsteg verbindet die Saat der beiden Ufer.",
                        Biome::Lake => "Der Schilfsteg trägt wieder Körbe und Wasserkrüge.",
                        Biome::Aegean => "Über die Doppelmole kommen die Oliven aus beiden Dörfern.",
                    },
                    "Hinter dem Fluss warten Menschen auf deine Antwort.",
                    "Bring ihnen die erbetene Gabe oder gewinne das Land mit deinen Wundern.",
                ],
                [2.1, 2.2, 2.65],
            ),
            Sequence::Islands => (
                [
                    (start.offset(-19, -8), 58.),
                    (bridge.offset(4, 0), 37.),
                    (start.offset(0, -3), 22.),
                ],
                [
                    "Zwischen weißen Felsen und Oliven trägt der Wind eine alte Geschichte.",
                    "Die Inseln warten auf das Fest der Winde.",
                    "Am Ufer beginnt dein Dorf. Die anderen Siedlungen hörst du jenseits des Wassers.",
                ],
                [2.6, 2.5, 2.9],
            ),
        };
        std::array::from_fn(|i| Beat {
            camera: shots[i],
            duration: timing[i],
            subtitle: lines[i],
        })
    }

    fn cue(self, beats: &[Beat; 3]) -> (usize, f32) {
        let mut passed = self.seconds;
        for (i, beat) in beats.iter().enumerate() {
            if passed < beat.duration {
                return (i, passed / beat.duration);
            }
            passed -= beat.duration;
        }
        (2, 1.)
    }

    pub fn caption(self, region: &Region) -> &'static str {
        let beats = self.beats(region);
        beats[self.cue(&beats).0].subtitle
    }

    pub fn frame(self, region: &Region, view: &mut CameraRig) {
        let beats = self.beats(region);
        let (cue, t) = self.cue(&beats);
        let (from, from_height) = beats[cue].camera;
        let (to, to_height) = beats[(cue + 1).min(2)].camera;
        let t = terrain::smooth(t);
        view.target = from.world().lerp(to.world(), t);
        view.height = from_height.lerp(to_height, t);
    }

    pub fn show_paw(self, region: &Region) -> bool {
        matches!(self.sequence, Sequence::Arrival | Sequence::FirstDelivery)
            && self.cue(&self.beats(region)).0 > 0
    }
}

pub fn update(
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut play: ResMut<Play>,
    mut view: ResMut<CameraRig>,
) {
    if play.at_title() {
        return;
    }
    play.observe_practice(&game, &mut view, time.elapsed_secs());
    play.observe_deliveries(&game, time.elapsed_secs());
    play.observe_quests(&game, &mut view, time.elapsed_secs());
    let Some(story) = &mut play.story else {
        return;
    };
    let region = game.region();
    story.seconds += time.delta_secs().min(0.24);
    if story.seconds
        >= story
            .beats(region)
            .iter()
            .map(|beat| beat.duration)
            .sum::<f32>()
    {
        play.finish_story(&mut game, &mut view);
    } else {
        story.frame(region, &mut view);
    }
}
