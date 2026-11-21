#[path = "island_work.rs"]
mod island_work;
pub use island_work::{Blueprint, Dedication, IslandWork, WorkStage};
#[path = "neighborhood.rs"]
mod neighborhood;
pub use neighborhood::SERVICES;
#[path = "village_projects.rs"]
mod village_projects;
pub use village_projects::{Need, Projects};
#[path = "village_start.rs"]
mod village_start;
pub use village_start::{VillageStart, VillageTask};
#[path = "influence.rs"]
mod influence;
pub use influence::{Allegiance, Contest, OCCUPATION_GRACE};
#[path = "district_trade.rs"]
mod district_trade;
#[path = "gathering.rs"]
mod gathering;
pub use gathering::{Gatherings, Rite};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, VecDeque},
};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::civic::{Civic, Goods, Research, Tech, goods_text};
use crate::growth::{CreatureGrowth, Reputation, Tone};
use crate::modes::Mode;
use crate::quests::{Goal, QuestStep};
use crate::region::{
    AREA_COUNT, AREA_RADIUS, Biome, CELL, DistrictNeed, Material, Region, Tile, Weather,
    WorldLayout,
};
use crate::rivals::Rival;
use crate::terrain;

pub const SAVE_VERSION: u32 = 6;
pub const RAID_NOTICE: f32 = 30.;
pub const PAW_BUNDLE: u16 = 6;
pub const PAW_LIMIT: u16 = 18;
pub const FARM_REWARD: Stock = [0, 5, 8];
const RIVAL_RESOLVE: u8 = 3;
pub const LOW_SUPPLY: u16 = 8;
const SUPPLY_LOOK: f32 = 1.1;
const ORIGINAL_SOURCE_COUNT: usize = 10;
pub const START_RADIUS: i16 = 7;
pub const FRONTIER_RADIUS: i16 = 8;
const RIVAL_RADIUS: i16 = 6;
pub const DISTRICT_RADIUS: i16 = 6;
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

#[derive(Clone, Copy)]
pub struct BuildingUpgrade {
    pub materials: Stock,
    pub believers: usize,
    pub faith: u16,
    pub benefit: &'static str,
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

    pub fn upgrade(self) -> Option<BuildingUpgrade> {
        let (materials, believers, faith, benefit) = match self {
            Self::House => ([7, 5, 2], 3, 8, "+1 Wohnplatz"),
            Self::Field => ([4, 6, 0], 3, 8, "+6 Nahrung je Ernte"),
            Self::Store => ([8, 6, 0], 3, 8, "+20 Lagerplatz pro Rohstoff"),
            Self::Granary => ([8, 6, 0], 4, 16, "+20 Lagerplatz pro Rohstoff"),
            Self::Well => (
                [4, 8, 0],
                4,
                14,
                "Bewässerungsradius steigt von 5 auf 8 Felder",
            ),
            Self::Market => ([10, 6, 4], 5, 20, "6 statt 4 Baustoffe je Handel"),
            Self::Tower => ([4, 8, 0], 5, 18, "Schutzradius steigt von 5 auf 9 Felder"),
            Self::Civic(Civic::Lodge) => ([8, 6, 2], 5, 18, "+2 Wohnplätze"),
            Self::Civic(Civic::Courtyard) => ([8, 6, 2], 6, 20, "+2 Wohnplätze"),
            Self::Civic(Civic::Forge) => ([8, 6, 2], 5, 20, "2 statt 1 Werkzeug je Fertigung"),
            Self::Civic(Civic::Bastion) => {
                ([8, 6, 2], 6, 24, "Schutzradius steigt von 10 auf 14 Felder")
            }
            _ => return None,
        };
        Some(BuildingUpgrade {
            materials,
            believers,
            faith,
            benefit,
        })
    }

    pub fn use_text(self) -> &'static str {
        match self {
            Self::House => "+1 Wohnplatz für Gläubige",
            Self::Store => "+24 Lagerplatz pro Rohstoff",
            Self::Granary => "+48 Lagerplatz pro Rohstoff",
            Self::Field => "Wächst mit Regen. Die Ernte bringt Nahrung.",
            Self::Shrine => "Erweitert den Einfluss am Übergang und lädt Wunderkraft schneller",
            Self::Workshop => "Schaltet Speicher und Ausbau frei",
            Self::Well => "Bewässert Felder im Umkreis von 5 Feldern",
            Self::Market => "Tauscht 2 Nahrung gegen 4 Holz oder Stein",
            Self::Tower => "Schützt gegen Überfälle im Umkreis von 5 Feldern",
            Self::Waystone => "Erweitert deinen Einfluss und verbraucht 5 Glauben",
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
    #[serde(default)]
    batch_finished: bool,
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
    pub last_deal: Option<(u32, bool)>,
    pub resolve: u8,
    pub outcome: Option<Outcome>,
}

impl Default for DistrictStatus {
    fn default() -> Self {
        Self {
            last_deal: None,
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
    pub creature: CreatureGrowth,
    pub gathers_low_wood: bool,
    gathers_low_stone: bool,
    gathers_low_food: bool,
}

impl Gifts {
    pub fn supplies(self, material: Material) -> bool {
        match material {
            Material::Wood => self.gathers_low_wood,
            Material::Stone => self.gathers_low_stone,
            Material::Food => self.gathers_low_food,
        }
    }

    fn teach_supply(&mut self, material: Material) {
        match material {
            Material::Wood => self.gathers_low_wood = true,
            Material::Stone => self.gathers_low_stone = true,
            Material::Food => self.gathers_low_food = true,
        }
    }

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
    Grab(Tile, u16),
    Build(Building, Tile, u8),
    Road(Tile, Tile),
    Upgrade(Tile),
    Charter(Tile),
    SeeWork,
    AcceptWork,
    SupplyWork,
    SummonBuilder,
    RaiseWork,
    Dedicate(Dedication),
    TrackWork(bool),
    TrackLocal(bool),
    Research(Tech),
    Muster(Tile),
    Patrol(Tile),
    Settle(usize),
    MeetNeighbor(usize),
    BeginRivalry,
    DeferAppeal(usize),
    Visit(usize, bool),
    Audience(usize),
    Withdraw(usize),
    VillageGift(usize),
    SendPilgrims(usize),
    Aid(usize, bool),
    IslandPact,
    Trade(usize, bool),
    Camp(usize),
    Assign(Tile, bool),
    AssignArea(usize, Material, bool),
    CancelArea(usize, Material),
    ShareMeal(Tile),
    Call(Rite, Tile),
    Invoke(bool),
    DismissCircle,
    Produce(Tile),
    AcceptLocal,
    FinishLocal,
    RecallHelpers,
    MoveHippo(Tile),
    Watch(Tile),
    Tend(Tile),
    Pet,
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
    #[serde(alias = "RefillWood")]
    Refill(usize),
    Move(Tile),
    Visit(usize),
    Watch(Tile),
    Tend(Tile),
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Lesson {
    Delivery(usize),
    Practiced(Job),
}

impl Lesson {
    fn job(self) -> Job {
        match self {
            Self::Delivery(source) => Job::Gather(source),
            Self::Practiced(job) => job,
        }
    }

    fn material(self, region: &Region) -> Option<Material> {
        match self.job() {
            Job::Gather(source) | Job::Refill(source) => Some(region.sources[source].material),
            Job::GatherArea(_, material) => Some(material),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum FirstDelivery {
    Village,
    Farm,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
struct Opening {
    choice: Option<FirstDelivery>,
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
    meal_day: Option<u32>,
}

impl Default for Society {
    fn default() -> Self {
        Self {
            goods: [0; 3],
            morale: 58,
            births: 0,
            meal_day: None,
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
    pub gatherings: Gatherings,
    #[serde(default)]
    pub contest: Contest,
    #[serde(default)]
    pub projects: Projects,
    #[serde(default)]
    pub village_start: Option<VillageStart>,
    #[serde(default)]
    pub work: IslandWork,
    #[serde(default)]
    pub citizens: Vec<Tile>,
    #[serde(default)]
    pub journal: VecDeque<String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    opening: Option<Opening>,
    pub quest: Quest,
    pub outcome: Option<Outcome>,
    pub bridge_fixed: bool,
    pub gift_given: bool,
    #[serde(default)]
    crossing_delivery: Stock,
    pub guards: Vec<u8>,
    #[serde(default = "rival_starting_resolve")]
    pub rival_resolve: u8,
    pub hand_deliveries: u16,
    pub helper_deliveries: u16,
    pub hippo_deliveries: u16,
    #[serde(skip)]
    pub last_hippo_load: u16,
    #[serde(default)]
    hippo_harvests: u16,
    #[serde(default)]
    hippo_watches: u16,
    #[serde(default)]
    pub reputation: Reputation,
    pub harvests: u16,
    #[serde(default)]
    pub olive_pressings: u16,
    pub lesson: Option<Lesson>,
    #[serde(default = "full_energy")]
    pub energy: f32,
    #[serde(default)]
    pet_cooldown: f32,
    #[serde(default)]
    pub care_interactions: u16,
    #[serde(default)]
    recovering: bool,
    #[serde(default, alias = "wood_help_paused")]
    supply_help_paused: bool,
    #[serde(default)]
    cooldowns: [f32; Miracle::ALL.len()],
    #[serde(default)]
    pub local_finished: bool,
    #[serde(default)]
    pub local_accepted: bool,
    #[serde(default)]
    pub local_tracked: bool,
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

fn rival_starting_resolve() -> u8 {
    RIVAL_RESOLVE
}

impl Settlement {
    fn completed_work(&self) -> [u16; 3] {
        [
            self.hippo_deliveries.saturating_add(self.gatherings.hauls),
            self.hippo_harvests,
            self.hippo_watches
                .saturating_add(self.contest.creature_visits),
        ]
    }

    pub fn new(region: &Region) -> Self {
        Self::with_village_start(region, Self::has_village_start(region))
    }

    fn with_village_start(region: &Region, guided: bool) -> Self {
        let mut city = Self {
            world_layout: region.world_layout(),
            stock: region.initial,
            faith: 2,
            mana: 20.,
            buildings: Vec::new(),
            society: Society::default(),
            gatherings: Gatherings::default(),
            contest: Contest::default(),
            projects: Projects::default(),
            village_start: guided.then_some(VillageStart::Building),
            work: IslandWork::default(),
            citizens: Vec::new(),
            journal: VecDeque::new(),
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
            opening: region
                .opening_farm()
                .filter(|_| !guided)
                .map(|_| Opening::default()),
            quest: Quest::Shelter,
            outcome: None,
            bridge_fixed: false,
            gift_given: false,
            crossing_delivery: [0; 3],
            guards: vec![2; region.guards.len()],
            rival_resolve: RIVAL_RESOLVE,
            hand_deliveries: 0,
            helper_deliveries: 0,
            hippo_deliveries: 0,
            last_hippo_load: 0,
            hippo_harvests: 0,
            hippo_watches: 0,
            reputation: Reputation::default(),
            harvests: 0,
            olive_pressings: 0,
            lesson: None,
            energy: full_energy(),
            pet_cooldown: 0.,
            care_interactions: 0,
            recovering: false,
            supply_help_paused: false,
            cooldowns: [0.; Miracle::ALL.len()],
            local_finished: false,
            local_accepted: false,
            local_tracked: false,
            shield: 0.,
            rainfall: 0.,
            weather_time: 0.,
            raid_clock: 0.,
            provisions: 22.,
            next_worker: 0,
        };
        // The first chapter teaches carrying and building. Later chapters start with a village.
        for (index, &tile) in region.starter_plots.iter().enumerate().filter(|_| !guided) {
            let kind = [Building::House, Building::Store, Building::Field][index];
            city.buildings.push(BuildingSite {
                kind,
                tile,
                rotation: 0,
                crop: Crop::Dry,
                upgraded: false,
                batch_finished: false,
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
        city.populate_villages(region);
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
        2 + self.citizens.len()
            + self
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
            + self.gatherings.renown.min(8)
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
                || region
                    .villages
                    .iter()
                    .zip(&self.contest.towns)
                    .any(|(v, t)| t.side == Allegiance::Ours && tile.within(v.center, 9))
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

    pub fn harvest_at(&self, region: &Region, tile: Tile) -> Option<usize> {
        region.source_at(tile).or_else(|| {
            let i = region.wild_index(tile)?;
            (self.building_at(tile).is_none()
                && !self.roads.contains(&tile)
                && self.opening_farm(region) != Some(tile))
            .then_some(i)
        })
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
            && self.opening_farm(region) != Some(tile)
            && region.source_at(tile).is_none()
            && self.building_at(tile).is_none_or(|site| site.kind.gate())
    }

    pub fn opening_farm(&self, region: &Region) -> Option<Tile> {
        self.opening.and_then(|_| region.opening_farm())
    }

    pub fn opening_choice(&self) -> Option<FirstDelivery> {
        self.opening.and_then(|opening| opening.choice)
    }

    pub fn waiting_for_first_delivery(&self) -> bool {
        self.opening.is_some_and(|opening| opening.choice.is_none())
    }

    pub fn first_delivery_ready(&self, region: &Region) -> bool {
        self.waiting_for_first_delivery()
            && self.held.is_some_and(|bundle| {
                bundle.count >= PAW_BUNDLE
                    && region.sources[bundle.source].material == Material::Wood
            })
    }

    pub fn first_delivery_at(&self, region: &Region, tile: Tile) -> Option<FirstDelivery> {
        if self.waiting_for_first_delivery() && region.opening_farm() == Some(tile) {
            Some(FirstDelivery::Farm)
        } else if tile == region.depot && self.first_delivery_ready(region) {
            Some(FirstDelivery::Village)
        } else {
            None
        }
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
        let upgrade = site
            .kind
            .upgrade()
            .ok_or("Dieses Gebäude hat keine zweite Stufe.")?;
        if site.upgraded {
            return Err("Das Gebäude ist bereits ausgebaut.".into());
        }
        if !region.sandbox && self.city_rank() < 3 {
            return Err("Baue zuerst eine Werkstatt.".into());
        }
        if !self.connected(region, site) {
            return Err("Schließe das Gebäude an einen Dorfweg an.".into());
        }
        if region.sandbox {
            return Ok((index, [0; 3]));
        }
        let believers = self.inhabitants();
        if believers < upgrade.believers {
            return Err("Mehr Gläubige brauchen zusätzliche Wohnplätze.".into());
        }
        if self.faith < upgrade.faith {
            return Err("Bauen, Ernten und Dorfhilfe bringen mehr Glauben.".into());
        }
        self.material_requirements(upgrade.materials, [0; 3])?;
        Ok((index, upgrade.materials))
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
        let benefit = kind.upgrade().expect("validated upgrade").benefit;
        Ok(format!("{} ausgebaut · {benefit}", kind.name()))
    }

    pub fn district_need(need: DistrictNeed) -> &'static str {
        match need {
            DistrictNeed::Water => "Brunnen",
            DistrictNeed::Trade => "Markt",
            DistrictNeed::Shelter => "ausgebaute Hütte",
            DistrictNeed::Patrol => "Wachturm",
        }
    }

    pub fn district_site(&self, region: &Region, index: usize) -> Option<&BuildingSite> {
        let district = region.districts.get(index)?;
        self.buildings
            .iter()
            .filter(|site| {
                site.kind == district.need.building()
                    && site.tile.within(district.center, DISTRICT_RADIUS)
                    && self.connected(region, site)
            })
            .max_by_key(|site| site.upgraded)
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
        let served = self
            .district_site(region, index)
            .is_some_and(|site| district.need != DistrictNeed::Shelter || site.upgraded);
        if !served && !self.gatherings.welcomed[index] {
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
        if let Some(t) = self.contest.towns.get_mut(index) {
            t.side = Allegiance::Ours;
            t.trust = 100.;
        }
        self.faith = self.faith.saturating_add(8);
        Ok(format!(
            "{} · {} · Sammelplatz und Handel eröffnet",
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

    fn material_requirements(&self, price: Stock, goods: Goods) -> Result<(), String> {
        if self.affordable(price) && self.can_buy_goods(goods) {
            return Ok(());
        }
        let raw = stock_text(std::array::from_fn(|i| {
            price[i].saturating_sub(self.stock[i])
        }));
        let refined = goods_text(std::array::from_fn(|i| {
            goods[i].saturating_sub(self.society.goods[i])
        }));
        let missing = [raw, refined]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        Err(format!("Es fehlen {}", missing.join(" · ")))
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
        self.placement_check(region, kind, tile, true)
    }

    pub fn placement_site(&self, region: &Region, kind: Building, tile: Tile) -> bool {
        self.placement_check(region, kind, tile, false).is_ok()
    }

    fn blueprint_requirements(&self, region: &Region, kind: Building) -> Result<(), String> {
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
                Building::Shrine if self.quest != Quest::Bridge || self.faith < region.faith => {
                    return Err(format!(
                        "Für den Grenzschrein brauchst du den dritten Auftrag und {} Glauben.",
                        region.faith
                    ));
                }
                Building::Workshop | Building::Well if self.city_rank() < 2 => {
                    return Err("Baue ein Lager und ein Feld für Dorfstufe 2.".into());
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
                _ => {}
            }
        }
        Ok(())
    }

    fn placement_check(
        &self,
        region: &Region,
        kind: Building,
        tile: Tile,
        check_cost: bool,
    ) -> Result<(), String> {
        self.blueprint_requirements(region, kind)?;
        match kind {
            Building::Shrine if !region.sandbox && tile != region.frontier => {
                return Err("Wähle den goldenen Schreinplatz am Übergang.".into());
            }
            Building::Waystone
                if !region.sandbox
                    && !tile
                        .neighbors()
                        .into_iter()
                        .any(|cell| self.road_at(region, cell) && self.walkable(region, cell)) =>
            {
                return Err("Setze den Grenzstein neben einen Weg an der goldenen Grenze.".into());
            }
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
            if region.village_at(cell).is_some()
                || (!region.tutorial && cell.within(region.rival_home(), 2))
            {
                return Err("Lass den Dorfplatz und die Häuser der Nachbarn frei.".into());
            }
            if self
                .gatherings
                .circle
                .as_ref()
                .is_some_and(|circle| circle.guests.iter().any(|(_, spot)| *spot == cell))
            {
                return Err("Dieser Platz gehört gerade zum Festkreis.".into());
            }
            if !self.influence(region, cell) {
                return Err("Der ganze Grundriss muss im Einflussgebiet liegen.".into());
            }
            if region.water(cell) {
                return Err("Verschiebe den Grundriss ganz aufs Land.".into());
            }
            if self.building_at(cell).is_some() || (!kind.gate() && self.road_at(region, cell)) {
                return Err("Der Grundriss überlappt ein Gebäude oder einen Weg.".into());
            }
            if !self.walkable(region, cell)
                || (!region.sandbox && cell == region.frontier && kind != Building::Shrine)
            {
                return Err("Verschiebe den Grundriss auf freien Boden.".into());
            }
            if self.walkers.iter().any(|walker| walker.tile() == cell) {
                return Err("Warte kurz, bis das Feld frei ist.".into());
            }
        }
        if check_cost && !region.sandbox {
            self.material_requirements(kind.price(), kind.goods_price())?;
        }
        if kind.gate()
            || self.footprint_has_detour(region, kind, tile)
            || self.footprint_keeps_routes(region, kind, tile)
        {
            Ok(())
        } else {
            Err("Diese Stelle würde einen Sammelweg versperren.".into())
        }
    }

    fn footprint_keeps_routes(&self, region: &Region, kind: Building, tile: Tile) -> bool {
        // Check that the footprint leaves each currently reachable resource accessible.
        let depot_edge = region.depot.offset(1, 0);
        let new_footprint = |p| kind.tiles(tile).any(|cell| cell == p);
        let bounds = region.bounds;
        let walk = self.walk_grid(region);
        let before = Self::reachable(region, &walk, depot_edge, |_| false);
        let after = Self::reachable(region, &walk, depot_edge, new_footprint);
        let source_blocked = region.sources[..region.managed_sources]
            .iter()
            .any(|source| {
                let accessible = |mask: &[bool]| {
                    source
                        .tile
                        .neighbors()
                        .into_iter()
                        .filter_map(|p| bounds.index(p))
                        .any(|index| mask[index])
                };
                accessible(&before) && !accessible(&after)
            });
        let walker_blocked = self.walkers.iter().any(|walker| {
            bounds
                .index(walker.tile())
                .is_some_and(|index| before[index] && !after[index])
        });
        !source_blocked && !walker_blocked
    }

    fn footprint_has_detour(&self, region: &Region, kind: Building, tile: Tile) -> bool {
        if kind
            .tiles(tile)
            .any(|cell| cell == region.depot.offset(1, 0))
        {
            return false;
        }
        let size = kind.size();
        let rim = (-1..size)
            .map(|x| tile.offset(x, -1))
            .chain((-1..size).map(|z| tile.offset(size, z)))
            .chain((0..=size).rev().map(|x| tile.offset(x, size)))
            .chain((0..=size).rev().map(|z| tile.offset(-1, z)));
        // A continuous walkable rim can replace every route through the footprint.
        // Sites touching a resource still need the full check for its access tiles.
        let mut previous = self.walkable(region, tile.offset(-1, 0));
        let mut runs = 0;
        for cell in rim {
            if region.source_at(cell).is_some() {
                return false;
            }
            let open = self.walkable(region, cell);
            runs += u8::from(open && !previous);
            if runs > 1 {
                return false;
            }
            previous = open;
        }
        true
    }

    fn walk_grid(&self, region: &Region) -> Vec<bool> {
        self.route_grid(region, false)
    }
    fn route_grid(&self, region: &Region, pilgrims: bool) -> Vec<bool> {
        let bounds = region.bounds;
        let mut walk = region.route_grid(self.bridge_fixed, pilgrims);
        if let Some(index) = self
            .opening_farm(region)
            .and_then(|tile| bounds.index(tile))
        {
            walk[index] = false;
        }
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
        self.travel_route(region, start, target, beside, false)
    }
    fn travel_route(
        &self,
        region: &Region,
        start: Tile,
        target: Tile,
        beside: bool,
        pilgrims: bool,
    ) -> Option<VecDeque<Tile>> {
        let bounds = region.bounds;
        let start_index = bounds.index(start)?;
        let target_index = bounds.index(target)?;
        if start.distance(target) == u16::from(beside) {
            return Some(VecDeque::new());
        }
        let walk = self.route_grid(region, pilgrims);
        let goal_open = if beside {
            target
                .neighbors()
                .into_iter()
                .any(|tile| bounds.index(tile).is_some_and(|index| walk[index]))
        } else {
            walk[target_index]
        };
        if !goal_open {
            return None;
        }
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

    fn deposit(&mut self, bundle: Bundle, region: &Region) -> Result<u16, String> {
        let material = region.sources[bundle.source].material;
        let accepted = bundle.count.min(
            self.storage_cap()
                .saturating_sub(self.stock[material.index()]),
        );
        if accepted == 0 {
            return Err(
                "Das Lager ist voll. Baue ein weiteres Lager oder verbrauche Vorräte.".into(),
            );
        }
        self.stock[material.index()] += accepted;
        Ok(accepted)
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

    pub fn carry_destination(&self, region: &Region, from: Tile) -> Option<Tile> {
        let bundle = self.held?;
        if self.first_delivery_ready(region) {
            return Some(region.depot);
        }
        let material = region.sources[bundle.source].material;
        Some(if self.crossing_due(region)[material.index()] > 0 {
            region.bridge()
        } else if self.can_deposit(material, 1) {
            self.nearest_store(region, from)
        } else {
            region.sources[bundle.source].tile
        })
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

    pub fn meal_ready(&self, region: &Region, tile: Tile) -> Result<(), &'static str> {
        if !self.influence(region, tile)
            || !(self
                .building_at(tile)
                .is_some_and(|site| site.kind == Building::House)
                || self
                    .walkers
                    .iter()
                    .any(|walker| !walker.hippo && !walker.guard && walker.tile() == tile))
        {
            return Err("Teile das Essen an einer Hütte oder bei einer Familie.");
        }
        if self.society.meal_day == Some((self.weather_time / DAY_LENGTH) as u32) {
            return Err("Heute hat das Dorf schon gemeinsam gegessen.");
        }
        if self.stock[2] < 3 {
            return Err("Ein gemeinsames Essen kostet drei Nahrung.");
        }
        Ok(())
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

    fn raids_active(&self, region: &Region) -> bool {
        !self.rival_waiting()
            && self.quest == Quest::Bridge
            && self.count(Building::Shrine) > 0
            && (self.guards.iter().any(|&spirit| spirit > 0) || self.rival_active(region))
    }

    pub fn next_raid(&self, region: &Region) -> Option<f32> {
        (self.raids_active(region) && self.raid_clock > 0.).then_some(self.raid_clock)
    }

    pub fn raid_warning(&self, region: &Region) -> bool {
        self.next_raid(region)
            .is_some_and(|seconds| seconds <= RAID_NOTICE)
    }

    pub fn rival_active(&self, region: &Region) -> bool {
        region.has_rival_creature() && self.rival_resolve > 0 && self.outcome.is_none()
    }

    pub fn rival_world(&self, region: &Region) -> Vec3 {
        let home = region.rival_lair().world();
        let approach =
            if self.raids_active(region) && self.raid_clock > 0. && self.rival_active(region) {
                1. - terrain::smooth(self.raid_clock / RAID_NOTICE)
            } else {
                0.
            };
        home.lerp(region.bridge().offset(0, 1).world(), approach)
    }

    pub fn rival_target(&self, region: &Region, radius: i16) -> Option<Tile> {
        let creature = Tile::from_world(self.rival_world(region));
        (-radius..=radius)
            .flat_map(|z| (-radius..=radius).map(move |x| creature.offset(x, z)))
            .filter(|&tile| tile.within(creature, radius) && self.influence(region, tile))
            .min_by_key(|&tile| tile.distance(creature))
    }

    fn rival_warded(&self, region: &Region) -> bool {
        let facilities: &[Building] = match region.kind {
            Biome::Valley => &[Building::Civic(Civic::Forester)],
            Biome::Fjord => &[Building::Granary, Building::Civic(Civic::WinterHall)],
            Biome::Terraces => &[Building::Well],
            Biome::Lake => &[Building::Civic(Civic::WaterTerrace)],
            Biome::Aegean => &[Building::Civic(Civic::Oracle)],
        };
        let needed = if region.kind == Biome::Terraces { 2 } else { 1 };
        self.buildings
            .iter()
            .filter(|site| facilities.contains(&site.kind) && self.connected(region, site))
            .take(needed)
            .count()
            == needed
    }

    fn rival_strike(&mut self, region: &Region) {
        match region.kind {
            Biome::Valley => self.stock[0] = self.stock[0].saturating_sub(4),
            Biome::Fjord => self.stock[2] -= self.stock[2].saturating_sub(4).min(4),
            Biome::Terraces => {
                for field in self
                    .buildings
                    .iter_mut()
                    .filter(|site| site.kind == Building::Field)
                {
                    if let Crop::Growing(seconds) = &mut field.crop {
                        *seconds = (*seconds + 9.).min(31.);
                    }
                }
            }
            Biome::Lake => {
                self.energy = (self.energy - 19.).max(0.);
                for walker in self.walkers.iter_mut().filter(|walker| !walker.guard) {
                    walker.pause = walker.pause.max(3.9);
                }
            }
            Biome::Aegean => self.mana = (self.mana - 8.).max(0.),
        }
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
                let ford = region.ford(step)
                    && (walker.hippo || matches!(walker.job, Some(Job::Move(_) | Job::Visit(_))));
                if !ford && !self.walkable(region, step) {
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

        let circle_spot = self
            .gatherings
            .circle
            .as_ref()
            .and_then(|c| c.guests.iter().find(|(i, _)| *i == index).map(|(_, t)| *t));
        if let Some(bundle) = walker.bundle
            && (circle_spot.is_none()
                || self.stock[region.sources[bundle.source].material.index()] < self.storage_cap())
        {
            if let Some(route) = self.route_to_store(region, walker.tile()) {
                if !route.is_empty() {
                    walker.route = route;
                    return;
                }
                if let Ok(accepted) = self.deposit(bundle, region) {
                    if accepted < bundle.count {
                        walker.bundle = Some(Bundle {
                            count: bundle.count - accepted,
                            ..bundle
                        });
                        walker.pause = 1.;
                        return;
                    }
                    walker.bundle = None;
                    walker.pause = 0.45;
                    if walker.hippo {
                        self.hippo_deliveries = self.hippo_deliveries.saturating_add(1);
                        self.last_hippo_load = accepted;
                        self.note(format!(
                            "Nilpferd liefert {accepted} {}",
                            region.kind.material(region.sources[bundle.source].material)
                        ));
                        let completed = walker.job.filter(|job| match *job {
                            Job::Gather(source) | Job::Refill(source) => source == bundle.source,
                            Job::GatherArea(area, material) => region
                                .area_sources(area, material)
                                .any(|(source, _)| source == bundle.source),
                            _ => false,
                        });
                        self.lesson = Some(match completed {
                            Some(job @ Job::GatherArea(..)) => Lesson::Practiced(job),
                            _ => Lesson::Delivery(bundle.source),
                        });
                        if matches!(completed, Some(Job::Refill(_)))
                            || (gifts.training == 0 && completed.is_some())
                        {
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

        if let Some(spot) = circle_spot {
            if walker.tile() != spot {
                walker.route = self
                    .route_to(region, walker.tile(), spot, false)
                    .unwrap_or_default();
            }
            if walker.route.is_empty() {
                walker.pause = 0.65;
            }
            return;
        }

        match walker.job {
            Some(Job::Visit(i)) => {
                if let Some(v) = region.villages.get(i) {
                    if walker.tile().distance(v.center) > 1 {
                        walker.route = self
                            .travel_route(region, walker.tile(), v.center, true, true)
                            .unwrap_or_default();
                    }
                    walker.pause = if walker.route.is_empty() { 1.0 } else { 0. };
                }
            }
            Some(Job::Move(target)) => {
                let route = self.travel_route(region, walker.tile(), target, false, true);
                walker.route = route.unwrap_or_default();
                if walker.route.is_empty() {
                    walker.job = None;
                }
            }
            Some(job @ (Job::Gather(_) | Job::GatherArea(..) | Job::Refill(_))) => {
                if let Job::Gather(source) | Job::Refill(source) = job
                    && source >= region.managed_sources
                    && self.harvest_at(region, region.sources[source].tile) != Some(source)
                {
                    walker.job = None;
                    return;
                }
                let next = match job {
                    Job::Gather(source) | Job::Refill(source) if self.nodes[source] > 0 => self
                        .route_to(region, walker.tile(), region.sources[source].tile, true)
                        .map(|route| (source, route)),
                    Job::GatherArea(area, material) if self.can_deposit(material, 1) => {
                        self.area_collection(region, area, material, walker.tile())
                    }
                    _ => None,
                };
                let Some((source, route)) = next else {
                    if matches!(job, Job::Refill(_)) {
                        walker.job = None;
                    }
                    walker.pause = match job {
                        Job::Gather(source) if self.nodes[source] > 0 => 1.,
                        _ => 2.,
                    };
                    return;
                };
                if route.is_empty() {
                    let mut count = if walker.hippo {
                        5 + u16::from(gifts.training) * 2 + gifts.creature.transport_bonus()
                    } else {
                        3
                    };
                    let material = region.sources[source].material;
                    count = count.min(
                        self.storage_cap()
                            .saturating_sub(self.stock[material.index()]),
                    );
                    if count == 0 && matches!(job, Job::Refill(_)) {
                        walker.job = None;
                    }
                    walker.bundle = self.take(source, count);
                    walker.pause = if walker.bundle.is_some() { 0.65 } else { 1.5 };
                } else {
                    walker.route = route;
                }
            }
            Some(Job::Watch(target)) if walker.tile() != target => {
                if let Some(route) =
                    self.travel_route(region, walker.tile(), target, false, walker.hippo)
                {
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
                                self.hippo_watches = self.hippo_watches.saturating_add(1);
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
                        self.hippo_harvests = self.hippo_harvests.saturating_add(1);
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
            None if walker.hippo
                && !region.tutorial
                && !self.supply_help_paused
                && Material::ALL
                    .into_iter()
                    .any(|m| gifts.supplies(m) && self.stock[m.index()] <= LOW_SUPPLY)
                && self.energy >= 40. =>
            {
                self.look_for_supply(walker, region, gifts);
            }
            None => self.daily_route(walker, region, index),
        }
    }

    fn look_for_supply(&self, walker: &mut Walker, region: &Region, gifts: Gifts) {
        let from = walker.tile();
        let mut candidates: Vec<_> = region
            .sources
            .iter()
            .enumerate()
            .filter(|(index, source)| {
                gifts.supplies(source.material)
                    && self.stock[source.material.index()] <= LOW_SUPPLY
                    && self.nodes[*index] > 0
                    && self.influence(region, source.tile)
            })
            .collect();
        candidates.sort_by_key(|(_, source)| {
            (
                self.stock[source.material.index()],
                from.distance(source.tile),
            )
        });
        if let Some((index, _)) = candidates
            .into_iter()
            .find(|(_, source)| self.route_to(region, from, source.tile, true).is_some())
        {
            walker.job = Some(Job::Refill(index));
            walker.pause = SUPPLY_LOOK;
        } else {
            walker.pause = 2.;
        }
    }

    fn tick(&mut self, region: &Region, gifts: Gifts, dt: f32) -> u16 {
        self.gatherings.rest = (self.gatherings.rest - dt).max(0.);
        self.pet_cooldown = (self.pet_cooldown - dt).max(0.);
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
        self.check_first_visit(region);
        self.visit_tick(region, dt);
        self.projects_tick(region, dt);

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
            self.supply_homes(region);
            self.operate_work(region, gifts);
            insight = self.civic_life(region, weather);
        }

        if self.raids_active(region) {
            let rival = self.rival_active(region);
            let interval = if rival {
                Rival::for_biome(region.kind).interval
            } else {
                90.
            };
            if self.raid_clock <= 0. {
                self.raid_clock = interval + 30.;
            }
            let previous = self.raid_clock;
            self.raid_clock -= dt;
            if previous > RAID_NOTICE && self.raid_warning(region) {
                self.note(
                    "Am Übergang sammelt sich ein Angriff. Öffne die Warnung für Hilfe.".into(),
                );
            }
            if self.raid_clock <= 0. {
                self.raid_clock = interval;
                let watch = |walker: &Walker| {
                    matches!(walker.job, Some(Job::Watch(_)))
                        && walker.tile().within(region.bridge(), 4)
                };
                let hippo_protects = watch(&self.walkers[0]);
                let protected = self.shield > 0.
                    || self.tower_protects(region, region.bridge())
                    || hippo_protects
                    || self
                        .walkers
                        .iter()
                        .any(|walker| walker.guard && watch(walker));
                if hippo_protects {
                    self.hippo_watches = self.hippo_watches.saturating_add(1);
                }
                if protected || (rival && self.rival_warded(region)) {
                    self.announce_village("Dein Dorf hat den Angriff abgewehrt.".into());
                } else if rival {
                    self.rival_strike(region);
                    self.announce_village(Rival::for_biome(region.kind).impact.into());
                } else {
                    let food = &mut self.stock[Material::Food.index()];
                    *food -= food.saturating_sub(4).min(2);
                    self.announce_village("Die Wachen haben bis zu 2 Nahrung genommen.".into());
                }
            }
        }
        insight
    }

    pub fn production_plan(
        &self,
        region: &Region,
        site: &BuildingSite,
    ) -> Result<(Stock, Goods, Goods), String> {
        let (cost, inputs, outputs) = match site.kind {
            Building::Civic(Civic::Sawmill) => ([2, 0, 0], [0; 3], [1, 0, 0]),
            Building::Civic(Civic::Masonry) => ([0, 2, 0], [0; 3], [0, 1, 0]),
            Building::Civic(Civic::Forge) => {
                ([0, 1, 0], [1, 0, 0], [0, 0, 1 + u16::from(site.upgraded)])
            }
            Building::Civic(Civic::OlivePress) => ([0, 0, 1], [0; 3], [0; 3]),
            _ => return Err("Dieser Betrieb hat keinen Fertigungsauftrag.".into()),
        };
        if !self.connected(region, site) {
            return Err("Der Betrieb braucht einen Anschluss an einen Dorfweg.".into());
        }
        if site.batch_finished {
            return Err(format!(
                "Auftrag erledigt · Nächste Schicht in {} s",
                self.next_production()
            ));
        }
        if site.kind == Building::Civic(Civic::OlivePress) && self.stock[2] < 8 {
            return Err(
                "Zum Pressen braucht das Dorf mindestens 8 Nahrung. Eine davon wird verbraucht."
                    .into(),
            );
        }
        self.material_requirements(cost, inputs)?;
        let cap = 32 + 8 * self.count(Building::Store).min(20) as u16;
        if (0..3).any(|i| self.society.goods[i] - inputs[i] + outputs[i] > cap) {
            return Err("Das Warenlager ist voll.".into());
        }
        Ok((cost, inputs, outputs))
    }

    fn produce(&mut self, region: &Region, tile: Tile, early: bool) -> GameResult {
        let index = self
            .buildings
            .iter()
            .position(|site| site.contains(tile))
            .ok_or("Wähle einen Betrieb.")?;
        let site = &self.buildings[index];
        let (cost, inputs, outputs) = self.production_plan(region, site)?;
        let press = site.kind == Building::Civic(Civic::OlivePress);
        self.pay(cost);
        self.pay_goods(inputs);
        for (total, amount) in self.society.goods.iter_mut().zip(outputs) {
            *total += amount;
        }
        self.buildings[index].batch_finished = early;
        if press {
            let faith = if region.weather(self.weather_time) == Weather::Breeze {
                5
            } else {
                3
            };
            self.olive_pressings = self.olive_pressings.saturating_add(1);
            self.faith = self.faith.saturating_add(faith);
            Ok(format!("Öl gepresst · +{faith} Glauben"))
        } else {
            Ok(format!("Fertig · {}", goods_text(outputs)))
        }
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
            .enumerate()
            .filter_map(|(index, site)| {
                site.kind
                    .civic()
                    .filter(|_| self.connected(region, site))
                    .map(|kind| (kind, site.tile, index))
            })
            .collect();
        active.sort_by_key(|(kind, _, _)| *kind as u8);
        let mut morale = if self.stock[2] >= 6 { 60 } else { 49 };
        if self.society.meal_day == Some((self.weather_time / DAY_LENGTH) as u32) {
            morale += 8;
        }
        let today = (self.weather_time / DAY_LENGTH) as u32;
        morale -= 6 * self
            .districts
            .iter()
            .filter(|d| d.last_deal == Some((today, true)))
            .count() as i16;
        match self.reputation.tone() {
            Tone::Gentle if self.stock[2] >= 6 => {
                morale += 7;
                self.faith = self.faith.saturating_add(1);
            }
            Tone::Stern => morale -= 6,
            _ => {}
        }
        let mut insight: u16 = 0;
        for (kind, tile, index) in active {
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
                Civic::Sawmill | Civic::Masonry | Civic::Forge | Civic::OlivePress => {
                    if !std::mem::take(&mut self.buildings[index].batch_finished) {
                        let _ = self.produce(region, tile, false);
                    }
                }
                Civic::Bakery if self.harvests > 0 => {
                    self.exchange([1, 0, 1], [0; 3], [0, 0, 4], [0; 3]);
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
                Civic::Temple => self.faith = self.faith.saturating_add(2),
                _ => {}
            }
        }
        self.society.morale = morale.clamp(0, 100) as u16;
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
            let range = range + i16::from(self.reputation.tone() == Tone::Stern) * 2;
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
        if let Some(destination) = self.first_delivery_at(region, target) {
            return self.deliver_first_wood(region, destination);
        }
        if !self.influence(region, target) {
            return Err("Die Wolkenpfote reicht nur bis zur goldenen Grenze.".into());
        }
        if let Some(bundle) = self.held {
            if target.within(region.bridge(), 1)
                && self.crossing_due(region)[region.sources[bundle.source].material.index()] > 0
            {
                return self.deliver_to_crossing(region, bundle);
            }
            if self.harvest_at(region, target) == Some(bundle.source) {
                self.nodes[bundle.source] = self.nodes[bundle.source]
                    .saturating_add(bundle.count)
                    .min(region.sources[bundle.source].stock);
                self.held = None;
                return Ok("Das Bündel liegt wieder an der Quelle.".into());
            }
            if !self.can_store_at(region, target) {
                return Err("Wähle einen Sammelplatz oder ein Lager zum Ablegen.".into());
            }
            let accepted = self.deposit(bundle, region)?;
            self.held = (accepted < bundle.count).then_some(Bundle {
                count: bundle.count - accepted,
                ..bundle
            });
            self.hand_deliveries = self.hand_deliveries.saturating_add(1);
            return Ok(format!(
                "{} {} abgelegt",
                accepted,
                region.kind.material(region.sources[bundle.source].material)
            ));
        }
        if self.harvest_at(region, target).is_some() {
            return self.grab(region, target, PAW_BUNDLE);
        }
        self.harvest(region, target).map(|amount| {
            format!("{amount} Nahrung nach Hause gebracht. Das Dorf schöpft Glauben.")
        })
    }

    fn deliver_first_wood(&mut self, region: &Region, destination: FirstDelivery) -> GameResult {
        if !self.first_delivery_ready(region) {
            return Err(format!("Jora braucht {PAW_BUNDLE} Holz für ihr Dach."));
        }
        let mut bundle = self.held.take().unwrap();
        bundle.count -= PAW_BUNDLE;
        self.opening.as_mut().unwrap().choice = Some(destination);
        self.hand_deliveries = self.hand_deliveries.saturating_add(1);
        let text = match destination {
            FirstDelivery::Village => {
                self.society.morale = self.society.morale.saturating_add(12).min(100);
                let plot = (self.count(Building::House) == 0)
                    .then(|| {
                        [(1, 0), (2, 1), (-3, -2), (1, 2)]
                            .into_iter()
                            .map(|(x, z)| region.start.offset(x, z))
                            .find(|&tile| self.placement_site(region, Building::House, tile))
                    })
                    .flatten();
                if let Some(tile) = plot {
                    self.raise_building(region, Building::House, tile, 0);
                }
                let deposit = bundle
                    .count
                    .min(self.storage_cap().saturating_sub(self.stock[0]));
                self.stock[0] += deposit;
                bundle.count -= deposit;
                if plot.is_some() {
                    "Miras Familie zieht in die erste Hütte ein.".into()
                } else {
                    "Mira verteilt das Holz. Die Stimmung im Dorf steigt.".into()
                }
            }
            FirstDelivery::Farm => {
                let capacity = self.storage_cap();
                self.stock
                    .iter_mut()
                    .zip(FARM_REWARD)
                    .for_each(|(stock, reward)| {
                        *stock = stock.saturating_add(reward).min(capacity);
                    });
                self.faith = self.faith.saturating_add(4);
                self.reputation.shift(8);
                "Joras Dach ist dicht. Sie schickt Vorräte.".into()
            }
        };
        self.held = (bundle.count > 0).then_some(bundle);
        Ok(text)
    }

    pub fn paw_limit(&self, region: &Region, tile: Tile) -> u16 {
        if self.held.is_some() || !self.influence(region, tile) {
            return 0;
        }
        self.harvest_at(region, tile)
            .map_or(0, |index| self.nodes[index].min(PAW_LIMIT))
    }

    fn grab(&mut self, region: &Region, tile: Tile, amount: u16) -> GameResult {
        let capacity = self.paw_limit(region, tile);
        let source = self
            .harvest_at(region, tile)
            .filter(|_| capacity > 0)
            .ok_or(
                "Zum Greifen brauchst du eine volle Quelle im Einflussgebiet und eine freie Pfote.",
            )?;
        let bundle = self
            .take(source, amount.clamp(1, capacity))
            .expect("Available paw load");
        self.held = Some(bundle);
        let destination = if self.first_delivery_ready(region) {
            "Wähle, wem du mit dem Holz hilfst."
        } else {
            "Das Ablageziel ist markiert."
        };
        Ok(format!(
            "{} {} in der Pfote. {destination}",
            bundle.count,
            region.kind.material(region.sources[source].material)
        ))
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
        if !region.sandbox {
            self.pay(kind.price());
            self.pay_goods(kind.goods_price());
        }
        self.raise_building(region, kind, tile, rotation);
        let linked = kind.civic().is_some_and(|civic| civic != Civic::Palisade)
            && !self.connected(region, self.buildings.last().unwrap());
        Ok(if linked {
            format!("{} gebaut · Anschluss an einen Weg fehlt", kind.name())
        } else {
            format!("{} gebaut", kind.name())
        })
    }

    fn raise_building(&mut self, region: &Region, kind: Building, tile: Tile, rotation: u8) {
        let rotation = self.build_rotation(region, kind, tile, rotation);
        self.buildings.push(BuildingSite {
            kind,
            tile,
            rotation,
            crop: Crop::Dry,
            upgraded: false,
            batch_finished: false,
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
            .travel_route(region, self.walkers[0].tile(), target, false, true)
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
        if walker.hippo {
            self.supply_help_paused = false;
        } else {
            self.next_worker = self.next_worker.wrapping_add(1);
        }
    }

    fn assign(&mut self, region: &Region, tile: Tile, hippo: bool) -> GameResult {
        let index = self
            .harvest_at(region, tile)
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
        if !self.walkable(region, tile)
            || self
                .travel_route(region, self.walkers[0].tile(), tile, false, true)
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

    pub fn pet_ready(&self) -> bool {
        self.pet_cooldown <= 0.
    }

    fn pet(&mut self) -> GameResult {
        if !self.pet_ready() {
            return Err("Gib dem Nilpferd einen Moment Ruhe.".into());
        }
        self.pet_cooldown = 7.;
        self.care_interactions = self.care_interactions.saturating_add(1);
        self.walkers[0].pause = self.walkers[0].pause.max(0.85);
        self.restore_energy(5.);
        Ok("Das Nilpferd schnauft zufrieden.".into())
    }

    pub fn creature_status(&self) -> &'static str {
        if self.recovering {
            return "Erholt sich";
        }
        if self.walkers[0].bundle.is_some() {
            return "Bringt eine Ladung heim";
        }
        if let Some(circle) = &self.gatherings.circle {
            return if circle.present(self).1 {
                "Wartet am Festkreis"
            } else {
                "Geht zum Festkreis"
            };
        }
        match self.walkers[0].job {
            Some(Job::Refill(_)) => "Sichert Vorräte",
            Some(Job::Gather(_) | Job::GatherArea(..)) => "Sammelt",
            Some(Job::Move(_)) => "Unterwegs",
            Some(Job::Visit(_)) => "Besucht ein Nachbardorf",
            Some(Job::Watch(_)) => "Hält Wache",
            Some(Job::Tend(_)) => "Pflegt ein Feld",
            None => "Rastet",
        }
    }

    pub fn supply_help_active(&self) -> bool {
        matches!(self.walkers[0].job, Some(Job::Refill(_)))
    }

    pub fn supply_lesson(&self, region: &Region) -> Option<Material> {
        self.lesson.filter(|_| !region.tutorial)?.material(region)
    }

    pub fn automatic_supply(&self, region: &Region) -> Option<Material> {
        match self.walkers[0].job {
            Some(Job::Refill(i)) => Some(region.sources[i].material),
            _ => None,
        }
    }

    pub fn looking_at_store(&self) -> bool {
        let hippo = &self.walkers[0];
        self.supply_help_active()
            && !self.recovering
            && hippo.route.is_empty()
            && hippo.bundle.is_none()
            && hippo.pause > 0.
    }

    fn threat_in(&self, region: &Region, tile: Tile, radius: i16) -> bool {
        (self.quest == Quest::Bridge
            && self.count(Building::Shrine) > 0
            && (self.guards.iter().enumerate().any(|(index, &spirit)| {
                spirit > 0 && Tile::from_world(self.guard_world(region, index)).within(tile, radius)
            }) || (self.rival_active(region)
                && Tile::from_world(self.rival_world(region)).within(tile, radius))))
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
            _ if self.rival_waiting() && spell.offensive() => {
                Some("Verfügbar, wenn du die Rivalität beginnst.")
            }
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
            if spell.offensive()
                && self.rival_active(region)
                && Tile::from_world(self.rival_world(region)).within(tile, 1)
            {
                return Some("Ziele vom Ufer aus auf die Kreatur oder warte, bis sie sich nähert.");
            }
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
                Some("Ziele auf eine Wache, die Gegnerkreatur oder eine widerständige Siedlung.")
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
                let creature_hit = self.rival_active(region)
                    && Tile::from_world(self.rival_world(region)).within(tile, radius);
                if creature_hit {
                    self.rival_resolve = self.rival_resolve.saturating_sub(force);
                }
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
                if creature_hit {
                    let name = Rival::for_biome(region.kind).creature;
                    Ok(if self.rival_resolve == 0 {
                        format!("{} weicht zum Schrein zurück.", name)
                    } else {
                        format!(
                            "{} · Widerstand {}/{}",
                            name, self.rival_resolve, RIVAL_RESOLVE
                        )
                    })
                } else {
                    Ok(format!("{} · der Widerstand sinkt", spell.name()))
                }
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

    pub fn next_production(&self) -> u16 {
        self.provisions.ceil().max(0.) as u16
    }

    pub fn departure_supplies(&self) -> Stock {
        let mut supplies = [0; 3];
        if self.quest == Quest::Complete {
            for district in &self.districts {
                match district.outcome {
                    Some(Outcome::Alliance) => supplies[Material::Food.index()] += 4,
                    Some(Outcome::Victory) => supplies[Material::Stone.index()] += 4,
                    None => {}
                }
            }
        }
        supplies
    }

    pub fn crossing_due(&self, region: &Region) -> Stock {
        if self.quest != Quest::Bridge || self.gift_given || self.count(Building::Shrine) == 0 {
            return [0; 3];
        }
        let mut price = region.rebuild;
        if self.bridge_fixed {
            price = [0; 3];
            price[region.gift.0.index()] = region.gift.1;
        }
        std::array::from_fn(|i| price[i].saturating_sub(self.crossing_delivery[i]))
    }

    fn deliver_to_crossing(&mut self, region: &Region, bundle: Bundle) -> GameResult {
        let material = region.sources[bundle.source].material;
        let index = material.index();
        let accepted = bundle.count.min(self.crossing_due(region)[index]);
        self.crossing_delivery[index] += accepted;
        self.held = (bundle.count > accepted).then_some(Bundle {
            count: bundle.count - accepted,
            ..bundle
        });
        let due = self.crossing_due(region);
        if due == [0; 3] {
            return if self.bridge_fixed {
                self.give(region)
            } else {
                self.repair(region)
            };
        }
        Ok(format!(
            "{} {} angekommen · Offen: {}",
            accepted,
            region.kind.material(material),
            stock_text(due)
        ))
    }

    fn repair(&mut self, region: &Region) -> GameResult {
        self.bridge_ready()?;
        if self.bridge_fixed {
            return Err("Der Übergang wurde bereits gebaut.".into());
        }
        let due = self.crossing_due(region);
        if !self.affordable(due) {
            return Err(format!(
                "Für den Übergang fehlen Baustoffe. Benötigt werden {}.",
                stock_text(due)
            ));
        }
        self.pay(due);
        self.crossing_delivery = [0; 3];
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
        let material = region.gift.0;
        let price = self.crossing_due(region);
        let amount = price[material.index()];
        if !self.affordable(price) {
            return Err(format!(
                "Das Dorf bittet um {amount} {}.",
                region.kind.material(material)
            ));
        }
        self.pay(price);
        self.crossing_delivery = [0; 3];
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
        if region.tutorial {
            if self.quest != Quest::Complete
                && crate::practice::Step::current(self) == crate::practice::Step::Complete
            {
                self.quest = Quest::Complete;
                return true;
            }
            return false;
        }
        if self.contest.pact && self.quest != Quest::Complete {
            if self.village_start.is_some() {
                self.village_start = Some(VillageStart::Open);
            }
            self.quest = Quest::Complete;
            self.faith = self.faith.saturating_add(12);
            return true;
        }
        if self.rival_waiting() {
            if self.quest == Quest::Shelter && self.village_milestones() == 4 {
                self.quest = Quest::Fields;
                self.faith = self.faith.saturating_add(4);
                return true;
            }
            return false;
        }
        match self.quest {
            Quest::Shelter
                if !self.waiting_for_first_delivery()
                    && self.hand_deliveries > 0
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
                let outcome = if self.guards.iter().all(|&spirit| spirit == 0)
                    && !self.rival_active(region)
                {
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

    pub fn note(&mut self, entry: String) {
        if self.journal.front() == Some(&entry) {
            return;
        }
        self.journal.truncate(4);
        self.journal.push_front(entry);
    }

    pub fn next_task(&self, region: &Region) -> String {
        if self.quest == Quest::Districts
            && self
                .quest_step(region)
                .and_then(|s| s.pending(self, region))
                .is_none()
            && let Some((i, (d, _))) = region
                .districts
                .iter()
                .zip(&self.districts)
                .enumerate()
                .find(|(_, (_, s))| s.outcome.is_none())
        {
            if self.district_plan(region, i).is_ok() {
                return format!("{} aufnehmen", d.name);
            }
            if self.gatherings.welcomed[i]
                || self
                    .district_site(region, i)
                    .is_some_and(|s| d.need != DistrictNeed::Shelter || s.upgraded)
            {
                return format!("Gabe für {}", d.name);
            }
            return format!("{} für {}", Self::district_need(d.need), d.name);
        }
        if self.quest == Quest::Shelter
            && !self.waiting_for_first_delivery()
            && self.held.is_none()
            && self.count(Building::House) > 0
            && self.count(Building::Store) > 0
            && self.hippo_deliveries == 0
            && matches!(
                self.walkers[0].job,
                Some(Job::Gather(_) | Job::GatherArea(..))
            )
        {
            return "Dein Nilpferd liefert".into();
        }
        if let Some(goal) = self
            .quest_step(region)
            .and_then(|s| s.pending(self, region))
        {
            return match goal {
                Goal::Building {
                    kind,
                    upgraded,
                    roadside,
                    ..
                } => {
                    let verb = if upgraded {
                        "ausbauen"
                    } else if roadside {
                        "am Weg bauen"
                    } else {
                        "bauen"
                    };
                    format!("{} {verb}", kind.name())
                }
                Goal::Reserve { material, count } => format!(
                    "Noch {} {} einlagern",
                    count.saturating_sub(self.stock[material.index()]),
                    region.kind.material(material)
                ),
                Goal::Roads { count } => format!("Weg anlegen · {}/{count}", self.roads.len()),
                _ => {
                    let (have, need) = goal.amount(self, region);
                    format!("{} · {have}/{need}", goal.prompt())
                }
            };
        }
        if self.quest == Quest::Bridge && self.gift_given && self.renown() < region.renown {
            return format!("Noch {} Ansehen gewinnen", region.renown - self.renown());
        }
        self.objective(region).0.into()
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
        if region.tutorial {
            return crate::practice::Step::current(self).text().0;
        }
        if self.waiting_for_first_delivery() {
            return "Die erste Holzlieferung";
        }
        if let Some(step) = self.quest_step(region) {
            &step.title
        } else if let (Quest::Bridge, Some(quests)) = (self.quest, region.quests) {
            &quests.request
        } else {
            self.quest.name()
        }
    }

    pub fn missing_for(&self, kind: Building) -> Option<Material> {
        Material::ALL.into_iter().find(|material| {
            let i = material.index();
            self.stock[i] < kind.price()[i]
        })
    }

    pub fn campaign_hint(&self, region: &Region, pad: bool) -> String {
        let point = if pad {
            "L3 zeigt das Ziel"
        } else {
            "„Zum Ziel“ antippen"
        };
        if self.first_delivery_ready(region) {
            return "Wähle Mira oder Jora und liefere das Holz.".into();
        }
        if self.quest == Quest::Shelter {
            let next = if self.count(Building::House) == 0 {
                Building::House
            } else {
                Building::Store
            };
            if self.held.is_none()
                && self.hand_deliveries > 0
                && self.count(next) == 0
                && let Some(material) = self.missing_for(next)
            {
                return format!("Erst {} holen · {point}", region.kind.material(material));
            }
            return match (
                self.held.is_some(),
                self.hand_deliveries,
                self.count(Building::House),
                self.count(Building::Store),
            ) {
                (true, _, _, _) => format!("Zum Sammelplatz · {point}"),
                (_, 0, _, _) if pad => "Am Holzstoß A halten und loslassen.".into(),
                (_, 0, _, _) => "Am Holzstoß unten „Greifen“ halten.".into(),
                (_, _, 0, _) if pad => "X → Hütte wählen · A baut".into(),
                (_, _, 0, _) => "Bauen → Hütte → Platzieren".into(),
                (_, _, _, 0) if pad => "X → Lager wählen · A baut".into(),
                (_, _, _, 0) => "Bauen → Lager → Platzieren".into(),
                _ => format!("Nilpferd → Sammeln · {point}"),
            };
        }
        if let Some(goal) = self
            .quest_step(region)
            .and_then(|step| step.pending(self, region))
        {
            let progress = goal.progress(self, region);
            let verb = match goal {
                crate::quests::Goal::Harvests { .. } if self.count(Building::Field) == 0 => {
                    "Feld bauen"
                }
                crate::quests::Goal::Harvests { .. }
                    if self.buildings.iter().any(|field| {
                        matches!(field.crop, Crop::Ripe) && field.kind == Building::Field
                    }) =>
                {
                    "Mit der Pfote ernten"
                }
                crate::quests::Goal::Harvests { .. }
                    if self.buildings.iter().any(|field| {
                        matches!(field.crop, Crop::Growing(_)) && field.kind == Building::Field
                    }) =>
                {
                    "Saat reift"
                }
                crate::quests::Goal::Harvests { .. } => "Regen aufs Feld",
                crate::quests::Goal::HippoDeliveries { .. } => "Nilpferd → Sammeln",
                crate::quests::Goal::Reserve { .. } if self.held.is_some() => "Bündel ablegen",
                crate::quests::Goal::Reserve { .. } => "Bündel greifen",
                crate::quests::Goal::OlivePressings { .. }
                    if self.service(region, Civic::OlivePress) && self.stock[2] >= 10 =>
                {
                    return format!(
                        "{progress} · Nächste Verarbeitung in {} s",
                        self.next_production()
                    );
                }
                _ => return progress,
            };
            return format!("{progress} · {verb}");
        }
        match self.quest {
            Quest::Fields if self.count(Building::Field) == 0 => "Bauen → Feld".into(),
            Quest::Fields => "Trockenes Feld beregnen, reife Saat ernten.".into(),
            Quest::Bridge if self.count(Building::Shrine) == 0 => {
                format!("Grenzschrein errichten · {point}")
            }
            Quest::Bridge if self.gift_given => format!(
                "Ansehen {}/{} · Dorf ausbauen",
                self.renown(),
                region.renown
            ),
            Quest::Bridge => format!(
                "{} · {}",
                if self.bridge_fixed {
                    "Dorfgabe"
                } else {
                    "Brückenbau"
                },
                stock_text(self.crossing_due(region))
            ),
            Quest::Districts => format!("Nachbardörfer versorgen · {point}"),
            Quest::Complete => {
                "Handeln führt zu den Waren. Im Reisebuch siehst du deine Inseln.".into()
            }
            _ => point.into(),
        }
    }

    pub fn objective(&self, region: &Region) -> (&'static str, String) {
        if region.tutorial {
            let (title, hint) = crate::practice::Step::current(self).text();
            return (title, hint.into());
        }
        if self.waiting_for_first_delivery() {
            return if self.first_delivery_ready(region) {
                ("Erste Lieferung wählen", "Miras Familie am Schrein und Joras Hof am anderen Ufer brauchen beide Holz. Zum Ziel zeigt dir ihre Bitten.".into())
            } else if self.held.is_some() {
                (
                    "Ladung ablegen",
                    format!(
                        "Lege deine Ladung ab. Für die erste Lieferung brauchst du {PAW_BUNDLE} Holz in der Pfote."
                    ),
                )
            } else {
                (
                    "Holz für beide Ufer",
                    format!(
                        "Mira und Jora bitten um Holz. Greife mindestens {PAW_BUNDLE} Holz für die erste Lieferung."
                    ),
                )
            };
        }
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
            Quest::Shelter if self.held.is_some() => (
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
            Quest::Bridge if self.gift_given => (
                "Vertrauen gewinnen",
                format!(
                    "Die Gabe ist angekommen. Ansehen {}/{} · Ernten, neue Gebäude und Ausbauten überzeugen die Nachbarn.",
                    self.renown(),
                    region.renown
                ),
            ),
            Quest::Bridge => (
                if self.bridge_fixed {
                    "Die Dorfgabe"
                } else {
                    "Den Übergang bauen"
                },
                format!(
                    "Noch {}. Bring passende Bündel direkt zum Übergang oder bezahle dort aus dem Lager. Du kannst die Wachen auch mit Wind vertreiben.",
                    stock_text(self.crossing_due(region))
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
        if region.sandbox || region.tutorial || self.local_finished {
            SideQuest::Complete
        } else if !self.local_accepted {
            SideQuest::Available
        } else if self.local_quest(region).2 {
            SideQuest::Ready
        } else {
            SideQuest::Active
        }
    }

    fn local_requirements(&self, region: &Region) -> (&'static str, [Goal; 2]) {
        let build = |kind, count, roadside, upgraded| Goal::Building {
            kind,
            count,
            roadside,
            upgraded,
        };
        if region.chapter() > 0 {
            let name = if region.chapter() == 1 {
                "Das Handwerk der Region"
            } else {
                "Das Erbe des Landes"
            };
            return (
                name,
                [
                    build(
                        Building::Civic(Civic::regional(region.kind)),
                        1,
                        true,
                        false,
                    ),
                    Goal::Camps {
                        count: region.chapter() as u16,
                    },
                ],
            );
        }
        match region.kind {
            Biome::Valley => (
                "Der Weidengarten",
                [
                    build(Building::Well, 1, true, false),
                    Goal::Harvests { count: 3 },
                ],
            ),
            Biome::Fjord => (
                "Für den ersten Frost",
                [
                    build(Building::Granary, 1, true, false),
                    Goal::Reserve {
                        material: Material::Wood,
                        count: 20,
                    },
                ],
            ),
            Biome::Terraces => (
                "Saatgut aus den Terrassen",
                [
                    build(Building::Field, 2, false, true),
                    Goal::Harvests { count: 4 },
                ],
            ),
            Biome::Lake => (
                "Vorrat für die Trockenzeit",
                [
                    build(Building::Well, 2, true, false),
                    Goal::Reserve {
                        material: Material::Food,
                        count: 23,
                    },
                ],
            ),
            Biome::Aegean => (
                "Fest der Winde",
                [
                    build(Building::Civic(Civic::Theater), 1, true, false),
                    build(Building::Civic(Civic::Oracle), 1, true, false),
                ],
            ),
        }
    }

    pub fn local_next(&self, region: &Region) -> Option<Goal> {
        if let Some(quests) = region.quests {
            quests.side.pending(self, region)
        } else {
            self.local_requirements(region)
                .1
                .into_iter()
                .find(|goal| !goal.met(self, region))
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
        let (title, goals) = self.local_requirements(region);
        (
            title,
            goals.map(|goal| goal.progress(self, region)).join(" · "),
            goals.iter().all(|goal| goal.met(self, region)),
        )
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
    practice_finished: bool,
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
            intro_seen: sandbox || regions[0].tutorial,
            practice_finished: false,
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
            .map_or(WorldLayout::Islands, |city| city.world_layout)
    }

    fn sync_creature_work(&mut self) {
        let mut total = [0_u32; 3];
        for city in self.settlements.iter().flatten() {
            for (sum, amount) in total.iter_mut().zip(city.completed_work()) {
                *sum = sum.saturating_add(u32::from(amount));
            }
        }
        self.gifts.creature.work = total;
    }

    fn restore(mut self, regions: &[Region]) -> Option<Self> {
        let add_village_start = (1..SAVE_VERSION).contains(&self.version);
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
        if self.version == 4 {
            for (city, region) in self.settlements.iter_mut().zip(regions) {
                if let Some(city) = city {
                    if city.nodes.len() != region.managed_sources
                        || city.regrowth.len() != region.managed_sources
                    {
                        return None;
                    }
                    city.nodes.extend(
                        region.sources[region.managed_sources..]
                            .iter()
                            .map(|s| s.stock),
                    );
                    city.regrowth.resize(region.sources.len(), 0.);
                }
            }
            self.version = SAVE_VERSION;
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
        if self.version == 5 {
            self.version = SAVE_VERSION;
        }
        for (entry, region) in self.settlements.iter_mut().zip(regions) {
            let Some(city) = entry else {
                continue;
            };
            city.populate_villages(region);
            if add_village_start {
                city.migrate_village_start(region);
            }
            city.journal.truncate(5);
        }
        self.sync_creature_work();
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
                            Job::Gather(source) | Job::Refill(source) => {
                                source < region.sources.len()
                            }

                            Job::GatherArea(area, _) => {
                                region.areas.get(area).is_some() && city.camps[area]
                            }
                            Job::Visit(i) => i < region.villages.len(),
                            Job::Move(tile) | Job::Watch(tile) | Job::Tend(tile) => {
                                region.bounds.contains(tile)
                            }
                        };
                        city.citizens.iter().enumerate().all(|(i, t)| {
                            !city.citizens[..i].contains(t)
                                && city.building_at(*t).is_some_and(|s| {
                                    s.tile == *t && s.kind == Building::House && s.upgraded
                                })
                        }) && city.valid_projects(region)
                            && city.valid_village_start(region)
                            && city.work.valid(region)
                            && city.contest.towns.len() == region.villages.len()
                            && city
                                .contest
                                .towns
                                .iter()
                                .all(|t| t.trust.is_finite() && t.trust.abs() <= 100.)
                            && city
                                .contest
                                .destination
                                .is_none_or(|i| i < region.villages.len())
                            && city
                                .contest
                                .rival_tile
                                .is_none_or(|t| region.bounds.contains(t))
                            && city.gatherings.rest.is_finite()
                            && city.gatherings.circle.as_ref().is_none_or(|c| {
                                region.bounds.contains(c.center)
                                    && (2..=4).contains(&c.guests.len())
                                    && c.guests[0].0 == 0
                                    && c.guests.iter().all(|(i, t)| {
                                        *i < city.walkers.len() && region.bounds.contains(*t)
                                    })
                            })
                            && city.nodes.len() == region.sources.len()
                            && city.regrowth.len() == region.sources.len()
                            && city.guards.len() == region.guards.len()
                            && (city.opening.is_none() || region.opening_farm().is_some())
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
    active_slot: bool,
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
            active_slot: true,
            regions,
            campaign,
        }
    }

    pub fn start(mode: Mode) -> Self {
        let mut game = Self::new(mode.regions(|_| WorldLayout::Islands), None);
        game.mode = mode;
        game
    }

    pub fn title_world() -> Self {
        let mut game = Self::start(Mode::Tutorial);
        game.active_slot = false;
        game
    }

    pub fn resume(mut saves: Saves) -> Option<Self> {
        if saves.profiles.is_empty() && saves.current == Mode::Tutorial {
            return Some(Self::title_world());
        }
        if saves.profiles.len() > Biome::ALL.len() * 2 + 2
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
            active_slot: true,
            regions,
            campaign,
        })
    }

    pub fn saved_games(&self) -> Saves {
        let mut profiles = self.archives.clone();
        if self.active_slot {
            profiles.push(Profile {
                mode: self.mode,
                campaign: self.campaign.clone(),
            });
        }
        Saves {
            current: self.mode,
            profiles,
        }
    }

    pub fn slots(&self) -> impl Iterator<Item = (Mode, &Campaign)> {
        self.active_slot
            .then_some((self.mode, &self.campaign))
            .into_iter()
            .chain(self.archives.iter().rev().map(|p| (p.mode, &p.campaign)))
    }

    pub fn delete_slot(
        &mut self,
        mode: Mode,
        persist: impl FnOnce(&Saves) -> std::io::Result<()>,
    ) -> Result<(), String> {
        let mut remaining = self.saved_games();
        let index = remaining
            .profiles
            .iter()
            .position(|p| p.mode == mode)
            .ok_or("Diesen Spielstand gibt es nicht mehr.")?;
        remaining.profiles.remove(index);
        let replacement = if self.mode == mode {
            // Keep the last playable profile selected when the current one is removed.
            let next = remaining
                .profiles
                .iter()
                .rev()
                .find(|p| p.mode != Mode::Tutorial)
                .or_else(|| remaining.profiles.last());
            remaining.current = next.map_or(Mode::Tutorial, |p| p.mode);
            Some(
                Self::resume(remaining.clone())
                    .ok_or("Der nächste Spielstand konnte nicht geöffnet werden.")?,
            )
        } else {
            None
        };
        persist(&remaining).map_err(|error| {
            warn!("Spielstand konnte nicht gelöscht werden: {error}");
            "Speichern fehlgeschlagen. Dein Spielstand ist noch da.".to_string()
        })?;
        if let Some(next) = replacement {
            *self = next;
        } else {
            self.archives.retain(|p| p.mode != mode);
        }
        Ok(())
    }

    pub fn has_save(&self, mode: Mode) -> bool {
        (self.active_slot && mode == self.mode && self.campaign.intro_seen)
            || self.archives.iter().any(|profile| profile.mode == mode)
    }

    pub fn continue_mode(&self) -> Option<Mode> {
        if self.mode == Mode::Tutorial {
            self.archives.last().map(|profile| profile.mode)
        } else {
            Some(self.mode)
        }
    }

    pub fn practice_city(&self) -> Option<&Settlement> {
        if self.mode == Mode::Tutorial {
            Some(self.city())
        } else {
            self.archives
                .iter()
                .find(|profile| profile.mode == Mode::Tutorial)
                .and_then(|profile| profile.campaign.settlements.first()?.as_ref())
        }
    }

    pub fn replay_finished_practice(&mut self) {
        if self.mode == Mode::Tutorial && self.campaign_complete() {
            self.campaign = Campaign::new(&self.regions);
            self.campaign.practice_finished = true;
        }
    }

    pub fn switch_mode(&mut self, mode: Mode) -> GameResult {
        if self.mode == mode {
            self.active_slot = true;
            return Ok(mode.name().into());
        }
        if self.practice_required() {
            return Err("Beende zuerst die kurze Übung auf der Insel.".into());
        }
        let entry = self
            .archives
            .iter()
            .position(|profile| profile.mode == mode);
        let archived = entry.map(|index| &self.archives[index].campaign);
        let regions = mode.regions(|chapter| {
            archived.map_or(WorldLayout::Islands, |save| save.world_layout(chapter))
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

    pub fn blueprint_readiness(&self, kind: Building) -> Result<(), String> {
        if let Some(civic) = kind.civic() {
            self.campaign.research.permits(civic)?;
        }
        let city = self.city();
        let region = self.region();
        city.blueprint_requirements(region, kind)?;
        if region.sandbox {
            return Ok(());
        }
        city.material_requirements(kind.price(), kind.goods_price())
    }

    pub fn act(&mut self, action: Action) -> GameResult {
        let index = self.campaign.active;
        let settlement = self.campaign.settlements[index].as_mut().unwrap();
        let region = &self.regions[index];
        let gifts = &mut self.campaign.gifts;
        let research = &mut self.campaign.research;
        let gift_before = settlement.gift_given;
        let mut text = match action {
            Action::Hand(target) => settlement.hand(region, target),
            Action::Grab(tile, amount) => settlement.grab(region, tile, amount),
            Action::Build(kind, tile, rotation) => {
                if let Some(kind) = kind.civic() {
                    research.permits(kind)?;
                }
                settlement.build(region, kind, tile, rotation)
            }
            Action::Road(start, end) => settlement.build_road(region, start, end),
            Action::Upgrade(tile) => settlement.upgrade(region, tile),
            Action::SeeWork => return Ok(Blueprint::on(region).title.into()),
            Action::AcceptWork => settlement.accept_work(region),
            Action::SupplyWork => settlement.supply_work(region),
            Action::SummonBuilder => settlement.summon_builder(region),
            Action::RaiseWork => settlement.raise_work(region, gifts),
            Action::Dedicate(choice) => settlement.dedicate_work(region, gifts, choice),
            Action::TrackLocal(on) => {
                settlement.local_tracked = on
                    && matches!(
                        settlement.side_quest(region),
                        SideQuest::Active | SideQuest::Ready
                    );
                if settlement.local_tracked {
                    settlement.work.pinned = false;
                }
                Ok(if settlement.local_tracked {
                    "Nebenauftrag im Tracker"
                } else {
                    "Inselauftrag im Tracker"
                }
                .into())
            }
            Action::TrackWork(on) => {
                if on {
                    settlement.local_tracked = false;
                }
                settlement.work.pinned = on
                    && matches!(
                        settlement.work.stage,
                        WorkStage::Supplies | WorkStage::Raising | WorkStage::Opening
                    );
                Ok(if settlement.work.pinned {
                    "Inselwerk im Tracker"
                } else {
                    "Inselauftrag im Tracker"
                }
                .into())
            }
            Action::Charter(tile) => settlement.charter(region, tile),
            Action::Research(tech) => {
                research.learn(tech, settlement.city_rank())?;
                Ok(format!("{} erforscht · Neue Pläne im Baubuch", tech.name()))
            }
            Action::Muster(tile) => settlement.muster(region, tile),
            Action::Patrol(tile) => settlement.patrol(region, tile),
            Action::Audience(i) => {
                return region
                    .villages
                    .get(i)
                    .map(|v| v.name.clone())
                    .ok_or_else(|| "Unbekanntes Dorf.".into());
            }
            Action::MeetNeighbor(i) => settlement.meet_neighbor(region, i),
            Action::BeginRivalry => settlement.begin_rivalry(region),
            Action::DeferAppeal(i) => settlement.defer_appeal(i),
            Action::Visit(i, hippo) => settlement.send_visitor(region, i, hippo),
            Action::SendPilgrims(i) => settlement.procession(region, i),
            Action::Aid(i, h) => settlement.aid(region, i, h),
            Action::Withdraw(i) => settlement.withdraw(region, i),
            Action::VillageGift(i) => settlement.village_gift(region, i),
            Action::IslandPact => {
                if region.tutorial
                    || region.sandbox
                    || settlement.quest == Quest::Complete
                    || settlement.contest.count(Allegiance::Ours) < settlement.contest.majority()
                {
                    return Err("Gewinne die Mehrheit der Dörfer für den Inselbund.".into());
                }
                settlement.contest.pact = true;
                settlement.contest.destination = None;
                settlement.bridge_fixed = true;
                settlement.outcome = Some(Outcome::Alliance);
                settlement.guards.fill(0);
                Ok("Der Inselbund ist besiegelt. Die Dörfer öffnen die Überfahrt.".into())
            }
            Action::Trade(index, tribute) => settlement.district_trade(region, index, tribute),
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
                let (title, _, _) = settlement.local_quest(region);
                Ok(format!("Auftrag angenommen · {title}"))
            }
            Action::FinishLocal => {
                if settlement.side_quest(region) != SideQuest::Ready {
                    return Err("Erfülle zuerst die Ziele des Nebenauftrags.".into());
                }
                settlement.local_finished = true;
                settlement.local_tracked = false;
                settlement.faith = settlement.faith.saturating_add(6);
                gifts.earn();
                research.earn(2);
                Ok("Nebenauftrag erfüllt · +6 Glauben · +2 Erkenntnis · +1 Talentpunkt".into())
            }
            Action::RecallHelpers => settlement.recall_helpers(),
            Action::MoveHippo(tile) => settlement.move_hippo(region, tile),
            Action::Watch(tile) => settlement.watch(region, tile, *gifts),
            Action::Tend(tile) => settlement.tend(region, tile, *gifts),
            Action::Call(kind, near) => settlement.gather(region, kind, near),
            Action::DismissCircle => settlement.dismiss_circle(),
            Action::Invoke(awe) => settlement.finish_circle(region, awe),
            Action::Produce(tile) => settlement.produce(region, tile, true),
            Action::ShareMeal(tile) => {
                settlement.meal_ready(region, tile)?;
                settlement.stock[2] -= 3;
                settlement.faith = settlement.faith.saturating_add(2);
                settlement.society.morale = (settlement.society.morale + 8).min(100);
                settlement.society.meal_day = Some((settlement.weather_time / DAY_LENGTH) as u32);
                Ok("Das Dorf teilt deine Gabe. Heute steigt die Stimmung um 8.".into())
            }
            Action::Pet => settlement.pet(),
            Action::Feed => settlement.feed(),
            Action::Rest => {
                let called = settlement.gatherings.circle.is_some();
                if called {
                    settlement.dismiss_circle()?;
                }
                settlement.supply_help_paused = true;
                let hippo = &mut settlement.walkers[0];
                hippo.job = None;
                hippo.route.clear();
                hippo.pause = 0.;
                Ok(if called {
                    "Dorfruf beendet · Das Nilpferd ruht."
                } else {
                    "Das Nilpferd legt eine Pause ein."
                }
                .into())
            }
            Action::Cast(spell, tile) => settlement.cast(region, spell, tile, *gifts),
            Action::Repair => settlement.repair(region),
            Action::Give => settlement.give(region),
            Action::Praise => {
                let lesson = settlement
                    .lesson
                    .take()
                    .ok_or("Lobe die Kreatur nach einer Lieferung, Wache oder Ernte.")?;
                let material = (!region.tutorial)
                    .then(|| lesson.material(region))
                    .flatten();
                if gifts.training < 5 {
                    gifts.coaching += 1;
                    if gifts.coaching >= gifts.training.saturating_sub(1).max(1) {
                        gifts.training += 1;
                        gifts.coaching = 0;
                    }
                }
                let hippo = &mut settlement.walkers[0];
                if let Some(material) = material {
                    if hippo.job == Some(lesson.job()) && !gifts.supplies(material) {
                        hippo.job = None;
                        hippo.route.clear();
                    }
                    gifts.teach_supply(material);
                    settlement.supply_help_paused = false;
                } else if hippo.job.is_none() {
                    hippo.job = Some(lesson.job());
                }
                Ok(if let Some(material) = material {
                    format!(
                        "Gelernt · Bei höchstens {LOW_SUPPLY} {} holt dein Nilpferd selbst Nachschub.",
                        region.kind.material(material)
                    )
                } else {
                    format!("Lob angenommen · Nilpferd-Rang {}", gifts.training)
                })
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
                let supplies = next
                    .checked_sub(1)
                    .and_then(|previous| self.campaign.settlements[previous].as_ref())
                    .map_or([0; 3], Settlement::departure_supplies);
                self.campaign.active = next;
                if self.campaign.settlements[next].is_none() {
                    let mut landing = Settlement::new(&self.regions[next]);
                    let cap = landing.storage_cap();
                    for (stock, bonus) in landing.stock.iter_mut().zip(supplies) {
                        *stock = stock.saturating_add(bonus).min(cap);
                    }
                    self.campaign.settlements[next] = Some(landing);
                    if supplies != [0; 3] {
                        return Ok(format!(
                            "Ankunft in {} · Mitgebracht: {}",
                            self.regions[next].name,
                            stock_text(supplies)
                        ));
                    }
                }
                return Ok(self.regions[next].subtitle.clone());
            }
        }?;
        let (city_change, creature_change) = match action {
            Action::Trade(_, true) => (-4, 0),
            Action::Aid(_, true) => (3, 5),
            Action::Aid(_, false) => (4, 0),
            Action::Invoke(false) => (3, 3),
            Action::Invoke(true) => (-6, -5),
            Action::Pet => (0, 3),
            Action::ShareMeal(_) => (3, 1),
            Action::Praise | Action::Feed => (0, 2),
            _ if !gift_before && settlement.gift_given => (8, 5),
            Action::Settle(district)
                if settlement.districts[district].outcome == Some(Outcome::Alliance) =>
            {
                (12, 6)
            }
            Action::Settle(_) => (-12, -7),
            Action::Build(Building::Well, ..) | Action::FinishLocal => (3, 0),
            Action::Cast(Miracle::Wind | Miracle::Thunder, _) => (-4, -3),
            Action::Cast(Miracle::Rain | Miracle::Bloom | Miracle::Ward, _) => (2, 1),
            Action::Cast(Miracle::Mend, _) => (0, 3),
            _ => (0, 0),
        };
        settlement.reputation.shift(city_change);
        gifts.creature.nature.shift(creature_change);
        if settlement.gatherings.circle.is_some()
            && matches!(
                action,
                Action::Assign(_, true)
                    | Action::AssignArea(_, _, true)
                    | Action::MoveHippo(_)
                    | Action::Watch(_)
                    | Action::Tend(_)
            )
        {
            text.push_str(" · Dein Nilpferd übernimmt nach dem Dorfruf.");
        }
        if matches!(action, Action::Invoke(_) | Action::Aid(_, true)) {
            self.campaign.sync_creature_work();
        }
        if !matches!(
            action,
            Action::Grab(..) | Action::MoveHippo(_) | Action::TrackWork(_)
        ) && !matches!(action, Action::Hand(_) if self.city().held.is_some())
        {
            self.campaign.settlements[index]
                .as_mut()
                .unwrap()
                .note(text.clone());
        }
        if self.advance_quests() {
            text.push_str(" · Auftrag erfüllt");
        }
        Ok(text)
    }

    pub fn step(&mut self, dt: f32) {
        let index = self.campaign.active;
        let city = self.campaign.settlements[index].as_mut().unwrap();
        let region = &self.regions[index];
        let previous = city.completed_work();
        let insight = city.tick(region, self.campaign.gifts, dt);
        if city.completed_work() != previous {
            self.campaign.sync_creature_work();
        }
        self.campaign.research.earn(insight);
        self.advance_quests();
    }

    fn advance_quests(&mut self) -> bool {
        let index = self.campaign.active;
        let city = self.campaign.settlements[index].as_mut().unwrap();
        let region = &self.regions[index];
        let previous = city.outcome;
        let main = city.advance_quest(region);
        if main && !region.tutorial {
            city.note(format!("Neues Ziel · {}", city.quest_title(region)));
        }
        if main && region.tutorial {
            self.campaign.practice_finished = true;
        }
        if let Some(outcome) = city.outcome.filter(|_| previous.is_none()) {
            let shift = if outcome == Outcome::Alliance {
                16
            } else {
                -18
            };
            city.reputation.shift(shift);
            self.campaign.gifts.creature.nature.shift(shift / 2);
        }
        if main && !region.tutorial {
            self.campaign.gifts.earn();
            self.campaign.research.earn(2);
        }
        if city.quest == Quest::Complete
            && (city.contest.pact || city.districts.iter().all(|state| state.outcome.is_some()))
        {
            self.campaign.unlocked = self
                .campaign
                .unlocked
                .max((index + 1).min(self.regions.len() - 1));
        }
        main
    }

    pub fn campaign_complete(&self) -> bool {
        if self.mode == Mode::Tutorial {
            return self.city().quest == Quest::Complete;
        }
        self.campaign.settlements.iter().all(|city| {
            city.as_ref()
                .is_some_and(|city| city.quest == Quest::Complete)
        })
    }

    pub fn practice_required(&self) -> bool {
        self.mode == Mode::Tutorial
            && !self.campaign.practice_finished
            && !self.campaign_complete()
            && self.archives.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::{CameraRig, Panel, Play, Tool, UiAction};

    fn fresh() -> Game {
        Game::new(Region::campaign(), None)
    }

    fn legacy_campaign(biome: Biome) -> Game {
        let mut game = Game::start(Mode::Campaign(biome));
        game.campaign.settlements[0] = Some(Settlement::with_village_start(game.region(), false));
        game
    }

    #[test]
    fn deleting_a_selected_profile_commits_only_after_a_successful_write() {
        let first = Mode::Campaign(Biome::Valley);
        let second = Mode::Sandbox(Biome::Valley);
        let mut game = Game::start(first);
        game.campaign.intro_seen = true;
        game.switch_mode(second).unwrap();
        game.switch_mode(Mode::Tutorial).unwrap();
        let original = serde_json::to_value(game.saved_games()).unwrap();
        let error = game.delete_slot(second, |_| Err(std::io::Error::other("full")));
        assert!(error.is_err());
        assert_eq!(serde_json::to_value(game.saved_games()).unwrap(), original);
        game.delete_slot(first, |_| Ok(())).unwrap();
        assert_eq!(game.mode, Mode::Tutorial);
        assert!(!game.has_save(first));
        game.delete_slot(Mode::Tutorial, |saves| {
            assert_eq!(saves.current, second);
            let sandbox = original["profiles"]
                .as_array()
                .unwrap()
                .iter()
                .find(|p| p["mode"] == serde_json::to_value(second).unwrap())
                .unwrap();
            assert_eq!(&serde_json::to_value(&saves.profiles[0]).unwrap(), sandbox);
            Ok(())
        })
        .unwrap();
        assert_eq!(game.continue_mode(), Some(second));
        game.delete_slot(second, |saves| {
            assert!(saves.profiles.is_empty());
            Ok(())
        })
        .unwrap();
        let mut reopened = Game::resume(game.saved_games()).unwrap();
        assert_eq!(reopened.slots().count(), 0);
        assert_eq!(reopened.continue_mode(), None);
        assert!(reopened.practice_required());
        assert!(!reopened.has_save(Mode::Tutorial));
        reopened.switch_mode(Mode::Tutorial).unwrap();
        assert_eq!(reopened.slots().count(), 1);
    }

    #[test]
    fn deleting_from_the_title_requires_the_selected_named_confirmation() {
        let mut game = Game::start(Mode::Sandbox(Biome::Fjord));
        let mut play = Play::new(&game);
        let mut view = CameraRig::new(&game);
        play.show_main_menu(&game);
        let slot = game.mode;
        play.activate(UiAction::DeleteSave(slot), &mut game, &mut view, 0.);
        assert!(play.pending_delete.is_none());
        play.activate(
            UiAction::Panel(Panel::SavedGames(0)),
            &mut game,
            &mut view,
            1.,
        );
        let confirm = UiAction::Panel(Panel::ConfirmDelete(slot));
        play.set_focus(Some(confirm));
        play.activate(confirm, &mut game, &mut view, 2.);
        assert_eq!(play.focus, Some(UiAction::Cancel));
        play.activate(UiAction::Cancel, &mut game, &mut view, 3.);
        assert_eq!(play.panel, Panel::SavedGames(0));
        assert_eq!(play.focus, Some(confirm));
        assert!(play.pending_delete.is_none());
        play.activate(confirm, &mut game, &mut view, 4.);
        play.activate(
            UiAction::DeleteSave(Mode::Tutorial),
            &mut game,
            &mut view,
            5.,
        );
        assert!(play.pending_delete.is_none());
        play.activate(UiAction::DeleteSave(slot), &mut game, &mut view, 6.);
        assert_eq!(play.pending_delete.take(), Some(slot));
        let result = game.delete_slot(slot, |_| Ok(()));
        play.deletion_result(result, slot, &game, 7.);
        assert_eq!(play.panel, Panel::SavedGames(0));
        assert_eq!(play.focus, Some(UiAction::Cancel));
        play.activate(UiAction::Cancel, &mut game, &mut view, 8.);
        assert_eq!(play.panel, Panel::MainMenu);
        assert_eq!(game.continue_mode(), None);
    }

    #[test]
    fn island_works_keep_partial_deliveries_and_validate_old_saves() {
        let mut game = Game::start(Mode::Campaign(Biome::Valley));
        let mut legacy = serde_json::to_value(game.saved_games()).unwrap();
        let profile = &mut legacy["profiles"][0]["campaign"];
        profile["settlements"][0]
            .as_object_mut()
            .unwrap()
            .remove("work");
        profile["gifts"]["creature"]
            .as_object_mut()
            .unwrap()
            .remove("craft");
        game = Game::resume(serde_json::from_value(legacy).unwrap()).unwrap();
        assert_eq!(game.city().work.stage, WorkStage::Offered);
        assert_eq!(game.campaign.gifts.creature.craft, 0);
        assert!(game.act(Action::RaiseWork).is_err());
        game.act(Action::AcceptWork).unwrap();
        let i = Material::Wood.index();
        game.campaign.settlements[0].as_mut().unwrap().stock[i] = 2;
        game.act(Action::SupplyWork).unwrap();
        assert!(game.act(Action::SupplyWork).is_err());
        game = Game::resume(game.saved_games()).unwrap();
        assert_eq!(
            (
                game.city().work.delivered,
                game.city().stock[i],
                game.city().work.pinned
            ),
            (2, 0, true)
        );
        game.act(Action::TrackWork(false)).unwrap();
        stock_up(&mut game, Material::Wood, 6);
        let before = game.city().stock[i];
        game.act(Action::SupplyWork).unwrap();
        assert_eq!(game.city().stock[i], before - 4);
        assert_eq!(game.city().work.stage, WorkStage::Raising);
        assert!(game.act(Action::SupplyWork).is_err());
        assert!(!game.city().work.pinned);
        let mut invalid = game.saved_games();
        invalid.profiles[0].campaign.settlements[0]
            .as_mut()
            .unwrap()
            .work
            .delivered = u16::MAX;
        assert!(Game::resume(invalid).is_none());
        let mut training = Game::start(Mode::Tutorial);
        assert!(training.act(Action::AcceptWork).is_err());
    }

    #[test]
    fn works_follow_each_chapter_and_each_regions_economy() {
        for kind in Biome::ALL {
            let regions = Mode::Campaign(kind).regions(|_| WorldLayout::Islands);
            assert_eq!(regions.len(), 3);
            for (chapter, r) in regions.iter().enumerate() {
                let b = Blueprint::on(r);
                assert_eq!(b.amount, 6 + 3 * chapter as u16);
                let mut city = Settlement::new(r);
                city.buildings.clear();
                let mut gifts = Gifts::default();
                city.accept_work(r).unwrap();
                city.stock[b.material.index()] = b.amount;
                city.supply_work(r).unwrap();
                city.walkers[0].position = r.depot.offset(4, 0).local().to_array();
                assert!(city.raise_work(r, &mut gifts).is_err());
                city.walkers[0].position = r.depot.offset(1, 0).local().to_array();
                city.energy = 11.;
                assert!(city.raise_work(r, &mut gifts).is_err());
                assert_eq!(city.energy, 11.);
                city.energy = 12.;
                city.raise_work(r, &mut gifts).unwrap();
                assert_eq!((city.energy, gifts.creature.craft), (0., 1));
                assert!(city.raise_work(r, &mut gifts).is_err());
                assert!(city.dedicate_work(r, &mut gifts, Dedication::Town).is_err());
                let site = r
                    .bounds
                    .tiles()
                    .filter(|&t| t.within(r.start, 9))
                    .map(|tile| BuildingSite {
                        kind: b.partner,
                        tile,
                        rotation: 0,
                        crop: Crop::Dry,
                        upgraded: false,
                        batch_finished: false,
                    })
                    .find(|site| city.connected(r, site))
                    .unwrap();
                city.buildings.push(site);
                assert!(city.work_partner(r));
                city.dedicate_work(r, &mut gifts, Dedication::Town).unwrap();
                assert!(
                    city.dedicate_work(r, &mut gifts, Dedication::Talent)
                        .is_err()
                );
                city.stock = [0; 3];
                city.mana = 0.;
                city.shield = 0.;
                city.rainfall = 0.;
                city.operate_work(r, gifts);
                match kind {
                    Biome::Valley => assert_eq!(city.stock[Material::Food.index()], 4),
                    Biome::Aegean => assert_eq!(city.stock[Material::Stone.index()], 4),
                    Biome::Fjord => assert_eq!(city.shield, 16.),
                    Biome::Terraces => assert_eq!(city.rainfall, 12.),
                    Biome::Lake => assert_eq!(city.mana, 8.),
                }
                assert!(city.work.valid(r));
                city.buildings.clear();
                city.operate_work(r, gifts);
                assert_eq!(city.work.rounds, 1);
            }
        }
    }

    #[test]
    fn work_guide_connects_an_existing_field_using_a_buildable_road() {
        let mut game = Game::start(Mode::Campaign(Biome::Valley));
        stock_for(&mut game, Building::Field.price());
        stock_up(&mut game, Material::Stone, 5);
        let r = game.region();
        let c = game.city();
        let tile = r
            .bounds
            .tiles()
            .filter(|&t| t.within(r.start, 7))
            .find(|&t| {
                game.building_plan(Building::Field, t).is_ok()
                    && !Building::Field
                        .tiles(t)
                        .flat_map(Tile::neighbors)
                        .any(|p| c.road_at(r, p))
            })
            .unwrap();
        game.act(Action::Build(Building::Field, tile, 0)).unwrap();
        assert!(!game.city().work_partner(game.region()));
        let (from, to, cost) = game
            .city()
            .work_connection(game.region(), Building::Field)
            .unwrap();
        let stone = game.city().stock[Material::Stone.index()];
        game.act(Action::Road(from, to)).unwrap();
        assert_eq!(
            game.city().stock[Material::Stone.index()],
            stone - cost[Material::Stone.index()]
        );
        assert!(game.city().work_partner(game.region()));
    }

    #[test]
    fn companion_walks_to_work_and_each_dedication_pays_only_once() {
        let mut game = Game::start(Mode::Campaign(Biome::Valley));
        game.act(Action::AcceptWork).unwrap();
        stock_up(&mut game, Material::Wood, 6);
        game.act(Action::SupplyWork).unwrap();
        game.act(Action::SummonBuilder).unwrap();
        for _ in 0..300 {
            if game.city().builder_present(game.region()) {
                break;
            }
            game.step(0.2);
        }
        assert!(game.city().builder_present(game.region()));
        game.act(Action::RaiseWork).unwrap();
        place_quest_building(&mut game, Building::Field, true);
        for choice in [Dedication::Companion, Dedication::Talent, Dedication::Town] {
            let mut run = Game::resume(game.saved_games()).unwrap();
            let points = run.campaign.gifts.points;
            let xp = run.campaign.gifts.creature.experience();
            run.act(Action::Dedicate(choice)).unwrap();
            assert!(!run.city().work.pinned);
            assert!(run.act(Action::Dedicate(choice)).is_err());
            run = Game::resume(run.saved_games()).unwrap();
            assert_eq!(
                run.campaign.gifts.creature.experience(),
                xp + if choice == Dedication::Companion {
                    4
                } else {
                    0
                }
            );
            assert_eq!(
                run.campaign.gifts.points,
                points + u16::from(choice == Dedication::Talent)
            );
            if choice == Dedication::Talent {
                run.act(Action::Learn(Talent::Rain)).unwrap();
                assert_eq!(run.campaign.gifts.rank(Talent::Rain), 1);
            }
        }
    }

    #[test]
    fn rival_creatures_have_specific_attacks_and_a_shared_ward_counter() {
        for biome in Biome::ALL {
            let game = Game::start(Mode::Campaign(biome));
            let region = game.region();
            let fixture = || {
                let mut city = game.city().clone();
                city.quest = Quest::Bridge;
                city.village_start = Some(VillageStart::Open);
                city.stock = [24, 22, 21];
                city.buildings.push(BuildingSite {
                    kind: Building::Shrine,
                    tile: region.frontier,
                    rotation: 0,
                    crop: Crop::Dry,
                    upgraded: false,
                    batch_finished: false,
                });
                city.buildings.push(BuildingSite {
                    kind: Building::Field,
                    tile: region.start,
                    rotation: 0,
                    crop: Crop::Growing(3.),
                    upgraded: false,
                    batch_finished: false,
                });
                city.raid_clock = 0.04;
                city
            };
            let mut blocked = fixture();
            blocked.shield = 4.;
            blocked.tick(region, Gifts::default(), 0.1);
            let mut attacked = fixture();
            attacked.tick(region, Gifts::default(), 0.1);
            match biome {
                Biome::Valley => assert_eq!(attacked.stock[0] + 4, blocked.stock[0]),
                Biome::Fjord => assert_eq!(attacked.stock[2] + 4, blocked.stock[2]),
                Biome::Terraces => assert!(
                    matches!(attacked.buildings.last().unwrap().crop, Crop::Growing(seconds) if seconds > 10.)
                ),
                Biome::Lake => assert!(attacked.energy < blocked.energy - 18.),
                Biome::Aegean => assert!(attacked.mana < blocked.mana - 7.),
            }
            assert_eq!(
                attacked.next_raid(region),
                Some(Rival::for_biome(biome).interval)
            );
        }
    }

    #[test]
    fn after_the_guards_leave_the_rival_can_still_block_victory() {
        let mut game = Game::start(Mode::Campaign(Biome::Fjord));
        prepare_bridge(&mut game);
        let city = game.campaign.settlements[0].as_mut().unwrap();
        city.guards.fill(0);
        game.step(1.);
        assert_eq!(game.city().quest, Quest::Bridge);
        for _ in 0..RIVAL_RESOLVE {
            let mut target = None;
            for _ in 0..240 {
                target = game
                    .city()
                    .rival_target(game.region(), Miracle::Wind.radius(game.campaign.gifts));
                if target.is_some() {
                    break;
                }
                game.step(1.);
            }
            let target = target.expect("The rival approaches into casting range");
            game.act(Action::Cast(Miracle::Wind, target)).unwrap();
            game.step(6.);
        }
        assert_eq!(game.city().outcome, Some(Outcome::Victory));
        assert_eq!(game.city().reputation.tone(), Tone::Stern);
        assert_eq!(game.city().rival_resolve, 0);
    }

    #[test]
    fn sharing_a_meal_spends_food_once_per_day_even_after_loading() {
        let mut game = Game::start(Mode::Campaign(Biome::Valley));
        game.campaign.settlements[0].as_mut().unwrap().stock[2] = 6;
        let tile = game.city().walkers[1].tile();
        let (stock, faith) = (game.city().stock, game.city().faith);
        game.act(Action::ShareMeal(tile)).unwrap();
        assert_eq!(game.city().stock[2], stock[2] - 3);
        assert_eq!(game.city().faith, faith + 2);
        let saved = serde_json::to_vec(&game.saved_games()).unwrap();
        let mut game = Game::resume(serde_json::from_slice(&saved).unwrap()).unwrap();
        assert!(game.act(Action::ShareMeal(tile)).is_err());
        assert_eq!(game.city().stock[2], stock[2] - 3);
        game.campaign.settlements[0].as_mut().unwrap().weather_time += DAY_LENGTH;
        game.act(Action::ShareMeal(tile)).unwrap();
        assert_eq!(game.city().stock[2], 0);
    }

    #[test]
    fn the_practice_island_teaches_the_main_actions_before_play() {
        use crate::practice::Step;

        let mut game = Game::start(Mode::Tutorial);
        let campaign = Mode::Campaign(Biome::Valley);
        assert!(game.practice_required());
        assert!(game.switch_mode(campaign).is_err());
        let mut play = Play::new(&game);
        let mut view = CameraRig::new(&game);
        play.observe_practice(&game, &mut view, 0.);
        assert_eq!(play.aim, game.region().sources[0].tile);
        assert!(play.start_grab(&game, 0.));
        assert_eq!(play.grab.unwrap().amount(0.10), PAW_BUNDLE);
        assert_eq!(game.city().nodes[0], game.region().sources[0].stock);
        assert!(play.finish_grab(&mut game, &mut view, 2.));
        assert_eq!(game.city().held.unwrap().count, PAW_LIMIT);
        assert!(!play.finish_grab(&mut game, &mut view, 2.));
        let saved = serde_json::to_vec(&game.saved_games()).unwrap();
        let mut game = Game::resume(serde_json::from_slice(&saved).unwrap()).unwrap();
        assert_eq!(game.city().held.unwrap().count, PAW_LIMIT);

        for step in [Step::Drop, Step::House, Step::Field, Step::Rain] {
            assert_eq!(Step::current(game.city()), step);
            play.observe_practice(&game, &mut view, 3.);
            if let Some((wheel, choice)) = step.wheel() {
                let before = game.city().stock;
                play.activate(UiAction::Confirm, &mut game, &mut view, 3.);
                assert_eq!(play.panel, Panel::Wheel(wheel));
                assert_eq!(play.focus, Some(choice));
                play.activate(choice, &mut game, &mut view, 3.);
                assert_eq!(play.panel, Panel::Closed);
                assert_eq!(before, game.city().stock);
            }
            assert!(
                play.activate(UiAction::Confirm, &mut game, &mut view, 3.),
                "{step:?} · {}",
                play.notice
            );
        }
        assert_eq!(Step::current(game.city()), Step::Grow);
        for _ in 0..480 {
            game.step(1. / 24.);
        }
        assert_eq!(Step::current(game.city()), Step::Harvest);
        play.observe_practice(&game, &mut view, 24.);
        assert!(play.activate(UiAction::Confirm, &mut game, &mut view, 24.));
        assert_eq!(Step::current(game.city()), Step::SendHippo);
        play.observe_practice(&game, &mut view, 25.);
        play.activate(UiAction::Confirm, &mut game, &mut view, 25.);
        let order = play.focus.expect("recommended gathering order");
        play.activate(order, &mut game, &mut view, 25.);
        assert!(play.activate(UiAction::Confirm, &mut game, &mut view, 25.));
        for _ in 0..720 {
            game.step(1. / 24.);
            if Step::current(game.city()) == Step::Pet {
                break;
            }
        }
        assert_eq!(Step::current(game.city()), Step::Pet);
        play.observe_practice(&game, &mut view, 55.);
        assert!(play.activate(UiAction::Confirm, &mut game, &mut view, 56.));
        play.observe_practice(&game, &mut view, 57.);
        assert!(game.campaign_complete());
        assert_eq!(play.panel, Panel::Graduation);
        game.switch_mode(campaign).unwrap();
        assert_eq!(game.mode, campaign);
        assert!(game.has_save(Mode::Tutorial));
        assert_eq!(game.city().buildings.len(), 0);
    }

    #[test]
    fn the_main_menu_reopens_practice_and_preserves_the_last_campaign() {
        let mut game = Game::start(Mode::Tutorial);
        let mut play = Play::new(&game);
        let mut view = CameraRig::new(&game);
        play.show_main_menu(&game);
        assert_eq!(play.focus, Some(UiAction::SelectMode(Mode::Tutorial)));
        for shortcut in [
            UiAction::Panel(Panel::Map),
            UiAction::ToolTarget,
            UiAction::Cancel,
        ] {
            play.activate(shortcut, &mut game, &mut view, 0.);
            assert_eq!(play.panel, Panel::MainMenu);
        }
        play.activate(
            UiAction::SelectMode(Mode::Tutorial),
            &mut game,
            &mut view,
            1.,
        );
        assert_eq!(play.panel, Panel::Closed);
        let source = game.region().sources[0].tile;
        game.act(Action::Grab(source, PAW_BUNDLE)).unwrap();
        play.activate(UiAction::Panel(Panel::Menu), &mut game, &mut view, 2.);
        assert_eq!(play.focus, Some(UiAction::MainMenu));
        play.activate(UiAction::MainMenu, &mut game, &mut view, 2.);
        play.activate(
            UiAction::SelectMode(Mode::Tutorial),
            &mut game,
            &mut view,
            2.,
        );
        assert_eq!(game.city().held.unwrap().count, PAW_BUNDLE);

        game.campaign.settlements[0].as_mut().unwrap().quest = Quest::Complete;
        play.show_main_menu(&game);
        play.activate(
            UiAction::SelectMode(Mode::Tutorial),
            &mut game,
            &mut view,
            3.,
        );
        assert_eq!(
            crate::practice::Step::current(game.city()),
            crate::practice::Step::Grab
        );
        assert!(!game.practice_required());
        let campaign = Mode::Campaign(Biome::Valley);
        game.switch_mode(campaign).unwrap();
        let source = game.region().sources[0].tile;
        game.act(Action::Grab(source, PAW_LIMIT)).unwrap();
        play.show_main_menu(&game);
        play.activate(
            UiAction::SelectMode(Mode::Tutorial),
            &mut game,
            &mut view,
            4.,
        );
        assert!(game.city().held.is_none());
        let saved = serde_json::to_vec(&game.saved_games()).unwrap();
        let mut game = Game::resume(serde_json::from_slice(&saved).unwrap()).unwrap();
        play.show_main_menu(&game);
        assert_eq!(play.focus, Some(UiAction::SelectMode(campaign)));
        play.activate(UiAction::SelectMode(campaign), &mut game, &mut view, 5.);
        assert_eq!(game.city().held.unwrap().count, PAW_LIMIT);
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
        assert_eq!(resumed.region().world_layout(), WorldLayout::Islands);
        assert_eq!(
            resumed.city().quest_title(resumed.region()),
            "Der Pfad zwischen den Brunnen"
        );
        let second = resumed.region().clone();
        let data = serde_json::to_vec(&resumed.saved_games()).unwrap();
        let resumed = Game::resume(serde_json::from_slice(&data).unwrap()).unwrap();
        assert_eq!(resumed.region().world_layout(), WorldLayout::Islands);
        for tile in second.bounds.tiles() {
            assert_eq!(second.water(tile), resumed.region().water(tile));
        }
    }

    #[test]
    fn the_fjord_guide_assigns_the_hippo_and_both_winter_goals_survive_a_reload() {
        let mut game = legacy_campaign(Biome::Fjord);
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
    fn the_first_village_delivery_builds_a_home_and_the_guide_selects_the_store() {
        let mut game = legacy_campaign(Biome::Valley);
        game.campaign.intro_seen = true;
        let depot = game.region().depot;
        let morale = game.city().society.morale;
        let mut play = Play::new(&game);
        let mut view = CameraRig::new(&game);

        assert!(play.activate(UiAction::Confirm, &mut game, &mut view, 0.));
        assert_eq!(play.panel, Panel::FirstDelivery);
        assert_eq!(play.focus, Some(UiAction::Focus(depot)));
        play.activate(UiAction::Focus(depot), &mut game, &mut view, 0.);
        assert!(game.city().opening_choice().is_none());
        assert!(play.activate(UiAction::Confirm, &mut game, &mut view, 0.));

        assert_eq!(game.city().opening_choice(), Some(FirstDelivery::Village));
        assert_eq!(game.city().count(Building::House), 1);
        assert_eq!(game.city().society.morale, morale + 12);
        assert_eq!(game.city().stock, game.region().initial);

        play.tool(Tool::Build);
        play.choose(Building::House.index());
        play.activate(UiAction::Guide, &mut game, &mut view, 0.);
        assert_eq!(play.building(), Building::Store);
        assert!(play.activate(UiAction::Confirm, &mut game, &mut view, 0.));
        assert_eq!(game.city().count(Building::House), 1);
        assert_eq!(game.city().count(Building::Store), 1);
    }

    #[test]
    fn the_farm_delivery_preserves_excess_wood_and_cannot_be_repeated_after_reload() {
        let mut game = legacy_campaign(Biome::Valley);
        game.campaign.intro_seen = true;
        let region = game.region().clone();
        let farm = game.city().opening_farm(&region).unwrap();
        assert!(!region.water(farm) && !region.rocky(farm) && !region.fixed_structure(farm));
        assert!(region.source_at(farm).is_none());
        assert!(
            game.city()
                .route_to(&region, farm.offset(-1, 0), farm, false)
                .is_none()
        );
        assert!(game.act(Action::Hand(farm)).is_err());

        game.act(Action::Grab(region.sources[0].tile, PAW_LIMIT))
            .unwrap();
        let saved = serde_json::to_vec(&game.saved_games()).unwrap();
        let mut game = Game::resume(serde_json::from_slice(&saved).unwrap()).unwrap();
        assert_eq!(Play::new(&game).panel, Panel::FirstDelivery);
        game.act(Action::Hand(farm)).unwrap();
        assert_eq!(game.city().opening_choice(), Some(FirstDelivery::Farm));
        assert_eq!(game.city().count(Building::House), 0);
        assert_eq!(
            game.city().stock,
            [
                region.initial[0],
                region.initial[1] + 5,
                region.initial[2] + 8
            ]
        );
        assert_eq!(game.city().held.unwrap().count, PAW_LIMIT - PAW_BUNDLE);

        let saved = serde_json::to_vec(&game.saved_games()).unwrap();
        let mut game = Game::resume(serde_json::from_slice(&saved).unwrap()).unwrap();
        assert_eq!(game.city().opening_choice(), Some(FirstDelivery::Farm));
        assert!(game.act(Action::Hand(farm)).is_err());
        game.act(Action::Hand(region.depot)).unwrap();
        assert_eq!(
            game.city().stock[0],
            region.initial[0] + PAW_LIMIT - PAW_BUNDLE
        );
        assert!(game.city().held.is_none());
    }

    #[test]
    fn a_wood_lesson_only_prompts_a_new_trip_when_the_stock_is_low() {
        let mut game = legacy_campaign(Biome::Valley);
        let wood = game.region().sources[0].tile;
        let depot = game.region().depot;
        game.act(Action::Hand(wood)).unwrap();
        game.act(Action::Hand(depot)).unwrap();
        game.act(Action::Assign(wood, true)).unwrap();
        for _ in 0..480 {
            game.step(1. / 24.);
            if game.city().hippo_deliveries == 1 {
                break;
            }
        }
        assert_eq!(game.city().hippo_deliveries, 1);
        game.act(Action::Praise).unwrap();
        assert!(game.campaign.gifts.gathers_low_wood);
        for _ in 0..48 {
            game.step(1. / 24.);
        }
        assert!(!game.city().supply_help_active());
        build(&mut game, Building::Store);
        assert_eq!(game.city().stock[0], LOW_SUPPLY);
        game.campaign.settlements[0].as_mut().unwrap().nodes[0] = 0;
        game.step(1. / 24.);
        assert!(game.city().looking_at_store());
        assert!(matches!(game.city().walkers[0].job, Some(Job::Refill(source)) if source != 0));

        let saved = serde_json::to_vec(&game.saved_games()).unwrap();
        let mut game = Game::resume(serde_json::from_slice(&saved).unwrap()).unwrap();
        for _ in 0..720 {
            game.step(1. / 24.);
            if game.city().hippo_deliveries == 2 {
                break;
            }
        }
        assert_eq!(game.city().hippo_deliveries, 2);
        assert!(game.city().stock[0] > LOW_SUPPLY);
        assert!(!game.city().supply_help_active());

        game.campaign.settlements[0].as_mut().unwrap().stock[0] = LOW_SUPPLY;
        game.act(Action::Rest).unwrap();
        for _ in 0..48 {
            game.step(1. / 24.);
        }
        assert!(!game.city().supply_help_active());
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
                batch_finished: false,
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
                        stock_for(game, kind.upgrade().unwrap().materials);
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

    fn complete_village_start(game: &mut Game) {
        game.campaign.intro_seen = true;
        let mut play = Play::new(game);
        let mut camera = CameraRig::new(game);
        for step in 0..24 {
            let task = game.city().village_task(game.region()).unwrap();
            match task {
                VillageTask::Explore => break,
                VillageTask::Grow(_) => game.step(11.),
                _ => {
                    play.activate(UiAction::Guide, game, &mut camera, step as f32);
                    assert!(
                        play.activate(UiAction::Confirm, game, &mut camera, step as f32),
                        "{} · {task:?} · {} · {:?}",
                        game.region().name,
                        play.notice,
                        play.target(game)
                    );
                }
            }
        }
        assert_eq!(
            game.city().village_task(game.region()),
            Some(VillageTask::Explore)
        );
        let i = game
            .city()
            .reachable_neighbor(game.region())
            .expect("A reachable village");
        game.act(Action::MeetNeighbor(i)).unwrap();
        for _ in 0..900 {
            if game.city().village_start == Some(VillageStart::Acquainted(i)) {
                return;
            }
            game.step(1. / 12.);
        }
        panic!("{} · The first visitor must arrive", game.region().name);
    }

    #[test]
    fn the_five_openings_stay_calm_until_the_player_starts_rivalry() {
        for biome in Biome::ALL {
            let mut game = Game::start(Mode::Campaign(biome));
            assert_eq!(
                game.city().village_task(game.region()),
                Some(VillageTask::Build(Building::House))
            );
            for _ in 0..6 {
                game.step(60.);
            }
            assert!(game.city().next_raid(game.region()).is_none());
            assert!(game.city().projects.appeal.is_none());
            assert!(game.city().contest.destination.is_none());
            assert_eq!(
                game.city().contest.count(Allegiance::Neutral),
                game.region().villages.len()
            );
            assert!(game.act(Action::BeginRivalry).is_err());
            assert!(game.act(Action::Visit(0, true)).is_err());

            complete_village_start(&mut game);
            let recently_met = game.city().village_start;
            let mut game = Game::resume(game.saved_games()).unwrap();
            assert_eq!(game.city().village_start, recently_met);
            game.step(60.);
            let appeal = game
                .city()
                .projects
                .appeal
                .as_ref()
                .expect("Known neighbor can ask for help")
                .village;
            game.step(230.);
            assert_eq!(
                game.city().projects.appeal.as_ref().unwrap().village,
                appeal
            );
            assert!(game.city().contest.destination.is_none());
            game.act(Action::DeferAppeal(appeal)).unwrap();
            assert!(game.city().projects.appeal.is_none());

            game.act(Action::BeginRivalry).unwrap();
            assert_eq!(game.city().village_start, Some(VillageStart::Open));
            game.step(112.);
            assert!(game.city().contest.destination.is_none());
            for _ in 0..8 {
                game.step(1.);
            }
            assert!(game.city().contest.destination.is_some(), "{biome:?}");
        }
    }

    #[test]
    fn an_early_saved_village_keeps_its_buildings_and_gets_the_calm_start() {
        let mut old = legacy_campaign(Biome::Valley);
        let tile = old.region().sources[0].tile;
        old.act(Action::Grab(tile, 9)).unwrap();
        old.act(Action::Hand(old.region().depot)).unwrap();
        assert_eq!(old.city().opening_choice(), Some(FirstDelivery::Village));
        old.campaign.version = 5;
        let homes = old.city().count(Building::House);
        let stock = old.city().stock;
        let mut resumed = Game::resume(old.saved_games()).unwrap();
        assert_eq!(resumed.city().count(Building::House), homes);
        assert_eq!(resumed.city().stock, stock);
        assert!(resumed.city().rival_waiting());
        assert_eq!(
            resumed.city().opening_choice(),
            Some(FirstDelivery::Village)
        );
        resumed.step(220.);
        assert!(resumed.city().contest.destination.is_none());
    }

    fn prepare_bridge(game: &mut Game) {
        if game.city().village_start == Some(VillageStart::Building) {
            complete_village_start(game);
            game.act(Action::BeginRivalry).unwrap();
        }
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
        if game.city().quest != Quest::Bridge {
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
            stock_for(game, kind.upgrade().unwrap().materials);
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
    fn a_half_full_store_accepts_only_what_fits_and_the_rest_can_be_returned() {
        let mut game = fresh();
        let region = game.region().clone();
        let source = region.sources[0].tile;
        let original = game.city().nodes[0];
        let cap = game.city().storage_cap();
        game.campaign.settlements[0].as_mut().unwrap().stock[0] = cap - 2;
        game.act(Action::Grab(source, 6)).unwrap();
        game.act(Action::Hand(region.depot)).unwrap();
        assert_eq!(game.city().stock[0], cap);
        assert_eq!(game.city().held.unwrap().count, 4);
        assert_eq!(
            game.city().carry_destination(&region, region.depot),
            Some(source)
        );
        game.act(Action::Hand(source)).unwrap();
        assert!(game.city().held.is_none());
        assert_eq!(game.city().nodes[0], original - 2);
    }

    #[test]
    fn praise_teaches_the_delivered_material_and_rest_survives_reload() {
        let mut game = Game::start(Mode::Campaign(Biome::Fjord));
        let region = game.region().clone();
        for m in [Material::Stone, Material::Food] {
            let source = region
                .sources
                .iter()
                .find(|s| s.material == m && game.city().influence(&region, s.tile))
                .unwrap()
                .tile;
            game.act(Action::Assign(source, true)).unwrap();
            let before = game.city().hippo_deliveries;
            for _ in 0..1600 {
                game.step(0.05);
                if game.city().hippo_deliveries > before {
                    break;
                }
            }
            assert_eq!(game.city().supply_lesson(&region), Some(m));
            game.act(Action::Praise).unwrap();
            assert!(game.campaign.gifts.supplies(m));
        }
        assert!(!game.campaign.gifts.supplies(Material::Wood));
        let city = game.campaign.settlements[0].as_mut().unwrap();
        city.stock = [0, 5, 1];
        city.walkers[0].route.clear();
        city.walkers[0].pause = 0.;
        game.step(0.05);
        assert_eq!(game.city().automatic_supply(&region), Some(Material::Food));
        let data = serde_json::to_vec(&game.saved_games()).unwrap();
        let mut game = Game::resume(serde_json::from_slice(&data).unwrap()).unwrap();
        let before = game.city().hippo_deliveries;
        for _ in 0..1600 {
            game.step(0.05);
            if game.city().hippo_deliveries > before {
                break;
            }
        }
        assert_eq!(game.city().hippo_deliveries, before + 1);
        assert!(game.city().stock[2] > 1);
        game.act(Action::Rest).unwrap();
        let data = serde_json::to_vec(&game.saved_games()).unwrap();
        let mut game = Game::resume(serde_json::from_slice(&data).unwrap()).unwrap();
        for _ in 0..100 {
            game.step(0.05);
        }
        assert_eq!(game.city().automatic_supply(&region), None);
    }

    #[test]
    fn construction_deliveries_survive_reload_and_only_the_balance_leaves_the_store() {
        let mut game = Game::start(Mode::Campaign(Biome::Valley));
        prepare_bridge(&mut game);
        let region = game.region().clone();
        let m = Material::ALL
            .into_iter()
            .find(|m| region.rebuild[m.index()] > 1)
            .unwrap();
        let source = region
            .sources
            .iter()
            .enumerate()
            .find(|(i, s)| {
                s.material == m
                    && game.city().nodes[*i] > 0
                    && game.city().influence(&region, s.tile)
            })
            .unwrap()
            .1
            .tile;
        game.act(Action::Grab(source, 1)).unwrap();
        assert_eq!(
            game.city().carry_destination(&region, source),
            Some(region.bridge())
        );
        game.act(Action::Hand(region.bridge())).unwrap();
        let due = game.city().crossing_due(&region);
        assert_eq!(due[m.index()], region.rebuild[m.index()] - 1);
        let data = serde_json::to_vec(&game.saved_games()).unwrap();
        let mut game = Game::resume(serde_json::from_slice(&data).unwrap()).unwrap();
        assert_eq!(game.city().crossing_due(&region), due);
        stock_for(&mut game, due);
        let before = game.city().stock;
        game.act(Action::Repair).unwrap();
        assert_eq!(
            game.city().stock,
            std::array::from_fn(|i| before[i] - due[i])
        );
        assert!(game.act(Action::Repair).is_err());
        let (gift, qty) = region.gift;
        let from = region
            .sources
            .iter()
            .enumerate()
            .find(|(i, s)| {
                s.material == gift
                    && game.city().nodes[*i] > qty
                    && game.city().influence(&region, s.tile)
            })
            .unwrap()
            .1
            .tile;
        let before = game.city().stock;
        game.act(Action::Grab(from, qty + 1)).unwrap();
        game.act(Action::Hand(region.bridge())).unwrap();
        assert!(game.city().gift_given);
        assert_eq!(game.city().held.unwrap().count, 1);
        assert_eq!(game.city().stock, before);
        assert!(game.act(Action::Give).is_err());
    }

    #[test]
    fn chapter_supplies_reflect_both_villages_and_arrive_only_on_first_landing() {
        let mut game = legacy_campaign(Biome::Valley);
        game.campaign.intro_seen = true;
        let city = game.campaign.settlements[0].as_mut().unwrap();
        city.quest = Quest::Complete;
        city.outcome = Some(Outcome::Alliance);
        city.districts[0].outcome = Some(Outcome::Alliance);
        city.districts[1].outcome = Some(Outcome::Victory);
        game.step(0.);
        let expected = game.regions[1].initial;
        let mut play = Play::new(&game);
        let mut camera = CameraRig::new(&game);
        play.activate(UiAction::Panel(Panel::Quests), &mut game, &mut camera, 0.);
        play.activate(UiAction::Guide, &mut game, &mut camera, 0.);
        assert_eq!(play.focus, Some(UiAction::Do(Action::Travel(1))));
        game.act(Action::Travel(1)).unwrap();
        assert_eq!(
            game.city().stock,
            [expected[0], expected[1] + 4, expected[2] + 4]
        );
        let landed = game.city().stock;
        let data = serde_json::to_vec(&game.saved_games()).unwrap();
        let mut game = Game::resume(serde_json::from_slice(&data).unwrap()).unwrap();
        game.act(Action::Travel(0)).unwrap();
        game.act(Action::Travel(1)).unwrap();
        assert_eq!(game.city().stock, landed);
    }

    #[test]
    fn both_villages_in_each_campaign_chapter_offer_working_local_build_plots() {
        for biome in Biome::ALL {
            let mut game = Game::start(Mode::Campaign(biome));
            game.campaign.intro_seen = true;
            for chapter in 0..game.regions.len() {
                game.campaign.unlocked = chapter;
                game.act(Action::Travel(chapter)).unwrap();
                prepare_bridge(&mut game);
                let region = game.region().clone();
                if game.city().count(Building::Workshop) == 0 {
                    place_quest_building(&mut game, Building::Workshop, false);
                }
                stock_for(&mut game, region.rebuild);
                game.act(Action::Repair).unwrap();
                stock_up(&mut game, region.gift.0, region.gift.1);
                game.act(Action::Give).unwrap();
                assert_eq!(game.city().outcome, Some(Outcome::Alliance));
                let mut play = Play::new(&game);
                let mut view = CameraRig::new(&game);
                for i in 0..2 {
                    let kind = region.districts[i].need.building();
                    stock_for(&mut game, kind.price());
                    play.activate(UiAction::HelpDistrict(i), &mut game, &mut view, 0.);
                    assert_eq!(play.tool, Tool::Build, "{}", region.name);
                    assert_eq!(play.building(), kind);
                    assert!(
                        play.activate(UiAction::Confirm, &mut game, &mut view, 0.),
                        "{}: {}",
                        region.name,
                        play.notice
                    );
                    assert!(
                        game.city().district_site(&region, i).is_some(),
                        "{} {}",
                        region.name,
                        region.districts[i].name
                    );
                }
            }
        }
    }

    #[test]
    fn lake_road_guidance_previews_a_connected_run_and_aegean_guidance_opens_research() {
        let mut game = legacy_campaign(Biome::Lake);
        game.campaign.intro_seen = true;
        let mut play = Play::new(&game);
        let mut view = CameraRig::new(&game);
        let well = Building::Well;
        // The first Papyrusufer goal needs a working roadside well.
        if game.city().count(Building::Field) == 0 {
            build(&mut game, Building::Field);
        }
        if game.city().count(Building::Store) == 0 {
            build(&mut game, Building::Store);
        }
        place_quest_building(&mut game, well, true);
        stock_up(&mut game, Material::Stone, 2);
        let before = game.city().stock;
        play.activate(UiAction::Guide, &mut game, &mut view, 0.);
        assert!(play.road_anchor.is_some());
        assert_eq!(game.city().stock, before);
        assert!(play.activate(UiAction::Confirm, &mut game, &mut view, 0.));
        assert!(game.city().roads.len() >= 3);

        let mut game = legacy_campaign(Biome::Aegean);
        game.campaign.unlocked = 1;
        game.act(Action::Travel(1)).unwrap();
        game.campaign.intro_seen = true;
        let mut play = Play::new(&game);
        let mut view = CameraRig::new(&game);
        play.activate(UiAction::Guide, &mut game, &mut view, 0.);
        assert_eq!(play.panel, Panel::Research);
        assert_eq!(
            play.focus,
            Some(UiAction::Do(Action::Research(Tech::Workshops)))
        );
        assert!(play.activate(play.focus.unwrap(), &mut game, &mut view, 0.));
        assert!(game.campaign.research.knows(Tech::Workshops));
    }

    #[test]
    fn a_workshop_shift_can_be_started_by_hand_once_and_resumes_automatic_production() {
        for (kind, material, product) in [
            (Civic::Sawmill, Material::Wood, 0),
            (Civic::Masonry, Material::Stone, 1),
            (Civic::OlivePress, Material::Food, 2),
        ] {
            let mut game = Game::start(Mode::Sandbox(Biome::Aegean));
            let region = game.region().clone();
            let tile = place_quest_building(&mut game, Building::Civic(kind), true);
            let city = game.campaign.settlements[0].as_mut().unwrap();
            city.stock = [16; 3];
            let before = if kind == Civic::OlivePress {
                city.olive_pressings
            } else {
                city.society.goods[product]
            };
            let mut play = Play::new(&game);
            play.aim = tile;
            let context = crate::context::ObjectMenu::at(&play, &game, 0.);
            assert_eq!(
                context.choices[0].action,
                UiAction::Do(Action::Produce(tile))
            );
            game.act(Action::Produce(tile)).unwrap();
            let after = game.city().stock[material.index()];
            assert_eq!(after, 16 - if kind == Civic::OlivePress { 1 } else { 2 });
            let data = serde_json::to_vec(&game.saved_games()).unwrap();
            let mut game = Game::resume(serde_json::from_slice(&data).unwrap()).unwrap();
            assert!(game.act(Action::Produce(tile)).is_err());
            let city = game.campaign.settlements[0].as_mut().unwrap();
            let output = |c: &Settlement| {
                if kind == Civic::OlivePress {
                    c.olive_pressings
                } else {
                    c.society.goods[product]
                }
            };
            city.civic_life(&region, Weather::Breeze);
            assert_eq!(output(city), before + 1);
            city.civic_life(&region, Weather::Breeze);
            assert_eq!(output(city), before + 2);
        }
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
        let themed = [WorldLayout::Regional, WorldLayout::Islands]
            .into_iter()
            .flat_map(|layout| {
                Biome::ALL
                    .into_iter()
                    .flat_map(move |biome| Mode::Campaign(biome).regions(|_| layout))
            });
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
    fn upgrades_check_believers_and_faith_then_pay_once() {
        let mut game = fresh();
        let tile = place_quest_building(&mut game, Building::House, true);
        for kind in [Building::Store, Building::Field, Building::Workshop] {
            build(&mut game, kind);
        }
        let upgrade = Building::House.upgrade().unwrap();
        stock_for(&mut game, upgrade.materials);
        let stock = game.city().stock;
        let city = game.campaign.settlements[0].as_mut().unwrap();
        assert_eq!(city.inhabitants(), upgrade.believers);
        let resident = city.walkers.pop().unwrap();
        assert!(
            game.act(Action::Upgrade(tile))
                .unwrap_err()
                .contains("Gläubige")
        );
        let city = game.campaign.settlements[0].as_mut().unwrap();
        city.walkers.push(resident);
        city.faith = upgrade.faith - 1;
        assert!(
            game.act(Action::Upgrade(tile))
                .unwrap_err()
                .contains("Glauben")
        );
        assert_eq!(game.city().stock, stock);
        let housing = game.city().housing();
        game.campaign.settlements[0].as_mut().unwrap().faith = upgrade.faith;
        assert!(
            game.act(Action::Upgrade(tile))
                .unwrap()
                .contains(upgrade.benefit)
        );
        assert_eq!(game.city().housing(), housing + 1);
        assert_eq!(game.city().faith, upgrade.faith + 3);
        let paid = std::array::from_fn(|i| stock[i] - upgrade.materials[i]);
        assert_eq!(game.city().stock, paid);
        let saved = serde_json::to_vec(&game.saved_games()).unwrap();
        let mut resumed = Game::resume(serde_json::from_slice(&saved).unwrap()).unwrap();
        assert!(resumed.city().building_at(tile).unwrap().upgraded);
        assert!(resumed.act(Action::Upgrade(tile)).is_err());
        assert_eq!(resumed.city().stock, paid);
    }

    #[test]
    fn local_building_checks_preserve_the_full_routes_on_different_shores() {
        for biome in [Biome::Valley, Biome::Aegean] {
            let game = Game::start(Mode::Sandbox(biome));
            let city = game.city();
            let region = game.region();
            let kind = Building::Field;
            let sites = region
                .bounds
                .tiles()
                .filter(|tile| tile.within(region.depot, 10))
                .filter(|&tile| kind.tiles(tile).all(|cell| city.walkable(region, cell)))
                .filter(|&tile| city.footprint_has_detour(region, kind, tile))
                .take(12);
            let mut count = 0;
            for tile in sites {
                assert!(
                    city.footprint_keeps_routes(region, kind, tile),
                    "{biome:?} · {tile:?}"
                );
                count += 1;
            }
            assert_eq!(count, 12);
        }
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
        assert_eq!(allied.city().reputation.tone(), Tone::Gentle);

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
        assert!(game.campaign.gifts.creature.stage() >= 1);
        let saved = serde_json::to_vec(&game.saved_games()).unwrap();
        let resumed = Game::resume(serde_json::from_slice(&saved).unwrap()).unwrap();
        assert_eq!(
            game.campaign.gifts.creature.experience(),
            resumed.campaign.gifts.creature.experience()
        );
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

#[cfg(test)]
mod wild_tests {
    use crate::terrain::wild_land;
    use crate::{
        game::{Action, Game},
        modes::Mode,
        region::{Biome, Material},
    };
    #[test]
    fn old_deposits_keep_their_indices_and_new_landscape_props_support_hand_and_workers() {
        let game = Game::start(Mode::Campaign(Biome::Valley));
        let mut old = game.saved_games();
        let p = old
            .profiles
            .iter_mut()
            .find(|p| p.mode == old.current)
            .unwrap();
        p.campaign.version = 4;
        let old_city = p.campaign.settlements[0].as_mut().unwrap();
        old_city.nodes.truncate(game.region().managed_sources);
        old_city.regrowth.truncate(game.region().managed_sources);
        old_city.nodes[0] = 3;
        let upgraded = Game::resume(old).unwrap();
        assert_eq!(upgraded.city().nodes[0], 3);
        for m in [Material::Wood, Material::Stone] {
            let mut game = Game::resume(upgraded.saved_games()).unwrap();
            let r = game.region();
            let (i, s) = r
                .sources
                .iter()
                .enumerate()
                .skip(r.managed_sources)
                .find(|(i, s)| {
                    s.material == m
                        && game.city().influence(r, s.tile)
                        && game.city().harvest_at(r, s.tile) == Some(*i)
                })
                .unwrap();
            let tile = s.tile;
            let before = game.city().nodes[i];
            assert_eq!(wild_land(r, tile).unwrap().kind, Some(m));
            game.act(Action::Assign(tile, true)).unwrap();
            game.act(Action::Hand(tile)).unwrap();
            assert_eq!(game.city().held.unwrap().source, i);
            assert!(game.city().nodes[i] < before);
            let loaded = Game::resume(game.saved_games()).unwrap();
            assert_eq!(loaded.city().nodes[i], game.city().nodes[i]);
            assert_eq!(loaded.city().held.unwrap().source, i);
        }
    }
}
