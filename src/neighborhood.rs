use super::*;
use std::collections::{BTreeMap, VecDeque};

#[derive(Clone, Copy)]
pub struct Amenities {
    pub roads: bool,
    pub providers: [Option<(Tile, u8)>; 3],
    pub food: bool,
}

impl Amenities {
    pub fn met(self, i: usize) -> bool {
        self.providers[i].is_some() && (i != 1 || self.food)
    }
    pub fn score(self) -> usize {
        (0..3).filter(|&i| self.met(i)).count()
    }
}

pub const SERVICES: [(&str, Building, &'static str); 3] = [
    (
        "Trinkwasser",
        Building::Well,
        "Ein Brunnen oder Badehaus versorgt die Straße.",
    ),
    (
        "Lebensmittel",
        Building::Market,
        "Ein Markt oder eine Bäckerei braucht vier Nahrung im Insellager.",
    ),
    (
        "Gemeinschaft",
        Building::Shrine,
        "Schrein, Tempel, Theater, Halle oder Schenke beleben das Viertel.",
    ),
];

impl Settlement {
    pub fn amenities(&self, r: &Region, home: Tile) -> Amenities {
        let kind = self
            .building_at(home)
            .map_or(Building::House, |site| site.kind);
        let street = |t: Tile| self.road_at(r, t) && self.walkable(r, t);
        let mut reached = BTreeMap::new();
        let mut queue = VecDeque::new();
        for t in kind
            .tiles(home)
            .flat_map(Tile::neighbors)
            .filter(|&t| street(t))
        {
            if reached.insert(t, 1u8).is_none() {
                queue.push_back(t);
            }
        }
        let roads = !queue.is_empty();
        while let Some(t) = queue.pop_front() {
            let d = reached[&t];
            if d >= 12 {
                continue;
            }
            for n in t.neighbors() {
                if street(n) && !reached.contains_key(&n) {
                    reached.insert(n, d + 1);
                    queue.push_back(n);
                }
            }
        }
        let mut providers: [Option<(Tile, u8)>; 3] = [None; 3];
        for site in &self.buildings {
            let i = match site.kind {
                Building::Well | Building::Civic(Civic::Bath) => 0,
                Building::Market | Building::Civic(Civic::Bakery) => 1,
                Building::Shrine
                | Building::Civic(Civic::Temple | Civic::Theater | Civic::Hall | Civic::Tavern) => {
                    2
                }
                _ => continue,
            };
            if let Some(d) = site
                .kind
                .tiles(site.tile)
                .flat_map(Tile::neighbors)
                .filter_map(|t| reached.get(&t).copied())
                .min()
                && providers[i].is_none_or(|(_, old)| d < old)
            {
                providers[i] = Some((site.tile, d));
            }
        }
        Amenities {
            roads,
            providers,
            food: self.stock[Material::Food.index()] >= 4,
        }
    }
    pub fn charter_ready(&self, r: &Region, t: Tile) -> Result<(), String> {
        let home = self
            .building_at(t)
            .filter(|s| s.kind == Building::House)
            .ok_or("Wähle eine gebaute Hütte.")?;
        if self.citizens.contains(&home.tile) {
            return Err("Dieses Haus besitzt das Bürgerrecht.".into());
        }
        if !home.upgraded {
            return Err("Baue die Hütte zuerst zur zweiten Stufe aus.".into());
        }
        let a = self.amenities(r, home.tile);
        if !a.roads {
            return Err("Lege einen Weg bis zur Haustür.".into());
        }
        if let Some(i) = (0..3).find(|&i| !a.met(i)) {
            return Err(format!("Es fehlt {} im Viertel.", SERVICES[i].0));
        }
        if !r.sandbox && !self.affordable([6, 4, 4]) {
            return Err("Benötigt werden 6 Holz, 4 Stein und 4 Nahrung.".into());
        }
        Ok(())
    }
    pub(super) fn charter(&mut self, r: &Region, t: Tile) -> GameResult {
        self.charter_ready(r, t)?;
        let at = self.building_at(t).unwrap().tile;
        if !r.sandbox {
            self.pay([6, 4, 4]);
        }
        self.citizens.push(at);
        Ok("Bürgerrecht verliehen · ein neuer Wohnplatz im Viertel.".into())
    }
    pub(super) fn supply_homes(&mut self, r: &Region) {
        let mut count = 0u16;
        for &home in &self.citizens {
            if self.stock[2] < 4 {
                break;
            }
            if self.amenities(r, home).score() == 3 {
                self.stock[2] -= 1;
                count += 1;
            }
        }
        if count > 0 {
            let faith = count.saturating_mul(2);
            self.faith = self.faith.saturating_add(faith);
            self.note(format!(
                "Bürgerhäuser versorgt · −{count} Nahrung · +{faith} Glauben"
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn citizens_need_a_real_street_and_a_stocked_market_and_only_pay_when_served() {
        let game = Game::start(Mode::Campaign(Biome::Valley));
        let r = game.region();
        let mut city = game.city().clone();
        city.buildings.clear();
        city.roads.clear();
        let origin = r
            .bounds
            .tiles()
            .find(|&t| {
                (-1..=14).all(|x| {
                    (-1..=4).all(|z| {
                        let t = t.offset(x, z);
                        city.walkable(r, t) && !r.road(t)
                    })
                })
            })
            .expect("A clear district site");
        let home = origin.offset(0, 1);
        for (kind, x) in [
            (Building::House, 0),
            (Building::Well, 3),
            (Building::Market, 6),
            (Building::Shrine, 9),
        ] {
            city.buildings.push(BuildingSite {
                kind,
                tile: origin.offset(x, 1),
                rotation: 0,
                crop: Crop::Dry,
                upgraded: true,
                batch_finished: false,
            });
        }
        city.stock = [10, 10, 10];
        let beds = city.housing();
        assert!(city.charter_ready(r, home).is_err());
        city.roads.extend((0..=10).map(|x| origin.offset(x, 0)));
        assert_eq!(city.amenities(r, home).score(), 3);
        city.stock[2] = 3;
        assert_eq!(city.amenities(r, home).score(), 2);
        city.stock[2] = 10;
        city.charter(r, home).unwrap();
        assert_eq!(city.stock, [4, 6, 6]);
        assert_eq!(city.housing(), beds + 1);
        let mut restored: Settlement =
            serde_json::from_slice(&serde_json::to_vec(&city).unwrap()).unwrap();
        assert!(restored.charter(r, home).is_err());
        let faith = restored.faith;
        restored.supply_homes(r);
        assert_eq!(restored.stock[2], 5);
        assert_eq!(restored.faith, faith + 2);
        restored.roads.retain(|&t| t != origin.offset(5, 0));
        assert_eq!(restored.amenities(r, home).score(), 1);
        restored.supply_homes(r);
        assert_eq!(restored.stock[2], 5);
        assert_eq!(restored.faith, faith + 2);
    }
}
