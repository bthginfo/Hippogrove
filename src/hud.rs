use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::{
    art::rgb,
    game::{Action, Game, Outcome, Quest, Talent, stock_text},
    play::{Panel, Play, Target, Tool, UiAction},
};

const CREAM: u32 = 0xf9eed0;
const PAPER: u32 = 0xfff7e3;
const INK: u32 = 0x25372b;
const MUTED: u32 = 0x58624b;
const GREEN: u32 = 0xd6dfb5;
const DARK: u32 = 0x344c3e;

#[derive(Component)]
pub struct HitArea;

#[derive(Component)]
pub struct ControlButton {
    pub action: UiAction,
    selected: bool,
    strong: bool,
}

pub type HitTest<'w, 's> = Query<
    'w,
    's,
    (
        Option<&'static ControlButton>,
        &'static ComputedNode,
        &'static GlobalTransform,
        &'static InheritedVisibility,
    ),
    With<HitArea>,
>;

#[derive(Clone, Copy, Component)]
pub(crate) enum Label {
    Stock,
    Faith,
    Objective,
    Quest,
    Notice,
    Confirm,
    Detail,
    Progress,
    Rival,
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct Layout {
    region: usize,
    tool: Tool,
    option: usize,
    panel: Panel,
    unlocked: usize,
    complete: bool,
    paused: bool,
}

#[derive(Resource, Default)]
pub struct Hud {
    root: Option<Entity>,
    layout: Option<Layout>,
    regular: Handle<Font>,
    bold: Handle<Font>,
    serif: Handle<Font>,
}

pub fn setup(mut hud: ResMut<Hud>, assets: Res<AssetServer>) {
    hud.regular = assets.load("embedded://fonts/DMSans-Medium.ttf");
    hud.bold = assets.load("embedded://fonts/DMSans-Bold.ttf");
    hud.serif = assets.load("embedded://fonts/DMSerifText-Regular.ttf");
}

fn panel_node() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(5.),
        padding: UiRect::axes(Val::Px(15.), Val::Px(10.)),
        ..default()
    }
}

fn text(
    parent: &mut ChildSpawnerCommands,
    hud: &Hud,
    value: &str,
    size: f32,
    color: u32,
) -> Entity {
    parent
        .spawn((
            Text::new(value),
            TextFont {
                font: if size >= 24. {
                    hud.serif.clone()
                } else {
                    hud.regular.clone()
                },
                font_size: size,
                ..default()
            },
            TextColor(rgb(color)),
        ))
        .id()
}

fn label(parent: &mut ChildSpawnerCommands, hud: &Hud, tag: Label, size: f32, color: u32) {
    let entity = text(parent, hud, "", size, color);
    parent.commands().entity(entity).insert(tag);
}

fn button(
    parent: &mut ChildSpawnerCommands,
    hud: &Hud,
    name: &str,
    action: UiAction,
    selected: bool,
    strong: bool,
) -> Entity {
    let surface = if strong {
        DARK
    } else if selected {
        GREEN
    } else {
        CREAM
    };
    parent
        .spawn((
            Button,
            HitArea,
            ControlButton {
                action,
                selected,
                strong,
            },
            Node {
                min_height: Val::Px(if strong { 56. } else { 50. }),
                padding: UiRect::axes(Val::Px(if strong { 17. } else { 13. }), Val::Px(7.)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(rgb(surface)),
            BorderRadius::all(Val::Px(12.)),
        ))
        .with_children(|child| {
            let color = if strong { PAPER } else { INK };
            let e = child
                .spawn((
                    Text::new(name),
                    TextFont {
                        font: hud.bold.clone(),
                        font_size: if strong { 19. } else { 17. },
                        ..default()
                    },
                    TextColor(rgb(color)),
                ))
                .id();
            if matches!(action, UiAction::Confirm) {
                child.commands().entity(e).insert(Label::Confirm);
            }
        })
        .id()
}

fn row() -> Node {
    Node {
        column_gap: Val::Px(8.),
        row_gap: Val::Px(7.),
        flex_wrap: FlexWrap::Wrap,
        ..default()
    }
}

fn top(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game) {
    parent
        .spawn((
            HitArea,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(21.),
                top: Val::Px(17.),
                ..panel_node()
            },
            BackgroundColor(rgb(CREAM)),
            BorderRadius::all(Val::Px(14.)),
        ))
        .with_children(|panel| {
            text(panel, hud, &game.region().name, 28., INK);
            label(panel, hud, Label::Stock, 17., INK);
            label(panel, hud, Label::Faith, 15., MUTED);
        });
    parent
        .spawn((
            HitArea,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(17.),
                right: Val::Px(21.),
                width: Val::Percent(29.),
                min_width: Val::Px(228.),
                max_width: Val::Px(360.),
                ..panel_node()
            },
            BackgroundColor(rgb(CREAM)),
            BorderRadius::all(Val::Px(14.)),
        ))
        .with_children(|panel| {
            label(panel, hud, Label::Quest, 25., INK);
            label(panel, hud, Label::Objective, 16., MUTED);
            panel.spawn(row()).with_children(|controls| {
                button(controls, hud, "Zum Ziel", UiAction::Guide, false, false);
                button(
                    controls,
                    hud,
                    "Aufträge",
                    UiAction::Panel(Panel::Quests),
                    false,
                    false,
                );
            });
        });
    parent
        .spawn((Node {
            position_type: PositionType::Absolute,
            top: Val::Px(19.),
            left: Val::Percent(32.),
            width: Val::Percent(35.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },))
        .with_children(|panel| label(panel, hud, Label::Notice, 16., PAPER));
}

fn bottom(parent: &mut ChildSpawnerCommands, hud: &Hud, play: &Play, complete: bool) {
    parent
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(17.),
            left: Val::Px(21.),
            width: Val::Percent(69.),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(7.),
            ..default()
        })
        .with_children(|toolbar| {
            toolbar.spawn(row()).with_children(|tabs| {
                button(
                    tabs,
                    hud,
                    if complete {
                        "Nächstes Kapitel"
                    } else {
                        "Karte"
                    },
                    UiAction::Panel(Panel::Journey),
                    complete,
                    false,
                );
                button(
                    tabs,
                    hud,
                    "Fortschritt",
                    UiAction::Panel(Panel::Progress),
                    false,
                    false,
                );
            });
            if play.tool.choices() > 1 {
                toolbar.spawn(row()).with_children(|options| {
                    (0..play.tool.choices()).for_each(|index| {
                        button(
                            options,
                            hud,
                            play.tool.choice_name(index),
                            UiAction::Choice(index),
                            index == play.option,
                            false,
                        );
                    });
                    if play.tool == Tool::Build {
                        button(options, hud, "Drehen", UiAction::Rotate, false, false);
                    }
                    if play.tool == Tool::Hippo {
                        button(
                            options,
                            hud,
                            "Loben",
                            UiAction::Do(Action::Praise),
                            false,
                            false,
                        );
                    }
                });
            }
            toolbar.spawn(row()).with_children(|tabs| {
                for tool in Tool::ALL {
                    button(
                        tabs,
                        hud,
                        tool.name(),
                        UiAction::Tool(tool),
                        tool == play.tool,
                        false,
                    );
                }
            });
        });
    parent
        .spawn((
            HitArea,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(17.),
                right: Val::Px(21.),
                width: Val::Percent(25.),
                min_width: Val::Px(194.),
                max_width: Val::Px(270.),
                ..panel_node()
            },
            BackgroundColor(rgb(CREAM)),
            BorderRadius::all(Val::Px(14.)),
        ))
        .with_children(|command| {
            label(command, hud, Label::Detail, 15., MUTED);
            button(command, hud, "Ausführen", UiAction::Confirm, false, true);
            if play.tool != Tool::Hand {
                button(command, hud, "Abbrechen", UiAction::Cancel, false, false);
            }
        });
}

fn modal(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, play: &Play) {
    let region = game.region();
    let city = game.city();
    parent.spawn((
        Node{width:Val::Percent(100.),height:Val::Percent(100.),align_items:AlignItems::Center,
            justify_content:JustifyContent::Center,..default()},
        BackgroundColor(rgb(INK).with_alpha(0.52)),
    )).with_children(|shade| {
        shade.spawn((
            HitArea,
            Node{
                width:Val::Percent(83.),max_width:Val::Px(690.),flex_direction:FlexDirection::Column,
                padding:UiRect::all(Val::Px(21.)),row_gap:Val::Px(10.),..default()
            },
            BackgroundColor(rgb(PAPER)),BorderRadius::all(Val::Px(18.)),
        )).with_children(|box_node| {
            box_node.spawn(row()).with_children(|nav| {
                button(nav,hud,"Kapitel",UiAction::Panel(Panel::Journey),play.panel==Panel::Journey,false);
                button(nav,hud,"Fortschritt",UiAction::Panel(Panel::Progress),play.panel==Panel::Progress,false);
                button(nav,hud,"Aufträge",UiAction::Panel(Panel::Quests),play.panel==Panel::Quests,false);
                button(nav,hud,"Zurück",UiAction::Cancel,false,true);
            });
            match play.panel {
                Panel::Journey => {
                    text(box_node,hud,"Die Schreine am Wasser",31.,INK);
                    for (index,candidate) in game.regions.iter().enumerate() {
                        let result=game.campaign.settlements[index].as_ref().and_then(|place|place.outcome);
                        let suffix=if index>game.campaign.unlocked {" · verschlossen"}
                            else if let Some(outcome)=result { if outcome==Outcome::Alliance {" · Bündnis"} else {" · befreit"} }
                            else {""};
                        button(box_node,hud,&format!("{} · {}{suffix}",candidate.name,candidate.subtitle),
                            UiAction::Do(Action::Travel(index)),index==game.campaign.active,false);
                    }
                }
                Panel::Progress => {
                    text(box_node,hud,"Gott, Siedlung und Kreatur",31.,INK);
                    label(box_node,hud,Label::Progress,17.,MUTED);
                    box_node.spawn(row()).with_children(|talents| {
                        for talent in Talent::ALL {
                            button(talents,hud,talent.name(),UiAction::Do(Action::Learn(talent)),false,false);
                        }
                        button(talents,hud,"Nilpferd loben",UiAction::Do(Action::Praise),false,false);
                    });
                    text(box_node,hud,"Aufträge geben Talentpunkte. Gebäude heben die Dorfstufe. Nach dem Tragen lernt das Nilpferd durch Lob.",17.,INK);
                }
                Panel::Quests => {
                    text(box_node,hud,&region.subtitle,31.,INK);
                    text(box_node,hud,&region.lore,17.,MUTED);
                    for quest in [Quest::Shelter,Quest::Fields,Quest::Bridge] {
                        let mark=if quest.index()<city.quest.index() {"Erfüllt"} else if quest==city.quest {"Aktiv"} else {"Offen"};
                        text(box_node,hud,&format!("{mark} · {}",quest.name()),18.,INK);
                    }
                    label(box_node,hud,Label::Rival,16.,MUTED);
                }
                Panel::Closed=>{}
            }
            // The same feedback is visible while purchasing a talent or choosing a chapter.
            label(box_node,hud,Label::Notice,16.,DARK);
        });
    });
}

pub fn layout(
    mut commands: Commands,
    mut hud: ResMut<Hud>,
    game: Res<Game>,
    play: Res<Play>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut scale: ResMut<UiScale>,
) {
    let size = window.resolution.size();
    let wanted = (size.x / 1130.).min(size.y / 600.).clamp(0.62, 1.22);
    if (scale.0 - wanted).abs() > 0.01 {
        scale.0 = wanted;
    }
    let key = Layout {
        region: game.campaign.active,
        tool: play.tool,
        option: play.option,
        panel: play.panel,
        unlocked: game.campaign.unlocked,
        complete: game.city().outcome.is_some(),
        paused: play.paused,
    };
    if hud.layout == Some(key) {
        return;
    }
    if let Some(root) = hud.root.take() {
        commands.entity(root).despawn()
    }
    let entity = commands
        .spawn((
            Name::new("Spieloberfläche"),
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            },
        ))
        .with_children(|root| {
            if play.panel == Panel::Closed {
                top(root, &hud, &game);
                bottom(root, &hud, &play, key.complete);
            } else {
                modal(root, &hud, &game, &play);
            }
        })
        .id();
    hud.root = Some(entity);
    hud.layout = Some(key);
}

pub fn update(
    game: Res<Game>,
    play: Res<Play>,
    target: Res<Target>,
    time: Res<Time>,
    mut labels: Query<(&Label, &mut Text)>,
    mut buttons: Query<(
        &ControlButton,
        &Interaction,
        &ComputedNode,
        &GlobalTransform,
        &mut BackgroundColor,
    )>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let city = game.city();
    let region = game.region();
    let gifts = game.campaign.gifts;
    for (tag, mut text) in &mut labels {
        let value = match tag {
            Label::Stock => format!(
                "Holz {}   Stein {}   Nahrung {}",
                city.stock[0], city.stock[1], city.stock[2]
            ),
            Label::Faith => format!(
                "Glaube {}  ·  Wunder {:.0}/{:.0}",
                city.faith,
                city.mana,
                gifts.mana_cap()
            ),
            Label::Objective => city.objective(region),
            Label::Quest => city.quest.name().into(),
            Label::Notice => {
                if time.elapsed_secs() < play.notice_until {
                    play.notice.clone()
                } else {
                    String::new()
                }
            }
            Label::Confirm => target.label.clone(),
            Label::Detail => target.detail.clone(),
            Label::Progress => format!(
                "{} Talentpunkte · Regen {} · Wunderquelle {}\nDorfstufe {} · Ansehen {} · Lager {}/{} Nahrung\nNilpferd-Rang {} · {} Lieferungen",
                gifts.points,
                gifts.rank(Talent::Rain),
                gifts.rank(Talent::Spring),
                city.city_rank(),
                city.renown(),
                city.stock[2],
                city.storage_cap(),
                gifts.training,
                city.hippo_deliveries
            ),
            Label::Rival => format!(
                "{} · {} · {} {} · Ansehen {}/{}\n{} für den Übergang",
                region.rival,
                region.request,
                region.gift.1,
                region.gift.0.name(),
                city.renown(),
                region.renown,
                stock_text(region.rebuild)
            ),
        };
        if text.0 != value {
            text.0 = value;
        }
    }
    let pointer = play.pointer.map(|p| p * window.scale_factor());
    for (button, interaction, node, layout, mut color) in &mut buttons {
        let hovered = *interaction == Interaction::Hovered
            || (play.gamepad && pointer.is_some_and(|p| contains(node, layout, p)));
        let shade = if matches!(button.action, UiAction::Confirm) && !target.legal {
            0x6a6050
        } else if hovered && button.strong {
            0x52745e
        } else if hovered {
            0xe5d5ac
        } else if button.strong {
            DARK
        } else if button.selected {
            GREEN
        } else {
            CREAM
        };
        color.0 = rgb(shade);
    }
}

fn contains(node: &ComputedNode, transform: &GlobalTransform, point: Vec2) -> bool {
    !node.is_empty()
        && ((point - transform.translation().truncate())
            .abs()
            .cmple(node.size() * 0.5))
        .all()
}

pub fn hit(areas: &HitTest, point: Vec2, scale: f32) -> (bool, Option<UiAction>) {
    areas
        .iter()
        .filter(|(_, node, pose, visible)| visible.get() && contains(node, pose, point * scale))
        .max_by_key(|(_, node, _, _)| node.stack_index())
        .map_or((false, None), |(button, _, _, _)| {
            (true, button.map(|control| control.action))
        })
}
