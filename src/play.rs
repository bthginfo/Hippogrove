use bevy::prelude::*;

use crate::{
    civic::{Branch, Civic, Goods, Tech},
    game::{Action, Building, Crop, Game, Miracle, Quest, stock_text},
    region::Tile,
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
            Self::Plan => "Planen",
        }
    }

    pub fn aims_with_drag(self) -> bool {
        matches!(self, Self::Build | Self::Miracles | Self::Plan)
    }

    pub fn choices(self) -> usize {
        match self {
            Self::Build => Building::CATALOG_SIZE,
            Self::Helpers => 3,
            Self::Miracles => Miracle::ALL.len(),
            Self::Hippo => 6,
            Self::Plan => 2,
            _ => 1,
        }
    }

    pub fn choice_name(self, index: usize) -> &'static str {
        match self {
            Self::Build => Building::from_index(index).name(),
            Self::Helpers => ["Sammeln", "Ausbilden", "Patrouille"][index],
            Self::Miracles => Miracle::ALL[index].name(),
            Self::Hippo => ["Sammeln", "Wache", "Feldpflege", "Loben", "Snack", "Pause"][index],
            Self::Plan => ["Wege", "Ausbau"][index],
            _ => self.name(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Panel {
    #[default]
    Closed,
    Journey,
    Map,
    Progress,
    Quests,
    Catalog,
    Research,
    Inspection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAction {
    Tool(Tool),
    Choice(usize),
    Confirm,
    Cancel,
    Rotate,
    Guide,
    Panel(Panel),
    Focus(Tile),
    Landscape,
    ToggleObjective,
    Catalog(Option<Branch>),
    CatalogPage(usize),
    SelectBuilding(Building),
    ResearchBranch(Branch),
    SelectTech(Tech),
    Do(Action),
}

#[derive(Resource)]
pub struct Play {
    pub aim: Tile,
    pub tool: Tool,
    pub panel: Panel,
    pub option: usize,
    pub rotation: u8,
    pub road_anchor: Option<Tile>,
    pub objective_expanded: bool,
    pub catalog_branch: Option<Branch>,
    pub catalog_page: usize,
    pub tech: Tech,
    pub focus: Option<UiAction>,
    pub notice: String,
    pub notice_until: f32,
    pub notice_ok: bool,
    pub last_magic: Option<(Miracle, Tile, u64)>,
    pub gamepad: bool,
    pub pointer: Option<Vec2>,
    pub paused: bool,
    pub revision: u64,
}

impl Play {
    pub fn new(game: &Game) -> Self {
        Self {
            aim: game.region().sources[0].tile,
            tool: Tool::Hand,
            panel: Panel::Closed,
            option: 0,
            rotation: 0,
            road_anchor: None,
            objective_expanded: true,
            catalog_branch: None,
            catalog_page: 0,
            tech: Tech::Foundations,
            focus: None,
            notice: String::new(),
            notice_until: 0.,
            notice_ok: true,
            last_magic: None,
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
            Tool::Hand if city.held.is_some() => region.depot,
            Tool::Miracles if self.miracle() == Miracle::Mend => city.walkers[0].tile(),
            Tool::Miracles if self.miracle().offensive() && city.quest == Quest::Bridge => region
                .guards
                .iter()
                .zip(&city.guards)
                .find(|(_, remaining)| **remaining > 0)
                .map(|(&tile, _)| tile)
                .unwrap_or(region.bridge()),
            Tool::Hand | Tool::Miracles if city.quest == Quest::Districts => region
                .districts
                .iter()
                .zip(&city.districts)
                .filter(|(_, state)| state.outcome.is_none())
                .min_by_key(|(district, _)| self.aim.distance(district.center))
                .map(|(district, _)| district.center)
                .unwrap_or(region.start),
            Tool::Miracles | Tool::Hand if city.quest == Quest::Fields => city
                .buildings
                .iter()
                .find(|site| site.kind == Building::Field)
                .map(|site| site.tile)
                .unwrap_or(region.start),
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

    pub fn activate(
        &mut self,
        action: UiAction,
        game: &mut Game,
        view: &mut CameraRig,
        now: f32,
    ) -> bool {
        match action {
            UiAction::Tool(tool) => self.tool(tool),
            UiAction::Choice(i) => self.choose(i),
            UiAction::Catalog(branch) => {
                self.panel = Panel::Catalog;
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
                        self.panel = Panel::Research;
                        self.focus = None;
                        return false;
                    }
                }
                self.tool(Tool::Build);
                self.choose(kind.index());
                self.panel = Panel::Closed;
                self.focus = None;
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
                    self.panel = Panel::Inspection;
                    self.focus = None;
                    return false;
                }
                return self.execute(self.target(game), game, view, now);
            }
            UiAction::Confirm => {}
            UiAction::Cancel => {
                if self.panel != Panel::Closed {
                    self.panel = Panel::Closed;
                    self.focus = None;
                } else if self.road_anchor.is_some() {
                    self.road_anchor = None;
                } else {
                    self.tool(Tool::Hand);
                }
            }
            UiAction::Rotate => self.rotation = (self.rotation + 1) % 4,
            UiAction::Guide => {
                self.aim = self.guide_tile(game);
                view.focus(self.aim);
                self.pointer = None;
            }
            UiAction::Focus(tile) => {
                self.aim = tile;
                self.tool(Tool::Hand);
                self.panel = Panel::Closed;
                self.focus = None;
                self.pointer = None;
                view.focus(tile);
            }
            UiAction::Landscape => {
                self.panel = Panel::Closed;
                self.focus = None;
                view.landscape();
                self.pointer = None;
            }
            UiAction::Panel(panel) => {
                self.panel = if self.panel == panel {
                    Panel::Closed
                } else {
                    panel
                };
                self.focus = None;
            }
            UiAction::ToggleObjective => self.objective_expanded = !self.objective_expanded,
            UiAction::Do(action) => return self.execute(action, game, view, now),
        }
        false
    }

    fn execute(&mut self, action: Action, game: &mut Game, view: &mut CameraRig, now: f32) -> bool {
        let previous = game.campaign.active;
        let result = game.act(action);
        let success = result.is_ok();
        self.notice = result.unwrap_or_else(|error| error);
        self.notice_until = now + 5.;
        self.notice_ok = success;
        if success {
            self.revision = self.revision.wrapping_add(1);
            if matches!(action, Action::Road(..)) {
                self.road_anchor = None;
            }
            if let Action::Cast(spell, tile) = action {
                self.last_magic = Some((spell, tile, self.revision));
            }
            if previous != game.campaign.active {
                self.tool = Tool::Hand;
                self.panel = Panel::Closed;
                self.focus = None;
                self.option = 0;
                self.road_anchor = None;
                self.last_magic = None;
                self.aim = game.region().sources[0].tile;
                self.notice = game.region().subtitle.clone();
                self.notice_until = now + 8.;
                *view = CameraRig::new(game);
            } else if matches!(action, Action::Travel(_)) {
                self.panel = Panel::Closed;
                self.focus = None;
            }
        }
        success
    }
}

#[derive(Resource)]
pub struct CameraRig {
    pub target: Vec3,
    pub height: f32,
    min: Vec3,
    max: Vec3,
    max_height: f32,
}

impl CameraRig {
    pub fn new(game: &Game) -> Self {
        let region = game.region();
        Self {
            target: region.start.world() + Vec3::new(-0.95, 0., -4.55),
            height: 22.,
            min: region.bounds.0.world(),
            max: region.bounds.1.world(),
            max_height: if region.kind == crate::region::Biome::Aegean {
                149.
            } else {
                106.
            },
        }
    }

    pub fn zoom(&mut self, change: f32) {
        self.height = (self.height * (-change).exp()).clamp(11., self.max_height);
    }

    pub fn focus(&mut self, tile: Tile) {
        self.target = (tile.world() - Vec3::Z).clamp(self.min, self.max);
        self.height = self.height.clamp(22., 29.);
    }

    pub fn landscape(&mut self) {
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
                self.detail = result
                    .as_ref()
                    .err()
                    .cloned()
                    .unwrap_or_else(|| kind.cost_text());
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
            Action::Assign(_, hippo) => {
                self.legal = region.source_at(tile).is_some();
                if let Some(index) = region.source_at(tile) {
                    let name = region.sources[index].material.name();
                    self.detail = format!("{name} · {} verfügbar", city.nodes[index]);
                } else {
                    self.detail = "Wähle einen Holzstoß, Felsen oder Beerenstrauch.".into();
                }
                if hippo {
                    "Nilpferd losschicken".into()
                } else {
                    "Helfer losschicken".into()
                }
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
                    region.gift.0.name(),
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
                    region.sources[index].material.name(),
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
