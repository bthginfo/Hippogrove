#[path = "hud_quests.rs"]
mod quests_ui;
#[path = "hud_neighborhood.rs"]
mod town_ui;
#[path = "hud_work.rs"]
mod works;
pub(crate) use quests_ui::update_meter;
#[path = "hud_villages.rs"]
mod villages;
use bevy::window::PrimaryWindow;
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::{
    art::rgb,
    civic::{Branch, Civic, Tech},
    game::{
        Action, Building, FARM_REWARD, FirstDelivery, Game, Miracle, Outcome, PAW_BUNDLE, Quest,
        Settlement, SideQuest, Talent, stock_text,
    },
    modes::Mode,
    play::{CATALOG_PAGE_SIZE, CameraRig, Feedback, Panel, Play, Target, Tool, UiAction},
    region::{Biome, Material, Region},
    sound::{Bus, Mix, Sound},
    view::Board,
    wheel::Wheel,
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
const PAD_SQUARE: usize = 32;
const PAD_TRIANGLE: usize = 33;
const PAD_CROSS: usize = 34;

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
    FaithUse,
    ManaUse,
    Objective,
    CampaignHint,
    CircleStatus,
    VillageWatch,
    Quest,
    QuestCount,
    SideTrack,
    WorkHint,
    Notice,
    Confirm,
    Detail,
    Rival,
    District(usize),
    DistrictBrief(usize),
    LocalQuest,
    Calendar,
    Raid,
    RaidAlert,
    Mana,
    Creature,
    Helpers,
    Spell(Miracle),
    Order(Tool, usize),
    Refined(usize),
    Carried,
    Capacity,
    AreaCamp(usize),
    AreaSupply(usize, Material),
    AreaStatus(usize, Material, bool),
    Subtitle,
    FirstDelivery(FirstDelivery),
    PracticeStatus,
    ResumeMode,
    AudioStatus,
    WheelChoice,
    WheelDetail,
    BlueprintStatus,
    CatalogName,
    CatalogPurpose,
    CatalogPrice,
    CatalogReadiness,
    CatalogAction,
}

#[derive(Component)]
pub(crate) struct Toast {
    overlay: bool,
}

#[derive(Component)]
pub(crate) struct FeedbackIcon;

#[derive(Component)]
pub struct CameraFade;

#[derive(Component)]
pub struct LoadingWorld;

#[derive(Component)]
pub struct PadReticle;

#[derive(Component)]
pub struct ObjectPopup;

#[derive(Component)]
pub struct WorldReceipt;

#[derive(Component)]
pub(crate) struct CatalogPortrait;

#[derive(Clone, Copy, Eq, PartialEq)]
struct Layout {
    region: usize,
    menu_page: u8,
    lesson: Option<(crate::practice::Step, UiAction)>,
    mode: Mode,
    tool: Tool,
    option: usize,
    panel: Panel,
    gamepad: bool,
    unlocked: usize,
    quest: Quest,
    side_quest: SideQuest,
    folded: bool,
    work_stage: crate::game::WorkStage,
    work_pinned: bool,
    reading: crate::display::ReadingSize,
    work_paid: u16,
    appeal: Option<usize>,
    mix: Option<Mix>,
    campaign_complete: bool,
    circle_active: bool,
    talents: [u8; Talent::ALL.len()],
    road_started: bool,
    compact: bool,
    short: bool,
    narrow_legend: bool,
    narrow_modal: bool,
    safe: [u16; 4],
    catalog: Option<Branch>,
    catalog_page: usize,
    inspected: Option<(Building, bool)>,
    can_inspect: bool,
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
    map: Option<((Mode, usize), Handle<Image>)>,
    world_art: Handle<Image>,
    work_art: [Handle<Image>; 5],
    world_cards: Handle<TextureAtlasLayout>,
    building_art: [Handle<Image>; Biome::ALL.len()],
    building_cards: Handle<TextureAtlasLayout>,
}

pub fn setup(
    mut commands: Commands,
    mut hud: ResMut<Hud>,
    assets: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    hud.work_art = ["valley", "fjord", "terraces", "lake", "aegean"]
        .map(|s| assets.load(format!("embedded://ui/works/{s}.png")));
    hud.regular = assets.load("embedded://fonts/DMSans-Medium.ttf");
    hud.bold = assets.load("embedded://fonts/DMSans-Bold.ttf");
    commands.spawn((
        PadReticle,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(13.),
            height: Val::Px(13.),
            border: UiRect::all(Val::Px(2.)),
            ..default()
        },
        BorderColor(rgb(LIGHT)),
        BorderRadius::all(Val::Px(7.)),
        Outline::new(Val::Px(1.), Val::Px(1.), rgb(DARK)),
        GlobalZIndex(40),
        Visibility::Hidden,
    ));
    commands
        .spawn((
            CameraFade,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            GlobalZIndex(80),
            BackgroundColor(rgb(0x172821)),
            Visibility::Visible,
        ))
        .with_children(|overlay| {
            overlay.spawn((
                LoadingWorld,
                Text::new("Die Welt lädt …"),
                TextFont {
                    font: hud.bold.clone(),
                    font_size: 24.,
                    ..default()
                },
                TextColor(rgb(LIGHT)),
                Visibility::Inherited,
            ));
        });
    commands.spawn((
        WorldReceipt,
        Text::new(""),
        TextFont {
            font: hud.bold.clone(),
            font_size: 14.,
            ..default()
        },
        TextColor(rgb(LIGHT)),
        Node {
            position_type: PositionType::Absolute,
            max_width: Val::Px(248.),
            padding: UiRect::axes(Val::Px(7.), Val::Px(3.)),
            ..default()
        },
        BackgroundColor(rgb(DARK)),
        BorderRadius::all(Val::Px(7.)),
        GlobalZIndex(18),
        Visibility::Hidden,
    ));
    hud.serif = assets.load("embedded://fonts/DMSerifText-Regular.ttf");
    hud.icons = assets.load("embedded://ui/atlas.png");
    hud.atlas = atlases.add(TextureAtlasLayout::from_grid(
        UVec2::splat(64),
        4,
        9,
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
    hud.building_art = ["valley", "fjord", "terraces", "lake", "aegean"]
        .map(|world| assets.load(format!("embedded://ui/buildings/{world}.png")));
    hud.building_cards = atlases.add(TextureAtlasLayout::from_grid(
        UVec2::splat(96),
        8,
        Building::CATALOG_SIZE.div_ceil(8) as u32,
        None,
        None,
    ));
}

pub fn camera_fade(
    camera: Res<CameraRig>,
    board: Res<Board>,
    game: Res<Game>,
    mut overlay: Single<
        (&mut BackgroundColor, &mut Visibility),
        (With<CameraFade>, Without<LoadingWorld>),
    >,
    mut message: Single<&mut Visibility, (With<LoadingWorld>, Without<CameraFade>)>,
) {
    let loading = board.loading(&game);
    let opacity = if loading {
        1.
    } else {
        camera.cover * camera.cover
    };
    overlay.0.0 = rgb(0x172821).with_alpha(opacity);
    *overlay.1 = if opacity > 0.001 {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    **message = if loading {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
}

pub fn pad_reticle(
    play: Res<Play>,
    game: Res<Game>,
    scale: Res<UiScale>,
    camera: Single<(&Camera, &GlobalTransform), With<crate::view::WorldCamera>>,
    reticle: Single<(&mut Node, &mut Visibility), With<PadReticle>>,
) {
    let (mut node, mut visible) = reticle.into_inner();
    if !play.gamepad || play.panel != Panel::Closed {
        visible.set_if_neq(Visibility::Hidden);
        return;
    }
    let (camera, pose) = *camera;
    let position = play
        .pointer
        .or_else(|| crate::controls::aim_on_screen(&play, &game, camera, pose));
    if let Some(point) = position {
        let logical = point / scale.0;
        let (left, top) = (Val::Px(logical.x - 6.5), Val::Px(logical.y - 6.5));
        if node.left != left || node.top != top {
            node.left = left;
            node.top = top;
        }
        visible.set_if_neq(Visibility::Visible);
    } else {
        visible.set_if_neq(Visibility::Hidden);
    }
}

pub fn world_labels(
    game: Res<Game>,
    play: Res<Play>,
    time: Res<Time>,
    scale: Res<UiScale>,
    camera: Single<(&Camera, &GlobalTransform), With<crate::view::WorldCamera>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut popups: Query<(&mut Node, &ComputedNode), With<ObjectPopup>>,
    receipt: Single<
        (&mut Text, &mut Node, &ComputedNode, &mut Visibility),
        (With<WorldReceipt>, Without<ObjectPopup>),
    >,
) {
    let (camera, pose) = *camera;
    let [left, top, right, bottom] = crate::controls::safe_insets(&window).map(f32::from);
    let margin = Vec2::new(left, top) / scale.0;
    let available = (Vec2::new(
        window.width() - left - right,
        window.height() - top - bottom,
    ) / scale.0)
        .max(Vec2::ONE);
    if let Ok((mut node, size)) = popups.single_mut() {
        let size = size.size() * size.inverse_scale_factor();
        let point = crate::controls::aim_on_screen(&play, &game, camera, pose)
            .map_or(available * 0.5, |point| point / scale.0 - margin);
        let x = if point.x + size.x + 20. <= available.x {
            point.x + 14.
        } else {
            point.x - size.x - 14.
        };
        node.left = Val::Px(x.clamp(5., (available.x - size.x - 5.).max(5.)));
        node.top =
            Val::Px((point.y - size.y * 0.48).clamp(5., (available.y - size.y - 5.).max(5.)));
    }
    let (mut text, mut node, size, mut visibility) = receipt.into_inner();
    let now = time.elapsed_secs();
    if let Some((tile, at, message)) = play
        .receipt
        .as_ref()
        .filter(|(_, at, _)| now - at < 2.3 && play.panel == Panel::Closed && available.y >= 460.)
    {
        let point = crate::terrain::on_ground(game.region(), tile.world()) + Vec3::Y * 1.5;
        if let Ok(point) = camera.world_to_viewport(pose, point) {
            let point = point / scale.0;
            let width = size.size().x * size.inverse_scale_factor();
            node.left = Val::Px((point.x - width * 0.5).clamp(
                margin.x + 4.,
                (margin.x + available.x - width - 4.).max(margin.x + 4.),
            ));
            node.top = Val::Px((point.y - 17. - (now - at) * 7.).max(margin.y));
            if text.0 != *message {
                text.0.clone_from(message);
            }
            visibility.set_if_neq(Visibility::Visible);
            return;
        }
    }
    visibility.set_if_neq(Visibility::Hidden);
}

fn object_popup(parent: &mut ChildSpawnerCommands, hud: &Hud, play: &Play) {
    let Some(menu) = &play.object_menu else {
        return;
    };
    parent
        .spawn((
            ObjectPopup,
            HitArea,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(228.),
                padding: UiRect::all(Val::Px(7.)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.),
                border: UiRect::all(Val::Px(2.)),
                ..default()
            },
            BackgroundColor(rgb(DARK)),
            BorderColor(rgb(BRASS)),
            BorderRadius::all(Val::Px(12.)),
        ))
        .with_children(|card| {
            card.spawn(Node {
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                column_gap: Val::Px(3.),
                ..default()
            })
            .with_children(|header| {
                text(header, hud, &menu.title, 16., GOLD);
                styled_button(
                    header,
                    hud,
                    "×",
                    UiAction::Cancel,
                    false,
                    false,
                    ButtonStyle {
                        compact: true,
                        height: Some(40.),
                        ..default()
                    },
                );
            });
            text(card, hud, &menu.detail, 13., SUBTLE);
            for (i, choice) in menu.choices.iter().enumerate() {
                styled_button(
                    card,
                    hud,
                    &choice.label,
                    choice.action,
                    false,
                    i == 0,
                    ButtonStyle {
                        compact: true,
                        height: Some(43.),
                        icon: Some(choice.icon),
                        ..default()
                    },
                );
            }
        });
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
            let color = if region.bridge_cell(tile) {
                0xe0c69b
            } else if region.water(tile) {
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

fn glyph(parent: &mut ChildSpawnerCommands, hud: &Hud, index: usize, size: f32) -> Entity {
    parent
        .spawn((
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
        ))
        .id()
}

fn building_icon(
    parent: &mut ChildSpawnerCommands,
    hud: &Hud,
    biome: Biome,
    kind: Building,
    size: f32,
) -> Entity {
    parent
        .spawn((
            ImageNode::from_atlas_image(
                hud.building_art[biome as usize].clone(),
                TextureAtlas {
                    layout: hud.building_cards.clone(),
                    index: kind.index(),
                },
            ),
            Node {
                width: Val::Px(size),
                height: Val::Px(size),
                flex_shrink: 0.,
                ..default()
            },
            BorderRadius::all(Val::Px(7.)),
        ))
        .id()
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
        UiAction::Panel(Panel::Quests) => Some(11),
        UiAction::Panel(Panel::Catalog) => Some(12),
        UiAction::Panel(Panel::Research) => Some(9),
        UiAction::Panel(Panel::Inventory) => Some(20),
        UiAction::Panel(Panel::Modes) => Some(10),
        UiAction::MainMenu => Some(31),
        UiAction::Panel(Panel::Controls) => Some(30),
        UiAction::Panel(Panel::Sound) | UiAction::Sound(_) | UiAction::TestSound => Some(28),
        UiAction::Rotate => Some(29),
        UiAction::Inspect => Some(12),
        UiAction::ToolTarget => Some(14),
        UiAction::Do(Action::Upgrade(_)) => Some(6),
        UiAction::Do(Action::AcceptLocal | Action::FinishLocal) => Some(11),
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
    height: Option<f32>,
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
                min_width: Val::Px(if tool && !style.compact { 60. } else { 48. }),
                min_height: Val::Px(style.height.unwrap_or(if tool {
                    if style.compact { 54. } else { 66. }
                } else if strong && !style.compact {
                    54.
                } else {
                    48.
                })),
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
                        if style.compact { 24. } else { 32. }
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
                } else if matches!(action, UiAction::CatalogBuild) {
                    copy.commands().entity(entity).insert(Label::CatalogAction);
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
    let edge = if layout.short { 8. } else { EDGE };
    let quick = ButtonStyle {
        compact: true,
        ..default()
    };
    parent
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(edge),
            left: Val::Px(edge),
            right: Val::Px(edge),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Start,
            column_gap: Val::Px(7.),
            ..default()
        })
        .with_children(|header| {
            header
                .spawn((
                    Button,
                    HitArea,
                    ControlButton {
                        action: UiAction::Panel(Panel::Inventory),
                        selected: false,
                        strong: false,
                    },
                    Node {
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
                        min_height: Val::Px(48.),
                        padding: UiRect::axes(Val::Px(9.), Val::Px(4.)),
                        column_gap: Val::Px(12.),
                        border: UiRect::all(Val::Px(1.)),
                        ..default()
                    },
                    BackgroundColor(rgb(DARK)),
                    BorderColor(rgb(BRASS)),
                    BorderRadius::all(Val::Px(12.)),
                    Outline::new(Val::Px(2.), Val::Px(2.), Color::NONE),
                ))
                .with_children(|economy| {
                    text(
                        economy,
                        hud,
                        &game.region().name,
                        if layout.compact { 19. } else { 25. },
                        LIGHT,
                    );
                    economy
                        .spawn(Node {
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(5.),
                            ..default()
                        })
                        .with_children(|stats| {
                            for (icon, value) in [
                                (0, Label::Stock(Material::Wood)),
                                (1, Label::Stock(Material::Stone)),
                                (2, Label::Stock(Material::Food)),
                                (3, Label::Faith),
                                (4, Label::Mana),
                            ] {
                                stats
                                    .spawn(Node {
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Px(1.),
                                        ..default()
                                    })
                                    .with_children(|count| {
                                        glyph(
                                            count,
                                            hud,
                                            icon,
                                            if layout.short { 17. } else { 20. },
                                        );
                                        label(count, hud, value, 14., LIGHT);
                                    });
                            }
                        });
                });
            header
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::End,
                    row_gap: Val::Px(5.),
                    ..default()
                })
                .with_children(|right| {
                    right
                        .spawn(Node {
                            column_gap: Val::Px(5.),
                            ..default()
                        })
                        .with_children(|links| {
                            styled_button(
                                links,
                                hud,
                                if game.region().tutorial {
                                    "Karte"
                                } else {
                                    "Dörfer"
                                },
                                UiAction::Panel(if game.region().tutorial {
                                    Panel::Map
                                } else {
                                    Panel::Council
                                }),
                                false,
                                false,
                                ButtonStyle {
                                    icon: Some(10),
                                    ..quick
                                },
                            );
                            styled_button(
                                links,
                                hud,
                                "Menü",
                                UiAction::Panel(Panel::Menu),
                                false,
                                false,
                                quick,
                            );
                            for (name, closer) in [("−", false), ("+", true)] {
                                styled_button(
                                    links,
                                    hud,
                                    name,
                                    UiAction::Zoom(closer),
                                    false,
                                    false,
                                    quick,
                                );
                            }
                        });
                });
        });

    if game.mode != Mode::Tutorial {
        island_agenda(parent, hud, game, layout);
        return;
    }
    feedback_toast(
        parent,
        hud,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(if layout.short { 63. } else { 74. }),
            left: Val::Px(if layout.short { 8. } else { EDGE }),
            width: Val::Percent(if layout.short { 47. } else { 43. }),
            max_width: Val::Px(390.),
            ..default()
        },
        true,
    );
}

fn island_agenda(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, layout: Layout) {
    let edge = if layout.short { 8. } else { EDGE };
    let quick = ButtonStyle {
        compact: true,
        height: Some(if layout.short { 37. } else { 44. }),
        ..default()
    };
    parent
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(if layout.compact { 63. } else { 70. }),
            left: Val::Px(edge),
            right: Val::Px(edge),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Start,
            column_gap: Val::Px(6.),
            ..default()
        })
        .with_children(|bar| {
            if layout.work_pinned
                || (!game.region().sandbox
                    && (game.city().quest != Quest::Complete
                        || matches!(layout.side_quest, SideQuest::Active | SideQuest::Ready)))
            {
                quests_ui::tracker(bar, hud, game, layout);
            } else {
                bar.spawn((
                    HitArea,
                    Node {
                        max_width: Val::Percent(if layout.short { 74. } else { 64. }),
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.),
                        padding: UiRect::axes(Val::Px(6.), Val::Px(3.)),
                        border: UiRect::left(Val::Px(3.)),
                        ..default()
                    },
                    BackgroundColor(rgb(PAPER)),
                    BorderColor(rgb(BRASS)),
                    BorderRadius::all(Val::Px(7.)),
                ))
                .with_children(|card| {
                    card.spawn(Node {
                        min_width: Val::Px(0.),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Start,
                        ..default()
                    })
                    .with_children(|copy| {
                        let title = if game.region().sandbox {
                            "Freies Bauen".into()
                        } else if game.city().quest == Quest::Complete {
                            "Insel gesichert".into()
                        } else {
                            format!("Inselziel · {} / 4", game.city().quest.index() + 1)
                        };
                        text(copy, hud, &title, 11., MUTED);
                        label(
                            copy,
                            hud,
                            Label::Quest,
                            if layout.short { 14. } else { 17. },
                            INK,
                        );
                        if game.region().sandbox && !layout.short {
                            text(copy, hud, "Die Nachbardörfer bieten Waren an.", 13., MUTED);
                        } else if !layout.short {
                            label(copy, hud, Label::CampaignHint, 13., MUTED);
                        }
                    });
                    if game.region().sandbox || game.city().quest == Quest::Complete {
                        let market = game
                            .region()
                            .districts
                            .iter()
                            .enumerate()
                            .find(|(i, _)| {
                                game.city().trade_ready(game.region(), *i, false).is_ok()
                            })
                            .map_or(UiAction::Panel(Panel::Map), |(_, district)| {
                                UiAction::Focus(district.center)
                            });
                        styled_button(card, hud, "Handeln", market, false, false, quick);
                    }
                    if !game.region().sandbox {
                        styled_button(
                            card,
                            hud,
                            if game.city().quest == Quest::Complete {
                                if layout.gamepad {
                                    "L3 · Reisebuch"
                                } else {
                                    "Reisebuch"
                                }
                            } else if layout.gamepad {
                                "L3 · Ziel"
                            } else {
                                "Ziel zeigen"
                            },
                            UiAction::Guide,
                            false,
                            false,
                            quick,
                        );
                    }
                });
            }
            bar.spawn(Node {
                max_width: Val::Percent(40.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::End,
                ..default()
            })
            .with_children(|right| {
                styled_button(
                    right,
                    hud,
                    if layout.circle_active {
                        "Zum Festkreis"
                    } else {
                        "Dorfruf"
                    },
                    UiAction::Panel(Panel::Gathering),
                    false,
                    layout.circle_active,
                    ButtonStyle {
                        icon: Some(3),
                        ..quick
                    },
                );
                if let Some(i) = layout.appeal {
                    styled_button(
                        right,
                        hud,
                        "Zum Hilferuf",
                        UiAction::Panel(Panel::Projects(i)),
                        false,
                        true,
                        ButtonStyle {
                            icon: Some(11),
                            ..quick
                        },
                    );
                }
                if !layout.circle_active && !game.region().sandbox {
                    right
                        .spawn((
                            Node {
                                max_width: Val::Px(if layout.short { 188. } else { 275. }),
                                padding: UiRect::axes(Val::Px(4.), Val::Px(2.)),
                                ..default()
                            },
                            BackgroundColor(rgb(DARK)),
                        ))
                        .with_children(|chip| label(chip, hud, Label::VillageWatch, 12., LIGHT));
                }
                if layout.circle_active {
                    right
                        .spawn((
                            Node {
                                padding: UiRect::axes(Val::Px(4.), Val::Px(2.)),
                                ..default()
                            },
                            BackgroundColor(rgb(DARK)),
                        ))
                        .with_children(|chip| label(chip, hud, Label::CircleStatus, 12., LIGHT));
                }
                if game.region().has_rival_creature() && layout.quest == Quest::Bridge {
                    label(right, hud, Label::RaidAlert, 13., GOLD);
                }
            });
        });
}

fn feedback_toast(parent: &mut ChildSpawnerCommands, hud: &Hud, mut node: Node, overlay: bool) {
    node.align_items = AlignItems::Center;
    node.column_gap = Val::Px(6.);
    node.padding = UiRect::axes(Val::Px(8.), Val::Px(5.));
    node.display = Display::None;
    parent
        .spawn((
            Toast { overlay },
            node,
            Visibility::Hidden,
            BackgroundColor(rgb(DARK)),
            BorderRadius::all(Val::Px(10.)),
        ))
        .with_children(|toast| {
            let image = glyph(toast, hud, 11, 22.);
            toast.commands().entity(image).insert(FeedbackIcon);
            label(toast, hud, Label::Notice, 14., LIGHT);
        });
}

fn lesson_bar(
    parent: &mut ChildSpawnerCommands,
    hud: &Hud,
    step: crate::practice::Step,
    action: UiAction,
    layout: Layout,
) {
    use crate::practice::Step;
    let (index, title) = step.chapter();
    let waiting = matches!(step, Step::Grow | Step::Delivery);
    let icon = match step {
        Step::House => 6,
        Step::Field | Step::Harvest | Step::Grow => 2,
        Step::Rain => 9,
        Step::SendHippo | Step::Delivery | Step::Pet => 8,
        _ => 5,
    };
    let edge = if layout.short { 8. } else { EDGE };
    parent
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(edge),
            right: Val::Px(edge),
            bottom: Val::Px(edge),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|anchor| {
            anchor
                .spawn((
                    HitArea,
                    Node {
                        width: Val::Percent(100.),
                        max_width: Val::Px(660.),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::axes(Val::Px(13.), Val::Px(8.)),
                        row_gap: Val::Px(5.),
                        border: UiRect::all(Val::Px(1.)),
                        ..default()
                    },
                    BackgroundColor(rgb(DARK)),
                    BorderColor(rgb(BRASS)),
                    BorderRadius::all(Val::Px(13.)),
                ))
                .with_children(|card| {
                    card.spawn(Node {
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.),
                        ..default()
                    })
                    .with_children(|heading| {
                        glyph(heading, hud, icon, 23.);
                        text(heading, hud, &format!("Übung {index}/8"), 14., GOLD);
                        text(
                            heading,
                            hud,
                            title,
                            if layout.short { 17. } else { 21. },
                            LIGHT,
                        );
                    });
                    let instruction = text(
                        card,
                        hud,
                        step.instruction(action, layout.gamepad),
                        16.,
                        LIGHT,
                    );
                    card.commands()
                        .entity(instruction)
                        .insert(TextLayout::new_with_justify(JustifyText::Left));
                    card.spawn(Node {
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.),
                        ..default()
                    })
                    .with_children(|buttons| {
                        let name = match action {
                            UiAction::QuickWheel(Wheel::Build) => "Baurad öffnen",
                            UiAction::QuickWheel(Wheel::Miracles) => "Wunderrad öffnen",
                            UiAction::QuickWheel(Wheel::Hippo) => "Nilpferdrad öffnen",
                            UiAction::Tool(_) => "Pfote nehmen",
                            UiAction::Guide if waiting => "Zuschauen",
                            UiAction::Guide => "Ziel zeigen",
                            _ => "Ausführen",
                        };
                        let name = if layout.gamepad {
                            format!("A  {name}")
                        } else {
                            name.into()
                        };
                        let style = ButtonStyle {
                            compact: true,
                            height: Some(44.),
                            ..default()
                        };
                        styled_button(buttons, hud, &name, action, false, true, style);
                        if action != UiAction::Guide {
                            styled_button(
                                buttons,
                                hud,
                                "Ziel zeigen",
                                UiAction::Guide,
                                false,
                                false,
                                style,
                            );
                        }
                        if matches!(step, Step::House | Step::Field) && action == UiAction::Confirm
                        {
                            text(
                                buttons,
                                hud,
                                if layout.gamepad {
                                    "Steuerkreuz versetzt"
                                } else {
                                    "Bauplatz antippen"
                                },
                                13.,
                                SUBTLE,
                            );
                        }
                    });
                });
        });
}

fn placement_bar(
    parent: &mut ChildSpawnerCommands,
    hud: &Hud,
    play: &Play,
    layout: Layout,
    biome: Biome,
) {
    let kind = play.building();
    let edge = if layout.short { 8. } else { EDGE };
    let compact = ButtonStyle {
        compact: true,
        height: Some(if layout.short { 37. } else { 45. }),
        ..default()
    };
    parent
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(edge),
            right: Val::Px(edge),
            bottom: Val::Px(edge),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|place| {
            place
                .spawn((
                    HitArea,
                    Node {
                        width: Val::Percent(100.),
                        max_width: Val::Px(610.),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::axes(Val::Px(8.), Val::Px(6.)),
                        row_gap: Val::Px(5.),
                        border: UiRect::all(Val::Px(1.)),
                        ..default()
                    },
                    BackgroundColor(rgb(DARK)),
                    BorderColor(rgb(BRASS)),
                    BorderRadius::all(Val::Px(13.)),
                ))
                .with_children(|toolbar| {
                    feedback_toast(toolbar, hud, Node::default(), true);
                    toolbar
                        .spawn(Node {
                            column_gap: Val::Px(7.),
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|preview| {
                            building_icon(
                                preview,
                                hud,
                                biome,
                                kind,
                                if layout.short { 49. } else { 67. },
                            );
                            preview
                                .spawn(Node {
                                    flex_grow: 1.,
                                    min_width: Val::Px(0.),
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Start,
                                    row_gap: Val::Px(2.),
                                    ..default()
                                })
                                .with_children(|copy| {
                                    text(
                                        copy,
                                        hud,
                                        &format!(
                                            "{} · {} × {} Felder",
                                            kind.name(),
                                            kind.size(),
                                            kind.size()
                                        ),
                                        15.,
                                        LIGHT,
                                    );
                                    text(
                                        copy,
                                        hud,
                                        kind.use_text(),
                                        if layout.short { 12. } else { 14. },
                                        SUBTLE,
                                    );
                                    label(
                                        copy,
                                        hud,
                                        Label::BlueprintStatus,
                                        if layout.short { 12. } else { 14. },
                                        GOLD,
                                    );
                                });
                            if kind == Building::House && layout.mode != Mode::Tutorial {
                                styled_button(
                                    preview,
                                    hud,
                                    "Versorgung",
                                    UiAction::CheckSite,
                                    false,
                                    false,
                                    compact,
                                );
                            }
                            styled_button(
                                preview,
                                hud,
                                "Bauen",
                                UiAction::Confirm,
                                false,
                                true,
                                ButtonStyle {
                                    icon: layout.gamepad.then_some(PAD_CROSS),
                                    ..compact
                                },
                            );
                        });
                    toolbar
                        .spawn(Node {
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(5.),
                            ..default()
                        })
                        .with_children(|actions| {
                            actions
                                .spawn(Node {
                                    column_gap: Val::Px(4.),
                                    ..default()
                                })
                                .with_children(|tools| {
                                    styled_button(
                                        tools,
                                        hud,
                                        if layout.gamepad { "X Drehen" } else { "Drehen" },
                                        UiAction::Rotate,
                                        false,
                                        false,
                                        ButtonStyle {
                                            icon: Some(29),
                                            ..compact
                                        },
                                    );
                                    styled_button(
                                        tools,
                                        hud,
                                        if layout.gamepad && layout.short {
                                            "RB/R1 Platz"
                                        } else if layout.gamepad {
                                            "RB/R1 Freier Platz"
                                        } else {
                                            "Freier Platz"
                                        },
                                        UiAction::ToolTarget,
                                        false,
                                        false,
                                        ButtonStyle {
                                            icon: Some(14),
                                            ..compact
                                        },
                                    );
                                    styled_button(
                                        tools,
                                        hud,
                                        if layout.gamepad && layout.short {
                                            "B Abbruch"
                                        } else if layout.gamepad {
                                            "B/Kreis Abbrechen"
                                        } else {
                                            "Abbrechen"
                                        },
                                        UiAction::Cancel,
                                        false,
                                        false,
                                        compact,
                                    );
                                });
                            text(
                                actions,
                                hud,
                                if layout.gamepad && layout.short {
                                    "Kreuz versetzt\nL3 zum Ziel"
                                } else if layout.gamepad {
                                    "Steuerkreuz versetzt · L3 zum Ziel"
                                } else if cfg!(any(target_os = "ios", target_os = "android")) {
                                    "Tippe auf den Bauplatz"
                                } else {
                                    "Maus setzt Vorschau · E baut"
                                },
                                12.,
                                LIGHT,
                            );
                        });
                });
        });
}

fn bottom(parent: &mut ChildSpawnerCommands, hud: &Hud, play: &Play, layout: Layout) {
    let edge = if layout.short { 8. } else { EDGE };
    let quick = ButtonStyle {
        compact: true,
        ..default()
    };
    parent
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(edge),
            left: Val::Px(edge),
            right: Val::Px(edge),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(5.),
            ..default()
        })
        .with_children(|hud_root| {
            feedback_toast(
                hud_root,
                hud,
                Node {
                    max_width: Val::Percent(96.),
                    ..default()
                },
                true,
            );
            hud_root
                .spawn((
                    HitArea,
                    Node {
                        width: Val::Percent(100.),
                        max_width: Val::Px(552.),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        column_gap: Val::Px(5.),
                        padding: UiRect::axes(Val::Px(6.), Val::Px(3.)),
                        border: UiRect::all(Val::Px(1.)),
                        ..default()
                    },
                    BackgroundColor(rgb(DARK)),
                    BorderColor(rgb(BRASS)),
                    BorderRadius::all(Val::Px(12.)),
                ))
                .with_children(|command| {
                    command
                        .spawn(Node {
                            flex_grow: 1.,
                            min_width: Val::Px(0.),
                            ..default()
                        })
                        .with_children(|detail| label(detail, hud, Label::Detail, 14., LIGHT));
                    if layout.can_inspect {
                        styled_button(
                            command,
                            hud,
                            "Details",
                            UiAction::Inspect,
                            false,
                            false,
                            quick,
                        );
                    }
                    if play.road_anchor.is_some() {
                        styled_button(
                            command,
                            hud,
                            "Abbruch",
                            UiAction::Cancel,
                            false,
                            false,
                            quick,
                        );
                    }
                    if !layout.gamepad && layout.compact {
                        styled_button(
                            command,
                            hud,
                            "Aktionen",
                            UiAction::ToggleWheel,
                            false,
                            false,
                            ButtonStyle {
                                icon: Some(12),
                                ..quick
                            },
                        );
                    } else if !layout.gamepad {
                        styled_button(
                            command,
                            hud,
                            "Zum Ziel",
                            UiAction::Guide,
                            false,
                            false,
                            quick,
                        );
                    }
                    styled_button(
                        command,
                        hud,
                        "Ausführen",
                        UiAction::Confirm,
                        false,
                        true,
                        ButtonStyle {
                            icon: layout.gamepad.then_some(PAD_CROSS),
                            ..quick
                        },
                    );
                });

            if !layout.gamepad && !layout.compact {
                let mut dock = column_grid(Tool::ALL.len() as u16);
                dock.max_width = Val::Px(512.);
                dock.padding = UiRect::all(Val::Px(4.));
                dock.border = UiRect::all(Val::Px(1.));
                dock.column_gap = Val::Px(4.);
                hud_root
                    .spawn((
                        HitArea,
                        dock,
                        BackgroundColor(rgb(DARK)),
                        BorderColor(rgb(BRASS)),
                        BorderRadius::all(Val::Px(13.)),
                    ))
                    .with_children(|tools| {
                        for tool in Tool::ALL {
                            styled_button(
                                tools,
                                hud,
                                tool.name(),
                                UiAction::Tool(tool),
                                play.tool == tool,
                                false,
                                quick,
                            );
                        }
                    });
            }
            if layout.gamepad {
                gamepad_legend(
                    hud_root,
                    hud,
                    layout,
                    &[
                        ("LB/L1", None, "Auswahlrad"),
                        ("X", Some(PAD_SQUARE), "Bauen"),
                        ("Y", Some(PAD_TRIANGLE), "Wunder"),
                        (
                            "RB/R1",
                            None,
                            if play.tool == Tool::Hand {
                                "Nilpferd"
                            } else {
                                "Werkzeugziel"
                            },
                        ),
                        ("L3", None, "Ziel"),
                        ("R3", None, "Objekt"),
                    ],
                );
            }
        });
}

fn gamepad_legend(
    parent: &mut ChildSpawnerCommands,
    hud: &Hud,
    layout: Layout,
    bindings: &[(&str, Option<usize>, &str)],
) {
    let spacious = !layout.short && !layout.compact;
    parent
        .spawn(Node {
            width: Val::Percent(100.),
            display: Display::Grid,
            grid_template_columns: vec![
                GridTrack::auto();
                if layout.narrow_legend { 3 } else { 6 }
            ],
            justify_items: JustifyItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(4.),
            row_gap: Val::Px(3.),
            ..default()
        })
        .with_children(|legend| {
            for &(binding, alternate, action) in bindings {
                legend
                    .spawn((
                        Node {
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(4.),
                            padding: UiRect::axes(Val::Px(5.), Val::Px(2.)),
                            ..default()
                        },
                        BackgroundColor(rgb(DARK)),
                        BorderRadius::all(Val::Px(5.)),
                    ))
                    .with_children(|item| {
                        text(item, hud, binding, if spacious { 16. } else { 12. }, GOLD);
                        if let Some(index) = alternate {
                            glyph(item, hud, index, if spacious { 18. } else { 13. });
                        }
                        text(item, hud, action, if spacious { 15. } else { 12. }, LIGHT);
                    });
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

fn modal_header(
    parent: &mut ChildSpawnerCommands,
    hud: &Hud,
    title: &str,
    short: bool,
    actions: &[(&str, UiAction, bool)],
) {
    let style = ButtonStyle {
        compact: short,
        height: Some(if short { 34. } else { 44. }),
        ..default()
    };
    parent
        .spawn(Node {
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.),
            ..default()
        })
        .with_children(|nav| {
            if title.is_empty() {
                nav.spawn(Node {
                    flex_grow: 1.,
                    ..default()
                });
            } else {
                panel_heading(nav, hud, title, short);
            }
            nav.spawn(Node {
                column_gap: Val::Px(5.),
                ..default()
            })
            .with_children(|actions_row| {
                for &(name, action, primary) in actions {
                    styled_button(actions_row, hud, name, action, false, primary, style);
                }
            });
        });
}

fn island_heading(
    parent: &mut ChildSpawnerCommands,
    hud: &Hud,
    biome: Biome,
    title: &str,
    line: &str,
    short: bool,
) {
    let height = if short { 59. } else { 88. };
    parent
        .spawn((
            Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.),
                padding: UiRect::all(Val::Px(5.)),
                ..default()
            },
            BackgroundColor(rgb(DARK)),
            BorderRadius::all(Val::Px(12.)),
        ))
        .with_children(|banner| {
            banner.spawn((
                ImageNode::from_atlas_image(
                    hud.world_art.clone(),
                    TextureAtlas {
                        layout: hud.world_cards.clone(),
                        index: biome as usize,
                    },
                ),
                Node {
                    width: Val::Px(height * 1.19),
                    height: Val::Px(height),
                    flex_shrink: 0.,
                    ..default()
                },
                BorderRadius::all(Val::Px(8.)),
            ));
            banner
                .spawn(Node {
                    min_width: Val::Px(0.),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    ..default()
                })
                .with_children(|copy| {
                    for (value, size, color) in [
                        (line, if short { 13. } else { 15. }, GOLD),
                        (title, if short { 22. } else { 30. }, LIGHT),
                    ] {
                        let e = text(copy, hud, value, size, color);
                        copy.commands()
                            .entity(e)
                            .insert(TextLayout::new_with_justify(JustifyText::Left));
                    }
                });
        });
}

fn main_menu(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, short: bool) {
    island_heading(
        parent,
        hud,
        game.region().kind,
        "Hippogrove",
        "Dein Dorf, dein Nilpferd und deine Wunder",
        short,
    );
    let style = ButtonStyle {
        compact: short,
        ..default()
    };
    if let Some(mode) = game.continue_mode() {
        styled_button(
            parent,
            hud,
            "Spiel fortsetzen",
            UiAction::SelectMode(mode),
            false,
            true,
            ButtonStyle {
                caption: Some(Label::ResumeMode),
                ..style
            },
        );
    }
    let locked = game.practice_required();
    let mut grid = column_grid(if locked || short { 2 } else { 4 });
    grid.row_gap = Val::Px(6.);
    parent.spawn(grid).with_children(|choices| {
        styled_button(
            choices,
            hud,
            "Tutorial",
            UiAction::SelectMode(Mode::Tutorial),
            false,
            locked,
            ButtonStyle {
                caption: Some(Label::PracticeStatus),
                ..style
            },
        );
        if !locked {
            styled_button(
                choices,
                hud,
                "Kampagnen",
                UiAction::ModeTab(false),
                false,
                false,
                style,
            );
            styled_button(
                choices,
                hud,
                "Freies Bauen",
                UiAction::ModeTab(true),
                false,
                false,
                style,
            );
        }
        styled_button(
            choices,
            hud,
            "Anzeige",
            UiAction::Panel(Panel::Display),
            false,
            false,
            ButtonStyle {
                icon: Some(10),
                ..style
            },
        );
        styled_button(
            choices,
            hud,
            "Ton",
            UiAction::Panel(Panel::Sound),
            false,
            false,
            ButtonStyle {
                icon: Some(28),
                ..style
            },
        );
    });
    if locked {
        text(
            parent,
            hud,
            "Begleite dein Nilpferd auf der Übungsinsel. Danach beginnt deine Kampagne.",
            15.,
            MUTED,
        );
    }
}

fn menu_panel(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, layout: Layout) {
    let style = ButtonStyle {
        compact: true,
        height: Some(if layout.short { 35. } else { 46. }),
        ..default()
    };
    parent.spawn(column_grid(3)).with_children(|tabs| {
        for (i, name) in ["Insel", "Dorf & Kreatur", "Spiel"].into_iter().enumerate() {
            styled_button(
                tabs,
                hud,
                name,
                UiAction::MenuPage(i as u8),
                layout.menu_page == i as u8,
                false,
                style,
            );
        }
    });
    if layout.gamepad && !layout.short {
        text(
            parent,
            hud,
            "LB/L1 und RB/R1 wechseln die Menüseite.",
            15.,
            MUTED,
        );
    }
    if layout.menu_page == 2 && !game.region().tutorial {
        styled_button(
            parent,
            hud,
            if layout.folded {
                "Questtracker ausklappen"
            } else {
                "Questtracker einklappen"
            },
            UiAction::FoldTracker,
            false,
            false,
            style,
        );
    }
    if layout.menu_page == 2 {
        label(parent, hud, Label::Calendar, 14., MUTED);
    }
    let items: &[(&str, Panel, usize)] = match layout.menu_page {
        1 => &[
            ("Inselwerk", Panel::Work, 12),
            ("Stadtplanung", Panel::Town(0), 6),
            ("Gebäude", Panel::Catalog, 6),
            ("Forschung", Panel::Research, 13),
            ("Vorräte", Panel::Inventory, 12),
            ("Kreatur & Talente", Panel::Progress, 8),
            ("Dorfruf", Panel::Gathering, 3),
            ("Wunder", Panel::Spells, 9),
        ],
        2 => &[
            ("Anzeige", Panel::Display, 10),
            ("Steuerung", Panel::Controls, 14),
            ("Ton", Panel::Sound, 28),
            ("Spielmodi", Panel::Modes, 10),
        ],
        _ => &[
            ("Inselwerk", Panel::Work, 12),
            ("Stadtplanung", Panel::Town(0), 6),
            ("Nachbardörfer", Panel::Council, 7),
            ("Landkarte", Panel::Map, 10),
            ("Aufträge", Panel::Quests, 11),
            ("Reisebuch", Panel::Journey, 14),
            ("Dorfchronik", Panel::VillageLog, 12),
        ],
    };
    let mut grid = column_grid(if layout.short { 3 } else { 2 });
    grid.row_gap = Val::Px(5.);
    parent.spawn(grid).with_children(|links| {
        for &(name, panel, icon) in items {
            if game.region().tutorial
                && matches!(
                    panel,
                    Panel::Council | Panel::Gathering | Panel::VillageLog | Panel::Work
                )
            {
                continue;
            }
            if game.region().sandbox && panel == Panel::Journey {
                continue;
            }
            styled_button(
                links,
                hud,
                name,
                UiAction::Panel(panel),
                false,
                false,
                ButtonStyle {
                    icon: Some(icon),
                    ..style
                },
            );
        }
    });
}

fn sound_panel(parent: &mut ChildSpawnerCommands, hud: &Hud, play: &Play, short: bool) {
    parent.spawn(column_grid(3)).with_children(|controls| {
        for bus in Bus::ALL {
            let enabled = play.mix.enabled(bus);
            styled_button(
                controls,
                hud,
                &format!("{} · {}", bus.name(), if enabled { "An" } else { "Aus" }),
                UiAction::Sound(bus),
                enabled,
                false,
                ButtonStyle {
                    compact: short,
                    ..default()
                },
            );
        }
    });
    let style = ButtonStyle {
        compact: short,
        ..default()
    };
    parent.spawn(column_grid(2)).with_children(|actions| {
        styled_button(
            actions,
            hud,
            "Ton testen",
            UiAction::TestSound,
            false,
            true,
            style,
        );
        styled_button(
            actions,
            hud,
            "Alles einschalten",
            UiAction::EnableSound,
            false,
            false,
            style,
        );
    });
    label(parent, hud, Label::AudioStatus, 15., MUTED);
}

fn controls_panel(parent: &mut ChildSpawnerCommands, hud: &Hud, gamepad: bool, short: bool) {
    let bindings: &[(&str, &str)] = if gamepad {
        &[
            ("Linker Stick · Rechter Stick", "Zielen · Kamera"),
            ("LT/RT · L2/R2", "Raus- und reinzoomen"),
            (
                "LB/L1 · X / Quadrat · Y / Dreieck",
                "Alle Aktionen · Bauen · Wunder",
            ),
            ("RB/R1", "Nilpferd oder Werkzeugziel"),
            ("L3 · R3", "Zum Auftragsziel · Objektaktionen"),
            ("Steuerkreuz", "Menüs · ↑ Auftrag · Bauplatz versetzen"),
            ("A / Kreuz · B / Kreis", "Bestätigen · Zurück"),
            ("Start", "Pause und Hauptmenü"),
        ]
    } else if cfg!(any(target_os = "android", target_os = "ios")) {
        &[
            ("Objekt antippen", "Aktionen direkt am Objekt öffnen"),
            ("Ziehen", "Kamera schwenken"),
            ("Zwei Finger · + / −", "Zoomen"),
            ("Quelle gedrückt halten", "Bis zu 18 Einheiten aufnehmen"),
        ]
    } else {
        &[
            ("Maus · WASD", "Ziel wählen und Kamera schwenken"),
            ("Mausrad · + / −", "Zoomen"),
            ("Leertaste", "Auswahlrad öffnen"),
            ("G", "Zum aktuellen Auftrag"),
            ("E", "Aktion ausführen · halten zum Greifen"),
            ("F", "Hauptmenü öffnen"),
        ]
    };
    let size = if short { 14. } else { 17. };
    for &(key, effect) in bindings {
        parent
            .spawn(Node {
                justify_content: JustifyContent::SpaceBetween,
                column_gap: Val::Px(12.),
                ..default()
            })
            .with_children(|row| {
                text(row, hud, key, size, INK);
                text(row, hud, effect, size, MUTED);
            });
    }
}

fn orders_panel(
    parent: &mut ChildSpawnerCommands,
    hud: &Hud,
    play: &Play,
    tool: Tool,
    short: bool,
) {
    let choices: &[usize] = match tool {
        Tool::Hippo => &[6, 0, 1, 2],
        Tool::Helpers => &[0, 1, 2],
        _ => &[0, 1],
    };
    if tool == Tool::Hippo {
        label(parent, hud, Label::Creature, 16., DARK);
    } else if tool == Tool::Helpers {
        label(parent, hud, Label::Helpers, 16., DARK);
    }
    parent
        .spawn(column_grid(choices.len() as u16))
        .with_children(|commands| {
            for &index in choices {
                styled_button(
                    commands,
                    hud,
                    tool.choice_name(index),
                    UiAction::Order(tool, index),
                    play.tool == tool && play.option == index,
                    false,
                    ButtonStyle {
                        compact: short,
                        caption: Some(Label::Order(tool, index)),
                        ..default()
                    },
                );
            }
        });
    if tool == Tool::Hippo {
        parent.spawn(column_grid(4)).with_children(|care| {
            for (name, action) in [
                ("Loben", Action::Praise),
                ("Snack", Action::Feed),
                ("Pause", Action::Rest),
                ("Streicheln", Action::Pet),
            ] {
                styled_button(
                    care,
                    hud,
                    name,
                    UiAction::Do(action),
                    false,
                    false,
                    ButtonStyle {
                        compact: short,
                        ..default()
                    },
                );
            }
        });
    } else if tool == Tool::Helpers {
        styled_button(
            parent,
            hud,
            "Sammler zurückrufen",
            UiAction::Do(Action::RecallHelpers),
            false,
            false,
            ButtonStyle {
                compact: short,
                ..default()
            },
        );
    }
}

fn spellbook(parent: &mut ChildSpawnerCommands, hud: &Hud, play: &Play, short: bool) {
    parent.spawn(row()).with_children(|power| {
        glyph(power, hud, 4, 21.);
        label(power, hud, Label::Mana, 16., DARK);
    });
    let mut grid = column_grid(3);
    grid.row_gap = Val::Px(6.);
    parent.spawn(grid).with_children(|spells| {
        for (index, icon) in [(0, 4), (3, 19), (4, 8), (2, 11), (1, 15), (5, 9)] {
            let spell = Miracle::ALL[index];
            styled_button(
                spells,
                hud,
                spell.name(),
                UiAction::Order(Tool::Miracles, index),
                play.tool == Tool::Miracles && play.option == index,
                false,
                ButtonStyle {
                    compact: short,
                    icon: (!short).then_some(icon),
                    caption: Some(Label::Spell(spell)),
                    ..default()
                },
            );
        }
    });
}

fn map_chart(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, short: bool, atlas: bool) {
    let region = game.region();
    let width = region.bounds.width();
    let tiles = Vec2::new(width as f32, (region.bounds.len() / width) as f32);
    let height = if short {
        100.
    } else if atlas {
        150.
    } else {
        208.
    };
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
            ImageNode::new(
                hud.map
                    .as_ref()
                    .expect("The map is generated when its panel opens")
                    .1
                    .clone(),
            ),
            BorderRadius::all(Val::Px(12.)),
        ))
        .with_children(|map| {
            let towns = region
                .villages
                .iter()
                .zip(&game.city().contest.towns)
                .map(|(v, t)| {
                    (
                        v.center,
                        match t.side {
                            crate::game::Allegiance::Neutral => PAPER,
                            crate::game::Allegiance::Ours => 0x53ad73,
                            crate::game::Allegiance::Rival => 0xbc6292,
                        },
                    )
                });
            let camps = region
                .areas
                .iter()
                .zip(game.city().camps)
                .map(|(area, built)| (area.center, if built { 0xfbe39e } else { 0xb67648 }));
            let markers = std::iter::once((region.start, DARK))
                .chain(towns)
                .chain(
                    (!region.tutorial && !region.sandbox)
                        .then_some((region.rival_home(), 0x893661)),
                )
                .chain(camps)
                .chain(
                    game.city()
                        .contest
                        .destination
                        .and(game.city().contest.rival_tile)
                        .map(|tile| (tile, 0xe55cb1)),
                );
            for (tile, color) in markers {
                let offset = tile.local() - region.bounds.0.local() + Vec2::splat(0.5);
                let p = (size * offset / tiles - Vec2::splat(dot * 0.5))
                    .clamp(Vec2::ZERO, size - Vec2::splat(dot));
                let mut pin = map.spawn((
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
                if !atlas && let Some(i) = region.village_at(tile) {
                    pin.insert((
                        Button,
                        HitArea,
                        ControlButton {
                            action: UiAction::Panel(Panel::Village(i)),
                            selected: false,
                            strong: false,
                        },
                    ));
                }
            }
            if atlas {
                return;
            }
            let (symbol, color) = match game.city().side_quest(region) {
                SideQuest::Available => ("!", GOLD),
                SideQuest::Ready => ("?", JADE),
                _ => return,
            };
            let origin = region.start.local() - region.bounds.0.local() + Vec2::splat(0.5);
            let marker = (size * origin / tiles + Vec2::new(4., -28.))
                .clamp(Vec2::ZERO, size - Vec2::splat(32.));
            let entity = styled_button(
                map,
                hud,
                symbol,
                UiAction::Panel(Panel::SideQuest),
                color == JADE,
                color == GOLD,
                ButtonStyle {
                    compact: true,
                    ..default()
                },
            );
            map.commands().entity(entity).insert(Node {
                position_type: PositionType::Absolute,
                left: Val::Px(marker.x),
                top: Val::Px(marker.y),
                width: Val::Px(32.),
                height: Val::Px(32.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(1.)),
                ..default()
            });
        });
}

fn gathering_panel(
    parent: &mut ChildSpawnerCommands,
    hud: &Hud,
    game: &Game,
    play: &Play,
    layout: Layout,
) {
    use crate::game::Rite;
    let city = game.city();
    let region = game.region();
    let compact = ButtonStyle {
        compact: true,
        height: Some(if layout.short { 34. } else { 45. }),
        ..default()
    };
    if let Some(c) = &city.gatherings.circle {
        if layout.short {
            panel_heading(parent, hud, c.kind.name(), true);
        } else {
            island_heading(parent, hud, region.kind, c.kind.name(), "Dorfruf", false);
        }
        let (present, _) = c.present(city);
        text(
            parent,
            hud,
            &format!("{present} von {} am Platz · 6 Wunderkraft", c.guests.len()),
            16.,
            INK,
        );
        let detail=match c.kind {
            Rite::Timber | Rite::Stone => format!("Je Person bis zu 3 {} aus der markierten Quelle.", region.kind.material(c.kind.material().unwrap())),
            Rite::Harvest => "Das Nilpferd lässt die Saat reifen. Jeder anwesende Dorfbewohner bringt ein Feld im Umkreis ein.".into(),
            Rite::Council if region.district_at(c.center).is_some() => "Drei Stimmen gewinnen das Vertrauen der Nachbarn. Mit der Dorfgabe kannst du das Bündnis besiegeln.".into(),
            Rite::Council => "Frieden bringt Ansehen und schützt das Dorf. Am umkämpften Übergang kann ein Machtruf die Wachen vertreiben.".into(),
        };
        text(parent, hud, &detail, 15., MUTED);
        parent.spawn(row()).with_children(|actions| {
            styled_button(
                actions,
                hud,
                "Zum Kreis",
                UiAction::Focus(c.center),
                false,
                !c.ready(city),
                compact,
            );
            if c.ready(city) {
                styled_button(
                    actions,
                    hud,
                    c.kind.finish(),
                    UiAction::Do(Action::Invoke(false)),
                    false,
                    true,
                    compact,
                );
                if c.kind == Rite::Council && city.council_threat(region, c.center) {
                    styled_button(
                        actions,
                        hud,
                        "Machtruf · −4 Stimmung",
                        UiAction::Do(Action::Invoke(true)),
                        false,
                        false,
                        compact,
                    );
                }
            }
            styled_button(
                actions,
                hud,
                "Ruf auflösen",
                UiAction::Do(Action::DismissCircle),
                false,
                false,
                compact,
            );
        });
    } else {
        if layout.short {
            panel_heading(parent, hud, "Ruf dein Dorf zusammen", true);
        } else {
            island_heading(
                parent,
                hud,
                region.kind,
                "Ruf dein Dorf zusammen",
                &region.name,
                false,
            );
        }
        if city.gatherings.rest > 0. {
            text(
                parent,
                hud,
                &format!(
                    "Dein Dorf erholt sich. In {:.0} Spielsekunden kannst du erneut rufen.",
                    city.gatherings.rest.ceil()
                ),
                16.,
                INK,
            );
            button(parent, hud, "Weiterspielen", UiAction::Resume, false, true);
            return;
        }
        text(
            parent,
            hud,
            "Wirke ab zwei Helfenden für 6 Wunderkraft. Mehr Helfende verstärken den Ruf.",
            15.,
            MUTED,
        );
        parent.spawn(column_grid(2)).with_children(|grid| {
            for kind in Rite::ALL {
                let place = city.gathering_place(region, kind, play.aim);
                let hint = place.err().unwrap_or_else(|| match kind {
                    Rite::Harvest => "Bis zu drei Felder auf einmal ernten".into(),
                    Rite::Timber | Rite::Stone => format!(
                        "Bis zu 12 {} bergen",
                        region.kind.material(kind.material().unwrap())
                    ),
                    Rite::Council if city.quest == Quest::Districts => {
                        "Ein Nachbardorf gewinnen oder einschüchtern".into()
                    }
                    Rite::Council
                        if city.quest == Quest::Bridge && city.count(Building::Shrine) > 0 =>
                    {
                        "Den Übergang schützen oder die Wachen vertreiben".into()
                    }
                    Rite::Council => "Schutz und Ansehen für dein Dorf".into(),
                });
                grid.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    min_width: Val::Px(0.),
                    ..default()
                })
                .with_children(|card| {
                    styled_button(
                        card,
                        hud,
                        kind.name(),
                        UiAction::Do(Action::Call(kind, play.aim)),
                        false,
                        false,
                        ButtonStyle {
                            icon: Some(kind.icon()),
                            ..compact
                        },
                    );
                    text(
                        card,
                        hud,
                        &hint,
                        if layout.short { 12. } else { 14. },
                        MUTED,
                    );
                });
            }
        });
    }
}

fn village_log(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, short: bool) {
    let city = game.city();
    island_heading(
        parent,
        hud,
        game.region().kind,
        "Dorfchronik",
        &game.region().name,
        short,
    );
    if city.journal.is_empty() {
        text(
            parent,
            hud,
            "Hier hält das Dorf seine Lieferungen, Bauwerke und Entscheidungen fest.",
            16.,
            MUTED,
        );
    }
    for (i, line) in city.journal.iter().enumerate() {
        text(
            parent,
            hud,
            line,
            if short { 14. } else { 17. },
            if i == 0 { INK } else { MUTED },
        );
    }
}

fn departure_panel(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, short: bool) {
    let region = game.region();
    let city = game.city();
    let next = game.campaign.active + 1;
    let complete = next == game.regions.len();
    island_heading(
        parent,
        hud,
        region.kind,
        if complete {
            "Kampagne vollendet"
        } else {
            "Insel gesichert"
        },
        &region.name,
        short,
    );
    if city.contest.pact {
        text(
            parent,
            hud,
            &format!(
                "{} von {} Dörfern haben den Inselbund getragen.",
                city.contest.count(crate::game::Allegiance::Ours),
                region.villages.len()
            ),
            17.,
            INK,
        );
    }
    for (district, state) in region
        .districts
        .iter()
        .zip(&city.districts)
        .filter(|_| !city.contest.pact)
    {
        let (icon, result) = if state.outcome == Some(Outcome::Alliance) {
            (2, "Bündnis · Tauschkurs 4 zu 8")
        } else {
            (1, "Wachen vertrieben · Handel oder Tribut")
        };
        parent.spawn(row()).with_children(|line| {
            glyph(line, hud, icon, 18.);
            text(
                line,
                hud,
                &format!("{} · {result}", district.name),
                16.,
                INK,
            );
        });
    }
    if let Some(destination) = game.regions.get(next) {
        if game.campaign.settlements[next].is_none() {
            text(
                parent,
                hud,
                &format!(
                    "Für die Überfahrt schicken deine Nachbarn {}.",
                    stock_text(city.departure_supplies())
                ),
                16.,
                MUTED,
            );
        }
        button(
            parent,
            hud,
            &format!("Weiter nach {}", destination.name),
            UiAction::Do(Action::Travel(next)),
            false,
            true,
        );
    } else {
        button(
            parent,
            hud,
            "Kampagnen ansehen",
            UiAction::Panel(Panel::Modes),
            false,
            false,
        );
    }
    button(
        parent,
        hud,
        "Auf dieser Insel bleiben",
        UiAction::Resume,
        false,
        complete,
    );
}

fn quest_journal(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, layout: Layout) {
    let city = game.city();
    let region = game.region();
    island_heading(
        parent,
        hud,
        region.kind,
        city.quest_title(region),
        &format!("{} · Hauptauftrag", region.name),
        layout.short,
    );
    quests_ui::journal_goals(parent, hud, game, layout.short);
    label(
        parent,
        hud,
        Label::Objective,
        if layout.short { 14. } else { 17. },
        INK,
    );
    if city.quest == Quest::Shelter && !region.tutorial {
        parent
            .spawn(Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(9.),
                ..default()
            })
            .with_children(|items| {
                for (icon, name, done) in [
                    (
                        0,
                        "Lieferung",
                        city.hand_deliveries > 0 && !city.waiting_for_first_delivery(),
                    ),
                    (6, "Hütte", city.count(Building::House) > 0),
                    (12, "Lager", city.count(Building::Store) > 0),
                    (8, "Nilpferd", city.hippo_deliveries > 0),
                ] {
                    items
                        .spawn((
                            Node {
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(3.),
                                padding: UiRect::all(Val::Px(3.)),
                                ..default()
                            },
                            BackgroundColor(rgb(if done { JADE } else { SUBTLE })),
                            BorderRadius::all(Val::Px(5.)),
                        ))
                        .with_children(|item| {
                            glyph(item, hud, icon, 15.);
                            text(item, hud, &format!("{name} {}/1", u8::from(done)), 13., INK);
                        });
                }
            });
    }
    let compact = ButtonStyle {
        compact: true,
        height: Some(44.),
        ..default()
    };
    parent.spawn(row()).with_children(|actions| {
        if city.quest == Quest::Bridge && city.count(Building::Shrine) > 0 {
            styled_button(
                actions,
                hud,
                "Wachen anvisieren",
                UiAction::Confront,
                false,
                false,
                compact,
            );
        }
        styled_button(
            actions,
            hud,
            if city.quest == Quest::Bridge {
                "Bündnis vorbereiten"
            } else {
                "Zum Auftragsziel"
            },
            UiAction::Guide,
            false,
            true,
            ButtonStyle {
                icon: Some(14),
                ..compact
            },
        );
        if !region.tutorial {
            styled_button(
                actions,
                hud,
                if city.work.pinned {
                    "Mein Inselwerk"
                } else {
                    "Inselwerk"
                },
                UiAction::Panel(Panel::Work),
                false,
                false,
                ButtonStyle {
                    icon: Some(12),
                    ..compact
                },
            );
        }
        if !region.tutorial {
            styled_button(
                actions,
                hud,
                "Dorfchronik",
                UiAction::Panel(Panel::VillageLog),
                false,
                false,
                compact,
            );
        }
        if !region.tutorial && (layout.side_quest != SideQuest::Complete || city.local_finished) {
            let name = match layout.side_quest {
                SideQuest::Available => "Bitte aus dem Dorf",
                SideQuest::Ready => "Belohnung abholen",
                SideQuest::Active => "Nebenauftrag",
                SideQuest::Complete => "Erfüllte Bitte",
            };
            styled_button(
                actions,
                hud,
                name,
                UiAction::Panel(Panel::SideQuest),
                false,
                false,
                ButtonStyle {
                    icon: Some(11),
                    ..compact
                },
            );
        }
    });
    if !layout.short {
        if let Some(choice) = city.opening_choice() {
            text(
                parent,
                hud,
                match choice {
                    FirstDelivery::Village => "Miras Familie hat ein Dach am Schrein.",
                    FirstDelivery::Farm => "Joras Hof ist wieder gedeckt.",
                },
                15.,
                MUTED,
            );
        }
        label(parent, hud, Label::Raid, 15., MUTED);
        if !region.tutorial {
            label(parent, hud, Label::Rival, 16., MUTED);
        }
    }
}

fn side_quest_panel(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, layout: Layout) {
    let (title, _, _) = game.city().local_quest(game.region());
    let status = match layout.side_quest {
        SideQuest::Available => "Neue Bitte",
        SideQuest::Active => "Angenommen",
        SideQuest::Ready => "Belohnung bereit",
        SideQuest::Complete => "Erfüllt",
    };
    island_heading(parent, hud, game.region().kind, title, status, layout.short);
    label(
        parent,
        hud,
        Label::LocalQuest,
        if layout.short { 14. } else { 18. },
        INK,
    );
    if let Some((name, action)) = match layout.side_quest {
        SideQuest::Available => Some(("Auftrag annehmen", Action::AcceptLocal)),
        SideQuest::Ready => Some(("Belohnung abholen", Action::FinishLocal)),
        _ => None,
    } {
        text(
            parent,
            hud,
            "6 Glauben · 2 Erkenntnis · 1 Talentpunkt",
            15.,
            MUTED,
        );
        button(parent, hud, name, UiAction::Do(action), false, true);
    } else {
        if layout.side_quest == SideQuest::Active {
            button(parent, hud, "Zum Ziel", UiAction::Guide, false, true);
        }
        if game.city().next_raid(game.region()).is_some() {
            label(parent, hud, Label::Raid, 15., MUTED);
        }
    }
}

fn map_panel(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, layout: Layout) {
    let region = game.region();
    if region.tutorial {
        map_chart(parent, hud, game, layout.short, false);
        parent.spawn(row()).with_children(|nav| {
            button(
                nav,
                hud,
                "Zum Startschrein",
                UiAction::Focus(region.start),
                false,
                true,
            );
            button(
                nav,
                hud,
                "Ganze Insel ansehen",
                UiAction::Landscape,
                false,
                false,
            );
        });
        return;
    }
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
            map_chart(chart, hud, game, layout.short, false);
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
            destinations
                .spawn(column_grid(
                    if region.has_rival_creature() && !layout.short {
                        3
                    } else {
                        2
                    },
                ))
                .with_children(|links| {
                    styled_button(
                        links,
                        hud,
                        "Startdorf",
                        UiAction::Focus(region.start),
                        false,
                        false,
                        compact,
                    );
                    if region.has_rival_creature() && !layout.short {
                        styled_button(
                            links,
                            hud,
                            "Rivale",
                            UiAction::Focus(region.rival_lair()),
                            false,
                            false,
                            compact,
                        );
                    }
                    styled_button(
                        links,
                        hud,
                        if layout.short || region.has_rival_creature() {
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
    let style = ButtonStyle {
        compact: short,
        height: short.then_some(35.),
        ..default()
    };
    let biome = game.region().kind;
    let entries = play.catalog_entries(game);
    let pages = entries.len().div_ceil(CATALOG_PAGE_SIZE);
    let index = play.catalog_page;

    parent
        .spawn(Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: Val::Px(5.),
            ..default()
        })
        .with_children(|header| {
            panel_heading(header, hud, "Baubuch", short);
            header.spawn(row()).with_children(|nav| {
                styled_button(
                    nav,
                    hud,
                    "Hauptmenü",
                    UiAction::MainMenu,
                    false,
                    false,
                    style,
                );
                styled_button(nav, hud, "Zurück", UiAction::Cancel, false, false, style);
            });
        });
    parent.spawn(column_grid(5)).with_children(|tabs| {
        for branch in std::iter::once(None).chain(Branch::ALL.into_iter().map(Some)) {
            styled_button(
                tabs,
                hud,
                branch.map_or("Grundbau", Branch::name),
                UiAction::Catalog(branch),
                play.catalog_branch == branch,
                false,
                style,
            );
        }
    });
    parent
        .spawn(Node {
            column_gap: Val::Px(8.),
            ..default()
        })
        .with_children(|body| {
            body.spawn(Node {
                width: Val::Percent(44.),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.),
                ..default()
            })
            .with_children(|cards| {
                for &kind in entries
                    .iter()
                    .skip(index * CATALOG_PAGE_SIZE)
                    .take(CATALOG_PAGE_SIZE)
                {
                    let action = UiAction::PreviewBuilding(kind);
                    cards
                        .spawn((
                            Button,
                            HitArea,
                            ControlButton {
                                action,
                                selected: false,
                                strong: false,
                            },
                            Node {
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(6.),
                                min_height: Val::Px(if short { 43. } else { 63. }),
                                padding: UiRect::axes(Val::Px(5.), Val::Px(2.)),
                                border: UiRect::all(Val::Px(2.)),
                                ..default()
                            },
                            BackgroundColor(rgb(SURFACE)),
                            BorderColor(rgb(BRASS)),
                            Outline::new(Val::Px(2.), Val::Px(1.), Color::NONE),
                            BorderRadius::all(Val::Px(10.)),
                        ))
                        .with_children(|card| {
                            building_icon(card, hud, biome, kind, if short { 35. } else { 51. });
                            card.spawn(Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Start,
                                ..default()
                            })
                            .with_children(|copy| {
                                text(copy, hud, kind.name(), if short { 14. } else { 17. }, LIGHT);
                                text(
                                    copy,
                                    hud,
                                    if game.region().sandbox {
                                        "Freier Bau".into()
                                    } else {
                                        stock_text(kind.price())
                                    }
                                    .as_str(),
                                    if short { 11. } else { 13. },
                                    GOLD,
                                );
                            });
                        });
                }
            });
            body.spawn((
                Node {
                    flex_grow: 1.,
                    flex_basis: Val::Px(0.),
                    min_width: Val::Px(0.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Start,
                    row_gap: Val::Px(if short { 3. } else { 7. }),
                    padding: UiRect::all(Val::Px(if short { 7. } else { 14. })),
                    border: UiRect::all(Val::Px(1.)),
                    ..default()
                },
                BackgroundColor(rgb(0xfcf3de)),
                BorderColor(rgb(0xdac398)),
                BorderRadius::all(Val::Px(12.)),
            ))
            .with_children(|details| {
                details
                    .spawn(Node {
                        column_gap: Val::Px(6.),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|heading| {
                        let portrait = building_icon(
                            heading,
                            hud,
                            biome,
                            play.catalog_preview,
                            if short { 40. } else { 63. },
                        );
                        heading.commands().entity(portrait).insert(CatalogPortrait);
                        label(
                            heading,
                            hud,
                            Label::CatalogName,
                            if short { 17. } else { 23. },
                            INK,
                        );
                    });
                let size = if short { 13. } else { 16. };
                label(details, hud, Label::CatalogPurpose, size, INK);
                label(details, hud, Label::CatalogPrice, size, MUTED);
                label(details, hud, Label::CatalogReadiness, size, 0x705127);
            });
        });
    parent
        .spawn(Node {
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            column_gap: Val::Px(4.),
            ..default()
        })
        .with_children(|bottom| {
            bottom
                .spawn(Node {
                    column_gap: Val::Px(5.),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|pagination| {
                    if index > 0 {
                        styled_button(
                            pagination,
                            hud,
                            if play.gamepad { "L1 ‹" } else { "‹" },
                            UiAction::CatalogPage(index - 1),
                            false,
                            false,
                            style,
                        );
                    }
                    text(
                        pagination,
                        hud,
                        &format!("{}/{}", index + 1, pages.max(1)),
                        14.,
                        MUTED,
                    );
                    if index + 1 < pages {
                        styled_button(
                            pagination,
                            hud,
                            if play.gamepad { "› R1" } else { "›" },
                            UiAction::CatalogPage(index + 1),
                            false,
                            false,
                            style,
                        );
                    }
                });
            bottom.spawn(row()).with_children(|actions| {
                styled_button(
                    actions,
                    hud,
                    "Forschung",
                    UiAction::Panel(Panel::Research),
                    false,
                    false,
                    style,
                );
                styled_button(
                    actions,
                    hud,
                    "Bauplatz wählen",
                    UiAction::CatalogBuild,
                    false,
                    true,
                    style,
                );
            });
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
    let Some(site) = city.building_at(play.aim) else {
        return;
    };
    let kind = site.kind;
    let style = ButtonStyle {
        compact: short,
        height: short.then_some(37.),
        ..default()
    };
    let body_text = if short { 14. } else { 17. };
    let upgrade = kind.upgrade();
    let connected = city.connected(region, site);
    let connection = if connected {
        "Am Wegenetz"
    } else {
        "Ohne Weganschluss"
    };
    modal_header(
        parent,
        hud,
        kind.name(),
        short,
        &[
            ("Hauptmenü", UiAction::MainMenu, false),
            ("Zurück", UiAction::Cancel, false),
        ],
    );
    if kind == Building::House {
        styled_button(
            parent,
            hud,
            "Versorgung & Bürgerrecht",
            UiAction::Panel(Panel::Household(site.tile)),
            false,
            false,
            ButtonStyle {
                compact: true,
                icon: Some(6),
                height: Some(34.),
                ..default()
            },
        );
    }
    parent
        .spawn(Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(9.),
            ..default()
        })
        .with_children(|overview| {
            building_icon(
                overview,
                hud,
                region.kind,
                kind,
                if short { 62. } else { 85. },
            );
            overview
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    row_gap: Val::Px(3.),
                    ..default()
                })
                .with_children(|summary| {
                    let stage = if site.upgraded {
                        "Stufe 2"
                    } else if upgrade.is_some() {
                        "Stufe 1"
                    } else {
                        "Fertig"
                    };
                    text(
                        summary,
                        hud,
                        &format!("{stage} · {connection}"),
                        body_text,
                        DARK,
                    );
                    text(summary, hud, kind.use_text(), body_text, MUTED);
                });
        });
    if let Some(upgrade) = upgrade {
        let status = city.upgrade_plan(region, site.tile);
        parent
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    padding: UiRect::all(Val::Px(8.)),
                    row_gap: Val::Px(3.),
                    ..default()
                },
                BackgroundColor(rgb(0xece3ca)),
                BorderRadius::all(Val::Px(10.)),
            ))
            .with_children(|details| {
                if site.upgraded {
                    text(
                        details,
                        hud,
                        &format!("Ausgebaut · {}", upgrade.benefit),
                        body_text,
                        DARK,
                    );
                } else {
                    text(
                        details,
                        hud,
                        &format!("Stufe 2 · {}", upgrade.benefit),
                        body_text,
                        DARK,
                    );
                    if region.sandbox {
                        text(details, hud, "Freier Ausbau", body_text, MUTED);
                    } else {
                        text(
                            details,
                            hud,
                            &format!("Material  {}", stock_text(upgrade.materials)),
                            body_text,
                            MUTED,
                        );
                        text(
                            details,
                            hud,
                            &format!(
                                "Gläubige {}/{} · Glauben {}/{}",
                                city.inhabitants(),
                                upgrade.believers,
                                city.faith,
                                upgrade.faith
                            ),
                            body_text,
                            MUTED,
                        );
                    }
                    if let Err(reason) = &status {
                        text(details, hud, reason, body_text, 0x874320);
                    }
                }
            });
        if !site.upgraded {
            styled_button(
                parent,
                hud,
                if play.gamepad {
                    "A  Ausbauen"
                } else {
                    "Ausbauen"
                },
                UiAction::Do(Action::Upgrade(site.tile)),
                false,
                true,
                ButtonStyle {
                    icon: play.gamepad.then_some(PAD_CROSS),
                    ..style
                },
            );
        }
    }
    if kind == Building::Civic(Civic::Barracks) {
        styled_button(
            parent,
            hud,
            "Wache ausbilden",
            UiAction::Do(Action::Muster(site.tile)),
            false,
            true,
            style,
        );
    }
}

fn mode_panel(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, layout: Layout) {
    if game.practice_required() {
        modal_header(
            parent,
            hud,
            "Kampagnen",
            layout.short,
            &[("Zurück", UiAction::Cancel, false)],
        );
        text(
            parent,
            hud,
            "Die Kampagnen stehen nach der kurzen Übung bereit.",
            17.,
            INK,
        );
        button(
            parent,
            hud,
            "Tutorial",
            UiAction::SelectMode(Mode::Tutorial),
            false,
            true,
        );
        return;
    }
    let title = if game.mode == Mode::Tutorial && game.campaign_complete() {
        "Übung abgeschlossen"
    } else {
        "Hippogrove"
    };
    modal_header(
        parent,
        hud,
        title,
        layout.short,
        &[("Zurück", UiAction::Cancel, true)],
    );
    let style = ButtonStyle {
        compact: layout.short,
        ..default()
    };
    parent.spawn(column_grid(2)).with_children(|tabs| {
        for (name, sandbox) in [("Kampagnen", false), ("Freies Bauen", true)] {
            styled_button(
                tabs,
                hud,
                name,
                UiAction::ModeTab(sandbox),
                layout.free_build_tab == sandbox,
                false,
                style,
            );
        }
    });
    let mut gallery = column_grid(3);
    gallery.row_gap = Val::Px(6.);
    parent.spawn(gallery).with_children(|cards| {
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
                        min_height: Val::Px(if layout.short { 55. } else { 85. }),
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(if layout.short { 5. } else { 8. }),
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
                            width: Val::Px(if layout.short { 43. } else { 70. }),
                            height: Val::Px(if layout.short { 41. } else { 64. }),
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
                        let name = if layout.short {
                            ["Weidenland", "Fjord", "Jadekanäle", "Oase", "Ägäis"][biome as usize]
                        } else {
                            mode.name()
                        };
                        text(
                            words,
                            hud,
                            name,
                            if layout.short { 13. } else { 15. },
                            foreground,
                        );
                        let detail = if game.has_save(mode) {
                            "Fortsetzen"
                        } else if mode.sandbox() {
                            if layout.short {
                                "Freies Bauen"
                            } else {
                                "Ohne Gegner · Alle Baupläne"
                            }
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
        if !layout.free_build_tab && game.has_save(Mode::Chronicle) {
            styled_button(
                cards,
                hud,
                "Chronik fortsetzen",
                UiAction::SelectMode(Mode::Chronicle),
                game.mode == Mode::Chronicle,
                false,
                style,
            );
        }
    });
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
                                min_height: Val::Px(if layout.short { 48. } else { 98. }),
                                border: UiRect::all(Val::Px(1.)),
                                ..default()
                            },
                            BackgroundColor(rgb(0xfcf3de)),
                            BorderColor(rgb(0xdac398)),
                            BorderRadius::all(Val::Px(12.)),
                        ))
                        .with_children(|card| {
                            glyph(card, hud, image, if layout.short { 32. } else { 52. });
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
    parent.spawn(column_grid(2)).with_children(|resources| {
        for (icon, tag) in [(3, Label::FaithUse), (4, Label::ManaUse)] {
            resources
                .spawn((
                    Node {
                        column_gap: Val::Px(5.),
                        padding: UiRect::all(Val::Px(4.)),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(rgb(0xfcf3de)),
                    BorderRadius::all(Val::Px(9.)),
                ))
                .with_children(|card| {
                    glyph(card, hud, icon, if layout.short { 26. } else { 36. });
                    label(card, hud, tag, if layout.short { 12. } else { 15. }, INK);
                });
        }
    });
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
            column_gap: Val::Px(6.),
            ..default()
        })
        .with_children(|skip| {
            button(skip, hud, "Hauptmenü", UiAction::MainMenu, false, false);
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

fn first_delivery_panel(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, short: bool) {
    let region = game.region();
    let Some(farm) = game.city().opening_farm(region) else {
        return;
    };
    if let Some(bundle) = game.city().held {
        text(
            parent,
            hud,
            &format!(
                "{} Holz in der Pfote · Wähle den Ablageplatz.",
                bundle.count
            ),
            16.,
            INK,
        );
    }
    parent.spawn(column_grid(2)).with_children(|choices| {
        for (choice, name, tile, icon) in [
            (FirstDelivery::Village, "Zu Mira ins Dorf", region.depot, 6),
            (FirstDelivery::Farm, "Zu Joras Uferhof", farm, 0),
        ] {
            styled_button(
                choices,
                hud,
                name,
                UiAction::Focus(tile),
                false,
                false,
                ButtonStyle {
                    compact: short,
                    icon: Some(icon),
                    caption: Some(Label::FirstDelivery(choice)),
                    ..default()
                },
            );
        }
    });
}

fn selection_wheel(
    parent: &mut ChildSpawnerCommands,
    hud: &Hud,
    wheel: Wheel,
    layout: Layout,
    biome: Biome,
) {
    let height = if layout.short { 200. } else { 307. };
    let width = if layout.short { 416. } else { 560. };
    let slot = if layout.short {
        Vec2::new(94., 40.)
    } else {
        Vec2::new(121., 53.)
    };
    parent
        .spawn((Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(7.),
            ..default()
        },))
        .with_children(|wheel_root| {
            text(
                wheel_root,
                hud,
                &layout
                    .lesson
                    .and_then(|(step, _)| step.wheel())
                    .filter(|(expected, _)| *expected == wheel)
                    .and_then(|(_, choice)| {
                        wheel.entries().iter().find(|entry| entry.action == choice)
                    })
                    .map_or_else(
                        || wheel.title().into(),
                        |entry| format!("{} · Wähle {}", wheel.title(), entry.name),
                    ),
                if layout.short { 23. } else { 30. },
                LIGHT,
            );
            wheel_root
                .spawn((
                    HitArea,
                    Node {
                        width: Val::Percent(94.),
                        max_width: Val::Px(width),
                        height: Val::Px(height),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(Val::Px(2.)),
                        ..default()
                    },
                    BackgroundColor(rgb(0x233b30).with_alpha(0.88)),
                    BorderColor(rgb(BRASS)),
                    BorderRadius::all(Val::Percent(50.)),
                ))
                .with_children(|ring| {
                    ring.spawn(Node {
                        max_width: Val::Px(if layout.short { 142. } else { 205. }),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(4.),
                        ..default()
                    })
                    .with_children(|detail| {
                        label(
                            detail,
                            hud,
                            Label::WheelChoice,
                            if layout.short { 16. } else { 21. },
                            GOLD,
                        );
                        label(
                            detail,
                            hud,
                            Label::WheelDetail,
                            if layout.short { 12. } else { 15. },
                            LIGHT,
                        );
                    });
                    let entries = wheel.entries();
                    for (index, entry) in entries.iter().enumerate() {
                        let phase = index as f32 * std::f32::consts::TAU / entries.len() as f32;
                        ring.spawn((
                            Button,
                            HitArea,
                            ControlButton {
                                action: entry.action,
                                selected: false,
                                strong: false,
                            },
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Percent(50. + phase.sin() * 37.),
                                top: Val::Percent(50. - phase.cos() * 37.),
                                margin: UiRect {
                                    left: Val::Px(-slot.x * 0.5),
                                    top: Val::Px(-slot.y * 0.5),
                                    ..default()
                                },
                                width: Val::Px(slot.x),
                                height: Val::Px(slot.y),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                column_gap: Val::Px(4.),
                                border: UiRect::all(Val::Px(2.)),
                                padding: UiRect::axes(Val::Px(3.), Val::Px(1.)),
                                ..default()
                            },
                            BackgroundColor(rgb(SURFACE)),
                            BorderColor(rgb(BRASS)),
                            Outline::new(Val::Px(3.), Val::Px(2.), Color::NONE),
                            BorderRadius::all(Val::Px(14.)),
                        ))
                        .with_children(|chip| {
                            if let UiAction::SelectBuilding(kind) = entry.action {
                                building_icon(
                                    chip,
                                    hud,
                                    biome,
                                    kind,
                                    if layout.short { 30. } else { 42. },
                                );
                            } else {
                                glyph(chip, hud, entry.glyph, if layout.short { 16. } else { 22. });
                            }
                            let title = text(
                                chip,
                                hud,
                                entry.name,
                                if layout.short { 12. } else { 14. },
                                LIGHT,
                            );
                            chip.commands().entity(title).insert(TextFont {
                                font: hud.bold.clone(),
                                font_size: if layout.short { 12. } else { 14. },
                                ..default()
                            });
                        });
                    }
                });
            text(
                wheel_root,
                hud,
                if layout.gamepad {
                    "Stick oder Steuerkreuz · A/Kreuz wählen · B/Kreis zurück"
                } else {
                    "Symbol antippen · Pfeile und Enter auf der Tastatur"
                },
                13.,
                LIGHT,
            );
        });
}

fn modal(parent: &mut ChildSpawnerCommands, hud: &Hud, game: &Game, play: &Play, layout: Layout) {
    let region = game.region();
    let city = game.city();
    let tight = layout.narrow_modal
        || (layout.short
            && matches!(
                play.panel,
                Panel::MainMenu
                    | Panel::Work
                    | Panel::Display
                    | Panel::Dedication
                    | Panel::Talent(_)
                    | Panel::Progress
                    | Panel::Graduation
                    | Panel::Briefing
                    | Panel::Gathering
                    | Panel::Projects(_)
                    | Panel::Town(_)
                    | Panel::Household(_)
                    | Panel::Council
                    | Panel::Village(_)
                    | Panel::VillageLog
                    | Panel::SideQuest
                    | Panel::Quests
                    | Panel::Map
                    | Panel::Menu
                    | Panel::Spells
                    | Panel::Modes
                    | Panel::Catalog
                    | Panel::Inspection
            ));
    parent.spawn((
        Node{width:Val::Percent(100.),height:Val::Percent(100.),flex_direction:FlexDirection::Column,align_items:AlignItems::Center,
            justify_content:JustifyContent::Center,..default()},

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
            if !matches!(play.panel, Panel::MainMenu | Panel::Inventory | Panel::Modes | Panel::Area(_) | Panel::Catalog | Panel::Inspection | Panel::Briefing | Panel::Graduation | Panel::Departure) {
                let actions: &[(&str, UiAction, bool)] = if play.at_title() {
                    &[("Zurück", UiAction::Cancel, true)]
                } else if play.panel == Panel::Menu {
                    &[("Hauptmenü", UiAction::MainMenu, false), ("Fortsetzen", UiAction::Resume, true)]
                } else {
                    &[("Zur Insel", UiAction::Resume, false), ("Zurück", UiAction::Cancel, true)]
                };
                modal_header(box_node, hud, match play.panel {
                    Panel::Town(_) => "Stadtplanung",
                    Panel::Household(t) => if city.building_at(t).is_some_and(|s|s.kind==Building::House) {"Wohnviertel"} else {"Standortprüfung"},
                    Panel::Work=>"Inselwerk",
                    Panel::Dedication=>"Das Werk widmen",
                    Panel::Talent(_)=> "Zauberschule",
                    Panel::Progress=>"Dein Fortschritt",
                    Panel::Council => "Dörfer der Insel",
                    Panel::Village(i) | Panel::Projects(i) => &region.villages[i].name,
                    Panel::Map => "Landkarte",
                    Panel::Menu => "Spielmenü",
                    Panel::Controls => "Steuerung",
                    Panel::Orders(tool) => tool.name(),
                    Panel::Spells => "Zauberbuch",
                    Panel::Quests => "Aufträge",
                    Panel::SideQuest => "Nebenauftrag",
                    Panel::FirstDelivery => "Wohin mit dem Holz?",
                    Panel::Sound => "Ton",
                    Panel::Display => "Anzeige",
                    _ => "",
                }, layout.short, actions);
            }
            match play.panel {
                Panel::Briefing => {
                    island_heading(box_node, hud, region.kind, city.quest_title(region), &region.name, layout.short);
                    if !region.villages.is_empty() {
                        button(box_node,hud,"Nachbardörfer besuchen",UiAction::Panel(Panel::Council),false,false);
                    }
                    let message = if city.waiting_for_first_delivery() {
                        "Zwei Dächer sind beschädigt. Nimm sechs Holz am markierten Holzstoß. Danach entscheidest du, ob du Mira im Dorf oder Jora am Fluss hilfst.".to_owned()
                    } else {
                        city.objective(region).1
                    };
                    text(box_node, hud, &message, 16., MUTED);
                    text(box_node, hud, if layout.gamepad {
                        "L3 zeigt dein Auftragsziel. Mit A führst du die Aktion dort aus."
                    } else {
                        "„Zum Ziel“ zeigt den Ort und wählt das Werkzeug. Unten bestätigst du die Aktion."
                    }, 15., INK);
                    button(box_node, hud, "Zum ersten Ziel", UiAction::Resume, false, true);
                }
                Panel::Graduation => {
                    island_heading(box_node, hud, region.kind, "Deine Insel ist versorgt", "Übungsinsel · 8 von 8", layout.short);
                    text(box_node, hud, "Dein Dorf hat ein Dach und Nahrung. Dein Nilpferd holt Vorräte.", 17., INK);
                    text(box_node, hud, "Im Weidental entscheidest du, welche Familie zuerst ein Dach bekommt.", 16., MUTED);
                    button(box_node, hud, "Ins Weidental", UiAction::SelectMode(Mode::Campaign(Biome::Valley)), false, true);
                    button(box_node, hud, "Andere Insel wählen", UiAction::Panel(Panel::Modes), false, false);
                }
                Panel::MainMenu => main_menu(box_node, hud, game, layout.short),
                Panel::Menu => menu_panel(box_node,hud,game,layout),
                Panel::Controls => controls_panel(box_node, hud, play.gamepad, layout.short),
                Panel::Orders(tool) => orders_panel(box_node,hud,play,tool,layout.short),
                Panel::Spells => spellbook(box_node,hud,play,layout.short),
                Panel::Gathering => gathering_panel(box_node,hud,game,play,layout),
                Panel::Council => villages::council(box_node,hud,game,layout.short),
                Panel::Projects(i) => villages::projects(box_node,hud,game,i,layout.short),
                Panel::Village(i) => villages::village(box_node,hud,game,i,layout.short),
                Panel::VillageLog => village_log(box_node,hud,game,layout.short),
                Panel::Departure => departure_panel(box_node, hud, game, layout.short),
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
                                else if index == game.campaign.active + 1 && city.quest == Quest::Complete {" · Weiterreise"} else {""};
                            chapters.spawn(Node { width: Val::Percent(48.), flex_direction: FlexDirection::Column, ..default() }).with_children(|card| {
                                button(card,hud,&format!("{}{suffix}",candidate.name),UiAction::Do(Action::Travel(index)),index==game.campaign.active,false);
                                if !layout.short { text(card,hud,&candidate.subtitle,14.,MUTED); }
                            });
                        }
                    });
                }
                Panel::Map => map_panel(box_node,hud,game,layout),
                Panel::SideQuest => side_quest_panel(box_node,hud,game,layout),
                Panel::Sound => sound_panel(box_node,hud,play,layout.short),
                Panel::Area(index) => area_panel(box_node,hud,game,layout,index),
                Panel::Catalog => catalog_panel(box_node,hud,game,play,layout.short),
                Panel::Research => research_panel(box_node,hud,game,play.tech,layout.short),
                Panel::Inspection => inspection_panel(box_node,hud,game,play,layout.short),
                Panel::Inventory => inventory_panel(box_node, hud, layout, region.kind),
                Panel::Modes => mode_panel(box_node, hud, game, layout),
                Panel::Progress=>works::progress(box_node,hud,game,layout.short),
                Panel::Display=>{
                    text(box_node, hud, "Leseabstand", if layout.short {18.} else {23.}, INK);
                    box_node.spawn(column_grid(3)).with_children(|choices| {
                        for value in crate::display::ReadingSize::ALL {
                            styled_button(choices,hud,value.name(),UiAction::Reading(value),play.reading==value,false,ButtonStyle {compact: layout.short, ..default()});
                        }
                    });
                    let hint = match play.reading {
                        crate::display::ReadingSize::Auto => "Mit Controller werden Schrift, Auswahlräder und Tastenhinweise für mehr Abstand größer.",
                        crate::display::ReadingSize::Large => "Große Schrift und Bedienelemente für das Tablet auf dem Tisch.",
                        crate::display::ReadingSize::Compact => "Kleinere Menüs lassen auf der Insel mehr Platz zum Planen.",
                    };
                    text(box_node,hud,hint,if layout.short {15.} else {18.},MUTED);
                }
                Panel::Talent(t)=>works::talents(box_node,hud,game,t,layout.short),
                Panel::Work=>works::panel(box_node,hud,game,layout.short),
                Panel::Dedication=>works::dedication(box_node,hud,game,layout.short),
                Panel::Town(page) => town_ui::atlas(box_node,hud,game,page,play.aim,layout.short),
                Panel::Household(tile) => town_ui::home(box_node,hud,game,tile,layout.short),
                Panel::Quests => quest_journal(box_node, hud, game, layout),
                Panel::FirstDelivery => first_delivery_panel(box_node, hud, game, layout.short),
                Panel::Closed | Panel::Context | Panel::Story | Panel::Wheel(_) => {}
            }
            if layout.gamepad && !layout.narrow_modal && !matches!(play.panel, Panel::Work | Panel::Progress|Panel::Talent(_)|Panel::Dedication|Panel::Controls | Panel::Catalog | Panel::Inspection | Panel::Quests | Panel::SideQuest | Panel::Graduation | Panel::Departure | Panel::Briefing | Panel::Village(_) | Panel::Projects(_) | Panel::Town(_) | Panel::Household(_) | Panel::Council) {
                text(box_node, hud, "Stick oder Steuerkreuz · A wählen · B zurück", 12., MUTED);
            }
        });
        // The message may wrap or expire while a player is reaching for the next button.
        // Reserve its space outside the card so the buttons keep their screen positions.
        if !matches!(play.panel, Panel::MainMenu | Panel::Display | Panel::Controls | Panel::Map | Panel::FirstDelivery | Panel::Modes | Panel::Catalog | Panel::Inspection) && !layout.narrow_modal {
            shade.spawn(Node {
                width: Val::Percent(92.), max_width: Val::Px(720.),
                height: Val::Px(if layout.short {38.} else {48.}), flex_shrink: 0.,
                align_items: AlignItems::Start, justify_content: JustifyContent::Center,
                padding: UiRect::top(Val::Px(4.)), ..default()
            }).with_children(|slot| feedback_toast(slot, hud, Node { max_width: Val::Percent(100.), ..default() }, false));
        }
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
    if matches!(play.panel, Panel::Map | Panel::Council) {
        let map_key = (game.mode, game.campaign.active);
        if hud
            .map
            .as_ref()
            .is_none_or(|(current, _)| *current != map_key)
        {
            hud.map = Some((map_key, images.add(chart(game.region()))));
        }
    }
    let safe = crate::controls::safe_insets(&window);
    let [left, top_inset, right, bottom_inset] = safe.map(f32::from);
    let size = Vec2::new(
        window.width() - left - right,
        window.height() - top_inset - bottom_inset,
    );
    let is_modal = !matches!(play.panel, Panel::Closed | Panel::Context | Panel::Story);
    let wanted = play.reading.scale(size, play.gamepad, is_modal);
    if (scale.0 - wanted).abs() > 0.01 {
        scale.0 = wanted;
    }
    let key = Layout {
        region: game.campaign.active,
        lesson: play
            .practice_step
            .map(|step| (step, step.next_action(&play, &game))),
        mode: game.mode,
        menu_page: play.menu_page,
        tool: play.tool,
        option: play.option,
        panel: play.panel,
        gamepad: play.gamepad,
        unlocked: game.campaign.unlocked,
        quest: game.city().quest,
        folded: play.tracker_folded,
        work_stage: game.city().work.stage,
        work_pinned: game.city().work.pinned,
        reading: play.reading,
        work_paid: game.city().work.delivered,
        appeal: game.city().projects.appeal.as_ref().map(|a| a.village),
        side_quest: if matches!(
            play.panel,
            Panel::Closed | Panel::Map | Panel::Quests | Panel::SideQuest
        ) {
            game.city().side_quest(game.region())
        } else {
            SideQuest::Complete
        },
        mix: (play.panel == Panel::Sound).then_some(play.mix),
        campaign_complete: game.campaign_complete(),
        circle_active: game.city().gatherings.circle.is_some(),
        talents: if matches!(play.panel, Panel::Progress | Panel::Talent(_)) {
            Talent::ALL.map(|talent| game.campaign.gifts.rank(talent))
        } else {
            [0; Talent::ALL.len()]
        },
        road_started: play.road_anchor.is_some(),
        compact: size.x / scale.0 < 920.,
        short: size.y / scale.0 < 490. || (play.panel == Panel::Map && size.x / scale.0 < 690.),
        narrow_legend: size.x / scale.0 < 550.,
        narrow_modal: match play.panel {
            Panel::Map => size.x / scale.0 < 550.,
            Panel::Area(_) => size.x / scale.0 < 550. || size.y / scale.0 < 320.,
            _ => false,
        },
        safe,
        catalog: play.catalog_branch,
        catalog_page: play.catalog_page,
        inspected: (play.panel == Panel::Inspection)
            .then(|| {
                game.city()
                    .building_at(play.aim)
                    .map(|site| (site.kind, site.upgraded))
            })
            .flatten(),
        can_inspect: play.panel == Panel::Closed
            && play.tool == Tool::Hand
            && game.city().held.is_none()
            && game
                .city()
                .building_at(play.aim)
                .is_some_and(|site| site.kind == Building::Field),
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
            BackgroundColor(match play.panel {
                Panel::Closed | Panel::Context | Panel::Story => Color::NONE,
                Panel::Wheel(_) => rgb(0x15291d).with_alpha(0.67),
                _ => rgb(0x14261d).with_alpha(0.64),
            }),
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
                    if let Some((step, action)) = key.lesson {
                        lesson_bar(safe_root, &hud, step, action, key);
                    } else if play.tool == Tool::Build {
                        placement_bar(safe_root, &hud, &play, key, game.region().kind);
                    } else {
                        bottom(safe_root, &hud, &play, key);
                    }
                } else if play.panel == Panel::Context {
                    object_popup(safe_root, &hud, &play);
                } else if play.panel == Panel::Story {
                    cutscene(safe_root, &hud, key);
                } else if let Panel::Wheel(wheel) = play.panel {
                    selection_wheel(safe_root, &hud, wheel, key, game.region().kind);
                } else {
                    modal(safe_root, &hud, &game, &play, key);
                }
            });
        })
        .id();
    hud.root = Some(entity);
    hud.layout = Some(key);
}

pub(crate) fn update(
    game: Res<Game>,
    play: Res<Play>,
    target: Res<Target>,
    time: Res<Time>,
    realtime: Res<Time<Real>>,
    sound: Res<Sound>,
    mut portraits: Query<&mut ImageNode, With<CatalogPortrait>>,
    mut feedback_icons: Query<&mut ImageNode, (With<FeedbackIcon>, Without<CatalogPortrait>)>,
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
    mut toasts: Query<(&Toast, &mut Visibility, &mut Node, &mut BackgroundColor)>,
) {
    let city = game.city();
    let region = game.region();
    let gifts = game.campaign.gifts;
    let growth = gifts.creature;
    let notice_active = !play.notice.is_empty() && time.elapsed_secs() < play.notice_until;
    let (notice_color, notice_icon) = match play.notice_kind {
        Feedback::Action => (DARK, 11),
        Feedback::Selection => (SURFACE, 14),
        Feedback::Quest => (0x555130, 12),
        Feedback::Blocked => (0x69402f, 27),
    };
    for mut image in &mut feedback_icons {
        if let Some(atlas) = &mut image.texture_atlas {
            if atlas.index != notice_icon {
                atlas.index = notice_icon;
            }
        }
    }
    for mut image in &mut portraits {
        if let Some(atlas) = &mut image.texture_atlas {
            let index = play.catalog_preview.index();
            if atlas.index != index {
                atlas.index = index;
            }
        }
    }
    for (tag, mut text) in &mut labels {
        let value = match tag {
            Label::Stock(material) => city.stock[material.index()].to_string(),
            Label::Faith => city.faith.to_string(),
            Label::FaithUse => format!("{} Glauben\nFür Ausbau und Einfluss", city.faith),
            Label::Mana => format!("{:.0}/{:.0}", city.mana, city.mana_cap(region, gifts)),
            Label::ManaUse => format!(
                "{:.0}/{:.0} Wunderkraft\nFür Wunder · lädt nach",
                city.mana,
                city.mana_cap(region, gifts)
            ),
            Label::Calendar => format!(
                "{} · {}",
                city.day_phase().name(),
                region.weather(city.weather_time).name()
            ),
            Label::Raid => city.next_raid(region).map_or_else(String::new, |seconds| {
                if city.rival_active(region) {
                    let rival = crate::rivals::Rival::for_biome(region.kind);
                    format!(
                        "{} · Angriff in {:.0} s\n{}\nAbwehr mit {}.",
                        rival.creature,
                        seconds.ceil(),
                        rival.warning,
                        rival.defense
                    )
                } else {
                    format!(
                        "Unterwegs zum Übergang · Überfall in {:.0} s",
                        seconds.ceil()
                    )
                }
            }),
            Label::RaidAlert => city
                .next_raid(region)
                .filter(|&seconds| seconds <= 10. && city.rival_active(region))
                .map_or_else(String::new, |seconds| {
                    format!(
                        "{} · Angriff in {:.0} s",
                        crate::rivals::Rival::for_biome(region.kind).creature,
                        seconds.ceil()
                    )
                }),
            Label::Objective => city.objective(region).1,
            Label::PracticeStatus => game
                .practice_city()
                .map_or(
                    "Übungsinsel",
                    |city| match crate::practice::Step::current(city) {
                        crate::practice::Step::Grab => "Übungsinsel",
                        crate::practice::Step::Complete => "Übung erneut spielen",
                        _ => "Übung fortsetzen",
                    },
                )
                .into(),
            Label::AudioStatus => sound.status(play.mix, realtime.elapsed_secs_f64()).into(),
            Label::ResumeMode => game.continue_mode().map_or("", Mode::name).into(),
            Label::BlueprintStatus => {
                if target.legal {
                    format!("Bauplatz frei · {}", target.detail)
                } else {
                    target.detail.clone()
                }
            }
            Label::CatalogName => play.catalog_preview.name().into(),
            Label::CatalogPurpose => play.catalog_preview.use_text().into(),
            Label::CatalogPrice => {
                if region.sandbox {
                    "Keine Baukosten im freien Bauen".into()
                } else {
                    format!("Baukosten · {}", play.catalog_preview.cost_text())
                }
            }
            Label::CatalogReadiness => game
                .blueprint_readiness(play.catalog_preview)
                .err()
                .unwrap_or_else(|| {
                    let size = play.catalog_preview.size();
                    format!("Bauplan bereit · {size} × {size} Felder")
                }),
            Label::CatalogAction => {
                let action = if play
                    .catalog_preview
                    .civic()
                    .is_some_and(|civic| !game.campaign.research.knows(civic.spec().unlock))
                {
                    "Forschung ansehen"
                } else {
                    "Bauplatz wählen"
                };
                if play.gamepad {
                    format!("A  {action}")
                } else {
                    action.into()
                }
            }
            Label::WheelChoice | Label::WheelDetail => {
                if let Panel::Wheel(wheel) = play.panel {
                    wheel
                        .entries()
                        .iter()
                        .find(|entry| Some(entry.action) == play.focus)
                        .map_or_else(String::new, |entry| {
                            if matches!(tag, Label::WheelChoice) {
                                entry.name.into()
                            } else {
                                entry.detail(&game)
                            }
                        })
                } else {
                    String::new()
                }
            }
            Label::FirstDelivery(FirstDelivery::Village) => {
                if city.count(Building::House) == 0 {
                    format!("{PAW_BUNDLE} Holz fürs erste Dach\nMehr Stimmung im Dorf")
                } else {
                    format!("{PAW_BUNDLE} Holz für die Familien\nMehr Stimmung im Dorf")
                }
            }
            Label::FirstDelivery(FirstDelivery::Farm) => {
                format!(
                    "{PAW_BUNDLE} Holz für Joras Dach\n{} als Dank",
                    stock_text(FARM_REWARD)
                )
            }
            Label::CampaignHint
                if city.first_delivery_ready(region)
                    && city.first_delivery_at(region, play.aim).is_some() =>
            {
                if play.gamepad {
                    format!("A · {}", target.label)
                } else {
                    format!("Unten „{}“ antippen", target.label)
                }
            }
            Label::CampaignHint => city.campaign_hint(region, play.gamepad),
            Label::Creature => {
                let learned = Material::ALL
                    .into_iter()
                    .filter(|&m| gifts.supplies(m))
                    .map(|m| region.kind.material(m))
                    .collect::<Vec<_>>();
                let help = if learned.is_empty() {
                    "Lerne durch Lob nach einer Lieferung".into()
                } else {
                    format!("Vorratshilfe · {}", learned.join(", "))
                };
                format!(
                    "{} · Rang {} · {:.0}% Kraft · {}\n{help}",
                    growth.description(),
                    gifts.training,
                    city.energy,
                    city.creature_status()
                )
            }
            Label::Helpers => format!(
                "{} sammeln · {}",
                city.gathering_helpers(),
                if city.idle_helper() {
                    "Gläubige verfügbar"
                } else {
                    "Alle beschäftigt"
                }
            ),
            Label::Spell(spell) => format!(
                "{:.0} Kraft · Radius {}\n{}",
                spell.cost(),
                spell.radius(gifts),
                city.ritual_status(region, *spell, gifts)
                    .unwrap_or_else(|| spell.effect())
            ),
            Label::Order(tool, index) => match (*tool, *index) {
                (Tool::Hippo, 6) => "Freien Boden wählen",
                (Tool::Hippo, 1) if gifts.training < 2 => "Ab Rang 2",
                (Tool::Hippo, 2) if gifts.training < 3 => "Ab Rang 3",
                (Tool::Hippo, 2) => "Feld wählen",
                (Tool::Helpers, 1) => "Am Wachhaus",
                (Tool::Hippo, 1) | (Tool::Helpers, 2) => "Posten wählen",
                (Tool::Plan, 0) => "Anfang und Ende wählen",
                (Tool::Plan, 1) => "Gebäude am Weg wählen",
                _ => "Quelle wählen",
            }
            .into(),
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
            Label::WorkHint => city.work_hint(region),
            Label::QuestCount => quests_ui::counter(&game),
            Label::SideTrack => {
                if city.side_quest(region) == SideQuest::Ready {
                    "Erfüllt · Belohnung im Tagebuch".into()
                } else {
                    city.local_next(region)
                        .map_or_else(|| "Zur Dorfbitte".into(), |g| g.progress(city, region))
                }
            }
            Label::Quest if region.sandbox => "Baue, ernte oder handle".into(),
            Label::Quest if city.quest == Quest::Complete && !region.tutorial => {
                "Deine Nachbardörfer".into()
            }
            Label::Quest => city.next_task(region),
            Label::VillageWatch => {
                let c = &city.contest;
                if let Some(a) = &city.projects.appeal {
                    format!(
                        "{} · Noch {:.0} s für Hilfe",
                        region.villages[a.village].name,
                        a.seconds.ceil()
                    )
                } else if c.occupation > 0. {
                    if c.occupation >= 75. {
                        "Rivale fordert Tribut".into()
                    } else {
                        format!(
                            "Rivalenmehrheit · Tribut in {:.0} s",
                            (75. - c.occupation).ceil()
                        )
                    }
                } else if let Some((i, v)) = region.villages.iter().enumerate().find(|(i, _)| {
                    city.visitor(*i, true).is_some() || city.visitor(*i, false).is_some()
                }) {
                    format!("{} · {}", v.name, c.status(i))
                } else if let Some(i) = c.destination {
                    format!("Rivale → {}", region.villages[i].name)
                } else {
                    format!(
                        "Inselbund · {}/{} Dörfer",
                        c.count(crate::game::Allegiance::Ours),
                        c.majority()
                    )
                }
            }
            Label::CircleStatus => city
                .gatherings
                .circle
                .as_ref()
                .map(|c| c.status(city))
                .unwrap_or_default(),
            Label::Notice => {
                if notice_active {
                    play.notice.clone()
                } else {
                    String::new()
                }
            }
            Label::Confirm if play.practice_step.is_some() && play.grab.is_some() => {
                let hold = play.grab.unwrap();
                format!(
                    "Loslassen · {}/{}",
                    hold.amount(time.elapsed_secs()),
                    hold.limit()
                )
            }
            Label::Confirm if play.gamepad => format!("A  {}", target.label),
            Label::Confirm => target.label.clone(),
            Label::Detail => target.detail.clone(),
            Label::Rival if city.outcome.is_none() => format!(
                "{}{} · {} · {} {} · Ansehen {}/{}\n{} für den Übergang",
                region.rival,
                if region.has_rival_creature() {
                    format!(
                        " und {}",
                        crate::rivals::Rival::for_biome(region.kind).creature
                    )
                } else {
                    String::new()
                },
                region.request,
                region.gift.1,
                region.kind.material(region.gift.0),
                city.renown(),
                region.renown,
                stock_text(region.rebuild)
            ),
            Label::Rival => format!("{} · {}", region.rival, city.outcome.unwrap().name()),
            Label::District(index) | Label::DistrictBrief(index)
                if city.has_trading_partner(region, *index) =>
            {
                let offer = city.trade_offer(region, *index);
                if city.districts[*index].last_deal.is_some_and(|(day, _)| {
                    day == (city.weather_time / crate::game::DAY_LENGTH) as u32
                }) {
                    "Nächste Lieferung morgen".into()
                } else {
                    format!("Handel · {}", offer.terms(region.kind))
                }
            }
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
            Label::LocalQuest => city.local_quest(region).1,
        };
        if text.0 != value {
            text.0 = value;
        }
    }
    for (toast, mut visible, mut node, mut surface) in &mut toasts {
        let show = notice_active && (toast.overlay || play.notice_panel == play.panel);
        visible.set_if_neq(if show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
        let display = if show { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
        surface.set_if_neq(BackgroundColor(rgb(notice_color)));
    }
    for (button, interaction, mut color, mut border, mut outline) in &mut buttons {
        let focused = play.panel != Panel::Closed && play.focus == Some(button.action);
        let blueprint_selected = button.action == UiAction::PreviewBuilding(play.catalog_preview);
        let hovered = *interaction != Interaction::None;
        let unavailable = match button.action {
            UiAction::Confirm => !target.legal,
            UiAction::Do(Action::Call(kind, near)) => {
                city.gathering_place(region, kind, near).is_err() || city.gatherings.rest > 0.
            }
            UiAction::Do(Action::Invoke(_)) => {
                city.mana < 6.
                    || !city
                        .gatherings
                        .circle
                        .as_ref()
                        .is_some_and(|c| c.ready(city))
            }
            UiAction::Do(Action::Upgrade(tile)) => city.upgrade_plan(region, tile).is_err(),
            UiAction::Do(Action::Travel(index)) => index > game.campaign.unlocked,
            UiAction::Do(Action::Learn(talent)) => gifts.rank(talent) >= 2 || gifts.points == 0,
            UiAction::Do(Action::Research(tech)) => game
                .campaign
                .research
                .available(tech, city.city_rank())
                .is_err(),
            UiAction::Do(Action::Camp(index)) => city.camp_plan(index).is_err(),
            UiAction::Order(Tool::Hippo, 1) => gifts.training < 2,
            UiAction::Order(Tool::Hippo, 2) => gifts.training < 3,
            UiAction::Order(Tool::Miracles, index) => city
                .ritual_status(region, Miracle::ALL[index], gifts)
                .is_some(),
            UiAction::Do(Action::Praise) => city.lesson.is_none(),
            UiAction::Do(Action::Pet) => !city.pet_ready(),
            UiAction::Do(Action::RecallHelpers) => city.gathering_helpers() == 0,
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
        } else if focused || hovered || blueprint_selected {
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
        border.0 = rgb(if focused || blueprint_selected {
            0xffecaa
        } else if button.selected || button.strong {
            GOLD
        } else {
            BRASS
        });
        outline.color = if focused { rgb(0x684924) } else { Color::NONE };
        if focused && play.gamepad {
            outline.width = Val::Px(4.);
            outline.offset = Val::Px(1.);
            border.0 = rgb(0xffdf86);
        }
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
