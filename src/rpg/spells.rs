use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Clone)]
pub enum SpellType {
    Single,
    Toggle,
    Summon,
    Unknown,
}

impl FromStr for SpellType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "single" => Ok(SpellType::Single),
            "toggle" => Ok(SpellType::Toggle),
            "summon" => Ok(SpellType::Summon),
            _ => Ok(SpellType::Unknown),
        }
    }
}

pub trait SpellResource {
    fn add(&mut self, other: &Self) -> Self;
}

#[derive(Clone)]
pub struct ManaSpellResource {
    pub mana: i32,
}
impl SpellResource for ManaSpellResource {
    fn add(&mut self, other: &Self) -> Self {
        return ManaSpellResource {
            mana: &self.mana + other.mana,
        };
    }
}
impl fmt::Display for ManaSpellResource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Mana: {}", self.mana)
    }
}

#[allow(dead_code)] //future planning
pub struct SlotSpellResource {
    pub slots: HashMap<i32, i32>,
}

#[derive(Clone)]
pub struct Spell<T: SpellResource> {
    pub name: Option<String>,
    pub cost: Option<T>,

    #[allow(dead_code)] //cargo is lying
    pub spell_type: Option<SpellType>,

    #[allow(dead_code)] //cargo is lying
    pub cast_time: Option<String>,
}
