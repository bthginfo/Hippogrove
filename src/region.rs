use std::collections::VecDeque;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{modes::Layout, quests::CampaignQuests, terrain};

mod geography;
mod islands;
use geography::Geography;
pub use geography::WorldLayout;
use islands::Islands;

const ORIGIN: Vec2 = Vec2::new(17., 13.);
pub const CELL: f32 = 1.65;
const WET: u8 = 1;
const ROCK: u8 = 2;
const CROSSING: u8 = 4;
const OCCUPIED: u8 = 8;
pub const AREA_COUNT: usize = 3;
pub const AREA_RADIUS: i16 = 8;

#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct Tile(pub i16, pub i16);

impl Tile {
    pub fn world(self) -> Vec3 {
        Vec3::new(
            (f32::from(self.0) - ORIGIN.x) * CELL,
            0.0,
            (f32::from(self.1) - ORIGIN.y) * CELL,
        )
    }

    pub fn local(self) -> Vec2 {
        Vec2::new(f32::from(self.0), f32::from(self.1))
    }

    pub fn from_world(point: Vec3) -> Self {
        let local = Self::coordinates(point);
        Self(local.x.round() as i16, local.y.round() as i16)
    }

    pub fn coordinates(world: Vec3) -> Vec2 {
        Vec2::new(world.x, world.z) / CELL + ORIGIN
    }

    pub fn world_point(local: Vec2) -> Vec3 {
        let point = (local - ORIGIN) * CELL;
        Vec3::new(point.x, 0., point.y)
    }

    pub fn offset(self, x: i16, z: i16) -> Self {
        Self(self.0.saturating_add(x), self.1.saturating_add(z))
    }

    pub fn neighbors(self) -> [Self; 4] {
        [
            self.offset(1, 0),
            self.offset(-1, 0),
            self.offset(0, 1),
            self.offset(0, -1),
        ]
    }

    pub fn within(self, other: Self, radius: i16) -> bool {
        let x = i32::from(self.0) - i32::from(other.0);
        let z = i32::from(self.1) - i32::from(other.1);
        let radius = i32::from(radius);
        x.abs() <= radius && z.abs() <= radius && x * x + z * z <= radius * radius
    }

    pub fn distance(self, other: Self) -> u16 {
        self.0
            .abs_diff(other.0)
            .saturating_add(self.1.abs_diff(other.1))
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Bounds(pub Tile, pub Tile);

impl Bounds {
    pub fn width(self) -> usize {
        usize::from(self.0.0.abs_diff(self.1.0)) + 1
    }

    pub fn len(self) -> usize {
        self.width() * (usize::from(self.0.1.abs_diff(self.1.1)) + 1)
    }

    pub fn index(self, tile: Tile) -> Option<usize> {
        self.contains(tile)
            .then(|| (tile.1 - self.0.1) as usize * self.width() + (tile.0 - self.0.0) as usize)
    }

    pub fn contains(self, tile: Tile) -> bool {
        (self.0.0..=self.1.0).contains(&tile.0) && (self.0.1..=self.1.1).contains(&tile.1)
    }

    pub fn tiles(self) -> impl Iterator<Item = Tile> {
        (self.0.1..=self.1.1).flat_map(move |z| (self.0.0..=self.1.0).map(move |x| Tile(x, z)))
    }

    pub fn padded(self, padding: i16) -> Self {
        Self(
            self.0.offset(-padding, -padding),
            self.1.offset(padding, padding),
        )
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Material {
    Wood,
    Stone,
    Food,
}

impl Material {
    pub const ALL: [Self; 3] = [Self::Wood, Self::Stone, Self::Food];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Wood => "Holz",
            Self::Stone => "Stein",
            Self::Food => "Nahrung",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Biome {
    Valley,
    Fjord,
    Terraces,
    Lake,
    Aegean,
}

impl Biome {
    pub const ALL: [Self; 5] = [
        Self::Valley,
        Self::Fjord,
        Self::Terraces,
        Self::Lake,
        Self::Aegean,
    ];

    pub fn theme(self) -> &'static str {
        match self {
            Self::Valley => "Europäisches Weidenland",
            Self::Fjord => "Nordischer Fjord",
            Self::Terraces => "Ostasiatische Terrassen",
            Self::Lake => "Afrikanisches Seeufer",
            Self::Aegean => "Griechische Ägäis",
        }
    }

    pub fn material(self, material: Material) -> &'static str {
        [
            ["Eichenholz", "Flussstein", "Waldbeeren"],
            ["Kiefernholz", "Schiefer", "Frostbeeren"],
            ["Bambus", "Jade", "Reis"],
            ["Akazienholz", "Sandstein", "Datteln"],
            ["Zypressenholz", "Kalkstein", "Oliven"],
        ][self as usize][material.index()]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Weather {
    Clear,
    Showers,
    Snow,
    Mist,
    Drought,
    Breeze,
}

impl Weather {
    pub fn name(self) -> &'static str {
        match self {
            Self::Clear => "Klar",
            Self::Showers => "Landregen",
            Self::Snow => "Schneefall",
            Self::Mist => "Flussnebel",
            Self::Drought => "Trockenzeit",
            Self::Breeze => "Seebrise",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Source {
    pub tile: Tile,
    pub material: Material,
    pub stock: u16,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Ridgeline(pub Tile, pub Tile, pub f32, pub f32);

impl Ridgeline {
    pub fn section(self, point: Vec2) -> (f32, f32) {
        let Self(start, end, width, _) = self;
        let from_start = point - start.local();
        let axis = end.local() - start.local();
        let length = (from_start.dot(axis) / axis.length_squared()).clamp(0., 1.);
        let away = (from_start - axis * length).length() / width;
        ((1. - away).max(0.), length)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ResourceArea {
    pub name: String,
    pub center: Tile,
    pub material: Material,
}

#[derive(Deserialize)]
struct Frontier {
    bounds: Bounds,
    ridges: Vec<Ridgeline>,
    areas: [ResourceArea; AREA_COUNT],
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub enum DistrictNeed {
    Water,
    Trade,
    Shelter,
    Patrol,
}

impl DistrictNeed {
    pub fn building(self) -> crate::game::Building {
        use crate::game::Building;
        match self {
            Self::Water => Building::Well,
            Self::Trade => Building::Market,
            Self::Shelter => Building::House,
            Self::Patrol => Building::Tower,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct District {
    pub name: String,
    pub story: String,
    pub center: Tile,
    pub need: DistrictNeed,
    pub gift: [u16; 3],
}

#[derive(Clone, Debug, Deserialize)]
pub struct Region {
    pub name: String,
    pub kind: Biome,
    pub subtitle: String,
    pub lore: String,
    pub rival: String,
    pub crossing: String,
    pub request: String,
    pub gift: (Material, u16),
    pub rebuild: [u16; 3],
    pub renown: u16,
    pub faith: u16,
    pub bounds: Bounds,
    pub start: Tile,
    pub depot: Tile,
    pub frontier: Tile,
    pub rival_shrine: Tile,
    pub cross_x: i16,
    pub river: i16,
    pub water_width: i16,
    pub crossings: [i16; 2],
    pub initial: [u16; 3],
    #[serde(default)]
    pub starter_plots: Vec<Tile>,
    pub highlands: Vec<(Tile, i16)>,
    pub guards: Vec<Tile>,
    pub sources: Vec<Source>,
    pub districts: [District; 2],
    #[serde(skip)]
    original_bounds: Option<Bounds>,
    #[serde(skip)]
    pub original_sources: usize,
    #[serde(skip)]
    pub ridges: Vec<Ridgeline>,
    #[serde(skip)]
    pub areas: Vec<ResourceArea>,
    #[serde(skip)]
    pub sandbox: bool,
    #[serde(skip)]
    pub tutorial: bool,
    #[serde(skip)]
    pub quests: Option<&'static CampaignQuests>,
    #[serde(skip)]
    layout: Layout,
    #[serde(skip)]
    geography: Option<Geography>,
    #[serde(skip)]
    islands: Option<Islands>,
    #[serde(skip)]
    world_layout: WorldLayout,
    #[serde(skip)]
    cells: Vec<u8>,
    #[serde(skip)]
    source_grid: Vec<u16>,
}

impl Region {
    fn authored() -> (Vec<Self>, Vec<Frontier>) {
        (
            serde_json::from_str(include_str!("../assets/world/regions.json"))
                .expect("The embedded region definitions must be valid"),
            serde_json::from_str(include_str!("../assets/world/frontiers.json"))
                .expect("The embedded frontier definitions must be valid"),
        )
    }

    pub fn campaign() -> Vec<Self> {
        let (mut regions, frontiers) = Self::authored();
        assert_eq!(regions.len(), frontiers.len());
        for (region, frontier) in regions.iter_mut().zip(frontiers) {
            region.extend(frontier, WorldLayout::Classic);
        }
        regions
    }

    pub fn practice() -> Self {
        let (mut regions, _) = Self::authored();
        let mut region = regions.remove(0);
        region.name = "Übungsinsel".into();
        region.subtitle = "Die ersten Schritte".into();
        region.lore = "Ein geschützter Garten für deine ersten Schritte als Inselgott.".into();
        region.tutorial = true;
        region.bounds = Bounds(Tile(1, 5), Tile(33, 32));
        region.start = Tile(17, 20);
        region.depot = Tile(14, 21);
        region.frontier = Tile(19, 15);
        region.cross_x = 18;
        region.river = 10;
        region.initial = [13, 12, 6];
        region.starter_plots.clear();
        region.guards.clear();
        region.highlands = vec![(Tile(13, 12), 4), (Tile(24, 12), 3)];
        region.sources = [
            (Tile(12, 18), Material::Wood, 52),
            (Tile(19, 16), Material::Wood, 44),
            (Tile(22, 22), Material::Stone, 54),
            (Tile(11, 22), Material::Stone, 50),
            (Tile(16, 25), Material::Food, 44),
        ]
        .into_iter()
        .map(|(tile, material, stock)| Source {
            tile,
            material,
            stock,
        })
        .collect();
        region.original_sources = region.sources.len();
        region.cache_terrain();
        region
    }

    pub fn themed(biome: Biome, stage: usize, world_layout: WorldLayout) -> Self {
        let (mut regions, mut frontiers) = Self::authored();
        let mut region = regions.remove(biome as usize);
        let mut frontier = frontiers.remove(biome as usize);
        let layout = Layout::for_stage(stage, biome);
        if stage != 0 {
            let transform = |tile| layout.tile(tile);
            let bounds = |bounds: Bounds| {
                let a = transform(bounds.0);
                let b = transform(bounds.1);
                Bounds(Tile(a.0.min(b.0), a.1), Tile(a.0.max(b.0), b.1))
            };
            region.layout = layout;
            region.bounds = bounds(region.bounds);
            region.start = transform(region.start);
            region.depot = transform(region.depot);
            region.frontier = transform(region.frontier);
            region.rival_shrine = transform(region.rival_shrine);
            region.cross_x = transform(Tile(region.cross_x, 0)).0;
            region.crossings = region.crossings.map(|x| transform(Tile(x, 0)).0);
            region.river += layout.shift.1;
            region.initial[0] += 4 * stage as u16;
            for (tile, _) in &mut region.highlands {
                *tile = transform(*tile);
            }
            for tile in &mut region.guards {
                *tile = transform(*tile);
            }
            for source in &mut region.sources {
                source.tile = transform(source.tile);
            }
            for district in &mut region.districts {
                district.center = transform(district.center);
            }
            if region.starter_plots.is_empty() {
                region.starter_plots = vec![Tile(19, 18), Tile(12, 17), Tile(20, 21)];
            }
            for tile in &mut region.starter_plots {
                *tile = layout.plot(*tile);
            }
            frontier.bounds = bounds(frontier.bounds);
            for (i, ridge) in frontier.ridges.iter_mut().enumerate() {
                let stagger = (i % 3) as i16 - 1;
                ridge.0 = transform(ridge.0).offset(stagger * 2, if stage == 1 { 3 } else { -4 });
                ridge.1 = transform(ridge.1).offset(stagger * -3, if stage == 1 { -5 } else { 4 });
                ridge.2 *= if stage == 1 { 0.91 } else { 1.12 };
                ridge.3 = (ridge.3 + f32::from(stagger) * 3.).min(40.);
            }
            for area in &mut frontier.areas {
                area.center = transform(area.center);
            }
        }
        region.extend(frontier, world_layout);
        if stage > 0 || world_layout != WorldLayout::Classic {
            region.keep_starter_plots_clear();
        }
        region
    }

    fn keep_starter_plots_clear(&mut self) {
        let wants = std::mem::take(&mut self.starter_plots);
        let reserved_shrine = self.geography.is_some().then_some(self.frontier);
        let residents = [
            self.depot.offset(0, 1),
            self.start.offset(-1, 0),
            self.depot.offset(-1, 0),
        ];
        let plot = |tile: Tile| {
            [
                tile,
                tile.offset(1, 0),
                tile.offset(0, 1),
                tile.offset(1, 1),
            ]
        };
        for preferred in wants {
            let site = self
                .bounds
                .tiles()
                .filter(|tile| tile.within(self.start, 7))
                .filter(|&tile| {
                    plot(tile).into_iter().all(|cell| {
                        cell.within(self.start, 7)
                            && !self.water(cell)
                            && !self.rocky(cell)
                            && !self.road(cell)
                            && !self.fixed_structure(cell)
                            && Some(cell) != reserved_shrine
                            && !residents.contains(&cell)
                            && self.source_at(cell).is_none()
                            && self
                                .starter_plots
                                .iter()
                                .all(|&existing| !plot(existing).contains(&cell))
                    })
                })
                .min_by_key(|&tile| tile.distance(preferred))
                .expect("A themed chapter needs three starting plots");
            self.starter_plots.push(site);
        }
    }

    pub fn original_bounds(&self) -> Bounds {
        self.original_bounds.unwrap_or(self.bounds)
    }

    pub fn chapter(&self) -> usize {
        self.layout.stage
    }

    pub fn world_layout(&self) -> WorldLayout {
        self.world_layout
    }

    pub fn island_world(&self) -> bool {
        self.tutorial || self.islands.is_some() || self.kind == Biome::Aegean
    }

    pub fn render_bounds(&self) -> Bounds {
        let Some(islands) = &self.islands else {
            return self.bounds;
        };
        let Bounds(from, to) = islands.bounds();
        let a = self.layout.tile(from);
        let b = self.layout.tile(to);
        Bounds(
            Tile(self.bounds.0.0.min(a.0.min(b.0)), self.bounds.0.1.min(a.1)),
            Tile(self.bounds.1.0.max(a.0.max(b.0)), self.bounds.1.1.max(b.1)),
        )
    }

    pub fn has_rival_creature(&self) -> bool {
        !self.sandbox && self.world_layout == WorldLayout::Islands
    }

    pub fn rival_lair(&self) -> Tile {
        let offset = if self.layout.mirror { -1 } else { 1 };
        [(2, -1), (-2, 1), (1, -2), (-1, 0), (0, 2)]
            .into_iter()
            .map(|(x, z)| self.rival_shrine.offset(x * offset, z))
            .find(|&tile| !self.water(tile) && !self.rocky(tile) && !self.fixed_structure(tile))
            .unwrap_or(self.rival_shrine)
    }

    fn extend(&mut self, frontier: Frontier, world_layout: WorldLayout) {
        self.original_bounds = Some(self.bounds);
        self.original_sources = self.sources.len();
        self.bounds = frontier.bounds;
        self.ridges = frontier.ridges;
        self.areas = frontier.areas.into();
        self.world_layout = world_layout;
        if world_layout != WorldLayout::Classic {
            self.geography = Some(Geography::apply_to(self));
        }
        if world_layout == WorldLayout::Islands {
            self.islands = Some(Islands::for_region(self));
        }
        self.cache_terrain();
        let mut accessible = vec![false; self.bounds.len()];
        let walk = self.walk_grid(true);
        let mut queue: VecDeque<_> = self
            .start
            .neighbors()
            .into_iter()
            .filter(|&tile| self.bounds.index(tile).is_some_and(|index| walk[index]))
            .collect();
        while let Some(tile) = queue.pop_front() {
            let index = self.bounds.index(tile).unwrap();
            if accessible[index] {
                continue;
            }
            accessible[index] = true;
            queue.extend(tile.neighbors().into_iter().filter(|&next| {
                self.bounds
                    .index(next)
                    .is_some_and(|index| walk[index] && !accessible[index])
            }));
        }
        for area in self.areas.clone() {
            let mut sites: Vec<_> = (-7..=7)
                .flat_map(|z| (-7..=7).map(move |x| (x, z)))
                .map(|(x, z)| area.center.offset(x, z))
                .filter(|tile| tile.within(area.center, 7) && !tile.within(area.center, 2))
                .collect();
            sites.sort_by_key(|&Tile(x, z)| terrain::hash(x, z, 391));
            let mut added = 0;
            for tile in sites {
                if !self.free_source_tile(tile, &accessible) {
                    continue;
                }
                let shift = match added % 4 {
                    2 => 1,
                    3 => 2,
                    _ => 0,
                };
                let material = Material::ALL[(area.material.index() + shift) % Material::ALL.len()];
                self.add_source(tile, material);
                added += 1;
                if added == 12 {
                    break;
                }
            }
        }
        let spacing = if world_layout == WorldLayout::Islands {
            24
        } else {
            53
        };
        let mut wildlands: Vec<_> = self
            .bounds
            .tiles()
            .filter(|tile| !self.original_bounds().contains(*tile))
            .filter(|tile| !self.areas.iter().any(|area| tile.within(area.center, 10)))
            .filter(|&Tile(x, z)| terrain::hash(x, z, 297) % spacing == 0)
            .collect();
        wildlands.sort_by_key(|&Tile(x, z)| terrain::hash(x, z, 593));
        for tile in wildlands {
            if self.sources.len() >= self.original_sources + 100 {
                break;
            }
            if self.free_source_tile(tile, &accessible) {
                let material = match terrain::hash(tile.0, tile.1, 271) % 7 {
                    0..=2 => Material::Wood,
                    3..=5 => Material::Stone,
                    _ => Material::Food,
                };
                self.add_source(tile, material);
            }
        }
    }

    fn cache_terrain(&mut self) {
        self.cells = self
            .bounds
            .tiles()
            .map(|tile| {
                let wet = self.sample_water(tile);
                u8::from(wet) * WET
                    | u8::from(self.sample_rock(tile)) * ROCK
                    | u8::from(wet && self.bridge_cell(tile)) * CROSSING
                    | u8::from(self.fixed_structure(tile)) * OCCUPIED
            })
            .collect();
        self.source_grid = vec![0; self.bounds.len()];
        for (index, source) in self.sources.iter().enumerate() {
            let cell = self
                .bounds
                .index(source.tile)
                .expect("Authored sources fit the map");
            self.source_grid[cell] = index as u16 + 1;
            self.cells[cell] |= OCCUPIED;
        }
    }

    fn free_source_tile(&self, tile: Tile, accessible: &[bool]) -> bool {
        let bridge_approach = self.world_layout == WorldLayout::Islands && self.crossing(tile) && {
            let bank = self.river_at(tile.0);
            (bank - 2..=bank + self.river_span(tile.0) + 1).contains(&tile.1)
        };
        !bridge_approach
            && !self.original_bounds().contains(tile)
            && !self.road(tile)
            && self
                .bounds
                .index(tile)
                .is_some_and(|index| accessible[index] && self.cells[index] == 0)
            && tile
                .neighbors()
                .into_iter()
                .filter(|&next| {
                    self.bounds
                        .index(next)
                        .is_some_and(|index| accessible[index] && self.cells[index] == 0)
                })
                .count()
                >= 3
    }

    fn add_source(&mut self, tile: Tile, material: Material) {
        let cell = self.bounds.index(tile).unwrap();
        let stock = 34 + (terrain::hash(tile.0, tile.1, 101) % 51) as u16;
        self.sources.push(Source {
            tile,
            material,
            stock,
        });
        self.cells[cell] |= OCCUPIED;
        self.source_grid[cell] = self.sources.len() as u16;
    }

    pub fn walk_grid(&self, bridge_fixed: bool) -> Vec<bool> {
        self.cells
            .iter()
            .map(|&cell| {
                cell & (ROCK | OCCUPIED) == 0
                    && (cell & WET == 0 || (bridge_fixed && cell & CROSSING != 0))
            })
            .collect()
    }

    pub fn area_at(&self, tile: Tile) -> Option<usize> {
        self.areas.iter().position(|area| area.center == tile)
    }

    pub fn area_sources(
        &self,
        area: usize,
        material: Material,
    ) -> impl Iterator<Item = (usize, &Source)> {
        let center = self.areas.get(area).map(|area| area.center);
        self.sources.iter().enumerate().filter(move |(_, source)| {
            source.material == material
                && center.is_some_and(|center| source.tile.within(center, AREA_RADIUS))
        })
    }

    pub fn weather(&self, time: f32) -> Weather {
        let offset = match self.kind {
            Biome::Valley => 0.,
            Biome::Fjord => 63.,
            Biome::Terraces => 47.,
            Biome::Lake => 70.,
            Biome::Aegean => 83.,
        };
        if (time + offset).rem_euclid(118.) < 65. {
            return Weather::Clear;
        }
        match self.kind {
            Biome::Valley => Weather::Showers,
            Biome::Fjord => Weather::Snow,
            Biome::Terraces => Weather::Mist,
            Biome::Lake => Weather::Drought,
            Biome::Aegean => Weather::Breeze,
        }
    }

    pub fn river_at(&self, column: i16) -> i16 {
        self.geography.as_ref().map_or_else(
            || self.classic_river_at(column),
            |shape| shape.channel(column).0,
        )
    }

    fn classic_river_at(&self, column: i16) -> i16 {
        let column = self.layout.original_x(column);
        let delta = column - self.layout.original_x(self.cross_x);
        let outer = f32::from((column - 33).max(0) - (-column).max(0));
        let bend = ((outer * 0.22).sin() * (outer.abs() * 0.22).min(3.3)).round() as i16;
        let lowlands = if self.layout.stage == 0 {
            0
        } else {
            let step = (delta.abs() - 24).clamp(0, 31);
            ((f32::from(column) * 0.14 + self.layout.stage as f32).sin() * f32::from(step) * 0.085)
                .round() as i16
        };
        self.river
            + bend
            + lowlands
            + match self.kind {
                Biome::Valley => (delta / 6).clamp(-2, 2),
                Biome::Fjord => (delta.abs() / 7).min(2),
                Biome::Terraces => -(delta / 5).clamp(-2, 2),
                Biome::Lake => -(delta.abs() / 6).min(2),
                Biome::Aegean => (delta / 8).clamp(-3, 3),
            }
    }

    pub fn water(&self, tile: Tile) -> bool {
        self.bounds
            .index(tile)
            .and_then(|i| self.cells.get(i))
            .map_or_else(|| self.sample_water(tile), |flags| flags & WET != 0)
    }

    fn sample_water(&self, tile: Tile) -> bool {
        if self.tutorial || self.geography.is_some() {
            return terrain::shore(self, tile.local()) < 0.;
        }
        self.in_channel(tile)
            || (self.kind == Biome::Aegean && self.island_shore(tile.local()) <= 0.)
    }

    fn in_channel(&self, tile: Tile) -> bool {
        let bank = self.river_at(tile.0);
        (bank..bank + self.river_span(tile.0)).contains(&tile.1)
    }

    pub fn island_shore(&self, place: Vec2) -> f32 {
        if self.tutorial {
            let shore = (place - Vec2::new(17., 19.)) / Vec2::new(12., 9.5);
            let bay = (place - Vec2::new(26.7, 14.3)) / Vec2::new(5., 3.2);
            return ((1. - shore.length_squared()) * 4.75).min((bay.length_squared() - 1.) * 1.60);
        }
        if let Some(islands) = &self.islands {
            return islands.shore(self.layout.original_point(place));
        }
        let islets = self
            .geography
            .as_ref()
            .map_or(f32::NEG_INFINITY, |shape| shape.islet_shore(place));
        let place = self.layout.original_point(place);
        let coast = (place.x * 0.31 + place.y * 0.45).sin() * 0.075;
        [
            (Vec2::new(17., -9.), Vec2::new(54., 29.)),
            (Vec2::new(24., 27.), Vec2::new(52., 29.)),
            (Vec2::new(-27., 35.), Vec2::new(12., 11.)),
            (Vec2::new(65., -25.), Vec2::new(14., 10.)),
            (Vec2::new(-34., -12.), Vec2::new(28., 27.)),
            (Vec2::new(77., -14.), Vec2::new(30., 28.)),
            (Vec2::new(-33., 32.), Vec2::new(27., 26.)),
            (Vec2::new(74., 36.), Vec2::new(33., 28.)),
            (Vec2::new(19., 51.), Vec2::new(40., 26.)),
            (Vec2::new(53., 49.), Vec2::new(39., 22.)),
        ]
        .into_iter()
        .map(|(center, radius)| {
            (1. + coast - ((place - center) / radius).length_squared()) * radius.min_element() * 0.5
        })
        .fold(islets, f32::max)
    }

    pub fn basin_shore(&self, point: Vec2) -> f32 {
        self.geography
            .as_ref()
            .map_or(f32::INFINITY, |shape| shape.basin_shore(self.kind, point))
    }

    pub fn rocky(&self, tile: Tile) -> bool {
        self.bounds
            .index(tile)
            .and_then(|i| self.cells.get(i))
            .map_or_else(|| self.sample_rock(tile), |flags| flags & ROCK != 0)
    }

    fn sample_rock(&self, tile: Tile) -> bool {
        self.highlands
            .iter()
            .any(|&(center, radius)| tile.within(center, radius))
            || (!self.original_bounds().contains(tile)
                && self.ridges.iter().any(|ridge| {
                    let (depth, _) = ridge.section(tile.local());
                    depth * depth * ridge.3 >= 5.1
                }))
    }

    pub fn river_span(&self, column: i16) -> i16 {
        self.geography.as_ref().map_or_else(
            || self.classic_river_span(column),
            |shape| shape.channel(column).1,
        )
    }

    fn classic_river_span(&self, column: i16) -> i16 {
        let column = self.layout.original_x(column);
        let outer = (column - 33).max(-column).max(0);
        self.water_width
            + match self.kind {
                Biome::Valley => (outer / 11).min(2),
                Biome::Fjord => (outer / 5).min(6),
                Biome::Terraces => (outer / 19).min(2),
                Biome::Lake => (outer / 3).min(11),
                Biome::Aegean => (outer / 10).min(5),
            }
    }

    pub fn bridge(&self) -> Tile {
        Tile(self.cross_x, self.river + self.water_width - 1)
    }

    pub fn opening_farm(&self) -> Option<Tile> {
        (self.kind == Biome::Valley
            && self.chapter() == 0
            && self.island_world()
            && self.quests.is_some())
        .then_some(self.rival_shrine.offset(-5, 0))
    }

    pub fn crossing(&self, tile: Tile) -> bool {
        !self.tutorial && (tile.0 == self.cross_x || self.crossings.contains(&tile.0))
    }

    pub fn bridge_cell(&self, tile: Tile) -> bool {
        self.crossing(tile) && self.in_channel(tile)
    }

    pub fn district_at(&self, tile: Tile) -> Option<usize> {
        self.districts
            .iter()
            .position(|district| !self.tutorial && district.center == tile)
    }

    pub fn district_houses(&self, index: usize) -> [Tile; 3] {
        let center = self.districts[index].center;
        [
            center.offset(-3, 0),
            center.offset(3, -2),
            center.offset(0, -4),
        ]
    }

    pub fn fixed_structure(&self, tile: Tile) -> bool {
        if self.tutorial {
            return [self.start, self.depot].contains(&tile);
        }
        [self.start, self.depot, self.rival_shrine].contains(&tile)
            || self.area_at(tile).is_some()
            || self.rival_buildings().contains(&tile)
            || self.district_at(tile).is_some()
            || (0..self.districts.len()).any(|index| self.district_houses(index).contains(&tile))
    }

    pub fn road(&self, tile: Tile) -> bool {
        if self.tutorial {
            return (tile.0 == self.cross_x && (17..=24).contains(&tile.1))
                || (tile.1 == self.depot.1 && (self.depot.0..self.cross_x).contains(&tile.0));
        }
        let center_path =
            tile.0 == self.cross_x && (self.rival_shrine.1..=self.start.1 + 5).contains(&tile.1);
        let plaza_path = tile.1 == self.depot.1
            && (self.depot.0.min(self.cross_x)..=self.depot.0.max(self.cross_x)).contains(&tile.0);
        center_path
            || plaza_path
            || self.districts.iter().any(|district| {
                tile.1 == district.center.1 + 2 && tile.0.abs_diff(district.center.0) <= 4
            })
    }

    pub fn source_at(&self, tile: Tile) -> Option<usize> {
        let cell = self.bounds.index(tile)?;
        self.source_grid.get(cell)?.checked_sub(1).map(usize::from)
    }

    pub fn rival_buildings(&self) -> [Tile; 2] {
        [
            self.rival_shrine.offset(-3, -1),
            self.rival_shrine.offset(4, -2),
        ]
    }
}
