use crate::region::Biome;

pub struct Rival {
    pub name: &'static str,
    pub creature: &'static str,
    pub warning: &'static str,
    pub defense: &'static str,
    pub interval: f32,
}

impl Rival {
    pub fn for_biome(biome: Biome) -> Self {
        match biome {
            Biome::Valley => Self {
                name: "Waldhüter Iven",
                creature: "Mooshirsch",
                warning: "Der Mooshirsch zieht Holz in den Wald.",
                defense: "Schutzglocke, Wache oder Försterhof am Weg",
                interval: 40.,
            },
            Biome::Fjord => Self {
                name: "Sigrun vom Runenufer",
                creature: "Runenwolf",
                warning: "Der Runenwolf bringt Frost und verdirbt einen Teil der Nahrung.",
                defense: "Schutzglocke, Wache oder Speicher am Weg",
                interval: 33.,
            },
            Biome::Terraces => Self {
                name: "Lin vom Wolkenwehr",
                creature: "Wolkendrache",
                warning: "Der Wolkendrache staut den Kanal und verzögert die Ernte.",
                defense: "Schutzglocke, Wache oder zwei Brunnen am Weg",
                interval: 36.,
            },
            Biome::Lake => Self {
                name: "Brunnenhüter Nuru",
                creature: "Schilfschlange",
                warning: "Die Schilfschlange vertreibt Sammler und raubt dem Nilpferd Kraft.",
                defense: "Schutzglocke, Wache oder Wasserterrasse am Weg",
                interval: 38.,
            },
            Biome::Aegean => Self {
                name: "Nerea von der Windmole",
                creature: "Windgreif",
                warning: "Der Windgreif zerstreut Wunderkraft.",
                defense: "Schutzglocke, Wache oder Orakel am Weg",
                interval: 32.,
            },
        }
    }
}
