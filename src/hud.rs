use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::{
    art::rgb,
    game::{Action, Game, Outcome, Quest, Settlement, Talent, stock_text},
    play::{Panel, Play, Target, Tool, UiAction},
    region::{Biome, Material, Tile},
};

const PAPER: u32 = 0xf6eed7;
const INK: u32 = 0x27372e;
const MUTED: u32 = 0x526054;
const DARK: u32 = 0x233b30;
const SURFACE: u32 = 0x304b3e;
const LIGHT: u32 = 0xfff4dc;
const SUBTLE: u32 = 0xe6ddc2;
const GOLD: u32 = 0xffd487;
const BRASS: u32 = 0xb99151;
const JADE: u32 = 0xd4e9b9;
const EDGE: f32 = 16.;

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
    Stock(Material),
    Faith,
    Objective,
    Quest,
    Notice,
    Confirm,
    Detail,
    Progress,
    Rival,
    District(usize),
    LocalQuest,
    Climate,
    Mana,
    Controls,
}

#[derive(Component)]
pub struct ManaFill;

#[derive(Component)]
pub struct Toast;

#[derive(Clone, Copy, Eq, PartialEq)]
struct Layout {
    region: usize,
    tool: Tool,
    option: usize,
    panel: Panel,
    unlocked: usize,
    quest: Quest,
    campaign_complete: bool,
    talents: [u8; Talent::ALL.len()],
    objective_expanded: bool,
    road_started: bool,
    compact: bool,
    short: bool,
    safe: [u16; 4],
}

#[derive(Resource, Default)]
pub struct Hud {
    root: Option<Entity>,
    layout: Option<Layout>,
    regular: Handle<Font>,
    bold: Handle<Font>,
    serif: Handle<Font>,
    icons: Handle<Image>,
    atlas: Handle<TextureAtlasLayout>,
}

pub fn setup(
    mut hud: ResMut<Hud>,
    assets: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    hud.regular = assets.load("embedded://fonts/DMSans-Medium.ttf");
    hud.bold = assets.load("embedded://fonts/DMSans-Bold.ttf");
    hud.serif = assets.load("embedded://fonts/DMSerifText-Regular.ttf");
    hud.icons = assets.load("embedded://ui/atlas.png");
    hud.atlas = atlases.add(TextureAtlasLayout::from_grid(
        UVec2::splat(64),
        4,
        4,
        None,
        None,
    ));
}

fn glyph(parent: &mut ChildSpawnerCommands, hud: &Hud, index: usize, size: f32) {
    parent.spawn((
        ImageNode::from_atlas_image(
            hud.icons.clone(),
            TextureAtlas {
                layout: hud.atlas.clone(),
                index,
            },
        ),
        Node {
            width: Val::Px(size),
            height: Val::Px(size),
            flex_shrink: 0.,
            ..default()
        },
    ));
}

fn button_glyph(action: UiAction) -> Option<usize> {
    match action {
        UiAction::Tool(Tool::Hand) => Some(5),
        UiAction::Tool(Tool::Build) => Some(6),
        UiAction::Tool(Tool::Helpers) => Some(7),
        UiAction::Tool(Tool::Hippo) => Some(8),
        UiAction::Tool(Tool::Miracles) => Some(9),
        UiAction::Tool(Tool::Plan) => Some(15),
        UiAction::Panel(Panel::Map) => Some(10),
        UiAction::Panel(Panel::Journey) => Some(12),
        UiAction::Panel(Panel::Progress) => Some(13),
        UiAction::Panel(Panel::Quests) | UiAction::ToggleObjective => Some(11),
        UiAction::Guide => Some(14),
        _ => None,
    }
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
        GOLD
    } else if selected {
        JADE
    } else {
        SURFACE
    };
    let tool = matches!(action, UiAction::Tool(_));
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
                min_width: Val::Px(if tool { 60. } else { 48. }),
                min_height: Val::Px(if tool {
                    66.
                } else if strong {
                    54.
                } else {
                    48.
                }),
                padding: UiRect::axes(Val::Px(if tool { 7. } else { 11. }), Val::Px(5.)),
                border: UiRect::all(Val::Px(2.)),
                flex_direction: if tool {
                    FlexDirection::Column
                } else {
                    FlexDirection::Row
                },
                column_gap: Val::Px(6.),
                row_gap: Val::Px(1.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(rgb(surface)),
            BorderColor(rgb(if selected || strong { GOLD } else { BRASS })),
            Outline::new(Val::Px(3.), Val::Px(2.), Color::NONE),
            BorderRadius::all(Val::Px(if tool { 13. } else { 11. })),
        ))
        .with_children(|child| {
            if let Some(index) = button_glyph(action) {
                glyph(child, hud, index, if tool { 32. } else { 20. });
            }
            let color = if strong || selected { INK } else { LIGHT };
            let e = child
                .spawn((
                    Text::new(name),
                    TextFont {
                        font: hud.bold.clone(),
                        font_size: if tool {
                            13.
                        } else if strong {
                            18.
                        } else {
                            15.
                        },
                        ..default()
                    },
                    TextColor(rgb(color)),
                ))
                .id();
            if matches!(action, UiAction::Confirm) {
                child.commands().entity(e).insert(Label::Confirm);
            } else if matches!(action, UiAction::ToggleObjective) {
                child.commands().entity(e).insert(Label::Quest);
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

fn top(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, layout: Layout) {
    let border = UiRect::all(Val::Px(2.));
    parent
        .spawn((
            HitArea,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(EDGE),
                top: Val::Px(EDGE),
                flex_direction: if layout.compact {
                    FlexDirection::Column
                } else {
                    FlexDirection::Row
                },
                align_items: if layout.compact {
                    AlignItems::Start
                } else {
                    AlignItems::Center
                },
                column_gap: Val::Px(15.),
                row_gap: Val::Px(4.),
                padding: UiRect::axes(Val::Px(12.), Val::Px(7.)),
                border,
                ..default()
            },
            BackgroundColor(rgb(DARK)),
            BorderColor(rgb(BRASS)),
            BorderRadius::all(Val::Px(15.)),
        ))
        .with_children(|panel| {
            text(
                panel,
                hud,
                &game.region().name,
                if layout.compact { 25. } else { 30. },
                LIGHT,
            );
            panel
                .spawn(Node {
                    column_gap: Val::Px(7.),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|stats| {
                    for material in Material::ALL {
                        stat(
                            stats,
                            hud,
                            material.index(),
                            material.name(),
                            Label::Stock(material),
                        );
                    }
                    stat(stats, hud, 3, "Glaube", Label::Faith);
                    stats
                        .spawn(Node {
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|mana| {
                            glyph(mana, hud, 4, 26.);
                            mana.spawn(Node {
                                flex_direction: FlexDirection::Column,
                                min_width: Val::Px(50.),
                                ..default()
                            })
                            .with_children(|meter| {
                                label(meter, hud, Label::Mana, 14., LIGHT);
                                meter
                                    .spawn((
                                        Node {
                                            width: Val::Px(48.),
                                            height: Val::Px(4.),
                                            ..default()
                                        },
                                        BackgroundColor(rgb(0x526a56)),
                                        BorderRadius::all(Val::Px(3.)),
                                    ))
                                    .with_children(|track| {
                                        track.spawn((
                                            ManaFill,
                                            Node {
                                                width: Val::Percent(0.),
                                                height: Val::Percent(100.),
                                                ..default()
                                            },
                                            BackgroundColor(rgb(0x94dbc1)),
                                            BorderRadius::all(Val::Px(3.)),
                                        ));
                                    });
                            });
                        });
                });
            if !layout.compact {
                label(panel, hud, Label::Climate, 14., SUBTLE);
            }
        });

    if layout.compact {
        let mut nav = row();
        nav.position_type = PositionType::Absolute;
        nav.top = Val::Px(EDGE + 90.);
        nav.left = Val::Px(EDGE);
        parent
            .spawn(nav)
            .with_children(|tabs| navigation(tabs, hud, layout.quest));
    }

    parent
        .spawn((
            HitArea,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(EDGE),
                right: Val::Px(EDGE),
                width: Val::Percent(if layout.compact { 34. } else { 28. }),
                min_width: Val::Px(if layout.compact { 205. } else { 244. }),
                max_width: Val::Px(365.),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(4.)),
                border,
                ..default()
            },
            BackgroundColor(rgb(DARK)),
            BorderColor(rgb(BRASS)),
            BorderRadius::all(Val::Px(15.)),
        ))
        .with_children(|panel| {
            button(
                panel,
                hud,
                game.city().quest.name(),
                UiAction::ToggleObjective,
                false,
                false,
            );
            if layout.objective_expanded {
                panel
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(5.),
                            padding: UiRect::all(Val::Px(9.)),
                            ..default()
                        },
                        BackgroundColor(rgb(PAPER)),
                        BorderRadius::all(Val::Px(9.)),
                    ))
                    .with_children(|card| {
                        label(card, hud, Label::Objective, 15., INK);
                        if layout.compact {
                            label(card, hud, Label::Climate, 13., MUTED);
                        }
                        card.spawn(row()).with_children(|links| {
                            button(links, hud, "Zum Ziel", UiAction::Guide, false, false);
                            button(
                                links,
                                hud,
                                "Aufträge",
                                UiAction::Panel(Panel::Quests),
                                false,
                                false,
                            );
                        });
                    });
            }
        });

    parent
        .spawn((
            Toast,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(if layout.compact { 106. } else { 78. }),
                left: Val::Percent(31.),
                width: Val::Percent(36.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::axes(Val::Px(12.), Val::Px(7.)),
                ..default()
            },
            Visibility::Hidden,
            BackgroundColor(rgb(DARK)),
            BorderRadius::all(Val::Px(10.)),
        ))
        .with_children(|toast| label(toast, hud, Label::Notice, 15., LIGHT));
}

fn stat(parent: &mut ChildSpawnerCommands, hud: &Hud, icon: usize, name: &str, value: Label) {
    parent
        .spawn(Node {
            align_items: AlignItems::Center,
            flex_shrink: 0.,
            ..default()
        })
        .with_children(|stat| {
            glyph(stat, hud, icon, 26.);
            stat.spawn(Node {
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|column| {
                text(column, hud, name, 11., SUBTLE);
                label(column, hud, value, 16., LIGHT);
            });
        });
}

fn navigation(parent: &mut ChildSpawnerCommands, hud: &Hud, quest: Quest) {
    let complete = quest == Quest::Complete;
    button(
        parent,
        hud,
        "Landkarte",
        UiAction::Panel(Panel::Map),
        false,
        false,
    );
    button(
        parent,
        hud,
        if complete { "Weiterreisen" } else { "Kapitel" },
        UiAction::Panel(Panel::Journey),
        complete,
        false,
    );
    button(
        parent,
        hud,
        "Fortschritt",
        UiAction::Panel(Panel::Progress),
        false,
        false,
    );
}

fn bottom(parent: &mut ChildSpawnerCommands, hud: &Hud, play: &Play, layout: Layout) {
    parent
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(EDGE),
            left: Val::Px(EDGE),
            width: Val::Percent(if layout.compact { 65. } else { 67. }),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Start,
            row_gap: Val::Px(6.),
            ..default()
        })
        .with_children(|toolbar| {
            if !layout.compact {
                toolbar
                    .spawn(row())
                    .with_children(|tabs| navigation(tabs, hud, layout.quest));
            }
            if play.tool.choices() > 1 {
                toolbar.spawn(row()).with_children(|options| {
                    let page_size = if play.tool == Tool::Hippo
                        || (layout.compact && play.tool == Tool::Miracles)
                    {
                        3
                    } else {
                        4
                    };
                    let page = (play.option / page_size) * page_size;
                    for index in page..play.tool.choices().min(page + page_size) {
                        button(
                            options,
                            hud,
                            play.tool.choice_name(index),
                            UiAction::Choice(index),
                            index == play.option,
                            false,
                        );
                    }
                    if play.tool.choices() > page_size {
                        let next = if page + page_size < play.tool.choices() {
                            page + page_size
                        } else {
                            0
                        };
                        let category = match play.tool {
                            Tool::Build => match next {
                                4 => "Versorgung ›",
                                8 => "Grenze ›",
                                _ => "Grundbau ›",
                            },
                            Tool::Hippo if next == 0 => "Aufträge ›",
                            Tool::Hippo => "Pflege ›",
                            _ => "Weitere ›",
                        };
                        button(options, hud, category, UiAction::Choice(next), false, false);
                    }
                });
            }
            let mut dock = row();
            dock.padding = UiRect::all(Val::Px(5.));
            dock.border = UiRect::all(Val::Px(2.));
            dock.column_gap = Val::Px(5.);
            toolbar
                .spawn((
                    HitArea,
                    dock,
                    BackgroundColor(rgb(DARK)),
                    BorderRadius::all(Val::Px(16.)),
                    BorderColor(rgb(BRASS)),
                ))
                .with_children(|tabs| {
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
                bottom: Val::Px(EDGE),
                right: Val::Px(EDGE),
                width: Val::Percent(if layout.compact { 29. } else { 25. }),
                min_width: Val::Px(if layout.compact { 166. } else { 197. }),
                max_width: Val::Px(290.),
                border: UiRect::all(Val::Px(2.)),
                ..panel_node()
            },
            BackgroundColor(rgb(DARK)),
            BorderColor(rgb(BRASS)),
            BorderRadius::all(Val::Px(16.)),
        ))
        .with_children(|command| {
            label(command, hud, Label::Detail, 15., LIGHT);
            button(command, hud, "Ausführen", UiAction::Confirm, false, true);
            label(command, hud, Label::Controls, 12., SUBTLE);
            if play.tool == Tool::Build {
                button(command, hud, "Drehen", UiAction::Rotate, false, false);
            }
            if play.road_anchor.is_some() {
                button(
                    command,
                    hud,
                    "Weg abbrechen",
                    UiAction::Cancel,
                    false,
                    false,
                );
            }
        });
}

fn map_panel(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, short: bool) {
    let region = game.region();
    let (land, water) = match region.kind {
        Biome::Valley => (0xb7bf80, 0x6daaae),
        Biome::Fjord => (0xb8c2aa, 0x7ba6b2),
        Biome::Terraces => (0xaebb76, 0x75b5a3),
        Biome::Lake => (0xdcc395, 0x65b5af),
    };
    let position = |tile: Tile| {
        let min = region.bounds.0;
        let max = region.bounds.1;
        Vec2::new(
            100. * f32::from(tile.0 - min.0) / f32::from(max.0 - min.0),
            100. * f32::from(tile.1 - min.1) / f32::from(max.1 - min.1),
        )
    };
    parent
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Px(if short { 86. } else { 156. }),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(rgb(land)),
            BorderRadius::all(Val::Px(12.)),
        ))
        .with_children(|map| {
            for column in region.bounds.0.0..=region.bounds.1.0 {
                let p = position(Tile(column, region.river_at(column)));
                map.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(p.x),
                        top: Val::Percent(p.y),
                        width: Val::Percent(103. / region.bounds.width() as f32),
                        height: Val::Percent(
                            100. * f32::from(region.river_span(column))
                                / (region.bounds.len() / region.bounds.width()) as f32,
                        ),
                        ..default()
                    },
                    BackgroundColor(rgb(water)),
                ));
            }
            for (tile, color) in [
                (region.start, DARK),
                (region.rival_shrine, 0x8b5b3f),
                (region.districts[0].center, PAPER),
                (region.districts[1].center, PAPER),
            ] {
                let p = position(tile);
                map.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(p.x),
                        top: Val::Percent(p.y),
                        width: Val::Px(12.),
                        height: Val::Px(12.),
                        ..default()
                    },
                    BackgroundColor(rgb(color)),
                    BorderRadius::all(Val::Px(6.)),
                ));
            }
        });
    parent.spawn(row()).with_children(|links| {
        button(
            links,
            hud,
            "Startdorf",
            UiAction::Focus(region.start),
            false,
            false,
        );
        button(
            links,
            hud,
            "Hauptübergang",
            UiAction::Focus(region.bridge()),
            false,
            false,
        );
        button(
            links,
            hud,
            "Landschaft ansehen",
            UiAction::Landscape,
            false,
            false,
        );
    });
    parent.spawn(row()).with_children(|districts| {
        for (index, district) in region.districts.iter().enumerate() {
            districts
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    width: Val::Percent(48.),
                    row_gap: Val::Px(3.),
                    ..default()
                })
                .with_children(|card| {
                    button(
                        card,
                        hud,
                        &district.name,
                        UiAction::Focus(district.center),
                        game.city().districts[index].outcome.is_some(),
                        false,
                    );
                    label(card, hud, Label::District(index), 15., INK);
                    text(card, hud, &district.story, 15., MUTED);
                });
        }
    });
}

fn modal(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, play: &Play, layout: Layout) {
    let region = game.region();
    let city = game.city();
    parent.spawn((
        Node{width:Val::Percent(100.),height:Val::Percent(100.),align_items:AlignItems::Center,
            justify_content:JustifyContent::Center,..default()},
        BackgroundColor(rgb(0x14261d).with_alpha(0.64)),
    )).with_children(|shade| {
        shade.spawn((
            HitArea,
            Node{
                width:Val::Percent(92.),max_width:Val::Px(720.),flex_direction:FlexDirection::Column,
                padding:UiRect::all(Val::Px(if layout.short { 12. } else { 21. })),
                border:UiRect::all(Val::Px(2.)),row_gap:Val::Px(if layout.short { 6. } else { 10. }),..default()
            },
            BackgroundColor(rgb(PAPER)),BorderColor(rgb(BRASS)),BorderRadius::all(Val::Px(18.)),
        )).with_children(|box_node| {
            box_node.spawn(row()).with_children(|nav| {
                button(nav,hud,"Kapitel",UiAction::Panel(Panel::Journey),play.panel==Panel::Journey,false);
                button(nav,hud,"Landkarte",UiAction::Panel(Panel::Map),play.panel==Panel::Map,false);
                button(nav,hud,"Fortschritt",UiAction::Panel(Panel::Progress),play.panel==Panel::Progress,false);
                button(nav,hud,"Aufträge",UiAction::Panel(Panel::Quests),play.panel==Panel::Quests,false);
                button(nav,hud,"Zurück",UiAction::Cancel,false,true);
            });
            match play.panel {
                Panel::Journey => {
                    text(box_node,hud,if game.campaign_complete() { "Die Schreine sind vereint" } else { "Die Schreine am Wasser" },31.,INK);
                    for (index,candidate) in game.regions.iter().enumerate() {
                        let result=game.campaign.settlements[index].as_ref().filter(|place| place.quest == Quest::Complete).and_then(|place|place.outcome);
                        let suffix=if index>game.campaign.unlocked {" · verschlossen"}
                            else if let Some(outcome)=result { if outcome==Outcome::Alliance {" · Bündnis"} else {" · befreit"} }
                            else {""};
                        button(box_node,hud,&format!("{} · {}{suffix}",candidate.name,candidate.subtitle),
                            UiAction::Do(Action::Travel(index)),index==game.campaign.active,false);
                    }
                }
                Panel::Map => map_panel(box_node,hud,game,layout.short),
                Panel::Progress => {
                    text(box_node,hud,"Gott, Siedlung und Kreatur",31.,INK);
                    label(box_node,hud,Label::Progress,17.,MUTED);
                    box_node.spawn(row()).with_children(|talents| {
                        for talent in Talent::ALL {
                            button(talents,hud,&format!("{} {}/2",talent.name(),game.campaign.gifts.rank(talent)),UiAction::Do(Action::Learn(talent)),false,false);
                        }
                        button(talents,hud,"Nilpferd loben",UiAction::Do(Action::Praise),false,false);
                    });
                    text(box_node,hud,"Aufträge geben Talentpunkte. Das Nilpferd lernt mit Lob nach erledigter Arbeit. Wache auf Rang 2, Feldpflege auf Rang 3.",17.,INK);
                }
                Panel::Quests => {
                    text(box_node,hud,&region.subtitle,31.,INK);
                    text(box_node,hud,&region.lore,17.,MUTED);
                    for quest in [Quest::Shelter,Quest::Fields,Quest::Bridge,Quest::Districts] {
                        let mark=if quest.index()<city.quest.index() {"Erfüllt"} else if quest==city.quest {"Aktiv"} else {"Offen"};
                        text(box_node,hud,&format!("{mark} · {}",quest.name()),18.,INK);
                    }
                    label(box_node,hud,Label::Rival,16.,MUTED);
                    label(box_node,hud,Label::LocalQuest,16.,DARK);
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
    let safe = crate::controls::safe_insets(&window);
    let [left, top_inset, right, bottom_inset] = safe.map(f32::from);
    let size = Vec2::new(
        window.width() - left - right,
        window.height() - top_inset - bottom_inset,
    );
    let wanted = (size.x / 1500.).min(size.y / 720.).clamp(1., 1.14);
    if (scale.0 - wanted).abs() > 0.01 {
        scale.0 = wanted;
    }
    let key = Layout {
        region: game.campaign.active,
        tool: play.tool,
        option: play.option,
        panel: play.panel,
        unlocked: game.campaign.unlocked,
        quest: game.city().quest,
        campaign_complete: game.campaign_complete(),
        talents: if play.panel == Panel::Progress {
            Talent::ALL.map(|talent| game.campaign.gifts.rank(talent))
        } else {
            [0; Talent::ALL.len()]
        },
        objective_expanded: play.objective_expanded,
        road_started: play.road_anchor.is_some(),
        compact: size.x / scale.0 < 920.,
        short: size.y / scale.0 < 490.,
        safe,
    };
    if hud.layout == Some(key) {
        return;
    }
    if let Some(root) = hud.root.take() {
        commands.entity(root).despawn()
    }
    let inset = |index| Val::Px(f32::from(safe[index]) / scale.0);
    let entity = commands
        .spawn((
            Name::new("Spieloberfläche"),
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                padding: UiRect {
                    left: inset(0),
                    top: inset(1),
                    right: inset(2),
                    bottom: inset(3),
                },
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn(Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                ..default()
            })
            .with_children(|safe_root| {
                if play.panel == Panel::Closed {
                    top(safe_root, &hud, &game, key);
                    bottom(safe_root, &hud, &play, key);
                } else {
                    modal(safe_root, &hud, &game, &play, key);
                }
            });
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
    mut buttons: Query<
        (
            &ControlButton,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut Outline,
        ),
        Without<Toast>,
    >,
    mut toasts: Query<(&mut Visibility, &mut BackgroundColor), With<Toast>>,
    mut mana: Query<&mut Node, With<ManaFill>>,
) {
    let city = game.city();
    let region = game.region();
    let gifts = game.campaign.gifts;
    for (tag, mut text) in &mut labels {
        let value = match tag {
            Label::Stock(material) => city.stock[material.index()].to_string(),
            Label::Faith => city.faith.to_string(),
            Label::Mana => format!("{:.0}/{:.0}", city.mana, gifts.mana_cap()),
            Label::Objective => city.objective(region),
            Label::Climate => region.weather(city.weather_time).name().into(),
            Label::Quest => format!(
                "{} {}",
                city.quest.name(),
                if play.objective_expanded { "−" } else { "+" }
            ),
            Label::Notice => {
                if time.elapsed_secs() < play.notice_until {
                    play.notice.clone()
                } else {
                    String::new()
                }
            }
            Label::Confirm => target.label.clone(),
            Label::Controls => {
                if play.gamepad {
                    "Südtaste · Bestätigen".into()
                } else {
                    String::new()
                }
            }
            Label::Detail if play.tool == Tool::Hippo => format!(
                "{} · {:.0}% Kraft\n{}",
                city.creature_status(),
                city.energy,
                target.detail
            ),
            Label::Detail => target.detail.clone(),
            Label::Progress => format!(
                "{} Talentpunkte · Dorfstufe {} · Ansehen {}\n{} Gläubige · {} Wegfelder · Platz für {} je Vorrat\nNilpferd-Rang {} · {} · {:.0}% Kraft",
                gifts.points,
                city.city_rank(),
                city.renown(),
                city.walkers.len().saturating_sub(1),
                city.roads.len(),
                city.storage_cap(),
                gifts.training,
                city.creature_status(),
                city.energy
            ),
            Label::Rival if city.outcome.is_none() => format!(
                "{} · {} · {} {} · Ansehen {}/{}\n{} für den Übergang",
                region.rival,
                region.request,
                region.gift.1,
                region.gift.0.name(),
                city.renown(),
                region.renown,
                stock_text(region.rebuild)
            ),
            Label::Rival => format!("{} · {}", region.rival, city.outcome.unwrap().name()),
            Label::District(index) => {
                let state = city.districts[*index];
                let district = &region.districts[*index];
                state.outcome.map_or_else(
                    || {
                        format!(
                            "{} · Widerstand {}",
                            Settlement::district_need(district.need),
                            state.resolve
                        )
                    },
                    |outcome| format!("{} · Sammelplatz", outcome.name()),
                )
            }
            Label::LocalQuest => {
                let (title, goal, _) = city.local_quest(region);
                if city.local_finished {
                    format!("{title} · erfüllt")
                } else {
                    format!("{title}\n{goal}")
                }
            }
        };
        if text.0 != value {
            text.0 = value;
        }
    }
    for mut meter in &mut mana {
        meter.width = Val::Percent((100. * city.mana / gifts.mana_cap()).clamp(0., 100.));
    }
    for (mut visible, mut surface) in &mut toasts {
        *visible = if !play.notice.is_empty() && time.elapsed_secs() < play.notice_until {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        surface.0 = rgb(if play.notice_ok { DARK } else { 0x69402f });
    }
    for (button, interaction, mut color, mut border, mut outline) in &mut buttons {
        let focused = play.panel != Panel::Closed && play.focus == Some(button.action);
        let hovered = *interaction != Interaction::None;
        let unavailable = match button.action {
            UiAction::Confirm => !target.legal,
            UiAction::Do(Action::Travel(index)) => index > game.campaign.unlocked,
            UiAction::Do(Action::Learn(talent)) => gifts.rank(talent) >= 2 || gifts.points == 0,
            _ => false,
        };
        let shade = if unavailable {
            if button.strong { 0xc9c3ab } else { 0x59665a }
        } else if focused || hovered {
            if button.strong {
                0xffe5b2
            } else if button.selected {
                0xe5f2cc
            } else {
                0x4a6855
            }
        } else if button.strong {
            GOLD
        } else if button.selected {
            JADE
        } else {
            SURFACE
        };
        color.0 = rgb(shade);
        border.0 = rgb(if focused {
            0xffecaa
        } else if button.selected || button.strong {
            GOLD
        } else {
            BRASS
        });
        outline.color = if focused { rgb(0x68502d) } else { Color::NONE };
    }
}

fn contains(node: &ComputedNode, transform: &GlobalTransform, point: Vec2) -> bool {
    !node.is_empty()
        && ((point - transform.translation().truncate())
            .abs()
            .cmple(node.size() * 0.5))
        .all()
}

pub fn move_focus(areas: &HitTest, current: Option<UiAction>, direction: Vec2) -> Option<UiAction> {
    let mut buttons = areas
        .iter()
        .filter_map(|(button, node, transform, visible)| {
            let button = button?;
            (visible.get() && !node.is_empty()).then_some((
                button.action,
                transform.translation().truncate(),
                node.stack_index(),
            ))
        })
        .collect::<Vec<_>>();
    buttons.sort_by_key(|item| item.2);
    focus_in(&buttons, current, direction)
}

fn focus_in(
    buttons: &[(UiAction, Vec2, u32)],
    current: Option<UiAction>,
    direction: Vec2,
) -> Option<UiAction> {
    let &(action, origin, _) = buttons
        .iter()
        .find(|item| Some(item.0) == current)
        .or_else(|| buttons.first())?;
    if direction == Vec2::ZERO {
        return Some(action);
    }
    let heading = direction.normalize();
    buttons
        .iter()
        .copied()
        .filter_map(|(next, point, _)| {
            let offset = point - origin;
            let along = offset.dot(heading);
            (along > 2.).then(|| (along + offset.perp_dot(heading).abs() * 2.7, next))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, next)| next)
        .or(Some(action))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_follows_spatial_directions_and_survives_relayout() {
        let map = UiAction::Panel(Panel::Map);
        let progress = UiAction::Panel(Panel::Progress);
        let chapter = UiAction::Panel(Panel::Journey);
        let travel = UiAction::Landscape;
        let controls = [
            (chapter, Vec2::new(60., 35.), 0),
            (map, Vec2::new(178., 35.), 1),
            (progress, Vec2::new(310., 35.), 2),
            (travel, Vec2::new(154., 170.), 3),
        ];
        assert_eq!(focus_in(&controls, Some(map), Vec2::X), Some(progress));
        assert_eq!(focus_in(&controls, Some(map), Vec2::Y), Some(travel));
        assert_eq!(focus_in(&controls, Some(travel), Vec2::NEG_Y), Some(map));
        assert_eq!(focus_in(&controls, Some(progress), Vec2::X), Some(progress));
        assert_eq!(
            focus_in(&controls, Some(UiAction::Cancel), Vec2::ZERO),
            Some(chapter)
        );
    }
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
