use bevy::window::PrimaryWindow;
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::{
    art::rgb,
    civic::{Branch, Civic, Tech},
    game::{Action, Building, Game, Outcome, Quest, Settlement, Talent, stock_text},
    modes::Mode,
    play::{Panel, Play, Target, Tool, UiAction},
    region::{Biome, Material, Region},
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
    DistrictBrief(usize),
    LocalQuest,
    Climate,
    Mana,
    Controls,
    CityGoods,
    Refined(usize),
    Carried,
    Capacity,
    AreaCamp(usize),
    AreaSupply(usize, Material),
    AreaStatus(usize, Material, bool),
    Subtitle,
}

#[derive(Component)]
pub struct ManaFill;

#[derive(Component)]
pub struct Toast;

#[derive(Clone, Copy, Eq, PartialEq)]
struct Layout {
    region: usize,
    mode: Mode,
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
    narrow_modal: bool,
    safe: [u16; 4],
    catalog: Option<Branch>,
    catalog_page: usize,
    tech: Tech,
    research_count: u32,
    insight: u16,
    carried: Option<Material>,
    free_build_tab: bool,
    area: Option<(Material, bool)>,
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
    maps: Vec<Handle<Image>>,
    maps_mode: Option<Mode>,
    world_art: Handle<Image>,
    world_cards: Handle<TextureAtlasLayout>,
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
        7,
        None,
        None,
    ));
    hud.world_art = assets.load("embedded://ui/welten.png");
    hud.world_cards = atlases.add(TextureAtlasLayout::from_grid(
        UVec2::new(200, 168),
        5,
        1,
        None,
        None,
    ));
}

fn chart(region: &Region) -> Image {
    let (land, water): (u32, u32) = match region.kind {
        Biome::Valley => (0xb7bf80, 0x6daaae),
        Biome::Fjord => (0xb8c2aa, 0x7ba6b2),
        Biome::Terraces => (0xaebb76, 0x75b5a3),
        Biome::Lake => (0xdcc395, 0x65b5af),
        Biome::Aegean => (0xc1bc80, 0x5497b2),
    };
    let pixels = region
        .bounds
        .tiles()
        .flat_map(|tile| {
            let color = if region.water(tile) {
                water
            } else if region.rocky(tile) {
                if region.kind == Biome::Fjord {
                    0xc3d2d9
                } else {
                    0x988f75
                }
            } else if region.source_at(tile).is_some() {
                0x73845e
            } else if region.road(tile) {
                0xe0c69b
            } else {
                land
            };
            [(color >> 16) as u8, (color >> 8) as u8, color as u8, 255]
        })
        .collect();
    let width = region.bounds.width();
    Image::new(
        Extent3d {
            width: width as u32,
            height: (region.bounds.len() / width) as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
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
        UiAction::Panel(Panel::Catalog) => Some(12),
        UiAction::Panel(Panel::Research) => Some(9),
        UiAction::Panel(Panel::Inventory) => Some(20),
        UiAction::Panel(Panel::Modes) => Some(10),
        UiAction::ResearchBranch(branch) => Some(branch_glyph(branch)),
        UiAction::SelectTech(tech) => Some(branch_glyph(tech.branch())),
        UiAction::Guide => Some(14),
        _ => None,
    }
}

fn branch_glyph(branch: Branch) -> usize {
    match branch {
        Branch::Homes => 6,
        Branch::Industry => 0,
        Branch::Culture => 7,
        Branch::Defense => 11,
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
            TextLayout::new_with_justify(JustifyText::Center),
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
    styled_button(
        parent,
        hud,
        name,
        action,
        selected,
        strong,
        ButtonStyle::default(),
    )
}

#[derive(Clone, Copy, Default)]
struct ButtonStyle {
    compact: bool,
    icon: Option<usize>,
    caption: Option<Label>,
}

fn styled_button(
    parent: &mut ChildSpawnerCommands,
    hud: &Hud,
    name: &str,
    action: UiAction,
    selected: bool,
    strong: bool,
    style: ButtonStyle,
) -> Entity {
    let surface = if strong {
        GOLD
    } else if selected {
        JADE
    } else {
        SURFACE
    };
    let tool = matches!(action, UiAction::Tool(_));
    let area_order = matches!(
        action,
        UiAction::AreaSource(..) | UiAction::Do(Action::AssignArea(..) | Action::CancelArea(..))
    );
    let icon = if style.compact && !tool {
        style.icon
    } else {
        style.icon.or_else(|| button_glyph(action))
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
                min_width: Val::Px(if tool { 60. } else { 48. }),
                min_height: Val::Px(if tool {
                    66.
                } else if strong && !style.compact {
                    54.
                } else {
                    48.
                }),
                padding: UiRect::axes(
                    Val::Px(if style.compact {
                        6.
                    } else if tool || area_order {
                        7.
                    } else {
                        11.
                    }),
                    Val::Px(5.),
                ),
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
            if let Some(index) = icon {
                glyph(
                    child,
                    hud,
                    index,
                    if tool {
                        32.
                    } else if style.compact {
                        18.
                    } else {
                        20.
                    },
                );
            }
            let color = if strong || selected { INK } else { LIGHT };
            let title = |copy: &mut ChildSpawnerCommands| {
                let entity = copy
                    .spawn((
                        Text::new(name),
                        TextFont {
                            font: hud.bold.clone(),
                            font_size: if tool {
                                13.
                            } else if style.compact {
                                14.
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
                    copy.commands().entity(entity).insert(Label::Confirm);
                } else if matches!(action, UiAction::ToggleObjective) {
                    copy.commands().entity(entity).insert(Label::Quest);
                }
            };
            if let Some(caption) = style.caption {
                child
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|copy| {
                        title(copy);
                        label(
                            copy,
                            hud,
                            caption,
                            if style.compact { 12. } else { 13. },
                            color,
                        );
                    });
            } else {
                title(child);
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
            panel
                .spawn(Node {
                    column_gap: Val::Px(11.),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|title| {
                    text(
                        title,
                        hud,
                        &game.region().name,
                        if layout.compact { 25. } else { 30. },
                        LIGHT,
                    );
                    button(
                        title,
                        hud,
                        "Vorräte",
                        UiAction::Panel(Panel::Inventory),
                        false,
                        false,
                    );
                });
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
        nav.top = Val::Px(EDGE + 105.);
        nav.left = Val::Px(EDGE);
        parent
            .spawn(nav)
            .with_children(|tabs| navigation(tabs, hud, layout.quest, layout.mode));
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
                                if game.mode.sandbox() {
                                    "Baubuch"
                                } else {
                                    "Aufträge"
                                },
                                UiAction::Panel(if game.mode.sandbox() {
                                    Panel::Catalog
                                } else {
                                    Panel::Quests
                                }),
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
                top: if layout.compact {
                    Val::Auto
                } else {
                    Val::Px(78.)
                },
                bottom: if layout.compact {
                    Val::Px(155.)
                } else {
                    Val::Auto
                },
                left: Val::Percent(if layout.compact { 28. } else { 31. }),
                width: Val::Percent(if layout.compact { 34. } else { 36. }),
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

fn navigation(parent: &mut ChildSpawnerCommands, hud: &Hud, quest: Quest, mode: Mode) {
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
        if mode.sandbox() {
            "Spielmodi"
        } else if complete {
            "Weiterreisen"
        } else {
            "Kapitel"
        },
        UiAction::Panel(if mode.sandbox() {
            Panel::Modes
        } else {
            Panel::Journey
        }),
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
                    .with_children(|tabs| navigation(tabs, hud, layout.quest, layout.mode));
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
                    let (page, next, total) =
                        if play.tool == Tool::Build && play.option >= Building::ALL.len() {
                            (play.option, play.option + 1, play.option + 1)
                        } else {
                            let total = if play.tool == Tool::Build {
                                Building::ALL.len()
                            } else {
                                play.tool.choices()
                            };
                            let page = (play.option / page_size) * page_size;
                            (page, (page + page_size).min(total), total)
                        };
                    for index in page..next {
                        button(
                            options,
                            hud,
                            play.tool.choice_name(index),
                            UiAction::Choice(index),
                            index == play.option,
                            false,
                        );
                    }
                    if total > page_size
                        && !(play.tool == Tool::Build && play.option >= Building::ALL.len())
                    {
                        let next = if next < total { next } else { 0 };
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
                    if play.tool == Tool::Build {
                        button(
                            options,
                            hud,
                            "Baubuch",
                            UiAction::Panel(Panel::Catalog),
                            false,
                            false,
                        );
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

fn column_grid(columns: u16) -> Node {
    Node {
        display: Display::Grid,
        width: Val::Percent(100.),
        grid_template_columns: RepeatedGridTrack::flex(columns, 1.),
        column_gap: Val::Px(6.),
        ..default()
    }
}

fn modal_navigation(parent: &mut ChildSpawnerCommands, hud: &Hud, panel: Panel, short: bool) {
    let style = ButtonStyle {
        compact: short,
        ..default()
    };
    parent.spawn(column_grid(5)).with_children(|nav| {
        for (name, tab) in [
            ("Kapitel", Panel::Journey),
            (if short { "Karte" } else { "Landkarte" }, Panel::Map),
            ("Fortschritt", Panel::Progress),
            ("Aufträge", Panel::Quests),
        ] {
            styled_button(
                nav,
                hud,
                name,
                UiAction::Panel(tab),
                tab == panel,
                false,
                style,
            );
        }
        styled_button(nav, hud, "Zurück", UiAction::Cancel, false, true, style);
    });
}

fn map_chart(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, short: bool) {
    let region = game.region();
    let width = region.bounds.width();
    let tiles = Vec2::new(width as f32, (region.bounds.len() / width) as f32);
    let height = if short { 100. } else { 208. };
    let size = tiles * (height / tiles.y);
    let dot = if short { 8. } else { 11. };
    parent
        .spawn((
            Node {
                width: Val::Px(size.x),
                height: Val::Px(size.y),
                align_self: AlignSelf::Center,
                overflow: Overflow::clip(),
                ..default()
            },
            ImageNode::new(hud.maps[game.campaign.active].clone()),
            BorderRadius::all(Val::Px(12.)),
        ))
        .with_children(|map| {
            let towns = [
                (region.start, DARK),
                (region.bridge(), 0x8b5b3f),
                (region.districts[0].center, PAPER),
                (region.districts[1].center, PAPER),
            ];
            let camps = region
                .areas
                .iter()
                .zip(game.city().camps)
                .map(|(area, built)| (area.center, if built { 0xfbe39e } else { 0xb67648 }));
            for (tile, color) in towns.into_iter().chain(camps) {
                let offset = tile.local() - region.bounds.0.local() + Vec2::splat(0.5);
                let p = (size * offset / tiles - Vec2::splat(dot * 0.5))
                    .clamp(Vec2::ZERO, size - Vec2::splat(dot));
                map.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(p.x),
                        top: Val::Px(p.y),
                        width: Val::Px(dot),
                        height: Val::Px(dot),
                        border: UiRect::all(Val::Px(1.)),
                        ..default()
                    },
                    BackgroundColor(rgb(color)),
                    BorderColor(rgb(if color == PAPER { BRASS } else { PAPER })),
                    BorderRadius::all(Val::Px(dot * 0.5)),
                ));
            }
        });
}

fn map_panel(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, layout: Layout) {
    let region = game.region();
    let compact = ButtonStyle {
        compact: layout.short,
        ..default()
    };
    let column = Node {
        flex_direction: FlexDirection::Column,
        min_width: Val::Px(0.),
        row_gap: Val::Px(if layout.short { 5. } else { 8. }),
        ..default()
    };
    parent.spawn(column_grid(2)).with_children(|body| {
        body.spawn(column.clone()).with_children(|chart| {
            map_chart(chart, hud, game, layout.short);
            styled_button(
                chart,
                hud,
                "Landschaft ansehen",
                UiAction::Landscape,
                false,
                false,
                compact,
            );
        });
        body.spawn(column).with_children(|destinations| {
            destinations.spawn(column_grid(2)).with_children(|links| {
                styled_button(
                    links,
                    hud,
                    "Startdorf",
                    UiAction::Focus(region.start),
                    false,
                    false,
                    compact,
                );
                styled_button(
                    links,
                    hud,
                    if layout.short {
                        "Übergang"
                    } else {
                        "Hauptübergang"
                    },
                    UiAction::Focus(region.bridge()),
                    false,
                    false,
                    compact,
                );
            });
            for (index, district) in region.districts.iter().enumerate() {
                let caption = if layout.short {
                    Label::DistrictBrief(index)
                } else {
                    Label::District(index)
                };
                styled_button(
                    destinations,
                    hud,
                    &district.name,
                    UiAction::Focus(district.center),
                    game.city().districts[index].outcome.is_some(),
                    false,
                    ButtonStyle {
                        caption: Some(caption),
                        ..compact
                    },
                );
                if !layout.short {
                    text(destinations, hud, &district.story, 14., MUTED);
                }
            }
        });
    });
    parent.spawn(column_grid(3)).with_children(|camps| {
        for (index, area) in region.areas.iter().enumerate() {
            styled_button(
                camps,
                hud,
                &area.name,
                UiAction::Panel(Panel::Area(index)),
                game.city().camps[index],
                false,
                ButtonStyle {
                    icon: (!layout.narrow_modal)
                        .then(|| material_glyph(region.kind, area.material)),
                    ..compact
                },
            );
        }
    });
}

fn panel_heading(parent: &mut ChildSpawnerCommands, hud: &Hud, title: &str, short: bool) {
    text(parent, hud, title, if short { 24. } else { 29. }, INK);
}

fn catalog_panel(
    parent: &mut ChildSpawnerCommands,
    hud: &Hud,
    game: &Game,
    play: &Play,
    short: bool,
) {
    parent.spawn(row()).with_children(|header| {
        panel_heading(header, hud, "Baubuch", short);
        label(header, hud, Label::CityGoods, 15., MUTED);
    });
    parent.spawn(row()).with_children(|tabs| {
        button(
            tabs,
            hud,
            "Grundbau",
            UiAction::Catalog(None),
            play.catalog_branch.is_none(),
            false,
        );
        for branch in Branch::ALL {
            button(
                tabs,
                hud,
                branch.name(),
                UiAction::Catalog(Some(branch)),
                play.catalog_branch == Some(branch),
                false,
            );
        }
    });
    let entries: Vec<_> = if let Some(branch) = play.catalog_branch {
        Civic::ALL
            .into_iter()
            .filter(|kind| kind.available_in(game.region().kind) && kind.spec().branch == branch)
            .map(Building::Civic)
            .collect()
    } else {
        Building::ALL.to_vec()
    };
    let size = if short { 2 } else { 3 };
    let pages = entries.len().div_ceil(size);
    let index = play.catalog_page.min(pages.saturating_sub(1));
    let research = game.campaign.research;
    for &building in entries.iter().skip(index * size).take(size) {
        let locked = building
            .civic()
            .map(|kind| kind.spec().unlock)
            .filter(|&tech| !research.knows(tech));
        let name = format!("{} · {}", building.name(), building.cost_text());
        let action = UiAction::SelectBuilding(building);
        let purpose = building
            .civic()
            .map_or_else(|| building.use_text(), |kind| kind.spec().purpose);
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.),
                ..default()
            })
            .with_children(|card| {
                button(card, hud, &name, action, false, false);
                if let Some(tech) = locked {
                    text(
                        card,
                        hud,
                        &format!("Forschung · {}", tech.name()),
                        13.,
                        MUTED,
                    );
                } else {
                    text(card, hud, purpose, 14., MUTED);
                }
            });
    }
    parent.spawn(row()).with_children(|bottom| {
        if index > 0 {
            button(
                bottom,
                hud,
                "Vorherige",
                UiAction::CatalogPage(index - 1),
                false,
                false,
            );
        }
        if index + 1 < pages {
            button(
                bottom,
                hud,
                "Nächste",
                UiAction::CatalogPage(index + 1),
                false,
                false,
            );
        }
        button(
            bottom,
            hud,
            "Forschung",
            UiAction::Panel(Panel::Research),
            false,
            false,
        );
    });
}

fn research_panel(
    parent: &mut ChildSpawnerCommands,
    hud: &Hud,
    game: &Game,
    selected: Tech,
    short: bool,
) {
    let current = game.campaign.research;
    let city_rank = game.city().city_rank();
    let branch = selected.branch();
    parent.spawn(row()).with_children(|header| {
        panel_heading(header, hud, "Forschung", short);
        text(
            header,
            hud,
            &format!("{} Erkenntnis", current.insight),
            16.,
            MUTED,
        );
    });
    parent.spawn(row()).with_children(|tabs| {
        for line in Branch::ALL {
            button(
                tabs,
                hud,
                line.name(),
                UiAction::ResearchBranch(line),
                line == branch,
                false,
            );
        }
        button(
            tabs,
            hud,
            "Baubuch",
            UiAction::Panel(Panel::Catalog),
            false,
            false,
        );
    });
    parent
        .spawn(Node {
            width: Val::Percent(100.),
            column_gap: Val::Px(12.),
            ..default()
        })
        .with_children(|pages| {
            pages
                .spawn(Node {
                    width: Val::Percent(49.),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    ..default()
                })
                .with_children(|topics| {
                    for (i, tech) in branch.topics().into_iter().enumerate() {
                        if i > 0 {
                            topics.spawn((
                                Node {
                                    width: Val::Px(3.),
                                    height: Val::Px(10.),
                                    align_self: AlignSelf::Center,
                                    ..default()
                                },
                                BackgroundColor(rgb(BRASS)),
                            ));
                        }
                        let mark = if current.knows(tech) {
                            "Erforscht"
                        } else if current.available(tech, city_rank).is_ok() {
                            "Bereit"
                        } else {
                            "Gesperrt"
                        };
                        button(
                            topics,
                            hud,
                            &format!("{} · {mark}", tech.name()),
                            UiAction::SelectTech(tech),
                            selected == tech,
                            false,
                        );
                    }
                });
            pages
                .spawn((
                    Node {
                        width: Val::Percent(49.),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(7.),
                        padding: UiRect::all(Val::Px(10.)),
                        ..default()
                    },
                    BackgroundColor(rgb(0xe7e1c7)),
                    BorderRadius::all(Val::Px(11.)),
                ))
                .with_children(|detail| {
                    text(
                        detail,
                        hud,
                        selected.name(),
                        if short { 20. } else { 24. },
                        INK,
                    );
                    let names: Vec<_> = Civic::ALL
                        .into_iter()
                        .filter(|kind| {
                            kind.available_in(game.region().kind) && kind.spec().unlock == selected
                        })
                        .map(|kind| kind.spec().name)
                        .collect();
                    text(detail, hud, &names.join(" · "), 15., INK);
                    let status = if current.knows(selected) {
                        "Erforscht".into()
                    } else if let Err(error) = current.available(selected, city_rank) {
                        error
                    } else {
                        "Bereit zur Forschung".into()
                    };
                    text(detail, hud, &status, 14., MUTED);
                    if !current.knows(selected) {
                        button(
                            detail,
                            hud,
                            &format!("Erforschen · {}", selected.price()),
                            UiAction::Do(Action::Research(selected)),
                            false,
                            true,
                        );
                    }
                });
        });
}

fn inspection_panel(
    parent: &mut ChildSpawnerCommands,
    hud: &Hud,
    game: &Game,
    play: &Play,
    short: bool,
) {
    let city = game.city();
    let region = game.region();
    if let Some(site) = city.building_at(play.aim) {
        panel_heading(parent, hud, site.kind.name(), short);
        let purpose = site
            .kind
            .civic()
            .map_or_else(|| site.kind.use_text(), |kind| kind.spec().purpose);
        text(parent, hud, purpose, 17., INK);
        let status = if city.connected(region, site) {
            "Am Wegenetz"
        } else {
            "Ohne Weganschluss"
        };
        text(
            parent,
            hud,
            if site.upgraded { "Stufe 2" } else { "Stufe 1" },
            15.,
            MUTED,
        );
        text(parent, hud, status, 15., MUTED);
        label(parent, hud, Label::CityGoods, 15., MUTED);
        parent.spawn(row()).with_children(|actions| {
            if site.kind.upgrade_price().is_some() && !site.upgraded {
                button(
                    actions,
                    hud,
                    "Ausbauen",
                    UiAction::Do(Action::Upgrade(site.tile)),
                    false,
                    false,
                );
            }
            if site.kind == Building::Civic(Civic::Barracks) {
                button(
                    actions,
                    hud,
                    "Wache ausbilden",
                    UiAction::Do(Action::Muster(site.tile)),
                    false,
                    true,
                );
            }
        });
    }
}

fn mode_panel(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, layout: Layout) {
    parent.spawn(row()).with_children(|header| {
        panel_heading(header, hud, "Hippogrove", layout.short);
        button(
            header,
            hud,
            "Kampagnen",
            UiAction::ModeTab(false),
            !layout.free_build_tab,
            false,
        );
        button(
            header,
            hud,
            "Freies Bauen",
            UiAction::ModeTab(true),
            layout.free_build_tab,
            false,
        );
        if game.has_save(game.mode) {
            button(header, hud, "Zurück", UiAction::Cancel, false, true);
        }
    });
    parent.spawn(row()).with_children(|cards| {
        for biome in Biome::ALL {
            let mode = if layout.free_build_tab {
                Mode::Sandbox(biome)
            } else {
                Mode::Campaign(biome)
            };
            let selected = game.mode == mode && game.has_save(mode);
            cards
                .spawn((
                    Button,
                    HitArea,
                    ControlButton {
                        action: UiAction::SelectMode(mode),
                        selected,
                        strong: false,
                    },
                    Node {
                        width: Val::Percent(if layout.short { 48. } else { 32. }),
                        min_height: Val::Px(if layout.short { 61. } else { 85. }),
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.),
                        padding: UiRect::all(Val::Px(4.)),
                        border: UiRect::all(Val::Px(2.)),
                        ..default()
                    },
                    BackgroundColor(rgb(if selected { JADE } else { SURFACE })),
                    BorderColor(rgb(BRASS)),
                    Outline::new(Val::Px(3.), Val::Px(2.), Color::NONE),
                    BorderRadius::all(Val::Px(13.)),
                ))
                .with_children(|card| {
                    card.spawn((
                        ImageNode::from_atlas_image(
                            hud.world_art.clone(),
                            TextureAtlas {
                                layout: hud.world_cards.clone(),
                                index: biome as usize,
                            },
                        ),
                        Node {
                            width: Val::Px(if layout.short { 61. } else { 70. }),
                            height: Val::Px(if layout.short { 49. } else { 64. }),
                            flex_shrink: 0.,
                            ..default()
                        },
                        BorderRadius::all(Val::Px(7.)),
                    ));
                    card.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        ..default()
                    })
                    .with_children(|words| {
                        let foreground = if selected { INK } else { LIGHT };
                        text(words, hud, mode.name(), 15., foreground);
                        let detail = if game.has_save(mode) {
                            "Fortsetzen"
                        } else if mode.sandbox() {
                            "Ohne Gegner · Alle Baupläne"
                        } else {
                            "3 Kapitel"
                        };
                        text(
                            words,
                            hud,
                            detail,
                            12.,
                            if selected { MUTED } else { SUBTLE },
                        );
                    });
                });
        }
    });
    if !layout.free_build_tab && game.has_save(Mode::Chronicle) {
        button(
            parent,
            hud,
            "Chronik fortsetzen",
            UiAction::SelectMode(Mode::Chronicle),
            game.mode == Mode::Chronicle,
            false,
        );
    }
}

fn material_glyph(biome: Biome, material: Material) -> usize {
    match (biome, material) {
        (Biome::Terraces, Material::Wood) => 25,
        (Biome::Terraces, Material::Stone) => 26,
        (_, Material::Wood) => 0,
        (_, Material::Stone) => 1,
        (Biome::Valley, Material::Food) => 19,
        (Biome::Fjord, Material::Food) => 21,
        (Biome::Terraces, Material::Food) => 22,
        (Biome::Lake, Material::Food) => 23,
        (Biome::Aegean, Material::Food) => 24,
    }
}

fn area_panel(
    parent: &mut ChildSpawnerCommands,
    hud: &Hud,
    game: &Game,
    layout: Layout,
    index: usize,
) {
    let region = game.region();
    let area = &region.areas[index];
    let (selected, active) = layout.area.unwrap();
    let style = ButtonStyle {
        compact: layout.narrow_modal,
        ..default()
    };
    let (card_height, icon_size) = if layout.narrow_modal {
        (103., 42.)
    } else if layout.short {
        (117., 52.)
    } else {
        (145., 68.)
    };
    parent
        .spawn(Node {
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.),
            ..default()
        })
        .with_children(|head| {
            head.spawn(Node {
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|title| {
                panel_heading(title, hud, &area.name, layout.short || layout.narrow_modal);
                label(title, hud, Label::AreaCamp(index), 15., DARK);
            });
            head.spawn(row()).with_children(|nav| {
                styled_button(
                    nav,
                    hud,
                    "Zum Lager",
                    UiAction::Focus(area.center),
                    false,
                    false,
                    style,
                );
                styled_button(nav, hud, "Landkarte", UiAction::Cancel, false, false, style);
            });
        });
    parent
        .spawn(Node {
            column_gap: Val::Px(8.),
            ..default()
        })
        .with_children(|cards| {
            for material in Material::ALL {
                let chosen = selected == material;
                cards
                    .spawn((
                        Button,
                        HitArea,
                        ControlButton {
                            action: UiAction::AreaMaterial(material),
                            selected: chosen,
                            strong: false,
                        },
                        Node {
                            flex_grow: 1.,
                            flex_basis: Val::Px(0.),
                            min_width: Val::Px(0.),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(2.),
                            padding: UiRect::all(Val::Px(6.)),
                            min_height: Val::Px(card_height),
                            border: UiRect::all(Val::Px(2.)),
                            ..default()
                        },
                        BackgroundColor(rgb(if chosen { JADE } else { 0xfcf3de })),
                        BorderColor(rgb(if chosen { GOLD } else { BRASS })),
                        Outline::new(Val::Px(3.), Val::Px(2.), Color::NONE),
                        BorderRadius::all(Val::Px(11.)),
                    ))
                    .with_children(|card| {
                        glyph(card, hud, material_glyph(region.kind, material), icon_size);
                        text(
                            card,
                            hud,
                            region.kind.material(material),
                            if layout.narrow_modal { 15. } else { 16. },
                            INK,
                        );
                        let supply = text(card, hud, "", 14., DARK);
                        card.commands().entity(supply).insert((
                            Label::AreaSupply(index, material),
                            TextLayout::new_with_justify(JustifyText::Center),
                        ));
                    });
            }
        });
    if active || layout.narrow_modal {
        label(
            parent,
            hud,
            Label::AreaStatus(index, selected, layout.narrow_modal),
            if layout.narrow_modal { 14. } else { 15. },
            DARK,
        );
    }
    let buttons = if layout.narrow_modal {
        column_grid(if active { 4 } else { 2 })
    } else {
        row()
    };
    parent.spawn(buttons).with_children(|actions| {
        styled_button(
            actions,
            hud,
            "Zur Quelle",
            UiAction::AreaSource(index, selected),
            false,
            true,
            style,
        );
        if active {
            for (name, action) in [
                ("Helfer senden", Action::AssignArea(index, selected, false)),
                ("Nilpferd senden", Action::AssignArea(index, selected, true)),
                ("Beenden", Action::CancelArea(index, selected)),
            ] {
                styled_button(
                    actions,
                    hud,
                    name,
                    UiAction::Do(action),
                    false,
                    false,
                    style,
                );
            }
        } else {
            styled_button(
                actions,
                hud,
                "Sammellager bauen",
                UiAction::Do(Action::Camp(index)),
                false,
                false,
                style,
            );
        }
    });
}

fn inventory_panel(parent: &mut ChildSpawnerCommands, hud: &Hud, layout: Layout, kind: Biome) {
    parent
        .spawn(Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: Val::Px(8.),
            ..default()
        })
        .with_children(|heading| {
            heading.spawn(row()).with_children(|name| {
                glyph(name, hud, 20, 31.);
                panel_heading(name, hud, "Vorräte", layout.short);
            });
            button(heading, hud, "Zurück", UiAction::Cancel, false, true);
        });
    for row in [
        [
            (
                kind.material(Material::Wood),
                material_glyph(kind, Material::Wood),
                Label::Stock(Material::Wood),
            ),
            (
                kind.material(Material::Stone),
                material_glyph(kind, Material::Stone),
                Label::Stock(Material::Stone),
            ),
            (
                kind.material(Material::Food),
                material_glyph(kind, Material::Food),
                Label::Stock(Material::Food),
            ),
        ],
        [
            ("Bretter", 16, Label::Refined(0)),
            ("Quader", 17, Label::Refined(1)),
            ("Werkzeug", 18, Label::Refined(2)),
        ],
    ] {
        parent
            .spawn(Node {
                width: Val::Percent(100.),
                column_gap: Val::Px(7.),
                ..default()
            })
            .with_children(|cards| {
                for (name, image, amount) in row {
                    cards
                        .spawn((
                            Node {
                                flex_grow: 1.,
                                flex_basis: Val::Px(0.),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                flex_direction: if layout.short {
                                    FlexDirection::Row
                                } else {
                                    FlexDirection::Column
                                },
                                column_gap: Val::Px(5.),
                                padding: UiRect::all(Val::Px(5.)),
                                min_height: Val::Px(if layout.short { 71. } else { 114. }),
                                border: UiRect::all(Val::Px(1.)),
                                ..default()
                            },
                            BackgroundColor(rgb(0xfcf3de)),
                            BorderColor(rgb(0xdac398)),
                            BorderRadius::all(Val::Px(12.)),
                        ))
                        .with_children(|card| {
                            glyph(card, hud, image, if layout.short { 48. } else { 64. });
                            card.spawn(Node {
                                align_items: AlignItems::Center,
                                flex_direction: FlexDirection::Column,
                                ..default()
                            })
                            .with_children(|details| {
                                text(details, hud, name, 15., INK);
                                label(details, hud, amount, 24., DARK);
                            });
                        });
                }
            });
    }
    parent
        .spawn((
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: Val::Px(8.),
                padding: UiRect::axes(Val::Px(10.), Val::Px(4.)),
                ..default()
            },
            BackgroundColor(rgb(0xe8dfc4)),
            BorderRadius::all(Val::Px(10.)),
        ))
        .with_children(|cargo| {
            cargo.spawn(row()).with_children(|bundle| {
                let picture = layout
                    .carried
                    .map_or(5, |material| material_glyph(kind, material));
                glyph(bundle, hud, picture, 34.);
                label(bundle, hud, Label::Carried, 15., INK);
            });
            label(cargo, hud, Label::Capacity, 13., MUTED);
        });
}

fn cutscene(parent: &mut ChildSpawnerCommands, hud: &Hud, layout: Layout) {
    parent
        .spawn(Node {
            position_type: PositionType::Absolute,
            right: Val::Px(EDGE),
            top: Val::Px(EDGE),
            ..default()
        })
        .with_children(|skip| {
            button(
                skip,
                hud,
                "Überspringen",
                UiAction::FinishStory,
                false,
                false,
            );
        });
    parent
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(EDGE),
            right: Val::Px(EDGE),
            bottom: Val::Px(if layout.short { 20. } else { 34. }),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Node {
                    max_width: Val::Px(660.),
                    padding: UiRect::axes(Val::Px(19.), Val::Px(10.)),
                    border: UiRect::all(Val::Px(1.)),
                    ..default()
                },
                BackgroundColor(rgb(0x172d25).with_alpha(0.93)),
                BorderColor(rgb(BRASS)),
                BorderRadius::all(Val::Px(13.)),
            ))
            .with_children(|card| {
                let subtitle = text(card, hud, "", if layout.short { 18. } else { 23. }, LIGHT);
                card.commands().entity(subtitle).insert((
                    Label::Subtitle,
                    TextLayout::new_with_justify(JustifyText::Center),
                ));
            });
        });
}

fn modal(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, play: &Play, layout: Layout) {
    let region = game.region();
    let city = game.city();
    let tight = layout.narrow_modal || (play.panel == Panel::Map && layout.short);
    parent.spawn((
        Node{width:Val::Percent(100.),height:Val::Percent(100.),align_items:AlignItems::Center,
            justify_content:JustifyContent::Center,..default()},
        BackgroundColor(rgb(0x14261d).with_alpha(0.64)),
    )).with_children(|shade| {
        shade.spawn((
            HitArea,
            Node{
                width:Val::Percent(if layout.narrow_modal { 94. } else { 92. }),max_width:Val::Px(720.),flex_direction:FlexDirection::Column,
                padding:UiRect::all(Val::Px(if tight { 8. } else if layout.short { 12. } else { 21. })),
                border:UiRect::all(Val::Px(2.)),row_gap:Val::Px(if tight { 5. } else if layout.short { 6. } else { 10. }),..default()
            },
            BackgroundColor(rgb(PAPER)),BorderColor(rgb(BRASS)),BorderRadius::all(Val::Px(18.)),
        )).with_children(|box_node| {
            if !matches!(play.panel, Panel::Inventory | Panel::Modes | Panel::Area(_)) {
                modal_navigation(box_node, hud, play.panel, layout.short);
            }
            match play.panel {
                Panel::Journey => {
                    box_node.spawn(row()).with_children(|heading| {
                        panel_heading(heading,hud,if game.campaign_complete() { "Die Schreine sind vereint" } else { game.mode.name() },layout.short);
                        if game.campaign.active == 0 && !game.mode.sandbox() {
                            button(heading,hud,"Einführung",UiAction::ReplayArrival,false,false);
                        }
                        button(heading,hud,"Spielmodi",UiAction::Panel(Panel::Modes),false,false);
                    });
                    box_node.spawn(row()).with_children(|chapters| {
                        for (index,candidate) in game.regions.iter().enumerate() {
                            let result=game.campaign.settlements[index].as_ref().filter(|place| place.quest == Quest::Complete).and_then(|place|place.outcome);
                            let suffix=if index>game.campaign.unlocked {" · verschlossen"}
                                else if let Some(outcome)=result { if outcome==Outcome::Alliance {" · Bündnis"} else {" · befreit"} }
                                else {""};
                            chapters.spawn(Node { width: Val::Percent(48.), flex_direction: FlexDirection::Column, ..default() }).with_children(|card| {
                                button(card,hud,&format!("{}{suffix}",candidate.name),UiAction::Do(Action::Travel(index)),index==game.campaign.active,false);
                                if !layout.short { text(card,hud,&candidate.subtitle,14.,MUTED); }
                            });
                        }
                    });
                }
                Panel::Map => map_panel(box_node,hud,game,layout),
                Panel::Area(index) => area_panel(box_node,hud,game,layout,index),
                Panel::Catalog => catalog_panel(box_node,hud,game,play,layout.short),
                Panel::Research => research_panel(box_node,hud,game,play.tech,layout.short),
                Panel::Inspection => inspection_panel(box_node,hud,game,play,layout.short),
                Panel::Inventory => inventory_panel(box_node, hud, layout, region.kind),
                Panel::Modes => mode_panel(box_node, hud, game, layout),
                Panel::Progress => {
                    text(box_node,hud,"Gott, Siedlung und Kreatur",31.,INK);
                    label(box_node,hud,Label::Progress,17.,MUTED);
                    box_node.spawn(row()).with_children(|talents| {
                        for talent in Talent::ALL {
                            button(talents,hud,&format!("{} {}/2",talent.name(),game.campaign.gifts.rank(talent)),UiAction::Do(Action::Learn(talent)),false,false);
                        }
                        button(talents,hud,"Forschung",UiAction::Panel(Panel::Research),false,false);
                        button(talents,hud,"Vorräte",UiAction::Panel(Panel::Inventory),false,false);
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
                Panel::Closed | Panel::Story => {}
            }
            // The same feedback is visible while purchasing a talent or choosing a chapter.
            if play.panel != Panel::Map && !layout.narrow_modal {
                label(box_node,hud,Label::Notice,16.,DARK);
            }
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
    mut images: ResMut<Assets<Image>>,
) {
    if hud.maps_mode != Some(game.mode) {
        hud.maps = game
            .regions
            .iter()
            .map(|region| images.add(chart(region)))
            .collect();
        hud.maps_mode = Some(game.mode);
    }
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
        mode: game.mode,
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
        short: size.y / scale.0 < 490. || (play.panel == Panel::Map && size.x / scale.0 < 690.),
        narrow_modal: match play.panel {
            Panel::Map => size.x / scale.0 < 550.,
            Panel::Area(_) => size.x / scale.0 < 550. || size.y / scale.0 < 320.,
            _ => false,
        },
        safe,
        catalog: play.catalog_branch,
        catalog_page: play.catalog_page,
        tech: play.tech,
        research_count: game.campaign.research.count(),
        insight: if play.panel == Panel::Research {
            game.campaign.research.insight
        } else {
            0
        },
        carried: if play.panel == Panel::Inventory {
            game.city()
                .held
                .map(|bundle| game.region().sources[bundle.source].material)
        } else {
            None
        },
        free_build_tab: play.free_build_tab,
        area: match play.panel {
            Panel::Area(index) => Some((play.area_material, game.city().camps[index])),
            _ => None,
        },
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
                } else if play.panel == Panel::Story {
                    cutscene(safe_root, &hud, key);
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
            Label::Mana => format!("{:.0}/{:.0}", city.mana, city.mana_cap(region, gifts)),
            Label::Objective => city.objective(region),
            Label::Climate => region.weather(city.weather_time).name().into(),
            Label::CityGoods => format!(
                "{} Bretter · {} Quader · {} Werkzeug",
                city.society.goods[0], city.society.goods[1], city.society.goods[2]
            ),
            Label::Refined(index) => city.society.goods[*index].to_string(),
            Label::Carried => city.held.map_or_else(
                || "Pfote ist frei".into(),
                |bundle| {
                    format!(
                        "{} {} in der Pfote",
                        bundle.count,
                        region.kind.material(region.sources[bundle.source].material)
                    )
                },
            ),
            Label::Capacity => format!("{} Lagerplatz je Rohstoff", city.storage_cap()),
            Label::AreaCamp(index) => if city.camps[*index] {
                "Sammellager aktiv"
            } else {
                city.camp_plan(*index)
                    .err()
                    .unwrap_or("Sammellager · 5 Holz · 6 Glauben")
            }
            .into(),
            Label::AreaSupply(area, material) => {
                let supply = city.area_supply(region, *area, *material);
                format!(
                    "{} verfügbar\n{} von {} Quellen",
                    supply.units, supply.stocked, supply.sources
                )
            }
            Label::AreaStatus(_, _, true)
                if time.elapsed_secs() < play.notice_until && !play.notice.is_empty() =>
            {
                play.notice.clone()
            }
            Label::AreaStatus(area, _, _) if !city.camps[*area] => String::new(),
            Label::AreaStatus(area, material, _) => {
                let (helpers, hippo) = city.area_crew(*area, *material);
                let mut activity = match (helpers, hippo) {
                    (0, false) => "Kein aktiver Auftrag für diesen Rohstoff".into(),
                    (0, true) => "Das Nilpferd sammelt hier".into(),
                    (1, false) => "Ein Gläubiger sammelt hier".into(),
                    (1, true) => "Ein Gläubiger und das Nilpferd sammeln hier".into(),
                    (count, false) => format!("{count} Gläubige sammeln hier"),
                    (count, true) => format!("{count} Gläubige und das Nilpferd sammeln hier"),
                };
                if helpers > 0 || hippo {
                    if city.stock[material.index()] >= city.storage_cap() {
                        activity.push_str(" · Lager voll");
                    } else if city.area_supply(region, *area, *material).units == 0 {
                        activity.push_str(" · Quellen erholen sich");
                    }
                }
                activity
            }
            Label::Subtitle => play
                .story
                .map_or_else(String::new, |story| story.caption(region).into()),
            Label::Quest if game.mode.sandbox() => if play.objective_expanded {
                "Freies Bauen −"
            } else {
                "Freies Bauen +"
            }
            .into(),
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
                if play.gamepad && play.tool == Tool::Build {
                    "Südtaste · Bauen · Linke Schulter · Baubuch".into()
                } else if play.gamepad {
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
                "{} Talentpunkte · {} Erkenntnis · Dorfstufe {}\n{} von {} Wohnplätzen · {} Geburten · Stimmung {}%\n{} Wachen · {} Wege · Lagerplatz {}\nNilpferd-Rang {} · {} · {:.0}% Kraft",
                gifts.points,
                game.campaign.research.insight,
                city.city_rank(),
                city.inhabitants(),
                city.housing(),
                city.society.births,
                city.society.morale,
                city.garrison(),
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
            Label::DistrictBrief(index) => {
                let state = city.districts[*index];
                match state.outcome {
                    Some(Outcome::Alliance) => "Verbündet".into(),
                    Some(Outcome::Victory) => "Befreit".into(),
                    None => format!("Widerstand {}", state.resolve),
                }
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
        meter.width =
            Val::Percent((100. * city.mana / city.mana_cap(region, gifts)).clamp(0., 100.));
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
            UiAction::Do(Action::Research(tech)) => game
                .campaign
                .research
                .available(tech, city.city_rank())
                .is_err(),
            UiAction::Do(Action::Camp(index)) => city.camp_plan(index).is_err(),
            UiAction::Do(Action::AssignArea(_, _, false)) => !city.idle_helper(),
            UiAction::Do(Action::AssignArea(area, material, true)) => {
                city.area_crew(area, material).1
            }
            UiAction::Do(Action::CancelArea(area, material)) => {
                city.area_crew(area, material) == (0, false)
            }
            _ => false,
        };
        let shade = if unavailable {
            if button.strong { 0xc9c3ab } else { 0x59665a }
        } else if focused || hovered {
            if button.strong {
                0xffe5b2
            } else if button.selected {
                0xe5f2cc
            } else if matches!(button.action, UiAction::AreaMaterial(_)) {
                0xf5e5c3
            } else {
                0x4a6855
            }
        } else if button.strong {
            GOLD
        } else if button.selected {
            JADE
        } else if matches!(button.action, UiAction::AreaMaterial(_)) {
            0xfcf3de
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
