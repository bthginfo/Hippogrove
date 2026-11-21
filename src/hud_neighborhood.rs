use super::*;
use crate::region::Tile;

pub(super) fn atlas(
    p: &mut ChildSpawnerCommands,
    h: &Hud,
    g: &Game,
    page: usize,
    aim: Tile,
    short: bool,
) {
    let city = g.city();
    let homes: Vec<_> = city
        .buildings
        .iter()
        .filter(|s| s.kind == Building::House)
        .collect();
    text(
        p,
        h,
        &format!(
            "{} Wohnhäuser · {} mit Bürgerrecht",
            homes.len(),
            city.citizens.len()
        ),
        if short { 17. } else { 22. },
        INK,
    );
    text(
        p,
        h,
        "Prüfe die Wege zu Wasser, Nahrung und Gemeinschaft. Ein Bürgerhaus braucht alle drei.",
        if short { 13. } else { 16. },
        MUTED,
    );
    let per = 3;
    let pages = homes.len().div_ceil(per).max(1);
    let page = page.min(pages - 1);
    for (n, home) in homes.iter().enumerate().skip(page * per).take(per) {
        let a = city.amenities(g.region(), home.tile);
        let caption = format!(
            "{} {}  ·  {}/3 Bedürfnisse",
            if city.citizens.contains(&home.tile) {
                "Bürgerhaus"
            } else {
                "Wohnhaus"
            },
            n + 1,
            a.score()
        );
        styled_button(
            p,
            h,
            &caption,
            UiAction::Panel(Panel::Household(home.tile)),
            false,
            false,
            ButtonStyle {
                compact: true,
                icon: Some(6),
                height: Some(35.),
                ..default()
            },
        );
    }
    p.spawn(row()).with_children(|nav| {
        if page > 0 {
            button(
                nav,
                h,
                "Vorherige",
                UiAction::Panel(Panel::Town(page - 1)),
                false,
                false,
            );
        }
        if page + 1 < pages {
            button(
                nav,
                h,
                "Weitere Häuser",
                UiAction::Panel(Panel::Town(page + 1)),
                false,
                false,
            );
        }
        styled_button(
            nav,
            h,
            "Standort prüfen",
            UiAction::Panel(Panel::Household(aim)),
            false,
            false,
            ButtonStyle {
                compact: true,
                icon: Some(14),
                height: Some(35.),
                ..default()
            },
        );
    });
}

pub(super) fn home(p: &mut ChildSpawnerCommands, h: &Hud, g: &Game, tile: Tile, short: bool) {
    let city = g.city();
    let r = g.region();
    let site = city.building_at(tile).filter(|s| s.kind == Building::House);
    let t = site.map_or(tile, |s| s.tile);
    let a = city.amenities(r, t);
    let status = if !a.roads {
        "Kein Weg an der Haustür. Verbinde Haus und Dienstgebäude."
    } else {
        "Bis zu zwölf verbundene Wegfelder zwischen Haustür und Dienstgebäude."
    };
    text(p, h, status, if short { 13. } else { 16. }, INK);
    let style = ButtonStyle {
        compact: true,
        height: Some(if short { 31. } else { 39. }),
        ..default()
    };
    for (i, (name, kind, hint)) in crate::game::SERVICES.iter().enumerate() {
        p.spawn((
            Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(7.),
                padding: UiRect::axes(Val::Px(6.), Val::Px(2.)),
                border: UiRect::left(Val::Px(4.)),
                ..default()
            },
            BackgroundColor(rgb(if a.met(i) { 0xe3ecd4 } else { 0xf1dfbf })),
            BorderColor(rgb(if a.met(i) { 0x658056 } else { 0xa16c39 })),
        ))
        .with_children(|line| {
            building_icon(line, h, r.kind, *kind, if short { 30. } else { 42. });
            line.spawn(Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.,
                flex_basis: Val::Px(0.),
                min_width: Val::Px(0.),
                align_items: AlignItems::Start,
                ..default()
            })
            .with_children(|copy| {
                let detail = match a.providers[i] {
                    Some((_, _)) if !a.met(i) => "Nahrung fehlt".into(),
                    Some((_, d)) => format!("versorgt · {d} Wegfelder"),
                    None => "fehlt".into(),
                };
                text(
                    copy,
                    h,
                    &format!("{name} · {detail}"),
                    if short { 14. } else { 17. },
                    INK,
                );
                if !short || !a.met(i) {
                    text(copy, h, hint, if short { 12. } else { 14. }, MUTED);
                }
            });
            if let Some((at, _)) = a.providers[i] {
                styled_button(line, h, "Zeigen", UiAction::Focus(at), false, false, style);
            } else {
                styled_button(
                    line,
                    h,
                    "Bauplan",
                    UiAction::SelectBuilding(*kind),
                    false,
                    false,
                    style,
                );
            }
        });
    }
    if site.is_some() {
        if city.citizens.contains(&t) {
            text(
                p,
                h,
                "Bürgerhaus · +1 Wohnplatz. Voll versorgt bringt 1 Nahrung je Versorgungsrunde 2 Glauben.",
                14.,
                INK,
            );
        } else {
            let ready = city.charter_ready(r, t);
            let cost = if r.sandbox {
                "im freien Bauen kostenlos"
            } else {
                "6 Holz · 4 Stein · 4 Nahrung"
            };
            text(
                p,
                h,
                &format!("Bürgerrecht · {cost} · +1 Wohnplatz"),
                14.,
                INK,
            );
            if let Err(why) = ready {
                text(p, h, &why, 13., MUTED);
            } else {
                styled_button(
                    p,
                    h,
                    "Bürgerrecht verleihen",
                    UiAction::Do(Action::Charter(t)),
                    false,
                    true,
                    style,
                );
            }
        }
    }
    p.spawn(row()).with_children(|links| {
        if a.score() < 3 {
            styled_button(
                links,
                h,
                "Weg anlegen",
                UiAction::Order(Tool::Plan, 0),
                false,
                false,
                style,
            );
        }
        styled_button(
            links,
            h,
            "Zum Standort",
            UiAction::Focus(t),
            false,
            false,
            style,
        );
        styled_button(
            links,
            h,
            "Stadtplanung",
            UiAction::Panel(Panel::Town(0)),
            false,
            false,
            style,
        );
    });
}
