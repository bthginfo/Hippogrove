use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tone {
    Gentle,
    Neutral,
    Stern,
}

impl Tone {
    pub fn name(self) -> &'static str {
        match self {
            Self::Gentle => "Fürsorglich",
            Self::Neutral => "Ausgeglichen",
            Self::Stern => "Gefürchtet",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Reputation(i16);

impl Reputation {
    pub fn shift(&mut self, amount: i16) {
        self.0 = self.0.saturating_add(amount).clamp(-100, 100);
    }

    pub fn tone(self) -> Tone {
        match self.0 {
            16.. => Tone::Gentle,
            ..=-16 => Tone::Stern,
            _ => Tone::Neutral,
        }
    }

    pub fn score(self) -> i16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct CreatureGrowth {
    // Deliveries, harvests, and successful watches. Credit comes from completed jobs.
    #[serde(skip)]
    pub work: [u32; 3],
    pub nature: Reputation,
}

impl CreatureGrowth {
    const THRESHOLDS: [u32; 5] = [0, 3, 9, 20, 39];

    pub fn experience(self) -> u32 {
        self.work[0]
            .saturating_add(self.work[1].saturating_mul(2))
            .saturating_add(self.work[2].saturating_mul(2))
    }

    pub fn stage(self) -> usize {
        Self::THRESHOLDS.partition_point(|&threshold| threshold <= self.experience()) - 1
    }

    pub fn description(self) -> &'static str {
        [
            "Jungtier",
            "Heranwachsend",
            "Kräftig",
            "Ausgewachsen",
            "Riese",
        ][self.stage()]
    }

    pub fn scale(self) -> f32 {
        [1.28, 1.49, 1.73, 2.01, 2.32][self.stage()]
    }

    pub fn next_stage(self) -> Option<u32> {
        Self::THRESHOLDS.get(self.stage() + 1).copied()
    }

    pub fn transport_bonus(self) -> u16 {
        (self.work[0] / 4).min(5) as u16
    }
}
