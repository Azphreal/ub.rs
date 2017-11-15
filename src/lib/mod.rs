#[macro_use]
extern crate error_chain;
extern crate ordermap;
extern crate rand;
extern crate serde;
extern crate serde_json as json;

mod err {
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            Json(::json::Error);
        }
        errors {}
    }
}

use err::*;
use rand::Rng;
use rand::distributions::{IndependentSample, Range};

use std::path::Path;
use std::fs::File;
use std::io::{BufReader, Read};
use std::fmt;

// const BASE_PATH: &'static str = "/home/xeal/Documents/projects/ub/";
const BASE_PATH: &str = "";

#[derive(Debug)]
pub struct Item {
    pub name: String,
    pub cost: u64,
}

pub struct Champion {
    pub name: String,
    pub title: String,
}

pub struct PrimaryTree {
    pub name: String,
    pub keystone: String,
    pub tier1: String,
    pub tier2: String,
    pub tier3: String,
}

pub struct SecondaryTree {
    pub name: String,
    pub runes: (String, String),
}

#[derive(PartialEq)]
pub enum Map {
    SummonersRift,
    HowlingAbyss,
    TwistedTreeline,
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

pub fn random_champion() -> Result<Champion> {
    let json = open_json("champions").chain_err(|| "Failed to open champions file.")?;
    let v = json.as_array().chain_err(|| "Not an array: champions")?;
    let mut rng = rand::thread_rng();

    let champ = rng.choose(v)
        .chain_err(|| "Unable to choose a random champion")?;

    Ok(Champion {
        name: String::from(champ["name"].as_str().unwrap()),
        title: String::from(champ["title"].as_str().unwrap()),
    })
}

/// Returns a list of random items guarenteed to be different.
pub fn random_items(map: &Map, number: usize, include_jungle: bool) -> Result<Vec<Item>> {
    let json = open_json("items").chain_err(|| "Failed to open items file.")?;

    macro_rules! json_key {
        ($key:expr) => (json[$key].as_array().chain_err(||format!("Not an array: {}", $key))?);
    }
    macro_rules! merge {
        ($from:expr, $into:ident) => (for i in $from {$into.push(i)});
    }

    let list = match map {
        m @ &Map::SummonersRift | m @ &Map::TwistedTreeline => {
            let mer = if m == &Map::SummonersRift {
                "rift"
            } else {
                "treeline"
            };

            let mut new_con = Vec::new();
            merge!(json_key!("common"), new_con);
            merge!(json_key!("classic"), new_con);
            merge!(json_key!(mer), new_con);

            let mut rng = rand::thread_rng();
            new_con.push(rng.choose(json_key!("support").as_slice()).unwrap());
            if include_jungle {
                new_con.push(rng.choose(json_key!("jungle").as_slice()).unwrap());
            }

            new_con
        }
        &Map::HowlingAbyss => {
            let mut new_con = Vec::new();
            merge!(json_key!("common"), new_con);
            merge!(json_key!("abyss"), new_con);
            new_con
        }
    };

    let range = Range::new(0, list.len() - 1);
    let mut rng = rand::thread_rng();

    let mut items = Vec::new();
    let mut seen = Vec::new();
    while items.len() < number {
        let i = range.ind_sample(&mut rng);
        if !seen.contains(&i) {
            let k = list[i];
            items.push(Item {
                name: String::from(k["name"].as_str().chain_err(|| "Not a string")?),
                cost: k["cost"].as_u64().chain_err(|| "Not a u64")?,
            });
            seen.push(i);
        }
    }

    Ok(items)
}

/// Returns a single random item from a given category. Primarily to be used for
/// the boots item.
pub fn random_item_from_category(cat: &'static str) -> Result<Item> {
    let json = open_json("items").chain_err(|| "Failed to open items file.")?;
    let list = json[cat]
        .as_array()
        // .chain_err(|| &format!("Category doesn't exist: {}", cat))?;
        .chain_err(|| "Category doesn't exist")?;

    let mut rng = rand::thread_rng();
    let item = rng.choose(list.as_slice())
        .chain_err(|| "Nothing to choose")?;
    Ok(Item {
        name: String::from(item["name"].as_str().chain_err(|| "Not a string")?),
        cost: item["cost"].as_u64().chain_err(|| "Not a u64")?,
    })
}

/// Returns a primary and secondary rune tree.
pub fn random_rune_page() -> Result<(PrimaryTree, SecondaryTree)> {
    let json = open_json("runes").chain_err(|| "Failed to open runes file.")?;
    let mut rng = rand::thread_rng();

    let _keys: Vec<_> = json.as_object().unwrap().keys().collect();
    let primary_path = rng.choose(_keys.as_slice()).unwrap();
    let mut secondary_path = rng.choose(_keys.as_slice()).unwrap();
    while primary_path == secondary_path {
        secondary_path = rng.choose(_keys.as_slice()).unwrap();
    }

    let mut rune_stack = Vec::new();
    for tier in json[primary_path].as_object().unwrap().values() {
        rune_stack.push(tier[rng.gen_range(0, 2)].clone());
    }

    let mut _i = [0, 1, 2];
    rng.shuffle(&mut _i);

    let _sec: Vec<_> = json[secondary_path].as_object().unwrap().values().collect();
    rune_stack.push(_sec[_i[0]][rng.gen_range(0, 2)].clone());
    rune_stack.push(_sec[_i[1]][rng.gen_range(0, 2)].clone());

    let mut runes = rune_stack.iter().map(|r| String::from(r.as_str().unwrap()));
    macro_rules! pop_rune {
        () => (runes.next().chain_err(|| "Not enough runes added")?);
    }

    Ok((
        PrimaryTree {
            name: String::from(primary_path.as_str()),
            keystone: pop_rune!(),
            tier1: pop_rune!(),
            tier2: pop_rune!(),
            tier3: pop_rune!(),
        },
        SecondaryTree {
            name: String::from(secondary_path.as_str()),
            runes: (pop_rune!(), pop_rune!()),
        },
    ))
}

fn open_json(name: &'static str) -> Result<json::Value> {
    let _path = &[BASE_PATH, "resources/", name, ".json"].concat();
    let path = Path::new(_path);
    let file = File::open(path).chain_err(|| "Unable to open file.")?;

    let mut reader = BufReader::new(file);
    let mut buf = String::new();

    reader.read_to_string(&mut buf)?;
    let j: json::Value = json::from_str(&buf).chain_err(|| "Failed to parse JSON.")?;
    Ok(j)
}

pub fn get_champion(name: &'static str) -> Result<Champion> {
    let json = open_json("champions").chain_err(|| "Failed to open champions file.")?;
    let v = json.as_array()
        .chain_err(|| "Not an array: champions")?
        .iter();

    for champ in v {
        if champ["name"] == name {
            return Ok(Champion {
                name: String::from(name),
                title: String::from(champ["title"].as_str().unwrap()),
            });
        }
    }
    Err(format!("No champion named {}", name).into())
}

pub fn get_summoner_spell(map: &Map) -> Result<String> {
    let json = open_json("spells").chain_err(|| "Failed to open spells file.")?;

    macro_rules! json_key {
        ($key:expr) => (json[$key].as_array().chain_err(||format!("Not an array: {}", $key))?);
    }
    macro_rules! merge {
        ($from:expr, $into:ident) => (for i in $from {$into.push(i)});
    }

    let list = match *map {
        Map::SummonersRift | Map::TwistedTreeline => {
            let mut new_con = Vec::new();
            merge!(json_key!("common"), new_con);
            merge!(json_key!("classic"), new_con);
            new_con
        }
        Map::HowlingAbyss => {
            let mut new_con = Vec::new();
            merge!(json_key!("common"), new_con);
            merge!(json_key!("abyss"), new_con);
            new_con
        }
    };

    let mut rng = rand::thread_rng();
    match rng.choose(list.as_slice()) {
        Some(s) => Ok(String::from(s.as_str().chain_err(|| "Not a string")?)),
        None => Err("Nothing to choose from".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // random_items(Map::SummonersRift, 5).expect("Didn't work.");
        // random_rune_page();
        panic!();
    }
}
