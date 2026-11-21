use super::*;

#[derive(Component)]
pub(crate) struct QuestMeter(bool);

fn milestones(game: &Game) -> (usize, usize) {
    let city = game.city();
    let r = game.region();
    let districts = city
        .districts
        .iter()
        .filter(|d| d.outcome.is_some())
        .count();
    if let Some(s) = city.quest_step(r) {
        let done = s.goals.iter().filter(|g| g.met(city, r)).count();
        return if city.quest == Quest::Districts {
            (done + districts, s.goals.len() + 2)
        } else {
            (done, s.goals.len().max(1))
        };
    }
    match city.quest {
        Quest::Shelter => (
            [
                city.hand_deliveries > 0 && !city.waiting_for_first_delivery(),
                city.count(Building::House) > 0,
                city.count(Building::Store) > 0,
                city.hippo_deliveries > 0,
            ]
            .into_iter()
            .filter(|v| *v)
            .count(),
            4,
        ),
        Quest::Fields => (
            usize::from(city.count(Building::Field) > 0) + usize::from(city.harvests > 0),
            2,
        ),
        Quest::Bridge => (
            usize::from(city.bridge_fixed)
                + usize::from(city.gift_given)
                + usize::from(city.renown() >= r.renown),
            3,
        ),
        Quest::Districts => (districts, 2),
        Quest::Complete => (1, 1),
    }
}

pub(super) fn counter(game: &Game) -> String {
    let (n, total) = milestones(game);
    format!("{n}/{total}")
}

fn meter(parent: &mut ChildSpawnerCommands, work: bool) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Px(4.),
                ..default()
            },
            BackgroundColor(rgb(0x516659)),
            BorderRadius::MAX,
        ))
        .with_children(|track| {
            track.spawn((
                QuestMeter(work),
                Node {
                    height: Val::Percent(100.),
                    ..default()
                },
                BackgroundColor(rgb(GOLD)),
                BorderRadius::MAX,
            ));
        });
}

pub(crate) fn update_meter(game: Res<Game>, mut bars: Query<(&QuestMeter, &mut Node)>) {
    if bars.is_empty() {
        return;
    }
    let (n, total) = milestones(&game);
    let width = Val::Percent(100. * n as f32 / total as f32);
    for (meter, mut node) in &mut bars {
        let width = if meter.0 {
            let w = &game.city().work;
            let done = match w.stage {
                crate::game::WorkStage::Offered => 0.,
                crate::game::WorkStage::Supplies => {
                    w.delivered as f32 / crate::game::Blueprint::on(game.region()).amount as f32
                }
                crate::game::WorkStage::Raising => 1.,
                crate::game::WorkStage::Opening => 2.,
                crate::game::WorkStage::Running => 3.,
            };
            Val::Percent(100. * done / 3.)
        } else {
            width
        };
        if node.width != width {
            node.width = width;
        }
    }
}

fn left_text(p: &mut ChildSpawnerCommands, h: &Hud, value: &str, size: f32, color: u32) -> Entity {
    let e = text(p, h, value, size, color);
    p.commands()
        .entity(e)
        .insert(TextLayout::new_with_justify(JustifyText::Left));
    e
}
fn live(p: &mut ChildSpawnerCommands, h: &Hud, label: Label, size: f32, color: u32) {
    let e = left_text(p, h, "", size, color);
    p.commands().entity(e).insert(label);
}

pub(super) fn tracker(p: &mut ChildSpawnerCommands, h: &Hud, game: &Game, l: Layout) {
    let r = game.region();
    let city = game.city();
    let small = l.short || l.compact;
    p.spawn((
        HitArea,
        Node {
            width: Val::Px(if small { 235. } else { 318. }),
            max_width: Val::Percent(63.),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            padding: UiRect::axes(Val::Px(8.), Val::Px(5.)),
            row_gap: Val::Px(3.),
            border: UiRect::left(Val::Px(3.)),
            ..default()
        },
        BackgroundColor(rgb(DARK).with_alpha(0.96)),
        BorderColor(rgb(BRASS)),
        BorderRadius::all(Val::Px(8.)),
    ))
    .with_children(|card| {
        card.spawn(Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .with_children(|header| {
            header
                .spawn(Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(5.),
                    ..default()
                })
                .with_children(|name| {
                    glyph(name, h, 11, 15.);
                    left_text(name, h, "Aufträge", 16., LIGHT);
                });
            styled_button(
                header,
                h,
                if l.folded { "Anzeigen" } else { "Einklappen" },
                UiAction::FoldTracker,
                false,
                false,
                ButtonStyle {
                    compact: true,
                    height: Some(28.),
                    ..default()
                },
            );
        });
        if l.folded {
            return;
        }
        card.spawn(Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: Val::Px(6.),
            ..default()
        })
        .with_children(|line| {
            line.spawn(Node {
                min_width: Val::Px(0.),
                flex_grow: 1.,
                flex_basis: Val::Px(0.),
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|copy| {
                left_text(
                    copy,
                    h,
                    if l.work_pinned {
                        crate::game::Blueprint::on(r).title
                    } else {
                        city.quest_title(r)
                    },
                    if small { 13. } else { 16. },
                    GOLD,
                );
                live(
                    copy,
                    h,
                    if l.work_pinned {
                        Label::WorkHint
                    } else {
                        Label::Quest
                    },
                    if small { 13. } else { 15. },
                    LIGHT,
                );
            });
            if l.work_pinned {
                text(
                    line,
                    h,
                    &format!(
                        "{}/3",
                        match l.work_stage {
                            crate::game::WorkStage::Supplies => 0,
                            crate::game::WorkStage::Raising => 1,
                            crate::game::WorkStage::Opening => 2,
                            _ => 3,
                        }
                    ),
                    13.,
                    GOLD,
                );
            } else {
                live(line, h, Label::QuestCount, 13., GOLD);
            }
        });
        meter(card, l.work_pinned);
        if matches!(l.side_quest, SideQuest::Active | SideQuest::Ready) {
            let (title, _, _) = city.local_quest(r);
            left_text(card, h, title, 13., JADE);
            live(card, h, Label::SideTrack, 12., LIGHT);
        }
        card.spawn(row()).with_children(|links| {
            let st = ButtonStyle {
                compact: true,
                height: Some(29.),
                ..default()
            };
            styled_button(
                links,
                h,
                if l.gamepad { "L3 · Ziel" } else { "Zum Ziel" },
                UiAction::Guide,
                false,
                false,
                st,
            );
            styled_button(
                links,
                h,
                if l.work_pinned {
                    "Werkbuch"
                } else {
                    "Tagebuch"
                },
                UiAction::Panel(if l.work_pinned {
                    Panel::Work
                } else {
                    Panel::Quests
                }),
                false,
                false,
                st,
            );
        });
    });
}

pub(super) fn journal_goals(p: &mut ChildSpawnerCommands, h: &Hud, game: &Game, short: bool) {
    let city = game.city();
    let r = game.region();
    p.spawn((
        Node {
            width: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(Val::Px(10.), Val::Px(6.)),
            row_gap: Val::Px(3.),
            border: UiRect::left(Val::Px(4.)),
            ..default()
        },
        BackgroundColor(rgb(DARK)),
        BorderColor(rgb(BRASS)),
        BorderRadius::all(Val::Px(9.)),
    ))
    .with_children(|card| {
        card.spawn(Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(7.),
            ..default()
        })
        .with_children(|line| {
            glyph(line, h, 14, 19.);
            left_text(line, h, "Nächstes Ziel", 13., GOLD);
            live(line, h, Label::QuestCount, 13., GOLD);
        });
        live(card, h, Label::Quest, if short { 16. } else { 19. }, LIGHT);
        meter(card, false);
        if let Some(step) = city.quest_step(r) {
            let lines: Vec<_> = step.goals.iter().map(|g| g.progress(city, r)).collect();
            left_text(
                card,
                h,
                &lines.join("  ·  "),
                if short { 12. } else { 15. },
                JADE,
            );
        }
    });
}
