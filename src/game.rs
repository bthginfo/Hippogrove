use std::collections::VecDeque;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::region::{Biome, CELL, DEPTH, Material, Region, Tile, WIDTH};

pub const SAVE_VERSION: u32 = 1;
pub const START_RADIUS: i16 = 7;
pub const FRONTIER_RADIUS: i16 = 8;
const RIVAL_RADIUS: i16 = 6;
const TILES: usize = (WIDTH * DEPTH) as usize;

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
}

impl Building {
    pub const ALL: [Self; 6] = [
        Self::House,
        Self::Store,
        Self::Field,
        Self::Shrine,
        Self::Workshop,
        Self::Granary,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::House => "Hütte",
            Self::Store => "Lager",
            Self::Field => "Feld",
            Self::Shrine => "Grenzschrein",
            Self::Workshop => "Werkstatt",
            Self::Granary => "Speicher",
        }
    }

    pub fn size(self) -> i16 {
        if self == Self::Shrine { 1 } else { 2 }
    }

    pub fn price(self) -> Stock {
        match self {
            Self::House => [12, 3, 0],
            Self::Store => [9, 5, 0],
            Self::Field => [5, 2, 0],
            Self::Shrine => [7, 8, 0],
            Self::Workshop => [12, 9, 0],
            Self::Granary => [16, 10, 0],
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
}

impl BuildingSite {
    pub fn contains(&self, tile: Tile) -> bool {
        self.kind.tiles(self.tile).any(|p| p == tile)
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Quest {
    #[default]
    Shelter,
    Fields,
    Bridge,
    Complete,
}

impl Quest {
    pub fn name(self) -> &'static str {
        match self {
            Self::Shelter => "Ein Zuhause am Wasser",
            Self::Fields => "Die trockenen Felder",
            Self::Bridge => "Ein Dorf am anderen Ufer",
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
}

impl Miracle {
    pub const ALL: [Self; 3] = [Self::Rain, Self::Wind, Self::Ward];

    pub fn name(self) -> &'static str {
        match self {
            Self::Rain => "Regenschauer",
            Self::Wind => "Windstoß",
            Self::Ward => "Schutzglocke",
        }
    }

    pub fn cost(self) -> f32 {
        match self {
            Self::Rain => 5.,
            Self::Wind => 8.,
            Self::Ward => 7.,
        }
    }

    pub fn radius(self, gifts: Gifts) -> i16 {
        match self {
            Self::Rain => 2 + i16::from(gifts.rain),
            Self::Wind => 2,
            Self::Ward => 3,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Talent {
    Rain,
    Spring,
}

impl Talent {
    pub const ALL: [Self; 2] = [Self::Rain, Self::Spring];

    pub fn name(self) -> &'static str {
        match self {
            Self::Rain => "Weiter Regenkreis",
            Self::Spring => "Tiefe Wunderquelle",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Gifts {
    pub points: u16,
    pub rain: u8,
    pub spring: u8,
    pub training: u8,
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
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Action {
    Hand(Tile),
    Build(Building, Tile, u8),
    Assign(Tile, bool),
    Watch(Tile),
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
    pub lesson: Option<usize>,
    pub shield: f32,
    pub rainfall: f32,
    pub weather_time: f32,
    raid_clock: f32,
    provisions: f32,
    next_worker: usize,
}

impl Settlement {
    pub fn new(region: &Region) -> Self {
        Self {
            stock: region.initial,
            faith: 2,
            mana: 20.,
            buildings: Vec::new(),
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
            shield: 0.,
            rainfall: 0.,
            weather_time: 0.,
            raid_clock: 0.,
            provisions: 22.,
            next_worker: 0,
        }
    }

    pub fn count(&self, building: Building) -> usize {
        self.buildings
            .iter()
            .filter(|site| site.kind == building)
            .count()
    }

    pub fn city_rank(&self) -> u8 {
        if self.count(Building::Workshop) > 0 {
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
        structures + (food / 3).min(6) + self.harvests.min(3) * 2
    }

    pub fn storage_cap(&self) -> u16 {
        32 + self.count(Building::Store).min(20) as u16 * 24
            + self.count(Building::Granary).min(20) as u16 * 48
    }

    pub fn influence(&self, region: &Region, tile: Tile) -> bool {
        tile.inside_map()
            && (tile.within(region.start, START_RADIUS)
                || (self.count(Building::Shrine) > 0
                    && tile.within(region.frontier, FRONTIER_RADIUS))
                || (self.outcome.is_some() && tile.within(region.rival_shrine, RIVAL_RADIUS)))
    }

    pub fn building_at(&self, tile: Tile) -> Option<&BuildingSite> {
        self.buildings
            .iter()
            .find(|building| building.contains(tile))
    }

    pub fn walkable(&self, region: &Region, tile: Tile) -> bool {
        tile.inside_map()
            && (!region.water(tile) || (self.bridge_fixed && tile.0 == region.cross_x))
            && tile != region.start
            && tile != region.rival_shrine
            && tile != region.depot
            && !region.rival_buildings().contains(&tile)
            && region.source_at(tile).is_none()
            && self.building_at(tile).is_none()
    }

    pub fn can_store_at(&self, region: &Region, tile: Tile) -> bool {
        tile == region.depot
            || self
                .building_at(tile)
                .is_some_and(|site| matches!(site.kind, Building::Store | Building::Granary))
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
            Building::Workshop if self.city_rank() < 2 => {
                return Err("Die Werkstatt wird auf Dorfstufe 2 verfügbar.".into());
            }
            Building::Granary if self.city_rank() < 3 => {
                return Err("Baue eine Werkstatt, um den Speicher freizuschalten.".into());
            }
            _ => {}
        }
        for cell in kind.tiles(tile) {
            if !self.influence(region, cell) {
                return Err("Der ganze Grundriss muss im Einflussgebiet liegen.".into());
            }
            if !self.walkable(region, cell)
                || region.road(cell)
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
        let before = self.reachable(region, depot_edge, |_| false);
        let after = self.reachable(region, depot_edge, new_footprint);
        if region.sources.iter().any(|source| {
            let accessible = |mask: &[bool; TILES]| {
                source
                    .tile
                    .neighbors()
                    .into_iter()
                    .any(|p| p.inside_map() && mask[p.index()])
            };
            accessible(&before) && !accessible(&after)
        }) || self.walkers.iter().any(|walker| {
            let p = walker.tile();
            p.inside_map() && before[p.index()] && !after[p.index()]
        }) {
            return Err("Diese Stelle würde einen Sammelweg versperren.".into());
        }
        Ok(())
    }

    fn reachable(
        &self,
        region: &Region,
        start: Tile,
        blocked: impl Fn(Tile) -> bool,
    ) -> [bool; TILES] {
        let mut seen = [false; TILES];
        if !self.walkable(region, start) || blocked(start) {
            return seen;
        }
        let mut queue = VecDeque::from([start]);
        seen[start.index()] = true;
        while let Some(p) = queue.pop_front() {
            for next in p.neighbors() {
                if self.walkable(region, next) && !blocked(next) && !seen[next.index()] {
                    seen[next.index()] = true;
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
        if !start.inside_map() {
            return None;
        }
        let mut came_from = [None; TILES];
        came_from[start.index()] = Some(start);
        let mut queue = VecDeque::from([start]);
        let end = loop {
            let p = queue.pop_front()?;
            if (!beside && p == target) || (beside && target.neighbors().contains(&p)) {
                break p;
            }
            for next in p.neighbors() {
                if self.walkable(region, next) && came_from[next.index()].is_none() {
                    came_from[next.index()] = Some(p);
                    queue.push_back(next);
                }
            }
        };
        let mut route = VecDeque::new();
        let mut p = end;
        while p != start {
            route.push_front(p);
            p = came_from[p.index()]?;
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

    fn advance_walker(&mut self, walker: &mut Walker, region: &Region, gifts: Gifts, dt: f32) {
        walker.pause = (walker.pause - dt).max(0.);
        if walker.pause > 0. {
            return;
        }
        if let Some(step) = walker.route.front().copied() {
            let target = Vec2::new(f32::from(step.0), f32::from(step.1));
            let point = Vec2::from_array(walker.position);
            let delta = target - point;
            let distance = dt * if walker.hippo { 2.65 } else { 2.25 };
            if delta.length_squared() <= distance * distance {
                walker.position = target.to_array();
                walker.route.pop_front();
            } else {
                walker.position = (point + delta.normalize() * distance).to_array();
            }
            return;
        }

        if let Some(bundle) = walker.bundle {
            if let Some(route) = self.route_to(region, walker.tile(), region.depot, true) {
                if !route.is_empty() {
                    walker.route = route;
                    return;
                }
                if self.deposit(bundle, region).is_ok() {
                    walker.bundle = None;
                    walker.pause = 0.45;
                    if walker.hippo {
                        self.hippo_deliveries = self.hippo_deliveries.saturating_add(1);
                        self.lesson = Some(bundle.source);
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

        for (index, (stock, delay)) in self.nodes.iter_mut().zip(&mut self.regrowth).enumerate() {
            if *stock == 0 && !self.held.is_some_and(|bundle| bundle.source == index) {
                *delay += dt;
                if *delay >= 28. {
                    *stock = region.sources[index].stock;
                    *delay = 0.;
                }
            }
        }
        for site in &mut self.buildings {
            if let Crop::Growing(remaining) = site.crop {
                site.crop = if remaining <= dt {
                    Crop::Ripe
                } else {
                    Crop::Growing(remaining - dt)
                };
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
            let food = &mut self.stock[Material::Food.index()];
            if *food > 4 {
                *food -= 1;
            }
        }

        if self.raid_clock > 0. && self.outcome.is_none() {
            self.raid_clock -= dt;
            if self.raid_clock <= 0. {
                self.raid_clock = 21.;
                let protected = self.shield > 0.
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
            let amount = room.min(if region.kind == Biome::Terraces {
                12
            } else {
                10
            });
            if amount == 0 {
                return Err("Schaffe Platz im Lager für die Ernte.".into());
            }
            site.crop = Crop::Dry;
            self.stock[food] += amount;
            self.harvests = self.harvests.saturating_add(1);
            self.faith = self.faith.saturating_add(5);
            return Ok(format!(
                "{amount} Nahrung nach Hause gebracht. Das Dorf schöpft Glauben."
            ));
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
        });
        for walker in &mut self.walkers {
            walker.route.clear();
        }
        if kind == Building::House && self.walkers.len() < 7 {
            let arrival = region.depot.offset(1, 0);
            self.walkers.push(Walker::new(arrival, false));
        }
        self.faith = self
            .faith
            .saturating_add(if kind == Building::Shrine { 3 } else { 2 });
        Ok(format!("{} gebaut", kind.name()))
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
        if !self.walkable(region, tile)
            || self
                .route_to(region, self.walkers[0].tile(), tile, false)
                .is_none()
        {
            return Err("Wähle einen erreichbaren Platz für die Wache.".into());
        }
        let hippo = &mut self.walkers[0];
        hippo.job = Some(Job::Watch(tile));
        hippo.route.clear();
        Ok("Das Nilpferd hält an dieser Stelle Wache.".into())
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
            Miracle::Wind if self.quest != Quest::Bridge || self.count(Building::Shrine) == 0 => {
                Some("Baue zuerst den Grenzschrein.")
            }
            Miracle::Wind
                if !region
                    .guards
                    .iter()
                    .zip(&self.guards)
                    .any(|(&guard, &spirit)| spirit > 0 && guard.within(tile, radius)) =>
            {
                Some("Ziele mit dem Windkreis auf Wachen am anderen Ufer.")
            }
            Miracle::Ward if self.city_rank() < 3 => Some("Die Werkstatt öffnet das Schutzwunder."),
            _ => None,
        }
    }

    fn cast(&mut self, region: &Region, spell: Miracle, tile: Tile, gifts: Gifts) -> GameResult {
        if let Some(reason) = self.ritual_error(region, spell, tile, gifts) {
            return Err(reason.into());
        }
        self.mana = (self.mana - spell.cost()).max(0.);
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
            Miracle::Wind => {
                for (&guard, spirit) in region.guards.iter().zip(&mut self.guards) {
                    if guard.within(tile, radius) {
                        *spirit = spirit.saturating_sub(1);
                    }
                }
                if self.raid_clock <= 0. {
                    self.raid_clock = 21.;
                }
                Ok("Der Windstoß treibt die Wachen zurück.".into())
            }
            Miracle::Ward => {
                self.shield = 26.;
                Ok("Die Schutzglocke hält Überfälle ab.".into())
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
                    self.quest = Quest::Complete;
                    self.bridge_fixed = true;
                    self.guards.fill(0);
                    self.faith = self.faith.saturating_add(10);
                    true
                } else {
                    false
                }
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
            Quest::Complete => self.outcome.map(Outcome::name).unwrap_or("Kapitel abgeschlossen").into(),
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

    pub fn valid(&self, regions: &[Region]) -> bool {
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
                        city.nodes.len() == region.sources.len()
                            && city.regrowth.len() == region.sources.len()
                            && city.guards.len() == region.guards.len()
                            && (2..=7).contains(&city.walkers.len())
                            && city.walkers[0].hippo
                            && city.held.is_none_or(valid_bundle)
                            && city
                                .lesson
                                .is_none_or(|lesson| lesson < region.sources.len())
                            && city.walkers.iter().all(|walker| {
                                walker.tile().inside_map()
                                    && walker.bundle.is_none_or(valid_bundle)
                                    && walker.route.iter().all(|p| p.inside_map())
                                    && match walker.job {
                                        Some(Job::Gather(source)) => source < region.sources.len(),
                                        Some(Job::Watch(tile)) => tile.inside_map(),
                                        None => true,
                                    }
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
            .filter(|save| save.valid(&regions))
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
            Action::Assign(tile, hippo) => settlement.assign(region, tile, hippo),
            Action::Watch(tile) => settlement.watch(region, tile, *gifts),
            Action::Cast(spell, tile) => settlement.cast(region, spell, tile, *gifts),
            Action::Repair => settlement.repair(region),
            Action::Give => settlement.give(region),
            Action::Praise => {
                let source = settlement
                    .lesson
                    .take()
                    .ok_or("Lobe die Kreatur nach einer Lieferung.")?;
                let hippo = &mut settlement.walkers[0];
                if gifts.training < 3 {
                    gifts.training += 1;
                }
                if hippo.job.is_none() {
                    hippo.job = Some(Job::Gather(source));
                }
                Ok(format!(
                    "Nilpferd-Rang {}. Es trägt mehr und wiederholt seinen Auftrag.",
                    gifts.training
                ))
            }
            Action::Learn(talent) => {
                if gifts.points == 0 {
                    return Err("Erfülle Aufträge für Talentpunkte.".into());
                }
                let rank = match talent {
                    Talent::Rain => &mut gifts.rain,
                    Talent::Spring => &mut gifts.spring,
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
        if settlement.advance_quest(region) {
            gifts.earn();
            if settlement.outcome.is_some() {
                self.campaign.unlocked = self
                    .campaign
                    .unlocked
                    .max((index + 1).min(self.regions.len() - 1));
            }
            text.push_str(" · Auftrag erfüllt");
        }
        Ok(text)
    }

    pub fn step(&mut self, dt: f32) {
        let index = self.campaign.active;
        let city = self.campaign.settlements[index].as_mut().unwrap();
        let region = &self.regions[index];
        city.tick(region, self.campaign.gifts, dt);
        if city.advance_quest(region) {
            self.campaign.gifts.earn();
            if city.outcome.is_some() {
                self.campaign.unlocked = self
                    .campaign
                    .unlocked
                    .max((index + 1).min(self.regions.len() - 1));
            }
        }
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
            (0..DEPTH)
                .flat_map(|z| (0..WIDTH).map(move |x| Tile(x, z)))
                .filter(|&tile| tile.within(region.start, 5))
                .find(|&tile| game.city().placement(region, kind, tile).is_ok())
                .expect("a buildable plot near the starting shrine")
        };
        game.act(Action::Build(kind, tile, 0)).unwrap();
        tile
    }

    fn prepare_bridge(game: &mut Game) {
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
        assert_eq!(game.city().quest, Quest::Fields);
        let field = build(game, Building::Field);
        game.act(Action::Cast(Miracle::Rain, field)).unwrap();
        assert!(game.act(Action::Hand(field)).is_err());
        for _ in 0..264 {
            game.step(1. / 24.);
        }
        game.act(Action::Hand(field)).unwrap();
        assert_eq!(game.city().quest, Quest::Bridge);
        let guard = game.region().guards[0];
        assert!(game.act(Action::Cast(Miracle::Wind, guard)).is_err());
        build(game, Building::Shrine);
        assert!(game.city().influence(game.region(), guard));
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
        for target in region.guards {
            for _ in 0..2 {
                conquered.step(8.);
                if conquered.city().quest != Quest::Complete
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
        assert_eq!(conquered.campaign.unlocked, 1);
        conquered.act(Action::Learn(Talent::Rain)).unwrap();
        let old_count = conquered.city().buildings.len();
        conquered.act(Action::Travel(1)).unwrap();
        assert_eq!(conquered.city().quest, Quest::Shelter);
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
        }
    }
}
