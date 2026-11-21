use std::f32::consts::TAU;

use bevy::prelude::Vec2;

use crate::{
    game::{Action, Building, Game, Miracle},
    play::{Panel, Tool, UiAction},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Wheel {
    Tools,
    Build,
    Miracles,
    Hippo,
    Helpers,
    Paths,
}

#[derive(Clone, Copy)]
pub struct Entry {
    pub name: &'static str,
    pub glyph: usize,
    pub action: UiAction,
    hint: &'static str,
}

const fn slot(name: &'static str, glyph: usize, action: UiAction, hint: &'static str) -> Entry {
    Entry {
        name,
        glyph,
        action,
        hint,
    }
}

const fn submenu(wheel: Wheel) -> UiAction {
    UiAction::Panel(Panel::Wheel(wheel))
}

impl Entry {
    pub fn detail(self, game: &Game) -> String {
        let city = game.city();
        let gifts = game.campaign.gifts;
        match self.action {
            UiAction::SelectBuilding(kind) => {
                game.blueprint_readiness(kind).err().unwrap_or_else(|| {
                    let cost = if game.region().sandbox {
                        "Freier Bau".into()
                    } else {
                        kind.cost_text()
                    };
                    format!("{}\n{cost}", kind.use_text())
                })
            }
            UiAction::Order(Tool::Miracles, index) => {
                let spell = Miracle::ALL[index];
                city.ritual_status(game.region(), spell, gifts).map_or_else(
                    || format!("{} · {:.0} Kraft", spell.effect(), spell.cost()),
                    str::to_owned,
                )
            }
            UiAction::Order(Tool::Hippo, 1) if gifts.training < 2 => "Ab Kreaturenrang 2".into(),
            UiAction::Order(Tool::Hippo, 2) if gifts.training < 3 => "Ab Kreaturenrang 3".into(),
            UiAction::Do(Action::Praise) if city.lesson.is_none() => {
                "Nach einer erledigten Aufgabe".into()
            }
            _ => self.hint.into(),
        }
    }
}

impl Wheel {
    pub fn title(self) -> &'static str {
        match self {
            Self::Tools => "Auswahlrad",
            Self::Build => "Bauen",
            Self::Miracles => "Wunder",
            Self::Hippo => "Nilpferd",
            Self::Helpers => "Gläubige",
            Self::Paths => "Wege",
        }
    }

    pub fn entries(self) -> &'static [Entry] {
        match self {
            Self::Tools => {
                const {
                    &[
                        slot(
                            "Pfote",
                            5,
                            UiAction::Tool(Tool::Hand),
                            "Greifen, Ablegen und Ernten",
                        ),
                        slot("Bauen", 6, submenu(Self::Build), "Baupläne wählen"),
                        slot(
                            "Wunder",
                            9,
                            submenu(Self::Miracles),
                            "Ein Wunder vorbereiten",
                        ),
                        slot("Nilpferd", 8, submenu(Self::Hippo), "Befehle und Pflege"),
                        slot("Gläubige", 7, submenu(Self::Helpers), "Helfer und Wachen"),
                        slot("Wege", 15, submenu(Self::Paths), "Wege bauen und ausbauen"),
                        slot(
                            "Fortschritt",
                            13,
                            UiAction::Panel(Panel::Progress),
                            "Talente, Kreatur und Stadt",
                        ),
                        slot(
                            "Aufträge",
                            11,
                            UiAction::Panel(Panel::Quests),
                            "Ziele und Nebenaufträge",
                        ),
                    ]
                }
            }
            Self::Build => {
                const {
                    &[
                        slot("Hütte", 6, UiAction::SelectBuilding(Building::House), ""),
                        slot("Lager", 12, UiAction::SelectBuilding(Building::Store), ""),
                        slot("Feld", 19, UiAction::SelectBuilding(Building::Field), ""),
                        slot("Brunnen", 4, UiAction::SelectBuilding(Building::Well), ""),
                        slot(
                            "Werkstatt",
                            6,
                            UiAction::SelectBuilding(Building::Workshop),
                            "",
                        ),
                        slot("Schrein", 3, UiAction::SelectBuilding(Building::Shrine), ""),
                        slot(
                            "Baubuch",
                            12,
                            UiAction::Panel(Panel::Catalog),
                            "Alle Baupläne und Forschung",
                        ),
                    ]
                }
            }
            Self::Miracles => {
                const {
                    &[
                        slot(
                            "Regen",
                            4,
                            UiAction::Order(Tool::Miracles, Miracle::Rain as usize),
                            "",
                        ),
                        slot(
                            "Wind",
                            3,
                            UiAction::Order(Tool::Miracles, Miracle::Wind as usize),
                            "",
                        ),
                        slot(
                            "Donner",
                            9,
                            UiAction::Order(Tool::Miracles, Miracle::Thunder as usize),
                            "",
                        ),
                        slot(
                            "Schutz",
                            11,
                            UiAction::Order(Tool::Miracles, Miracle::Ward as usize),
                            "",
                        ),
                        slot(
                            "Blüte",
                            19,
                            UiAction::Order(Tool::Miracles, Miracle::Bloom as usize),
                            "",
                        ),
                        slot(
                            "Heillicht",
                            9,
                            UiAction::Order(Tool::Miracles, Miracle::Mend as usize),
                            "",
                        ),
                        slot(
                            "Dorfruf",
                            3,
                            UiAction::Panel(Panel::Gathering),
                            "Nilpferd und Dorf für ein gemeinsames Wunder rufen",
                        ),
                    ]
                }
            }
            Self::Hippo => {
                const {
                    &[
                        slot(
                            "Sammeln",
                            0,
                            UiAction::Order(Tool::Hippo, 0),
                            "Eine Quelle wählen",
                        ),
                        slot(
                            "Hierher",
                            14,
                            UiAction::Order(Tool::Hippo, 6),
                            "Einen freien Platz wählen",
                        ),
                        slot(
                            "Streicheln",
                            8,
                            UiAction::Do(Action::Pet),
                            "Pflege gibt etwas Kraft",
                        ),
                        slot(
                            "Loben",
                            13,
                            UiAction::Do(Action::Praise),
                            "Lob belohnt eine Aufgabe",
                        ),
                        slot(
                            "Snack",
                            19,
                            UiAction::Do(Action::Feed),
                            "2 Nahrung · gibt Kraft",
                        ),
                        slot(
                            "Pause",
                            8,
                            UiAction::Do(Action::Rest),
                            "Beendet den Auftrag",
                        ),
                        slot(
                            "Wache",
                            11,
                            UiAction::Order(Tool::Hippo, 1),
                            "Einen Posten wählen",
                        ),
                        slot(
                            "Feldpflege",
                            19,
                            UiAction::Order(Tool::Hippo, 2),
                            "Ein Feld wählen",
                        ),
                    ]
                }
            }
            Self::Helpers => {
                const {
                    &[
                        slot(
                            "Sammeln",
                            0,
                            UiAction::Order(Tool::Helpers, 0),
                            "Eine Quelle wählen",
                        ),
                        slot(
                            "Ausbilden",
                            7,
                            UiAction::Order(Tool::Helpers, 1),
                            "Am Wachhaus",
                        ),
                        slot(
                            "Patrouille",
                            11,
                            UiAction::Order(Tool::Helpers, 2),
                            "Einen Posten wählen",
                        ),
                        slot(
                            "Zurückrufen",
                            14,
                            UiAction::Do(Action::RecallHelpers),
                            "Sammelaufträge beenden",
                        ),
                    ]
                }
            }
            Self::Paths => {
                const {
                    &[
                        slot(
                            "Weg bauen",
                            15,
                            UiAction::Order(Tool::Plan, 0),
                            "Anfang und Ende wählen",
                        ),
                        slot(
                            "Ausbau",
                            6,
                            UiAction::Order(Tool::Plan, 1),
                            "Gebäude am Weg wählen",
                        ),
                    ]
                }
            }
        }
    }

    pub fn initial(self, tool: Tool, option: usize) -> UiAction {
        let entries = self.entries();
        if self == Self::Tools {
            let index = match tool {
                Tool::Hand => 0,
                Tool::Build => 1,
                Tool::Miracles => 2,
                Tool::Hippo => 3,
                Tool::Helpers => 4,
                Tool::Plan => 5,
            };
            return entries[index].action;
        }
        let active = if tool == Tool::Build {
            UiAction::SelectBuilding(Building::from_index(option))
        } else {
            UiAction::Order(tool, option)
        };
        entries
            .iter()
            .find(|entry| entry.action == active)
            .unwrap_or(&entries[0])
            .action
    }

    fn index(self, current: Option<UiAction>) -> usize {
        self.entries()
            .iter()
            .position(|entry| Some(entry.action) == current)
            .unwrap_or(0)
    }

    pub fn step(self, current: Option<UiAction>, amount: i32) -> UiAction {
        let entries = self.entries();
        let index = (self.index(current) as i32 + amount).rem_euclid(entries.len() as i32);
        entries[index as usize].action
    }

    pub fn aim(self, current: Option<UiAction>, stick: Vec2) -> UiAction {
        let entries = self.entries();
        let previous = self.index(current);
        if stick.length_squared() < 0.50 * 0.50 {
            return entries[previous].action;
        }
        let count = entries.len() as f32;
        let sector = stick.x.atan2(stick.y) * count / TAU;
        let distance = (sector - previous as f32 + count * 0.5).rem_euclid(count) - count * 0.5;
        if distance.abs() <= 0.56 {
            return entries[previous].action;
        }
        let index = (sector.round() as i32).rem_euclid(entries.len() as i32) as usize;
        entries[index].action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wheel_keeps_the_selection_on_stick_release_and_wraps_by_buttons() {
        let wheel = Wheel::Tools;
        let building = submenu(Wheel::Build);
        assert_eq!(wheel.aim(None, Vec2::ONE), building);
        assert_eq!(wheel.aim(Some(building), Vec2::splat(0.1)), building);
        assert_eq!(wheel.aim(Some(building), Vec2::X), submenu(Wheel::Miracles));
        assert_eq!(
            Wheel::Build.initial(Tool::Build, Building::Field.index()),
            UiAction::SelectBuilding(Building::Field)
        );
        assert_eq!(wheel.step(None, -1), UiAction::Panel(Panel::Quests));
        assert_eq!(
            wheel.step(Some(UiAction::Panel(Panel::Quests)), 1),
            UiAction::Tool(Tool::Hand)
        );
    }
}
