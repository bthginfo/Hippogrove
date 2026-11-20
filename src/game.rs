use std::{
    cmp::Reverse,
    collections::{BinaryHeap, VecDeque},
};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::civic::{Civic, Goods, Research, Tech, goods_text};
use crate::modes::Mode;
use crate::quests::QuestStep;
use crate::region::{
    AREA_COUNT, AREA_RADIUS, Biome, CELL, DistrictNeed, Material, Region, Tile, Weather,
    WorldLayout,
};

pub const SAVE_VERSION: u32 = 4;
const ORIGINAL_SOURCE_COUNT: usize = 10;
pub const START_RADIUS: i16 = 7;
pub const FRONTIER_RADIUS: i16 = 8;
const RIVAL_RADIUS: i16 = 6;
const DISTRICT_RADIUS: i16 = 6;
const WAYSTONE_FAITH: u16 = 5;
const WAYSTONE_LIMIT: usize = 24;
pub const MAX_ROAD_RUN: u16 = 12;
pub const MAX_WALKERS: usize = 32;
pub const CAMP_FAITH: u16 = 6;
pub const CAMP_WOOD: u16 = 5;
pub const DAY_LENGTH: f32 = 192.;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DayPhase {
    Morning,
    Work,
    Evening,
    Night,
}

impl DayPhase {
    pub fn name(self) -> &'static str {
        match self {
            Self::Morning => "Morgen",
            Self::Work => "Tag",
            Self::Evening => "Abend",
            Self::Night => "Nacht",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SideQuest {
    Available,
    Active,
    Ready,
    Complete,
}

pub type Stock = [u16; 3];
pub type GameResult = Result<String, String>;

pub fn stock_text(stock: Stock) -> String {
    Material::ALL
        .into_iter()
        .filter_map(|material| {
            let amount = stock[material.index()];
            (amount > 0).then(|| format!("{amount} {}", material.name()))
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Building {
    House,
    Store,
    Field,
    Shrine,
    Workshop,
    Granary,
    Well,
    Market,
    Tower,
    Waystone,
    Civic(Civic),
}

impl Building {
    pub const ALL: [Self; 10] = [
        Self::House,
        Self::Store,
        Self::Field,
        Self::Shrine,
        Self::Workshop,
        Self::Granary,
        Self::Well,
        Self::Market,
        Self::Tower,
        Self::Waystone,
    ];

    pub const CATALOG_SIZE: usize = Self::ALL.len() + Civic::ALL.len();

    pub fn from_index(index: usize) -> Self {
        Self::ALL
            .get(index)
            .copied()
            .unwrap_or_else(|| Self::Civic(Civic::ALL[index - Self::ALL.len()]))
    }

    pub fn index(self) -> usize {
        (0..Self::CATALOG_SIZE)
            .position(|index| Self::from_index(index) == self)
            .unwrap()
    }

    pub fn civic(self) -> Option<Civic> {
        if let Self::Civic(kind) = self {
            Some(kind)
        } else {
            None
        }
    }

    pub fn gate(self) -> bool {
        self == Self::Civic(Civic::Gate)
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::House => "Hütte",
            Self::Store => "Lager",
            Self::Field => "Feld",
            Self::Shrine => "Grenzschrein",
            Self::Workshop => "Werkstatt",
            Self::Granary => "Speicher",
            Self::Well => "Brunnen",
            Self::Market => "Markt",
            Self::Tower => "Wachturm",
            Self::Waystone => "Grenzstein",
            Self::Civic(kind) => kind.spec().name,
        }
    }

    pub fn size(self) -> i16 {
        if let Self::Civic(kind) = self {
            return kind.spec().size;
        }
        if matches!(
            self,
            Self::Shrine | Self::Well | Self::Tower | Self::Waystone
        ) {
            1
        } else {
            2
        }
    }

    pub fn price(self) -> Stock {
        match self {
            Self::House => [12, 3, 0],
            Self::Store => [9, 5, 0],
            Self::Field => [5, 2, 0],
            Self::Shrine => [7, 8, 0],
            Self::Workshop => [12, 9, 0],
            Self::Granary => [16, 10, 0],
            Self::Well => [4, 7, 0],
            Self::Market => [10, 6, 0],
            Self::Tower => [7, 9, 0],
            Self::Waystone => [3, 5, 0],
            Self::Civic(kind) => kind.spec().stock,
        }
    }

    pub fn goods_price(self) -> Goods {
        self.civic().map_or([0; 3], |kind| kind.spec().goods)
    }

    pub fn cost_text(self) -> String {
        let raw = stock_text(self.price());
        let refined = goods_text(self.goods_price());
        if refined.is_empty() {
            raw
        } else {
            format!("{raw} · {refined}")
        }
    }

    pub fn upgrade_price(self) -> Option<Stock> {
        match self {
            Self::House => Some([7, 5, 2]),
            Self::Field => Some([4, 6, 0]),
            Self::Store | Self::Granary => Some([8, 6, 0]),
            Self::Well | Self::Tower => Some([4, 8, 0]),
            Self::Market => Some([10, 6, 4]),
            Self::Shrine | Self::Workshop | Self::Waystone => None,
            Self::Civic(Civic::Lodge | Civic::Courtyard | Civic::Forge | Civic::Bastion) => {
                Some([8, 6, 2])
            }
            Self::Civic(_) => None,
        }
    }

    pub fn use_text(self) -> &'static str {
        match self {
            Self::House => "Ein weiterer Gläubiger",
            Self::Store | Self::Granary => "Mehr Platz für Vorräte",
            Self::Field => "Größere Ernte",
            Self::Shrine => "Glauben am Übergang",
            Self::Workshop => "Schaltet Speicher und Ausbau frei",
            Self::Well => "Bewässert Felder in größerem Umkreis",
            Self::Market => "Tauscht Nahrung gegen mehr Baustoffe",
            Self::Tower => "Beschützt einen größeren Umkreis",
            Self::Waystone => "Erweitert deinen Einfluss",
            Self::Civic(Civic::Lodge | Civic::Courtyard) => "Zwei zusätzliche Wohnplätze",
            Self::Civic(Civic::Forge) => "Fertigt zwei Werkzeuge je Auftrag",
            Self::Civic(Civic::Bastion) => "Vergrößert den Schutzbereich",
            Self::Civic(kind) => kind.spec().purpose,
        }
    }

    pub fn tiles(self, tile: Tile) -> impl Iterator<Item = Tile> {
        (0..self.size()).flat_map(move |z| (0..self.size()).map(move |x| tile.offset(x, z)))
    }

    pub fn center(self, tile: Tile) -> Vec3 {
        let offset = f32::from(self.size() - 1) * CELL * 0.5;
        tile.world() + Vec3::new(offset, 0., offset)
    }

    fn renown(self) -> u16 {
        match self {
            Self::House => 5,
            Self::Store => 3,
            Self::Field => 4,
            Self::Shrine => 5,
            Self::Workshop => 7,
            Self::Granary => 6,
            Self::Well => 4,
            Self::Market => 6,
            Self::Tower => 5,
            Self::Waystone => 1,
            Self::Civic(kind) => 2 + u16::from(kind.spec().unlock.city_rank()),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Crop {
    Dry,
    Growing(f32),
    Ripe,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuildingSite {
    pub kind: Building,
    pub tile: Tile,
    pub rotation: u8,
    pub crop: Crop,
    #[serde(default)]
    pub upgraded: bool,
}

impl BuildingSite {
    pub fn contains(&self, tile: Tile) -> bool {
        self.kind.tiles(self.tile).any(|p| p == tile)
    }

    fn roadside(&self, road: impl Fn(Tile) -> bool) -> bool {
        self.kind
            .tiles(self.tile)
            .flat_map(Tile::neighbors)
            .any(road)
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Quest {
    #[default]
    Shelter,
    Fields,
    Bridge,
    Districts,
    Complete,
}

impl Quest {
    pub fn name(self) -> &'static str {
        match self {
            Self::Shelter => "Ein Zuhause am Wasser",
            Self::Fields => "Die trockenen Felder",
            Self::Bridge => "Ein Dorf am anderen Ufer",
            Self::Districts => "Zwei Dörfer am Horizont",
            Self::Complete => "Das Land hat deinen Ruf gehört",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Outcome {
    Alliance,
    Victory,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct DistrictStatus {
    pub resolve: u8,
    pub outcome: Option<Outcome>,
}

impl Default for DistrictStatus {
    fn default() -> Self {
        Self {
            resolve: 4,
            outcome: None,
        }
    }
}

impl Outcome {
    pub fn name(self) -> &'static str {
        match self {
            Self::Alliance => "Dem Glauben gewonnen",
            Self::Victory => "Wachen vertrieben",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Miracle {
    Rain,
    Wind,
    Ward,
    Bloom,
    Mend,
    Thunder,
}

impl Miracle {
    pub const ALL: [Self; 6] = [
        Self::Rain,
        Self::Wind,
        Self::Ward,
        Self::Bloom,
        Self::Mend,
        Self::Thunder,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Rain => "Regenschauer",
            Self::Wind => "Windstoß",
            Self::Ward => "Schutzglocke",
            Self::Bloom => "Blütensegen",
            Self::Mend => "Heillicht",
            Self::Thunder => "Donnergrollen",
        }
    }

    pub fn cost(self) -> f32 {
        match self {
            Self::Rain => 5.,
            Self::Wind => 8.,
            Self::Ward => 7.,
            Self::Bloom => 10.,
            Self::Mend => 7.,
            Self::Thunder => 14.,
        }
    }

    pub fn effect(self) -> &'static str {
        match self {
            Self::Rain => "Bewässert trockene Felder",
            Self::Wind => "Schwächt Wachen",
            Self::Ward => "Schützt Vorräte",
            Self::Bloom => "Reift Saat, erneuert Quellen",
            Self::Mend => "Stärkt das Nilpferd",
            Self::Thunder => "Bricht Widerstand",
        }
    }

    pub fn radius(self, gifts: Gifts) -> i16 {
        match self {
            Self::Rain => 2 + i16::from(gifts.rain),
            Self::Wind => 2,
            Self::Ward => 3,
            Self::Bloom => 2 + i16::from(gifts.growth),
            Self::Mend => 3,
            Self::Thunder => 3 + i16::from(gifts.storm),
        }
    }

    pub fn offensive(self) -> bool {
        matches!(self, Self::Wind | Self::Thunder)
    }

    fn recovery(self) -> f32 {
        match self {
            Self::Rain => 2.,
            Self::Wind => 1.25,
            Self::Ward => 8.,
            Self::Bloom => 9.,
            Self::Mend => 6.,
            Self::Thunder => 7.,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Talent {
    Rain,
    Spring,
    Growth,
    Storm,
}

impl Talent {
    pub const ALL: [Self; 4] = [Self::Rain, Self::Spring, Self::Growth, Self::Storm];

    pub fn name(self) -> &'static str {
        match self {
            Self::Rain => "Weiter Regenkreis",
            Self::Spring => "Tiefe Wunderquelle",
            Self::Growth => "Hände der Blüte",
            Self::Storm => "Stimme des Donners",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Gifts {
    pub points: u16,
    pub rain: u8,
    pub spring: u8,
    pub training: u8,
    pub growth: u8,
    pub storm: u8,
    pub coaching: u8,
}

impl Gifts {
    pub fn mana_cap(self) -> f32 {
        28. + f32::from(self.spring) * 8.
    }

    fn earn(&mut self) {
        self.points = self.points.saturating_add(1);
    }

    pub fn rank(self, talent: Talent) -> u8 {
        match talent {
            Talent::Rain => self.rain,
            Talent::Spring => self.spring,
            Talent::Growth => self.growth,
            Talent::Storm => self.storm,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Hand(Tile),
    Build(Building, Tile, u8),
    Road(Tile, Tile),
    Upgrade(Tile),
    Research(Tech),
    Muster(Tile),
    Patrol(Tile),
    Settle(usize),
    Camp(usize),
    Assign(Tile, bool),
    AssignArea(usize, Material, bool),
    CancelArea(usize, Material),
    AcceptLocal,
    FinishLocal,
    RecallHelpers,
    MoveHippo(Tile),
    Watch(Tile),
    Tend(Tile),
    Feed,
    Rest,
    Praise,
    Repair,
    Give,
    Cast(Miracle, Tile),
    Learn(Talent),
    Travel(usize),
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Bundle {
    pub source: usize,
    pub count: u16,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Job {
    Gather(usize),
    GatherArea(usize, Material),
    Move(Tile),
    Watch(Tile),
    Tend(Tile),
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Lesson {
    Delivery(usize),
    Practiced(Job),
}

#[derive(Default)]
pub struct AreaSupply {
    pub units: u32,
    pub stocked: usize,
    pub sources: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Walker {
    pub position: [f32; 2],
    pub hippo: bool,
    #[serde(default)]
    pub guard: bool,
    #[serde(default)]
    pub childhood: f32,
    pub job: Option<Job>,
    pub bundle: Option<Bundle>,
    pub route: VecDeque<Tile>,
    pub pause: f32,
    #[serde(skip)]
    stride: f32,
}

impl Walker {
    fn new(tile: Tile, hippo: bool) -> Self {
        Self {
            position: [f32::from(tile.0), f32::from(tile.1)],
            hippo,
            guard: false,
            childhood: 0.,
            job: None,
            bundle: None,
            route: VecDeque::new(),
            pause: 0.,
            stride: 0.,
        }
    }

    pub fn tile(&self) -> Tile {
        Tile(
            self.position[0].round() as i16,
            self.position[1].round() as i16,
        )
    }

    pub fn world(&self) -> Vec3 {
        let tile = self.tile();
        tile.world()
            + Vec3::new(
                (self.position[0] - f32::from(tile.0)) * CELL,
                0.,
                (self.position[1] - f32::from(tile.1)) * CELL,
            )
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Society {
    pub goods: Goods,
    pub morale: u16,
    pub births: u16,
}

impl Default for Society {
    fn default() -> Self {
        Self {
            goods: [0; 3],
            morale: 58,
            births: 0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Settlement {
    #[serde(default)]
    world_layout: WorldLayout,
    pub stock: Stock,
    pub faith: u16,
    pub mana: f32,
    pub buildings: Vec<BuildingSite>,
    #[serde(default)]
    pub society: Society,
    #[serde(default)]
    pub roads: Vec<Tile>,
    #[serde(default)]
    pub districts: [DistrictStatus; 2],
    #[serde(default)]
    pub camps: [bool; AREA_COUNT],
    #[serde(default)]
    last_market: usize,
    pub nodes: Vec<u16>,
    regrowth: Vec<f32>,
    pub walkers: Vec<Walker>,
    pub held: Option<Bundle>,
    pub quest: Quest,
    pub outcome: Option<Outcome>,
    pub bridge_fixed: bool,
    pub gift_given: bool,
    pub guards: Vec<u8>,
    pub hand_deliveries: u16,
    pub helper_deliveries: u16,
    pub hippo_deliveries: u16,
    pub harvests: u16,
    #[serde(default)]
    pub olive_pressings: u16,
    pub lesson: Option<Lesson>,
    #[serde(default = "full_energy")]
    pub energy: f32,
    #[serde(default)]
    recovering: bool,
    #[serde(default)]
    cooldowns: [f32; Miracle::ALL.len()],
    #[serde(default)]
    pub local_finished: bool,
    #[serde(default)]
    pub local_accepted: bool,
    pub shield: f32,
    pub rainfall: f32,
    pub weather_time: f32,
    raid_clock: f32,
    provisions: f32,
    next_worker: usize,
}

fn full_energy() -> f32 {
    100.
}

impl Settlement {
    pub fn new(region: &Region) -> Self {
        let mut city = Self {
            world_layout: region.world_layout(),
            stock: region.initial,
            faith: 2,
            mana: 20.,
            buildings: Vec::new(),
            society: Society::default(),
            roads: Vec::new(),
            districts: [DistrictStatus::default(); 2],
            camps: [false; AREA_COUNT],
            last_market: 0,
            nodes: region.sources.iter().map(|source| source.stock).collect(),
            regrowth: vec![0.; region.sources.len()],
            walkers: vec![
                Walker::new(region.depot.offset(0, 1), true),
                Walker::new(region.start.offset(-1, 0), false),
                Walker::new(region.depot.offset(-1, 0), false),
            ],
            held: None,
            quest: Quest::Shelter,
            outcome: None,
            bridge_fixed: false,
            gift_given: false,
            guards: vec![2; region.guards.len()],
            hand_deliveries: 0,
            helper_deliveries: 0,
            hippo_deliveries: 0,
            harvests: 0,
            olive_pressings: 0,
            lesson: None,
            energy: full_energy(),
            recovering: false,
            cooldowns: [0.; Miracle::ALL.len()],
            local_finished: false,
            local_accepted: false,
            shield: 0.,
            rainfall: 0.,
            weather_time: 0.,
            raid_clock: 0.,
            provisions: 22.,
            next_worker: 0,
        };
        // The first chapter teaches carrying and building. Later chapters start with a village.
        for (index, &tile) in region.starter_plots.iter().enumerate() {
            let kind = [Building::House, Building::Store, Building::Field][index];
            city.buildings.push(BuildingSite {
                kind,
                tile,
                rotation: 0,
                crop: Crop::Dry,
                upgraded: false,
            });
        }
        if !city.buildings.is_empty() {
            city.quest = Quest::Fields;
            city.faith = 8;
            city.welcome_resident(region, 0);
        }
        if region.sandbox {
            city.quest = Quest::Complete;
            city.bridge_fixed = true;
            city.outcome = Some(Outcome::Alliance);
            city.camps.fill(true);
            city.districts.iter_mut().for_each(|district| {
                district.outcome = Some(Outcome::Alliance);
                district.resolve = 0;
            });
            city.faith = 42;
            city.society.goods = [13, 13, 7];
            let cap = city.storage_cap();
            city.stock = [cap, cap * 3 / 4, cap * 2 / 3];
        }
        city
    }

    pub fn count(&self, building: Building) -> usize {
        self.buildings
            .iter()
            .filter(|site| site.kind == building)
            .count()
    }

    pub fn inhabitants(&self) -> usize {
        self.walkers
            .iter()
            .filter(|walker| !walker.hippo && !walker.guard)
            .count()
    }

    pub fn housing(&self) -> usize {
        2 + self
            .buildings
            .iter()
            .map(|site| match site.kind {
                Building::House => 1 + usize::from(site.upgraded),
                Building::Civic(kind) if kind.beds() > 0 => {
                    kind.beds() + 2 * usize::from(site.upgraded)
                }
                _ => 0,
            })
            .sum::<usize>()
    }

    pub fn garrison(&self) -> usize {
        self.walkers.iter().filter(|walker| walker.guard).count()
    }

    pub fn city_rank(&self) -> u8 {
        if self
            .buildings
            .iter()
            .any(|site| site.kind == Building::House && site.upgraded)
            && self.count(Building::Well) > 0
            && self.count(Building::Market) > 0
        {
            4
        } else if self.count(Building::Workshop) > 0 {
            3
        } else if self.count(Building::Field) > 0 && self.count(Building::Store) > 0 {
            2
        } else if self.count(Building::House) > 0 {
            1
        } else {
            0
        }
    }

    pub fn renown(&self) -> u16 {
        let structures =
            (0..Building::CATALOG_SIZE)
                .map(Building::from_index)
                .fold(0, |renown, kind| {
                    let count = self.count(kind);
                    renown
                        + if count > 0 {
                            kind.renown() + (count - 1).min(4) as u16
                        } else {
                            0
                        }
                });
        let food = self.stock[Material::Food.index()];
        structures
            + (food / 3).min(6)
            + self.harvests.min(3) * 2
            + self
                .buildings
                .iter()
                .filter(|site| site.upgraded)
                .count()
                .min(20) as u16
                * 2
    }

    pub fn storage_cap(&self) -> u16 {
        32 + self.count(Building::Store).min(20) as u16 * 24
            + self.count(Building::Granary).min(20) as u16 * 48
            + self
                .buildings
                .iter()
                .filter(|site| {
                    site.upgraded && matches!(site.kind, Building::Store | Building::Granary)
                })
                .count()
                .min(20) as u16
                * 20
    }

    pub fn influence(&self, region: &Region, tile: Tile) -> bool {
        region.bounds.contains(tile)
            && (region.sandbox
                || tile.within(region.start, START_RADIUS)
                || (tile.within(region.start, START_RADIUS + 5) && self.city_rank() == 4)
                || (self.count(Building::Shrine) > 0
                    && tile.within(region.frontier, FRONTIER_RADIUS))
                || (self.outcome.is_some()
                    && (tile.within(region.rival_shrine, RIVAL_RADIUS)
                        || region.districts.iter().zip(&self.districts).any(
                            |(district, state)| {
                                tile.within(
                                    district.center,
                                    if state.outcome.is_some() {
                                        13
                                    } else {
                                        DISTRICT_RADIUS
                                    },
                                )
                            },
                        )))
                || region
                    .areas
                    .iter()
                    .zip(self.camps)
                    .any(|(area, active)| active && tile.within(area.center, AREA_RADIUS))
                || self
                    .buildings
                    .iter()
                    .any(|site| site.kind == Building::Waystone && tile.within(site.tile, 7)))
    }

    pub fn building_at(&self, tile: Tile) -> Option<&BuildingSite> {
        self.buildings
            .iter()
            .find(|building| building.contains(tile))
    }

    pub fn walkable(&self, region: &Region, tile: Tile) -> bool {
        region.bounds.contains(tile)
            && (!region.water(tile) || (self.bridge_fixed && region.bridge_cell(tile)))
            && !region.rocky(tile)
            && !region.fixed_structure(tile)
            && region.source_at(tile).is_none()
            && self.building_at(tile).is_none_or(|site| site.kind.gate())
    }

    pub fn can_store_at(&self, region: &Region, tile: Tile) -> bool {
        tile == region.depot
            || region.area_at(tile).is_some_and(|index| self.camps[index])
            || region
                .district_at(tile)
                .is_some_and(|index| self.districts[index].outcome.is_some())
            || self.building_at(tile).is_some_and(|site| {
                matches!(
                    site.kind,
                    Building::Store | Building::Granary | Building::Civic(Civic::Port)
                )
            })
    }

    pub fn area_supply(&self, region: &Region, area: usize, material: Material) -> AreaSupply {
        let mut supply = AreaSupply::default();
        for (index, _) in region.area_sources(area, material) {
            let amount = self.nodes[index];
            supply.units += u32::from(amount);
            supply.stocked += usize::from(amount > 0);
            supply.sources += 1;
        }
        supply
    }

    pub fn area_crew(&self, area: usize, material: Material) -> (usize, bool) {
        let job = Some(Job::GatherArea(area, material));
        let hippo = self.walkers[0].job == job;
        let helpers = self.walkers[1..]
            .iter()
            .filter(|walker| walker.job == job)
            .count();
        (helpers, hippo)
    }

    pub fn camp_plan(&self, index: usize) -> Result<(), &'static str> {
        if self.camps[index] {
            Err("Hier steht bereits dein Sammellager.")
        } else if !self.bridge_fixed {
            Err("Repariere zuerst den Hauptübergang.")
        } else if self.faith < CAMP_FAITH || self.stock[0] < CAMP_WOOD {
            Err("Das Sammellager braucht 5 Holz und 6 Glauben.")
        } else {
            Ok(())
        }
    }

    fn open_camp(&mut self, region: &Region, index: usize) -> GameResult {
        let area = region.areas.get(index).ok_or("Wähle ein Rohstoffgebiet.")?;
        self.camp_plan(index)?;
        self.stock[0] -= CAMP_WOOD;
        self.faith -= CAMP_FAITH;
        self.camps[index] = true;
        for walker in &mut self.walkers {
            if walker.bundle.is_some() {
                walker.route.clear();
            }
        }
        Ok(format!("{} · Sammellager geöffnet", area.name))
    }

    pub fn road_at(&self, region: &Region, tile: Tile) -> bool {
        region.road(tile) || self.roads.contains(&tile)
    }

    pub fn connected(&self, region: &Region, site: &BuildingSite) -> bool {
        site.roadside(|cell| self.road_at(region, cell) && self.walkable(region, cell))
    }

    fn service(&self, region: &Region, kind: Civic) -> bool {
        self.buildings
            .iter()
            .any(|site| site.kind == Building::Civic(kind) && self.connected(region, site))
    }

    pub fn mana_cap(&self, region: &Region, gifts: Gifts) -> f32 {
        gifts.mana_cap()
            + if self.service(region, Civic::Temple) {
                14.
            } else {
                0.
            }
    }

    pub fn road_plan(
        &self,
        region: &Region,
        from: Tile,
        to: Tile,
    ) -> Result<(Vec<Tile>, Stock), String> {
        if from.distance(to) > MAX_ROAD_RUN {
            return Err("Ziehe höchstens 12 Felder auf einmal.".into());
        }
        let branch = |horizontal_first: bool| {
            let mut cells = vec![from];
            let mut cell = from;
            for horizontal in [horizontal_first, !horizontal_first] {
                let offset = if horizontal {
                    (to.0 - cell.0, 0)
                } else {
                    (0, to.1 - cell.1)
                };
                for _ in 0..offset.0.abs() + offset.1.abs() {
                    cell = cell.offset(offset.0.signum(), offset.1.signum());
                    cells.push(cell);
                }
            }
            cells
        };
        let cells = [branch(true), branch(false)]
            .into_iter()
            .find(|cells| {
                cells
                    .iter()
                    .all(|&cell| self.influence(region, cell) && self.walkable(region, cell))
                    && (region.sandbox
                        || cells.iter().any(|&cell| {
                            std::iter::once(cell).chain(cell.neighbors()).any(|next| {
                                self.road_at(region, next) && self.walkable(region, next)
                            })
                        }))
            })
            .ok_or("Der Weg braucht freien Boden und Anschluss an einen Dorfweg.")?;
        let tiles: Vec<_> = cells
            .into_iter()
            .filter(|&tile| !self.road_at(region, tile))
            .collect();
        if tiles.is_empty() {
            return Err("Hier liegt schon ein Weg.".into());
        }
        let price = if region.sandbox {
            [0; 3]
        } else {
            [0, (tiles.len() as u16).div_ceil(3), 0]
        };
        if !self.affordable(price) {
            return Err(format!(
                "Für den Weg fehlen Baustoffe · {}",
                stock_text(price)
            ));
        }
        Ok((tiles, price))
    }

    fn build_road(&mut self, region: &Region, from: Tile, to: Tile) -> GameResult {
        let (tiles, price) = self.road_plan(region, from, to)?;
        let paved = tiles.len();
        self.pay(price);
        self.roads.extend(tiles);
        for walker in &mut self.walkers {
            walker.route.clear();
        }
        Ok(format!("{paved} Wegfelder gebaut"))
    }

    pub fn upgrade_plan(&self, region: &Region, tile: Tile) -> Result<(usize, Stock), String> {
        let (index, site) = self
            .buildings
            .iter()
            .enumerate()
            .find(|(_, site)| site.contains(tile))
            .ok_or("Wähle ein Gebäude für den Ausbau.")?;
        let price = site
            .kind
            .upgrade_price()
            .ok_or("Dieses Gebäude ist fertig ausgebaut.")?;
        if site.upgraded {
            return Err("Das Gebäude ist bereits ausgebaut.".into());
        }
        if !region.sandbox && self.city_rank() < 3 {
            return Err("Baue zuerst eine Werkstatt.".into());
        }
        if !self.connected(region, site) {
            return Err("Schließe das Gebäude an einen Dorfweg an.".into());
        }
        let price = if region.sandbox { [0; 3] } else { price };
        if !self.affordable(price) {
            return Err(format!(
                "Für den Ausbau fehlen Baustoffe · {}",
                stock_text(price)
            ));
        }
        Ok((index, price))
    }

    fn upgrade(&mut self, region: &Region, tile: Tile) -> GameResult {
        let (index, price) = self.upgrade_plan(region, tile)?;
        self.pay(price);
        self.buildings[index].upgraded = true;
        let kind = self.buildings[index].kind;
        if kind == Building::House {
            self.welcome_resident(region, index);
        }
        self.faith = self.faith.saturating_add(3);
        Ok(format!("{} ausgebaut · {}", kind.name(), kind.use_text()))
    }

    pub fn district_need(need: DistrictNeed) -> &'static str {
        match need {
            DistrictNeed::Water => "Brunnen",
            DistrictNeed::Trade => "Markt",
            DistrictNeed::Shelter => "ausgebaute Hütte",
            DistrictNeed::Patrol => "Wachturm",
        }
    }

    pub fn district_plan(&self, region: &Region, index: usize) -> Result<Outcome, String> {
        let district = region
            .districts
            .get(index)
            .ok_or("Diese Siedlung ist unbekannt.")?;
        let state = &self.districts[index];
        if self.outcome.is_none() {
            return Err("Löse zuerst den Konflikt am Hauptübergang.".into());
        }
        if let Some(outcome) = state.outcome {
            return Err(format!("{} · {}", district.name, outcome.name()));
        }
        if state.resolve == 0 {
            return Ok(Outcome::Victory);
        }
        let served = self.buildings.iter().any(|site| {
            site.tile.within(district.center, DISTRICT_RADIUS)
                && match district.need {
                    DistrictNeed::Water => site.kind == Building::Well,
                    DistrictNeed::Trade => site.kind == Building::Market,
                    DistrictNeed::Shelter => site.kind == Building::House && site.upgraded,
                    DistrictNeed::Patrol => site.kind == Building::Tower,
                }
                && site.roadside(|cell| self.roads.contains(&cell))
        });
        if !served {
            return Err(format!(
                "{} am örtlichen Weg oder Windstöße · Widerstand {}",
                Self::district_need(district.need),
                state.resolve
            ));
        }
        if !self.affordable(district.gift) {
            return Err(format!(
                "Gabe für {} · {}",
                district.name,
                stock_text(district.gift)
            ));
        }
        Ok(Outcome::Alliance)
    }

    fn claim_district(&mut self, region: &Region, index: usize) -> GameResult {
        let outcome = self.district_plan(region, index)?;
        let district = &region.districts[index];
        if outcome == Outcome::Alliance {
            self.pay(district.gift);
        }
        self.districts[index].outcome = Some(outcome);
        self.faith = self.faith.saturating_add(8);
        Ok(format!(
            "{} · {} · neuer Sammelplatz",
            district.name,
            outcome.name()
        ))
    }

    fn affordable(&self, stock: Stock) -> bool {
        self.stock
            .into_iter()
            .zip(stock)
            .all(|(owned, cost)| owned >= cost)
    }

    fn pay(&mut self, stock: Stock) {
        for (owned, cost) in self.stock.iter_mut().zip(stock) {
            *owned -= cost;
        }
    }

    fn can_buy_goods(&self, goods: Goods) -> bool {
        self.society
            .goods
            .into_iter()
            .zip(goods)
            .all(|(owned, price)| owned >= price)
    }

    fn pay_goods(&mut self, goods: Goods) {
        for (owned, price) in self.society.goods.iter_mut().zip(goods) {
            *owned -= price;
        }
    }

    fn can_deposit(&self, material: Material, count: u16) -> bool {
        self.stock[material.index()].saturating_add(count) <= self.storage_cap()
    }

    pub fn placement(&self, region: &Region, kind: Building, tile: Tile) -> Result<(), String> {
        if kind
            .civic()
            .is_some_and(|civic| !civic.available_in(region.kind))
        {
            return Err("Dieser Bauplan gehört zu einer anderen Landschaft.".into());
        }
        if !region.sandbox {
            match kind {
                Building::Shrine if self.count(kind) > 0 => {
                    return Err("Der Grenzschrein steht bereits.".into());
                }
                Building::Shrine if tile != region.frontier => {
                    return Err("Wähle den goldenen Schreinplatz am Übergang.".into());
                }
                Building::Shrine if self.quest != Quest::Bridge || self.faith < region.faith => {
                    return Err(format!(
                        "Für den Grenzschrein brauchst du den dritten Auftrag und {} Glauben.",
                        region.faith
                    ));
                }
                Building::Workshop | Building::Well if self.city_rank() < 2 => {
                    return Err("Dieses Gebäude wird auf Dorfstufe 2 verfügbar.".into());
                }
                Building::Granary | Building::Market | Building::Tower if self.city_rank() < 3 => {
                    return Err("Baue eine Werkstatt für diese Gebäudeart.".into());
                }
                Building::Waystone if self.outcome.is_none() => {
                    return Err("Der Grenzstein wird nach dem Hauptübergang verfügbar.".into());
                }
                Building::Waystone if self.faith < WAYSTONE_FAITH => {
                    return Err(format!("Ein Grenzstein braucht {WAYSTONE_FAITH} Glauben."));
                }
                Building::Waystone if self.count(kind) >= WAYSTONE_LIMIT => {
                    return Err("Hier sind bereits alle Grenzsteine gesetzt.".into());
                }
                Building::Waystone
                    if !tile
                        .neighbors()
                        .into_iter()
                        .any(|cell| self.road_at(region, cell) && self.walkable(region, cell)) =>
                {
                    return Err(
                        "Setze den Grenzstein neben einen Weg an der goldenen Grenze.".into(),
                    );
                }
                _ => {}
            }
        }
        match kind {
            Building::Civic(Civic::Port | Civic::WaterTerrace)
                if !kind
                    .tiles(tile)
                    .flat_map(Tile::neighbors)
                    .any(|cell| region.water(cell)) =>
            {
                return Err("Dieses Gebäude braucht einen Bauplatz am Wasser.".into());
            }
            Building::Civic(Civic::Gate) if self.gate_axes(region, tile) == [false, false] => {
                return Err("Das Tor braucht ein gerades Wegstück mit freiem Durchgang.".into());
            }
            _ => {}
        }
        for cell in kind.tiles(tile) {
            if !self.influence(region, cell) {
                return Err("Der ganze Grundriss muss im Einflussgebiet liegen.".into());
            }
            if region.water(cell)
                || !self.walkable(region, cell)
                || self.building_at(cell).is_some()
                || (!kind.gate() && self.road_at(region, cell))
                || (!region.sandbox && cell == region.frontier && kind != Building::Shrine)
                || self.walkers.iter().any(|walker| walker.tile() == cell)
            {
                return Err("Suche freien Boden neben dem Weg.".into());
            }
        }
        if !region.sandbox
            && (!self.affordable(kind.price()) || !self.can_buy_goods(kind.goods_price()))
        {
            return Err(format!(
                "Es fehlen Baustoffe. Benötigt werden {}.",
                kind.cost_text()
            ));
        }
        if kind.gate() {
            return Ok(());
        }
        // Check that the footprint leaves each currently reachable resource accessible.
        let depot_edge = region.depot.offset(1, 0);
        let new_footprint = |p| kind.tiles(tile).any(|cell| cell == p);
        let bounds = region.bounds;
        let walk = self.walk_grid(region);
        let before = Self::reachable(region, &walk, depot_edge, |_| false);
        let after = Self::reachable(region, &walk, depot_edge, new_footprint);
        if region.sources.iter().any(|source| {
            let accessible = |mask: &[bool]| {
                source
                    .tile
                    .neighbors()
                    .into_iter()
                    .filter_map(|p| bounds.index(p))
                    .any(|index| mask[index])
            };
            accessible(&before) && !accessible(&after)
        }) || self.walkers.iter().any(|walker| {
            bounds
                .index(walker.tile())
                .is_some_and(|index| before[index] && !after[index])
        }) {
            return Err("Diese Stelle würde einen Sammelweg versperren.".into());
        }
        Ok(())
    }

    fn walk_grid(&self, region: &Region) -> Vec<bool> {
        let bounds = region.bounds;
        let mut walk = region.walk_grid(self.bridge_fixed);
        for tile in self
            .buildings
            .iter()
            .filter(|site| !site.kind.gate())
            .flat_map(|site| site.kind.tiles(site.tile))
        {
            if let Some(index) = bounds.index(tile) {
                walk[index] = false;
            }
        }
        walk
    }

    fn gate_axes(&self, region: &Region, tile: Tile) -> [bool; 2] {
        if !self.road_at(region, tile) {
            return [false, false];
        }
        let road = |cell| self.road_at(region, cell) && self.walkable(region, cell);
        [
            road(tile.offset(0, -1)) && road(tile.offset(0, 1)),
            road(tile.offset(-1, 0)) && road(tile.offset(1, 0)),
        ]
    }

    pub fn build_rotation(&self, region: &Region, kind: Building, tile: Tile, rotation: u8) -> u8 {
        if !kind.gate() {
            return rotation % 4;
        }
        match self.gate_axes(region, tile) {
            [true, false] => 0,
            [false, true] => 1,
            _ => rotation % 2,
        }
    }

    fn reachable(
        region: &Region,
        walk: &[bool],
        start: Tile,
        blocked: impl Fn(Tile) -> bool,
    ) -> Vec<bool> {
        let bounds = region.bounds;
        let mut seen = vec![false; bounds.len()];
        let Some(index) = bounds
            .index(start)
            .filter(|&index| walk[index] && !blocked(start))
        else {
            return seen;
        };
        let mut queue = VecDeque::from([start]);
        seen[index] = true;
        while let Some(p) = queue.pop_front() {
            for next in p.neighbors() {
                if let Some(index) = bounds
                    .index(next)
                    .filter(|&index| walk[index] && !seen[index] && !blocked(next))
                {
                    seen[index] = true;
                    queue.push_back(next);
                }
            }
        }
        seen
    }

    fn route_to(
        &self,
        region: &Region,
        start: Tile,
        target: Tile,
        beside: bool,
    ) -> Option<VecDeque<Tile>> {
        let bounds = region.bounds;
        let start_index = bounds.index(start)?;
        bounds.index(target)?;
        if start.distance(target) == u16::from(beside) {
            return Some(VecDeque::new());
        }
        let walk = self.walk_grid(region);
        let mut paved = vec![false; bounds.len()];
        for &tile in &self.roads {
            if let Some(index) = bounds.index(tile) {
                paved[index] = true;
            }
        }
        let remaining =
            |tile: Tile| u32::from(tile.distance(target)).saturating_sub(u32::from(beside)) * 2;
        let mut came_from = vec![None; bounds.len()];
        let mut score = vec![u32::MAX; bounds.len()];
        score[start_index] = 0;
        let mut queue = BinaryHeap::from([Reverse((remaining(start), remaining(start), 0, start))]);
        let end = loop {
            let Reverse((_, _, cost, p)) = queue.pop()?;
            if cost > score[bounds.index(p)?] {
                continue;
            }
            if p.distance(target) == u16::from(beside) {
                break p;
            }
            for next in p.neighbors() {
                if let Some(index) = bounds.index(next).filter(|&index| walk[index]) {
                    let next_cost = cost
                        + if paved[index] || region.road(next) {
                            2
                        } else {
                            3
                        };
                    if next_cost >= score[index] {
                        continue;
                    }
                    score[index] = next_cost;
                    came_from[index] = Some(p);
                    let estimate = remaining(next);
                    queue.push(Reverse((next_cost + estimate, estimate, next_cost, next)));
                }
            }
        };
        let mut route = VecDeque::new();
        let mut p = end;
        while p != start {
            route.push_front(p);
            p = came_from[bounds.index(p)?]?;
        }
        Some(route)
    }

    fn take(&mut self, source: usize, count: u16) -> Option<Bundle> {
        let count = count.min(self.nodes[source]);
        if count == 0 {
            return None;
        }
        self.nodes[source] -= count;
        Some(Bundle { source, count })
    }

    fn deposit(&mut self, bundle: Bundle, region: &Region) -> Result<(), String> {
        let material = region.sources[bundle.source].material;
        if !self.can_deposit(material, bundle.count) {
            return Err(
                "Das Lager ist voll. Baue ein weiteres Lager oder verbrauche Vorräte.".into(),
            );
        }
        self.stock[material.index()] += bundle.count;
        Ok(())
    }

    fn stores<'a>(&'a self, region: &'a Region) -> impl Iterator<Item = Tile> + 'a {
        std::iter::once(region.depot)
            .chain(
                region
                    .areas
                    .iter()
                    .zip(self.camps)
                    .filter_map(|(area, active)| active.then_some(area.center)),
            )
            .chain(
                region
                    .districts
                    .iter()
                    .zip(&self.districts)
                    .filter(|(_, state)| state.outcome.is_some())
                    .map(|(district, _)| district.center),
            )
            .chain(
                self.buildings
                    .iter()
                    .filter(|site| {
                        matches!(
                            site.kind,
                            Building::Store | Building::Granary | Building::Civic(Civic::Port)
                        )
                    })
                    .flat_map(|site| site.kind.tiles(site.tile)),
            )
    }

    pub fn nearest_store(&self, region: &Region, start: Tile) -> Tile {
        self.stores(region)
            .min_by_key(|&tile| start.distance(tile))
            .unwrap_or(region.depot)
    }

    fn route_to_store(&self, region: &Region, start: Tile) -> Option<VecDeque<Tile>> {
        let mut stores: Vec<_> = self.stores(region).collect();
        stores.sort_by_key(|&tile| start.distance(tile));
        stores
            .into_iter()
            .find_map(|tile| self.route_to(region, start, tile, true))
    }

    fn area_collection(
        &self,
        region: &Region,
        area: usize,
        material: Material,
        start: Tile,
    ) -> Option<(usize, VecDeque<Tile>)> {
        let mut candidates: Vec<_> = region
            .area_sources(area, material)
            .filter(|(index, _)| self.nodes[*index] > 0)
            .collect();
        candidates.sort_by_key(|(_, source)| start.distance(source.tile));
        candidates.into_iter().find_map(|(index, source)| {
            self.route_to(region, start, source.tile, true)
                .map(|route| (index, route))
        })
    }

    pub fn day_phase(&self) -> DayPhase {
        let phase = self.weather_time.rem_euclid(DAY_LENGTH);
        if phase < 34. {
            DayPhase::Morning
        } else if phase < 111. {
            DayPhase::Work
        } else if phase < 150. {
            DayPhase::Evening
        } else {
            DayPhase::Night
        }
    }

    pub fn guard_world(&self, region: &Region, index: usize) -> Vec3 {
        let origin = region.guards[index];
        if self.outcome.is_some() || self.quest.index() < Quest::Bridge.index() {
            return origin.world();
        }
        let post = |index: usize| match index % 4 {
            1 => origin.offset(-1, 0),
            3 => origin.offset(1, 0),
            _ => origin,
        };
        let time = (self.weather_time * 0.24 + index as f32 * 0.9).rem_euclid(4.);
        let leg = time as usize;
        let safe = |position: Tile| {
            if self.walkable(region, position) {
                position
            } else {
                origin
            }
        };
        safe(post(leg))
            .world()
            .lerp(safe(post(leg + 1)).world(), time.fract())
    }

    fn raids_active(&self) -> bool {
        self.quest == Quest::Bridge
            && self.count(Building::Shrine) > 0
            && self.guards.iter().any(|&spirit| spirit > 0)
    }

    pub fn next_raid(&self) -> Option<f32> {
        (self.raids_active() && self.raid_clock > 0.).then_some(self.raid_clock)
    }

    fn daily_route(&self, walker: &mut Walker, region: &Region, index: usize) {
        if walker.hippo || walker.guard {
            return;
        }
        let phase = self.day_phase();
        let sites = self.buildings.iter().filter(|site| match phase {
            DayPhase::Morning => false,
            DayPhase::Work => matches!(site.kind, Building::Workshop | Building::Field),
            DayPhase::Evening => {
                matches!(site.kind, Building::Well | Building::Civic(Civic::Tavern))
            }
            DayPhase::Night => site.kind == Building::House,
        });
        let fallback = if phase == DayPhase::Night {
            region.start
        } else {
            region.depot
        };
        let destination = sites
            .clone()
            .nth(index % sites.clone().count().max(1))
            .and_then(|site| {
                site.kind
                    .tiles(site.tile)
                    .flat_map(Tile::neighbors)
                    .find(|&tile| self.road_at(region, tile) && self.walkable(region, tile))
            });
        let target = destination.unwrap_or(fallback.offset(
            if phase == DayPhase::Night {
                0
            } else {
                (index % 3) as i16 - 1
            },
            if phase == DayPhase::Night {
                (index % 3) as i16
            } else {
                0
            },
        ));
        let target = if self.road_at(region, target) && self.walkable(region, target) {
            target
        } else {
            region.depot
        };
        if let Some(route) = self.route_to(region, walker.tile(), target, false) {
            walker.route = route;
        }
        if walker.route.is_empty() {
            walker.pause = 6. + (index % 4) as f32 * 2.;
        }
    }

    fn advance_walker(
        &mut self,
        walker: &mut Walker,
        region: &Region,
        gifts: Gifts,
        dt: f32,
        index: usize,
    ) {
        if walker.childhood > 0. {
            walker.childhood = (walker.childhood - dt).max(0.);
            return;
        }
        if walker.hippo {
            if self.energy <= 9. {
                self.recovering = true;
            }
            let moving = !walker.route.is_empty() || walker.bundle.is_some();
            self.energy = if moving && !self.recovering {
                (self.energy - dt * if gifts.training >= 3 { 1.7 } else { 2.6 }).max(0.)
            } else {
                let aid = if self.service(region, Civic::Infirmary) {
                    3.
                } else {
                    0.
                };
                (self.energy + dt * (aid + if self.recovering { 7. } else { 4. }))
                    .min(full_energy())
            };
            if self.recovering {
                self.recovering = self.energy < 38.;
                walker.stride = 0.;
                return;
            }
        }
        walker.pause = (walker.pause - dt).max(0.);
        if walker.pause > 0. {
            walker.stride = 0.;
            return;
        }
        if !walker.route.is_empty() {
            walker.stride += (1. - walker.stride) * (1. - (-9. * dt).exp());
            let mut remaining = dt;
            while let Some(step) = walker.route.front().copied() {
                if remaining <= 0. {
                    break;
                }
                if !self.walkable(region, step) {
                    walker.route.clear();
                    break;
                }
                let point = Vec2::from_array(walker.position);
                let delta = step.local() - point;
                let speed = if walker.hippo {
                    2.65 + f32::from(gifts.training >= 4) * 0.41
                } else if walker.guard
                    && self.buildings.iter().any(|site| {
                        site.kind == Building::Civic(Civic::Beacon)
                            && step.within(site.tile, 8)
                            && self.connected(region, site)
                    })
                {
                    3.15
                } else {
                    2.25
                };
                let remaining_step = if walker.route.len() == 1 {
                    0.71 + 0.29 * (delta.length() / 0.7).clamp(0., 1.)
                } else {
                    1.
                };
                let pace = (0.68 + 0.32 * walker.stride) * remaining_step;
                let speed = speed * pace * if self.road_at(region, step) { 1.5 } else { 1. };
                let travel = delta.length() / speed;
                if travel > remaining {
                    walker.position = (point + delta * (remaining / travel)).to_array();
                    break;
                }
                walker.position = step.local().to_array();
                walker.route.pop_front();
                remaining -= travel;
            }
            return;
        }
        walker.stride = 0.;

        if let Some(bundle) = walker.bundle {
            if let Some(route) = self.route_to_store(region, walker.tile()) {
                if !route.is_empty() {
                    walker.route = route;
                    return;
                }
                if self.deposit(bundle, region).is_ok() {
                    walker.bundle = None;
                    walker.pause = 0.45;
                    if walker.hippo {
                        self.hippo_deliveries = self.hippo_deliveries.saturating_add(1);
                        let completed = walker.job.filter(|job| match *job {
                            Job::Gather(source) => source == bundle.source,
                            Job::GatherArea(area, material) => region
                                .area_sources(area, material)
                                .any(|(source, _)| source == bundle.source),
                            _ => false,
                        });
                        self.lesson = Some(match completed {
                            Some(job @ Job::GatherArea(..)) => Lesson::Practiced(job),
                            _ => Lesson::Delivery(bundle.source),
                        });
                        if gifts.training == 0 && completed.is_some() {
                            walker.job = None;
                        }
                    } else {
                        self.helper_deliveries = self.helper_deliveries.saturating_add(1);
                    }
                    return;
                }
            }
            walker.pause = 1.;
            return;
        }

        match walker.job {
            Some(Job::Move(target)) => {
                let route = self.route_to(region, walker.tile(), target, false);
                walker.route = route.unwrap_or_default();
                if walker.route.is_empty() {
                    walker.job = None;
                }
            }
            Some(job @ (Job::Gather(_) | Job::GatherArea(..))) => {
                let next = match job {
                    Job::Gather(source) if self.nodes[source] > 0 => self
                        .route_to(region, walker.tile(), region.sources[source].tile, true)
                        .map(|route| (source, route)),
                    Job::GatherArea(area, material) if self.can_deposit(material, 1) => {
                        self.area_collection(region, area, material, walker.tile())
                    }
                    _ => None,
                };
                let Some((source, route)) = next else {
                    walker.pause = match job {
                        Job::Gather(source) if self.nodes[source] > 0 => 1.,
                        _ => 2.,
                    };
                    return;
                };
                if route.is_empty() {
                    let mut count = if walker.hippo {
                        5 + u16::from(gifts.training) * 2
                    } else {
                        3
                    };
                    if let Job::GatherArea(_, material) = job {
                        count = count.min(self.storage_cap() - self.stock[material.index()]);
                    }
                    walker.bundle = self.take(source, count);
                    walker.pause = if walker.bundle.is_some() { 0.65 } else { 1.5 };
                } else {
                    walker.route = route;
                }
            }
            Some(Job::Watch(target)) if walker.tile() != target => {
                if let Some(route) = self.route_to(region, walker.tile(), target, false) {
                    walker.route = route;
                }
                walker.pause = 0.2;
            }
            Some(Job::Watch(target)) => {
                if self.outcome.is_some() {
                    for (district, state) in region.districts.iter().zip(&mut self.districts) {
                        if state.outcome.is_none()
                            && target.within(district.center, 3)
                            && state.resolve > 0
                        {
                            state.resolve -= 1;
                            if walker.hippo {
                                self.lesson = Some(Lesson::Practiced(Job::Watch(target)));
                            }
                        }
                    }
                } else if walker.hippo
                    && self.count(Building::Shrine) > 0
                    && target.within(region.bridge(), 4)
                {
                    self.lesson = Some(Lesson::Practiced(Job::Watch(target)));
                }
                walker.pause = 10.;
            }
            Some(Job::Tend(target)) => {
                if let Some(route) = self.route_to(region, walker.tile(), target, true) {
                    if !route.is_empty() {
                        walker.route = route;
                        return;
                    }
                    if self.harvest(region, target).is_ok() {
                        self.lesson = Some(Lesson::Practiced(Job::Tend(target)));
                        self.energy = (self.energy - 13.).max(0.);
                    } else if let Some(site) = self
                        .buildings
                        .iter_mut()
                        .find(|site| site.tile == target && site.kind == Building::Field)
                    {
                        if matches!(site.crop, Crop::Dry) {
                            site.crop = Crop::Growing(13.);
                        }
                    }
                }
                walker.pause = 2.5;
            }
            None => self.daily_route(walker, region, index),
        }
    }

    fn tick(&mut self, region: &Region, gifts: Gifts, dt: f32) -> u16 {
        let base = if self.count(Building::Shrine) > 0 {
            1.70
        } else {
            1.0
        };
        let recharge = base
            + if self.service(region, Civic::Oracle) {
                0.55
            } else {
                0.
            };
        self.mana = (self.mana + recharge * dt).min(self.mana_cap(region, gifts));
        self.shield = (self.shield - dt).max(0.);
        self.rainfall = (self.rainfall - dt).max(0.);
        self.weather_time += dt;
        let weather = region.weather(self.weather_time);
        for cooldown in &mut self.cooldowns {
            *cooldown = (*cooldown - dt).max(0.);
        }

        for (index, (stock, delay)) in self.nodes.iter_mut().zip(&mut self.regrowth).enumerate() {
            if *stock == 0 && !self.held.is_some_and(|bundle| bundle.source == index) {
                let renewal = if index < region.original_sources {
                    28.
                } else {
                    match region.sources[index].material {
                        Material::Wood => 64.,
                        Material::Stone => 86.,
                        Material::Food => 38.,
                    }
                };
                let rain = weather == Weather::Showers || self.rainfall > 0.;
                *delay += if rain && region.sources[index].material != Material::Stone {
                    dt * 1.28
                } else {
                    dt
                };
                if *delay >= renewal {
                    *stock = region.sources[index].stock;
                    *delay = 0.;
                }
            }
        }
        let mut seeds = Vec::new();
        let mut wells = Vec::new();
        for site in self
            .buildings
            .iter()
            .filter(|site| self.connected(region, site))
        {
            match site.kind {
                Building::Well => wells.push((site.tile, if site.upgraded { 8 } else { 5 })),
                Building::Civic(Civic::WaterTerrace) => wells.push((site.tile, 10)),
                Building::Civic(Civic::SeedPavilion) => seeds.push(site.tile),
                _ => {}
            }
        }
        for site in &mut self.buildings {
            let watered = wells
                .iter()
                .any(|&(well, radius)| site.tile.within(well, radius));
            let growth_step =
                dt * match weather {
                    Weather::Drought if !watered => 0.55,
                    Weather::Mist => 1.22,
                    _ => 1.,
                } * if seeds.iter().any(|seed| site.tile.within(*seed, 9)) {
                    1.36
                } else {
                    1.
                };
            if let Crop::Growing(remaining) = site.crop {
                site.crop = if remaining <= growth_step {
                    Crop::Ripe
                } else {
                    Crop::Growing(remaining - growth_step)
                };
            } else if site.kind == Building::Field
                && self.harvests > 0
                && matches!(site.crop, Crop::Dry)
                && (watered || weather == Weather::Showers)
            {
                site.crop = Crop::Growing(if region.kind == Biome::Lake { 24. } else { 17. });
            }
        }

        let mut walkers = std::mem::take(&mut self.walkers);
        for (index, walker) in walkers.iter_mut().enumerate() {
            self.advance_walker(walker, region, gifts, dt, index);
        }
        self.walkers = walkers;

        self.provisions -= dt;
        let mut insight = 0;
        if self.provisions <= 0. {
            self.provisions = match region.kind {
                Biome::Fjord => 17.,
                Biome::Lake => 19.,
                Biome::Aegean if weather == Weather::Breeze => 20.,
                _ => 24.,
            };
            let winter_cost = u16::from(
                weather == Weather::Snow
                    && !self.buildings.iter().any(|site| {
                        matches!(
                            site.kind,
                            Building::Granary | Building::Civic(Civic::WinterHall)
                        ) && self.connected(region, site)
                    }),
            );
            let food = &mut self.stock[Material::Food.index()];
            if *food > 4 {
                *food = (*food - (1 + winter_cost + self.walkers.len() as u16 / 7)).max(4);
            }
            self.trade(region);
            insight = self.civic_life(region, weather);
        }

        if self.raids_active() {
            if self.raid_clock <= 0. {
                self.raid_clock = 34.;
            }
            self.raid_clock -= dt;
            if self.raid_clock <= 0. {
                self.raid_clock = 29.;
                let protected = self.shield > 0.
                    || self.tower_protects(region, region.bridge())
                    || self.walkers.iter().any(|walker| {
                        (walker.hippo || walker.guard)
                            && matches!(walker.job, Some(Job::Watch(_)))
                            && walker.tile().within(region.bridge(), 4)
                    });
                if !protected {
                    let food = &mut self.stock[Material::Food.index()];
                    *food -= food.saturating_sub(4).min(2);
                }
            }
        }
        insight
    }

    fn exchange(&mut self, input: Stock, goods: Goods, output: Stock, product: Goods) -> bool {
        if !self.affordable(input) || !self.can_buy_goods(goods) {
            return false;
        }
        let stock_cap = self.storage_cap();
        let goods_cap = 32 + 8 * self.count(Building::Store).min(20) as u16;
        let next_stock =
            std::array::from_fn(|index| self.stock[index] - input[index] + output[index]);
        let next_goods =
            std::array::from_fn(|index| self.society.goods[index] - goods[index] + product[index]);
        if next_stock.iter().any(|&quantity| quantity > stock_cap)
            || next_goods.iter().any(|&quantity| quantity > goods_cap)
        {
            return false;
        }
        self.stock = next_stock;
        self.society.goods = next_goods;
        true
    }

    fn civic_life(&mut self, region: &Region, weather: Weather) -> u16 {
        let mut active: Vec<_> = self
            .buildings
            .iter()
            .filter_map(|site| {
                site.kind
                    .civic()
                    .filter(|_| self.connected(region, site))
                    .map(|kind| (kind, site.tile, site.upgraded))
            })
            .collect();
        active.sort_by_key(|(kind, _, _)| *kind as u8);
        let mut morale = if self.stock[2] >= 6 { 60 } else { 49 };
        let mut insight: u16 = 0;
        for (kind, tile, upgraded) in active {
            let supplies = self.stock[2] >= 8;
            match kind {
                Civic::Courtyard | Civic::Infirmary => morale += 3,
                Civic::Bath
                    if self.buildings.iter().any(|site| {
                        site.kind == Building::Well
                            && site.tile.within(tile, 6)
                            && self.connected(region, site)
                    }) =>
                {
                    morale += 9
                }
                Civic::Sawmill => {
                    self.exchange([2, 0, 0], [0; 3], [0; 3], [1, 0, 0]);
                }
                Civic::Masonry => {
                    self.exchange([0, 2, 0], [0; 3], [0; 3], [0, 1, 0]);
                }
                Civic::Bakery if self.harvests > 0 => {
                    self.exchange([1, 0, 1], [0; 3], [0, 0, 4], [0; 3]);
                }
                Civic::Forge => {
                    self.exchange(
                        [0, 1, 0],
                        [1, 1, 0],
                        [0; 3],
                        [0, 0, 1 + u16::from(upgraded)],
                    );
                }
                Civic::Port => {
                    let haul = if weather == Weather::Breeze {
                        [0, 6, 7]
                    } else {
                        [0, 4, 5]
                    };
                    self.exchange([0; 3], [0, 0, 1], haul, [0; 3]);
                }
                Civic::Garden => morale += 5,
                Civic::Tavern if supplies => {
                    self.stock[2] -= 1;
                    morale += 8;
                }
                Civic::Hall if supplies && self.affordable([1, 0, 2]) => {
                    self.pay([1, 0, 2]);
                    morale += 11;
                    self.faith = self.faith.saturating_add(3);
                }
                Civic::Theater if supplies => {
                    self.stock[2] -= 1;
                    morale += 10;
                    insight = insight.saturating_add(1);
                }
                Civic::Archive if self.stock[0] >= 5 => {
                    self.stock[0] -= 1;
                    insight = insight.saturating_add(1);
                }
                Civic::Forester => {
                    for (source, stock) in region.sources.iter().zip(&mut self.nodes) {
                        if source.material == Material::Wood && source.tile.within(tile, 9) {
                            *stock = source.stock.min(stock.saturating_add(4));
                        }
                    }
                    self.stock[0] = self.storage_cap().min(self.stock[0].saturating_add(2));
                }
                Civic::WinterHall if weather == Weather::Snow => morale += 11,
                Civic::OlivePress if supplies => {
                    self.stock[2] -= 1;
                    self.olive_pressings = self.olive_pressings.saturating_add(1);
                    self.faith =
                        self.faith
                            .saturating_add(if weather == Weather::Breeze { 5 } else { 3 });
                }
                Civic::Temple => self.faith = self.faith.saturating_add(2),
                _ => {}
            }
        }
        self.society.morale = morale.min(100);
        self.new_child(region);
        insight
    }

    fn new_child(&mut self, region: &Region) {
        if self.walkers.len() >= MAX_WALKERS
            || self.inhabitants() >= self.housing()
            || self.society.morale < 55
        {
            return;
        }
        let nursery = self.service(region, Civic::Nursery);
        let food_cost = if nursery { 2 } else { 3 };
        if self.stock[2] < 4 + food_cost {
            return;
        }
        let Some(arrival) = self
            .buildings
            .iter()
            .filter(|site| {
                site.kind.civic().is_some_and(|kind| kind.beds() > 0)
                    && self.connected(region, site)
            })
            .find_map(|site| self.entrance(region, site))
        else {
            return;
        };
        self.stock[2] -= food_cost;
        let mut child = Walker::new(arrival, false);
        child.childhood = if nursery { 26. } else { 46. };
        self.walkers.push(child);
        self.society.births = self.society.births.saturating_add(1);
    }

    fn trade(&mut self, region: &Region) {
        if self.stock[2] < 4 {
            return;
        }
        let Some(market) = self
            .buildings
            .iter()
            .filter(|site| site.kind == Building::Market && self.connected(region, site))
            .max_by_key(|site| site.upgraded)
        else {
            return;
        };
        let cap = self.storage_cap();
        let material = self.last_market;
        if self.stock[material] < cap {
            self.stock[2] -= 2;
            self.stock[material] =
                cap.min(self.stock[material] + if market.upgraded { 6 } else { 4 });
            self.faith = self.faith.saturating_add(1);
        }
        self.last_market = 1 - material;
    }

    fn tower_protects(&self, region: &Region, point: Tile) -> bool {
        let fortress = self.buildings.iter().any(|site| {
            let range = match site.kind {
                Building::Tower => {
                    if site.upgraded {
                        9
                    } else {
                        5
                    }
                }
                Building::Civic(Civic::Bastion) => {
                    if site.upgraded {
                        14
                    } else {
                        10
                    }
                }
                _ => return false,
            };
            site.tile.within(point, range) && self.connected(region, site)
        });
        fortress
            || self
                .buildings
                .iter()
                .filter(|site| {
                    matches!(site.kind, Building::Civic(Civic::Palisade | Civic::Gate))
                        && site.tile.within(point, 5)
                })
                .take(3)
                .count()
                >= 3
    }

    fn hand(&mut self, region: &Region, target: Tile) -> GameResult {
        if !self.influence(region, target) {
            return Err("Die Wolkenpfote reicht nur bis zur goldenen Grenze.".into());
        }
        if let Some(bundle) = self.held {
            if !self.can_store_at(region, target) {
                return Err("Wähle einen Sammelplatz oder ein Lager zum Ablegen.".into());
            }
            self.deposit(bundle, region)?;
            self.held = None;
            self.hand_deliveries = self.hand_deliveries.saturating_add(1);
            return Ok(format!(
                "{} {} abgelegt",
                bundle.count,
                region.kind.material(region.sources[bundle.source].material)
            ));
        }
        if let Some(index) = region.source_at(target) {
            let Some(bundle) = self.take(index, 6) else {
                return Err("Diese Quelle erholt sich. Wähle einen anderen Vorrat.".into());
            };
            self.held = Some(bundle);
            return Ok(format!(
                "{} {} in der Pfote. Wähle den Sammelplatz.",
                bundle.count,
                region.kind.material(region.sources[index].material)
            ));
        }
        self.harvest(region, target).map(|amount| {
            format!("{amount} Nahrung nach Hause gebracht. Das Dorf schöpft Glauben.")
        })
    }

    fn harvest(&mut self, region: &Region, target: Tile) -> Result<u16, String> {
        let food = Material::Food.index();
        let room = self.storage_cap().saturating_sub(self.stock[food]);
        if let Some(site) = self
            .buildings
            .iter_mut()
            .find(|site| site.kind == Building::Field && site.contains(target))
        {
            if !matches!(site.crop, Crop::Ripe) {
                return Err("Ein Regenschauer lässt das Feld wachsen.".into());
            }
            let amount = room.min(
                if region.kind == Biome::Terraces {
                    12
                } else {
                    10
                } + if site.upgraded { 6 } else { 0 },
            );
            if amount == 0 {
                return Err("Schaffe Platz im Lager für die Ernte.".into());
            }
            site.crop = Crop::Dry;
            self.stock[food] += amount;
            self.harvests = self.harvests.saturating_add(1);
            self.faith = self.faith.saturating_add(5);
            return Ok(amount);
        }
        Err("Wähle einen markierten Vorrat oder ein reifes Feld.".into())
    }

    fn build(&mut self, region: &Region, kind: Building, tile: Tile, rotation: u8) -> GameResult {
        self.placement(region, kind, tile)?;
        let rotation = self.build_rotation(region, kind, tile, rotation);
        if !region.sandbox {
            self.pay(kind.price());
            self.pay_goods(kind.goods_price());
        }
        self.buildings.push(BuildingSite {
            kind,
            tile,
            rotation,
            crop: Crop::Dry,
            upgraded: false,
        });
        for walker in &mut self.walkers {
            walker.route.clear();
        }
        if kind == Building::House {
            self.welcome_resident(region, self.buildings.len() - 1);
        }
        if kind == Building::Waystone && !region.sandbox {
            self.faith -= WAYSTONE_FAITH;
        } else {
            self.faith = self
                .faith
                .saturating_add(if kind == Building::Shrine { 3 } else { 2 });
        }
        let linked = kind.civic().is_some_and(|civic| civic != Civic::Palisade)
            && !self.connected(region, self.buildings.last().unwrap());
        Ok(if linked {
            format!("{} gebaut · Anschluss an einen Weg fehlt", kind.name())
        } else {
            format!("{} gebaut", kind.name())
        })
    }

    fn welcome_resident(&mut self, region: &Region, index: usize) {
        if self.walkers.len() >= MAX_WALKERS {
            return;
        }
        if let Some(arrival) = self.entrance(region, &self.buildings[index]) {
            self.walkers.push(Walker::new(arrival, false));
        }
    }

    fn entrance(&self, region: &Region, site: &BuildingSite) -> Option<Tile> {
        site.kind
            .tiles(site.tile)
            .flat_map(Tile::neighbors)
            .filter(|&tile| self.walkable(region, tile))
            .min_by_key(|&tile| !self.road_at(region, tile))
    }

    fn gathering_helper(&self) -> Result<usize, &'static str> {
        let eligible: Vec<_> = self
            .walkers
            .iter()
            .enumerate()
            .filter(|(_, walker)| !walker.hippo && !walker.guard && walker.childhood <= 0.)
            .collect();
        if eligible.is_empty() {
            return Err("Es gibt gerade keine erwachsenen Gläubigen für diesen Auftrag.");
        }
        eligible
            .iter()
            .cycle()
            .skip(self.next_worker % eligible.len())
            .take(eligible.len())
            .find_map(|(index, walker)| walker.job.is_none().then_some(*index))
            .ok_or("Alle Gläubigen sind beschäftigt. Unter Gläubige kannst du sie zurückrufen.")
    }

    pub fn idle_helper(&self) -> bool {
        self.walkers.iter().any(|walker| {
            !walker.hippo && !walker.guard && walker.childhood <= 0. && walker.job.is_none()
        })
    }

    pub fn gathering_helpers(&self) -> usize {
        self.walkers
            .iter()
            .filter(|walker| {
                !walker.hippo && matches!(walker.job, Some(Job::Gather(_) | Job::GatherArea(..)))
            })
            .count()
    }

    fn recall_helpers(&mut self) -> GameResult {
        if self.gathering_helpers() == 0 {
            return Err("Gerade sammeln keine Gläubigen.".into());
        }
        for walker in &mut self.walkers {
            if !walker.hippo && matches!(walker.job, Some(Job::Gather(_) | Job::GatherArea(..))) {
                walker.job = None;
                if walker.bundle.is_none() {
                    walker.route.clear();
                    walker.pause = 0.;
                }
            }
        }
        Ok("Gläubige freigegeben. Getragene Ladungen kommen noch an.".into())
    }

    fn move_hippo(&mut self, region: &Region, target: Tile) -> GameResult {
        if !self.walkable(region, target) {
            return Err("Wähle freien, begehbaren Boden.".into());
        }
        let route = self
            .route_to(region, self.walkers[0].tile(), target, false)
            .ok_or("Dorthin führt noch kein Weg.")?;
        let hippo = &mut self.walkers[0];
        hippo.pause = 0.;
        hippo.job = Some(Job::Move(target));
        hippo.route = if hippo.bundle.is_some() {
            VecDeque::new()
        } else {
            route
        };
        Ok("Das Nilpferd kommt.".into())
    }

    fn dispatch_gatherer(&mut self, worker: usize, job: Job) {
        let walker = &mut self.walkers[worker];
        walker.job = Some(job);
        walker.route.clear();
        walker.pause = 0.;
        if !walker.hippo {
            self.next_worker = self.next_worker.wrapping_add(1);
        }
    }

    fn assign(&mut self, region: &Region, tile: Tile, hippo: bool) -> GameResult {
        let index = region
            .source_at(tile)
            .ok_or("Wähle Holz, Stein oder Nahrung für den Auftrag.")?;
        let worker = if hippo { 0 } else { self.gathering_helper()? };
        if self
            .route_to(region, self.walkers[worker].tile(), tile, true)
            .is_none()
        {
            return Err("Der Übergang zu dieser Quelle ist noch versperrt.".into());
        }
        self.dispatch_gatherer(worker, Job::Gather(index));
        Ok(format!(
            "{} holt {}.",
            if hippo {
                "Dein Nilpferd"
            } else {
                "Ein Gläubiger"
            },
            region.kind.material(region.sources[index].material),
        ))
    }

    fn assign_area(
        &mut self,
        region: &Region,
        area: usize,
        material: Material,
        hippo: bool,
    ) -> GameResult {
        let place = region.areas.get(area).ok_or("Wähle ein Rohstoffgebiet.")?;
        if !self.camps[area] {
            return Err("Baue im Gebiet ein Sammellager für dauerhafte Aufträge.".into());
        }
        let worker = if hippo { 0 } else { self.gathering_helper()? };
        let job = Job::GatherArea(area, material);
        if self.walkers[worker].job == Some(job) {
            return Err("Das Nilpferd sammelt diesen Rohstoff bereits.".into());
        }
        if self
            .route_to(region, self.walkers[worker].tile(), place.center, true)
            .is_none()
        {
            return Err("Der Weg zu diesem Sammellager ist versperrt.".into());
        }
        self.dispatch_gatherer(worker, job);
        let gatherer = if hippo {
            "Das Nilpferd"
        } else {
            "Ein Gläubiger"
        };
        Ok(format!(
            "{gatherer} sammelt {} im Gebiet {}.",
            region.kind.material(material),
            place.name
        ))
    }

    fn cancel_area(&mut self, area: usize, material: Material) -> GameResult {
        let job = Some(Job::GatherArea(area, material));
        let mut released = false;
        for walker in self.walkers.iter_mut().filter(|walker| walker.job == job) {
            walker.job = None;
            if walker.bundle.is_none() {
                walker.route.clear();
                walker.pause = 0.;
            }
            released = true;
        }
        if !released {
            return Err("Für diesen Rohstoff gibt es hier keinen Sammelauftrag.".into());
        }
        Ok("Aufträge beendet. Getragene Ladungen werden abgelegt.".into())
    }

    fn watch(&mut self, region: &Region, tile: Tile, gifts: Gifts) -> GameResult {
        if gifts.training < 2 {
            return Err("Auf Kreaturenrang 2 kann das Nilpferd Wache halten.".into());
        }
        if !self.influence(region, tile)
            || !self.walkable(region, tile)
            || self
                .route_to(region, self.walkers[0].tile(), tile, false)
                .is_none()
        {
            return Err("Wähle einen erreichbaren Platz für die Wache.".into());
        }
        let hippo = &mut self.walkers[0];
        hippo.job = Some(Job::Watch(tile));
        hippo.route.clear();
        hippo.pause = 0.;
        Ok("Das Nilpferd hält an dieser Stelle Wache.".into())
    }

    pub fn muster_plan(&self, region: &Region, tile: Tile) -> Result<Tile, String> {
        let barracks = self
            .building_at(tile)
            .filter(|site| site.kind == Building::Civic(Civic::Barracks))
            .ok_or("Wähle ein Wachhaus.")?;
        if !self.connected(region, barracks) {
            return Err("Das Wachhaus braucht einen Anschluss an einen Weg.".into());
        }
        let capacity = self
            .buildings
            .iter()
            .filter(|site| {
                matches!(site.kind, Building::Civic(Civic::Barracks | Civic::Bastion))
                    && self.connected(region, site)
            })
            .count()
            * 2;
        if self.garrison() >= capacity.min(8) || self.walkers.len() >= MAX_WALKERS {
            return Err("Die Wache ist voll besetzt.".into());
        }
        if !self.affordable([0, 0, 3]) || !self.can_buy_goods([0, 0, 1]) {
            return Err("Eine Wache braucht 3 Nahrung und 1 Werkzeug.".into());
        }
        self.entrance(region, barracks)
            .ok_or_else(|| "Am Wachhaus fehlt ein freier Ausgang.".into())
    }

    fn muster(&mut self, region: &Region, tile: Tile) -> GameResult {
        let arrival = self.muster_plan(region, tile)?;
        self.pay([0, 0, 3]);
        self.pay_goods([0, 0, 1]);
        let mut guard = Walker::new(arrival, false);
        guard.guard = true;
        guard.job = Some(Job::Watch(arrival));
        self.walkers.push(guard);
        Ok("Eine Dorfwache steht bereit. Wähle Gläubige und dann Patrouille.".into())
    }

    fn patrol(&mut self, region: &Region, tile: Tile) -> GameResult {
        if !self.influence(region, tile) || !self.walkable(region, tile) {
            return Err("Wähle einen erreichbaren Wachposten im Einflussgebiet.".into());
        }
        let mut choices: Vec<_> = self
            .walkers
            .iter()
            .enumerate()
            .filter(|(_, walker)| walker.guard)
            .collect();
        choices.sort_by_key(|(_, walker)| {
            (
                matches!(walker.job, Some(Job::Watch(target)) if walker.tile() != target),
                walker.tile().distance(tile),
            )
        });
        let (index, route) = choices
            .into_iter()
            .find_map(|(index, walker)| {
                self.route_to(region, walker.tile(), tile, false)
                    .map(|route| (index, route))
            })
            .ok_or("Eine erreichbare Dorfwache aus einem Wachhaus fehlt.")?;
        let walker = &mut self.walkers[index];
        walker.job = Some(Job::Watch(tile));
        walker.route = route;
        walker.pause = 0.;
        Ok("Die Dorfwache zieht zum Posten.".into())
    }

    fn tend(&mut self, region: &Region, tile: Tile, gifts: Gifts) -> GameResult {
        if gifts.training < 3 {
            return Err("Auf Kreaturenrang 3 kann das Nilpferd Felder pflegen.".into());
        }
        let target = self
            .building_at(tile)
            .filter(|site| site.kind == Building::Field)
            .map(|site| site.tile)
            .ok_or("Wähle ein Feld zum Pflegen und Ernten.")?;
        if self
            .route_to(region, self.walkers[0].tile(), target, true)
            .is_none()
        {
            return Err("Das Nilpferd braucht einen Weg zu diesem Feld.".into());
        }
        let hippo = &mut self.walkers[0];
        hippo.job = Some(Job::Tend(target));
        hippo.route.clear();
        hippo.pause = 0.;
        Ok("Das Nilpferd pflegt und erntet dieses Feld.".into())
    }

    fn feed(&mut self) -> GameResult {
        if self.energy >= 95. {
            return Err("Das Nilpferd hat genug Kraft.".into());
        }
        if !self.affordable([0, 0, 2]) {
            return Err("Für den Snack brauchst du 2 Nahrung.".into());
        }
        self.pay([0, 0, 2]);
        self.restore_energy(55.);
        Ok("Nach dem Snack hat das Nilpferd wieder Kraft.".into())
    }

    fn restore_energy(&mut self, amount: f32) {
        self.energy = (self.energy + amount).min(full_energy());
        if self.energy >= 38. {
            self.recovering = false;
        }
    }

    pub fn creature_status(&self) -> &'static str {
        if self.recovering {
            return "Erholt sich";
        }
        match self.walkers[0].job {
            Some(Job::Gather(_) | Job::GatherArea(..)) => "Sammelt",
            Some(Job::Move(_)) => "Unterwegs",
            Some(Job::Watch(_)) => "Hält Wache",
            Some(Job::Tend(_)) => "Pflegt ein Feld",
            None if self.walkers[0].bundle.is_some() => "Bringt eine Ladung heim",
            None => "Rastet",
        }
    }

    fn threat_in(&self, region: &Region, tile: Tile, radius: i16) -> bool {
        (self.quest == Quest::Bridge
            && self.count(Building::Shrine) > 0
            && self.guards.iter().enumerate().any(|(index, &spirit)| {
                spirit > 0 && Tile::from_world(self.guard_world(region, index)).within(tile, radius)
            }))
            || (self.outcome.is_some()
                && region
                    .districts
                    .iter()
                    .zip(&self.districts)
                    .any(|(district, state)| {
                        state.outcome.is_none()
                            && state.resolve > 0
                            && district.center.within(tile, radius)
                    }))
    }

    fn blooms_at(&self, region: &Region, tile: Tile, radius: i16) -> bool {
        self.buildings
            .iter()
            .any(|site| matches!(site.crop, Crop::Growing(_)) && site.tile.within(tile, radius))
            || region
                .sources
                .iter()
                .zip(&self.nodes)
                .any(|(source, &stock)| {
                    source.material != Material::Stone
                        && stock < source.stock
                        && source.tile.within(tile, radius)
                })
    }

    pub fn ritual_requirement(
        &self,
        region: &Region,
        spell: Miracle,
        gifts: Gifts,
    ) -> Option<&'static str> {
        match spell {
            _ if region.sandbox && spell.offensive() => Some("Nur in Kampagnen"),
            Miracle::Thunder if gifts.storm == 0 => Some("Talent Stimme des Donners"),
            Miracle::Ward if self.city_rank() < 3 => Some("Benötigt Werkstatt"),
            Miracle::Bloom if gifts.growth == 0 => Some("Talent Hände der Blüte"),
            Miracle::Mend if gifts.training == 0 => Some("Nach dem ersten Lob"),
            _ => None,
        }
    }

    pub fn ritual_status(
        &self,
        region: &Region,
        spell: Miracle,
        gifts: Gifts,
    ) -> Option<&'static str> {
        self.ritual_requirement(region, spell, gifts).or_else(|| {
            if self.mana + 0.001 < spell.cost() {
                Some("Wunderkraft lädt am Schrein")
            } else if self.cooldowns[spell as usize] > 0.001 {
                Some("Wunder erholt sich")
            } else {
                None
            }
        })
    }

    pub fn ritual_error(
        &self,
        region: &Region,
        spell: Miracle,
        tile: Tile,
        gifts: Gifts,
    ) -> Option<&'static str> {
        if let Some(reason) = self.ritual_status(region, spell, gifts) {
            return Some(reason);
        }
        if !self.influence(region, tile) {
            return Some("Das Ziel liegt außerhalb deines Einflusses.");
        }
        let radius = spell.radius(gifts);
        match spell {
            Miracle::Rain
                if !self.buildings.iter().any(|site| {
                    site.kind == Building::Field
                        && matches!(site.crop, Crop::Dry)
                        && site
                            .kind
                            .tiles(site.tile)
                            .any(|cell| cell.within(tile, radius))
                }) =>
            {
                Some("Richte den Regenkreis auf ein trockenes Feld.")
            }
            Miracle::Wind | Miracle::Thunder if !self.threat_in(region, tile, radius) => {
                Some("Ziele auf Wachen am Übergang oder eine widerständige Siedlung.")
            }
            Miracle::Bloom if !self.blooms_at(region, tile, radius) => {
                Some("Ziele auf wachsende Saat, Obst oder genutztes Holz.")
            }
            Miracle::Mend if self.energy >= 95. || !self.walkers[0].tile().within(tile, radius) => {
                Some("Ziele auf das müde Nilpferd, um seine Kraft zu erneuern.")
            }
            _ => None,
        }
    }

    fn cast(&mut self, region: &Region, spell: Miracle, tile: Tile, gifts: Gifts) -> GameResult {
        if let Some(reason) = self.ritual_error(region, spell, tile, gifts) {
            return Err(reason.into());
        }
        self.mana = (self.mana - spell.cost()).max(0.);
        self.cooldowns[spell as usize] = spell.recovery();
        let radius = spell.radius(gifts);
        match spell {
            Miracle::Rain => {
                for site in &mut self.buildings {
                    if site.kind == Building::Field
                        && matches!(site.crop, Crop::Dry)
                        && site
                            .kind
                            .tiles(site.tile)
                            .any(|cell| cell.within(tile, radius))
                    {
                        let growing = if region.kind == Biome::Fjord { 10. } else { 7. };
                        site.crop = Crop::Growing(growing);
                    }
                }
                self.rainfall = 3.6;
                Ok("Regen lässt die Saat keimen.".into())
            }
            Miracle::Wind | Miracle::Thunder => {
                let force = if spell == Miracle::Wind {
                    1
                } else {
                    1 + gifts.storm
                };
                for index in 0..self.guards.len() {
                    let guard = Tile::from_world(self.guard_world(region, index));
                    let spirit = &mut self.guards[index];
                    if guard.within(tile, radius) {
                        *spirit = spirit.saturating_sub(force);
                    }
                }
                if self.outcome.is_some() {
                    for (district, state) in region.districts.iter().zip(&mut self.districts) {
                        if state.outcome.is_none() && district.center.within(tile, radius) {
                            state.resolve = state.resolve.saturating_sub(force);
                        }
                    }
                }
                if self.raid_clock <= 0. {
                    self.raid_clock = 21.;
                }
                Ok(format!("{} · der Widerstand sinkt", spell.name()))
            }
            Miracle::Ward => {
                self.shield = 26.;
                Ok("Die Schutzglocke hält Überfälle ab.".into())
            }
            Miracle::Bloom => {
                for site in &mut self.buildings {
                    if matches!(site.crop, Crop::Growing(_)) && site.tile.within(tile, radius) {
                        site.crop = Crop::Ripe;
                    }
                }
                for (index, (source, stock)) in
                    region.sources.iter().zip(&mut self.nodes).enumerate()
                {
                    if source.material != Material::Stone && source.tile.within(tile, radius) {
                        *stock = source
                            .stock
                            .min(stock.saturating_add(10 + u16::from(gifts.growth) * 8));
                        self.regrowth[index] = 0.;
                    }
                }
                Ok("Die Saat reift. Holz und Obst wachsen nach.".into())
            }
            Miracle::Mend => {
                self.restore_energy(44.);
                Ok("Das Heillicht stärkt dein Nilpferd.".into())
            }
        }
    }

    fn repair(&mut self, region: &Region) -> GameResult {
        self.bridge_ready()?;
        if self.bridge_fixed {
            return Err("Der Übergang wurde bereits gebaut.".into());
        }
        if !self.affordable(region.rebuild) {
            return Err(format!(
                "Für den Übergang fehlen Baustoffe. Benötigt werden {}.",
                stock_text(region.rebuild)
            ));
        }
        self.pay(region.rebuild);
        self.bridge_fixed = true;
        self.faith = self.faith.saturating_add(3);
        Ok(format!(
            "Der Übergang {} ist jetzt begehbar.",
            region.crossing
        ))
    }

    fn give(&mut self, region: &Region) -> GameResult {
        self.bridge_ready()?;
        if !self.bridge_fixed {
            return Err("Stelle den Übergang wieder her, damit die Gabe ankommt.".into());
        }
        if self.gift_given {
            return Err("Die Bitte wurde bereits erfüllt.".into());
        }
        let (material, amount) = region.gift;
        let mut price = [0; 3];
        price[material.index()] = amount;
        if !self.affordable(price) {
            return Err(format!(
                "Das Dorf bittet um {amount} {}.",
                region.kind.material(material)
            ));
        }
        self.pay(price);
        self.gift_given = true;
        self.faith = self.faith.saturating_add(5);
        Ok(if self.renown() < region.renown {
            format!(
                "Gabe angenommen · Ansehen {}/{}",
                self.renown(),
                region.renown
            )
        } else {
            "Die Nachbarn haben deine Gabe angenommen.".into()
        })
    }

    fn bridge_ready(&self) -> Result<(), String> {
        if self.quest != Quest::Bridge || self.count(Building::Shrine) == 0 {
            return Err("Baue den Grenzschrein am Übergang.".into());
        }
        Ok(())
    }

    fn advance_quest(&mut self, region: &Region) -> bool {
        match self.quest {
            Quest::Shelter
                if self.hand_deliveries > 0
                    && self.hippo_deliveries > 0
                    && self.count(Building::House) > 0
                    && self.count(Building::Store) > 0 =>
            {
                self.quest = Quest::Fields;
                self.faith = self.faith.saturating_add(4);
                true
            }
            Quest::Fields
                if self.quest_step(region).map_or_else(
                    || self.harvests > 0 && self.count(Building::Field) > 0,
                    |step| step.pending(self, region).is_none(),
                ) =>
            {
                self.quest = Quest::Bridge;
                self.faith = self.faith.max(region.faith);
                true
            }
            Quest::Bridge => {
                let outcome = if self.guards.iter().all(|&spirit| spirit == 0) {
                    Some(Outcome::Victory)
                } else if self.bridge_fixed && self.gift_given && self.renown() >= region.renown {
                    Some(Outcome::Alliance)
                } else {
                    None
                };
                if let Some(outcome) = outcome {
                    self.outcome = Some(outcome);
                    self.quest = Quest::Districts;
                    self.bridge_fixed = true;
                    self.guards.fill(0);
                    self.faith = self.faith.saturating_add(10);
                    true
                } else {
                    false
                }
            }
            Quest::Districts
                if self.districts.iter().all(|state| state.outcome.is_some())
                    && self
                        .quest_step(region)
                        .is_none_or(|step| step.pending(self, region).is_none()) =>
            {
                self.quest = Quest::Complete;
                self.faith = self.faith.saturating_add(12);
                true
            }
            _ => false,
        }
    }

    pub fn quest_step(&self, region: &Region) -> Option<&'static QuestStep> {
        let quests = region.quests?;
        match self.quest {
            Quest::Fields => quests.preparation.as_ref(),
            Quest::Districts => quests.finale.as_ref(),
            _ => None,
        }
    }

    pub fn quest_title(&self, region: &Region) -> &'static str {
        if let Some(step) = self.quest_step(region) {
            &step.title
        } else if let (Quest::Bridge, Some(quests)) = (self.quest, region.quests) {
            &quests.request
        } else {
            self.quest.name()
        }
    }

    pub fn objective(&self, region: &Region) -> (&'static str, String) {
        if let Some(step) = self.quest_step(region) {
            let action = step
                .pending(self, region)
                .map_or("Siedlungen vereinen", |goal| goal.prompt());
            let progress = step.progress(self, region);
            let progress = if self.quest == Quest::Districts {
                let ready = self
                    .districts
                    .iter()
                    .filter(|state| state.outcome.is_some())
                    .count();
                format!("{progress} · Nachbardörfer {ready}/2")
            } else {
                progress
            };
            return (action, format!("{}\n{progress}", step.hint));
        }
        match self.quest {
            Quest::Shelter if self.hand_deliveries == 0 && self.held.is_some() => (
                "Ladung ablegen",
                "Wähle den Sammelplatz und tippe auf Ablegen. Zum Ziel zeigt dir den Sammelplatz."
                    .into(),
            ),
            Quest::Shelter if self.hand_deliveries == 0 => (
                "Holz greifen",
                "Tippe mit der Pfote auf einen Holzstoß und wähle Greifen.".into(),
            ),
            Quest::Shelter if self.count(Building::House) == 0 => (
                "Hütte bauen",
                "Wähle unter Bauen eine Hütte und setze sie auf freien Boden am Schrein.".into(),
            ),
            Quest::Shelter if self.count(Building::Store) == 0 => (
                "Lager bauen",
                "Wähle ein Lager im Baubuch. Gläubige können dort ihre Ladung ablegen.".into(),
            ),
            Quest::Shelter => (
                "Nilpferd losschicken",
                "Wähle Nilpferd und dann Sammeln. Markiere eine Quelle und bestätige den Auftrag."
                    .into(),
            ),
            Quest::Fields if self.count(Building::Field) == 0 => (
                "Feld anlegen",
                "Wähle unter Bauen ein Feld. Darauf lässt Regen deine erste Saat wachsen.".into(),
            ),
            Quest::Fields
                if self.buildings.iter().any(|site| {
                    site.kind == Building::Field && matches!(site.crop, Crop::Ripe)
                }) =>
            {
                (
                    "Feld ernten",
                    "Tippe mit der Pfote auf das reife Feld und wähle Ernten.".into(),
                )
            }
            Quest::Fields
                if self.buildings.iter().any(|site| {
                    site.kind == Building::Field && matches!(site.crop, Crop::Growing(_))
                }) =>
            {
                (
                    "Saat wächst",
                    "Sobald das Feld reif ist, kannst du es mit der Pfote ernten.".into(),
                )
            }
            Quest::Fields => (
                "Regen wirken",
                "Wähle Wunder und dann Regenschauer. Richte den Regenkreis auf ein trockenes Feld."
                    .into(),
            ),
            Quest::Bridge if self.count(Building::Shrine) == 0 => (
                "Grenzschrein bauen",
                "Baue den Grenzschrein auf dem goldenen Platz am Übergang.".into(),
            ),
            Quest::Bridge => (
                "Übergang sichern",
                format!(
                    "Repariere den Übergang und bring {} {} bei Ansehen {}/{}. Windstöße vertreiben die Wachen.",
                    region.gift.1,
                    region.kind.material(region.gift.0),
                    self.renown(),
                    region.renown
                ),
            ),
            Quest::Districts => (
                "Siedlungen vereinen",
                format!(
                    "Führe die Nachbardörfer zusammen ({}/2). Siedlungen wählst du auf der Landkarte.",
                    self.districts
                        .iter()
                        .filter(|state| state.outcome.is_some())
                        .count()
                ),
            ),
            Quest::Complete if region.sandbox => (
                "Freies Bauen",
                "Wähle Pläne im Baubuch. Straßen und Brücken sind offen.".into(),
            ),
            Quest::Complete => (
                "Kapitel abgeschlossen",
                "Du kannst weiterbauen oder zum nächsten Kapitel reisen.".into(),
            ),
        }
    }

    pub fn side_quest(&self, region: &Region) -> SideQuest {
        if region.sandbox || self.local_finished {
            SideQuest::Complete
        } else if !self.local_accepted {
            SideQuest::Available
        } else if self.local_quest(region).2 {
            SideQuest::Ready
        } else {
            SideQuest::Active
        }
    }

    pub fn local_quest(&self, region: &Region) -> (&'static str, String, bool) {
        if let Some(quests) = region.quests {
            let step = &quests.side;
            return (
                &step.title,
                format!("{}\n{}", step.hint, step.progress(self, region)),
                step.pending(self, region).is_none(),
            );
        }
        if region.chapter() > 0 {
            let hall = Civic::regional(region.kind);
            let crafted = self.service(region, hall);
            let camps = self.camps.iter().filter(|&&active| active).count();
            return (
                if region.chapter() == 1 {
                    "Das Handwerk der Region"
                } else {
                    "Das Erbe des Landes"
                },
                format!(
                    "{} am Weg ({}/1) und Sammellager ({}/{})",
                    hall.spec().name,
                    u8::from(crafted),
                    camps,
                    region.chapter()
                ),
                crafted && camps >= region.chapter(),
            );
        }
        let service_count = |kind| {
            self.buildings
                .iter()
                .filter(|site| site.kind == kind && self.connected(region, site))
                .count()
        };
        let wells = service_count(Building::Well);
        match region.kind {
            Biome::Valley => (
                "Der Weidengarten",
                format!(
                    "Brunnen am Weg ({}/1), Ernten ({}/3)",
                    wells.min(1),
                    self.harvests.min(3)
                ),
                wells >= 1 && self.harvests >= 3,
            ),
            Biome::Fjord => {
                let granaries = service_count(Building::Granary);
                (
                    "Für den ersten Frost",
                    format!(
                        "Speicher am Weg ({}/1), Holzvorrat ({}/20)",
                        granaries.min(1),
                        self.stock[0].min(20)
                    ),
                    granaries >= 1 && self.stock[0] >= 20,
                )
            }
            Biome::Terraces => {
                let fields = self
                    .buildings
                    .iter()
                    .filter(|site| site.kind == Building::Field && site.upgraded)
                    .count();
                (
                    "Saatgut aus den Terrassen",
                    format!(
                        "Ausgebaute Felder ({}/2), Ernten ({}/4)",
                        fields.min(2),
                        self.harvests.min(4)
                    ),
                    fields >= 2 && self.harvests >= 4,
                )
            }
            Biome::Lake => (
                "Vorrat für die Trockenzeit",
                format!(
                    "Brunnen am Weg ({}/2), Nahrungsvorrat ({}/23)",
                    wells.min(2),
                    self.stock[2].min(23)
                ),
                wells >= 2 && self.stock[2] >= 23,
            ),
            Biome::Aegean => {
                let theater = self.service(region, Civic::Theater);
                let oracle = self.service(region, Civic::Oracle);
                (
                    "Fest der Winde",
                    format!(
                        "Theater am Weg ({}/1), Orakel am Weg ({}/1)",
                        u8::from(theater),
                        u8::from(oracle)
                    ),
                    theater && oracle,
                )
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Campaign {
    pub version: u32,
    pub active: usize,
    pub unlocked: usize,
    pub gifts: Gifts,
    #[serde(default)]
    pub intro_seen: bool,
    #[serde(default)]
    pub research: Research,
    pub settlements: Vec<Option<Settlement>>,
}

impl Campaign {
    fn new(regions: &[Region]) -> Self {
        let mut settlements = vec![None; regions.len()];
        settlements[0] = Some(Settlement::new(&regions[0]));
        let sandbox = regions[0].sandbox;
        let mut campaign = Self {
            version: SAVE_VERSION,
            active: 0,
            unlocked: 0,
            gifts: Gifts::default(),
            intro_seen: sandbox,
            research: Research::default(),
            settlements,
        };
        if sandbox {
            campaign.gifts.training = 5;
            campaign.gifts.points = 12;
            campaign.research = Research::all_plans();
        }
        campaign
    }

    fn world_layout(&self, chapter: usize) -> WorldLayout {
        self.settlements
            .get(chapter)
            .and_then(Option::as_ref)
            .map_or(WorldLayout::Regional, |city| city.world_layout)
    }

    pub fn restore(mut self, regions: &[Region]) -> Option<Self> {
        if (1..=3).contains(&self.version) {
            for (city, region) in self.settlements.iter_mut().zip(regions) {
                if let Some(city) = city {
                    let old_size = if self.version == 1 {
                        ORIGINAL_SOURCE_COUNT
                    } else {
                        region.original_sources
                    };
                    if city.nodes.len() != old_size || city.regrowth.len() != old_size {
                        return None;
                    }
                    city.nodes
                        .extend(region.sources[old_size..].iter().map(|source| source.stock));
                    city.regrowth.resize(region.sources.len(), 0.);
                }
            }
        }
        if matches!(self.version, 1 | 2) {
            if self.settlements.len() == 4 && regions.len() == 5 {
                self.settlements.push(None);
            }
            for city in self.settlements.iter().flatten() {
                let districts = city
                    .districts
                    .iter()
                    .filter(|state| state.outcome.is_some())
                    .count() as u16;
                self.research.earn(
                    2 * city.quest.index() as u16
                        + 2 * districts
                        + 2 * u16::from(city.local_finished),
                );
            }
            if self
                .settlements
                .get(3)
                .and_then(Option::as_ref)
                .is_some_and(|city| {
                    city.quest == Quest::Complete
                        && city.districts.iter().all(|state| state.outcome.is_some())
                })
            {
                self.unlocked = self.unlocked.max(4);
            }
        }
        if (1..=3).contains(&self.version) {
            self.version = SAVE_VERSION;
            self.intro_seen = true;
        }
        self.valid(regions).then_some(self)
    }

    fn valid(&self, regions: &[Region]) -> bool {
        self.version == SAVE_VERSION
            && self.settlements.len() == regions.len()
            && self.active <= self.unlocked
            && self.unlocked < regions.len()
            && self.settlements[self.active].is_some()
            && self
                .settlements
                .iter()
                .zip(regions)
                .all(|(settlement, region)| {
                    settlement.as_ref().is_none_or(|city| {
                        let valid_bundle =
                            |b: Bundle| b.source < region.sources.len() && b.count > 0;
                        let valid_job = |job| match job {
                            Job::Gather(source) => source < region.sources.len(),
                            Job::GatherArea(area, _) => {
                                region.areas.get(area).is_some() && city.camps[area]
                            }
                            Job::Move(tile) | Job::Watch(tile) | Job::Tend(tile) => {
                                region.bounds.contains(tile)
                            }
                        };
                        city.nodes.len() == region.sources.len()
                            && city.regrowth.len() == region.sources.len()
                            && city.guards.len() == region.guards.len()
                            && (2..=MAX_WALKERS).contains(&city.walkers.len())
                            && city.walkers[0].hippo
                            && city.last_market < 2
                            && city.energy.is_finite()
                            && city.society.morale <= 100
                            && city.held.is_none_or(valid_bundle)
                            && city.roads.iter().all(|&tile| region.bounds.contains(tile))
                            && city.buildings.iter().all(|site| {
                                site.kind
                                    .tiles(site.tile)
                                    .all(|tile| region.bounds.contains(tile))
                            })
                            && city.lesson.is_none_or(|lesson| match lesson {
                                Lesson::Delivery(source) => source < region.sources.len(),
                                Lesson::Practiced(job) => valid_job(job),
                            })
                            && city.walkers.iter().all(|walker| {
                                region.bounds.contains(walker.tile())
                                    && walker.childhood.is_finite()
                                    && walker.childhood >= 0.
                                    && walker.bundle.is_none_or(valid_bundle)
                                    && walker.route.iter().all(|&p| region.bounds.contains(p))
                                    && walker.job.is_none_or(valid_job)
                            })
                    })
                })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Profile {
    pub mode: Mode,
    pub campaign: Campaign,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Saves {
    pub current: Mode,
    pub profiles: Vec<Profile>,
}

impl Saves {
    pub fn legacy(campaign: Campaign) -> Self {
        Self {
            current: Mode::Chronicle,
            profiles: vec![Profile {
                mode: Mode::Chronicle,
                campaign,
            }],
        }
    }
}

#[derive(Resource)]
pub struct Game {
    pub mode: Mode,
    archives: Vec<Profile>,
    pub regions: Vec<Region>,
    pub campaign: Campaign,
}

impl Game {
    pub fn new(regions: Vec<Region>, saved: Option<Campaign>) -> Self {
        let campaign = saved
            .and_then(|save| save.restore(&regions))
            .unwrap_or_else(|| Campaign::new(&regions));
        Self {
            mode: Mode::Chronicle,
            archives: Vec::new(),
            regions,
            campaign,
        }
    }

    pub fn start(mode: Mode) -> Self {
        let mut game = Self::new(mode.regions(|_| WorldLayout::Regional), None);
        game.mode = mode;
        game
    }

    pub fn resume(mut saves: Saves) -> Option<Self> {
        if saves.profiles.len() > Biome::ALL.len() * 2 + 1
            || saves.profiles.iter().enumerate().any(|(index, profile)| {
                saves.profiles[..index]
                    .iter()
                    .any(|other| other.mode == profile.mode)
            })
        {
            return None;
        }
        let selected = saves
            .profiles
            .iter()
            .position(|profile| profile.mode == saves.current)?;
        let profile = saves.profiles.swap_remove(selected);
        let regions = profile
            .mode
            .regions(|chapter| profile.campaign.world_layout(chapter));
        let campaign = profile.campaign.restore(&regions)?;
        Some(Self {
            mode: profile.mode,
            archives: saves.profiles,
            regions,
            campaign,
        })
    }

    pub fn saved_games(&self) -> Saves {
        let mut profiles = self.archives.clone();
        profiles.push(Profile {
            mode: self.mode,
            campaign: self.campaign.clone(),
        });
        Saves {
            current: self.mode,
            profiles,
        }
    }

    pub fn has_save(&self, mode: Mode) -> bool {
        (mode == self.mode && self.campaign.intro_seen)
            || self.archives.iter().any(|profile| profile.mode == mode)
    }

    pub fn switch_mode(&mut self, mode: Mode) -> GameResult {
        if self.mode == mode {
            return Ok(mode.name().into());
        }
        let entry = self
            .archives
            .iter()
            .position(|profile| profile.mode == mode);
        let archived = entry.map(|index| &self.archives[index].campaign);
        let regions = mode.regions(|chapter| {
            archived.map_or(WorldLayout::Regional, |save| save.world_layout(chapter))
        });
        let campaign = if let Some(saved) = archived {
            saved
                .clone()
                .restore(&regions)
                .ok_or("Der Spielstand dieses Spielmodus konnte nicht gelesen werden.")?
        } else {
            Campaign::new(&regions)
        };
        if let Some(index) = entry {
            self.archives.swap_remove(index);
        }
        self.archives.push(Profile {
            mode: self.mode,
            campaign: std::mem::replace(&mut self.campaign, campaign),
        });
        self.mode = mode;
        self.regions = regions;
        Ok(mode.name().into())
    }

    pub fn region(&self) -> &Region {
        &self.regions[self.campaign.active]
    }

    pub fn city(&self) -> &Settlement {
        self.campaign.settlements[self.campaign.active]
            .as_ref()
            .unwrap()
    }

    pub fn building_plan(&self, kind: Building, tile: Tile) -> Result<(), String> {
        if let Some(kind) = kind.civic() {
            self.campaign.research.permits(kind)?;
        }
        self.city().placement(self.region(), kind, tile)
    }

    pub fn act(&mut self, action: Action) -> GameResult {
        let index = self.campaign.active;
        let settlement = self.campaign.settlements[index].as_mut().unwrap();
        let region = &self.regions[index];
        let gifts = &mut self.campaign.gifts;
        let research = &mut self.campaign.research;
        let mut text = match action {
            Action::Hand(target) => settlement.hand(region, target),
            Action::Build(kind, tile, rotation) => {
                if let Some(kind) = kind.civic() {
                    research.permits(kind)?;
                }
                settlement.build(region, kind, tile, rotation)
            }
            Action::Road(start, end) => settlement.build_road(region, start, end),
            Action::Upgrade(tile) => settlement.upgrade(region, tile),
            Action::Research(tech) => {
                research.learn(tech, settlement.city_rank())?;
                Ok(format!("{} erforscht · Neue Pläne im Baubuch", tech.name()))
            }
            Action::Muster(tile) => settlement.muster(region, tile),
            Action::Patrol(tile) => settlement.patrol(region, tile),
            Action::Settle(index) => settlement.claim_district(region, index).inspect(|_| {
                gifts.earn();
                research.earn(2);
            }),
            Action::Camp(index) => settlement
                .open_camp(region, index)
                .inspect(|_| research.earn(1)),
            Action::Assign(tile, hippo) => settlement.assign(region, tile, hippo),
            Action::AssignArea(area, material, hippo) => {
                settlement.assign_area(region, area, material, hippo)
            }
            Action::CancelArea(area, material) => settlement.cancel_area(area, material),
            Action::AcceptLocal => {
                if settlement.side_quest(region) != SideQuest::Available {
                    return Err("Dieser Auftrag wurde bereits angenommen.".into());
                }
                settlement.local_accepted = true;
                Ok("Der Nebenauftrag steht jetzt unter Aufträge.".into())
            }
            Action::FinishLocal => {
                if settlement.side_quest(region) != SideQuest::Ready {
                    return Err("Erfülle zuerst die Ziele des Nebenauftrags.".into());
                }
                settlement.local_finished = true;
                settlement.faith = settlement.faith.saturating_add(6);
                gifts.earn();
                research.earn(2);
                Ok("Nebenauftrag erfüllt · 6 Glauben · 2 Erkenntnis".into())
            }
            Action::RecallHelpers => settlement.recall_helpers(),
            Action::MoveHippo(tile) => settlement.move_hippo(region, tile),
            Action::Watch(tile) => settlement.watch(region, tile, *gifts),
            Action::Tend(tile) => settlement.tend(region, tile, *gifts),
            Action::Feed => settlement.feed(),
            Action::Rest => {
                let hippo = &mut settlement.walkers[0];
                hippo.job = None;
                hippo.route.clear();
                hippo.pause = 0.;
                Ok("Das Nilpferd legt eine Pause ein.".into())
            }
            Action::Cast(spell, tile) => settlement.cast(region, spell, tile, *gifts),
            Action::Repair => settlement.repair(region),
            Action::Give => settlement.give(region),
            Action::Praise => {
                let lesson = settlement
                    .lesson
                    .take()
                    .ok_or("Lobe die Kreatur nach einer Lieferung, Wache oder Ernte.")?;
                let hippo = &mut settlement.walkers[0];
                if gifts.training < 5 {
                    gifts.coaching += 1;
                    if gifts.coaching >= gifts.training.saturating_sub(1).max(1) {
                        gifts.training += 1;
                        gifts.coaching = 0;
                    }
                }
                if hippo.job.is_none() {
                    hippo.job = Some(match lesson {
                        Lesson::Delivery(source) => Job::Gather(source),
                        Lesson::Practiced(job) => job,
                    });
                }
                Ok(format!("Lob angenommen · Nilpferd-Rang {}", gifts.training))
            }
            Action::Learn(talent) => {
                if gifts.points == 0 {
                    return Err("Erfülle Aufträge für Talentpunkte.".into());
                }
                let rank = match talent {
                    Talent::Rain => &mut gifts.rain,
                    Talent::Spring => &mut gifts.spring,
                    Talent::Growth => &mut gifts.growth,
                    Talent::Storm => &mut gifts.storm,
                };
                if *rank >= 2 {
                    return Err("Dieses Talent hat schon seine höchste Stufe.".into());
                }
                gifts.points -= 1;
                *rank += 1;
                Ok(format!("{} · Rang {}", talent.name(), rank))
            }
            Action::Travel(next) => {
                if next > self.campaign.unlocked {
                    return Err("Schließe zuerst das vorherige Kapitel ab.".into());
                }
                self.campaign.active = next;
                self.campaign.settlements[next]
                    .get_or_insert_with(|| Settlement::new(&self.regions[next]));
                return Ok(self.regions[next].subtitle.clone());
            }
        }?;
        if self.advance_quests() {
            text.push_str(" · Auftrag erfüllt");
        }
        Ok(text)
    }

    pub fn step(&mut self, dt: f32) {
        let index = self.campaign.active;
        let city = self.campaign.settlements[index].as_mut().unwrap();
        let region = &self.regions[index];
        let insight = city.tick(region, self.campaign.gifts, dt);
        self.campaign.research.earn(insight);
        self.advance_quests();
    }

    fn advance_quests(&mut self) -> bool {
        let index = self.campaign.active;
        let city = self.campaign.settlements[index].as_mut().unwrap();
        let region = &self.regions[index];
        let main = city.advance_quest(region);
        if main {
            self.campaign.gifts.earn();
            self.campaign.research.earn(2);
        }
        if city.quest == Quest::Complete
            && city.districts.iter().all(|state| state.outcome.is_some())
        {
            self.campaign.unlocked = self
                .campaign
                .unlocked
                .max((index + 1).min(self.regions.len() - 1));
        }
        main
    }

    pub fn campaign_complete(&self) -> bool {
        self.campaign.settlements.iter().all(|city| {
            city.as_ref().is_some_and(|city| {
                city.quest == Quest::Complete
                    && city.districts.iter().all(|state| state.outcome.is_some())
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::{CameraRig, Panel, Play, Tool, UiAction};

    fn fresh() -> Game {
        Game::new(Region::campaign(), None)
    }

    #[test]
    fn a_saved_chapter_keeps_its_map_and_an_unvisited_chapter_gets_the_regional_layout() {
        let mode = Mode::Campaign(Biome::Lake);
        let mut previous = Game::new(mode.regions(|_| WorldLayout::Classic), None);
        previous.mode = mode;
        previous.campaign.unlocked = 1;
        let pickup = previous.region().sources[0].tile;
        previous.act(Action::Hand(pickup)).unwrap();
        let mut data = serde_json::to_value(previous.saved_games()).unwrap();
        data["profiles"][0]["campaign"]["settlements"][0]
            .as_object_mut()
            .unwrap()
            .remove("world_layout");
        let mut resumed = Game::resume(serde_json::from_value(data).unwrap()).unwrap();

        assert_eq!(resumed.region().world_layout(), WorldLayout::Classic);
        assert!(resumed.region().quests.is_none());
        assert_eq!(resumed.city().held.unwrap().source, 0);
        assert_eq!(resumed.region().sources[0].tile, pickup);
        for tile in previous.region().bounds.tiles() {
            assert_eq!(previous.region().water(tile), resumed.region().water(tile));
        }

        resumed.act(Action::Travel(1)).unwrap();
        assert_eq!(resumed.region().world_layout(), WorldLayout::Regional);
        assert_eq!(
            resumed.city().quest_title(resumed.region()),
            "Der Pfad zwischen den Brunnen"
        );
        let second = resumed.region().clone();
        let data = serde_json::to_vec(&resumed.saved_games()).unwrap();
        let resumed = Game::resume(serde_json::from_slice(&data).unwrap()).unwrap();
        assert_eq!(resumed.region().world_layout(), WorldLayout::Regional);
        for tile in second.bounds.tiles() {
            assert_eq!(second.water(tile), resumed.region().water(tile));
        }
    }

    #[test]
    fn the_fjord_guide_assigns_the_hippo_and_both_winter_goals_survive_a_reload() {
        let mut game = Game::start(Mode::Campaign(Biome::Fjord));
        game.campaign.intro_seen = true;
        let mut play = Play::new(&game);
        let mut view = CameraRig::new(&game);
        play.activate(UiAction::Panel(Panel::Quests), &mut game, &mut view, 0.);
        play.activate(UiAction::Guide, &mut game, &mut view, 0.);
        assert!(matches!(play.target(&game), Action::Assign(_, true)));
        assert!(play.activate(UiAction::Confirm, &mut game, &mut view, 0.));

        stock_up(&mut game, Material::Wood, 20);
        assert_eq!(game.city().quest, Quest::Fields);
        assert!(
            game.city()
                .objective(game.region())
                .1
                .contains("Kiefernholz 20/20")
        );
        let data = serde_json::to_vec(&game.saved_games()).unwrap();
        let mut resumed = Game::resume(serde_json::from_slice(&data).unwrap()).unwrap();
        fulfill_goal(
            &mut resumed,
            crate::quests::Goal::HippoDeliveries { count: 1 },
        );
        assert_eq!(resumed.city().quest, Quest::Bridge);
        assert_eq!(
            resumed.city().quest_title(resumed.region()),
            "Brennholz für den Frost"
        );
    }

    #[test]
    fn tutorial_advances_after_grabbing_and_delivering_wood() {
        let mut game = Game::start(Mode::Campaign(Biome::Valley));
        let wood = game.region().sources[0].tile;
        let depot = game.region().depot;
        let goal = |game: &Game| game.city().objective(game.region()).0;

        assert_eq!(goal(&game), "Holz greifen");
        game.act(Action::Hand(wood)).unwrap();
        assert_eq!(goal(&game), "Ladung ablegen");
        game.act(Action::Hand(depot)).unwrap();
        assert_eq!(goal(&game), "Hütte bauen");
    }

    #[test]
    fn a_side_quest_can_only_be_rewarded_once_even_after_loading() {
        let mut game = Game::start(Mode::Campaign(Biome::Valley));
        let region = game.region().clone();
        assert_eq!(game.city().side_quest(&region), SideQuest::Available);
        assert!(game.act(Action::FinishLocal).is_err());
        game.act(Action::AcceptLocal).unwrap();
        assert_eq!(game.city().side_quest(&region), SideQuest::Active);

        let city = game.campaign.settlements[0].as_mut().unwrap();
        let well = region
            .bounds
            .tiles()
            .filter(|tile| tile.within(region.start, 5))
            .map(|tile| BuildingSite {
                kind: Building::Well,
                tile,
                rotation: 0,
                crop: Crop::Dry,
                upgraded: false,
            })
            .find(|site| {
                city.connected(&region, site)
                    && site
                        .kind
                        .tiles(site.tile)
                        .all(|tile| city.walkable(&region, tile) && !city.road_at(&region, tile))
            })
            .unwrap();
        city.buildings.push(well);
        city.harvests = 3;
        let reload = |game: &Game| {
            let data = serde_json::to_vec(&game.saved_games()).unwrap();
            Game::resume(serde_json::from_slice(&data).unwrap()).unwrap()
        };
        let mut game = reload(&game);
        assert_eq!(game.city().side_quest(&region), SideQuest::Ready);

        let faith = game.city().faith;
        let insight = game.campaign.research.insight;
        let points = game.campaign.gifts.points;
        game.act(Action::FinishLocal).unwrap();
        let mut game = reload(&game);
        assert_eq!(game.city().side_quest(&region), SideQuest::Complete);
        assert!(game.act(Action::FinishLocal).is_err());
        assert_eq!(game.city().faith, faith + 6);
        assert_eq!(game.campaign.research.insight, insight + 2);
        assert_eq!(game.campaign.gifts.points, points + 1);
    }

    #[test]
    fn spellbook_keeps_locked_spells_open_and_guides_rain_to_dry_fields() {
        let mut game = Game::start(Mode::Sandbox(Biome::Aegean));
        let field = game
            .city()
            .buildings
            .iter()
            .find(|site| site.kind == Building::Field)
            .unwrap()
            .tile;
        let mut play = Play::new(&game);
        let mut view = CameraRig::new(&game);

        play.activate(UiAction::Tool(Tool::Miracles), &mut game, &mut view, 0.);
        play.activate(
            UiAction::Order(Tool::Miracles, Miracle::Thunder as usize),
            &mut game,
            &mut view,
            0.,
        );
        assert_eq!(play.panel, Panel::Spells);
        assert_eq!(play.notice, "Nur in Kampagnen");

        play.activate(
            UiAction::Order(Tool::Miracles, Miracle::Rain as usize),
            &mut game,
            &mut view,
            0.,
        );
        play.aim = game.region().depot;
        play.activate(UiAction::Guide, &mut game, &mut view, 0.);
        assert_eq!(play.aim, field);
        assert!(play.activate(UiAction::Confirm, &mut game, &mut view, 0.));
        assert!(matches!(
            game.city().building_at(field).unwrap().crop,
            Crop::Growing(_)
        ));
        assert!(
            game.city()
                .ritual_status(game.region(), Miracle::Rain, game.campaign.gifts)
                .is_some()
        );
    }

    fn stock_up(game: &mut Game, material: Material, needed: u16) {
        let depot = game.region().depot;
        while game.city().stock[material.index()] < needed {
            let source = game
                .region()
                .sources
                .iter()
                .zip(&game.city().nodes)
                .find(|(source, remaining)| {
                    source.material == material
                        && **remaining > 0
                        && game.city().influence(game.region(), source.tile)
                })
                .map(|(source, _)| source.tile)
                .expect("an accessible source with enough stock");
            game.act(Action::Hand(source)).unwrap();
            game.act(Action::Hand(depot)).unwrap();
        }
    }

    fn build(game: &mut Game, kind: Building) -> Tile {
        let price = kind.price();
        stock_up(game, Material::Wood, price[0]);
        stock_up(game, Material::Stone, price[1]);
        let region = game.region();
        let tile = if kind == Building::Shrine {
            region.frontier
        } else {
            region
                .bounds
                .tiles()
                .filter(|&tile| tile.within(region.start, 5))
                .find(|&tile| game.city().placement(region, kind, tile).is_ok())
                .expect("a buildable plot near the starting shrine")
        };
        game.act(Action::Build(kind, tile, 0)).unwrap();
        tile
    }

    fn learn_quest_plan(game: &mut Game, tech: Tech) {
        if !game.campaign.research.knows(tech) {
            for &parent in tech.parents() {
                learn_quest_plan(game, parent);
            }
            game.act(Action::Research(tech)).unwrap_or_else(|error| {
                panic!("{} · {}: {error}", game.region().name, tech.name())
            });
        }
    }

    fn place_quest_building(game: &mut Game, kind: Building, roadside: bool) -> Tile {
        let needs_workshop = matches!(kind, Building::Granary | Building::Market | Building::Tower);
        if (needs_workshop || matches!(kind, Building::Well | Building::Workshop))
            && game.city().city_rank() < 2
        {
            for building in [Building::Store, Building::Field] {
                if game.city().count(building) == 0 {
                    build(game, building);
                }
            }
        }
        if needs_workshop && game.city().city_rank() < 3 {
            build(game, Building::Workshop);
        }
        if let Some(civic) = kind.civic() {
            learn_quest_plan(game, civic.spec().unlock);
        }
        for (index, producer) in [Civic::Sawmill, Civic::Masonry].into_iter().enumerate() {
            let required = kind.goods_price()[index];
            if game.city().society.goods[index] < required {
                let producer = Building::Civic(producer);
                if !game
                    .city()
                    .buildings
                    .iter()
                    .any(|site| site.kind == producer && game.city().connected(game.region(), site))
                {
                    place_quest_building(game, producer, true);
                }
                for _ in 0..required {
                    stock_up(game, Material::ALL[index], 8);
                    game.step(25.);
                    if game.city().society.goods[index] >= required {
                        break;
                    }
                }
                assert!(game.city().society.goods[index] >= required);
            }
        }
        stock_for(game, kind.price());
        let region = game.region();
        let city = game.city();
        let tile = region
            .bounds
            .tiles()
            .filter(|tile| tile.within(region.start, 9))
            .filter(|&tile| game.building_plan(kind, tile).is_ok())
            .min_by_key(|&tile| {
                let near = kind
                    .tiles(tile)
                    .flat_map(Tile::neighbors)
                    .any(|cell| city.road_at(region, cell) && city.walkable(region, cell));
                (roadside && !near, tile.distance(region.start))
            })
            .unwrap_or_else(|| panic!("{} has no site for {}", region.name, kind.name()));
        game.act(Action::Build(kind, tile, 0)).unwrap();
        if roadside {
            connect_quest_site(game, tile);
        }
        tile
    }

    fn connect_quest_site(game: &mut Game, tile: Tile) {
        let site = game.city().building_at(tile).unwrap().clone();
        if game.city().connected(game.region(), &site) {
            return;
        }
        stock_up(game, Material::Stone, 5);
        let region = game.region();
        let city = game.city();
        let route = site
            .kind
            .tiles(tile)
            .flat_map(Tile::neighbors)
            .filter_map(|end| {
                region
                    .bounds
                    .tiles()
                    .filter(|&from| from.distance(end) <= 12 && city.road_at(region, from))
                    .filter(|&from| city.road_plan(region, from, end).is_ok())
                    .min_by_key(|&from| from.distance(end))
                    .map(|from| (from, end))
            })
            .min_by_key(|&(from, end)| from.distance(end));
        let (from, end) = route.unwrap_or_else(|| {
            panic!(
                "{} has no road to {} at {tile:?}",
                region.name,
                site.kind.name()
            )
        });
        game.act(Action::Road(from, end)).unwrap();
    }

    fn fulfill_goal(game: &mut Game, goal: crate::quests::Goal) {
        use crate::quests::Goal;

        let met = |game: &Game| goal.met(game.city(), game.region());
        if met(game) {
            return;
        }
        match goal {
            Goal::Building {
                kind,
                roadside,
                upgraded,
                ..
            } => {
                for _ in 0..3 {
                    if upgraded && game.city().city_rank() < 3 {
                        if game.city().count(Building::Field) == 0 {
                            build(game, Building::Field);
                        }
                        build(game, Building::Workshop);
                    }
                    let site = if upgraded {
                        game.city()
                            .buildings
                            .iter()
                            .find(|site| site.kind == kind && !site.upgraded)
                            .map(|site| site.tile)
                    } else {
                        None
                    };
                    let tile = site.unwrap_or_else(|| place_quest_building(game, kind, roadside));
                    if upgraded {
                        connect_quest_site(game, tile);
                        stock_for(game, kind.upgrade_price().unwrap());
                        game.act(Action::Upgrade(tile)).unwrap();
                    }
                    if met(game) {
                        break;
                    }
                }
            }
            Goal::Harvests { count } => {
                let field = game
                    .city()
                    .buildings
                    .iter()
                    .find(|site| site.kind == Building::Field)
                    .map(|site| site.tile)
                    .unwrap_or_else(|| build(game, Building::Field));
                for _ in game.city().harvests..count {
                    for _ in 0..45 {
                        match game.city().building_at(field).unwrap().crop {
                            Crop::Ripe => break,
                            Crop::Dry
                                if game
                                    .city()
                                    .ritual_error(
                                        game.region(),
                                        Miracle::Rain,
                                        field,
                                        game.campaign.gifts,
                                    )
                                    .is_none() =>
                            {
                                game.act(Action::Cast(Miracle::Rain, field)).unwrap();
                            }
                            _ => {}
                        }
                        game.step(1.);
                    }
                    game.act(Action::Hand(field)).unwrap();
                }
            }
            Goal::Reserve { material, count } => stock_up(game, material, count),
            Goal::HippoDeliveries { .. } => {
                let tile = game.region().sources[0].tile;
                game.act(Action::Assign(tile, true)).unwrap();
                for _ in 0..3_600 {
                    game.step(1. / 24.);
                    if met(game) {
                        break;
                    }
                }
            }
            Goal::Roads { count } => {
                for _ in game.city().roads.len()..usize::from(count) {
                    stock_up(game, Material::Stone, 1);
                    let region = game.region();
                    let tile = region
                        .bounds
                        .tiles()
                        .filter(|cell| cell.within(region.start, 7))
                        .filter(|&cell| game.city().road_plan(region, cell, cell).is_ok())
                        .min_by_key(|&cell| cell.distance(region.start))
                        .unwrap();
                    game.act(Action::Road(tile, tile)).unwrap();
                }
            }
            Goal::Camps { count } => {
                for _ in game.city().camps.iter().filter(|&&open| open).count()..usize::from(count)
                {
                    let index = game.city().camps.iter().position(|&open| !open).unwrap();
                    stock_up(game, Material::Wood, CAMP_WOOD);
                    game.act(Action::Camp(index)).unwrap();
                }
            }
            Goal::OlivePressings { count } => {
                let kind = Building::Civic(Civic::OlivePress);
                if !game.city().buildings.iter().any(|site| site.kind == kind) {
                    place_quest_building(game, kind, true);
                }
                for _ in game.city().olive_pressings..count {
                    stock_up(game, Material::Food, 16);
                    game.step(25.);
                }
            }
        }
        assert!(met(game), "{} · {goal:?}", game.region().name);
    }

    fn prepare_bridge(game: &mut Game) {
        if game.city().quest == Quest::Shelter {
            let first = game.region().sources[0].tile;
            let stone = game.region().sources[2].tile;
            let depot = game.region().depot;
            game.act(Action::Hand(first)).unwrap();
            game.act(Action::Hand(depot)).unwrap();
            game.act(Action::Assign(first, true)).unwrap();
            game.act(Action::Assign(stone, false)).unwrap();
            for _ in 0..480 {
                game.step(1. / 24.);
            }
            assert!(game.city().hippo_deliveries > 0);
            assert!(game.city().helper_deliveries > 0);
            game.act(Action::Praise).unwrap();
            build(game, Building::House);
            build(game, Building::Store);
        }
        assert_eq!(game.city().quest, Quest::Fields);
        if let Some(step) = game.city().quest_step(game.region()) {
            for &goal in &step.goals {
                fulfill_goal(game, goal);
            }
        } else {
            let starter_field = game
                .city()
                .buildings
                .iter()
                .find(|site| site.kind == Building::Field)
                .map(|site| site.tile);
            let field = starter_field.unwrap_or_else(|| build(game, Building::Field));
            game.act(Action::Cast(Miracle::Rain, field)).unwrap();
            assert!(game.act(Action::Hand(field)).is_err());
            for _ in 0..480 {
                game.step(1. / 24.);
            }
            game.act(Action::Hand(field)).unwrap();
        }
        assert_eq!(game.city().quest, Quest::Bridge, "{}", game.region().name);
        let guard = game.region().guards[0];
        assert!(game.act(Action::Cast(Miracle::Wind, guard)).is_err());
        build(game, Building::Shrine);
        assert!(game.city().influence(game.region(), guard));
    }

    fn stock_for(game: &mut Game, price: Stock) {
        for (material, quantity) in Material::ALL.into_iter().zip(price) {
            stock_up(game, material, quantity);
        }
    }

    fn serve_district(game: &mut Game, index: usize) {
        let region = game.region().clone();
        let district = &region.districts[index];
        let kind = match district.need {
            DistrictNeed::Water => Building::Well,
            DistrictNeed::Trade => Building::Market,
            DistrictNeed::Shelter => Building::House,
            DistrictNeed::Patrol => Building::Tower,
        };
        assert!(game.act(Action::Settle(index)).is_err());
        stock_for(game, kind.price());
        stock_for(game, [0, kind.price()[1] + 3, 0]);
        let (from, to, price, site) = [-2, -1, -3, 0, 1, 2, 3, 4, -4]
            .into_iter()
            .flat_map(|x| [2, -2].into_iter().map(move |z| (x, z)))
            .find_map(|(x, z)| {
                let from = district.center.offset(x, 2);
                let to = from.offset(0, z);
                let (cells, price) = game.city().road_plan(&region, from, to).ok()?;
                let site = region
                    .bounds
                    .tiles()
                    .filter(|tile| tile.within(district.center, 5))
                    .filter(|&tile| kind.tiles(tile).all(|foot| !cells.contains(&foot)))
                    .filter(|&tile| {
                        kind.tiles(tile)
                            .flat_map(Tile::neighbors)
                            .any(|neighbor| cells.contains(&neighbor))
                    })
                    .find(|&tile| game.city().placement(&region, kind, tile).is_ok())?;
                Some((from, to, price, site))
            })
            .unwrap_or_else(|| {
                panic!(
                    "kein Anschluss für {} in {} · {}",
                    kind.name(),
                    region.name,
                    district.name
                )
            });
        let stock = game.city().stock;
        game.act(Action::Road(from, to)).unwrap();
        assert_eq!(game.city().stock[1], stock[1] - price[1]);
        assert!(game.act(Action::Road(from, to)).is_err());
        game.act(Action::Build(kind, site, 0)).unwrap();
        assert!(
            game.city()
                .connected(&region, game.city().building_at(site).unwrap())
        );
        if kind == Building::House {
            stock_for(game, kind.upgrade_price().unwrap());
            game.act(Action::Upgrade(site)).unwrap();
            assert!(game.act(Action::Upgrade(site)).is_err());
        }
        stock_for(game, district.gift);
        let stock = game.city().stock;
        game.act(Action::Settle(index)).unwrap();
        for (owned, (before, gift)) in game
            .city()
            .stock
            .into_iter()
            .zip(stock.into_iter().zip(district.gift))
        {
            assert_eq!(owned, before - gift);
        }
        assert_eq!(
            game.city().districts[index].outcome,
            Some(Outcome::Alliance)
        );
        assert!(game.city().can_store_at(&region, district.center));
        assert!(game.city().influence(&region, district.center.offset(9, 0)));
        assert!(game.act(Action::Settle(index)).is_err());
    }

    #[test]
    fn a_carried_bundle_survives_reload_and_deposits_exactly_once() {
        let mut game = fresh();
        let source = game.region().sources[0].tile;
        let depot = game.region().depot;
        let wood = game.city().stock[0];
        let stored_at_source = game.city().nodes[0];
        game.act(Action::Hand(source)).unwrap();
        assert_eq!(game.city().nodes[0], stored_at_source - 6);
        assert_eq!(game.city().stock[0], wood);
        let saved: Campaign =
            serde_json::from_slice(&serde_json::to_vec(&game.campaign).unwrap()).unwrap();
        let mut resumed = Game::new(Region::campaign(), Some(saved));
        assert_eq!(resumed.city().held.unwrap().count, 6);
        resumed.act(Action::Hand(depot)).unwrap();
        assert_eq!(resumed.city().stock[0], wood + 6);
        assert!(resumed.act(Action::Hand(depot)).is_err());
        assert_eq!(resumed.city().stock[0], wood + 6);
    }

    #[test]
    fn campaigns_follow_their_theme_and_keep_independent_saves() {
        for biome in Biome::ALL {
            let mut game = Game::start(Mode::Campaign(biome));
            assert_eq!(game.regions.len(), 3);
            for chapter in 0..3 {
                game.act(Action::Travel(chapter)).unwrap();
                let region = game.region().clone();
                assert_eq!(region.kind, biome);
                assert_eq!(region.chapter(), chapter);
                assert!(
                    game.city()
                        .buildings
                        .iter()
                        .all(|site| site.kind.tiles(site.tile).all(|tile| {
                            !region.water(tile)
                                && !region.rocky(tile)
                                && !region.fixed_structure(tile)
                                && !region.road(tile)
                                && region.source_at(tile).is_none()
                        })),
                    "{}",
                    region.name
                );
                prepare_bridge(&mut game);
                if game.city().count(Building::Workshop) == 0 {
                    place_quest_building(&mut game, Building::Workshop, false);
                }
                stock_for(&mut game, region.rebuild);
                game.act(Action::Repair)
                    .unwrap_or_else(|reason| panic!("{}: {reason}", region.name));
                stock_up(&mut game, region.gift.0, region.gift.1);
                game.act(Action::Give)
                    .unwrap_or_else(|reason| panic!("{}: {reason}", region.name));
                for kind in [Building::Well, Building::Market, Building::Granary] {
                    if game.city().renown() >= region.renown {
                        break;
                    }
                    build(&mut game, kind);
                }
                assert_eq!(
                    game.city().outcome,
                    Some(Outcome::Alliance),
                    "{}",
                    region.name
                );
                for i in 0..region.districts.len() {
                    serve_district(&mut game, i);
                }
                if let Some(step) = game.city().quest_step(game.region()) {
                    assert!(game.act(Action::Travel(chapter + 1)).is_err());
                    for &goal in &step.goals {
                        fulfill_goal(&mut game, goal);
                    }
                }
                assert_eq!(game.city().quest, Quest::Complete, "{}", region.name);
            }
            assert!(game.campaign_complete());
            let learned = game.campaign.research.count();
            game.switch_mode(Mode::Sandbox(biome)).unwrap();
            assert_eq!(game.campaign.active, 0);
            assert_eq!(game.campaign.research.count(), 12);
            let archive = serde_json::to_vec(&game.saved_games()).unwrap();
            let mut resumed = Game::resume(serde_json::from_slice(&archive).unwrap()).unwrap();
            assert_eq!(resumed.mode, Mode::Sandbox(biome));
            resumed.switch_mode(Mode::Campaign(biome)).unwrap();
            assert_eq!(resumed.campaign.active, 2);
            assert_eq!(resumed.campaign.research.count(), learned);
            assert!(resumed.campaign_complete());
        }
    }

    #[test]
    fn sandbox_builds_with_all_plans_and_without_threats_or_resource_costs() {
        let mut game = Game::start(Mode::Sandbox(Biome::Aegean));
        let region = game.region().clone();
        let press = Building::Civic(Civic::OlivePress);
        game.campaign.settlements[0].as_mut().unwrap().stock = [0; 3];
        game.campaign.settlements[0].as_mut().unwrap().society.goods = [0; 3];
        assert!(game.city().guards.is_empty());
        assert_eq!(game.city().quest, Quest::Complete);
        assert!(game.city().bridge_fixed);
        assert!(
            game.act(Action::Cast(Miracle::Wind, region.rival_shrine))
                .is_err()
        );
        let remote = region.areas[2].center;
        let tile = region
            .bounds
            .tiles()
            .find(|&tile| tile.within(remote, 6) && game.building_plan(press, tile).is_ok())
            .unwrap();
        game.act(Action::Build(press, tile, 0)).unwrap();
        assert_eq!(game.city().stock, [0; 3]);
        assert!(
            game.building_plan(Building::Civic(Civic::WinterHall), tile.offset(4, 0))
                .is_err()
        );
    }

    #[test]
    fn expanded_areas_have_routes_and_a_camp_accepts_new_materials() {
        let mut game = fresh();
        let themed = Biome::ALL
            .into_iter()
            .flat_map(|biome| Mode::Campaign(biome).regions(|_| WorldLayout::Regional));
        for region in game.regions.iter().cloned().chain(themed) {
            let region = &region;
            let mut city = Settlement::new(region);
            city.bridge_fixed = true;
            let walk = city.walk_grid(region);
            let start = city.walkers[0].tile();
            let reachable = Settlement::reachable(region, &walk, start, |_| false);
            assert!(region.bounds.len() > 2 * region.original_bounds().len());
            let extra = &region.sources[region.original_sources..];
            assert!(extra.len() >= 68, "{}: {}", region.name, extra.len());
            for area in &region.areas {
                assert!(
                    area.center
                        .neighbors()
                        .into_iter()
                        .any(|tile| { region.bounds.index(tile).is_some_and(|i| reachable[i]) }),
                    "{} · {:?} · {} · {}",
                    region.name,
                    region.world_layout(),
                    region.chapter(),
                    area.name
                );
                for material in Material::ALL {
                    let count = extra
                        .iter()
                        .filter(|source| {
                            source.material == material
                                && source.tile.within(area.center, AREA_RADIUS)
                        })
                        .count();
                    let minimum = if material == area.material { 5 } else { 2 };
                    assert!(
                        count >= minimum,
                        "{} · {} · {}: {count}",
                        region.name,
                        area.name,
                        material.name()
                    );
                }
            }
            for source in extra {
                assert!(
                    source
                        .tile
                        .neighbors()
                        .into_iter()
                        .any(|tile| { region.bounds.index(tile).is_some_and(|i| reachable[i]) }),
                    "{} · {:?}",
                    region.name,
                    source.tile
                );
            }
        }
        assert!(game.act(Action::Camp(0)).is_err());
        assert!(
            game.act(Action::AssignArea(0, Material::Wood, false))
                .is_err()
        );
        let region = game.region().clone();
        let center = region.areas[0].center;
        let (index, source) = region
            .sources
            .iter()
            .enumerate()
            .skip(region.original_sources)
            .find(|(_, source)| source.tile.within(center, AREA_RADIUS))
            .unwrap();
        let city = game.campaign.settlements[0].as_mut().unwrap();
        city.bridge_fixed = true;
        city.faith = 10;
        let stock = city.stock[0];
        assert!(game.act(Action::Hand(source.tile)).is_err());
        game.act(Action::Camp(0)).unwrap();
        assert!(game.act(Action::Camp(0)).is_err());
        assert_eq!(game.city().stock[0], stock - CAMP_WOOD);
        game.act(Action::Hand(source.tile)).unwrap();
        assert_eq!(game.city().nodes[index], source.stock - 6);
        game.act(Action::Hand(center)).unwrap();
        game.act(Action::Assign(source.tile, false)).unwrap();
        game.act(Action::Assign(source.tile, true)).unwrap();
    }

    #[test]
    fn area_orders_switch_sources_and_deliver_the_last_bundle_after_cancellation() {
        let mut game = Game::start(Mode::Sandbox(Biome::Aegean));
        let region = game.region().clone();
        let area = 1;
        let material = Material::Stone;
        let (first, primary) = region.area_sources(area, material).next().unwrap();
        let (second, _) = region
            .area_sources(area, material)
            .find(|(_, source)| source.tile.distance(primary.tile) > 2)
            .unwrap();
        let city = game.campaign.settlements[0].as_mut().unwrap();
        let start = primary
            .tile
            .neighbors()
            .into_iter()
            .find(|&tile| city.walkable(&region, tile))
            .unwrap();
        city.walkers[1].position = start.local().to_array();
        for (index, _) in region.area_sources(area, material) {
            city.nodes[index] = 0;
            city.regrowth[index] = 0.;
        }
        city.nodes[first] = 3;
        city.nodes[second] = 9;
        city.stock[material.index()] = 0;

        game.act(Action::AssignArea(area, material, false)).unwrap();
        game.step(0.2);
        let first_bundle = game.city().walkers[1].bundle.unwrap();
        assert_eq!((first_bundle.source, first_bundle.count), (first, 3));
        let supply = game.city().area_supply(&region, area, material);
        assert_eq!((supply.units, supply.stocked), (9, 1));
        assert_eq!(game.city().area_crew(area, material), (1, false));
        assert_eq!(
            game.city().nearest_store(&region, primary.tile),
            region.areas[area].center
        );

        let saved = serde_json::to_vec(&game.saved_games()).unwrap();
        let mut game = Game::resume(serde_json::from_slice(&saved).unwrap()).unwrap();
        for _ in 0..180 {
            game.step(0.2);
            if game.city().walkers[1]
                .bundle
                .is_some_and(|bundle| bundle.source == second)
            {
                break;
            }
        }
        let second_bundle = game.city().walkers[1].bundle.unwrap();
        assert_eq!((second_bundle.source, second_bundle.count), (second, 3));
        assert_eq!(game.city().stock[material.index()], 3);
        game.act(Action::CancelArea(area, material)).unwrap();
        assert_eq!(game.city().area_crew(area, material), (0, false));
        assert!(game.act(Action::CancelArea(area, material)).is_err());
        for _ in 0..180 {
            game.step(0.2);
        }
        assert_eq!(game.city().stock[material.index()], 6);
        assert_eq!(game.city().nodes[second], 6);
        assert!(game.city().walkers[1].bundle.is_none());
    }

    #[test]
    fn the_gate_aligns_to_the_road_and_walkers_carry_motion_through_it() {
        let mut game = fresh();
        let region = game.region().clone();
        let tile = Tile(region.cross_x, region.start.1 + 3);
        let gate = Building::Civic(Civic::Gate);
        game.campaign.research = Research::all_plans();
        assert!(game.act(Action::Build(gate, tile.offset(3, 0), 0)).is_err());
        game.act(Action::Build(gate, tile, 1)).unwrap();
        assert_eq!(game.city().building_at(tile).unwrap().rotation, 0);
        assert!(game.city().walkable(&region, tile));
        let from = tile.offset(0, 1);
        let to = tile.offset(0, -1);
        let mut walker = Walker::new(from, false);
        let city = game.campaign.settlements[0].as_mut().unwrap();
        walker.route = city.route_to(&region, from, to, false).unwrap();
        assert_eq!(walker.route.front(), Some(&tile));
        city.advance_walker(&mut walker, &region, Gifts::default(), 0.45, 0);
        assert!(walker.position[1] < f32::from(tile.1));
        assert_eq!(walker.route.back(), Some(&to));
    }

    #[test]
    fn building_uses_one_payment_and_remains_outside_the_reserved_footpath() {
        let mut game = fresh();
        let start = game.city().stock;
        let road = Tile(game.region().cross_x, game.region().start.1);
        assert!(game.act(Action::Build(Building::House, road, 0)).is_err());
        assert_eq!(game.city().stock, start);
        let target = build(&mut game, Building::House);
        let remaining = game.city().stock;
        assert!(game.act(Action::Build(Building::House, target, 0)).is_err());
        assert_eq!(game.city().stock, remaining);
        assert_eq!(game.city().count(Building::House), 1);
    }

    #[test]
    fn both_victories_unlock_the_next_chapter_and_save_the_village() {
        let mut seeded = fresh();
        prepare_bridge(&mut seeded);
        let previous = seeded.campaign.clone();
        let mut allied = Game::new(Region::campaign(), Some(previous.clone()));
        build(&mut allied, Building::Workshop);
        let region = allied.region().clone();
        stock_up(&mut allied, Material::Wood, region.rebuild[0]);
        stock_up(&mut allied, Material::Stone, region.rebuild[1]);
        allied.act(Action::Repair).unwrap();
        stock_up(&mut allied, region.gift.0, region.gift.1);
        allied.act(Action::Give).unwrap();
        assert_eq!(allied.city().outcome, Some(Outcome::Alliance));

        let mut conquered = Game::new(Region::campaign(), Some(previous));
        for &target in &region.guards {
            for _ in 0..2 {
                conquered.step(8.);
                if conquered.city().quest == Quest::Bridge
                    && conquered
                        .city()
                        .ritual_error(
                            conquered.region(),
                            Miracle::Wind,
                            target,
                            conquered.campaign.gifts,
                        )
                        .is_none()
                {
                    conquered.act(Action::Cast(Miracle::Wind, target)).unwrap();
                }
            }
        }
        assert_eq!(conquered.city().outcome, Some(Outcome::Victory));
        assert_eq!(conquered.city().quest, Quest::Districts);
        assert!(conquered.act(Action::Travel(1)).is_err());
        conquered.act(Action::Learn(Talent::Storm)).unwrap();
        for (index, district) in region.districts.iter().enumerate() {
            for _ in 0..2 {
                conquered.step(10.);
                conquered
                    .act(Action::Cast(Miracle::Thunder, district.center))
                    .unwrap();
                assert!(
                    conquered
                        .act(Action::Cast(Miracle::Thunder, district.center))
                        .is_err()
                );
            }
            conquered.act(Action::Settle(index)).unwrap();
            assert_eq!(
                conquered.city().districts[index].outcome,
                Some(Outcome::Victory)
            );
        }
        assert_eq!(conquered.campaign.unlocked, 1);
        conquered.act(Action::Learn(Talent::Rain)).unwrap();
        let old_count = conquered.city().buildings.len();
        conquered.act(Action::Travel(1)).unwrap();
        assert_eq!(conquered.city().quest, Quest::Fields);
        assert_eq!(conquered.campaign.gifts.training, 1);
        assert_eq!(conquered.campaign.gifts.rain, 1);
        conquered.act(Action::Travel(0)).unwrap();
        assert_eq!(conquered.city().buildings.len(), old_count);
    }

    #[test]
    fn all_chapters_have_reachable_sources_and_a_viable_peaceful_route() {
        let mut game = fresh();
        for index in 0..game.regions.len() {
            game.act(Action::Travel(index)).unwrap();
            let region = game.region();
            for (i, site) in game.city().buildings.iter().enumerate() {
                for cell in site.kind.tiles(site.tile) {
                    assert!(
                        game.city().influence(region, cell)
                            && !region.rocky(cell)
                            && !region.water(cell)
                    );
                    assert!(
                        !region.road(cell)
                            && !region.fixed_structure(cell)
                            && region.source_at(cell).is_none()
                    );
                    assert!(
                        !game.city().buildings[..i]
                            .iter()
                            .any(|other| other.contains(cell))
                    );
                    assert!(
                        !game
                            .city()
                            .walkers
                            .iter()
                            .any(|walker| walker.tile() == cell)
                    );
                }
            }
            prepare_bridge(&mut game);
            let region = game.region().clone();
            build(&mut game, Building::Workshop);
            stock_up(
                &mut game,
                Material::Wood,
                region.rebuild[0]
                    + if region.gift.0 == Material::Wood {
                        region.gift.1
                    } else {
                        0
                    },
            );
            stock_up(
                &mut game,
                Material::Stone,
                region.rebuild[1]
                    + if region.gift.0 == Material::Stone {
                        region.gift.1
                    } else {
                        0
                    },
            );
            let bank = Tile(region.cross_x, region.river + region.water_width);
            let far = Tile(region.cross_x, region.river - 1);
            assert!(game.city().route_to(&region, bank, far, false).is_none());
            game.act(Action::Repair).unwrap();
            assert!(game.city().route_to(&region, bank, far, false).is_some());
            stock_up(&mut game, region.gift.0, region.gift.1);
            game.act(Action::Give).unwrap();
            assert_eq!(
                game.city().outcome,
                Some(Outcome::Alliance),
                "{}",
                region.name
            );
            assert_eq!(game.city().quest, Quest::Districts);
            for district in 0..region.districts.len() {
                serve_district(&mut game, district);
            }
            assert_eq!(game.city().quest, Quest::Complete);
            if region.kind == Biome::Valley {
                let before = game.city().faith;
                let ward = region.districts[0].center;
                let edge = ward.offset(7, 4);
                assert!(game.city().influence(&region, edge));
                assert!(!game.city().influence(&region, edge.offset(6, 0)));
                stock_for(&mut game, [3, 9, 0]);
                game.act(Action::Road(ward.offset(4, 2), edge.offset(-1, 0)))
                    .unwrap();
                game.act(Action::Build(Building::Waystone, edge, 0))
                    .unwrap();
                assert_eq!(game.city().faith, before - WAYSTONE_FAITH);
                assert!(game.city().influence(&region, edge.offset(6, 0)));
            }
            let saved: Campaign =
                serde_json::from_slice(&serde_json::to_vec(&game.campaign).unwrap()).unwrap();
            game = Game::new(Region::campaign(), Some(saved));
            assert_eq!(game.city().quest, Quest::Complete);
            assert!(!game.city().roads.is_empty());
        }
        assert!(game.campaign_complete());
    }

    #[test]
    fn a_legacy_save_keeps_the_bundle_lessons_and_chapters() {
        let mut game = fresh();
        prepare_bridge(&mut game);
        let source = game.region().sources[1].tile;
        game.act(Action::Hand(source)).unwrap();
        let mut legacy = serde_json::to_value(&game.campaign).unwrap();
        legacy["version"] = 1.into();
        legacy["unlocked"] = 1.into();
        legacy.as_object_mut().unwrap().remove("research");
        legacy["settlements"].as_array_mut().unwrap().truncate(4);
        let city = legacy["settlements"][0].as_object_mut().unwrap();
        for key in [
            "roads",
            "districts",
            "local_finished",
            "energy",
            "recovering",
            "cooldowns",
            "last_market",
            "society",
        ] {
            city.remove(key);
        }
        for entry in ["nodes", "regrowth"] {
            city.get_mut(entry)
                .unwrap()
                .as_array_mut()
                .unwrap()
                .truncate(ORIGINAL_SOURCE_COUNT);
        }
        for site in city["buildings"].as_array_mut().unwrap() {
            site.as_object_mut().unwrap().remove("upgraded");
        }
        city.insert("lesson".into(), 0.into());
        let saved = serde_json::from_value::<Campaign>(legacy).unwrap();
        let resumed = Game::new(Region::campaign(), Some(saved));
        assert_eq!(resumed.campaign.version, SAVE_VERSION);
        assert_eq!(resumed.campaign.unlocked, 1);
        assert_eq!(resumed.city().stock, game.city().stock);
        assert_eq!(
            resumed.city().held.unwrap().count,
            game.city().held.unwrap().count
        );
        assert!(matches!(resumed.city().lesson, Some(Lesson::Delivery(0))));
        assert_eq!(
            resumed.city().nodes[..ORIGINAL_SOURCE_COUNT],
            game.city().nodes[..ORIGINAL_SOURCE_COUNT]
        );
        assert_eq!(
            resumed.city().nodes[ORIGINAL_SOURCE_COUNT],
            resumed.region().sources[ORIGINAL_SOURCE_COUNT].stock
        );
    }

    #[test]
    fn a_coached_hippo_tends_food_and_mend_restores_energy() {
        let mut game = fresh();
        prepare_bridge(&mut game);
        let field = game
            .city()
            .buildings
            .iter()
            .find(|site| site.kind == Building::Field)
            .unwrap()
            .tile;
        assert!(game.act(Action::Tend(field)).is_err());
        for rank in 2..=3 {
            for _ in 0..1200 {
                game.step(1. / 24.);
                if game.city().lesson.is_some() {
                    break;
                }
            }
            game.act(Action::Praise).unwrap();
            assert_eq!(game.campaign.gifts.training, rank);
        }
        game.act(Action::Learn(Talent::Growth)).unwrap();
        game.act(Action::Tend(field)).unwrap();
        for _ in 0..800 {
            game.step(1. / 24.);
            if game
                .city()
                .building_at(field)
                .is_some_and(|site| matches!(site.crop, Crop::Growing(_)))
            {
                break;
            }
        }
        game.act(Action::Cast(Miracle::Bloom, field)).unwrap();
        assert!(matches!(
            game.city().building_at(field).unwrap().crop,
            Crop::Ripe
        ));
        let harvests = game.city().harvests;
        for _ in 0..240 {
            game.step(1. / 24.);
            if game.city().harvests > harvests {
                break;
            }
        }
        assert!(game.city().harvests > harvests);
        assert!(matches!(
            game.city().lesson,
            Some(Lesson::Practiced(Job::Tend(_)))
        ));
        let hippo = game.city().walkers[0].tile();
        let before = game.city().energy;
        game.act(Action::Cast(Miracle::Mend, hippo)).unwrap();
        assert!(game.city().energy > before);
    }

    fn linked_civic(game: &mut Game, kind: Civic) -> Tile {
        let building = Building::Civic(kind);
        let region = game.region();
        let tile = region
            .bounds
            .tiles()
            .filter(|&tile| tile.within(region.start, 7))
            .filter(|&tile| {
                building.tiles(tile).flat_map(Tile::neighbors).any(|cell| {
                    game.city().road_at(region, cell) && game.city().walkable(region, cell)
                })
            })
            .find(|&tile| game.building_plan(building, tile).is_ok())
            .unwrap_or_else(|| panic!("Kein Bauplatz für {}", building.name()));
        game.act(Action::Build(building, tile, 0)).unwrap();
        assert!(
            game.city()
                .connected(game.region(), game.city().building_at(tile).unwrap())
        );
        tile
    }

    #[test]
    fn research_building_chains_and_civilian_growth_work_together() {
        let mut game = fresh();
        assert!(game.act(Action::Research(Tech::Foundations)).is_err());
        prepare_bridge(&mut game);
        build(&mut game, Building::Workshop);
        let sample = game.region().start.offset(1, 2);
        assert!(
            game.building_plan(Building::Civic(Civic::Lodge), sample)
                .unwrap_err()
                .contains("Erforsche")
        );
        assert!(game.act(Action::Research(Tech::Smithing)).is_err());
        game.campaign.research.earn(30);
        for tech in [
            Tech::Foundations,
            Tech::Workshops,
            Tech::Smithing,
            Tech::Commons,
            Tech::Palisades,
            Tech::Militia,
        ] {
            let balance = game.campaign.research.insight;
            game.act(Action::Research(tech)).unwrap();
            assert_eq!(game.campaign.research.insight, balance - tech.price());
            assert!(game.act(Action::Research(tech)).is_err());
        }
        game.campaign.settlements[0].as_mut().unwrap().stock = [48, 48, 38];
        linked_civic(&mut game, Civic::Lodge);
        linked_civic(&mut game, Civic::Sawmill);
        linked_civic(&mut game, Civic::Masonry);
        linked_civic(&mut game, Civic::Garden);
        let before = game.city().inhabitants();
        for _ in 0..4 {
            game.campaign.settlements[0].as_mut().unwrap().provisions = 0.;
            game.step(1.);
        }
        assert!(game.city().inhabitants() > before);
        assert!(
            game.city()
                .walkers
                .iter()
                .any(|walker| walker.childhood > 0.)
        );
        assert!(game.city().society.morale > 60);
        assert!(game.city().society.goods[0] >= 4 && game.city().society.goods[1] >= 4);
        game.campaign.settlements[0].as_mut().unwrap().stock = [48, 48, 38];
        linked_civic(&mut game, Civic::Forge);
        let barracks = linked_civic(&mut game, Civic::Barracks);
        game.campaign.settlements[0].as_mut().unwrap().provisions = 0.;
        game.step(1.);
        assert_eq!(game.city().society.goods[2], 1);
        game.act(Action::Muster(barracks)).unwrap();
        assert_eq!(game.city().garrison(), 1);
        assert_eq!(game.city().society.goods[2], 0);
        assert!(game.act(Action::Muster(barracks)).is_err());
        let post = game.region().bridge().offset(0, 1);
        game.act(Action::Patrol(post)).unwrap();
        let save = serde_json::to_vec(&game.campaign).unwrap();
        let restored = Game::new(
            Region::campaign(),
            Some(serde_json::from_slice(&save).unwrap()),
        );
        assert!(restored.campaign.research.knows(Tech::Smithing));
        assert_eq!(restored.city().garrison(), 1);
        assert!(restored.city().walkers.iter().any(|walker| walker.guard
            && matches!(walker.job, Some(Job::Watch(target)) if target == post)));
    }

    #[test]
    fn an_existing_four_world_campaign_opens_the_new_islands_with_progress_intact() {
        let mut game = fresh();
        game.campaign.version = 2;
        game.campaign.settlements.truncate(4);
        game.campaign.active = 3;
        game.campaign.unlocked = 3;
        let mut previous = Settlement::new(&game.regions[3]);
        previous.quest = Quest::Complete;
        previous.outcome = Some(Outcome::Alliance);
        previous
            .districts
            .iter_mut()
            .for_each(|state| state.outcome = Some(Outcome::Alliance));
        game.campaign.settlements[3] = Some(previous);
        for (city, region) in game.campaign.settlements.iter_mut().zip(&game.regions) {
            if let Some(city) = city {
                city.nodes.truncate(region.original_sources);
                city.regrowth.truncate(region.original_sources);
            }
        }
        let mut upgraded = Game::new(Region::campaign(), Some(game.campaign));
        assert_eq!(upgraded.campaign.active, 3);
        assert_eq!(upgraded.campaign.unlocked, 4);
        assert!(upgraded.campaign.research.insight >= 12);
        upgraded.act(Action::Travel(4)).unwrap();
        assert_eq!(upgraded.region().kind, Biome::Aegean);
        let bay = Tile(upgraded.region().cross_x, -40);
        assert!(upgraded.region().water(bay));
        assert!(!upgraded.region().bridge_cell(bay));
    }
}
