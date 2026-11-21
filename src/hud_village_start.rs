use super::*;
use crate::game::VillageTask;

fn left(parent: &mut ChildSpawnerCommands, hud: &Hud, value: &str, size: f32, color: u32) {
    let id = text(parent, hud, value, size, color);
    parent
        .commands()
        .entity(id)
        .insert(TextLayout::new_with_justify(JustifyText::Left));
}

fn task_words(game: &Game, task: VillageTask, pad: bool) -> (String, String) {
    let region = game.region();
    match task {
        VillageTask::Build(kind) => (
            match kind {
                Building::House => "Eine Hütte bauen".into(),
                Building::Field => "Ein Feld anlegen".into(),
                _ => format!("Ein {} bauen", kind.name()),
            },
            match kind {
                Building::House => "Hier ziehen deine ersten Bewohner ein.",
                Building::Store => "Ein Lager schafft Platz für deine Vorräte.",
                _ => "Auf dem Feld wächst Nahrung für dein Dorf.",
            }
            .into(),
        ),
        VillageTask::Collect(material, count) => (
            format!("{} holen", region.kind.material(material)),
            if pad {
                format!("Dir fehlen {count}. Halte A an der markierten Quelle.")
            } else {
                format!("Dir fehlen {count}. Halte „Greifen“ an der markierten Quelle.")
            },
        ),
        VillageTask::Deposit => (
            "Zum Sammelplatz".into(),
            if pad {
                "Drücke A am Sammelplatz, um das Bündel abzulegen."
            } else {
                "Tippe am Sammelplatz auf „Ablegen“."
            }
            .into(),
        ),
        VillageTask::Water(_) => (
            "Dein Feld wässern".into(),
            "Ein Regenschauer lässt die Saat keimen.".into(),
        ),
        VillageTask::Grow(_) => (
            "Die Saat wächst".into(),
            "Das Feld ist bald reif. Du kannst solange weiterbauen.".into(),
        ),
        VillageTask::Harvest(_) => (
            "Deine erste Ernte".into(),
            if pad {
                "Drücke A über dem goldenen Feld."
            } else {
                "Tippe auf das goldene Feld und wähle „Ernten“."
            }
            .into(),
        ),
        VillageTask::Explore => (
            "Dein Dorf steht".into(),
            "Baue in Ruhe weiter oder lerne ein Nachbardorf kennen.".into(),
        ),
        VillageTask::Visiting(i) if game.city().visitor(i, true).is_none() => (
            "Besuch fortsetzen".into(),
            format!(
                "Schick dein Nilpferd erneut zum Dorfplatz in {}.",
                region.villages[i].name
            ),
        ),
        VillageTask::Visiting(i) => (
            format!("Besuch in {}", region.villages[i].name),
            "Dein Nilpferd geht zum Dorfplatz. Dort lernt es die Bewohner kennen.".into(),
        ),
        VillageTask::Acquainted(i) => (
            format!("Willkommen in {}", region.villages[i].name),
            "Du kannst weiterbauen, Dörfer besuchen oder die Rivalität beginnen.".into(),
        ),
    }
}

pub(super) fn tracker(
    parent: &mut ChildSpawnerCommands,
    hud: &Hud,
    game: &Game,
    layout: Layout,
    task: VillageTask,
) {
    let short = layout.short;
    let edge = if short { 8. } else { EDGE };
    let (title, hint) = task_words(game, task, layout.gamepad);
    let step = (game.city().village_milestones() + 1).min(4);
    let local = !matches!(
        task,
        VillageTask::Explore | VillageTask::Visiting(_) | VillageTask::Acquainted(_)
    );
    parent
        .spawn((
            HitArea,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(if layout.compact { 65. } else { 73. }),
                left: Val::Px(edge),
                width: Val::Px(if short { 232. } else { 297. }),
                max_width: Val::Percent(if short { 53. } else { 43. }),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                padding: UiRect::axes(Val::Px(if short { 9. } else { 13. }), Val::Px(8.)),
                row_gap: Val::Px(if short { 3. } else { 6. }),
                border: UiRect::left(Val::Px(3.)),
                ..default()
            },
            BackgroundColor(rgb(DARK).with_alpha(0.97)),
            BorderColor(rgb(BRASS)),
            BorderRadius::all(Val::Px(9.)),
        ))
        .with_children(|card| {
            if local {
                left(
                    card,
                    hud,
                    &format!("Dorfaufbau · {step} von 4"),
                    if short { 12. } else { 14. },
                    JADE,
                );
            }
            left(card, hud, &title, if short { 17. } else { 22. }, GOLD);
            left(card, hud, &hint, if short { 12. } else { 15. }, LIGHT);
            let next = match task {
                VillageTask::Explore => "Nachbardorf kennenlernen",
                VillageTask::Acquainted(_) => "Rivalität ansehen",
                VillageTask::Visiting(_) => "Besuch zeigen",
                VillageTask::Build(_) => "Bauplatz zeigen",
                VillageTask::Collect(..) => "Quelle zeigen",
                VillageTask::Deposit => "Sammelplatz zeigen",
                VillageTask::Water(_) | VillageTask::Grow(_) | VillageTask::Harvest(_) => {
                    "Feld zeigen"
                }
            };
            let name = if layout.gamepad {
                format!("L3 · {next}")
            } else {
                next.into()
            };
            styled_button(
                card,
                hud,
                &name,
                UiAction::Guide,
                false,
                matches!(task, VillageTask::Explore),
                ButtonStyle {
                    compact: true,
                    height: Some(if short { 31. } else { 38. }),
                    ..default()
                },
            );
        });
}

pub(super) fn briefing(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, short: bool) {
    let region = game.region();
    island_heading(
        parent,
        hud,
        region.kind,
        &region.name,
        "Dein neues Dorf",
        short,
    );
    left(
        parent,
        hud,
        "Gib deinen Bewohnern zuerst ein Dach. Danach legst du Vorräte und ein Feld an.",
        17.,
        INK,
    );
    left(
        parent,
        hud,
        "Oben links steht der nächste Schritt. Du bestimmst, wann du die Insel erkundest.",
        15.,
        MUTED,
    );
    parent.spawn(column_grid(2)).with_children(|actions| {
        button(actions, hud, "Hauptmenü", UiAction::MainMenu, false, false);
        button(actions, hud, "Dorf ansehen", UiAction::Resume, false, true);
    });
}

pub(super) fn neighbor(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, short: bool) {
    let city = game.city();
    let region = game.region();
    let index = match city.village_task(region) {
        Some(VillageTask::Visiting(i)) => Some(i),
        _ => city.reachable_neighbor(region),
    };
    panel_heading(parent, hud, "Ein Dorf kennenlernen", short);
    if let Some(i) = index {
        let village = &region.villages[i];
        left(
            parent,
            hud,
            &format!(
                "{} liegt bei dir in der Nähe. Schick dein Nilpferd zum Dorfplatz, um die Bewohner zu begrüßen.",
                village.name
            ),
            17.,
            INK,
        );
        left(
            parent,
            hud,
            "Der Besuch ist friedlich. Während das Nilpferd unterwegs ist, kannst du zu Hause weiterbauen.",
            15.,
            MUTED,
        );
        parent.spawn(column_grid(2)).with_children(|actions| {
            button(actions, hud, "Weiterbauen", UiAction::Cancel, false, false);
            button(
                actions,
                hud,
                "Nilpferd losschicken",
                UiAction::Do(Action::MeetNeighbor(i)),
                false,
                true,
            );
        });
    } else {
        left(
            parent,
            hud,
            "Vom Nilpferd führt gerade kein freier Weg zu einem Nachbardorf. Ruf es näher zu deinem Dorf und versuche es erneut.",
            16.,
            INK,
        );
        button(parent, hud, "Weiterbauen", UiAction::Cancel, false, true);
    }
}

pub(super) fn rivalry(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, short: bool) {
    let rival = crate::rivals::Rival::for_biome(game.region().kind);
    panel_heading(parent, hud, "Die Rivalität beginnt auf Wunsch", short);
    left(
        parent,
        hud,
        &format!(
            "{} wirbt um die Nachbardörfer. Du kannst sie mit Besuchen und Gaben für dich gewinnen.",
            rival.name
        ),
        17.,
        INK,
    );
    left(
        parent,
        hud,
        "Der erste Gesandte bricht frühestens nach knapp zwei Spielminuten auf. Sobald du den Grenzschrein baust, kündigt das Spiel Angriffe mit Vorlauf an.",
        15.,
        MUTED,
    );
    parent.spawn(column_grid(2)).with_children(|actions| {
        button(actions, hud, "Weiterbauen", UiAction::Cancel, false, true);
        button(
            actions,
            hud,
            "Rivalität beginnen",
            UiAction::Do(Action::BeginRivalry),
            false,
            false,
        );
    });
}

pub(super) fn raid(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, short: bool) {
    let region = game.region();
    panel_heading(parent, hud, "Angriff am Übergang", short);
    label(parent, hud, Label::Raid, 16., INK);
    left(
        parent,
        hud,
        "Eine Wache am Übergang, ein Wachturm in der Nähe oder eine aktive Schutzglocke hält den Angriff ab.",
        15.,
        MUTED,
    );
    parent.spawn(column_grid(2)).with_children(|actions| {
        button(actions, hud, "Zurück", UiAction::Cancel, false, false);
        button(
            actions,
            hud,
            "Übergang zeigen",
            UiAction::Focus(region.bridge()),
            false,
            true,
        );
    });
}

pub(super) fn raid_card(parent: &mut ChildSpawnerCommands, hud: &Hud, layout: Layout) {
    let edge = if layout.short { 8. } else { EDGE };
    parent
        .spawn((
            HitArea,
            Node {
                left: Val::Px(edge),
                top: Val::Px(if layout.compact { 65. } else { 73. }),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                padding: UiRect::all(Val::Px(8.)),
                row_gap: Val::Px(4.),
                ..default()
            },
            BackgroundColor(rgb(DARK)),
            BorderColor(rgb(BRASS)),
            BorderRadius::all(Val::Px(9.)),
        ))
        .with_children(|card| {
            label(card, hud, Label::RaidAlert, 18., GOLD);
            styled_button(
                card,
                hud,
                if layout.gamepad {
                    "L3 · Abwehr ansehen"
                } else {
                    "Abwehr ansehen"
                },
                UiAction::Guide,
                false,
                true,
                ButtonStyle {
                    compact: true,
                    ..default()
                },
            );
        });
}
