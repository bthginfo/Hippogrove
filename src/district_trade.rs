use super::*;

/// A village offers its own surplus in return for the supplies its trade needs.
#[derive(Clone, Copy)]
pub struct Offer {
    pub wants: Material,
    pub gives: Material,
    pub price: u16,
    pub quantity: u16,
}

impl Offer {
    pub fn terms(self, biome: Biome) -> String {
        format!(
            "{} {} gegen {} {}",
            self.price,
            biome.material(self.wants),
            self.quantity,
            biome.material(self.gives)
        )
    }
}

impl Settlement {
    pub fn has_trading_partner(&self, region: &Region, index: usize) -> bool {
        self.districts
            .get(index)
            .is_some_and(|d| !region.tutorial && (d.outcome.is_some() || region.sandbox))
    }

    pub fn trade_offer(&self, region: &Region, index: usize) -> Offer {
        let (wants, gives) = match region.districts[index].need {
            DistrictNeed::Water => (Material::Stone, Material::Food),
            DistrictNeed::Shelter => (Material::Wood, Material::Stone),
            DistrictNeed::Patrol => (Material::Food, Material::Wood),
            DistrictNeed::Trade => (Material::Stone, Material::Wood),
        };
        Offer {
            wants,
            gives,
            price: 4,
            quantity: if self.districts[index].outcome == Some(Outcome::Victory) {
                5
            } else {
                8
            },
        }
    }

    pub fn trade_ready(
        &self,
        region: &Region,
        index: usize,
        tribute: bool,
    ) -> Result<Offer, String> {
        if !self.has_trading_partner(region, index) {
            return Err("Gewinne das Dorf am Übergang und dann diesen Ort.".into());
        }
        let district = &self.districts[index];
        if tribute && district.outcome != Some(Outcome::Victory) {
            return Err("Verbündete handeln mit dir.".into());
        }
        if district
            .last_deal
            .is_some_and(|(day, _)| day == (self.weather_time / DAY_LENGTH) as u32)
        {
            return Err("Heute schon abgerechnet. Morgen kommt die nächste Lieferung.".into());
        }
        let offer = self.trade_offer(region, index);
        if !tribute && self.stock[offer.wants.index()] < offer.price {
            return Err(format!(
                "{} · Es fehlen {} {}.",
                offer.terms(region.kind),
                offer.price - self.stock[offer.wants.index()],
                region.kind.material(offer.wants)
            ));
        }
        if self
            .storage_cap()
            .saturating_sub(self.stock[offer.gives.index()])
            < offer.quantity
        {
            return Err(format!(
                "Schaffe Platz für {} {} im Lager.",
                offer.quantity,
                region.kind.material(offer.gives)
            ));
        }
        Ok(offer)
    }

    pub(super) fn district_trade(
        &mut self,
        region: &Region,
        index: usize,
        tribute: bool,
    ) -> GameResult {
        let offer = self.trade_ready(region, index, tribute)?;
        let name = &region.districts[index].name;
        let report = if tribute {
            self.society.morale = self.society.morale.saturating_sub(6);
            format!(
                "{name} zahlt Tribut · +{} {} · −6 Stimmung",
                offer.quantity,
                region.kind.material(offer.gives)
            )
        } else {
            self.stock[offer.wants.index()] -= offer.price;
            format!(
                "{name} liefert · +{} {} für {} {}",
                offer.quantity,
                region.kind.material(offer.gives),
                offer.price,
                region.kind.material(offer.wants)
            )
        };
        self.stock[offer.gives.index()] += offer.quantity;
        self.districts[index].last_deal = Some(((self.weather_time / DAY_LENGTH) as u32, tribute));
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn villages_trade_once_per_day_keep_terms_on_reload_and_charge_only_on_success() {
        for biome in Biome::ALL {
            let mut game = Game::start(Mode::Sandbox(biome));
            let offer = game.city().trade_offer(game.region(), 0);
            assert_ne!(offer.wants, offer.gives);
            let city = game.campaign.settlements[0].as_mut().unwrap();
            city.stock = [0; 3];
            assert!(game.act(Action::Trade(0, false)).is_err());
            assert!(game.act(Action::Trade(0, true)).is_err());
            let city = game.campaign.settlements[0].as_mut().unwrap();
            city.stock[offer.wants.index()] = 12;
            city.stock[offer.gives.index()] = city.storage_cap() - 1;
            assert!(game.act(Action::Trade(0, false)).is_err());
            let city = game.campaign.settlements[0].as_mut().unwrap();
            assert_eq!(city.stock[offer.wants.index()], 12);
            city.stock[offer.gives.index()] = 0;
            game.act(Action::Trade(0, false)).unwrap();
            assert_eq!(game.city().stock[offer.gives.index()], 8);
            assert_eq!(game.city().stock[offer.wants.index()], 8);
            let mut game = Game::resume(game.saved_games()).unwrap();
            assert!(game.act(Action::Trade(0, false)).is_err());
            let city = game.campaign.settlements[0].as_mut().unwrap();
            city.weather_time += DAY_LENGTH;
            city.districts[0].outcome = Some(Outcome::Victory);
            assert_eq!(game.city().trade_offer(game.region(), 0).quantity, 5);
            let morale = game.city().society.morale;
            game.act(Action::Trade(0, true)).unwrap();
            assert_eq!(game.city().society.morale, morale - 6);
            assert_eq!(game.city().stock[offer.wants.index()], 8);
            assert_eq!(game.city().stock[offer.gives.index()], 13);
            assert!(game.act(Action::Trade(0, false)).is_err());
        }
    }
}
