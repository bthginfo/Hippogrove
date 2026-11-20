use bevy::prelude::*;

use crate::{
    civic::{Branch, Civic, Goods, Tech},
    game::{Action, Building, Crop, Game, Miracle, Quest, stock_text},
    modes::Mode,
    quests::Goal,
    region::{Biome, Material, Tile},
    sound::{Bus, Cue, Mix},
    story::{Sequence, Story},
    terrain,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Tool {
    #[default]
    Hand,
    Build,
    Helpers,
    Hippo,
    Miracles,
    Plan,
}

impl Tool {
    pub const ALL: [Self; 6] = [
        Self::Hand,
        Self::Build,
        Self::Helpers,
        Self::Hippo,
        Self::Miracles,
        Self::Plan,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Hand => "Pfote",
            Self::Build => "Bauen",
            Self::Helpers => "Gläubige",
            Self::Hippo => "Nilpferd",
            Self::Miracles => "Wunder",
            Self::Plan => "Wege",
        }
    }

    pub fn choices(self) -> usize {
        match self {
            Self::Build => Building::CATALOG_SIZE,
            Self::Helpers => 3,
            Self::Miracles => Miracle::ALL.len(),
            Self::Hippo => 7,
            Self::Plan => 2,
            _ => 1,
        }
    }

    pub fn choice_name(self, index: usize) -> &'static str {
        match self {
            Self::Build => Building::from_index(index).name(),
            Self::Helpers => ["Sammeln", "Ausbilden", "Patrouille"][index],
            Self::Miracles => Miracle::ALL[index].name(),
            Self::Hippo => [
                "Sammeln",
                "Wache",
                "Feldpflege",
                "Loben",
                "Snack",
                "Pause",
                "Hierher",
            ][index],
            Self::Plan => ["Wege", "Ausbau"][index],
            _ => self.name(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Panel {
    #[default]
    Closed,
    Menu,
    Orders(Tool),
    Spells,
    Journey,
    Map,
    Area(usize),
    Progress,
    Quests,
    SideQuest,
    Sound,
    Catalog,
    Research,
    Inspection,
    Inventory,
    Story,
    Modes,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAction {
    Tool(Tool),
    Order(Tool, usize),
    Confirm,
    Cancel,
    Rotate,
    Guide,
    Panel(Panel),
    Focus(Tile),
    AreaMaterial(Material),
    AreaSource(usize, Material),
    Landscape,
    ToggleObjective,
    Catalog(Option<Branch>),
    CatalogPage(usize),
    SelectBuilding(Building),
    ResearchBranch(Branch),
    SelectTech(Tech),
    Do(Action),
    FinishStory,
    ReplayArrival,
    ModeTab(bool),
    SelectMode(Mode),
    Sound(Bus),
}

#[derive(Clone, Copy, Debug)]
pub enum PawGesture {
    Grab,
    Release,
    Cast,
    Point,
}

#[derive(Resource)]
pub struct Play {
    pub aim: Tile,
    pub tool: Tool,
    pub panel: Panel,
    panel_history: Vec<(Panel, Option<UiAction>)>,
    pub story: Option<Story>,
    previous_quest: Quest,
    pub free_build_tab: bool,
    pub area_material: Material,
    pub option: usize,
    pub rotation: u8,
    pub road_anchor: Option<Tile>,
    pub catalog_branch: Option<Branch>,
    pub catalog_page: usize,
    pub tech: Tech,
    pub focus: Option<UiAction>,
    pub notice: String,
    pub notice_until: f32,
    pub notice_ok: bool,
    pub last_magic: Option<(Miracle, Tile, u64)>,
    pub last_sound: Option<(Cue, u64)>,
    pub mix: Mix,
    pub paw_gesture: Option<(PawGesture, f32)>,
    pub gamepad: bool,
    pub pointer: Option<Vec2>,
    pub paused: bool,
    pub revision: u64,
}

impl Play {
    pub fn new(game: &Game) -> Self {
        let story = (!game.campaign.intro_seen && game.campaign.active == 0).then(|| {
            Story::new(if game.region().kind == Biome::Aegean {
                Sequence::Islands
            } else {
                Sequence::Arrival
            })
        });
        Self {
            aim: game.region().sources[0].tile,
            tool: Tool::Hand,
            panel: if story.is_some() {
                Panel::Story
            } else {
                Panel::Closed
            },
            panel_history: Vec::new(),
            story,
            previous_quest: game.city().quest,
            free_build_tab: game.mode.sandbox(),
            area_material: Material::Wood,
            option: 0,
            rotation: 0,
            road_anchor: None,
            catalog_branch: None,
            catalog_page: 0,
            tech: Tech::Foundations,
            focus: None,
            notice: String::new(),
            notice_until: 0.,
            notice_ok: true,
            last_magic: None,
            last_sound: None,
            mix: Mix::default(),
            paw_gesture: None,
            gamepad: false,
            pointer: None,
            paused: false,
            revision: 0,
        }
    }

    pub fn building(&self) -> Building {
        Building::from_index(self.option.min(Building::CATALOG_SIZE - 1))
    }

    pub fn miracle(&self) -> Miracle {
        Miracle::ALL[self.option.min(Miracle::ALL.len() - 1)]
    }

    pub fn tool(&mut self, tool: Tool) {
        if self.tool != tool {
            self.tool = tool;
            self.option = 0;
            self.road_anchor = None;
        }
    }

    pub fn choose(&mut self, index: usize) {
        self.option = index.min(self.tool.choices() - 1);
        self.road_anchor = None;
    }

    pub fn target(&self, game: &Game) -> Action {
        let city = game.city();
        let region = game.region();
        match self.tool {
            Tool::Hand
                if city.held.is_none()
                    && self.aim.within(region.bridge(), 1)
                    && city.quest == Quest::Bridge
                    && city.count(Building::Shrine) > 0 =>
            {
                if city.bridge_fixed {
                    Action::Give
                } else {
                    Action::Repair
                }
            }
            Tool::Hand if city.held.is_none() && region.district_at(self.aim).is_some() => {
                Action::Settle(region.district_at(self.aim).unwrap())
            }
            Tool::Hand
                if city.held.is_none()
                    && region
                        .area_at(self.aim)
                        .is_some_and(|index| !city.camps[index]) =>
            {
                Action::Camp(region.area_at(self.aim).unwrap())
            }
            Tool::Hand => Action::Hand(self.aim),
            Tool::Build => Action::Build(self.building(), self.aim, self.rotation),
            Tool::Helpers if self.option == 1 => Action::Muster(self.aim),
            Tool::Helpers if self.option == 2 => Action::Patrol(self.aim),
            Tool::Helpers => Action::Assign(self.aim, false),
            Tool::Hippo if self.option == 1 => Action::Watch(self.aim),
            Tool::Hippo if self.option == 2 => Action::Tend(self.aim),
            Tool::Hippo if self.option == 3 => Action::Praise,
            Tool::Hippo if self.option == 4 => Action::Feed,
            Tool::Hippo if self.option == 5 => Action::Rest,
            Tool::Hippo if self.option == 6 => Action::MoveHippo(self.aim),
            Tool::Hippo => Action::Assign(self.aim, true),
            Tool::Miracles => Action::Cast(self.miracle(), self.aim),
            Tool::Plan if self.option == 1 => Action::Upgrade(self.aim),
            Tool::Plan => Action::Road(self.road_anchor.unwrap_or(self.aim), self.aim),
        }
    }

    fn guide_tile(&self, game: &Game) -> Tile {
        let city = game.city();
        let region = game.region();
        match self.tool {
            Tool::Build if self.building() == Building::Shrine => region.frontier,
            Tool::Helpers if self.option == 1 => city
                .buildings
                .iter()
                .find(|site| site.kind == Building::Civic(Civic::Barracks))
                .map_or(region.start, |site| site.tile),
            Tool::Hand if city.held.is_some() => city.nearest_store(region, self.aim),
            Tool::Hippo if self.option == 6 => region.start,
            Tool::Miracles if self.miracle() == Miracle::Mend => city.walkers[0].tile(),
            Tool::Miracles if self.miracle() == Miracle::Ward => region.start,
            Tool::Miracles if self.miracle() == Miracle::Rain => city
                .buildings
                .iter()
                .filter(|site| site.kind == Building::Field && city.influence(region, site.tile))
                .min_by_key(|site| {
                    (
                        !matches!(site.crop, Crop::Dry),
                        self.aim.distance(site.tile),
                    )
                })
                .map_or(region.start, |site| site.tile),
            Tool::Miracles if self.miracle() == Miracle::Bloom => {
                let growing_fields = city
                    .buildings
                    .iter()
                    .filter(|site| matches!(site.crop, Crop::Growing(_)))
                    .map(|site| site.tile);
                let depleted_plants = region
                    .sources
                    .iter()
                    .zip(&city.nodes)
                    .filter(|(source, stock)| {
                        source.material != Material::Stone && **stock < source.stock
                    })
                    .map(|(source, _)| source.tile);
                growing_fields
                    .chain(depleted_plants)
                    .filter(|&tile| city.influence(region, tile))
                    .min_by_key(|&tile| self.aim.distance(tile))
                    .unwrap_or(region.start)
            }
            Tool::Miracles if self.miracle().offensive() && city.quest == Quest::Bridge => city
                .guards
                .iter()
                .position(|&remaining| remaining > 0)
                .map(|index| Tile::from_world(city.guard_world(region, index)))
                .unwrap_or(region.bridge()),
            Tool::Hand | Tool::Miracles if city.quest == Quest::Districts => region
                .districts
                .iter()
                .zip(&city.districts)
                .filter(|(_, state)| state.outcome.is_none())
                .min_by_key(|(district, _)| self.aim.distance(district.center))
                .map(|(district, _)| district.center)
                .unwrap_or(region.start),
            Tool::Hand if city.quest == Quest::Fields => city
                .buildings
                .iter()
                .filter(|site| site.kind == Building::Field)
                .min_by_key(|site| {
                    (
                        !matches!(site.crop, Crop::Ripe),
                        self.aim.distance(site.tile),
                    )
                })
                .map_or(region.start, |site| site.tile),
            Tool::Hand if city.quest == Quest::Bridge => region.bridge(),
            _ => region
                .sources
                .iter()
                .zip(&city.nodes)
                .find(|(source, remaining)| **remaining > 0 && city.influence(region, source.tile))
                .map(|(source, _)| source.tile)
                .unwrap_or(region.start),
        }
    }

    fn guide_building(&mut self, game: &Game, kind: Building, roadside: bool) -> Tile {
        let region = game.region();
        let city = game.city();
        self.tool(Tool::Build);
        self.choose(kind.index());
        region
            .bounds
            .tiles()
            .filter(|tile| tile.within(region.start, 8))
            .filter(|&tile| game.building_plan(kind, tile).is_ok())
            .min_by_key(|&tile| {
                let connected = kind
                    .tiles(tile)
                    .flat_map(Tile::neighbors)
                    .any(|cell| city.road_at(region, cell) && city.walkable(region, cell));
                (roadside && !connected, tile.distance(region.start))
            })
            .unwrap_or(region.start)
    }

    fn guide_quest(&mut self, game: &Game) -> Option<Tile> {
        let city = game.city();
        let region = game.region();
        let step = match self.panel {
            Panel::Quests => city.quest_step(region),
            Panel::SideQuest => region.quests.map(|quests| &quests.side),
            _ => None,
        }?;
        let goal = step.pending(city, region)?;
        let supply = |material| {
            region
                .sources
                .iter()
                .zip(&city.nodes)
                .filter(|(source, stock)| {
                    **stock > 0
                        && source.material == material
                        && city.influence(region, source.tile)
                })
                .min_by_key(|(source, _)| self.aim.distance(source.tile))
                .map_or(region.start, |(source, _)| source.tile)
        };
        let tile = match goal {
            Goal::Harvests { .. } => {
                let field = city
                    .buildings
                    .iter()
                    .filter(|site| site.kind == Building::Field)
                    .min_by_key(|site| {
                        (
                            !matches!(site.crop, Crop::Ripe),
                            self.aim.distance(site.tile),
                        )
                    });
                if let Some(field) = field {
                    if matches!(field.crop, Crop::Dry) {
                        self.tool(Tool::Miracles);
                        self.choose(Miracle::Rain as usize);
                    } else {
                        self.tool(Tool::Hand);
                    }
                    field.tile
                } else {
                    self.guide_building(game, Building::Field, false)
                }
            }
            Goal::HippoDeliveries { .. } => {
                let tile = supply(Material::Wood);
                self.tool(Tool::Hippo);
                self.choose(0);
                tile
            }
            Goal::Reserve { material, .. } => {
                let tile = if city.held.is_some() {
                    city.nearest_store(region, self.aim)
                } else {
                    supply(material)
                };
                self.tool(Tool::Hand);
                tile
            }
            Goal::Roads { .. } => {
                self.tool(Tool::Plan);
                self.choose(0);
                Tile(region.cross_x, region.start.1 + 4)
            }
            Goal::Camps { .. } => {
                self.tool(Tool::Hand);
                region
                    .areas
                    .iter()
                    .zip(city.camps)
                    .find(|(_, open)| !open)
                    .map_or(region.start, |(area, _)| area.center)
            }
            Goal::OlivePressings { .. } => {
                let press = Building::Civic(Civic::OlivePress);
                if let Some(site) = city
                    .buildings
                    .iter()
                    .find(|site| site.kind == press && city.connected(region, site))
                {
                    let tile = if city.stock[Material::Food.index()] < 10 {
                        if city.held.is_some() {
                            city.nearest_store(region, self.aim)
                        } else {
                            supply(Material::Food)
                        }
                    } else {
                        site.tile
                    };
                    self.tool(Tool::Hand);
                    tile
                } else {
                    self.guide_building(game, press, true)
                }
            }
            Goal::Building {
                kind,
                roadside,
                upgraded,
                ..
            } => {
                let workshop_needed = upgraded
                    || matches!(kind, Building::Granary | Building::Market | Building::Tower);
                if (workshop_needed || kind == Building::Well) && city.city_rank() < 2 {
                    let missing = if city.count(Building::Store) == 0 {
                        Building::Store
                    } else {
                        Building::Field
                    };
                    self.guide_building(game, missing, false)
                } else if workshop_needed && city.city_rank() < 3 {
                    self.guide_building(game, Building::Workshop, false)
                } else if let Some(site) = city.buildings.iter().find(|site| {
                    site.kind == kind
                        && ((roadside && !city.connected(region, site))
                            || (upgraded && !site.upgraded))
                }) {
                    self.tool(Tool::Plan);
                    if city.connected(region, site) {
                        self.choose(1);
                        site.tile
                    } else {
                        self.choose(0);
                        kind.tiles(site.tile)
                            .flat_map(Tile::neighbors)
                            .filter(|&cell| {
                                city.walkable(region, cell) && !city.road_at(region, cell)
                            })
                            .min_by_key(|&cell| cell.distance(region.depot))
                            .unwrap_or(site.tile)
                    }
                } else {
                    self.guide_building(game, kind, roadside)
                }
            }
        };
        Some(tile)
    }

    pub fn activate(
        &mut self,
        action: UiAction,
        game: &mut Game,
        view: &mut CameraRig,
        now: f32,
    ) -> bool {
        if self.story.is_some() {
            if matches!(
                action,
                UiAction::FinishStory | UiAction::Cancel | UiAction::Confirm
            ) {
                self.finish_story(game, view);
            }
            return false;
        }
        match action {
            UiAction::Tool(Tool::Hand) => {
                self.tool(Tool::Hand);
                self.close_panels();
            }
            UiAction::Tool(tool) => self.open_tool(tool),
            UiAction::Order(tool, index) => {
                let gifts = game.campaign.gifts;
                let requirement = match (tool, index) {
                    (Tool::Hippo, 1) if gifts.training < 2 => {
                        Some("Wache wird auf Rang 2 verfügbar.")
                    }
                    (Tool::Hippo, 2) if gifts.training < 3 => {
                        Some("Feldpflege wird auf Rang 3 verfügbar.")
                    }
                    (Tool::Miracles, _) => {
                        game.city()
                            .ritual_requirement(game.region(), Miracle::ALL[index], gifts)
                    }
                    _ => None,
                };
                if let Some(reason) = requirement {
                    self.focus = Some(action);
                    self.notice = reason.into();
                    self.notice_ok = false;
                    self.notice_until = now + 4.;
                    return false;
                }
                self.tool(tool);
                self.choose(index);
                self.close_panels();
            }
            UiAction::Catalog(branch) => {
                if self.panel != Panel::Catalog {
                    self.open_panel(Panel::Catalog);
                }
                self.catalog_branch = branch;
                self.catalog_page = 0;
                self.focus = None;
            }
            UiAction::CatalogPage(index) => self.catalog_page = index,
            UiAction::SelectBuilding(kind) => {
                if let Some(civic) = kind.civic() {
                    let tech = civic.spec().unlock;
                    if !game.campaign.research.knows(tech) {
                        self.tech = tech;
                        self.open_panel(Panel::Research);
                        return false;
                    }
                }
                self.tool(Tool::Build);
                self.choose(kind.index());
                self.close_panels();
            }
            UiAction::SelectTech(tech) => {
                self.tech = tech;
                self.panel = Panel::Research;
            }
            UiAction::ResearchBranch(branch) => {
                self.tech = branch.topics()[0];
                self.panel = Panel::Research;
            }
            UiAction::Confirm
                if self.panel == Panel::Closed
                    && self.tool == Tool::Plan
                    && self.option == 0
                    && self.road_anchor.is_none() =>
            {
                if game.city().influence(game.region(), self.aim)
                    && game.city().walkable(game.region(), self.aim)
                {
                    self.road_anchor = Some(self.aim);
                } else {
                    self.notice = "Beginne auf freiem Boden im Einflussgebiet.".into();
                    self.notice_until = now + 4.;
                    self.notice_ok = false;
                }
            }
            UiAction::Confirm if self.panel == Panel::Closed => {
                if self.target(game) == Action::Hand(self.aim)
                    && game.city().held.is_none()
                    && game
                        .city()
                        .building_at(self.aim)
                        .is_some_and(|site| site.kind != Building::Field)
                {
                    self.open_panel(Panel::Inspection);
                    return false;
                }
                return self.execute(self.target(game), game, view, now);
            }
            UiAction::Confirm => {}
            UiAction::Cancel => {
                if self.panel == Panel::Modes && !game.campaign.intro_seen {
                    self.choose_mode(game.mode, game, view, now);
                } else if let Panel::Area(index) = self.panel {
                    self.back_panel();
                    if self.panel != Panel::Map {
                        self.open_panel(Panel::Map);
                    }
                    self.focus = Some(UiAction::Panel(Panel::Area(index)));
                } else if self.panel != Panel::Closed {
                    self.back_panel();
                } else if self.road_anchor.is_some() {
                    self.road_anchor = None;
                } else {
                    self.tool(Tool::Hand);
                }
            }
            UiAction::Rotate => self.rotation = (self.rotation + 1) % 4,
            UiAction::Guide => {
                self.aim = self
                    .guide_quest(game)
                    .unwrap_or_else(|| self.guide_tile(game));
                self.close_panels();
                view.focus(self.aim);
                self.pointer = None;
            }
            UiAction::Focus(tile) => self.focus_tile(tile, view),
            UiAction::AreaMaterial(material) => {
                self.area_material = material;
                self.focus = Some(action);
            }
            UiAction::AreaSource(index, material) => {
                let region = game.region();
                if let Some(area) = region.areas.get(index) {
                    let city = game.city();
                    let tile = region
                        .area_sources(index, material)
                        .min_by_key(|(source, node)| {
                            (city.nodes[*source] == 0, node.tile.distance(area.center))
                        })
                        .map_or(area.center, |(_, source)| source.tile);
                    self.focus_tile(tile, view);
                }
            }
            UiAction::Landscape => {
                self.close_panels();
                view.landscape();
                self.pointer = None;
            }
            UiAction::Panel(Panel::Area(index)) => {
                if let Some(area) = game.region().areas.get(index) {
                    self.open_panel(Panel::Area(index));
                    self.area_material = area.material;
                    self.focus = Some(UiAction::AreaMaterial(area.material));
                }
            }
            UiAction::Panel(panel) => self.open_panel(panel),
            UiAction::ToggleObjective => self.open_panel(Panel::Quests),
            UiAction::ReplayArrival if game.campaign.active == 0 => {
                self.start_story(Sequence::Arrival, game, view);
            }
            UiAction::ReplayArrival | UiAction::FinishStory => {}
            UiAction::ModeTab(sandbox) => self.free_build_tab = sandbox,
            UiAction::SelectMode(mode) => {
                self.choose_mode(mode, game, view, now);
            }
            UiAction::Sound(bus) => self.mix.toggle(bus),
            UiAction::Do(action) => return self.execute(action, game, view, now),
        }
        false
    }

    fn open_tool(&mut self, tool: Tool) {
        self.open_panel(match tool {
            Tool::Build => Panel::Catalog,
            Tool::Miracles => Panel::Spells,
            _ => Panel::Orders(tool),
        });
    }

    fn open_panel(&mut self, panel: Panel) {
        if panel == self.panel {
            self.back_panel();
            return;
        }
        if let Some(index) = self
            .panel_history
            .iter()
            .rposition(|(previous, _)| *previous == panel)
        {
            self.panel_history.truncate(index + 1);
            self.back_panel();
            return;
        }
        if self.panel != Panel::Closed {
            self.panel_history.push((self.panel, self.focus));
        }
        self.panel = panel;
        self.focus = match panel {
            Panel::Menu => Some(UiAction::Panel(Panel::Map)),
            Panel::Map => Some(UiAction::Landscape),
            Panel::Orders(Tool::Hippo) => Some(UiAction::Order(Tool::Hippo, 6)),
            Panel::Orders(tool) => Some(UiAction::Order(tool, 0)),
            Panel::Spells => Some(UiAction::Order(Tool::Miracles, 0)),
            _ => None,
        };
    }

    fn back_panel(&mut self) {
        (self.panel, self.focus) = self.panel_history.pop().unwrap_or((Panel::Closed, None));
    }

    fn close_panels(&mut self) {
        self.panel_history.clear();
        self.panel = Panel::Closed;
        self.focus = None;
    }

    fn focus_tile(&mut self, tile: Tile, view: &mut CameraRig) {
        self.aim = tile;
        self.tool(Tool::Hand);
        self.close_panels();
        self.pointer = None;
        view.focus(tile);
    }

    fn choose_mode(&mut self, mode: Mode, game: &mut Game, view: &mut CameraRig, now: f32) {
        match game.switch_mode(mode) {
            Ok(name) => {
                let revision = self.revision.wrapping_add(1);
                let gamepad = self.gamepad;
                let mix = self.mix;
                *self = Self::new(game);
                self.revision = revision;
                self.gamepad = gamepad;
                self.mix = mix;
                *view = CameraRig::new(game);
                view.jump = true;
                if let Some(story) = self.story {
                    story.frame(game.region(), view);
                    self.focus = Some(UiAction::FinishStory);
                } else {
                    self.notice = name;
                    self.notice_until = now + 3.;
                    self.notice_ok = true;
                }
            }
            Err(reason) => {
                self.notice = reason;
                self.notice_until = now + 5.;
                self.notice_ok = false;
            }
        }
    }

    fn execute(&mut self, action: Action, game: &mut Game, view: &mut CameraRig, now: f32) -> bool {
        let previous = game.campaign.active;
        let carried = game.city().held.is_some();
        let first_island_landing = matches!(action, Action::Travel(4))
            && game.mode == Mode::Chronicle
            && game
                .campaign
                .settlements
                .get(4)
                .is_some_and(Option::is_none);
        let result = game.act(action);
        let success = result.is_ok();
        self.notice = result.unwrap_or_else(|error| error);
        self.notice_until = now + 5.;
        self.notice_ok = success;
        if success {
            self.revision = self.revision.wrapping_add(1);
            let cue = match action {
                Action::Hand(_) if carried => Some(Cue::Drop),
                Action::Hand(tile) if game.region().source_at(tile).is_none() => Some(Cue::Gather),
                Action::Cast(spell, _) => Some(Cue::Cast(spell)),
                Action::Build(..) | Action::Upgrade(_) | Action::Road(..) | Action::Repair => {
                    Some(Cue::Build)
                }
                Action::MoveHippo(_)
                | Action::Assign(..)
                | Action::AssignArea(..)
                | Action::RecallHelpers => Some(Cue::Order),
                Action::FinishLocal | Action::Research(_) | Action::Learn(_) => Some(Cue::Reward),
                _ => None,
            };
            if let Some(cue) = cue {
                self.last_sound = Some((cue, self.revision));
            }
            let gesture = match action {
                Action::Hand(_) if carried => Some(PawGesture::Release),
                Action::Hand(_) if game.city().held.is_some() => Some(PawGesture::Grab),
                Action::Cast(..) => Some(PawGesture::Cast),
                Action::Build(..) | Action::MoveHippo(_) | Action::Assign(..) => {
                    Some(PawGesture::Point)
                }
                _ => None,
            };
            if let Some(gesture) = gesture {
                self.paw_gesture = Some((gesture, now));
            }
            if matches!(action, Action::Road(..)) {
                self.road_anchor = None;
            }
            if let Action::Cast(spell, tile) = action {
                self.last_magic = Some((spell, tile, self.revision));
            }
            if previous != game.campaign.active {
                self.tool = Tool::Hand;
                self.close_panels();
                self.option = 0;
                self.road_anchor = None;
                self.last_magic = None;
                self.aim = game.region().sources[0].tile;
                self.notice = game.region().subtitle.clone();
                self.notice_until = now + 8.;
                *view = CameraRig::new(game);
                view.jump = true;
                self.previous_quest = game.city().quest;
            } else if matches!(action, Action::Travel(_)) {
                self.close_panels();
            }
            let scene = match action {
                Action::Travel(4) if first_island_landing => Some(Sequence::Islands),
                Action::Repair if previous == 0 && !game.mode.sandbox() => Some(Sequence::Crossing),
                _ => None,
            };
            if let Some(scene) = scene {
                self.start_story(scene, game, view);
            }
        }
        success
    }

    pub fn observe_shelter(&mut self, game: &Game, view: &mut CameraRig) {
        let quest = game.city().quest;
        if self.story.is_none()
            && game.campaign.active == 0
            && self.previous_quest == Quest::Shelter
            && quest == Quest::Fields
        {
            self.start_story(Sequence::Shelter, game, view);
        }
        self.previous_quest = quest;
    }

    fn start_story(&mut self, sequence: Sequence, game: &Game, view: &mut CameraRig) {
        let story = Story::new(sequence);
        story.frame(game.region(), view);
        self.story = Some(story);
        self.panel_history.clear();
        self.panel = Panel::Story;
        self.focus = Some(UiAction::FinishStory);
        self.pointer = None;
    }

    pub fn finish_story(&mut self, game: &mut Game, view: &mut CameraRig) {
        let Some(story) = self.story.take() else {
            return;
        };
        self.close_panels();
        self.pointer = None;
        if !game.campaign.intro_seen {
            game.campaign.intro_seen = true;
            self.revision = self.revision.wrapping_add(1);
        }
        let region = game.region();
        match story.sequence {
            Sequence::Arrival => {
                self.tool(Tool::Hand);
                self.aim = self.guide_tile(game);
                *view = CameraRig::new(game);
            }
            Sequence::Crossing => {
                self.aim = region.bridge();
                view.focus(self.aim);
            }
            Sequence::Shelter => {
                view.focus(region.start);
            }
            Sequence::Islands => {
                self.tool(Tool::Hand);
                self.aim = self.guide_tile(game);
                view.focus(self.aim);
            }
        }
    }
}

#[derive(Resource)]
pub struct CameraRig {
    pub target: Vec3,
    pub height: f32,
    pub rendered_height: f32,
    pub jump: bool,
    pub cover: f32,
    min: Vec3,
    max: Vec3,
    max_height: f32,
}

impl CameraRig {
    pub fn arm(height: f32) -> Vec3 {
        let near = terrain::smooth((27. - height) / 16.);
        Vec3::new(13., 23., 26.).lerp(Vec3::new(13.8, 14.7, 27.6), near) * (height / 22.).max(1.)
    }

    pub fn new(game: &Game) -> Self {
        let region = game.region();
        Self {
            target: region.start.world() + Vec3::new(-0.95, 0., -4.55),
            height: 22.,
            rendered_height: 22.,
            jump: false,
            cover: 0.,
            min: region.bounds.0.world(),
            max: region.bounds.1.world(),
            max_height: ((region.bounds.1.world() - region.bounds.0.world())
                .dot(Vec3::new(0.35, 0., 0.57))
                + 26.)
                .max(150.),
        }
    }

    pub fn zoom(&mut self, change: f32) {
        self.height = (self.height * (-change).exp()).clamp(11., self.max_height);
    }

    pub fn zoom_toward(&mut self, change: f32, ground: Option<Vec3>) {
        let previous = self.height;
        self.zoom(change);
        if let Some(point) = ground.map(|point| point.with_y(0.)) {
            self.target = (point + (self.target - point) * (self.height / previous))
                .clamp(self.min, self.max);
        }
    }

    pub fn focus(&mut self, tile: Tile) {
        let target = (tile.world() - Vec3::Z).clamp(self.min, self.max);
        self.jump = self.target.distance(target) > 28. || self.rendered_height > 52.;
        self.target = target;
        self.height = self.height.clamp(22., 29.);
    }

    pub fn landscape(&mut self) {
        self.jump = true;
        self.target = (self.min + self.max) * 0.5;
        self.height = self.max_height;
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        let right = Vec3::new(0.89, 0., -0.45);
        let forward = Vec3::new(-0.45, 0., -0.89);
        self.target += right * dx + forward * dy;
        self.target = self.target.clamp(self.min, self.max);
    }
}

#[derive(Resource, Default)]
pub struct Target {
    pub label: String,
    pub detail: String,
    pub legal: bool,
    pub road_preview: Vec<Tile>,
    placement: Option<(PlacementKey, Result<(), String>)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PlacementKey {
    region: usize,
    kind: Building,
    tile: Tile,
    stock: [u16; 3],
    goods: Goods,
    faith: u16,
    quest: Quest,
    count: usize,
    bridge: bool,
    revision: u64,
    walkers: Vec<Tile>,
}

impl Target {
    pub fn refresh(&mut self, play: &Play, game: &Game) {
        let city = game.city();
        let region = game.region();
        let tile = play.aim;
        self.legal = true;
        self.detail.clear();
        self.road_preview.clear();
        self.label = match play.target(game) {
            Action::Build(kind, tile, _) => {
                let key = PlacementKey {
                    region: game.campaign.active,
                    kind,
                    tile,
                    stock: city.stock,
                    goods: city.society.goods,
                    faith: city.faith,
                    quest: city.quest,
                    count: city.buildings.len(),
                    bridge: city.bridge_fixed,
                    revision: play.revision,
                    walkers: city.walkers.iter().map(|walker| walker.tile()).collect(),
                };
                if self
                    .placement
                    .as_ref()
                    .is_none_or(|(previous, _)| *previous != key)
                {
                    self.placement = Some((key, game.building_plan(kind, tile)));
                }
                let result = &self.placement.as_ref().unwrap().1;
                self.legal = result.is_ok();
                self.detail = result.as_ref().err().cloned().unwrap_or_else(|| {
                    if region.sandbox {
                        "Freier Bau".into()
                    } else {
                        kind.cost_text()
                    }
                });
                format!("{} bauen", kind.name())
            }
            Action::Cast(spell, tile) => {
                let error = city.ritual_error(region, spell, tile, game.campaign.gifts);
                self.legal = error.is_none();
                self.detail = error.map(str::to_owned).unwrap_or_else(|| {
                    format!(
                        "{:.0} Wunderkraft · Radius {}",
                        spell.cost(),
                        spell.radius(game.campaign.gifts)
                    )
                });
                format!("{} wirken", spell.name())
            }
            Action::Road(_, end) if play.road_anchor.is_none() => {
                self.legal = city.influence(region, end) && city.walkable(region, end);
                self.detail = "Wähle den Anfang. Verbinde den Weg mit einem Dorfweg.".into();
                "Weg beginnen".into()
            }
            Action::Road(start, end) => {
                match city.road_plan(region, start, end) {
                    Ok((cells, price)) => {
                        self.detail = format!("{} Felder · {}", cells.len(), stock_text(price));
                        self.road_preview = cells;
                    }
                    Err(error) => {
                        self.legal = false;
                        self.detail = error;
                    }
                }
                "Weg bauen".into()
            }
            Action::Upgrade(tile) => {
                self.detail = match city.upgrade_plan(region, tile) {
                    Ok((index, price)) => format!(
                        "{} · {}",
                        stock_text(price),
                        city.buildings[index].kind.use_text()
                    ),
                    Err(error) => {
                        self.legal = false;
                        error
                    }
                };
                city.building_at(tile).map_or_else(
                    || "Ausbauen".into(),
                    |site| format!("{} ausbauen", site.kind.name()),
                )
            }
            Action::Settle(index) => {
                let district = &region.districts[index];
                let mut label = "Beitreten lassen";
                self.detail = match city.district_plan(region, index) {
                    Ok(crate::game::Outcome::Victory) => {
                        label = "Siedlung übernehmen";
                        format!("{} · Wachen vertrieben", district.name)
                    }
                    Ok(crate::game::Outcome::Alliance) => {
                        format!("{} · Gabe {}", district.name, stock_text(district.gift))
                    }
                    Err(error) => {
                        self.legal = false;
                        error
                    }
                };
                label.into()
            }
            Action::Camp(index) => {
                let plan = city.camp_plan(index);
                self.legal = plan.is_ok();
                self.detail = plan.err().map_or_else(
                    || format!("{} · 5 Holz · 6 Glauben", region.areas[index].name),
                    str::to_owned,
                );
                "Sammellager bauen".into()
            }
            Action::Assign(_, hippo) => {
                if !hippo && !city.idle_helper() {
                    self.legal = false;
                    self.detail = "Alle Gläubigen haben Aufträge.".into();
                } else if let Some(index) = region.source_at(tile) {
                    let name = region.kind.material(region.sources[index].material);
                    self.detail = format!("{name} · {} verfügbar", city.nodes[index]);
                } else {
                    self.legal = false;
                    self.detail = "Tippe auf eine Rohstoffquelle.".into();
                }
                if hippo {
                    "Nilpferd losschicken".into()
                } else {
                    "Helfer losschicken".into()
                }
            }
            Action::MoveHippo(tile) => {
                self.legal = city.walkable(region, tile);
                self.detail = if self.legal {
                    "Das Nilpferd läuft zum markierten Platz."
                } else {
                    "Wähle freien Boden für das Nilpferd."
                }
                .into();
                "Hierher".into()
            }
            Action::Watch(_) => {
                self.legal = game.campaign.gifts.training >= 2
                    && city.walkable(region, tile)
                    && city.influence(region, tile);
                self.detail = if game.campaign.gifts.training < 2 {
                    "Die Kreatur lernt das Bewachen auf Rang 2.".into()
                } else {
                    "Wache schützt Vorräte und senkt den Widerstand naher Siedlungen.".into()
                };
                "Wache halten".into()
            }
            Action::Muster(tile) => {
                let plan = city.muster_plan(region, tile);
                self.legal = plan.is_ok();
                self.detail = plan
                    .err()
                    .unwrap_or_else(|| "3 Nahrung · 1 Werkzeug".into());
                "Wache ausbilden".into()
            }
            Action::Patrol(tile) => {
                self.legal = city.garrison() > 0
                    && city.influence(region, tile)
                    && city.walkable(region, tile);
                self.detail = if city.garrison() == 0 {
                    "Bilde zuerst eine Wache im Wachhaus aus."
                } else {
                    "Patrouillen schützen Vorräte und senken den Widerstand in der Nähe."
                }
                .into();
                "Patrouille schicken".into()
            }
            Action::Tend(tile) => {
                self.legal = game.campaign.gifts.training >= 3
                    && city
                        .building_at(tile)
                        .is_some_and(|site| site.kind == Building::Field);
                self.detail = if game.campaign.gifts.training < 3 {
                    "Feldpflege lernt die Kreatur auf Rang 3."
                } else {
                    "Wähle ein Feld zum Pflegen und Ernten."
                }
                .into();
                "Feld pflegen".into()
            }
            Action::Praise => {
                self.legal = city.lesson.is_some();
                self.detail = if self.legal {
                    "Lob stärkt den Kreaturenrang."
                } else {
                    "Lobe die Kreatur nach einer Lieferung, Wache oder Ernte."
                }
                .into();
                "Nilpferd loben".into()
            }
            Action::Feed => {
                self.legal = city.energy < 95. && city.stock[2] >= 2;
                self.detail = "2 Nahrung · stellt bis zu 55 Kraft wieder her".into();
                "Snack geben".into()
            }
            Action::Rest => {
                self.detail = "Beendet den Auftrag. Die Kreatur erholt sich.".into();
                "Pause einlegen".into()
            }
            Action::Repair => {
                self.detail = stock_text(region.rebuild);
                format!("{} reparieren", region.crossing)
            }
            Action::Give => {
                self.detail = format!(
                    "{} {} · {}",
                    region.gift.1,
                    region.kind.material(region.gift.0),
                    region.request
                );
                "Bitte erfüllen".into()
            }
            Action::Hand(_) if city.held.is_some() => {
                self.legal = city.can_store_at(region, tile);
                self.detail = "Ablage am Sammelplatz oder im Lager".into();
                "Ablegen".into()
            }
            Action::Hand(_) if region.source_at(tile).is_some() => {
                let index = region.source_at(tile).unwrap();
                self.legal = city.influence(region, tile) && city.nodes[index] > 0;
                self.detail = format!(
                    "{} · {} verfügbar",
                    region.kind.material(region.sources[index].material),
                    city.nodes[index]
                );
                "Greifen".into()
            }
            Action::Hand(_)
                if city
                    .building_at(tile)
                    .is_some_and(|site| site.kind == Building::Field) =>
            {
                self.legal = city
                    .building_at(tile)
                    .is_some_and(|site| matches!(site.crop, Crop::Ripe));
                self.detail = match city.building_at(tile).unwrap().crop {
                    Crop::Dry => "Trockene Saat".into(),
                    Crop::Growing(seconds) => format!("Wächst · noch {:.0} s", seconds.ceil()),
                    Crop::Ripe => "Die Ernte ist bereit".into(),
                };
                "Ernten".into()
            }
            Action::Hand(_) if city.building_at(tile).is_some() => {
                let site = city.building_at(tile).unwrap();
                self.detail = site.kind.civic().map_or_else(
                    || site.kind.name().to_string(),
                    |kind| {
                        format!(
                            "{} · {}",
                            kind.spec().name,
                            if city.connected(region, site) {
                                "Am Weg"
                            } else {
                                "Ohne Weganschluss"
                            }
                        )
                    },
                );
                "Gebäude ansehen".into()
            }
            _ => {
                self.legal = false;
                self.detail = "Wähle eine Quelle oder ein reifes Feld.".into();
                "Greifen".into()
            }
        };
    }
}
