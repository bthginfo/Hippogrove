use std::{
    cmp::Reverse,
    collections::{BinaryHeap, VecDeque},
};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::region::{Biome, CELL, DistrictNeed, Material, Region, Tile, Weather};

pub const SAVE_VERSION: u32 = 2;
const ORIGINAL_SOURCE_COUNT: usize = 10;
pub const START_RADIUS: i16 = 7;
pub const FRONTIER_RADIUS: i16 = 8;
const RIVAL_RADIUS: i16 = 6;
const DISTRICT_RADIUS: i16 = 6;
const WAYSTONE_FAITH: u16 = 5;
const WAYSTONE_LIMIT: usize = 24;
pub const MAX_ROAD_RUN: u16 = 12;
pub const MAX_WALKERS: usize = 16;

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
        }
    }

    pub fn size(self) -> i16 {
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

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
pub enum Action {
    Hand(Tile),
    Build(Building, Tile, u8),
    Road(Tile, Tile),
    Upgrade(Tile),
    Settle(usize),
    Assign(Tile, bool),
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Job {
    Gather(usize),
    Watch(Tile),
    Tend(Tile),
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Lesson {
    Delivery(usize),
    Practiced(Job),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Walker {
    pub position: [f32; 2],
    pub hippo: bool,
    pub job: Option<Job>,
    pub bundle: Option<Bundle>,
    pub route: VecDeque<Tile>,
    pub pause: f32,
}

impl Walker {
    fn new(tile: Tile, hippo: bool) -> Self {
        Self {
            position: [f32::from(tile.0), f32::from(tile.1)],
            hippo,
            job: None,
            bundle: None,
            route: VecDeque::new(),
            pause: 0.,
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
pub struct Settlement {
    pub stock: Stock,
    pub faith: u16,
    pub mana: f32,
    pub buildings: Vec<BuildingSite>,
    #[serde(default)]
    pub roads: Vec<Tile>,
    #[serde(default)]
    pub districts: [DistrictStatus; 2],
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
    pub lesson: Option<Lesson>,
    #[serde(default = "full_energy")]
    pub energy: f32,
    #[serde(default)]
    recovering: bool,
    #[serde(default)]
    cooldowns: [f32; Miracle::ALL.len()],
    #[serde(default)]
    pub local_finished: bool,
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
            stock: region.initial,
            faith: 2,
            mana: 20.,
            buildings: Vec::new(),
            roads: Vec::new(),
            districts: [DistrictStatus::default(); 2],
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
            lesson: None,
            energy: full_energy(),
            recovering: false,
            cooldowns: [0.; Miracle::ALL.len()],
            local_finished: false,
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
        city
    }

    pub fn count(&self, building: Building) -> usize {
        self.buildings
            .iter()
            .filter(|site| site.kind == building)
            .count()
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
        let structures = Building::ALL.iter().fold(0, |renown, &kind| {
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
            && (tile.within(region.start, START_RADIUS)
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
            && (!region.water(tile) || (self.bridge_fixed && region.crossing(tile)))
            && !region.rocky(tile)
            && !region.fixed_structure(tile)
            && region.source_at(tile).is_none()
            && self.building_at(tile).is_none()
    }

    pub fn can_store_at(&self, region: &Region, tile: Tile) -> bool {
        tile == region.depot
            || region
                .district_at(tile)
                .is_some_and(|index| self.districts[index].outcome.is_some())
            || self
                .building_at(tile)
                .is_some_and(|site| matches!(site.kind, Building::Store | Building::Granary))
    }

    pub fn road_at(&self, region: &Region, tile: Tile) -> bool {
        region.road(tile) || self.roads.contains(&tile)
    }

    pub fn connected(&self, region: &Region, site: &BuildingSite) -> bool {
        site.roadside(|cell| self.road_at(region, cell) && self.walkable(region, cell))
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
                    && cells.iter().any(|&cell| {
                        std::iter::once(cell)
                            .chain(cell.neighbors())
                            .any(|next| self.road_at(region, next) && self.walkable(region, next))
                    })
            })
            .ok_or("Der Weg braucht freien Boden und Anschluss an einen Dorfweg.")?;
        let tiles: Vec<_> = cells
            .into_iter()
            .filter(|&tile| !self.road_at(region, tile))
            .collect();
        if tiles.is_empty() {
            return Err("Hier liegt schon ein Weg.".into());
        }
        let price = [0, (tiles.len() as u16).div_ceil(3), 0];
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
        if self.city_rank() < 3 {
            return Err("Baue zuerst eine Werkstatt.".into());
        }
        if !self.connected(region, site) {
            return Err("Schließe das Gebäude an einen Dorfweg an.".into());
        }
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

    fn can_deposit(&self, material: Material, count: u16) -> bool {
        self.stock[material.index()].saturating_add(count) <= self.storage_cap()
    }

    pub fn placement(&self, region: &Region, kind: Building, tile: Tile) -> Result<(), String> {
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
                return Err("Setze den Grenzstein neben einen Weg an der goldenen Grenze.".into());
            }
            _ => {}
        }
        for cell in kind.tiles(tile) {
            if !self.influence(region, cell) {
                return Err("Der ganze Grundriss muss im Einflussgebiet liegen.".into());
            }
            if !self.walkable(region, cell)
                || self.road_at(region, cell)
                || (cell == region.frontier && kind != Building::Shrine)
                || self.walkers.iter().any(|walker| walker.tile() == cell)
            {
                return Err("Suche freien Boden neben dem Weg.".into());
            }
        }
        if !self.affordable(kind.price()) {
            return Err(format!(
                "Es fehlen Baustoffe. Benötigt werden {}.",
                stock_text(kind.price())
            ));
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
        let mut walk = vec![true; bounds.len()];
        for &(center, radius) in &region.highlands {
            for x in -radius..=radius {
                for z in -radius..=radius {
                    let tile = center.offset(x, z);
                    if let Some(index) = bounds.index(tile).filter(|_| tile.within(center, radius))
                    {
                        walk[index] = false;
                    }
                }
            }
        }
        for x in bounds.0.0..=bounds.1.0 {
            let bank = region.river_at(x);
            for z in bank..bank + region.river_span(x) {
                let tile = Tile(x, z);
                if let Some(index) = bounds.index(tile) {
                    walk[index] &= self.bridge_fixed && region.crossing(tile);
                }
            }
        }
        let landmarks = [region.start, region.depot, region.rival_shrine]
            .into_iter()
            .chain(region.rival_buildings())
            .chain(
                region
                    .districts
                    .iter()
                    .enumerate()
                    .flat_map(|(index, district)| {
                        std::iter::once(district.center).chain(region.district_houses(index))
                    }),
            );
        for tile in landmarks
            .chain(region.sources.iter().map(|source| source.tile))
            .chain(
                self.buildings
                    .iter()
                    .flat_map(|site| site.kind.tiles(site.tile)),
            )
        {
            if let Some(index) = bounds.index(tile) {
                walk[index] = false;
            }
        }
        walk
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

    fn route_to_store(&self, region: &Region, start: Tile) -> Option<VecDeque<Tile>> {
        let mut stores: Vec<_> = std::iter::once(region.depot)
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
                    .filter(|site| matches!(site.kind, Building::Store | Building::Granary))
                    .flat_map(|site| site.kind.tiles(site.tile)),
            )
            .collect();
        stores.sort_by_key(|&tile| start.distance(tile));
        stores
            .into_iter()
            .find_map(|tile| self.route_to(region, start, tile, true))
    }

    fn advance_walker(&mut self, walker: &mut Walker, region: &Region, gifts: Gifts, dt: f32) {
        if walker.hippo {
            if self.energy <= 9. {
                self.recovering = true;
            }
            let moving = !walker.route.is_empty() || walker.bundle.is_some();
            self.energy = if moving && !self.recovering {
                (self.energy - dt * if gifts.training >= 3 { 1.7 } else { 2.6 }).max(0.)
            } else {
                (self.energy + dt * if self.recovering { 7. } else { 4. }).min(full_energy())
            };
            if self.recovering {
                self.recovering = self.energy < 38.;
                return;
            }
        }
        walker.pause = (walker.pause - dt).max(0.);
        if walker.pause > 0. {
            return;
        }
        if let Some(step) = walker.route.front().copied() {
            let target = Vec2::new(f32::from(step.0), f32::from(step.1));
            let point = Vec2::from_array(walker.position);
            let delta = target - point;
            let speed = if walker.hippo {
                2.65 + f32::from(gifts.training >= 4) * 0.41
            } else {
                2.25
            };
            let distance = dt * speed * if self.road_at(region, step) { 1.5 } else { 1. };
            if delta.length_squared() <= distance * distance {
                walker.position = target.to_array();
                walker.route.pop_front();
            } else {
                walker.position = (point + delta.normalize() * distance).to_array();
            }
            return;
        }

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
                        self.lesson = Some(Lesson::Delivery(bundle.source));
                        if gifts.training == 0 {
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
            Some(Job::Gather(source)) => {
                if self.nodes[source] == 0 {
                    walker.pause = 2.;
                    return;
                }
                let target = region.sources[source].tile;
                let Some(route) = self.route_to(region, walker.tile(), target, true) else {
                    walker.pause = 1.;
                    return;
                };
                if route.is_empty() {
                    let count = if walker.hippo {
                        5 + u16::from(gifts.training) * 2
                    } else {
                        3
                    };
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
                            self.lesson = Some(Lesson::Practiced(Job::Watch(target)));
                        }
                    }
                } else if self.count(Building::Shrine) > 0 && target.within(region.bridge(), 4) {
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
            _ => {}
        }
    }

    fn tick(&mut self, region: &Region, gifts: Gifts, dt: f32) {
        let recharge = if self.count(Building::Shrine) > 0 {
            1.70
        } else {
            1.0
        };
        self.mana = (self.mana + recharge * dt).min(gifts.mana_cap());
        self.shield = (self.shield - dt).max(0.);
        self.rainfall = (self.rainfall - dt).max(0.);
        self.weather_time += dt;
        let weather = region.weather(self.weather_time);
        for cooldown in &mut self.cooldowns {
            *cooldown = (*cooldown - dt).max(0.);
        }

        for (index, (stock, delay)) in self.nodes.iter_mut().zip(&mut self.regrowth).enumerate() {
            if *stock == 0 && !self.held.is_some_and(|bundle| bundle.source == index) {
                *delay += dt;
                if *delay >= 28. {
                    *stock = region.sources[index].stock;
                    *delay = 0.;
                }
            }
        }
        let wells: Vec<_> = self
            .buildings
            .iter()
            .filter(|site| site.kind == Building::Well && self.connected(region, site))
            .map(|site| (site.tile, if site.upgraded { 8 } else { 5 }))
            .collect();
        for site in &mut self.buildings {
            let watered = wells
                .iter()
                .any(|&(well, radius)| site.tile.within(well, radius));
            let growth_step = dt
                * match weather {
                    Weather::Drought if !watered => 0.55,
                    Weather::Mist => 1.22,
                    _ => 1.,
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
        for walker in &mut walkers {
            self.advance_walker(walker, region, gifts, dt);
        }
        self.walkers = walkers;

        self.provisions -= dt;
        if self.provisions <= 0. {
            self.provisions = match region.kind {
                Biome::Fjord => 17.,
                Biome::Lake => 19.,
                _ => 24.,
            };
            let winter_cost = u16::from(
                weather == Weather::Snow
                    && !self
                        .buildings
                        .iter()
                        .any(|site| site.kind == Building::Granary && self.connected(region, site)),
            );
            let food = &mut self.stock[Material::Food.index()];
            if *food > 4 {
                *food = (*food - (1 + winter_cost + self.walkers.len() as u16 / 7)).max(4);
            }
            self.trade(region);
        }

        if self.raid_clock > 0. && self.outcome.is_none() {
            self.raid_clock -= dt;
            if self.raid_clock <= 0. {
                self.raid_clock = 21.;
                let protected = self.shield > 0.
                    || self.tower_protects(region, region.bridge())
                    || self.walkers.iter().any(|walker| {
                        walker.hippo
                            && matches!(walker.job, Some(Job::Watch(_)))
                            && walker.tile().within(region.bridge(), 4)
                    });
                if !protected {
                    self.stock[Material::Food.index()] =
                        self.stock[Material::Food.index()].saturating_sub(2);
                }
            }
        }
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
        self.buildings.iter().any(|site| {
            site.kind == Building::Tower
                && self.connected(region, site)
                && site.tile.within(point, if site.upgraded { 9 } else { 5 })
        })
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
                region.sources[bundle.source].material.name()
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
                region.sources[index].material.name()
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
        self.pay(kind.price());
        self.buildings.push(BuildingSite {
            kind,
            tile,
            rotation: rotation % 4,
            crop: Crop::Dry,
            upgraded: false,
        });
        for walker in &mut self.walkers {
            walker.route.clear();
        }
        if kind == Building::House {
            self.welcome_resident(region, self.buildings.len() - 1);
        }
        if kind == Building::Waystone {
            self.faith -= WAYSTONE_FAITH;
        } else {
            self.faith = self
                .faith
                .saturating_add(if kind == Building::Shrine { 3 } else { 2 });
        }
        Ok(format!("{} gebaut", kind.name()))
    }

    fn welcome_resident(&mut self, region: &Region, index: usize) {
        if self.walkers.len() >= MAX_WALKERS {
            return;
        }
        let site = &self.buildings[index];
        if let Some(arrival) = site
            .kind
            .tiles(site.tile)
            .flat_map(Tile::neighbors)
            .find(|&tile| self.walkable(region, tile))
        {
            self.walkers.push(Walker::new(arrival, false));
        }
    }

    fn assign(&mut self, region: &Region, tile: Tile, hippo: bool) -> GameResult {
        let index = region
            .source_at(tile)
            .ok_or("Wähle Holz, Stein oder Nahrung für den Auftrag.")?;
        let worker = if hippo {
            0
        } else {
            let index = self
                .walkers
                .iter()
                .position(|walker| !walker.hippo && walker.job.is_none())
                .unwrap_or_else(|| 1 + self.next_worker % (self.walkers.len() - 1));
            self.next_worker = self.next_worker.wrapping_add(1);
            index
        };
        if self
            .route_to(region, self.walkers[worker].tile(), tile, true)
            .is_none()
        {
            return Err("Der Übergang zu dieser Quelle ist noch versperrt.".into());
        }
        let walker = &mut self.walkers[worker];
        walker.job = Some(Job::Gather(index));
        walker.route.clear();
        walker.pause = 0.;
        Ok(format!(
            "{} holt {}.",
            if hippo {
                "Dein Nilpferd"
            } else {
                "Ein Gläubiger"
            },
            region.sources[index].material.name(),
        ))
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
            Some(Job::Gather(_)) => "Sammelt",
            Some(Job::Watch(_)) => "Hält Wache",
            Some(Job::Tend(_)) => "Pflegt ein Feld",
            None if self.walkers[0].bundle.is_some() => "Bringt eine Ladung heim",
            None => "Rastet",
        }
    }

    fn threat_in(&self, region: &Region, tile: Tile, radius: i16) -> bool {
        (self.quest == Quest::Bridge
            && self.count(Building::Shrine) > 0
            && region
                .guards
                .iter()
                .zip(&self.guards)
                .any(|(&guard, &spirit)| spirit > 0 && guard.within(tile, radius)))
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

    pub fn ritual_error(
        &self,
        region: &Region,
        spell: Miracle,
        tile: Tile,
        gifts: Gifts,
    ) -> Option<&'static str> {
        if !self.influence(region, tile) {
            return Some("Das Ziel liegt außerhalb deines Einflusses.");
        }
        if self.mana + 0.001 < spell.cost() {
            return Some("Wunderkraft lädt am Schrein wieder auf.");
        }
        if self.cooldowns[spell as usize] > 0.001 {
            return Some("Das Wunder muss sich kurz erholen.");
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
            Miracle::Thunder if gifts.storm == 0 => {
                Some("Lerne Stimme des Donners im Fortschrittsmenü.")
            }
            Miracle::Wind | Miracle::Thunder if !self.threat_in(region, tile, radius) => {
                Some("Ziele auf Wachen am Übergang oder eine widerständige Siedlung.")
            }
            Miracle::Ward if self.city_rank() < 3 => Some("Die Werkstatt öffnet das Schutzwunder."),
            Miracle::Bloom if gifts.growth == 0 => {
                Some("Lerne Hände der Blüte im Fortschrittsmenü.")
            }
            Miracle::Bloom if !self.blooms_at(region, tile, radius) => {
                Some("Ziele auf wachsende Saat, Obst oder genutztes Holz.")
            }
            Miracle::Mend if gifts.training == 0 => {
                Some("Dein Nilpferd lernt Heillicht nach dem ersten Lob kennen.")
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
                for (&guard, spirit) in region.guards.iter().zip(&mut self.guards) {
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
            return Err(format!("Das Dorf bittet um {amount} {}.", material.name()));
        }
        self.pay(price);
        self.gift_given = true;
        self.faith = self.faith.saturating_add(5);
        Ok("Die Nachbarn haben deine Gabe angenommen.".into())
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
            Quest::Fields if self.harvests > 0 && self.count(Building::Field) > 0 => {
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
            Quest::Districts if self.districts.iter().all(|state| state.outcome.is_some()) => {
                self.quest = Quest::Complete;
                self.faith = self.faith.saturating_add(12);
                true
            }
            _ => false,
        }
    }

    pub fn objective(&self, region: &Region) -> String {
        match self.quest {
            Quest::Shelter if self.hand_deliveries == 0 =>
                "Pfote wählen, einen Holzstoß antippen, greifen und am Sammelplatz ablegen.".into(),
            Quest::Shelter if self.count(Building::House) == 0 =>
                "Baue eine Hütte auf freiem Boden bei deinem Schrein.".into(),
            Quest::Shelter if self.count(Building::Store) == 0 =>
                "Baue ein Lager. Helfer können beim Sammeln mitarbeiten.".into(),
            Quest::Shelter => "Gib dem Nilpferd einen Sammelauftrag und warte auf seine Lieferung.".into(),
            Quest::Fields if self.count(Building::Field) == 0 =>
                "Lege ein Feld an. Dort lässt Regen deine erste Saat wachsen.".into(),
            Quest::Fields => "Wirke Regenschauer auf das Feld. Ernte mit der Pfote, sobald die Pflanzen reif sind.".into(),
            Quest::Bridge if self.count(Building::Shrine) == 0 =>
                "Baue den Grenzschrein auf dem goldenen Platz am Übergang.".into(),
            Quest::Bridge => format!("Baue Ansehen ({}/{}) auf und erfülle die Bitte. Oder vertreibe die Wachen mit Windstößen.", self.renown(), region.renown),
            Quest::Districts => format!("Führe die Nachbardörfer zusammen ({}/2). Siedlungen wählst du auf der Landkarte.", self.districts.iter().filter(|state| state.outcome.is_some()).count()),
            Quest::Complete => "Kapitel abgeschlossen. Du kannst weiterbauen oder weiterreisen.".into(),
        }
    }

    pub fn local_quest(&self, region: &Region) -> (&'static str, String, bool) {
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
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Campaign {
    pub version: u32,
    pub active: usize,
    pub unlocked: usize,
    pub gifts: Gifts,
    pub settlements: Vec<Option<Settlement>>,
}

impl Campaign {
    fn new(regions: &[Region]) -> Self {
        let mut settlements = vec![None; regions.len()];
        settlements[0] = Some(Settlement::new(&regions[0]));
        Self {
            version: SAVE_VERSION,
            active: 0,
            unlocked: 0,
            gifts: Gifts::default(),
            settlements,
        }
    }

    pub fn restore(mut self, regions: &[Region]) -> Option<Self> {
        if self.version == 1 {
            for (city, region) in self.settlements.iter_mut().zip(regions) {
                if let Some(city) = city {
                    if city.nodes.len() != ORIGINAL_SOURCE_COUNT
                        || city.regrowth.len() != ORIGINAL_SOURCE_COUNT
                    {
                        return None;
                    }
                    city.nodes.extend(
                        region
                            .sources
                            .iter()
                            .skip(ORIGINAL_SOURCE_COUNT)
                            .map(|source| source.stock),
                    );
                    city.regrowth.resize(region.sources.len(), 0.);
                }
            }
            self.version = SAVE_VERSION;
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
                            Job::Watch(tile) | Job::Tend(tile) => region.bounds.contains(tile),
                        };
                        city.nodes.len() == region.sources.len()
                            && city.regrowth.len() == region.sources.len()
                            && city.guards.len() == region.guards.len()
                            && (2..=MAX_WALKERS).contains(&city.walkers.len())
                            && city.walkers[0].hippo
                            && city.last_market < 2
                            && city.energy.is_finite()
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
                                    && walker.bundle.is_none_or(valid_bundle)
                                    && walker.route.iter().all(|&p| region.bounds.contains(p))
                                    && walker.job.is_none_or(valid_job)
                            })
                    })
                })
    }
}

#[derive(Resource)]
pub struct Game {
    pub regions: Vec<Region>,
    pub campaign: Campaign,
}

impl Game {
    pub fn new(regions: Vec<Region>, saved: Option<Campaign>) -> Self {
        let campaign = saved
            .and_then(|save| save.restore(&regions))
            .unwrap_or_else(|| Campaign::new(&regions));
        Self { regions, campaign }
    }

    pub fn region(&self) -> &Region {
        &self.regions[self.campaign.active]
    }

    pub fn city(&self) -> &Settlement {
        self.campaign.settlements[self.campaign.active]
            .as_ref()
            .unwrap()
    }

    pub fn act(&mut self, action: Action) -> GameResult {
        let index = self.campaign.active;
        let settlement = self.campaign.settlements[index].as_mut().unwrap();
        let region = &self.regions[index];
        let gifts = &mut self.campaign.gifts;
        let mut text = match action {
            Action::Hand(target) => settlement.hand(region, target),
            Action::Build(kind, tile, rotation) => settlement.build(region, kind, tile, rotation),
            Action::Road(start, end) => settlement.build_road(region, start, end),
            Action::Upgrade(tile) => settlement.upgrade(region, tile),
            Action::Settle(index) => settlement
                .claim_district(region, index)
                .inspect(|_| gifts.earn()),
            Action::Assign(tile, hippo) => settlement.assign(region, tile, hippo),
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
        city.tick(region, self.campaign.gifts, dt);
        self.advance_quests();
    }

    fn advance_quests(&mut self) -> bool {
        let index = self.campaign.active;
        let city = self.campaign.settlements[index].as_mut().unwrap();
        let region = &self.regions[index];
        let main = city.advance_quest(region);
        if main {
            self.campaign.gifts.earn();
            if city.quest == Quest::Complete {
                self.campaign.unlocked = self
                    .campaign
                    .unlocked
                    .max((index + 1).min(self.regions.len() - 1));
            }
        }
        if !city.local_finished
            && city.quest.index() >= Quest::Bridge.index()
            && city.local_quest(region).2
        {
            city.local_finished = true;
            city.faith = city.faith.saturating_add(6);
            self.campaign.gifts.earn();
            return true;
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

    fn fresh() -> Game {
        Game::new(Region::campaign(), None)
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
        assert_eq!(game.city().quest, Quest::Bridge);
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
        let from = district.center.offset(-2, 2);
        let to = from.offset(0, 2);
        stock_for(game, [0, 3, 0]);
        let (cells, price) = game.city().road_plan(&region, from, to).unwrap();
        let stock = game.city().stock;
        game.act(Action::Road(from, to)).unwrap();
        assert_eq!(game.city().stock[1], stock[1] - price[1]);
        assert!(game.act(Action::Road(from, to)).is_err());
        stock_for(game, kind.price());
        let site = region
            .bounds
            .tiles()
            .filter(|tile| tile.within(district.center, 5))
            .filter(|&tile| {
                kind.tiles(tile)
                    .flat_map(Tile::neighbors)
                    .any(|cell| cells.contains(&cell))
            })
            .find(|&tile| game.city().placement(&region, kind, tile).is_ok())
            .unwrap_or_else(|| panic!("no road-connected {} near {}", kind.name(), district.name));
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
        let city = legacy["settlements"][0].as_object_mut().unwrap();
        for key in [
            "roads",
            "districts",
            "local_finished",
            "energy",
            "recovering",
            "cooldowns",
            "last_market",
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
}
