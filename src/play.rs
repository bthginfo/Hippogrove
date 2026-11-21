#[path = "play_work.rs"]
mod work_guidance;
use bevy::prelude::*;

use crate::{
    civic::{Branch, Civic, Goods, Tech, goods_text},
    game::{
        Action, Building, Crop, FARM_REWARD, FirstDelivery, Game, Miracle, PAW_BUNDLE, PAW_LIMIT,
        Quest, SideQuest, stock_text,
    },
    modes::Mode,
    practice::Step as PracticeStep,
    quests::Goal,
    region::{Biome, Bounds, Material, Tile},
    sound::{Bus, Cue, Mix},
    story::{Sequence, Story},
    terrain,
    wheel::Wheel,
};

pub const CATALOG_PAGE_SIZE: usize = 3;

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
    MainMenu,
    Menu,
    Controls,
    Orders(Tool),
    Spells,
    Journey,
    Gathering,
    Council,
    Village(usize),
    Projects(usize),
    VillageLog,
    Departure,
    Map,
    Area(usize),
    Progress,
    Work,
    Dedication,
    Talent(crate::game::Talent),
    Quests,
    SideQuest,
    Sound,
    Display,
    Catalog,
    Research,
    Inspection,
    Town(usize),
    Household(Tile),
    Inventory,
    FirstDelivery,
    Graduation,
    Briefing,
    Context,
    Story,
    Modes,
    Wheel(Wheel),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAction {
    MenuPage(u8),
    FoldTracker,
    WorkGuide,
    Reading(crate::display::ReadingSize),
    CheckSite,
    Tool(Tool),
    Order(Tool, usize),
    Confirm,
    Cancel,
    MainMenu,
    Resume,
    ToggleWheel,
    QuickWheel(Wheel),
    Rotate,
    Zoom(bool),
    Guide,
    ToolTarget,
    Confront,
    HelpDistrict(usize),
    Panel(Panel),
    Focus(Tile),
    AreaMaterial(Material),
    AreaSource(usize, Material),
    Landscape,
    Catalog(Option<Branch>),
    CatalogPage(usize),
    PreviewBuilding(Building),
    CatalogBuild,
    SelectBuilding(Building),
    Inspect,
    ResearchBranch(Branch),
    SelectTech(Tech),
    Do(Action),
    FinishStory,
    ReplayArrival,
    ModeTab(bool),
    SelectMode(Mode),
    Sound(Bus),
    TestSound,
    EnableSound,
}

#[derive(Clone, Copy, Debug)]
pub enum PawGesture {
    Grab,
    Release,
    Cast,
    Point,
    Pet,
}

#[derive(Clone, Copy, Debug)]
pub struct GrabHold {
    tile: Tile,
    pub(crate) started: f32,
    limit: u16,
}

impl GrabHold {
    pub fn amount(self, now: f32) -> u16 {
        let extra = ((now - self.started - 0.18).max(0.) * 18.) as u16;
        PAW_BUNDLE.saturating_add(extra).min(self.limit)
    }

    pub fn limit(self) -> u16 {
        self.limit
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Feedback {
    #[default]
    Action,
    Selection,
    Quest,
    Blocked,
}

#[derive(Resource)]
pub struct Play {
    pub aim: Tile,
    pub tool: Tool,
    pub panel: Panel,
    panel_history: Vec<(Panel, Option<UiAction>)>,
    pub story: Option<Story>,
    previous_growth_stage: usize,
    previous_quest: Quest,
    previous_side_quest: SideQuest,
    quest_check_at: f32,
    pub free_build_tab: bool,
    pub area_material: Material,
    pub option: usize,
    pub rotation: u8,
    pub road_anchor: Option<Tile>,
    pub catalog_branch: Option<Branch>,
    pub catalog_page: usize,
    pub catalog_preview: Building,
    pub tech: Tech,
    pub focus: Option<UiAction>,
    pub menu_page: u8,
    pub reading: crate::display::ReadingSize,
    pub notice: String,
    pub notice_until: f32,
    pub notice_kind: Feedback,
    pub notice_panel: Panel,
    pub last_magic: Option<(Miracle, Tile, u64)>,
    pub sound_cues: Vec<Cue>,
    pub mix: Mix,
    pub paw_gesture: Option<(PawGesture, f32)>,
    pub gamepad: bool,
    pub tracker_folded: bool,
    pub pointer: Option<Vec2>,
    pub paused: bool,
    pub revision: u64,
    pub grab: Option<GrabHold>,
    pub guide_marker: Option<(Tile, f32)>,
    pub hippo_happy_at: Option<f32>,
    pub celebration: Option<(Tile, bool, f32, crate::game::Rite)>,
    circle_ready: bool,
    contest_event: u32,
    supply_help: Option<Material>,
    hippo_deliveries: u16,
    pub(crate) practice_step: Option<PracticeStep>,
    lesson_tile: Option<Tile>,
    pub object_menu: Option<crate::context::ObjectMenu>,
    pub receipt: Option<(Tile, f32, String)>,
}

impl Play {
    pub fn new(game: &Game) -> Self {
        let story = (!game.campaign.intro_seen && game.campaign.active == 0)
            .then(|| Story::new(Self::arrival(game)));
        let delivery = game.city().first_delivery_ready(game.region());
        Self {
            aim: game.region().sources[0].tile,
            tool: Tool::Hand,
            panel: if story.is_some() {
                Panel::Story
            } else if delivery {
                Panel::FirstDelivery
            } else {
                Panel::Closed
            },
            panel_history: Vec::new(),
            story,
            previous_growth_stage: game.campaign.gifts.creature.stage(),
            previous_quest: game.city().quest,
            previous_side_quest: game.city().side_quest(game.region()),
            quest_check_at: 0.,
            free_build_tab: game.mode.sandbox(),
            area_material: Material::Wood,
            option: 0,
            rotation: 0,
            road_anchor: None,
            catalog_branch: None,
            catalog_page: 0,
            catalog_preview: Building::House,
            tech: Tech::Foundations,
            focus: delivery.then_some(UiAction::Focus(game.region().depot)),
            menu_page: 0,
            reading: Default::default(),
            notice: String::new(),
            notice_until: 0.,
            notice_kind: Feedback::Action,
            notice_panel: Panel::Closed,
            last_magic: None,
            sound_cues: Vec::new(),
            mix: Mix::default(),
            paw_gesture: None,
            gamepad: false,
            tracker_folded: false,
            pointer: None,
            paused: false,
            revision: 0,
            grab: None,
            guide_marker: None,
            hippo_happy_at: None,
            celebration: None,
            circle_ready: false,
            contest_event: game.city().contest.event,
            supply_help: game.city().automatic_supply(game.region()),
            hippo_deliveries: game.city().hippo_deliveries,
            practice_step: None,
            lesson_tile: None,
            object_menu: None,
            receipt: None,
        }
    }

    fn arrival(game: &Game) -> Sequence {
        if game.city().opening_farm(game.region()).is_some() {
            Sequence::FirstDelivery
        } else if game.region().kind == Biome::Aegean {
            Sequence::Islands
        } else {
            Sequence::Arrival
        }
    }

    pub fn building(&self) -> Building {
        Building::from_index(self.option.min(Building::CATALOG_SIZE - 1))
    }

    pub fn catalog_entries(&self, game: &Game) -> Vec<Building> {
        match self.catalog_branch {
            None => Building::ALL.to_vec(),
            Some(branch) => Civic::ALL
                .into_iter()
                .filter(|kind| {
                    kind.spec().branch == branch && kind.available_in(game.region().kind)
                })
                .map(Building::Civic)
                .collect(),
        }
    }

    pub fn set_focus(&mut self, action: Option<UiAction>) {
        if self.panel != Panel::Closed && action.is_some() && self.focus != action {
            self.sound_cues.push(Cue::Focus);
        }
        if let Some(UiAction::PreviewBuilding(kind)) = action {
            self.catalog_preview = kind;
        }
        self.focus = action;
    }

    fn notify(&mut self, message: impl Into<String>, kind: Feedback, expires: f32) {
        self.notice = message.into();
        self.notice_kind = kind;
        self.notice_panel = self.panel;
        self.notice_until = expires;
        if kind == Feedback::Blocked {
            self.sound_cues.push(Cue::Blocked);
        }
    }

    fn set_catalog_page(&mut self, game: &Game, index: usize) {
        let entries = self.catalog_entries(game);
        let pages = entries.len().div_ceil(CATALOG_PAGE_SIZE);
        self.catalog_page = index.min(pages.saturating_sub(1));
        self.set_focus(
            entries
                .get(self.catalog_page * CATALOG_PAGE_SIZE)
                .copied()
                .map(UiAction::PreviewBuilding),
        );
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
                    && city
                        .gatherings
                        .circle
                        .as_ref()
                        .is_some_and(|c| c.center == self.aim) =>
            {
                Action::Invoke(false)
            }
            Tool::Hand
                if city.held.is_none()
                    && self.aim.within(region.bridge(), 1)
                    && city.quest == Quest::Bridge
                    && !city.gift_given
                    && city.count(Building::Shrine) > 0 =>
            {
                if city.bridge_fixed && !city.gift_given {
                    Action::Give
                } else {
                    Action::Repair
                }
            }
            Tool::Hand if !region.tutorial && city.held.is_none() && self.aim == region.depot => {
                Action::SeeWork
            }
            Tool::Hand
                if city.held.is_none()
                    && let Some(i) = region.village_at(self.aim)
                    && (i >= region.districts.len() || city.outcome.is_none()) =>
            {
                Action::Audience(i)
            }
            Tool::Hand if city.held.is_none() && region.district_at(self.aim).is_some() => {
                let index = region.district_at(self.aim).unwrap();
                if city.has_trading_partner(region, index) {
                    Action::Trade(index, false)
                } else {
                    Action::Settle(index)
                }
            }
            Tool::Hand
                if city.held.is_none()
                    && region
                        .area_at(self.aim)
                        .is_some_and(|index| !city.camps[index]) =>
            {
                Action::Camp(region.area_at(self.aim).unwrap())
            }
            Tool::Hand if city.held.is_none() && self.aim == city.walkers[0].tile() => Action::Pet,
            Tool::Hand
                if city.held.is_none()
                    && city.building_at(self.aim).is_some_and(|site| {
                        site.kind.civic().and_then(Civic::production).is_some()
                    }) =>
            {
                Action::Produce(self.aim)
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

    pub fn start_grab(&mut self, game: &Game, now: f32) -> bool {
        if self.panel != Panel::Closed
            || self.grab.is_some()
            || self
                .practice_step
                .is_some_and(|step| step.next_action(self, game) != UiAction::Confirm)
            || self.target(game) != Action::Hand(self.aim)
        {
            return false;
        }
        let limit = game.city().paw_limit(game.region(), self.aim);
        if limit == 0 {
            return false;
        }
        self.grab = Some(GrabHold {
            tile: self.aim,
            started: now,
            limit,
        });
        true
    }

    pub fn finish_grab(&mut self, game: &mut Game, view: &mut CameraRig, now: f32) -> bool {
        let Some(hold) = self.grab.take() else {
            return false;
        };
        if self.panel != Panel::Closed || self.target(game) != Action::Hand(hold.tile) {
            return false;
        }
        self.execute(Action::Grab(hold.tile, hold.amount(now)), game, view, now)
    }

    fn guide_tile(&self, game: &Game) -> Tile {
        let city = game.city();
        let region = game.region();
        match self.tool {
            Tool::Build if self.building() == Building::Shrine => region.frontier,
            Tool::Build => {
                let origin = if city.influence(region, self.aim) {
                    self.aim
                } else {
                    region.start
                };
                Self::building_tile(game, self.building(), origin, false)
            }
            Tool::Plan if self.option == 1 => city
                .buildings
                .iter()
                .filter(|site| site.kind.upgrade().is_some() && !site.upgraded)
                .min_by_key(|site| (!city.connected(region, site), self.aim.distance(site.tile)))
                .map_or(self.aim, |site| site.tile),
            Tool::Helpers if self.option == 1 => city
                .buildings
                .iter()
                .find(|site| site.kind == Building::Civic(Civic::Barracks))
                .map_or(region.start, |site| site.tile),
            Tool::Hand if city.held.is_some() => city.nearest_store(region, self.aim),
            Tool::Hippo if self.option == 6 => region.start,
            Tool::Hippo if self.option == 1 && city.quest == Quest::Bridge => {
                region.bridge().offset(0, 1)
            }
            Tool::Hippo if self.option == 2 => city
                .buildings
                .iter()
                .filter(|site| site.kind == Building::Field)
                .min_by_key(|site| self.aim.distance(site.tile))
                .map_or(region.start, |site| site.tile),
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
            Tool::Miracles
                if self.miracle().offensive()
                    && city.quest == Quest::Bridge
                    && city.rival_active(region) =>
            {
                city.rival_target(region, self.miracle().radius(game.campaign.gifts))
                    .unwrap_or(region.bridge())
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
            _ => region.sources[..region.managed_sources]
                .iter()
                .zip(&city.nodes)
                .find(|(source, remaining)| **remaining > 0 && city.influence(region, source.tile))
                .map(|(source, _)| source.tile)
                .unwrap_or(region.start),
        }
    }

    fn guide_cost(&mut self, game: &Game, price: crate::game::Stock) -> Option<Tile> {
        let city = game.city();
        let region = game.region();
        let missing = Material::ALL
            .into_iter()
            .find(|m| city.stock[m.index()] < price[m.index()])?;
        self.tool(Tool::Hand);
        if let Some(bundle) = city.held {
            let m = region.sources[bundle.source].material;
            return Some(if city.stock[m.index()] < city.storage_cap() {
                city.nearest_store(region, self.aim)
            } else {
                region.sources[bundle.source].tile
            });
        }
        region.sources[..region.managed_sources]
            .iter()
            .zip(&city.nodes)
            .filter(|(source, left)| {
                source.material == missing && **left > 0 && city.influence(region, source.tile)
            })
            .min_by_key(|(source, _)| source.tile.distance(self.aim))
            .map(|(source, _)| source.tile)
    }

    fn guide_district(&mut self, game: &Game, index: usize) -> Tile {
        let city = game.city();
        let region = game.region();
        let district = &region.districts[index];
        let center = district.center;
        let kind = district.need.building();
        self.tool(Tool::Hand);
        if city.districts[index].resolve == 0 || city.districts[index].outcome.is_some() {
            return center;
        }
        let local = city.district_site(region, index);
        let upgrade = district.need == crate::region::DistrictNeed::Shelter;
        if city.gatherings.welcomed[index] || local.is_some_and(|site| !upgrade || site.upgraded) {
            return self.guide_cost(game, district.gift).unwrap_or(center);
        }
        if city.city_rank() < 2 {
            let prerequisite = if city.count(Building::Store) == 0 {
                Building::Store
            } else {
                Building::Field
            };
            return self.guide_building(game, prerequisite, false);
        }
        if kind != Building::Well && city.city_rank() < 3 {
            return self.guide_building(game, Building::Workshop, false);
        }
        if let Some(site) = local {
            let cost = kind.upgrade().expect("district shelter upgrade");
            if city.inhabitants() < cost.believers {
                return self.guide_building(game, Building::House, false);
            }
            if let Some(tile) = self.guide_cost(game, cost.materials) {
                return tile;
            }
            self.tool(Tool::Plan);
            self.choose(1);
            return site.tile;
        }
        if let Some(tile) = self.guide_cost(game, kind.price()) {
            return tile;
        }
        self.tool(Tool::Build);
        self.choose(kind.index());
        Self::nearby_building_tile(game, kind, center, true, None).unwrap_or(center)
    }

    fn guide_building(&mut self, game: &Game, kind: Building, roadside: bool) -> Tile {
        let region = game.region();
        if let Some(civic) = kind.civic() {
            let city = game.city();
            let producer = [Civic::Sawmill, Civic::Masonry, Civic::Forge];
            if let Some(i) = (0..3).find(|&i| city.society.goods[i] < civic.spec().goods[i]) {
                let industry = Building::Civic(producer[i]);
                if city.count(industry) == 0 && kind != industry {
                    return self.guide_building(game, industry, true);
                }
                if let Some(site) = city
                    .buildings
                    .iter()
                    .find(|s| s.kind == industry && city.connected(game.region(), s))
                {
                    let price = match producer[i] {
                        Civic::Sawmill => [2, 0, 0],
                        Civic::Masonry => [0, 2, 0],
                        _ => [0, 1, 0],
                    };
                    if let Some(tile) = self.guide_cost(game, price) {
                        return tile;
                    }
                    self.tool(Tool::Hand);
                    return site.tile;
                }
            }
        }
        if let Some(tile) = self.guide_cost(game, kind.price()) {
            return tile;
        }
        self.tool(Tool::Build);
        self.choose(kind.index());
        if kind == Building::Shrine {
            region.frontier
        } else {
            Self::building_tile(game, kind, region.start, roadside)
        }
    }

    fn building_tile(game: &Game, kind: Building, origin: Tile, roadside: bool) -> Tile {
        Self::nearby_building_tile(game, kind, origin, roadside, None).unwrap_or(origin)
    }

    fn nearby_building_tile(
        game: &Game,
        kind: Building,
        origin: Tile,
        roadside: bool,
        skip: Option<Tile>,
    ) -> Option<Tile> {
        let region = game.region();
        let city = game.city();
        let mut nearby: Vec<_> = Bounds(origin.offset(-8, -8), origin.offset(8, 8))
            .tiles()
            .filter(|tile| {
                let radius = if region.district_at(origin).is_some() {
                    crate::game::DISTRICT_RADIUS
                } else {
                    8
                };
                Some(*tile) != skip && region.bounds.contains(*tile) && tile.within(origin, radius)
            })
            .map(|tile| {
                let disconnected = roadside
                    && !kind
                        .tiles(tile)
                        .flat_map(Tile::neighbors)
                        .any(|cell| city.road_at(region, cell) && city.walkable(region, cell));
                ((disconnected, tile.distance(origin), tile.1, tile.0), tile)
            })
            .collect();
        nearby.sort_unstable_by_key(|&(rank, _)| rank);
        nearby
            .into_iter()
            .map(|(_, tile)| tile)
            .find(|&tile| city.placement_site(region, kind, tile))
    }

    fn guide_quest(&mut self, game: &Game) -> Option<Tile> {
        if game.mode == Mode::Tutorial {
            return self.lesson_target(game);
        }
        let city = game.city();
        let region = game.region();
        let main = matches!(self.panel, Panel::Closed | Panel::Quests)
            && !region.sandbox
            && city.quest != Quest::Complete;
        let step = city.quest_step(region);
        let pending = match self.panel {
            Panel::SideQuest => city.local_next(region),
            _ if main => step.and_then(|step| step.pending(city, region)),
            _ => return None,
        };
        if main && city.quest == Quest::Shelter && city.held.is_some() {
            self.tool(Tool::Hand);
            return Some(city.nearest_store(region, self.aim));
        }
        let supply = |material| {
            region.sources[..region.managed_sources]
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
        let goal = if let Some(goal) = pending {
            goal
        } else if main {
            match city.quest {
                Quest::Shelter
                    if city.hand_deliveries == 0 || city.waiting_for_first_delivery() =>
                {
                    let tile = supply(Material::Wood);
                    self.tool(Tool::Hand);
                    return Some(tile);
                }
                Quest::Shelter if city.count(Building::House) == 0 => {
                    return Some(self.guide_building(game, Building::House, false));
                }
                Quest::Shelter if city.count(Building::Store) == 0 => {
                    return Some(self.guide_building(game, Building::Store, false));
                }
                Quest::Shelter => Goal::HippoDeliveries { count: 1 },
                Quest::Fields if step.is_none() => Goal::Harvests { count: 1 },
                Quest::Bridge if city.count(Building::Shrine) == 0 => {
                    return Some(self.guide_building(game, Building::Shrine, false));
                }
                Quest::Bridge
                    if self.panel == Panel::Quests
                        || !self.miracle().offensive()
                        || self.tool != Tool::Miracles =>
                {
                    if city.gift_given {
                        let kind = [
                            Building::Field,
                            Building::Workshop,
                            Building::Well,
                            Building::House,
                        ]
                        .into_iter()
                        .find(|&kind| city.count(kind) == 0)
                        .unwrap_or(Building::House);
                        return Some(self.guide_building(game, kind, false));
                    }
                    let due = city.crossing_due(region);
                    let tile = if city.held.is_some() {
                        city.carry_destination(region, self.aim)
                            .unwrap_or(region.depot)
                    } else if let Some(material) = Material::ALL
                        .into_iter()
                        .find(|m| due[m.index()] > city.stock[m.index()])
                    {
                        supply(material)
                    } else {
                        region.bridge()
                    };
                    self.tool(Tool::Hand);
                    return Some(tile);
                }
                Quest::Districts => {
                    self.tool(Tool::Hand);
                    return Some(self.guide_tile(game));
                }
                _ => return None,
            }
        } else {
            return None;
        };
        let tile = match goal {
            Goal::Harvests { .. } => {
                let field = city
                    .buildings
                    .iter()
                    .filter(|site| site.kind == Building::Field)
                    .min_by_key(|site| {
                        (
                            match site.crop {
                                Crop::Ripe => 0,
                                Crop::Growing(_) => 1,
                                Crop::Dry => 2,
                            },
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
            Goal::Roads { count } => {
                if let Some(tile) = self.guide_cost(game, [0, 1, 0]) {
                    return Some(tile);
                }
                let length = (count.saturating_sub(city.roads.len() as u16)).clamp(1, 3) as i16;
                let mut starts: Vec<_> =
                    Bounds(region.start.offset(-8, -8), region.start.offset(8, 8))
                        .tiles()
                        .filter(|&t| city.road_at(region, t) && city.walkable(region, t))
                        .collect();
                starts.sort_by_key(|t| t.distance(self.aim));
                self.tool(Tool::Plan);
                self.choose(0);
                for n in (1..=length).rev() {
                    for start in &starts {
                        for end in [
                            start.offset(n, 0),
                            start.offset(-n, 0),
                            start.offset(0, n),
                            start.offset(0, -n),
                        ] {
                            if city
                                .road_plan(region, *start, end)
                                .is_ok_and(|(cells, _)| cells.len() == n as usize)
                            {
                                self.road_anchor = Some(*start);
                                return Some(end);
                            }
                        }
                    }
                }
                self.aim
            }
            Goal::Camps { .. } => {
                if let Some(tile) = self.guide_cost(game, [crate::game::CAMP_WOOD, 0, 0]) {
                    return Some(tile);
                }
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
                        if let Some(upgrade) = kind.upgrade() {
                            if city.inhabitants() < upgrade.believers {
                                return Some(self.guide_building(game, Building::House, false));
                            }
                            if let Some(tile) = self.guide_cost(game, upgrade.materials) {
                                return Some(tile);
                            }
                        }
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

    pub fn lesson_target(&self, game: &Game) -> Option<Tile> {
        if matches!(
            self.practice_step,
            Some(PracticeStep::Delivery | PracticeStep::Pet)
        ) {
            Some(game.city().walkers[0].tile())
        } else {
            self.lesson_tile
        }
    }

    pub fn at_title(&self) -> bool {
        self.panel == Panel::MainMenu
            || self
                .panel_history
                .iter()
                .any(|(panel, _)| *panel == Panel::MainMenu)
    }

    pub fn observe_practice(&mut self, game: &Game, view: &mut CameraRig, now: f32) {
        if game.mode != Mode::Tutorial || self.panel != Panel::Closed {
            return;
        }
        let step = PracticeStep::current(game.city());
        if step == PracticeStep::Complete
            && self
                .paw_gesture
                .is_some_and(|(gesture, at)| matches!(gesture, PawGesture::Pet) && now - at < 0.95)
        {
            return;
        }
        let previous = self.practice_step.replace(step);
        if previous == Some(step) {
            return;
        }
        if step == PracticeStep::Complete {
            self.revision = self.revision.wrapping_add(1);
            self.sound_cues.push(Cue::Reward);
            self.show_main_menu(game);
            self.free_build_tab = false;
            self.open_panel(Panel::Graduation);
            self.focus = Some(UiAction::SelectMode(Mode::Campaign(Biome::Valley)));
        } else {
            self.tool(Tool::Hand);
            self.aim = step.tile(game);
            self.lesson_tile = Some(self.aim);
            self.guide_marker = Some((self.aim, now + 6.));
            self.pointer = None;
            view.focus_action(self.aim);
        }
    }

    pub fn inspect_world(&mut self, game: &Game, now: f32) {
        if self.panel != Panel::Closed {
            return;
        }
        if self.tool == Tool::Hand
            && game.city().held.is_none()
            && let Some(i) = game.region().village_at(self.aim)
            && (i >= game.region().districts.len() || game.city().outcome.is_none())
        {
            self.open_panel(Panel::Village(i));
            self.focus = Some(if let Some(w) = game.city().visitor(i, true) {
                UiAction::Focus(w.tile())
            } else {
                UiAction::Do(Action::Visit(i, true))
            });
            return;
        }
        let menu = crate::context::ObjectMenu::at(self, game, now);
        self.open_panel(Panel::Context);
        self.focus = menu.choices.first().map(|choice| choice.action);
        self.object_menu = Some(menu);
        self.sound_cues.push(Cue::Focus);
    }

    pub fn activate(
        &mut self,
        action: UiAction,
        game: &mut Game,
        view: &mut CameraRig,
        now: f32,
    ) -> bool {
        self.grab = None;
        if self.panel == Panel::Context && action != UiAction::Cancel {
            self.close_panels();
        }
        let before = self.panel;
        let at_title = self.at_title();
        let action = if action == UiAction::Confirm && self.panel == Panel::Closed {
            self.practice_step
                .map_or(action, |step| step.next_action(self, game))
        } else {
            action
        };
        let action = if action == UiAction::Panel(Panel::Menu) && (at_title || self.story.is_some())
        {
            UiAction::MainMenu
        } else {
            action
        };
        if at_title
            && !matches!(
                action,
                UiAction::MainMenu
                    | UiAction::Cancel
                    | UiAction::ModeTab(_)
                    | UiAction::SelectMode(_)
                    | UiAction::Panel(Panel::Sound | Panel::Modes | Panel::Display)
                    | UiAction::Reading(_)
                    | UiAction::Sound(_)
                    | UiAction::TestSound
                    | UiAction::EnableSound
            )
        {
            return false;
        }
        if self.story.is_some() && action != UiAction::MainMenu {
            if matches!(
                action,
                UiAction::FinishStory | UiAction::Cancel | UiAction::Confirm
            ) {
                self.finish_story(game, view);
            }
            return false;
        }
        match action {
            UiAction::ToggleWheel => match self.panel {
                Panel::Closed => self.open_panel(Panel::Wheel(Wheel::Tools)),
                Panel::Wheel(_) => self.close_panels(),
                _ => {}
            },
            UiAction::QuickWheel(wheel) => {
                if matches!(self.panel, Panel::Closed | Panel::Wheel(_)) {
                    let same = self.panel == Panel::Wheel(wheel);
                    self.close_panels();
                    if !same {
                        self.open_panel(Panel::Wheel(wheel));
                    }
                }
            }
            UiAction::Tool(Tool::Hand) => {
                self.tool(Tool::Hand);
                self.close_panels();
            }
            UiAction::Tool(tool) => {
                self.open_tool(tool);
                if self.panel == Panel::Catalog {
                    self.set_catalog_page(game, self.catalog_page);
                }
                if self.panel == Panel::Orders(Tool::Hippo)
                    && game
                        .city()
                        .supply_lesson(game.region())
                        .is_some_and(|m| !game.campaign.gifts.supplies(m))
                {
                    self.focus = Some(UiAction::Do(Action::Praise));
                }
            }
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
                    self.notify(reason, Feedback::Blocked, now + 4.);
                    return false;
                }
                let assist = matches!(self.panel, Panel::Wheel(_))
                    && match (tool, index) {
                        (Tool::Hippo | Tool::Helpers, 0) => {
                            game.city().harvest_at(game.region(), self.aim).is_none()
                        }
                        (Tool::Miracles, index) if Miracle::ALL[index] == Miracle::Rain => {
                            !game.city().building_at(self.aim).is_some_and(|site| {
                                site.kind == Building::Field && matches!(site.crop, Crop::Dry)
                            })
                        }
                        (Tool::Plan, 1) => game
                            .city()
                            .building_at(self.aim)
                            .is_none_or(|site| site.upgraded || site.kind.upgrade().is_none()),
                        _ => false,
                    };
                self.tool(tool);
                self.choose(index);
                self.close_panels();
                if assist {
                    self.focus_tool_target(game, view, now);
                }
            }
            UiAction::Catalog(branch) => {
                if self.panel != Panel::Catalog {
                    self.open_panel(Panel::Catalog);
                }
                self.catalog_branch = branch;
                self.set_catalog_page(game, 0);
            }
            UiAction::CatalogPage(index) => self.set_catalog_page(game, index),
            UiAction::PreviewBuilding(kind) if !self.gamepad => {
                self.set_focus(Some(UiAction::PreviewBuilding(kind)));
            }
            UiAction::PreviewBuilding(kind) | UiAction::SelectBuilding(kind) => {
                self.select_blueprint(kind, game, view, now);
            }
            UiAction::CatalogBuild => self.select_blueprint(self.catalog_preview, game, view, now),
            UiAction::Inspect => {
                if self.panel == Panel::Closed {
                    self.inspect(game);
                }
            }
            UiAction::SelectTech(tech) => {
                self.tech = tech;
                self.panel = Panel::Research;
                self.focus = Some(
                    if self.gamepad
                        && game
                            .campaign
                            .research
                            .available(tech, game.city().city_rank())
                            .is_ok()
                    {
                        UiAction::Do(Action::Research(tech))
                    } else {
                        UiAction::SelectTech(tech)
                    },
                );
            }
            UiAction::ResearchBranch(branch) => {
                self.tech = branch.topics()[0];
                self.panel = Panel::Research;
                self.focus = Some(UiAction::SelectTech(self.tech));
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
                    self.sound_cues.push(Cue::Choose);
                } else {
                    self.notify(
                        "Beginne auf freiem Boden im Einflussgebiet.",
                        Feedback::Blocked,
                        now + 4.,
                    );
                }
            }
            UiAction::Confirm if self.panel == Panel::Closed => {
                if self.tool == Tool::Plan && self.option == 1 && self.inspect(game) {
                    self.sound_cues.push(Cue::Choose);
                    return false;
                }
                if self.target(game) == Action::Hand(self.aim)
                    && game.city().held.is_none()
                    && game
                        .city()
                        .building_at(self.aim)
                        .is_some_and(|site| site.kind != Building::Field)
                {
                    self.inspect(game);
                    self.sound_cues.push(Cue::Choose);
                    return false;
                }
                return self.execute(self.target(game), game, view, now);
            }
            UiAction::Confirm => {}
            UiAction::Cancel => {
                if self.panel == Panel::MainMenu {
                    return false;
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
                } else if self.tool != Tool::Hand {
                    self.tool(Tool::Hand);
                } else {
                    self.open_panel(Panel::Menu);
                }
            }
            UiAction::CheckSite => self.open_panel(Panel::Household(self.aim)),
            UiAction::Panel(Panel::Work) => self.open_work(game),
            UiAction::WorkGuide => self.guide_work(game, view, now),
            UiAction::Guide if self.panel == Panel::Closed && game.city().work.pinned => {
                self.guide_work(game, view, now)
            }
            UiAction::Panel(Panel::Talent(t)) if matches!(self.panel, Panel::Talent(_)) => {
                self.panel = Panel::Talent(t);
                self.focus = Some(UiAction::Panel(Panel::Talent(t)));
            }
            UiAction::FoldTracker => {
                self.tracker_folded = !self.tracker_folded;
            }
            UiAction::MenuPage(page) => {
                self.menu_page = page;
                self.set_focus(Some(UiAction::MenuPage(page)));
            }
            UiAction::MainMenu => self.show_main_menu(game),
            UiAction::Resume => {
                self.close_panels();
                if game.city().first_delivery_ready(game.region()) {
                    self.open_delivery(game);
                }
            }
            UiAction::Rotate if self.tool == Tool::Build && self.panel == Panel::Closed => {
                self.rotation = (self.rotation + 1) % 4;
                self.sound_cues.push(Cue::Focus);
            }
            UiAction::Rotate => {}
            UiAction::Zoom(closer) => {
                view.zoom_toward(if closer { 0.24 } else { -0.24 }, Some(self.aim.world()))
            }
            UiAction::ToolTarget if self.panel == Panel::Closed && self.tool == Tool::Build => {
                self.suggest_plot(game, view, now);
            }
            UiAction::HelpDistrict(index) => {
                if index < game.region().districts.len() && game.city().outcome.is_some() {
                    let tile = self.guide_district(game, index);
                    self.close_panels();
                    self.pointer = None;
                    self.aim = tile;
                    view.focus_action(tile);
                    self.guide_marker = Some((tile, now + 6.));
                    self.sound_cues.push(Cue::Choose);
                }
            }
            UiAction::Confront => {
                self.close_panels();
                self.tool(Tool::Miracles);
                self.choose(Miracle::Wind as usize);
                if game.city().quest == Quest::Bridge {
                    self.aim = self.guide_tile(game);
                }
                self.pointer = None;
                view.focus_action(self.aim);
                self.guide_marker = Some((self.aim, now + 6.));
                self.notify(
                    "Windstoß ist bereit. Bestätige am markierten Gegner.",
                    Feedback::Selection,
                    now + 4.,
                );
            }
            UiAction::Guide | UiAction::ToolTarget => {
                let city = game.city();
                if matches!(action, UiAction::Guide)
                    && matches!(self.panel, Panel::Closed | Panel::Quests)
                    && !game.region().tutorial
                    && !game.region().sandbox
                {
                    if city.quest == Quest::Complete {
                        self.open_departure(game);
                        return false;
                    }
                    if city.quest == Quest::Districts
                        && city
                            .quest_step(game.region())
                            .and_then(|step| step.pending(city, game.region()))
                            .is_none()
                    {
                        if let Some(index) = city
                            .districts
                            .iter()
                            .position(|state| state.outcome.is_none())
                        {
                            self.aim = self.guide_district(game, index);
                            self.close_panels();
                            self.pointer = None;
                            view.focus_action(self.aim);
                            self.guide_marker = Some((self.aim, now + 6.));
                            self.sound_cues.push(Cue::Choose);
                            return false;
                        }
                    }
                }
                let tool_target = action == UiAction::ToolTarget
                    && self.panel == Panel::Closed
                    && self.tool != Tool::Hand
                    && game.mode != Mode::Tutorial;
                if !tool_target
                    && matches!(self.panel, Panel::Closed | Panel::Quests | Panel::SideQuest)
                {
                    let goal = if self.panel == Panel::SideQuest {
                        city.local_next(game.region())
                    } else {
                        city.quest_step(game.region())
                            .and_then(|step| step.pending(city, game.region()))
                    };
                    let civic = match goal {
                        Some(Goal::Building {
                            kind: Building::Civic(kind),
                            ..
                        }) => Some(kind),
                        Some(Goal::OlivePressings { .. }) => Some(Civic::OlivePress),
                        _ => None,
                    };
                    let civic = civic.map(|kind| {
                        let producer = [Civic::Sawmill, Civic::Masonry, Civic::Forge];
                        let needed = (0..3).find(|&i| {
                            city.society.goods[i] < kind.spec().goods[i]
                                && city.count(Building::Civic(producer[i])) == 0
                        });
                        if game.campaign.research.knows(kind.spec().unlock) {
                            needed.map_or(kind, |i| producer[i])
                        } else {
                            kind
                        }
                    });
                    if let Some(mut tech) = civic
                        .map(|kind| kind.spec().unlock)
                        .filter(|&tech| !game.campaign.research.knows(tech))
                    {
                        while let Some(&parent) = tech
                            .parents()
                            .iter()
                            .find(|&&p| !game.campaign.research.knows(p))
                        {
                            tech = parent;
                        }
                        self.open_panel(Panel::Research);
                        self.tech = tech;
                        let readiness = game.campaign.research.available(tech, city.city_rank());
                        self.focus = Some(if readiness.is_ok() {
                            UiAction::Do(Action::Research(tech))
                        } else {
                            UiAction::SelectTech(tech)
                        });
                        self.notify(
                            readiness.err().unwrap_or_else(|| {
                                format!("{} für den Bauplan erforschen", tech.name())
                            }),
                            Feedback::Selection,
                            now + 5.,
                        );
                        return false;
                    }
                }
                if !tool_target
                    && matches!(
                        self.panel,
                        Panel::Closed | Panel::Quests | Panel::FirstDelivery
                    )
                    && game.city().first_delivery_ready(game.region())
                {
                    if self.panel != Panel::FirstDelivery {
                        self.open_delivery(game);
                    }
                    return false;
                }
                self.aim = if tool_target {
                    self.guide_tile(game)
                } else {
                    self.guide_quest(game)
                        .unwrap_or_else(|| self.guide_tile(game))
                };
                self.close_panels();
                view.focus_action(self.aim);
                self.pointer = None;
                self.guide_marker = Some((self.aim, now + 6.));
                self.sound_cues.push(Cue::Choose);
            }
            UiAction::Focus(tile) => {
                if self.panel == Panel::FirstDelivery {
                    self.notice_until = now;
                }
                let show_work = self.panel == Panel::Work && tile == game.region().depot;
                self.focus_tile(tile, view);
                if show_work {
                    view.frame_work(tile);
                }
            }
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
            UiAction::Panel(panel @ (Panel::Council | Panel::Village(_))) => {
                if game.region().villages.is_empty() {
                    self.notify(
                        "Auf den Kampagneninseln warten die Nachbardörfer.",
                        Feedback::Selection,
                        now + 4.,
                    );
                } else {
                    self.open_panel(panel);
                    if self.panel == panel {
                        self.focus = Some(match panel {
                            Panel::Village(i) if game.city().visitor(i, true).is_some() => {
                                if game.city().gift_ready(game.region(), i).is_ok() {
                                    UiAction::Do(Action::VillageGift(i))
                                } else if game.city().visitor(i, false).is_none() {
                                    UiAction::Do(Action::Visit(i, false))
                                } else {
                                    UiAction::Focus(game.region().villages[i].center)
                                }
                            }
                            Panel::Village(i) => UiAction::Do(Action::Visit(i, true)),
                            _ => UiAction::Panel(Panel::Village(
                                game.city().nearest_village(game.region()),
                            )),
                        });
                    }
                }
            }
            UiAction::Panel(Panel::Gathering) if game.region().tutorial => {
                self.notify(
                    "Auf den großen Inseln kannst du dein Dorf zusammenrufen.",
                    Feedback::Selection,
                    now + 4.,
                );
            }
            UiAction::Panel(Panel::Gathering) => {
                self.open_panel(Panel::Gathering);
                if self.panel == Panel::Gathering {
                    let city = game.city();
                    self.focus = Some(if let Some(c) = &city.gatherings.circle {
                        if c.ready(city) {
                            UiAction::Do(Action::Invoke(false))
                        } else {
                            UiAction::Focus(c.center)
                        }
                    } else if city.gatherings.rest > 0. {
                        UiAction::Resume
                    } else {
                        let kind = crate::game::Rite::ALL
                            .into_iter()
                            .find(|&kind| {
                                city.gathering_place(game.region(), kind, self.aim).is_ok()
                            })
                            .unwrap_or(crate::game::Rite::Council);
                        UiAction::Do(Action::Call(kind, self.aim))
                    });
                }
            }
            UiAction::Panel(panel) => {
                self.open_panel(panel);
                if self.panel == Panel::Catalog && panel == Panel::Catalog {
                    self.set_catalog_page(game, self.catalog_page);
                }
                if self.panel == Panel::SideQuest && panel == Panel::SideQuest {
                    self.focus = Some(match game.city().side_quest(game.region()) {
                        SideQuest::Available => UiAction::Do(Action::AcceptLocal),
                        SideQuest::Ready => UiAction::Do(Action::FinishLocal),
                        SideQuest::Active => UiAction::Guide,
                        _ => UiAction::Cancel,
                    });
                }
            }
            UiAction::ReplayArrival if game.campaign.active == 0 => {
                self.start_story(Self::arrival(game), game, view);
            }
            UiAction::ReplayArrival | UiAction::FinishStory => {}
            UiAction::ModeTab(sandbox) => {
                self.free_build_tab = sandbox;
                if self.panel != Panel::Modes {
                    self.open_panel(Panel::Modes);
                    self.focus = Some(UiAction::SelectMode(if sandbox {
                        Mode::Sandbox(Biome::Valley)
                    } else {
                        Mode::Campaign(Biome::Valley)
                    }));
                }
            }
            UiAction::SelectMode(mode) => {
                self.choose_mode(mode, game, view, now);
            }
            UiAction::Reading(size) => {
                self.reading = size;
                self.set_focus(Some(action));
            }
            UiAction::Sound(bus) => self.mix.toggle(bus),
            UiAction::TestSound => {
                self.sound_cues.push(Cue::Test);
            }
            UiAction::EnableSound => self.mix = Mix::default(),
            UiAction::Do(action) => return self.execute(action, game, view, now),
        }
        if action == UiAction::Cancel
            || (matches!(before, Panel::Wheel(_))
                && self.panel == Panel::Closed
                && matches!(action, UiAction::ToggleWheel | UiAction::QuickWheel(_)))
        {
            self.sound_cues.push(Cue::Back);
        } else if self.panel != before
            || matches!(
                action,
                UiAction::Catalog(_)
                    | UiAction::CatalogPage(_)
                    | UiAction::ResearchBranch(_)
                    | UiAction::SelectTech(_)
                    | UiAction::Sound(_)
                    | UiAction::EnableSound
            )
        {
            self.sound_cues.push(Cue::Choose);
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

    fn select_blueprint(&mut self, kind: Building, game: &Game, view: &mut CameraRig, now: f32) {
        if let Some(civic) = kind.civic() {
            let tech = civic.spec().unlock;
            if !game.campaign.research.knows(tech) {
                self.tech = tech;
                self.open_panel(Panel::Research);
                return;
            }
        }
        let assist = matches!(self.panel, Panel::Catalog | Panel::Wheel(_))
            && !game.city().placement_site(game.region(), kind, self.aim);
        if self.tool != Tool::Build || self.building() != kind {
            self.rotation = 0;
        }
        self.tool(Tool::Build);
        self.choose(kind.index());
        self.close_panels();
        self.pointer = None;
        if assist {
            self.focus_tool_target(game, view, now);
        }
    }

    fn suggest_plot(&mut self, game: &Game, view: &mut CameraRig, now: f32) {
        let kind = self.building();
        let region = game.region();
        let tile = if kind == Building::Shrine {
            Some(region.frontier)
        } else {
            let origin = if game.city().influence(region, self.aim) {
                self.aim
            } else {
                region.start
            };
            Self::nearby_building_tile(game, kind, origin, false, Some(self.aim))
        };
        if let Some(tile) = tile {
            self.aim = tile;
            self.pointer = None;
            self.guide_marker = Some((tile, now + 6.));
            view.focus_action(tile);
            self.sound_cues.push(Cue::Choose);
        } else {
            let reason = game.blueprint_readiness(kind).err().unwrap_or_else(|| {
                "Kein freier Bauplatz in der Nähe. Suche in einem anderen Bereich.".into()
            });
            self.notify(reason, Feedback::Blocked, now + 4.);
        }
    }

    fn inspect(&mut self, game: &Game) -> bool {
        let Some(site) = game.city().building_at(self.aim) else {
            return false;
        };
        self.open_panel(Panel::Inspection);
        self.focus = Some(if site.kind.upgrade().is_some() && !site.upgraded {
            UiAction::Do(Action::Upgrade(site.tile))
        } else if site.kind == Building::Civic(Civic::Barracks) {
            UiAction::Do(Action::Muster(site.tile))
        } else {
            UiAction::Cancel
        });
        true
    }

    fn focus_tool_target(&mut self, game: &Game, view: &mut CameraRig, now: f32) {
        self.aim = self.guide_tile(game);
        self.pointer = None;
        self.guide_marker = Some((self.aim, now + 6.));
        view.focus_action(self.aim);
    }

    pub fn show_main_menu(&mut self, game: &Game) {
        self.story = None;
        self.panel_history.clear();
        self.panel = Panel::MainMenu;
        self.pointer = None;
        self.focus = Some(if let Some(mode) = game.continue_mode() {
            UiAction::SelectMode(mode)
        } else if game.practice_required() {
            UiAction::SelectMode(Mode::Tutorial)
        } else {
            UiAction::ModeTab(false)
        });
        self.revision = self.revision.wrapping_add(1);
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
            Panel::Menu => Some(UiAction::MainMenu),
            Panel::Projects(_) => Some(UiAction::Cancel),
            Panel::Display => Some(UiAction::Reading(self.reading)),
            Panel::Sound => Some(UiAction::TestSound),
            Panel::Wheel(wheel) => Some(
                self.practice_step
                    .and_then(PracticeStep::wheel)
                    .filter(|(expected, _)| *expected == wheel)
                    .map_or_else(
                        || wheel.initial(self.tool, self.option),
                        |(_, choice)| choice,
                    ),
            ),
            Panel::Controls | Panel::VillageLog => Some(UiAction::Cancel),
            Panel::Briefing => Some(UiAction::Resume),
            Panel::Map => Some(UiAction::Landscape),
            Panel::Orders(tool) => Some(UiAction::Order(
                tool,
                if self.tool == tool { self.option } else { 0 },
            )),
            Panel::Spells => Some(UiAction::Order(
                Tool::Miracles,
                if self.tool == Tool::Miracles {
                    self.option
                } else {
                    0
                },
            )),
            Panel::Quests => Some(UiAction::Guide),
            Panel::Research => Some(UiAction::SelectTech(self.tech)),
            Panel::Progress => Some(UiAction::Panel(Panel::Talent(crate::game::Talent::Rain))),
            Panel::Talent(t) => Some(UiAction::Panel(Panel::Talent(t))),
            Panel::Dedication => Some(UiAction::Do(Action::Dedicate(
                crate::game::Dedication::Companion,
            ))),
            _ => None,
        };
    }

    fn back_panel(&mut self) {
        let (panel, focus) = self.panel_history.pop().unwrap_or((Panel::Closed, None));
        self.panel = panel;
        self.set_focus(focus);
    }

    fn close_panels(&mut self) {
        self.panel_history.clear();
        self.panel = Panel::Closed;
        self.focus = None;
    }

    fn open_departure(&mut self, game: &Game) {
        self.open_panel(Panel::Departure);
        let next = game.campaign.active + 1;
        self.focus = Some(if next <= game.campaign.unlocked {
            UiAction::Do(Action::Travel(next))
        } else {
            UiAction::Resume
        });
    }

    fn open_delivery(&mut self, game: &Game) {
        self.open_panel(Panel::FirstDelivery);
        let region = game.region();
        let focused = game
            .city()
            .opening_farm(region)
            .filter(|&farm| farm == self.aim);
        self.focus = Some(UiAction::Focus(focused.unwrap_or(region.depot)));
    }

    fn focus_tile(&mut self, tile: Tile, view: &mut CameraRig) {
        if self.panel == Panel::FirstDelivery {
            view.focus_action(tile);
        } else {
            view.focus(tile);
        }
        self.aim = tile;
        self.tool(Tool::Hand);
        self.close_panels();
        self.pointer = None;
    }

    fn choose_mode(&mut self, mode: Mode, game: &mut Game, view: &mut CameraRig, now: f32) {
        match game.switch_mode(mode) {
            Ok(name) => {
                game.replay_finished_practice();
                let revision = self.revision.wrapping_add(1);
                let gamepad = self.gamepad;
                let mix = self.mix;
                let reading = self.reading;
                *self = Self::new(game);
                self.revision = revision;
                self.gamepad = gamepad;
                self.mix = mix;
                self.reading = reading;
                *view = CameraRig::new(game);
                view.jump = true;
                if let Some(story) = self.story {
                    story.frame(game.region(), view);
                    self.focus = Some(UiAction::FinishStory);
                } else {
                    self.notify(name, Feedback::Selection, now + 3.);
                }
            }
            Err(reason) => {
                self.notify(reason, Feedback::Blocked, now + 5.);
            }
        }
    }

    fn execute(&mut self, action: Action, game: &mut Game, view: &mut CameraRig, now: f32) -> bool {
        if action == Action::SeeWork {
            self.open_work(game);
            return false;
        }
        if let Action::Audience(i) = action {
            self.open_panel(Panel::Village(i));
            self.focus = Some(UiAction::Do(Action::Visit(i, true)));
            return false;
        }
        let previous = game.campaign.active;
        let quest = game.city().quest;
        let carried = game.city().held.is_some();
        let opening_choice = game.city().opening_choice();
        let first_island_landing = matches!(action, Action::Travel(4))
            && game.mode == Mode::Chronicle
            && game
                .campaign
                .settlements
                .get(4)
                .is_some_and(Option::is_none);
        let circle_before = game
            .city()
            .gatherings
            .circle
            .as_ref()
            .map(|c| (c.center, c.kind));
        let stock_before = game.city().stock;
        let faith_before = game.city().faith;
        let result = game.act(action);
        let success = result.is_ok();
        let advanced = success && previous == game.campaign.active && quest != game.city().quest;
        let kind = if !success {
            Feedback::Blocked
        } else if advanced
            || matches!(
                action,
                Action::AcceptLocal | Action::FinishLocal | Action::Give
            )
        {
            Feedback::Quest
        } else {
            Feedback::Action
        };
        self.notify(result.unwrap_or_else(|message| message), kind, now + 5.);
        if success {
            if let Action::Learn(talent) = action
                && self.panel == Panel::Talent(talent)
                && (game.campaign.gifts.points == 0 || game.campaign.gifts.rank(talent) == 2)
            {
                self.focus = Some(UiAction::Panel(Panel::Talent(talent)));
            }
            if shows_world_receipt(action) && previous == game.campaign.active {
                let city = game.city();
                let mut changes = Vec::new();
                if matches!(action, Action::Produce(_)) {
                    changes.push(self.notice.clone());
                }
                for material in Material::ALL {
                    let i = material.index();
                    let change = i32::from(city.stock[i]) - i32::from(stock_before[i]);
                    if change != 0 {
                        changes.push(format!(
                            "{change:+} {}",
                            game.region().kind.material(material)
                        ));
                    }
                }
                if let Some(bundle) = city.held.filter(|_| !carried) {
                    changes.push(format!(
                        "{} {} gegriffen",
                        bundle.count,
                        game.region()
                            .kind
                            .material(game.region().sources[bundle.source].material)
                    ));
                }
                let faith = i32::from(city.faith) - i32::from(faith_before);
                if faith != 0 {
                    changes.push(format!("{faith:+} Glauben"));
                }
                if changes.is_empty() {
                    changes.push(match action {
                        Action::Pet => "Zufriedenes Schnauben".into(),
                        Action::Assign(_, true) => "Nilpferd unterwegs".into(),
                        Action::Assign(_, false) => "Helfer unterwegs".into(),
                        Action::Cast(spell, _) => format!("{} gewirkt", spell.name()),
                        _ => self.notice.clone(),
                    });
                }
                self.receipt = Some((self.aim, now, changes.join(" · ")));
            }
            if let Action::Visit(i, _)
            | Action::VillageGift(i)
            | Action::Withdraw(i)
            | Action::Aid(i, _)
            | Action::SendPilgrims(i) = action
            {
                let place = game.region().villages[i].center;
                self.focus_tile(place, view);
                if matches!(action, Action::Aid(..) | Action::VillageGift(_)) {
                    self.celebration = Some((place, false, now, crate::game::Rite::Council));
                    self.hippo_happy_at = Some(now);
                }
                self.notice_panel = Panel::Closed;
            }
            if matches!(action, Action::Call(..))
                && let Some(c) = &game.city().gatherings.circle
            {
                self.focus_tile(c.center, view);
                self.circle_ready = false;
                self.notice_panel = Panel::Closed;
            }
            if let Action::Invoke(awe) = action
                && let Some((center, kind)) = circle_before
            {
                self.focus_tile(center, view);
                self.celebration = Some((center, awe, now, kind));
                self.hippo_happy_at = Some(now);
                self.notice_panel = Panel::Closed;
            }
            if action == Action::DismissCircle {
                self.close_panels();
                self.notice_panel = Panel::Closed;
            }
            if matches!(action, Action::Build(..))
                && !game.region().sandbox
                && !game.region().tutorial
            {
                self.tool(Tool::Hand);
            }
            self.guide_marker = None;
            let took = !carried && game.city().held.is_some();
            let chose_delivery = opening_choice.is_none()
                && game.city().opening_choice().is_some()
                && matches!(action, Action::Hand(_));
            self.revision = self.revision.wrapping_add(1);
            let cue = match action {
                Action::Hand(_) if chose_delivery => Cue::Reward,
                Action::Hand(_) if carried => Cue::Drop,
                Action::Grab(..) | Action::Hand(_) if game.city().held.is_some() => Cue::Lift,
                Action::Pet | Action::Praise => Cue::Pet,
                Action::ShareMeal(_) | Action::Trade(..) | Action::Produce(_) => Cue::Reward,
                Action::Hand(tile) if game.city().harvest_at(game.region(), tile).is_none() => {
                    Cue::Gather
                }
                Action::Cast(spell, _) => Cue::Cast(spell),
                Action::Build(..) | Action::Upgrade(_) | Action::Road(..) | Action::Repair => {
                    Cue::Build
                }
                Action::MoveHippo(_)
                | Action::Assign(..)
                | Action::AssignArea(..)
                | Action::RecallHelpers => Cue::Order,
                Action::Visit(..) | Action::Withdraw(_) => Cue::Order,
                Action::SendPilgrims(_) => Cue::Order,
                Action::Aid(..)
                | Action::Charter(_)
                | Action::VillageGift(_)
                | Action::IslandPact => Cue::Reward,
                Action::Call(..) => Cue::Quest,
                Action::Invoke(true) => Cue::Cast(Miracle::Thunder),
                Action::Invoke(false) => Cue::Reward,
                Action::AcceptWork | Action::AcceptLocal => Cue::Quest,
                Action::Dedicate(_) | Action::RaiseWork => Cue::Reward,
                Action::FinishLocal | Action::Research(_) | Action::Learn(_) => Cue::Reward,
                Action::Give => Cue::Drop,
                _ => Cue::Choose,
            };
            self.sound_cues.push(cue);
            if advanced && !game.region().tutorial {
                self.sound_cues.push(Cue::Reward);
            }
            let gesture = match action {
                Action::Hand(_) if carried => Some(PawGesture::Release),
                Action::Grab(..) | Action::Hand(_) if game.city().held.is_some() => {
                    Some(PawGesture::Grab)
                }
                Action::Cast(..) => Some(PawGesture::Cast),
                Action::Pet => Some(PawGesture::Pet),
                Action::Build(..) | Action::MoveHippo(_) | Action::Assign(..) => {
                    Some(PawGesture::Point)
                }
                _ => None,
            };
            if let Some(gesture) = gesture {
                self.paw_gesture = Some((gesture, now));
            }
            if took && game.mode != Mode::Tutorial {
                self.guide_marker = None;
                if game.city().first_delivery_ready(game.region()) {
                    self.open_delivery(game);
                } else if let Some(store) = game.city().carry_destination(game.region(), self.aim) {
                    self.aim = store;
                    self.guide_marker = Some((store, now + 6.));
                    self.pointer = None;
                }
            }
            if action == Action::Praise {
                self.hippo_happy_at = Some(now);
            }
            if matches!(action, Action::Pet | Action::Praise) {
                self.close_panels();
                self.tool(Tool::Hand);
                self.aim = game.city().walkers[0].tile();
                view.focus_action(self.aim);
            }
            if matches!(action, Action::Road(..)) {
                self.road_anchor = None;
            }
            if let Action::Charter(t) = action {
                self.focus_tile(t, view);
                self.notice_panel = Panel::Closed;
                self.receipt = Some((t, now, "Bürgerrecht verliehen · +1 Wohnplatz".into()));
            }
            if matches!(action, Action::Upgrade(_)) {
                self.focus = Some(UiAction::Cancel);
            }
            if matches!(
                action,
                Action::AcceptWork | Action::SupplyWork | Action::TrackWork(_)
            ) {
                self.open_work_after_action(game);
            }
            if matches!(
                action,
                Action::RaiseWork | Action::Dedicate(_) | Action::SummonBuilder
            ) {
                let site = game.region().depot;
                self.focus_tile(site, view);
                if action != Action::SummonBuilder {
                    view.frame_work(site);
                } else {
                    view.focus_action(site);
                }
                self.guide_marker = Some((site, now + 8.));
                self.notice_panel = Panel::Closed;
                if action != Action::SummonBuilder {
                    self.celebration = Some((site, false, now, crate::game::Rite::Council));
                    self.hippo_happy_at = Some(now);
                }
            }
            if action == Action::AcceptLocal {
                self.focus = Some(match game.city().side_quest(game.region()) {
                    SideQuest::Ready => UiAction::Do(Action::FinishLocal),
                    SideQuest::Active => UiAction::Guide,
                    _ => UiAction::Cancel,
                });
            } else if action == Action::FinishLocal {
                self.focus = Some(UiAction::Cancel);
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
                self.notice_until = now + 8.;
                *view = CameraRig::new(game);
                view.jump = true;
                self.previous_quest = game.city().quest;
                self.contest_event = game.city().contest.event;
                self.previous_side_quest = game.city().side_quest(game.region());
                self.celebration = None;
                self.circle_ready = false;
                self.hippo_happy_at = None;
                self.hippo_deliveries = game.city().hippo_deliveries;
                self.supply_help = game.city().automatic_supply(game.region());
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

    pub fn observe_deliveries(&mut self, game: &Game, now: f32) {
        let city = game.city();
        let region = game.region();
        let active = city.automatic_supply(region);
        if city.hippo_deliveries > self.hippo_deliveries && !region.tutorial {
            let material = city.supply_lesson(region);
            let name = material.map_or("Vorräte", |m| region.kind.material(m));
            let learned = material.is_some_and(|m| game.campaign.gifts.supplies(m));
            let message = if self.supply_help.is_some() {
                format!("{} selbstständig nachgefüllt", name)
            } else if !learned {
                format!(
                    "{name} geliefert · Lobe dein Nilpferd, damit es bei Knappheit selbst sammelt."
                )
            } else {
                format!("{name} im Lager angekommen")
            };
            self.receipt = Some((
                city.walkers[0].tile(),
                now,
                format!("+{} {name}", city.last_hippo_load),
            ));
            self.hippo_happy_at = Some(now);
            self.sound_cues.push(Cue::Gather);
            self.notify(message, Feedback::Action, now + 5.5);
        } else if let Some(m) = active.filter(|_| self.supply_help.is_none()) {
            self.notify(
                format!(
                    "{} wird knapp · Dein Nilpferd holt Nachschub",
                    region.kind.material(m)
                ),
                Feedback::Action,
                now + 4.,
            );
        }
        self.supply_help = active;
        self.hippo_deliveries = city.hippo_deliveries;
    }

    pub fn observe_quests(&mut self, game: &Game, view: &mut CameraRig, now: f32) {
        let city = game.city();
        let region = game.region();
        let quest = city.quest;
        let animal = game.campaign.gifts.creature;
        if animal.stage() > self.previous_growth_stage {
            self.hippo_happy_at = Some(now);
            let message = format!(
                "Dein Nilpferd ist gewachsen · {} · {} Erfahrung",
                animal.description(),
                animal.experience()
            );
            self.receipt = Some((city.walkers[0].tile(), now, message.clone()));
            if self.notice_kind != Feedback::Quest || now >= self.notice_until {
                self.notify(message, Feedback::Action, now + 6.);
            }
            self.sound_cues.push(Cue::Reward);
        }
        self.previous_growth_stage = animal.stage();
        if self.contest_event != city.contest.event {
            self.contest_event = city.contest.event;
            self.notify(city.contest.news.clone(), Feedback::Quest, now + 6.);
            self.sound_cues.push(Cue::Quest);
        }
        let ready = city
            .gatherings
            .circle
            .as_ref()
            .is_some_and(|c| c.ready(city));
        if ready && !self.circle_ready {
            self.notify(
                "Der Kreis ist bereit. Du kannst wirken oder auf weitere Helfer warten.",
                Feedback::Quest,
                now + 5.,
            );
            self.sound_cues.push(Cue::Gather);
        }
        self.circle_ready = ready;
        if quest != self.previous_quest
            && !region.tutorial
            && (self.notice_kind != Feedback::Quest || now >= self.notice_until)
        {
            let message = if quest == Quest::Complete {
                "Hauptauftrag abgeschlossen".into()
            } else {
                format!("Nächster Hauptauftrag · {}", city.quest_title(region))
            };
            self.notify(message, Feedback::Quest, now + 6.);
            self.sound_cues.push(Cue::Reward);
        }
        if city.local_accepted && now >= self.quest_check_at {
            self.quest_check_at = now + 0.35;
            let side = city.side_quest(region);
            if self.previous_side_quest == SideQuest::Active && side == SideQuest::Ready {
                let (title, _, _) = city.local_quest(region);
                self.notify(
                    format!("{title} · Belohnung abholbereit"),
                    Feedback::Quest,
                    now + 6.,
                );
                self.sound_cues.push(Cue::Quest);
            }
            self.previous_side_quest = side;
        }
        if self.story.is_none()
            && game.campaign.active == 0
            && game.city().opening_choice().is_none()
            && self.previous_quest == Quest::Shelter
            && quest == Quest::Fields
        {
            self.start_story(Sequence::Shelter, game, view);
        }
        if self.previous_quest != quest
            && quest == Quest::Complete
            && !region.tutorial
            && !region.sandbox
            && self.panel == Panel::Closed
        {
            self.open_departure(game);
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
            Sequence::Arrival | Sequence::FirstDelivery | Sequence::Islands => {
                self.tool(Tool::Hand);
                self.aim = self
                    .guide_quest(game)
                    .unwrap_or_else(|| self.guide_tile(game));
                *view = CameraRig::new(game);
                view.focus_action(self.aim);
                self.guide_marker = Some((self.aim, f32::INFINITY));
                self.open_panel(Panel::Briefing);
            }
            Sequence::Crossing => {
                self.aim = region.bridge();
                view.focus(self.aim);
            }
            Sequence::Shelter => {
                view.focus(region.start);
            }
        }
    }
}

fn shows_world_receipt(action: Action) -> bool {
    matches!(
        action,
        Action::Hand(_)
            | Action::Produce(_)
            | Action::ShareMeal(_)
            | Action::Grab(..)
            | Action::Build(..)
            | Action::Cast(..)
            | Action::Assign(..)
            | Action::Pet
            | Action::Upgrade(_)
            | Action::Give
            | Action::Repair
    )
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
    landscape_center: Vec3,
    max_height: f32,
}

impl CameraRig {
    pub fn arm(height: f32) -> Vec3 {
        let near = terrain::smooth((27. - height) / 16.);
        Vec3::new(13., 23., 26.).lerp(Vec3::new(13.8, 14.7, 27.6), near) * (height / 22.).max(1.)
    }

    pub fn new(game: &Game) -> Self {
        let region = game.region();
        let scenery = region.render_bounds();
        let from = scenery.0.world();
        let to = scenery.1.world();
        Self {
            target: region.start.world() + Vec3::new(-0.95, 0., -4.55),
            height: 22.,
            rendered_height: 22.,
            jump: false,
            cover: 0.,
            min: region.bounds.0.world(),
            max: region.bounds.1.world(),
            landscape_center: (from + to) * 0.5,
            max_height: if region.tutorial {
                49.
            } else {
                ((to - from).dot(Vec3::new(0.35, 0., 0.57)) + 26.).max(150.)
            },
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

    fn focus_action(&mut self, tile: Tile) {
        self.focus(tile);
        self.target = (self.target + Vec3::new(1.9, 0., 5.1)).clamp(self.min, self.max);
    }

    fn frame_work(&mut self, site: Tile) {
        self.focus(site);
        self.target = (self.target + Vec3::new(0.8, 0., 2.0)).clamp(self.min, self.max);
        self.height = 15.;
    }

    pub fn landscape(&mut self) {
        self.jump = true;
        self.target = self.landscape_center;
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
    pub fn refresh(&mut self, play: &Play, game: &Game, now: f32) {
        let city = game.city();
        let region = game.region();
        let tile = play.aim;
        let first_delivery = city.first_delivery_at(region, tile);
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
            Action::SeeWork => {
                self.detail = game.city().work_hint(region);
                "Inselwerk öffnen".into()
            }
            Action::AcceptWork
            | Action::SupplyWork
            | Action::RaiseWork
            | Action::SummonBuilder
            | Action::Dedicate(_)
            | Action::TrackWork(_) => "Inselwerk".into(),
            Action::Charter(_) => "Bürgerrecht verleihen".into(),
            Action::Upgrade(tile) => {
                self.detail = match city.upgrade_plan(region, tile) {
                    Ok((index, _)) => city.buildings[index].kind.upgrade().unwrap().benefit.into(),
                    Err(error) => error,
                };
                self.legal = city.building_at(tile).is_some();
                "Ausbau ansehen".into()
            }
            Action::Trade(index, tribute) => {
                let terms = city.trade_offer(region, index).terms(region.kind);
                self.detail = match city.trade_ready(region, index, tribute) {
                    Ok(_) => format!("{} · {terms}", region.districts[index].name),
                    Err(reason) => {
                        self.legal = false;
                        reason
                    }
                };
                "Waren tauschen".into()
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
                } else if let Some(index) = city.harvest_at(region, tile) {
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
            Action::Audience(i) => {
                self.detail = format!("{} · {}", region.villages[i].name, city.contest.status(i));
                "Dorfrat öffnen".into()
            }
            Action::SendPilgrims(_)
            | Action::Aid(..)
            | Action::Visit(_, _)
            | Action::Withdraw(_)
            | Action::VillageGift(_)
            | Action::IslandPact => "Dorfrat".into(),
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
                self.legal = game.campaign.gifts.training >= 2 && city.walkable(region, tile);
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
            Action::Pet => {
                self.legal = city.pet_ready();
                self.detail = if self.legal {
                    "Das Nilpferd ist bei dir · Streicheln erholt es ein wenig"
                } else {
                    "Das Nilpferd braucht einen Moment Ruhe"
                }
                .into();
                "Streicheln".into()
            }
            Action::Rest => {
                self.detail = "Stoppt den Auftrag und die Vorratshilfe. Sammeln oder neues Lob setzt sie fort.".into();
                "Pause einlegen".into()
            }
            Action::Produce(tile) => {
                let site = city
                    .building_at(tile)
                    .expect("targeted production building");
                match city.production_plan(region, site) {
                    Ok((cost, inputs, outputs)) => {
                        let result = if site.kind == Building::Civic(Civic::OlivePress) {
                            "Öl und Glauben".into()
                        } else {
                            goods_text(outputs)
                        };
                        let costs = [stock_text(cost), goods_text(inputs)]
                            .into_iter()
                            .filter(|s| !s.is_empty())
                            .collect::<Vec<_>>()
                            .join(" + ");
                        self.detail = format!("{costs} → {result}");
                    }
                    Err(reason) => {
                        self.detail = reason;
                        self.legal = false;
                    }
                }
                site.kind
                    .civic()
                    .and_then(Civic::production)
                    .unwrap_or("Fertigen")
                    .into()
            }
            Action::Invoke(_) => {
                if let Some(c) = &city.gatherings.circle {
                    self.legal = c.ready(city) && city.mana >= 6.;
                    self.detail = format!("{} · {} · 6 Wunderkraft", c.kind.name(), c.status(city));
                    c.kind.finish().into()
                } else {
                    self.legal = false;
                    "Dorfruf beendet".into()
                }
            }
            Action::Repair => {
                let due = city.crossing_due(region);
                self.legal = (0..3).all(|i| city.stock[i] >= due[i]);
                self.detail = format!(
                    "Aus dem Lager · {}. Bündel sind auch einzeln willkommen.",
                    stock_text(due)
                );
                format!("{} reparieren", region.crossing)
            }
            Action::Give => {
                let due = city.crossing_due(region);
                self.legal = (0..3).all(|i| city.stock[i] >= due[i]);
                self.detail = format!("{} · {}", stock_text(due), region.request);
                "Bitte erfüllen".into()
            }
            Action::Hand(_) if first_delivery.is_some() => {
                self.legal = city.first_delivery_ready(region);
                match first_delivery.unwrap() {
                    FirstDelivery::Village => {
                        self.detail = format!(
                            "{PAW_BUNDLE} Holz an Mira · das Dorf teilt den Rest im Lager."
                        );
                        "Dorf beliefern".into()
                    }
                    FirstDelivery::Farm => {
                        self.detail = format!(
                            "{PAW_BUNDLE} Holz für Joras Dach · {} als Dank.",
                            stock_text(FARM_REWARD)
                        );
                        "Hof beliefern".into()
                    }
                }
            }
            Action::Hand(_) if city.held.is_none() && city.opening_farm(region) == Some(tile) => {
                self.legal = false;
                self.detail = if city.opening_choice() == Some(FirstDelivery::Farm) {
                    "Joras Dach ist wieder dicht."
                } else {
                    "Auf diesem Hof lebt Jora."
                }
                .into();
                "Uferhof".into()
            }
            Action::Hand(_) if city.held.is_some() => {
                self.legal = city.can_store_at(region, tile) && city.influence(region, tile);
                let bundle = city.held.unwrap();
                let owed =
                    city.crossing_due(region)[region.sources[bundle.source].material.index()];
                if tile.within(region.bridge(), 1) && owed > 0 {
                    self.legal = city.influence(region, tile);
                    self.detail = format!(
                        "{} {} für {} · Rest bleibt in der Pfote",
                        bundle.count.min(owed),
                        region.kind.material(region.sources[bundle.source].material),
                        region.crossing
                    );
                    return self.label = if city.bridge_fixed {
                        "Zur Dorfgabe legen"
                    } else {
                        "Zur Baustelle bringen"
                    }
                    .into();
                }
                if city.harvest_at(region, tile) == Some(bundle.source) {
                    self.legal = true;
                    self.detail = "Lege die Ladung wieder an der Quelle ab.".into();
                    return self.label = "Zurücklegen".into();
                }
                let material = region.sources[bundle.source].material;
                let room = city
                    .storage_cap()
                    .saturating_sub(city.stock[material.index()]);
                self.legal &= room > 0;
                self.detail = if room == 0 {
                    "Lager voll · Bündel an die Quelle zurücklegen".into()
                } else if room < bundle.count {
                    format!(
                        "{room} ablegen · {} bleiben in der Pfote",
                        bundle.count - room
                    )
                } else {
                    format!(
                        "{} {} in der Pfote · Ablage am Sammelplatz oder Lager",
                        bundle.count,
                        region.kind.material(material)
                    )
                };
                "Ablegen".into()
            }
            Action::Hand(_) if city.harvest_at(region, tile).is_some() => {
                let index = city.harvest_at(region, tile).unwrap();
                self.legal = city.influence(region, tile) && city.nodes[index] > 0;
                let material = region.kind.material(region.sources[index].material);
                if let Some(hold) = play.grab {
                    self.detail = format!(
                        "{material} · {} von {} · beim Loslassen greifen",
                        hold.amount(now),
                        hold.limit()
                    );
                    "Loslassen".into()
                } else {
                    self.detail = format!(
                        "{material} · {} verfügbar · Halten bis {PAW_LIMIT}",
                        city.nodes[index]
                    );
                    "Greifen".into()
                }
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
