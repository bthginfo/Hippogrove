use bevy::prelude::*;

use crate::{
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
}

impl Tool {
    pub const ALL: [Self; 5] = [
        Self::Hand,
        Self::Build,
        Self::Helpers,
        Self::Hippo,
        Self::Miracles,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Hand => "Pfote",
            Self::Build => "Bauen",
            Self::Helpers => "Gläubige",
            Self::Hippo => "Nilpferd",
            Self::Miracles => "Wunder",
        }
    }

    pub fn aims_with_drag(self) -> bool {
        matches!(self, Self::Build | Self::Miracles)
    }

    pub fn choices(self) -> usize {
        match self {
            Self::Build => Building::ALL.len(),
            Self::Miracles => Miracle::ALL.len(),
            Self::Hippo => 2,
            _ => 1,
        }
    }

    pub fn choice_name(self, index: usize) -> &'static str {
        match self {
            Self::Build => Building::ALL[index].name(),
            Self::Miracles => Miracle::ALL[index].name(),
            Self::Hippo => ["Sammeln", "Wache"][index],
            _ => self.name(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Panel {
    #[default]
    Closed,
    Journey,
    Progress,
    Quests,
}

#[derive(Clone, Copy, Debug)]
pub enum UiAction {
    Tool(Tool),
    Choice(usize),
    Confirm,
    Cancel,
    Rotate,
    Guide,
    Panel(Panel),
    Do(Action),
}

#[derive(Resource)]
pub struct Play {
    pub aim: Tile,
    pub tool: Tool,
    pub panel: Panel,
    pub option: usize,
    pub rotation: u8,
    pub notice: String,
    pub notice_until: f32,
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
            notice: game.region().lore.clone(),
            notice_until: 8.,
            last_magic: None,
            gamepad: false,
            pointer: None,
            paused: false,
            revision: 0,
        }
    }

    pub fn building(&self) -> Building {
        Building::ALL[self.option.min(Building::ALL.len() - 1)]
    }

    pub fn miracle(&self) -> Miracle {
        Miracle::ALL[self.option.min(Miracle::ALL.len() - 1)]
    }

    pub fn tool(&mut self, tool: Tool) {
        if self.tool != tool {
            self.tool = tool;
            self.option = 0;
        }
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
            Tool::Hand => Action::Hand(self.aim),
            Tool::Build => Action::Build(self.building(), self.aim, self.rotation),
            Tool::Helpers => Action::Assign(self.aim, false),
            Tool::Hippo if self.option == 1 => Action::Watch(self.aim),
            Tool::Hippo => Action::Assign(self.aim, true),
            Tool::Miracles => Action::Cast(self.miracle(), self.aim),
        }
    }

    fn guide_tile(&self, game: &Game) -> Tile {
        let city = game.city();
        let region = game.region();
        match self.tool {
            Tool::Build if self.building() == Building::Shrine => region.frontier,
            Tool::Hand if city.held.is_some() => region.depot,
            Tool::Miracles if self.miracle() == Miracle::Wind => region
                .guards
                .iter()
                .zip(&city.guards)
                .find(|(_, remaining)| **remaining > 0)
                .map(|(&tile, _)| tile)
                .unwrap_or(region.bridge()),
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
            UiAction::Choice(i) => self.option = i.min(self.tool.choices() - 1),
            UiAction::Confirm if self.panel == Panel::Closed => {
                return self.execute(self.target(game), game, view, now);
            }
            UiAction::Confirm => {}
            UiAction::Cancel => {
                if self.panel != Panel::Closed {
                    self.panel = Panel::Closed;
                } else {
                    self.tool(Tool::Hand);
                }
            }
            UiAction::Rotate => self.rotation = (self.rotation + 1) % 4,
            UiAction::Guide => {
                self.aim = self.guide_tile(game);
                view.target = (game.region().start.world() + self.aim.world() * 2.) / 3.;
            }
            UiAction::Panel(panel) => {
                self.panel = if self.panel == panel {
                    Panel::Closed
                } else {
                    panel
                };
            }
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
        if success {
            self.revision = self.revision.wrapping_add(1);
            if let Action::Cast(spell, tile) = action {
                self.last_magic = Some((spell, tile, self.revision));
            }
            if previous != game.campaign.active {
                self.tool = Tool::Hand;
                self.panel = Panel::Closed;
                self.option = 0;
                self.last_magic = None;
                self.aim = game.region().sources[0].tile;
                self.notice = game.region().lore.clone();
                self.notice_until = now + 8.;
                *view = CameraRig::new(game);
            } else if matches!(action, Action::Travel(_)) {
                self.panel = Panel::Closed;
            }
        }
        success
    }
}

#[derive(Resource)]
pub struct CameraRig {
    pub target: Vec3,
    pub height: f32,
}

impl CameraRig {
    pub fn new(game: &Game) -> Self {
        Self {
            target: game.region().start.world() + Vec3::new(-0.95, 0., -4.55),
            height: 22.,
        }
    }

    pub fn zoom(&mut self, change: f32) {
        self.height = (self.height * (-change).exp()).clamp(11., 42.);
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        let right = Vec3::new(0.89, 0., -0.45);
        let forward = Vec3::new(-0.45, 0., -0.89);
        self.target += right * dx + forward * dy;
        self.target.x = self.target.x.clamp(-26., 25.);
        self.target.z = self.target.z.clamp(-18., 19.);
    }
}

#[derive(Resource, Default)]
pub struct Target {
    pub label: String,
    pub detail: String,
    pub legal: bool,
    placement: Option<(PlacementKey, Result<(), String>)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PlacementKey {
    region: usize,
    kind: Building,
    tile: Tile,
    stock: [u16; 3],
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
        self.label = match play.target(game) {
            Action::Build(kind, tile, _) => {
                let key = PlacementKey {
                    region: game.campaign.active,
                    kind,
                    tile,
                    stock: city.stock,
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
                    self.placement = Some((key, city.placement(region, kind, tile)));
                }
                let result = &self.placement.as_ref().unwrap().1;
                self.legal = result.is_ok();
                self.detail = result
                    .as_ref()
                    .err()
                    .cloned()
                    .unwrap_or_else(|| stock_text(kind.price()));
                format!("{} bauen", kind.name())
            }
            Action::Cast(spell, tile) => {
                let error = city.ritual_error(region, spell, tile, game.campaign.gifts);
                self.legal = error.is_none();
                self.detail = error
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("{:.0} Wunderkraft", spell.cost()));
                format!("{} wirken", spell.name())
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
                self.legal = game.campaign.gifts.training >= 2 && city.walkable(region, tile);
                self.detail = "Die Kreatur lernt das Bewachen auf Rang 2.".into();
                "Wache halten".into()
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
            _ => {
                self.legal = false;
                self.detail = "Wähle eine Quelle oder ein reifes Feld.".into();
                "Greifen".into()
            }
        };
    }
}
