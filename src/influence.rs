use super::*;

const FIRST_ENVOY_DELAY: f32 = 115.;
pub const OCCUPATION_GRACE: f32 = 200.;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Allegiance {
    #[default]
    Neutral,
    Ours,
    Rival,
}
impl Allegiance {
    pub fn name(self) -> &'static str {
        match self {
            Self::Neutral => "Neutral",
            Self::Ours => "Unser Dorf",
            Self::Rival => "Beim Rivalen",
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Standing {
    pub protection: f32,
    pub pilgrim_rest: f32,
    pub side: Allegiance,
    pub trust: f32,
    pub gift_given: bool,
    won_once: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Contest {
    pub towns: Vec<Standing>,
    pub pact: bool,
    pub rival_tile: Option<Tile>,
    pub destination: Option<usize>,
    path: VecDeque<Tile>,
    delay: f32,
    stride: f32,
    effort: f32,
    supplies: f32,
    pub event: u32,
    pub news: String,
    pub creature_visits: u16,
    pub occupation: f32,
}
impl Default for Contest {
    fn default() -> Self {
        Self {
            towns: Vec::new(),
            pact: false,
            rival_tile: None,
            destination: None,
            path: VecDeque::new(),
            delay: FIRST_ENVOY_DELAY,
            stride: 0.,
            effort: 0.,
            supplies: 40.,
            event: 0,
            news: String::new(),
            creature_visits: 0,
            occupation: 0.,
        }
    }
}
impl Contest {
    pub(super) fn let_village_settle(&mut self) {
        self.destination = None;
        self.rival_tile = None;
        self.path.clear();
        self.delay = FIRST_ENVOY_DELAY;
        self.occupation = 0.;
    }

    pub fn count(&self, side: Allegiance) -> usize {
        self.towns.iter().filter(|t| t.side == side).count()
    }
    pub fn majority(&self) -> usize {
        self.towns.len() / 2 + 1
    }
    pub fn status(&self, index: usize) -> String {
        let t = &self.towns[index];
        match t.side {
            Allegiance::Neutral if t.trust.abs() < 1. => "Neutral".into(),
            Allegiance::Neutral if t.trust < 0. => format!("Neutral · Rivale {:.0} %", -t.trust),
            Allegiance::Neutral => format!("Neutral · Vertrauen {:.0} %", t.trust),
            side => format!("{} · Bindung {:.0} %", side.name(), t.trust.abs()),
        }
    }
}

impl Settlement {
    pub(super) fn populate_villages(&mut self, region: &Region) {
        if self.contest.towns.is_empty() && !region.villages.is_empty() {
            self.contest
                .towns
                .resize(region.villages.len(), Standing::default());
            for (standing, d) in self.contest.towns.iter_mut().zip(self.districts) {
                if d.outcome.is_some() {
                    standing.side = Allegiance::Ours;
                    standing.trust = 100.;
                    standing.won_once = true;
                }
            }
        }
    }
    pub fn visitor(&self, i: usize, hippo: bool) -> Option<&Walker> {
        self.walkers
            .iter()
            .find(|w| w.hippo == hippo && w.job == Some(Job::Visit(i)))
    }
    pub fn nearest_village(&self, region: &Region) -> usize {
        region
            .villages
            .iter()
            .enumerate()
            .min_by_key(|(i, v)| {
                (
                    self.contest.towns[*i].side == Allegiance::Ours,
                    v.center.distance(self.walkers[0].tile()),
                )
            })
            .map_or(0, |(i, _)| i)
    }
    pub fn send_visitor(&mut self, region: &Region, i: usize, hippo: bool) -> GameResult {
        if self.settling_in() {
            return Err("Baue zuerst dein Dorf auf oder begleite deinen ersten Besuch.".into());
        }
        self.dispatch_visitor(region, i, hippo)
    }

    pub(super) fn dispatch_visitor(
        &mut self,
        region: &Region,
        i: usize,
        hippo: bool,
    ) -> GameResult {
        let village = region
            .villages
            .get(i)
            .ok_or("Wähle ein Dorf auf der Insel.")?;
        if self.visitor(i, hippo).is_some() {
            return Err("Dieser Besuch läuft bereits.".into());
        }
        let worker = if hippo {
            0
        } else {
            self.walkers
                .iter()
                .enumerate()
                .filter(|(_, w)| !w.hippo && !w.guard && w.childhood <= 0.)
                .min_by_key(|(_, w)| {
                    (
                        w.job.is_some(),
                        w.bundle.is_some(),
                        w.tile().distance(village.center),
                    )
                })
                .map(|(i, _)| i)
                .ok_or("Im Dorf ist noch kein erwachsener Gläubiger verfügbar.")?
        };
        if self
            .gatherings
            .circle
            .as_ref()
            .is_some_and(|c| c.guests.iter().any(|(id, _)| *id == worker))
        {
            return Err("Am Festkreis eingeteilt. Beende den Dorfruf oder löse ihn auf.".into());
        }
        let route = self
            .travel_route(
                region,
                self.walkers[worker].tile(),
                village.center,
                true,
                true,
            )
            .ok_or("Kein Weg zum Dorfplatz. Halte einen Zugang zum Dorf frei.")?;
        self.dispatch_gatherer(worker, Job::Visit(i));
        if self.walkers[worker].bundle.is_none() {
            self.walkers[worker].route = route;
        }
        Ok(format!(
            "{} besucht {}.",
            if hippo {
                "Dein Nilpferd"
            } else {
                "Ein Gläubiger"
            },
            village.name
        ))
    }
    pub fn visiting(&self, w: &Walker, region: &Region, i: usize) -> bool {
        w.job == Some(Job::Visit(i))
            && w.bundle.is_none()
            && w.route.is_empty()
            && w.tile().distance(region.villages[i].center) <= 1
            && self.gatherings.circle.as_ref().is_none_or(|c| {
                !c.guests.iter().any(|(id, _)| {
                    self.walkers
                        .get(*id)
                        .is_some_and(|guest| std::ptr::eq(w, guest))
                })
            })
    }
    pub(super) fn withdraw(&mut self, region: &Region, i: usize) -> GameResult {
        let village = region.villages.get(i).ok_or("Unbekanntes Dorf.")?;
        let mut found = false;
        let home = region
            .depot
            .neighbors()
            .into_iter()
            .find(|&t| self.walkable(region, t))
            .ok_or("Halte einen Zugang zum Sammelplatz frei.")?;
        for w in &mut self.walkers {
            if w.job == Some(Job::Visit(i)) {
                found = true;
                w.job = Some(Job::Move(home));
                w.route.clear();
                w.pause = 0.;
            }
        }
        if !found {
            return Err("Hier ist niemand von dir eingeteilt.".into());
        }
        Ok(format!(
            "Deine Besucher verlassen {} und kehren heim.",
            village.name
        ))
    }
    pub fn gift_ready(&self, region: &Region, i: usize) -> Result<(), String> {
        let v = region.villages.get(i).ok_or("Unbekanntes Dorf.")?;
        if !self.walkers.iter().any(|w| self.visiting(w, region, i)) {
            return Err("Die Besucher müssen zuerst den Dorfplatz erreichen.".into());
        }
        let town = &self.contest.towns[i];
        if town.side == Allegiance::Ours || town.gift_given {
            return Err("Das Dorf hat deine Hilfe bereits angenommen.".into());
        }
        if self.stock[v.need.index()] < 4 {
            return Err(format!(
                "Die Familien bitten um 4 {}.",
                region.kind.material(v.need)
            ));
        }
        Ok(())
    }
    pub(super) fn village_gift(&mut self, region: &Region, i: usize) -> GameResult {
        self.gift_ready(region, i)?;
        let v = &region.villages[i];
        self.stock[v.need.index()] -= 4;
        self.contest.towns[i].gift_given = true;
        let gain = (100. - self.contest.towns[i].trust).min(35.);
        self.change_trust(region, i, gain);
        Ok(format!(
            "{} nimmt die Gabe an · +{gain:.0} Vertrauen",
            v.name
        ))
    }
    pub(super) fn announce_village(&mut self, message: String) {
        self.contest.event = self.contest.event.wrapping_add(1);
        self.contest.news.clone_from(&message);
        self.note(message);
    }
    pub(super) fn change_trust(&mut self, region: &Region, i: usize, change: f32) {
        let t = &mut self.contest.towns[i];
        t.trust = (t.trust + change).clamp(-100., 100.);
        let old = t.side;
        if t.trust >= 100. {
            t.side = Allegiance::Ours;
        } else if t.trust <= -100. {
            t.side = Allegiance::Rival;
        } else if (old == Allegiance::Ours && t.trust <= 0.)
            || (old == Allegiance::Rival && t.trust >= 0.)
        {
            t.side = Allegiance::Neutral;
        }
        if t.side == old {
            return;
        }
        let side = t.side;
        t.gift_given = false;
        if side == Allegiance::Ours && !t.won_once {
            t.won_once = true;
            self.faith = self.faith.saturating_add(4);
            if self
                .walkers
                .first()
                .is_some_and(|w| self.visiting(w, region, i))
            {
                self.contest.creature_visits = self.contest.creature_visits.saturating_add(1);
            }
        }
        if let Some(d) = self.districts.get_mut(i) {
            d.outcome = if side == Allegiance::Ours {
                Some(Outcome::Alliance)
            } else {
                None
            };
            if side != Allegiance::Ours {
                d.resolve = 4;
            }
        }
        let v = &region.villages[i];
        self.announce_village(format!(
            "{} {}",
            v.name,
            match side {
                Allegiance::Ours => "folgt dir · Einfluss und Lieferungen gewonnen",
                Allegiance::Neutral => "hat sich losgesagt und ist wieder neutral",
                Allegiance::Rival => "folgt jetzt deinem Rivalen",
            }
        ));
    }
    fn rival_mission(&mut self, region: &Region, dt: f32) {
        if region.sandbox || self.rival_waiting() || self.contest.pact || self.rival_resolve == 0 {
            self.contest.destination = None;
            self.contest.path.clear();
            return;
        }
        let pos = self.contest.rival_tile.unwrap_or(region.rival_lair());
        if self.contest.delay > 0. {
            self.contest.delay -= dt;
            return;
        }
        if self.contest.destination.is_none() {
            if self.raid_warning(region) {
                return;
            }
            let mut choices: Vec<_> = region
                .villages
                .iter()
                .enumerate()
                .filter(|(i, _)| self.contest.towns[*i].side != Allegiance::Rival)
                .collect();
            choices.sort_by_key(|(i, v)| {
                (
                    self.contest.towns[*i].side == Allegiance::Ours,
                    v.center.distance(pos),
                )
            });
            if let Some((i, path)) = choices.into_iter().find_map(|(i, v)| {
                self.travel_route(region, pos, v.center, true, true)
                    .map(|p| (i, p))
            }) {
                self.contest.destination = Some(i);
                self.contest.path = path;
                self.contest.effort = 0.;
                self.note(format!(
                    "Ein Gesandter des Rivalen reist nach {}.",
                    region.villages[i].name
                ));
            } else {
                self.contest.delay = 12.;
            }
        }
        if let Some(i) = self.contest.destination {
            self.contest.stride -= dt;
            if self.contest.stride <= 0.
                && let Some(next) = self.contest.path.pop_front()
            {
                if self.walkable(region, next) || region.ford(next) {
                    self.contest.rival_tile = Some(next);
                    self.contest.stride = 0.68;
                } else {
                    self.contest.path.clear();
                    self.contest.destination = None;
                    self.contest.delay = 3.;
                }
            } else if self.contest.path.is_empty() && pos.distance(region.villages[i].center) <= 1 {
                self.contest.effort += dt;
            }
            if self.contest.towns[i].side == Allegiance::Rival || self.contest.effort > 95. {
                self.contest.destination = None;
                self.contest.path.clear();
                self.contest.delay = 62.;
            }
        }
    }
    pub(super) fn visit_tick(&mut self, region: &Region, dt: f32) {
        if self.contest.towns.is_empty() {
            return;
        }
        self.rival_mission(region, dt);
        for (i, v) in region.villages.iter().enumerate() {
            let mut rate = 0.;
            for w in self.walkers.iter().filter(|w| self.visiting(w, region, i)) {
                rate += if w.hippo { 5. } else { 3.5 };
            }
            if self.contest.destination == Some(i)
                && self
                    .contest
                    .rival_tile
                    .is_some_and(|p| p.distance(v.center) <= 1)
            {
                rate -= if self.contest.towns[i].protection > 0. {
                    0.8
                } else {
                    1.7
                };
            }
            if rate != 0. {
                self.change_trust(region, i, rate * dt);
            }
        }
        let was_occupied = self.contest.occupation >= OCCUPATION_GRACE;
        if !region.sandbox
            && !self.rival_waiting()
            && self.rival_resolve > 0
            && !self.contest.pact
            && self.contest.count(Allegiance::Rival) >= self.contest.majority()
        {
            self.contest.occupation = (self.contest.occupation + dt).min(OCCUPATION_GRACE);
        } else {
            self.contest.occupation = 0.;
        }
        if !was_occupied && self.contest.occupation >= OCCUPATION_GRACE {
            self.announce_village(
                "Der Rivale kontrolliert die Mehrheit und fordert Tribut. Gewinne ein Dorf zurück."
                    .into(),
            );
        } else if was_occupied && self.contest.occupation == 0. {
            self.announce_village("Die Vorherrschaft ist gebrochen. Der Tribut endet.".into());
        }
        self.contest.supplies -= dt;
        if self.contest.supplies <= 0. {
            self.contest.supplies = 40.;
            let before = self.stock;
            for (i, v) in region.villages.iter().enumerate() {
                if self.contest.towns[i].side == Allegiance::Ours
                    && (!region.sandbox || i >= region.districts.len())
                {
                    let m = v.need.index();
                    self.stock[m] = self.stock[m]
                        .saturating_add(2)
                        .min(self.storage_cap())
                        .max(self.stock[m]);
                }
            }
            let shipment = std::array::from_fn(|i| self.stock[i] - before[i]);
            if shipment != [0; 3] {
                self.note(format!("Lieferung der Dörfer · {}", stock_text(shipment)));
            }
            if self.contest.occupation >= OCCUPATION_GRACE
                && let Some(m) = Material::ALL
                    .into_iter()
                    .max_by_key(|m| self.stock[m.index()])
            {
                let levy = self.stock[m.index()].saturating_sub(6).min(2);
                if levy > 0 {
                    self.stock[m.index()] -= levy;
                    self.note(format!(
                        "Tribut an den Rivalen · −{levy} {}",
                        region.kind.material(m)
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rival_tribute_needs_a_sustained_majority_and_ends_when_the_villages_are_free() {
        let game = Game::start(Mode::Campaign(Biome::Valley));
        let mut city = game.city().clone();
        let r = game.region();
        let majority = city.contest.majority();
        for t in &mut city.contest.towns[..majority] {
            t.side = Allegiance::Rival;
            t.trust = -100.;
        }
        city.village_start = Some(VillageStart::Open);
        city.contest.delay = 1000.;
        city.stock = [12, 6, 4];
        city.contest.supplies = OCCUPATION_GRACE;
        city.visit_tick(r, OCCUPATION_GRACE - 1.);
        assert_eq!(city.stock, [12, 6, 4]);
        city.visit_tick(r, 1.);
        assert_eq!(city.stock, [10, 6, 4]);
        city.stock = [7, 4, 6];
        city.visit_tick(r, 40.);
        assert_eq!(city.stock, [6, 4, 6]);
        city.contest.towns[0].side = Allegiance::Neutral;
        city.stock = [14, 4, 6];
        city.visit_tick(r, 40.);
        assert_eq!(city.stock, [14, 4, 6]);
        assert_eq!(city.contest.occupation, 0.);
        city.contest.destination = Some(0);
        city.contest.rival_tile = Some(r.villages[0].center);
        city.rival_resolve = 0;
        city.contest.towns[0].trust = 0.;
        city.visit_tick(r, 1.);
        assert_eq!(city.contest.towns[0].trust, 0.);
        assert!(city.contest.destination.is_none());
    }

    #[test]
    fn every_chapter_starts_between_villages_and_visitors_can_reach_both_banks() {
        for kind in Biome::ALL {
            let regions = Mode::Campaign(kind).regions(|_| WorldLayout::Islands);
            for (ch, r) in regions.iter().enumerate() {
                let city = Settlement::new(r);
                assert_eq!(r.villages.len(), 4 + ch);
                assert!(r.rival_home().distance(r.start) >= 40);
                assert!(r.villages.iter().all(|v| v.center != r.rival_home()));
                assert!(
                    r.villages
                        .iter()
                        .any(|v| v.center.distance(r.start) < r.start.distance(r.rival_home()) / 2)
                );
                assert_eq!(city.contest.count(Allegiance::Neutral), r.villages.len());
                for v in &r.villages {
                    for from in [city.walkers[0].tile(), r.rival_lair()] {
                        assert!(
                            city.travel_route(r, from, v.center, true, true).is_some(),
                            "{} {:?} → {} {:?}",
                            r.name,
                            from,
                            v.name,
                            v.center
                        );
                    }
                }
            }
        }
    }

    fn run_until(game: &mut Game, seconds: usize, done: impl Fn(&Settlement) -> bool) {
        for _ in 0..seconds * 12 {
            if done(game.city()) {
                return;
            }
            game.step(1. / 12.);
        }
        assert!(
            done(game.city()),
            "{} · {}",
            game.region().name,
            game.city().contest.news
        );
    }

    #[test]
    fn a_visit_must_arrive_then_a_gift_and_two_visitors_can_win_the_village() {
        let mut game = Game::start(Mode::Campaign(Biome::Valley));
        game.campaign.settlements[0].as_mut().unwrap().village_start = Some(VillageStart::Open);
        let i = game.city().nearest_village(game.region());
        let v = game.region().villages[i].clone();
        assert!(!game.city().influence(game.region(), v.center));
        game.act(Action::Visit(i, true)).unwrap();
        assert!(game.act(Action::VillageGift(i)).is_err());
        game.act(Action::Visit(i, false)).unwrap();
        assert!(game.act(Action::Visit(i, true)).is_err());
        run_until(&mut game, 40, |c| c.contest.towns[i].trust > 2.);
        let mut game = Game::resume(
            serde_json::from_slice(&serde_json::to_vec(&game.saved_games()).unwrap()).unwrap(),
        )
        .unwrap();
        let stock = game.city().stock;
        game.act(Action::VillageGift(i)).unwrap();
        assert_eq!(game.city().stock[v.need.index()], stock[v.need.index()] - 4);
        assert!(game.act(Action::VillageGift(i)).is_err());
        run_until(&mut game, 35, |c| {
            c.contest.towns[i].side == Allegiance::Ours
        });
        assert!(game.city().influence(game.region(), v.center));
        assert_eq!(game.city().contest.creature_visits, 1);
        game.act(Action::Withdraw(i)).unwrap();
        assert!(game.city().visitor(i, true).is_none());
    }

    #[test]
    fn rival_takes_neutral_land_and_players_can_win_it_back_and_secure_a_chapter() {
        let mut game = Game::start(Mode::Campaign(Biome::Valley));
        game.campaign.settlements[0].as_mut().unwrap().village_start = Some(VillageStart::Open);
        for _ in 0..240 {
            game.step(1. / 12.);
        }
        assert!(game.city().contest.destination.is_none());
        run_until(&mut game, 260, |c| c.contest.count(Allegiance::Rival) > 0);
        let index = game
            .city()
            .contest
            .towns
            .iter()
            .position(|t| t.side == Allegiance::Rival)
            .unwrap();
        let mut game = Game::resume(game.saved_games()).unwrap();
        game.act(Action::Visit(index, true)).unwrap();
        game.act(Action::Visit(index, false)).unwrap();
        run_until(&mut game, 150, |c| {
            c.contest.towns[index].side == Allegiance::Ours
        });
        assert!(game.act(Action::IslandPact).is_err());
        for _ in 0..game.region().villages.len() {
            if game.city().contest.count(Allegiance::Ours) >= game.city().contest.majority() {
                break;
            }
            let next = game.city().nearest_village(game.region());
            game.act(Action::Visit(next, true)).unwrap();
            game.act(Action::Visit(next, false)).unwrap();
            run_until(&mut game, 150, |c| {
                c.contest.towns[next].side == Allegiance::Ours
            });
        }
        game.act(Action::IslandPact).unwrap();
        assert_eq!(game.city().quest, Quest::Complete);
        let faith = game.city().faith;
        assert!(game.act(Action::IslandPact).is_err());
        assert_eq!(game.city().faith, faith);
        game.act(Action::Travel(1)).unwrap();
        assert_eq!(game.city().contest.count(Allegiance::Neutral), 5);
        game.act(Action::Travel(0)).unwrap();
        assert!(game.city().contest.pact);
    }
}
