use super::*;
use crate::game::{Blueprint, Dedication, WorkStage};

fn compact() -> ButtonStyle {
    ButtonStyle {
        compact: true,
        height: Some(35.),
        ..default()
    }
}

fn strip(p: &mut ChildSpawnerCommands, h: &Hud, title: &str, body: &str, small: bool) {
    p.spawn((
        Node {
            width: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Start,
            padding: UiRect::axes(Val::Px(9.), Val::Px(5.)),
            border: UiRect::left(Val::Px(3.)),
            ..default()
        },
        BackgroundColor(rgb(DARK)),
        BorderColor(rgb(BRASS)),
        BorderRadius::all(Val::Px(8.)),
    ))
    .with_children(|q| {
        text(q, h, title, if small { 15. } else { 19. }, GOLD);
        let e = text(q, h, body, if small { 13. } else { 16. }, LIGHT);
        q.commands()
            .entity(e)
            .insert(TextLayout::new_with_justify(JustifyText::Left));
    });
}

pub(super) fn panel(p: &mut ChildSpawnerCommands, h: &Hud, g: &Game, small: bool) {
    let r = g.region();
    let c = g.city();
    let b = Blueprint::on(r);
    let stage = c.work.stage;
    let st = compact();
    let (step, help) = match stage {
        WorkStage::Offered => ("Der Wunsch", b.request.into()),
        WorkStage::Supplies => (
            "1 · Material liefern",
            format!(
                "{} / {} {} verbaut. Im Lager sind {}. Teillieferungen bleiben am Bauplatz.",
                c.work.delivered,
                b.amount,
                r.kind.material(b.material),
                c.stock[b.material.index()]
            ),
        ),
        WorkStage::Raising => (
            "2 · Gemeinsam bauen",
            if c.builder_present(r) {
                format!(
                    "Dein Nilpferd ist am Bauplatz. Der Einsatz kostet 12 Kraft und bringt 1 Erfahrung. Kraft: {:.0}.",
                    c.energy
                )
            } else if c.builder_en_route(r) {
                "Dein Nilpferd ist auf dem Weg. Sobald es am Bauplatz steht, kannst du mit ihm heben.".into()
            } else {
                "Rufe dein Nilpferd zum Sammelplatz. Es legt seine Ladung ab und läuft selbst zum Bau.".into()
            },
        ),
        WorkStage::Opening => (
            "3 · Werk eröffnen",
            if c.work_partner(r) {
                format!(
                    "{} ist am Weg. Entscheide, wem du das Werk widmest.",
                    b.partner.name()
                )
            } else {
                format!(
                    "Der Betrieb braucht ein Gebäude mit Weganschluss: {}. Ein bereits gebautes zählt.",
                    b.partner.name()
                )
            },
        ),
        WorkStage::Running => (
            if c.work_partner(r) {
                "In Betrieb"
            } else {
                "Betrieb ruht"
            },
            format!("{} · {} Versorgungsrunden.", c.work_hint(r), c.work.rounds),
        ),
    };
    p.spawn(Node {
        align_items: AlignItems::Center,
        column_gap: Val::Px(12.),
        ..default()
    })
    .with_children(|hero| {
        hero.spawn((
            ImageNode::new(h.work_art[r.kind as usize].clone()),
            Node {
                width: Val::Px(if small { 85. } else { 124. }),
                aspect_ratio: Some(256. / 228.),
                flex_shrink: 0.,
                ..default()
            },
        ));
        hero.spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Start,
            min_width: Val::Px(0.),
            flex_grow: 1.,
            flex_basis: Val::Px(0.),
            ..default()
        })
        .with_children(|q| {
            text(q, h, &b.title, if small { 22. } else { 29. }, INK);
            let e = text(
                q,
                h,
                &format!(
                    "{} im Betrieb · alle 17–24 Spielsekunden",
                    Blueprint::effect(r, c.work.dedication == Some(Dedication::Town))
                ),
                if small { 13. } else { 16. },
                MUTED,
            );
            q.commands()
                .entity(e)
                .insert(TextLayout::new_with_justify(JustifyText::Left));
        });
    });
    strip(p, h, step, &help, small);
    p.spawn(row()).with_children(|actions| {
        let (label, primary) = match stage {
            WorkStage::Offered => ("Auftrag annehmen".into(), UiAction::Do(Action::AcceptWork)),
            WorkStage::Supplies if c.stock[b.material.index()] > 0 => (
                format!(
                    "{} {} liefern",
                    c.stock[b.material.index()].min(b.amount - c.work.delivered),
                    r.kind.material(b.material)
                ),
                UiAction::Do(Action::SupplyWork),
            ),
            WorkStage::Supplies => ("Material zeigen".into(), UiAction::WorkGuide),
            WorkStage::Raising if c.builder_en_route(r) => (
                "Nilpferd unterwegs · ansehen".into(),
                UiAction::Focus(c.walkers[0].tile()),
            ),
            WorkStage::Raising if !c.builder_present(r) => (
                "Nilpferd zum Bau rufen".into(),
                UiAction::Do(Action::SummonBuilder),
            ),
            WorkStage::Raising if c.energy < 12. => {
                ("Nilpferd versorgen".into(), UiAction::WorkGuide)
            }
            WorkStage::Raising => (b.raise.into(), UiAction::Do(Action::RaiseWork)),
            WorkStage::Opening if c.work_partner(r) => {
                ("Widmung wählen".into(), UiAction::Panel(Panel::Dedication))
            }
            WorkStage::Opening => ("Anschluss vorbereiten".into(), UiAction::WorkGuide),
            WorkStage::Running => ("Werk auf der Insel zeigen".into(), UiAction::Focus(r.depot)),
        };
        styled_button(actions, h, &label, primary, false, true, st);
        if matches!(
            stage,
            WorkStage::Supplies | WorkStage::Raising | WorkStage::Opening
        ) {
            styled_button(
                actions,
                h,
                if c.work.pinned {
                    "Inselauftrag verfolgen"
                } else {
                    "Werk verfolgen"
                },
                UiAction::Do(Action::TrackWork(!c.work.pinned)),
                false,
                false,
                st,
            );
        }
        if stage == WorkStage::Running {
            styled_button(
                actions,
                h,
                "Mein Fortschritt",
                UiAction::Panel(Panel::Progress),
                false,
                false,
                st,
            );
        }
    });
}

pub(super) fn dedication(p: &mut ChildSpawnerCommands, h: &Hud, g: &Game, small: bool) {
    let r = g.region();
    let c = g.city();
    let b = Blueprint::on(r);
    text(
        p,
        h,
        &format!("{} · Wähle eine dauerhafte Widmung", b.title),
        if small { 17. } else { 21. },
        INK,
    );
    for (d, title, body, icon) in [
        (
            Dedication::Companion,
            "Dem Nilpferd",
            format!(
                "+4 Erfahrung. Dein Nilpferd hat bisher {}.",
                g.campaign.gifts.creature.experience()
            ),
            8,
        ),
        (
            Dedication::Talent,
            "Deiner Gottheit",
            "+1 Talentpunkt für die Zauberschule.".into(),
            9,
        ),
        (
            Dedication::Town,
            "Den Familien",
            format!(
                "Doppelte Leistung · {} je Runde.",
                Blueprint::effect(r, true)
            ),
            6,
        ),
    ] {
        p.spawn(Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(9.),
            ..default()
        })
        .with_children(|q| {
            styled_button(
                q,
                h,
                title,
                UiAction::Do(Action::Dedicate(d)),
                false,
                false,
                ButtonStyle {
                    icon: Some(icon),
                    ..compact()
                },
            );
            text(q, h, &body, if small { 13. } else { 16. }, MUTED);
        });
    }
    if !c.work_partner(r) {
        text(
            p,
            h,
            "Für die Eröffnung fehlt der Weganschluss.",
            14.,
            MUTED,
        );
    }
}

pub(super) fn progress(p: &mut ChildSpawnerCommands, h: &Hud, g: &Game, small: bool) {
    let c = g.city();
    let gifts = g.campaign.gifts;
    let animal = gifts.creature;
    let xp = animal.experience();
    let next = animal.next_stage().map_or_else(
        || "Höchste Wachstumsstufe".into(),
        |n| format!("Noch {} Erfahrung bis zum nächsten Wachstum", n - xp),
    );
    strip(
        p,
        h,
        &format!(
            "{} · {} · {xp} Erfahrung",
            animal.description(),
            animal.nature.tone().name()
        ),
        &format!("{next}. Eine Lieferung gibt 1 Erfahrung, Ernte und erfolgreiche Wache je 2."),
        small,
    );
    let next_city = match c.city_rank() {
        0 => "Baue eine Hütte.",
        1 => "Feld und Lager erschließen Stufe 2.",
        2 => "Eine Werkstatt erschließt Stufe 3.",
        3 => "Ausgebaute Hütte, Markt und Brunnen erschließen Stufe 4.",
        _ => "Versorge die Wohnviertel und verleihe Bürgerrechte.",
    };
    strip(
        p,
        h,
        &format!(
            "Stadtstufe {} · {} Gläubige · {} ({:+})",
            c.city_rank(),
            c.inhabitants(),
            c.reputation.tone().name(),
            c.reputation.score()
        ),
        next_city,
        small,
    );
    text(
        p,
        h,
        &format!(
            "{} Talentpunkte · {} Erkenntnis für Forschung",
            gifts.points, g.campaign.research.insight
        ),
        if small { 14. } else { 18. },
        INK,
    );
    p.spawn(row()).with_children(|q| {
        for (name, act, icon) in [
            ("Zum Nilpferd", UiAction::Focus(c.walkers[0].tile()), 8),
            (
                "Zauberschule",
                UiAction::Panel(Panel::Talent(Talent::Rain)),
                9,
            ),
            ("Stadtplanung", UiAction::Panel(Panel::Town(0)), 6),
            ("Forschung", UiAction::Panel(Panel::Research), 13),
        ] {
            styled_button(
                q,
                h,
                name,
                act,
                false,
                false,
                ButtonStyle {
                    icon: Some(icon),
                    ..compact()
                },
            );
        }
    });
}

pub(super) fn talents(p: &mut ChildSpawnerCommands, h: &Hud, g: &Game, t: Talent, small: bool) {
    let gifts = g.campaign.gifts;
    let rank = gifts.rank(t);
    let n = (rank + 1).min(2);
    p.spawn(column_grid(2)).with_children(|grid| {
        for (talent, short_name) in
            Talent::ALL
                .into_iter()
                .zip(["Regenkreis", "Wunderquelle", "Blüte", "Donner"])
        {
            styled_button(
                grid,
                h,
                &format!("{short_name} · {}/2", gifts.rank(talent)),
                UiAction::Panel(Panel::Talent(talent)),
                t == talent,
                false,
                compact(),
            );
        }
    });
    let effect = match t {
        Talent::Rain => format!("Regen erreicht Felder im Umkreis von {} Feldern.", 2 + n),
        Talent::Spring => format!(
            "Die Wunderquelle speichert {} Kraft, vor Tempelboni.",
            28 + 8 * n
        ),
        Talent::Growth => format!(
            "Blüte lässt Saat reifen und füllt Holz und Nahrung um bis zu {} auf. Radius {}.",
            10 + 8 * n,
            2 + n
        ),
        Talent::Storm => format!(
            "Donner senkt den Widerstand um {}. Radius {}.",
            1 + n,
            3 + n
        ),
    };
    strip(p, h, &format!("{} · Rang {n}", t.name()), &effect, small);
    if rank == 2 {
        text(p, h, "Vollständig gelernt.", 15., MUTED);
    } else if gifts.points == 0 {
        text(
            p,
            h,
            "Dorfbitten erfüllen oder ein Inselwerk deiner Gottheit widmen, um einen Talentpunkt zu verdienen.",
            14.,
            MUTED,
        );
    } else {
        styled_button(
            p,
            h,
            &format!("Rang {n} lernen · 1 von {} Punkten", gifts.points),
            UiAction::Do(Action::Learn(t)),
            false,
            true,
            compact(),
        );
    }
}
