use rand::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ElementType {
    Air = 0,
    Stone = 1,
    Wood = 2,
    Coal = 3,
    Sand = 4,
    Gravel = 5,
    Gold = 6,
    Water = 7,
    Oil = 8,
    Acid = 9,
    Lava = 10,
    Gunpowder = 11,
    Fire = 12,
    Smoke = 13,
    Steam = 14,
    Ice = 15,
}

impl ElementType {
    pub fn name(&self) -> &'static str {
        match self {
            ElementType::Air => "Air",
            ElementType::Stone => "Stone",
            ElementType::Wood => "Wood",
            ElementType::Coal => "Coal",
            ElementType::Sand => "Sand",
            ElementType::Gravel => "Gravel",
            ElementType::Gold => "Gold",
            ElementType::Water => "Water",
            ElementType::Oil => "Oil",
            ElementType::Acid => "Acid",
            ElementType::Lava => "Lava",
            ElementType::Gunpowder => "Gunpowder",
            ElementType::Fire => "Fire",
            ElementType::Smoke => "Smoke",
            ElementType::Steam => "Steam",
            ElementType::Ice => "Ice",
        }
    }

    /// Density determines displacement rules (heavier items sink, lighter float/rise).
    /// Air = 0, Gases < 0, Solids/Liquids > 0
    pub fn density(&self) -> i8 {
        match self {
            ElementType::Air => 0,
            ElementType::Steam => -1,
            ElementType::Smoke => -2,
            ElementType::Fire => -3,
            ElementType::Oil => 4,
            ElementType::Water => 5,
            ElementType::Acid => 6,
            ElementType::Lava => 8,
            ElementType::Sand => 10,
            ElementType::Gravel => 11,
            ElementType::Gunpowder => 12,
            ElementType::Gold => 30,
            ElementType::Ice => 20,  // Stays static unless melted
            ElementType::Wood => 40, // Static unless burned/corroded
            ElementType::Coal => 45, // Static unless burned
            ElementType::Stone => 100, // Static, indestructible by normal physics
        }
    }

    pub fn is_static_solid(&self) -> bool {
        matches!(
            self,
            ElementType::Stone | ElementType::Wood | ElementType::Coal | ElementType::Ice
        )
    }

    pub fn is_falling_solid(&self) -> bool {
        matches!(
            self,
            ElementType::Sand | ElementType::Gravel | ElementType::Gunpowder | ElementType::Gold
        )
    }

    pub fn is_liquid(&self) -> bool {
        matches!(
            self,
            ElementType::Water | ElementType::Oil | ElementType::Acid | ElementType::Lava
        )
    }

    pub fn is_gas(&self) -> bool {
        matches!(self, ElementType::Smoke | ElementType::Steam | ElementType::Fire)
    }

    pub fn is_flammable(&self) -> bool {
        matches!(
            self,
            ElementType::Wood | ElementType::Oil | ElementType::Gunpowder | ElementType::Coal
        )
    }

    pub fn color_preview(&self) -> &'static str {
        match self {
            ElementType::Air => "#0a0a0a",
            ElementType::Stone => "#7a7a7a",
            ElementType::Wood => "#8a5a36",
            ElementType::Coal => "#2c2c2c",
            ElementType::Sand => "#dfc48c",
            ElementType::Gravel => "#9c9a96",
            ElementType::Gold => "#ffd700",
            ElementType::Water => "#2b7dfa",
            ElementType::Oil => "#4b3f35",
            ElementType::Acid => "#39ff14",
            ElementType::Lava => "#ff4500",
            ElementType::Gunpowder => "#53535e",
            ElementType::Fire => "#ff6e00",
            ElementType::Smoke => "#5a5a5a",
            ElementType::Steam => "#cfd8dc",
            ElementType::Ice => "#a5f2f3",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ElementType::Air => "Empty space.",
            ElementType::Stone => "Solid, immovable wall. Highly resistant to acid and fire.",
            ElementType::Wood => "Immovable organic material. Highly flammable, burns into ash/coal.",
            ElementType::Coal => "Solid fuel. Burns very slowly when ignited, doesn't fall.",
            ElementType::Sand => "Loose grains of silica. Falls and piles up under gravity.",
            ElementType::Gravel => "Coarse rock particles. Heavier and darker than sand, falls in steeper piles.",
            ElementType::Gold => "Heavy, valuable metal. Sinks deep through all liquids.",
            ElementType::Water => "Universal solvent. Flows, extinguishes fires, and hardens lava.",
            ElementType::Oil => "Viscous hydrocarbon. Floats on water, highly flammable, burns intensely.",
            ElementType::Acid => "Highly corrosive green liquid. Eats through wood, stone, sand, and ice.",
            ElementType::Lava => "Superheated molten rock. Flows slowly, ignites flammables, melts ice, hardens in water.",
            ElementType::Gunpowder => "Explosive powder. Falls like sand, detonates instantly when touched by fire/lava.",
            ElementType::Fire => "Hot plasma. Burns upwards, ignites flammables, evaporates water, creates smoke.",
            ElementType::Smoke => "Light gas. Rises into the air, diffuses, and eventually fades away.",
            ElementType::Steam => "Vaporized water. Rises, cools, and can condense back into water drops.",
            ElementType::Ice => "Frozen water. Static wall, melts into water when heated by fire or lava.",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub element: ElementType,
    pub color: [u8; 3],
    pub life: u8,
    pub generation: u8,
}

impl Cell {
    pub fn new(element: ElementType) -> Self {
        let mut rng = rand::thread_rng();
        let color = match element {
            ElementType::Air => [10, 12, 16], // Dark background instead of absolute black
            ElementType::Stone => {
                let val = rng.gen_range(110..=135);
                [val, val, val]
            }
            ElementType::Wood => {
                let r = rng.gen_range(110..130);
                let g = rng.gen_range(65..85);
                let b = rng.gen_range(30..45);
                [r, g, b]
            }
            ElementType::Coal => {
                let val = rng.gen_range(28..40);
                [val, val, val]
            }
            ElementType::Sand => {
                let r = rng.gen_range(215..235);
                let g = rng.gen_range(180..205);
                let b = rng.gen_range(115..135);
                [r, g, b]
            }
            ElementType::Gravel => {
                let val = rng.gen_range(130..155);
                let r_offset = rng.gen_range(-5..=5);
                [
                    (val as i16 + r_offset).clamp(0, 255) as u8,
                    val,
                    (val as i16 - r_offset).clamp(0, 255) as u8,
                ]
            }
            ElementType::Gold => {
                let r = rng.gen_range(225..255);
                let g = rng.gen_range(180..210);
                let b = rng.gen_range(10..40);
                [r, g, b]
            }
            ElementType::Water => {
                let r = rng.gen_range(35..55);
                let g = rng.gen_range(100..130);
                let b = rng.gen_range(225..250);
                [r, g, b]
            }
            ElementType::Oil => {
                let r = rng.gen_range(50..65);
                let g = rng.gen_range(40..55);
                let b = rng.gen_range(30..45);
                [r, g, b]
            }
            ElementType::Acid => {
                let r = rng.gen_range(40..70);
                let g = rng.gen_range(230..255);
                let b = rng.gen_range(40..70);
                [r, g, b]
            }
            ElementType::Lava => {
                let r = rng.gen_range(230..255);
                let g = rng.gen_range(60..90);
                let b = rng.gen_range(0..25);
                [r, g, b]
            }
            ElementType::Gunpowder => {
                let val = rng.gen_range(70..85);
                [val, val, (val as f32 * 1.15).min(255.0) as u8]
            }
            ElementType::Fire => {
                let r = 255;
                let g = rng.gen_range(80..160);
                let b = rng.gen_range(0..40);
                [r, g, b]
            }
            ElementType::Smoke => {
                let val = rng.gen_range(75..100);
                [val, val, val]
            }
            ElementType::Steam => {
                let val = rng.gen_range(210..235);
                [val, val, val]
            }
            ElementType::Ice => {
                let r = rng.gen_range(175..195);
                let g = rng.gen_range(230..245);
                let b = 255;
                [r, g, b]
            }
        };

        let life = match element {
            ElementType::Fire => rng.gen_range(15..45),
            ElementType::Smoke => rng.gen_range(25..75),
            ElementType::Steam => rng.gen_range(30..90),
            ElementType::Acid => rng.gen_range(60..120),
            _ => 0,
        };

        Cell {
            element,
            color,
            life,
            generation: 0,
        }
    }
}
