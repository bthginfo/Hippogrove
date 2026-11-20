use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const WIDTH: i16 = 34;
pub const DEPTH: i16 = 26;
pub const CELL: f32 = 1.65;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Tile(pub i16, pub i16);

impl Tile {
    pub fn world(self) -> Vec3 {
        Vec3::new(
            (f32::from(self.0) - f32::from(WIDTH) * 0.5) * CELL,
            0.0,
            (f32::from(self.1) - f32::from(DEPTH) * 0.5) * CELL,
        )
    }

    pub fn from_world(point: Vec3) -> Self {
        Self(
            (point.x / CELL + f32::from(WIDTH) * 0.5).round() as i16,
            (point.z / CELL + f32::from(DEPTH) * 0.5).round() as i16,
        )
    }

    pub fn offset(self, x: i16, z: i16) -> Self {
        Self(self.0 + x, self.1 + z)
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
        x * x + z * z <= i32::from(radius) * i32::from(radius)
    }

    pub fn inside_map(self) -> bool {
        (0..WIDTH).contains(&self.0) && (0..DEPTH).contains(&self.1)
    }

    pub fn index(self) -> usize {
        (self.1 * WIDTH + self.0) as usize
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
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Source {
    pub tile: Tile,
    pub material: Material,
    pub stock: u16,
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
    pub start: Tile,
    pub depot: Tile,
    pub frontier: Tile,
    pub rival_shrine: Tile,
    pub cross_x: i16,
    pub river: i16,
    pub water_width: i16,
    pub initial: [u16; 3],
    pub guards: Vec<Tile>,
    pub sources: Vec<Source>,
}

impl Region {
    pub fn campaign() -> Vec<Self> {
        serde_json::from_str(include_str!("../assets/world/regions.json"))
            .expect("The embedded region definitions must be valid")
    }

    pub fn river_at(&self, column: i16) -> i16 {
        let delta = column - self.cross_x;
        self.river
            + match self.kind {
                Biome::Valley => (delta / 6).clamp(-2, 2),
                Biome::Fjord => (delta.abs() / 7).min(2),
                Biome::Terraces => -(delta / 5).clamp(-2, 2),
                Biome::Lake => -(delta.abs() / 6).min(2),
            }
    }

    pub fn water(&self, tile: Tile) -> bool {
        let bank = self.river_at(tile.0);
        (bank..bank + self.water_width).contains(&tile.1)
    }

    pub fn bridge(&self) -> Tile {
        Tile(self.cross_x, self.river + self.water_width - 1)
    }

    pub fn road(&self, tile: Tile) -> bool {
        let center_path =
            tile.0 == self.cross_x && (self.rival_shrine.1..=self.start.1 + 5).contains(&tile.1);
        let plaza_path = tile.1 == self.depot.1
            && (self.depot.0.min(self.cross_x)..=self.depot.0.max(self.cross_x)).contains(&tile.0);
        center_path || plaza_path
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
