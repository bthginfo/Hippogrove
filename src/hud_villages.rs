use super::*;
use crate::game::Allegiance;

fn compact(short: bool) -> ButtonStyle {
    ButtonStyle {
        compact: true,
        height: Some(if short { 34. } else { 45. }),
        ..default()
    }
}

pub(super) fn council(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, short: bool) {
    let city = game.city();
    let race = &city.contest;
    let region = game.region();
    let style = compact(short);
    let ours = race.count(Allegiance::Ours);
    let other = race.count(Allegiance::Rival);
    let goal = if region.sandbox {
        "Besuche die freien Dörfer".into()
    } else if race.pact {
        "Inselbund besiegelt".into()
    } else {
        format!("Inselbund · {ours}/{} Dörfer", race.majority())
    };
    parent.spawn(row()).with_children(|bar| {
        text(bar, hud, &goal, if short { 17. } else { 22. }, INK);
        if !region.sandbox {
            text(bar, hud, &format!("Rivale {other}"), 15., MUTED);
        }
    });
    let intent = race.destination.map_or_else(
        || {
            if region.sandbox {
                "Dein Nilpferd und deine Gläubigen können jeden Ort besuchen.".into()
            } else if race.pact {
                "Die Dörfer haben Frieden geschlossen.".into()
            } else if ours >= race.majority() {
                "Die Mehrheit steht. Du kannst jetzt den Inselbund schließen.".into()
            } else if ours > 0 {
                "Ein verbündetes Dorf kann Pilger schicken. Wähle ihren Zielort.".into()
            } else if let Some((_, v)) = region.villages.iter().enumerate().find(|(i, _)| {
                city.visitor(*i, true).is_some() || city.visitor(*i, false).is_some()
            }) {
                format!(
                    "Besuch in {}. Öffne den Dorfplatz für Gäste, Gaben und Vertrauen.",
                    v.name
                )
            } else {
                "Wähle ein Dorf und schicke deinen ersten Besucher.".into()
            }
        },
        |i| {
            let v = &region.villages[i];
            if race
                .rival_tile
                .is_some_and(|tile| tile.distance(v.center) <= 1)
            {
                format!(
                    "Gesandte werben in {}. Gemeinsam gewinnen deine Besucher mehr Vertrauen.",
                    v.name
                )
            } else {
                format!(
                    "Der Gesandte zieht nach {}. Du kannst ihm zuvorkommen.",
                    v.name
                )
            }
        },
    );
    let intent = if let Some(a) = &city.projects.appeal {
        format!(
            "{} bittet um Hilfe · {}",
            region.villages[a.village].name,
            crate::game::Need::on(region.kind).title
        )
    } else if race.occupation >= crate::game::OCCUPATION_GRACE {
        "Der Rivale fordert Tribut. Brich seine Mehrheit.".into()
    } else if race.occupation > 0. {
        format!(
            "Rivalenmehrheit · Noch {:.0} s bis zum Tribut.",
            (crate::game::OCCUPATION_GRACE - race.occupation).ceil()
        )
    } else {
        intent
    };
    text(parent, hud, &intent, if short { 13. } else { 16. }, MUTED);
    let mut villages: Vec<_> = region.villages.iter().enumerate().collect();
    villages.sort_by_key(|(_, v)| v.center.distance(region.start));
    parent
        .spawn(Node {
            column_gap: Val::Px(8.),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|body| {
            body.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(4.)),
                    ..default()
                },
                BackgroundColor(rgb(DARK)),
                BorderRadius::all(Val::Px(9.)),
            ))
            .with_children(|map| {
                map_chart(map, hud, game, short, true);
                text(
                    map,
                    hud,
                    "Heimat · Dörfer · Rivalensitz",
                    if short { 10. } else { 13. },
                    LIGHT,
                );
            });
            let mut grid = column_grid(2);
            grid.width = Val::Auto;
            grid.flex_grow = 1.;
            grid.min_width = Val::Px(0.);
            grid.row_gap = Val::Px(5.);
            body.spawn(grid).with_children(|list| {
                for (i, v) in villages {
                    let side = race.towns[i].side;
                    let color = match side {
                        Allegiance::Ours => 0x69a97d,
                        Allegiance::Neutral => 0xbfa97b,
                        Allegiance::Rival => 0xad648d,
                    };
                    list.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            min_width: Val::Px(0.),
                            border: UiRect::left(Val::Px(3.)),
                            padding: UiRect::horizontal(Val::Px(3.)),
                            ..default()
                        },
                        BorderColor(rgb(color)),
                    ))
                    .with_children(|card| {
                        styled_button(
                            card,
                            hud,
                            &v.name,
                            UiAction::Panel(Panel::Village(i)),
                            false,
                            false,
                            ButtonStyle {
                                icon: Some(v.need.index()),
                                ..style
                            },
                        );
                        text(
                            card,
                            hud,
                            &race.status(i),
                            if short { 11. } else { 14. },
                            MUTED,
                        );
                    });
                }
            });
        });
    parent.spawn(row()).with_children(|bar| {
        if let Some(a) = &city.projects.appeal {
            styled_button(
                bar,
                hud,
                "Dorfbitte ansehen",
                UiAction::Panel(Panel::Projects(a.village)),
                false,
                true,
                style,
            );
        }
        styled_button(
            bar,
            hud,
            "Landkarte",
            UiAction::Panel(Panel::Map),
            false,
            false,
            style,
        );
        if !region.sandbox {
            styled_button(
                bar,
                hud,
                "Rivalensitz",
                UiAction::Focus(region.rival_home()),
                false,
                false,
                style,
            );
        }
        if !region.sandbox && city.quest != Quest::Complete && ours >= race.majority() {
            styled_button(
                bar,
                hud,
                "Inselbund schließen",
                UiAction::Do(Action::IslandPact),
                false,
                true,
                style,
            );
        }
    });
}

pub(super) fn village(
    parent: &mut ChildSpawnerCommands,
    hud: &Hud,
    game: &Game,
    i: usize,
    short: bool,
) {
    let city = game.city();
    let region = game.region();
    let v = &region.villages[i];
    let t = &city.contest.towns[i];
    let style = compact(short);
    let goods = region.kind.material(v.need);
    let report = if t.side == Allegiance::Ours {
        format!("Baurecht im Umkreis · 2 {goods} je Lieferung")
    } else {
        format!("Die Familien bitten um 4 {goods}. Besucher am Dorfplatz gewinnen Vertrauen.")
    };
    text(parent, hud, &report, if short { 14. } else { 17. }, INK);
    // Signed loyalty puts a neutral midpoint between the two factions.
    parent
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Px(8.),
            ..default()
        })
        .with_children(|bar| {
            for (width, color) in [
                ((50. + t.trust * 0.5).clamp(0., 100.), JADE),
                ((50. - t.trust * 0.5).clamp(0., 100.), 0x97594d),
            ] {
                bar.spawn((
                    Node {
                        width: Val::Percent(width),
                        height: Val::Percent(100.),
                        ..default()
                    },
                    BackgroundColor(rgb(color)),
                ));
            }
        });
    let enemy = city.contest.destination == Some(i);
    text(
        parent,
        hud,
        &format!(
            "{}{}",
            city.contest.status(i),
            if enemy {
                " · Rivalengesandte auf Mission"
            } else {
                ""
            }
        ),
        14.,
        MUTED,
    );
    if t.side != Allegiance::Ours || enemy {
        text(
            parent,
            hud,
            if t.protection > 0. {
                "Nilpferd +5/s · Gläubiger +3,5/s · Rivale −2/s durch den Rückhalt aus deiner Hilfe."
            } else {
                "Nilpferd +5/s · Gläubiger +3,5/s · Rivale −4/s. Gemeinsam gewinnen deine Besucher schneller Vertrauen."
            },
            if short { 12. } else { 15. },
            MUTED,
        );
    }
    let mut grid = column_grid(2);
    grid.row_gap = Val::Px(5.);
    parent.spawn(grid).with_children(|grid| {
        for (hippo, who, icon) in [(true, "Nilpferd", 8), (false, "Gläubige", 7)] {
            let visitor = city.visitor(i, hippo);
            let word = visitor.map_or_else(
                || format!("{who} schicken"),
                |w| {
                    if city.visiting(w, region, i) {
                        format!("{who} · am Platz")
                    } else {
                        format!("{who} · unterwegs")
                    }
                },
            );
            let action = visitor.map_or(UiAction::Do(Action::Visit(i, hippo)), |w| {
                UiAction::Focus(w.tile())
            });
            styled_button(
                grid,
                hud,
                &word,
                action,
                false,
                false,
                ButtonStyle {
                    icon: Some(icon),
                    ..style
                },
            );
        }
        styled_button(
            grid,
            hud,
            "Dorfplatz zeigen",
            UiAction::Focus(v.center),
            false,
            false,
            ButtonStyle {
                icon: Some(14),
                ..style
            },
        );
        if city.gift_ready(region, i).is_ok() {
            styled_button(
                grid,
                hud,
                &format!("4 {goods} schenken"),
                UiAction::Do(Action::VillageGift(i)),
                false,
                true,
                ButtonStyle {
                    icon: Some(v.need.index()),
                    ..style
                },
            );
        } else if city.visitor(i, true).is_some() || city.visitor(i, false).is_some() {
            styled_button(
                grid,
                hud,
                "Besucher heimrufen",
                UiAction::Do(Action::Withdraw(i)),
                false,
                false,
                style,
            );
        }
        let help = city
            .projects
            .appeal
            .as_ref()
            .is_some_and(|a| a.village == i);
        styled_button(
            grid,
            hud,
            if help {
                "Hilferuf ansehen"
            } else {
                "Pilgerzüge"
            },
            UiAction::Panel(Panel::Projects(i)),
            false,
            help,
            style,
        );
        if i < region.districts.len() && city.trade_ready(region, i, false).is_ok() {
            styled_button(
                grid,
                hud,
                &city.trade_offer(region, i).terms(region.kind),
                UiAction::Do(Action::Trade(i, false)),
                false,
                false,
                style,
            );
        }
    });
}

pub(super) fn projects(
    parent: &mut ChildSpawnerCommands,
    hud: &Hud,
    game: &Game,
    i: usize,
    short: bool,
) {
    let city = game.city();
    let r = game.region();
    let style = compact(short);
    if city
        .projects
        .appeal
        .as_ref()
        .is_some_and(|a| a.village == i)
    {
        let need = crate::game::Need::on(r.kind);
        text(parent, hud, need.title, if short { 16. } else { 21. }, INK);
        text(
            parent,
            hud,
            need.request,
            if short { 13. } else { 16. },
            MUTED,
        );
        parent.spawn(column_grid(2)).with_children(|grid| {
            for (hippo, title, effect) in [
                (
                    true,
                    need.feat.to_owned(),
                    "18 Kreaturenkraft · 4 Glauben und bis zu 55 Vertrauen".to_owned(),
                ),
                (
                    false,
                    "Reparaturtrupp bilden".into(),
                    format!(
                        "4 {} · 4 Glauben, bis zu 35 Vertrauen und 60 s halber Rivalendruck",
                        r.kind.material(need.material)
                    ),
                ),
            ] {
                grid.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    min_width: Val::Px(0.),
                    ..default()
                })
                .with_children(|card| {
                    let ready = city.aid_ready(r, i, hippo);
                    if ready.is_err() && city.visitor(i, hippo).is_none() {
                        let who = if hippo {
                            "Nilpferd schicken"
                        } else {
                            "Gläubigen schicken"
                        };
                        styled_button(
                            card,
                            hud,
                            who,
                            UiAction::Do(Action::Visit(i, hippo)),
                            false,
                            false,
                            style,
                        );
                    } else {
                        styled_button(
                            card,
                            hud,
                            &title,
                            UiAction::Do(Action::Aid(i, hippo)),
                            false,
                            ready.is_ok(),
                            style,
                        );
                    }
                    text(card, hud, &effect, if short { 12. } else { 15. }, INK);
                    if let Err(reason) = ready {
                        text(card, hud, &reason, if short { 11. } else { 13. }, MUTED);
                    }
                });
            }
        });
    }
    if city
        .projects
        .appeal
        .as_ref()
        .is_some_and(|a| a.village == i)
    {
        button(
            parent,
            hud,
            "Später helfen",
            UiAction::Do(Action::DeferAppeal(i)),
            false,
            false,
        );
        return;
    }
    if let Some(p) = &city.projects.pilgrims {
        text(
            parent,
            hud,
            &format!(
                "Pilger von {} nach {} · Empfang am Ziel bringt +55 Vertrauen.",
                r.villages[p.from].name, r.villages[p.to].name
            ),
            14.,
            INK,
        );
        styled_button(
            parent,
            hud,
            "Pilgerzug zeigen",
            UiAction::Focus(p.tile),
            false,
            false,
            style,
        );
    } else if let Ok(origin) = city.sponsor(r, i) {
        let s = &r.villages[origin];
        text(
            parent,
            hud,
            &format!(
                "{} gibt 20 Bindung ab. Am Ziel bringen die Pilger +30 Vertrauen, mit Empfang +55.",
                s.name
            ),
            if short { 13. } else { 16. },
            MUTED,
        );
        styled_button(
            parent,
            hud,
            "Pilger schicken · 4 Nahrung",
            UiAction::Do(Action::SendPilgrims(i)),
            false,
            false,
            style,
        );
    } else if city.projects.appeal.as_ref().is_none_or(|a| a.village != i) {
        text(
            parent,
            hud,
            "Pilger bringen den Glauben eines verbündeten Dorfs zum Ziel. Steht dein Nilpferd oder ein Gläubiger am Platz, steigt der Gewinn von 30 auf 55 Vertrauen.",
            15.,
            INK,
        );
        if let Err(reason) = city.sponsor(r, i) {
            text(parent, hud, &reason, 14., MUTED);
        }
    }
}
