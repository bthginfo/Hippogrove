use bevy::prelude::*;
use serde::{Deserialize, Serialize};

const ORIGIN: Vec2 = Vec2::new(17., 13.);
pub const CELL: f32 = 1.65;

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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub enum Biome {
    Valley,
    Fjord,
    Terraces,
    Lake,
    Aegean,
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub enum DistrictNeed {
    Water,
    Trade,
    Shelter,
    Patrol,
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
}

impl Region {
    pub fn campaign() -> Vec<Self> {
        serde_json::from_str(include_str!("../assets/world/regions.json"))
            .expect("The embedded region definitions must be valid")
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
        let delta = column - self.cross_x;
        let outer = f32::from((column - 33).max(0) - (-column).max(0));
        let bend = ((outer * 0.22).sin() * (outer.abs() * 0.22).min(3.3)).round() as i16;
        self.river
            + bend
            + match self.kind {
                Biome::Valley => (delta / 6).clamp(-2, 2),
                Biome::Fjord => (delta.abs() / 7).min(2),
                Biome::Terraces => -(delta / 5).clamp(-2, 2),
                Biome::Lake => -(delta.abs() / 6).min(2),
                Biome::Aegean => (delta / 8).clamp(-3, 3),
            }
    }

    pub fn water(&self, tile: Tile) -> bool {
        self.in_channel(tile)
            || (self.kind == Biome::Aegean
                && self.island_shore(Vec2::new(f32::from(tile.0), f32::from(tile.1))) <= 0.)
    }

    fn in_channel(&self, tile: Tile) -> bool {
        let bank = self.river_at(tile.0);
        (bank..bank + self.river_span(tile.0)).contains(&tile.1)
    }

    pub fn island_shore(&self, place: Vec2) -> f32 {
        let coast = (place.x * 0.31 + place.y * 0.45).sin() * 0.075;
        [
            (Vec2::new(17., -9.), Vec2::new(54., 29.)),
            (Vec2::new(24., 27.), Vec2::new(52., 29.)),
            (Vec2::new(-27., 35.), Vec2::new(12., 11.)),
            (Vec2::new(65., -25.), Vec2::new(14., 10.)),
        ]
        .into_iter()
        .map(|(center, radius)| {
            (1. + coast - ((place - center) / radius).length_squared()) * radius.min_element() * 0.5
        })
        .fold(f32::NEG_INFINITY, f32::max)
    }

    pub fn rocky(&self, tile: Tile) -> bool {
        self.highlands
            .iter()
            .any(|&(center, radius)| tile.within(center, radius))
    }

    pub fn river_span(&self, column: i16) -> i16 {
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

    pub fn crossing(&self, tile: Tile) -> bool {
        tile.0 == self.cross_x || self.crossings.contains(&tile.0)
    }

    pub fn bridge_cell(&self, tile: Tile) -> bool {
        self.crossing(tile) && self.in_channel(tile)
    }

    pub fn district_at(&self, tile: Tile) -> Option<usize> {
        self.districts
            .iter()
            .position(|district| district.center == tile)
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
        [self.start, self.depot, self.rival_shrine].contains(&tile)
            || self.rival_buildings().contains(&tile)
            || self.district_at(tile).is_some()
            || (0..self.districts.len()).any(|index| self.district_houses(index).contains(&tile))
    }

    pub fn road(&self, tile: Tile) -> bool {
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
        self.sources.iter().position(|source| source.tile == tile)
    }

    pub fn rival_buildings(&self) -> [Tile; 2] {
        [
            self.rival_shrine.offset(-3, -1),
            self.rival_shrine.offset(4, -2),
        ]
    }
}
