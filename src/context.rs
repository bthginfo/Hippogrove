use crate::region::Material;
use crate::{
    game::{Action, Building, Crop, Game, Miracle},
    play::{Panel, Play, Target, Tool, UiAction},
    wheel::Wheel,
};

pub struct Choice {
    pub label: String,
    pub action: UiAction,
    pub icon: usize,
}

impl Choice {
    fn new(label: impl Into<String>, action: UiAction, icon: usize) -> Self {
        Self {
            label: label.into(),
            action,
            icon,
        }
    }
}

pub struct ObjectMenu {
    pub title: String,
    pub detail: String,
    pub choices: Vec<Choice>,
}

impl ObjectMenu {
    pub fn at(play: &Play, game: &Game, now: f32) -> Self {
        let city = game.city();
        let region = game.region();
        let tile = play.aim;
        let mut target = Target::default();
        target.refresh(play, game, now);
        let mut menu = Self {
            title: String::new(),
            detail: target.detail,
            choices: Vec::new(),
        };
        let guide = Choice::new("Auftragsziel zeigen", UiAction::Guide, 14);
        let primary = Choice::new(target.label, UiAction::Confirm, 5);
        let source = city.harvest_at(region, tile);
        let site = city.building_at(tile);
        let family = city
            .walkers
            .iter()
            .any(|walker| !walker.hippo && !walker.guard && walker.tile() == tile);
        if let Some(step) = play.practice_step {
            let action = step.next_action(play, game);
            menu.title = step.chapter().1.into();
            if step == crate::practice::Step::Grab && action == UiAction::Confirm {
                menu.title = source
                    .map_or("Holzstoß", |i| {
                        region.kind.material(region.sources[i].material)
                    })
                    .into();
                menu.detail =
                    "Ein Bündel bringt bis zu 6 Holz. Halten direkt am Holzstoß greift bis zu 18."
                        .into();
            } else {
                menu.detail = step.instruction(action, play.gamepad).into();
            }
            menu.choices.push(match action {
                UiAction::Confirm => primary,
                UiAction::QuickWheel(wheel) => {
                    Choice::new(format!("{} öffnen", wheel.title()), action, 12)
                }
                UiAction::Tool(_) => Choice::new("Pfote nehmen", action, 5),
                _ => guide,
            });
            return menu;
        }
        if play.tool != Tool::Hand {
            menu.title = play.tool.choice_name(play.option).into();
            if target.legal {
                menu.choices.push(primary);
            }
            if play.tool == Tool::Build {
                menu.choices
                    .push(Choice::new("Drehen", UiAction::Rotate, 15));
                menu.choices
                    .push(Choice::new("Freien Platz finden", UiAction::ToolTarget, 14));
            }
        } else if let Some(bundle) = city.held {
            menu.title = format!(
                "{} {} in der Pfote",
                bundle.count,
                region.kind.material(region.sources[bundle.source].material)
            );
            if target.legal {
                menu.choices.push(primary);
            } else {
                menu.choices.push(Choice::new(
                    "Ablageziel zeigen",
                    UiAction::Focus(city.carry_destination(region, tile).unwrap_or(region.depot)),
                    14,
                ));
            }
        } else if let Some(c) = city.gatherings.circle.as_ref().filter(|c| c.center == tile) {
            menu.title = c.kind.name().into();
            menu.detail = c.status(city);
            if c.ready(city) {
                menu.choices.push(Choice::new(
                    c.kind.finish(),
                    UiAction::Do(Action::Invoke(false)),
                    c.kind.icon(),
                ));
            }
            menu.choices.push(Choice::new(
                "Festkreis ansehen",
                UiAction::Panel(Panel::Gathering),
                3,
            ));
        } else if play.target(game) == Action::Pet {
            menu.title = "Dein Nilpferd".into();
            menu.detail = format!("{} · {:.0} % Kraft", city.creature_status(), city.energy);
            if city.pet_ready() {
                menu.choices
                    .push(Choice::new("Streicheln", UiAction::Do(Action::Pet), 8));
            }
            if city.lesson.is_some() {
                menu.choices.push(Choice::new(
                    city.supply_lesson(region).map_or_else(
                        || "Für die Arbeit loben".into(),
                        |material| format!("{} · Loben", region.kind.material(material)),
                    ),
                    UiAction::Do(Action::Praise),
                    13,
                ));
            }
            menu.choices.push(Choice::new(
                "Befehle öffnen",
                UiAction::QuickWheel(Wheel::Hippo),
                8,
            ));
        } else if let Some(i) = source {
            let material = region.sources[i].material;
            let raw = region.kind.material(material);
            menu.title = if i >= region.managed_sources {
                format!(
                    "{} · {raw}",
                    if material == Material::Wood {
                        "Baum"
                    } else {
                        "Felsblock"
                    }
                )
            } else {
                raw.into()
            };
            if !city.influence(region, tile) && city.nodes[i] > 0 {
                menu.detail = format!(
                    "{} verfügbar · außerhalb der Pfote. Schicke einen Sammler.",
                    city.nodes[i]
                );
            }
            if target.legal {
                menu.detail = format!("{} verfügbar · Halten greift bis zu 18", city.nodes[i]);
            }
            if target.legal {
                menu.choices.push(Choice::new(
                    "Bündel greifen",
                    UiAction::Do(Action::Hand(tile)),
                    material.index(),
                ));
            }
            if city.nodes[i] > 0 {
                menu.choices.push(Choice::new(
                    "Nilpferd sammeln lassen",
                    UiAction::Do(Action::Assign(tile, true)),
                    8,
                ));
            }
            if city.idle_helper() && city.nodes[i] > 0 {
                menu.choices.push(Choice::new(
                    "Dorfbewohner einteilen",
                    UiAction::Do(Action::Assign(tile, false)),
                    11,
                ));
            }
        } else if city.quest == crate::game::Quest::Bridge
            && tile.within(region.bridge(), 1)
            && city.count(Building::Shrine) > 0
        {
            menu.title = region.crossing.to_string();
            menu.detail = city.objective(region).1;
            if target.legal && !city.gift_given {
                menu.choices.push(primary);
            }
            if !target.legal || city.gift_given {
                menu.choices
                    .push(Choice::new("Bündnis vorbereiten", UiAction::Guide, 14));
            }
            menu.choices
                .push(Choice::new("Wachen vertreiben", UiAction::Confront, 9));
        } else if let Some(index) = region.district_at(tile) {
            let district = &region.districts[index];
            menu.title = district.name.clone();
            menu.choices.push(Choice::new(
                "Dorfrat und Besucher",
                UiAction::Panel(Panel::Village(index)),
                7,
            ));
            if city.has_trading_partner(region, index) {
                let offer = city.trade_offer(region, index);
                menu.detail = city
                    .trade_ready(region, index, false)
                    .err()
                    .unwrap_or_else(|| {
                        format!("Ein Handel pro Spieltag · {}", offer.terms(region.kind))
                    });
                if city.trade_ready(region, index, false).is_ok() {
                    menu.choices.push(Choice::new(
                        offer.terms(region.kind),
                        UiAction::Do(Action::Trade(index, false)),
                        12,
                    ));
                }
                if city.trade_ready(region, index, true).is_ok() {
                    menu.choices.push(Choice::new(
                        format!(
                            "Tribut nehmen · {} {} · −6 Stimmung",
                            offer.quantity,
                            region.kind.material(offer.gives)
                        ),
                        UiAction::Do(Action::Trade(index, true)),
                        9,
                    ));
                }
                menu.choices
                    .push(Choice::new("Landkarte", UiAction::Panel(Panel::Map), 14));
                return menu;
            }
            menu.detail = match city.district_plan(region, index) {
                Ok(outcome) => {
                    menu.choices.push(primary);
                    outcome.name().into()
                }
                Err(reason) => reason,
            };
            if city.outcome.is_some() && city.districts[index].outcome.is_none() {
                if city.district_plan(region, index).is_err() {
                    menu.choices.push(Choice::new(
                        format!(
                            "{} vorbereiten",
                            crate::game::Settlement::district_need(district.need)
                        ),
                        UiAction::HelpDistrict(index),
                        6,
                    ));
                }
                if city.districts[index].resolve > 0 {
                    menu.choices
                        .push(Choice::new("Windstoß anvisieren", UiAction::Confront, 9));
                }
            }
        } else if let Some(site) = site {
            menu.title = site.kind.name().into();
            if let Some(name) = site.kind.civic().and_then(crate::civic::Civic::production) {
                if target.legal {
                    menu.choices
                        .push(Choice::new(name, UiAction::Do(Action::Produce(tile)), 12));
                }
            }
            if city.influence(region, tile) && site.kind == Building::Field {
                match site.crop {
                    Crop::Dry => menu.choices.push(Choice::new(
                        format!("Regen · {:.0} Kraft", Miracle::Rain.cost()),
                        UiAction::Do(Action::Cast(Miracle::Rain, tile)),
                        9,
                    )),
                    Crop::Ripe => menu.choices.push(Choice::new(
                        "Ernte einbringen",
                        UiAction::Do(Action::Hand(tile)),
                        2,
                    )),
                    Crop::Growing(_) => {}
                }
            }
            menu.choices
                .push(Choice::new("Gebäudedetails", UiAction::Inspect, 12));
        } else if target.legal {
            menu.title = primary.label.clone();
            menu.choices.push(primary);
        } else if family {
            menu.title = "Stimme aus dem Dorf".into();
            menu.detail = city.objective(region).0.into();
            menu.choices.push(Choice::new(
                "Bitte ansehen",
                UiAction::Panel(Panel::Quests),
                11,
            ));
        } else if city.influence(region, tile) && city.walkable(region, tile) {
            menu.title = "Dorfgrund".into();
            menu.detail = "Wähle einen Bauplan für die Vorschau.".into();
            menu.choices.push(Choice::new(
                "Hier bauen",
                UiAction::QuickWheel(Wheel::Build),
                6,
            ));
        } else {
            menu.title = "Landschaft".into();
            menu.detail = "Hier erreicht die Pfote keinen Gegenstand.".into();
        }
        if play.tool == Tool::Hand
            && city.held.is_none()
            && (site.is_some_and(|site| site.kind == Building::House)
                || (family
                    && site.is_none()
                    && source.is_none()
                    && play.target(game) == Action::Hand(tile)))
        {
            if let Err(reason) = city.meal_ready(region, tile) {
                menu.detail = format!("{} · {reason}", menu.detail);
            } else {
                menu.choices.push(Choice::new(
                    "Essen teilen · 3 Nahrung",
                    UiAction::Do(Action::ShareMeal(tile)),
                    2,
                ));
            }
        }
        if menu.choices.is_empty() {
            menu.choices.push(guide);
        }
        menu
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{modes::Mode, play::CameraRig, region::Biome};

    #[test]
    fn dry_field_offers_rain_then_harvest_in_the_same_world_menu() {
        let mut game = Game::start(Mode::Campaign(Biome::Valley));
        game.campaign.intro_seen = true;
        let tile = game
            .region()
            .bounds
            .tiles()
            .find(|&tile| game.building_plan(Building::Field, tile).is_ok())
            .unwrap();
        game.act(Action::Build(Building::Field, tile, 0)).unwrap();
        let mut play = Play::new(&game);
        let mut view = CameraRig::new(&game);
        play.aim = tile;
        play.inspect_world(&game, 1.);
        assert_eq!(play.panel, Panel::Context);
        let rain = UiAction::Do(Action::Cast(Miracle::Rain, tile));
        assert_eq!(play.focus, Some(rain));
        let mana = game.city().mana;
        assert!(play.activate(rain, &mut game, &mut view, 1.));
        assert_eq!(play.panel, Panel::Closed);
        assert!(game.city().mana < mana);
        for _ in 0..960 {
            game.step(1. / 24.);
        }
        play.inspect_world(&game, 42.);
        let harvest = UiAction::Do(Action::Hand(tile));
        assert_eq!(play.focus, Some(harvest));
        let before = game.city().stock[2];
        assert!(play.activate(harvest, &mut game, &mut view, 42.));
        assert!(game.city().stock[2] > before);
        assert!(play.receipt.as_ref().unwrap().2.contains("Glauben"));
    }
}
