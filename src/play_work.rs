use super::*;
use crate::game::{Blueprint, WorkStage};

impl Play {
    pub(super) fn open_work(&mut self, game: &Game) {
        self.open_panel(Panel::Work);
        self.open_work_after_action(game);
    }
    pub(super) fn open_work_after_action(&mut self, game: &Game) {
        let c = game.city();
        let r = game.region();
        self.focus = Some(match c.work.stage {
            WorkStage::Offered => UiAction::Do(Action::AcceptWork),
            WorkStage::Supplies if c.stock[Blueprint::on(r).material.index()] > 0 => {
                UiAction::Do(Action::SupplyWork)
            }
            WorkStage::Raising if c.builder_present(r) && c.energy >= 12. => {
                UiAction::Do(Action::RaiseWork)
            }
            WorkStage::Raising if c.builder_en_route(r) => UiAction::Focus(c.walkers[0].tile()),
            WorkStage::Raising if !c.builder_present(r) => UiAction::Do(Action::SummonBuilder),
            WorkStage::Opening if c.work_partner(r) => UiAction::Panel(Panel::Dedication),
            WorkStage::Running => UiAction::Focus(r.depot),
            _ => UiAction::WorkGuide,
        });
    }
    pub(super) fn guide_work(&mut self, game: &Game, view: &mut CameraRig, now: f32) {
        let c = game.city();
        let r = game.region();
        let b = Blueprint::on(r);
        let (target, msg) = match c.work.stage {
            WorkStage::Supplies if c.stock[b.material.index()] == 0 => {
                self.tool(Tool::Hand);
                if c.held.is_some() {
                    (
                        c.nearest_store(r, self.aim),
                        "Bündel im Lager ablegen. Danach im Inselwerk liefern.".into(),
                    )
                } else if let Some((_, s)) = r
                    .sources
                    .iter()
                    .enumerate()
                    .filter(|(i, s)| {
                        s.material == b.material
                            && c.nodes[*i] > 0
                            && c.influence(r, s.tile)
                            && c.harvest_at(r, s.tile) == Some(*i)
                    })
                    .min_by_key(|(_, s)| s.tile.distance(r.depot))
                {
                    (
                        s.tile,
                        format!(
                            "{} greifen und am Sammelplatz ablegen.",
                            r.kind.material(b.material)
                        ),
                    )
                } else {
                    (r.depot,"Die verfügbaren Quellen liegen außerhalb der Pfote. Schicke einen Sammler über das Objektmenü.".into())
                }
            }
            WorkStage::Raising if c.builder_present(r) && c.energy < 12. => {
                self.tool(Tool::Hippo);
                self.choose(4);
                (
                    c.walkers[0].tile(),
                    "Füttere dein Nilpferd oder lass es bis 12 Kraft ruhen.".into(),
                )
            }
            WorkStage::Opening if !c.work_partner(r) => {
                if c.count(b.partner) > 0 {
                    if let Some((start, end, price)) = c.work_connection(r, b.partner) {
                        self.tool(Tool::Plan);
                        self.choose(0);
                        self.road_anchor = Some(start);
                        (
                            end,
                            format!(
                                "Weg zu {} bestätigen · {} Stein.",
                                b.partner.name(),
                                price[1]
                            ),
                        )
                    } else if let Some(tile) = self.guide_cost(game, [0, 4, 0]) {
                        (
                            tile,
                            "Besorge Stein für den Weg. Ein Stein reicht für drei Wegfelder."
                                .into(),
                        )
                    } else {
                        self.tool(Tool::Plan);
                        self.choose(0);
                        let tile = c
                            .buildings
                            .iter()
                            .find(|s| s.kind == b.partner)
                            .unwrap()
                            .tile;
                        (
                            tile,
                            format!(
                                "Verlängere den Dorfweg in Richtung {}. Freier Boden und höchstens zwölf Felder pro Abschnitt sind nötig.",
                                b.partner.name()
                            ),
                        )
                    }
                } else {
                    let prerequisite = match b.partner {
                        Building::Well | Building::Workshop if c.city_rank() < 2 => {
                            [Building::Store, Building::Field]
                                .into_iter()
                                .find(|&k| c.count(k) == 0)
                        }
                        Building::Granary | Building::Market | Building::Tower
                            if c.city_rank() < 3 =>
                        {
                            Some(if c.city_rank() < 2 {
                                if c.count(Building::Store) == 0 {
                                    Building::Store
                                } else {
                                    Building::Field
                                }
                            } else {
                                Building::Workshop
                            })
                        }
                        _ => None,
                    };
                    let kind = prerequisite.unwrap_or(b.partner);
                    let tile = self.guide_building(game, kind, true);
                    let hint = if self.tool == Tool::Hand {
                        "Material für den Bau holen".to_owned()
                    } else if prerequisite.is_some() {
                        format!(
                            "Zuerst {} bauen, um {} freizuschalten.",
                            kind.name(),
                            b.partner.name()
                        )
                    } else {
                        format!(
                            "{} an den Dorfweg setzen und den Bau bestätigen.",
                            kind.name()
                        )
                    };
                    (tile, hint)
                }
            }
            _ => {
                self.open_work(game);
                return;
            }
        };
        self.close_panels();
        self.aim = target;
        self.pointer = None;
        view.focus_action(target);
        self.guide_marker = Some((target, now + 10.));
        self.notify(msg, Feedback::Selection, now + 6.);
    }
}
