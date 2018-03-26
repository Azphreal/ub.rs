#[macro_use]
extern crate structopt_derive;
extern crate structopt;
extern crate rand;
extern crate ub;
extern crate failure;

use rand::Rng;
use failure::{Error, ResultExt};
use structopt::StructOpt;

use ub::*;

mod app {
    use ub::Map;

    #[derive(StructOpt, Debug)]
    #[structopt(name = "ub")]
    pub struct App {
        #[structopt(short = "m", long = "map", help = "Map to play on", default_value = "rift")]
        pub map: Map,
        #[structopt(short = "c", long = "champion", help = "Use a specific champion (case sensitive)")]
        pub champion: Option<String>,
        #[structopt(short = "j", long = "force_jungle",
                    help = "Force the build to include smite and a jungle item")]
        pub jungle: bool,
        #[structopt(short = "R", long = "no_runes", help = "Don't generate a rune page")]
        pub no_runes: bool,
        #[structopt(short = "s", long = "skills", help = "Length of skill max order", default_value = "1")]
        pub skills: f64,
        #[structopt(short = "t", long = "trinket", help = "Generate a trinket")]
        pub trinket: bool,
    }
}


fn run() -> Result<(), Error> {
    let args = app::App::from_args();

    // Determine spells.
    let _spell1 = if args.jungle {
        String::from("Smite")
    } else {
        random_summoner_spell(args.map)?
    };
    let spells =  {
        let mut _spell2 = random_summoner_spell(args.map)?;
        while _spell1 == _spell2 {
            _spell2 = random_summoner_spell(args.map)?;
        }
        (_spell1, _spell2)
    };

    let champ = if let Some(c) = args.champion {
        get_champion(&c)
    } else {
        random_champion()
    }.context("Invalid value for champion. \
               Remember that it's case sensitive.")?;

    let mut items = Vec::new();

    // Set up base items based on the champion.
    match champ.name.as_str() {
        "Casseopeia" => (),
        "Viktor" => {
            items.push(random_item_from_category("boots")?);
            items.push(random_item_from_category("Viktor")?);
        },
        "Ornn" => {
            items.push(random_item_from_category("boots")?);
            items.push(random_item_from_category("Ornn")?);
        },
        _ => items.push(random_item_from_category("boots")?)
    }

    let rem_items = 6 - items.len();
    if args.jungle {
        items.push(random_item_from_category("jungle")?);
        items.extend(random_items(args.map, rem_items - 1, &champ.range, false)?);
    } else if spells.0 == "Smite" || spells.1 == "Smite" {
        items.extend(random_items(args.map, rem_items, &champ.range, true)?);
    } else {
        items.extend(random_items(args.map, rem_items, &champ.range, false)?);
    }

    // Print it all.
    println!{"
  _   _ _ _   _                 _         ____
 | | | | | |_(_)_ __ ___   __ _| |_ ___  | __ ) _ __ __ ___   _____ _ __ _   _
 | | | | | __| | '_ ` _ \\ / _` | __/ _ \\ |  _ \\| '__/ _` \\ \\ / / _ \\ '__| | | |
 | |_| | | |_| | | | | | | (_| | ||  __/ | |_) | | | (_| |\\ V /  __/ |  | |_| |
  \\___/|_|\\__|_|_| |_| |_|\\__,_|\\__\\___| |____/|_|  \\__,_| \\_/ \\___|_|   \\__, |
                                                                         |___/
 {champ: ^78}

  Map: {map: >25}    Summoners: {spells: >31}",
                map = args.map.to_string(),
                champ = champ.to_string(),
                spells = spells.0 + ", " + &spells.1};
    println!();

    // Split 36 and 42.
    if !args.no_runes {
        let (pri, sec) = random_rune_page().expect("Unable to get rune page");

        // Bold the headers on Unix.
        #[cfg(unix)]
        println!(
            "  {bold}{itemt: ^33} {runet: ^42}{unbold}",
            bold = "\u{1b}[1;30m",
            unbold = "\u{1b}[0m",
            itemt = "Items",
            runet = "Runes"
        );

        // Can't on Windows.
        #[cfg(windows)]
        println!(
            "  {itemt: ^33} {runet: ^42}",
            itemt = "Items",
            runet = "Runes"
        );

        println!(
            "  {item0: <33}   {runet1: <18}  {runet2: >18}
  {item1: <33} {runek: <20}
  {item2: <33} {rune11: <20}  {rune21: >20}
  {item3: <33} {rune12: <20}  {rune22: >20}
  {item4: <33} {rune13: <20}
  {item5: <33}",
            item0 = items[0].name,
            item1 = items[1].name,
            item2 = items[2].name,
            item3 = items[3].name,
            item4 = items[4].name,
            item5 = items[5].name,
            runet1 = pri.name.to_uppercase(),
            runek = pri.keystone,
            rune11 = pri.tier1,
            rune12 = pri.tier2,
            rune13 = pri.tier3,
            runet2 = sec.name.to_uppercase(),
            rune21 = sec.runes.0,
            rune22 = sec.runes.1
        );
    } else {
        #[cfg(unix)]
        println!(
            "   {bold}{itemt: ^72}{unbold}",
            bold = "\u{1b}[1;30m",
            unbold = "\u{1b}[0m",
            itemt = "Items"
        );

        // Can't on Windows.
        #[cfg(windows)]
        println!("   {itemt: ^72}", itemt = "Items",);

        println!(
            "  {item0: ^76}
  {item1: ^76}
  {item2: ^76}
  {item3: ^76}
  {item4: ^76}
  {item5: ^76}",
            item0 = items[0].name,
            item1 = items[1].name,
            item2 = items[2].name,
            item3 = items[3].name,
            item4 = items[4].name,
            item5 = items[5].name,
        );
    }

    let cost = items.iter().fold(0, |acc, i| acc + i.cost);

    if args.trinket {
        println!("  {}", random_item_from_category("trinket")?);
    }

    println!();
    print!(
        "  Total cost: {cost: <21}",
        cost = cost.to_string() + " gold"
    );

    if true {
        let mut rng = rand::thread_rng();

        let mut skills = if champ.name.as_str() == "Udyr" {
            vec!('Q', 'W', 'E', 'R')
        } else if champ.name.as_str() == "Jayce" {
            vec!('Q', 'W', 'E')
        } else if args.skills == 4. {
            vec!('Q', 'W', 'E', 'R')
        } else {
            vec!('Q', 'W', 'E')
        };

        rng.shuffle(&mut skills.as_mut_slice());

        let mut order = String::new();
        let mut ix = args.skills;
        for (i, s) in skills.iter().enumerate() {
            if ix == 0. {
                break
            }
            order.push(*s);
            if i < skills.len() - 1 && ix > 1. {
                order.push_str(" -> ");
            }
            ix -= 1.;
        }

        if order.len() == 1 {
            print!(" Max first: {}", order);
        } else {
            print!(" Skill order: {}", order);
        }
    }
    println!();
    println!();

    Ok(())
}

fn main() {
    match run() {
        Ok(_) => (),
        Err(e) => {
            println!("Fatal error: {}", e);
            ::std::process::exit(1);
        }
    }
}
