use crate::region::Biome;

pub struct Rival {
    pub name: &'static str,
    pub creature: &'static str,
    pub warning: &'static str,
    pub impact: &'static str,
    pub defense: &'static str,
    pub interval: f32,
}

impl Rival {
    pub fn for_biome(biome: Biome) -> Self {
        match biome {
            Biome::Valley => Self {
                name: "Waldhüter Iven",
                creature: "Mooshirsch",
                warning: "Der Mooshirsch kann bis zu 4 Holz mitnehmen.",
                impact: "Der Mooshirsch hat bis zu 4 Holz mitgenommen.",
                defense: "Schutzglocke, Wache oder Försterhof am Weg",
                interval: 145.,
            },
            Biome::Fjord => Self {
                name: "Sigrun vom Runenufer",
                creature: "Runenwolf",
                warning: "Der Frost des Runenwolfs kann bis zu 4 Nahrung verderben.",
                impact: "Der Runenwolf hat Frost gebracht. Bis zu 4 Nahrung sind verdorben.",
                defense: "Schutzglocke, Wache oder Speicher am Weg",
                interval: 125.,
            },
            Biome::Terraces => Self {
                name: "Lin vom Wolkenwehr",
                creature: "Wolkendrache",
                warning: "Der Wolkendrache verzögert die wachsende Saat um bis zu 9 Sekunden.",
                impact: "Der Wolkendrache hat den Kanal gestaut. Die Saat braucht etwas länger.",
                defense: "Schutzglocke, Wache oder zwei Brunnen am Weg",
                interval: 135.,
            },
            Biome::Lake => Self {
                name: "Brunnenhüter Nuru",
                creature: "Schilfschlange",
                warning: "Die Schilfschlange hält Sammler kurz auf und kostet das Nilpferd bis zu 19 Kraft.",
                impact: "Die Schilfschlange hat Sammler aufgehalten. Das Nilpferd hat Kraft verloren.",
                defense: "Schutzglocke, Wache oder Wasserterrasse am Weg",
                interval: 140.,
            },
            Biome::Aegean => Self {
                name: "Nerea von der Windmole",
                creature: "Windgreif",
                warning: "Der Windgreif kann bis zu 8 Wunderkraft zerstreuen.",
                impact: "Der Windgreif hat bis zu 8 Wunderkraft zerstreut.",
                defense: "Schutzglocke, Wache oder Orakel am Weg",
                interval: 120.,
            },
        }
    }
}
