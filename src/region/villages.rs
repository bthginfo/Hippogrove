use super::*;

#[derive(Clone, Debug)]
pub struct Village {
    pub center: Tile,
    pub name: String,
    pub need: Material,
    pub houses: Vec<Tile>,
}

const NAMES: [[&str; 4]; 5] = [
    ["Lindenau", "Hirschquell", "Erlenwacht", "Birkenfurt"],
    ["Birkenvik", "Rabenholm", "Eiswacht", "Runenhavn"],
    ["Lotusfähre", "Kranichhain", "Seidenufer", "Mondterrasse"],
    ["Papyrushafen", "Akazienquell", "Ibisinsel", "Sonnenfurt"],
    [
        "Myrtenhafen",
        "Nereidenquell",
        "Lorbeerfels",
        "Delphinbucht",
    ],
];

impl Region {
    /// Keep the campaign's authored districts. Place the further villages and the opposite
    /// seat on connected land after all resource nodes have acquired their stable save indices.
    pub(super) fn lay_out_villages(&mut self, reachable: &[bool]) {
        self.villages = self
            .districts
            .iter()
            .map(|d| Village {
                center: d.center,
                name: d.name.clone(),
                need: d
                    .gift
                    .into_iter()
                    .enumerate()
                    .max_by_key(|(_, n)| *n)
                    .map_or(Material::Food, |(i, _)| Material::ALL[i]),
                houses: Vec::new(),
            })
            .collect();
        let bounds = self.bounds;
        let dry = |t: Tile| {
            bounds.index(t).is_some_and(|i| reachable[i])
                && !self.water(t)
                && !self.rocky(t)
                && !self.fixed_structure(t)
                && self.source_at(t).is_none()
        };
        let sites: Vec<_> = bounds
            .tiles()
            .filter(|&t| {
                dry(t)
                    && t.neighbors().into_iter().filter(|&n| dry(n)).count() >= 3
                    && !self.areas.iter().any(|a| t.within(a.center, 4))
                    && !self.districts.iter().any(|d| t.within(d.center, 7))
            })
            .collect();
        let dir = if self.layout.mirror { -1 } else { 1 };
        let seat_goal = self.start.offset((43 + 3 * self.kind as i16) * dir, -28);
        let seat = sites
            .iter()
            .copied()
            .filter(|t| t.distance(self.start) >= 40)
            .min_by_key(|t| t.distance(seat_goal))
            .expect("Island needs a distant rival seat");
        let goals = [
            self.start.offset(-13 * dir, 6),
            self.start.offset(28 * dir, -15),
            self.start.offset(-24 * dir, -13),
            self.start.offset(18 * dir, 27),
        ];
        let mut added = Vec::<Village>::new();
        let count = 2 + self.chapter();
        for (index, goal) in goals.into_iter().take(count).enumerate() {
            let center = sites
                .iter()
                .copied()
                .filter(|t| {
                    !t.within(self.start, 10)
                        && !t.within(seat, 8)
                        && !added.iter().any(|v| t.within(v.center, 9))
                })
                .min_by_key(|t| t.distance(goal))
                .expect("Island needs connected village sites");
            let houses = [(-2, -1), (2, -2), (-2, 2), (2, 1)]
                .into_iter()
                .map(|(x, z)| center.offset(x, z))
                .filter(|&p| dry(p) && !self.road(p))
                .take(2)
                .collect();
            added.push(Village {
                center,
                name: NAMES[self.kind as usize][index].into(),
                houses,
                need: Material::ALL[(index + self.chapter() + self.kind as usize) % 3],
            });
        }
        self.rival_seat = Some(seat);
        self.villages.extend(added);
        // Keep old routes and building footprints valid on restored islands. Plazas are
        // walkable meeting places. New construction must leave these sites free.
    }

    pub fn village_at(&self, tile: Tile) -> Option<usize> {
        self.villages
            .iter()
            .position(|v| v.center == tile || v.houses.contains(&tile))
    }
    pub fn rival_home(&self) -> Tile {
        self.rival_seat.unwrap_or(self.rival_shrine)
    }
}
