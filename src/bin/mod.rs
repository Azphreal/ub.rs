#[macro_use]
extern crate clap;
extern crate rand;
extern crate ub;
#[macro_use]
extern crate error_chain;

use rand::Rng;

mod app {
    use clap::{App, Arg};

    // Input args for: map, champion
    // Flags for: force jungling, runes, skill order
    pub fn app() -> App<'static, 'static> {
        App::new(crate_name!())
            .version(crate_version!())
            .author(crate_version!())
            .about(crate_description!())
            .arg(
                Arg::with_name("map")
                    .short("m")
                    .long("map")
                    .help(
                        "Map to play on \
                         Values: rift, abyss, treeline",
                    )
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("champion")
                    .short("c")
                    .long("champion")
                    .help("Use a specific champion (case sensitive)")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("jungle")
                    .short("j")
                    .long("force_jungle")
                    .help("Force the build to include smite and a jungle item"),
            )
            .arg(
                Arg::with_name("runes")
                    .short("r")
                    .long("gen_runes")
                    .help("Generate a rune page (default)"),
            )
            .arg(
                Arg::with_name("no_runes")
                    .short("R")
                    .long("no_gen_runes")
                    .help("Don't generate a rune page"),
            )
            .arg(
                Arg::with_name("skills")
                    .short("s")
                    .long("skills")
                    .help("Length of skill max order (default 1)")
                    .takes_value(true)
            )
            .arg(
                Arg::with_name("no_skill")
                    .short("S")
                    .long("no_skill_order")
                    .help("Don't generate a skill order"),
            )
            .arg(
                Arg::with_name("trinket")
                    .short("t")
                    .long("trinket")
                    .help("Generate a trinket"),
            )
    }
}

mod err {
    error_chain! {
        foreign_links {
            UbLib(::ub::err::Error);
            Int(::std::num::ParseIntError);
        }
        errors {}
    }
}

use ub::*;
use err::*;

macro_rules! summoner {
    ($map:ident) => (random_summoner_spell(&$map).chain_err(|| "Unable to get summoner spell")?);
}

macro_rules! item_from {
    ($cat:expr) => (random_item_from_category($cat).chain_err(|| ["Unable to get ", $cat].concat())?);
}

macro_rules! special {
    ($cat:expr) => (random_item_from_special($cat).chain_err(|| ["Unable to get ", $cat, " item"].concat())?);
}

macro_rules! random_slice {
    ($map:ident, $n:expr, $extra:expr, $jung:expr) => (random_items(&$map, $n, $extra, $jung).chain_err(|| "Unable to get random items")?.as_slice());
}

fn run() -> Result<()> {
    let matches = app::app().get_matches();

    // Determine map. Default to Summoner's Rift.
    let map = if matches.is_present("map") {
        match matches.value_of("map") {
            Some("rift") => Map::SummonersRift,
            Some("abyss") => Map::HowlingAbyss,
            Some("treeline") => Map::TwistedTreeline,
            _ => {
                println!(
                    "Invalid value given for map. \
                     See the help for valid values."
                );
                std::process::exit(1);
            }
        }
    } else {
        Map::SummonersRift
    };

    // Determine spells.
    let _spell1 = if matches.is_present("jungle") {
        String::from("Smite")
    } else {
        summoner!(map)
    };
    let spells =  {
        let mut _spell2 = summoner!(map);
        while _spell1 == _spell2 {
            _spell2 = summoner!(map);
        }
        (_spell1, _spell2)
    };

    // Determine champion.
    // let champ = match matches.value_of("champion") {
    //     Some(ch) => get_champion(ch).expect("No matching champion"),
    //     None => match random_champion() {
    //         Ok(c) => c,
    //         Err(_) => {
    //             println!("Invalid value given for champion. \
    //                       Remember that it's case sensitive.");
    //             std::process::exit(1)
    //         }
    //     },
    // };

    let champ = random_champion()
        .chain_err(|| "Invalid value for champion. \
                       Remember that it's case sensitive.")?;

    let mut items = Vec::new();

    // Set up base items based on the champion.
    match champ.name.as_str() {
        "Casseopeia" => (),
        "Viktor" => {
            items.push(item_from!("boots"));
            items.push(special!("Viktor"));
        },
        "Ornn" => {
            items.push(item_from!("boots"));
            items.push(special!("Ornn"));
        },
        _ => items.push(item_from!("boots"))
    }

    let rem_items = 6 - items.len();
    if matches.is_present("jungle") {
        items.push(item_from!("jungle"));
        items.extend_from_slice(
            random_slice!(map, rem_items - 1, &champ.range, false));
    } else if spells.0 == "Smite" || spells.1 == "Smite" {
        items.extend_from_slice(
            random_slice!(map, rem_items, &champ.range, true));
    } else {
        items.extend_from_slice(
            random_slice!(map, rem_items, &champ.range, false));
    }

    println!("{}", items.len());

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
                map = String::from(map),
                champ = String::from(champ.clone()),
                spells = spells.0 + ", " + &spells.1};
    println!();

    // Split 36 and 42.
    if !matches.is_present("no_runes") {
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

    if matches.is_present("trinket") {
        println!("  {}", item_from!("trinket"));
    }

    println!();
    print!(
        "  Total cost: {cost: <21}",
        cost = cost.to_string() + " gold"
    );

    if !matches.is_present("no_skill") {
        let mut rng = rand::thread_rng();

        let mut skills = if champ.name.as_str() == "Udyr" {
            vec!('Q', 'W', 'E', 'R')
        } else if champ.name.as_str() == "Jayce" {
            vec!('Q', 'W', 'E')
        } else if matches.value_of("skills") == Some("4") {
            vec!('Q', 'W', 'E', 'R')
        } else {
            vec!('Q', 'W', 'E')
        };

        let mut num_skills = if matches.is_present("skills") {
            let _n = matches.value_of("skills").unwrap().parse()?;
            if _n > 4 || _n < 0 { 1 } else { _n }
        } else {
            1
        };

        rng.shuffle(&mut skills.as_mut_slice());

        let mut order = String::new();
        for (ix, s) in skills.iter().enumerate() {
            if num_skills == 0 {
                break
            }
            order.push(*s);
            if ix < skills.len() - 1 && num_skills > 1 {
                order.push_str(" -> ");
            }
            num_skills -= 1;
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

quick_main!(run);
