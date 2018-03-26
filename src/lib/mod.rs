#[macro_use]
extern crate failure;
extern crate ordermap;
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use failure::{Error, ResultExt};

use rand::Rng;
use rand::distributions::{IndependentSample, Range};

use std::path::Path;
use std::fs::File;
use std::io::{BufReader, Read};
use std::fmt;

// const BASE_PATH: &'static str = "/home/xeal/Documents/projects/ub/";
const BASE_PATH: &str = "";

#[derive(Debug, Serialize, Deserialize)]
pub struct Champion {
    pub name: String,
    pub title: String,
    pub range: String,
}

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Map {
    SummonersRift,
    HowlingAbyss,
    TwistedTreeline,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemList {
    boots: Vec<Item>,
    common: Vec<Item>,
    ranged: Vec<Item>,
    melee: Vec<Item>,
    jungle: Vec<Item>,
    support: Vec<Item>,
    classic: Vec<Item>,
    rift: Vec<Item>,
    abyss: Vec<Item>,
    treeline: Vec<Item>,
    featured: Vec<Item>,
    // incomplete: Vec<Item>,
    special: SpecialItemList,
    trinket: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpecialItemList {
    #[serde(rename = "Ornn")]
    ornn: Vec<Item>,
    #[serde(rename = "Viktor")]
    viktor: Vec<Item>,
    other: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub cost: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RuneList {
    precision: RuneTree,
    domination: RuneTree,
    sorcery: RuneTree,
    resolve: RuneTree,
    inspiration: RuneTree,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuneTree {
    keystone: Vec<String>,
    tier1: Vec<String>,
    tier2: Vec<String>,
    tier3: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrimaryTree {
    pub name: String,
    pub keystone: String,
    pub tier1: String,
    pub tier2: String,
    pub tier3: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecondaryTree {
    pub name: String,
    pub runes: (String, String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpellList {
    common: Vec<String>,
    classic: Vec<String>,
    abyss: Vec<String>,
}

impl ::std::str::FromStr for Map {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rift" => Ok(Map::SummonersRift),
            "abyss" => Ok(Map::HowlingAbyss),
            "treeline" => Ok(Map::TwistedTreeline),
            _ => Err(format_err!("unrecognised map: {}", s))
        }
    }
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Map::*;

        match *self {
            SummonersRift => write!(f, "Summoner's Rift"),
            HowlingAbyss => write!(f, "Howling Abyss"),
            TwistedTreeline => write!(f, "Twisted Treeline"),
        }
    }
}

impl std::clone::Clone for Item {
    fn clone(&self) -> Self {
        Item {
            name: self.name.clone(),
            cost: self.cost,
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Display for Champion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {}", self.name, self.title)
    }
}

impl std::convert::From<Champion> for String {
    fn from(c: Champion) -> Self {
        c.name + ", " + &c.title
    }
}

impl std::clone::Clone for Champion {
    fn clone(&self) -> Self {
        Champion {
            name: self.name.clone(),
            title: self.title.clone(),
            range: self.range.clone(),
        }
    }
}

impl fmt::Display for PrimaryTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {}, {}, {}, {}",
            self.name.to_uppercase(),
            self.keystone,
            self.tier1,
            self.tier2,
            self.tier3
        )
    }
}

impl fmt::Display for SecondaryTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {}, {}",
            self.name.to_uppercase(),
            self.runes.0,
            self.runes.1
        )
    }
}

impl std::convert::From<Map> for String {
    fn from(m: Map) -> Self {
        use Map::*;

        match m {
            SummonersRift => String::from("Summoner's Rift"),
            HowlingAbyss => String::from("Howling Abyss"),
            TwistedTreeline => String::from("Twisted Treeline"),
        }
    }
}

pub fn random_champion() -> Result<Champion, Error> {
    let v: Vec<Champion> = serde_json::from_str(&open_file("champions")?)?;
    let mut rng = rand::thread_rng();

    rng.choose(&v).map(|s| s.clone()).ok_or(failure::err_msg("Unable to choose a random champion"))
}

/// Returns a list of random items guarenteed to be different.
pub fn random_items(map: Map, number: usize, extra: &str, include_jungle: bool) -> Result<Vec<Item>, Error> {
    let all_items: ItemList = serde_json::from_str(&open_file("items")?)?;

    let mut item_pool = Vec::new();
    item_pool.extend(all_items.common);
    match extra {
        "melee" => item_pool.extend(all_items.melee),
        "ranged" => item_pool.extend(all_items.ranged),
        "hybrid" => {
            item_pool.extend(all_items.melee);
            item_pool.extend(all_items.ranged);
        }
        _ => ()
    }

    match map {
        m @ Map::SummonersRift | m @ Map::TwistedTreeline => {
            item_pool.extend(all_items.classic);

            match m {
                Map::SummonersRift => item_pool.extend(all_items.rift),
                Map::TwistedTreeline => item_pool.extend(all_items.treeline),
                _ => ()
            }

            let mut rng = rand::thread_rng();
            item_pool.push(rng.choose(&all_items.support).unwrap().clone());
            if include_jungle {
                item_pool.push(rng.choose(&all_items.jungle).unwrap().clone());
            }
        }
        Map::HowlingAbyss => {
            item_pool.extend(all_items.abyss);
        }
    };

    let range = Range::new(0, item_pool.len() - 1);
    let mut rng = rand::thread_rng();

    let mut items = Vec::new();
    let mut seen = Vec::new();
    while items.len() < number {
        let i = range.ind_sample(&mut rng);
        if !seen.contains(&i) {
            items.push(item_pool[i].clone());
            seen.push(i);
        }
    }

    Ok(items)
}

/// Returns a single random item from a given category. Used primarily for boots, jungle, and special items.
pub fn random_item_from_category(cat: &str) -> Result<Item, Error> {
    let items: ItemList = serde_json::from_str(&open_file("items")?)?;
    let sub = match cat {
        "boots" => items.boots,
        "common" => items.common,
        "ranged" => items.ranged,
        "melee" => items.melee,
        "jungle" => items.jungle,
        "support" => items.support,
        "classic" => items.classic,
        "rift" => items.rift,
        "abyss" => items.abyss,
        "treeline" => items.treeline,
        "featured" => items.featured,
        "Ornn" | "ornn" => items.special.ornn,
        "Viktor" | "viktor" => items.special.viktor,
        "other" => items.special.other,
        _ => Vec::new(),
    };

    let mut rng = rand::thread_rng();
    Ok(rng.choose(&sub).unwrap().clone())
}

/// Returns a primary and secondary rune tree.
pub fn random_rune_page() -> Result<(PrimaryTree, SecondaryTree), Error> {
    let runes: RuneList = serde_json::from_str(&open_file("runes")?)?;
    let mut rng = rand::thread_rng();

    let path_names = ["Precision", "Domination", "Sorcery", "Resolve", "Inspiration"];
    let path_arr = [runes.precision, runes.domination,
                    runes.sorcery, runes.resolve, runes.inspiration];

    // Get the rune paths
    let r = Range::new(0, 5);
    let primary_ix = r.ind_sample(&mut rng);
    let secondary_ix = {
        let mut t = r.ind_sample(&mut rng);
        while t == primary_ix {
            t = r.ind_sample(&mut rng);
        }
        t
    };

    let primary_path = &path_arr[primary_ix];
    let secondary_path = &path_arr[secondary_ix];

    // Determine the secondary rune path tiers
    let r = Range::new(1, 4);
    let sec_ai = r.ind_sample(&mut rng);
    let sec_bi = {
        let mut t = r.ind_sample(&mut rng);
        while t == sec_ai {
            t = r.ind_sample(&mut rng);
        }
        t
    };

    macro_rules! tier {
        ($ix:ident) => (match $ix {
            1 => &secondary_path.tier1,
            2 => &secondary_path.tier2,
            3 => &secondary_path.tier3,
            _ => &secondary_path.tier1,
        })
    }

    let sec_a = tier!(sec_ai);
    let sec_b = tier!(sec_bi);

    // Finalise
    Ok((
        PrimaryTree {
            name: path_names[primary_ix].to_string(),
            keystone: rng.choose(&primary_path.keystone).ok_or(failure::err_msg("Empty keystone?"))?.to_string(),
            tier1: rng.choose(&primary_path.tier1).ok_or(failure::err_msg("Empty tier1?"))?.to_string(),
            tier2: rng.choose(&primary_path.tier2).ok_or(failure::err_msg("Empty tier2?"))?.to_string(),
            tier3: rng.choose(&primary_path.tier3).ok_or(failure::err_msg("Empty tier3?"))?.to_string(),
        },
        SecondaryTree {
            name: path_names[secondary_ix].to_string(),
            runes: (
                rng.choose(sec_a).ok_or(failure::err_msg("Empty secondary?"))?.to_string(),
                rng.choose(sec_b).ok_or(failure::err_msg("Empty secondary?"))?.to_string(),
            )
        },
    ))
}

fn open_file(name: &'static str) -> Result<String, Error> {
    let _path = &[BASE_PATH, "resources/", name, ".json"].concat();
    let path = Path::new(_path);
    let file = File::open(path).context("Unable to open file.")?;

    let mut reader = BufReader::new(file);
    let mut buf = String::new();

    reader.read_to_string(&mut buf)?;
    Ok(buf)
}

pub fn get_champion(name: &str) -> Result<Champion, Error> {
    let champions: Vec<Champion> = serde_json::from_str(&open_file("champions")?)?;

    for champ in champions {
        if champ.name == name {
            return Ok(champ);
        }
    }
    return Err(failure::err_msg("Unable to find champion"))
}

pub fn random_summoner_spell(map: Map) -> Result<String, Error> {
    let spells: SpellList = serde_json::from_str(&open_file("spells")?)?;

    let mut list = Vec::new();
    list.extend(spells.common);
    match map {
        Map::SummonersRift | Map::TwistedTreeline => {
            list.extend(spells.classic);
        }
        Map::HowlingAbyss => {
            list.extend(spells.abyss);
        }
    };

    let mut rng = rand::thread_rng();
    rng.choose(&list).map(|s| s.clone()).ok_or(failure::err_msg("No spell found"))
}
