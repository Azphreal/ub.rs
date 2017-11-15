#[macro_use]
extern crate clap;
extern crate ub;

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
                    .long("Generate a rune page (default)"),
            )
            .arg(
                Arg::with_name("no_runes")
                    .short("R")
                    .long("no_gen_runes")
                    .long("Don't generate a rune page"),
            )
    }
}

use ub::*;

fn main() {
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
    let _spell1 = get_summoner_spell(&map).expect("Unable to get summoner spell");
    let spells = if matches.is_present("jungle") {
        (String::from("Smite"), _spell1)
    } else {
        let mut _spell2 = get_summoner_spell(&map).expect("Unable to get summoner spell");
        while _spell1 == _spell2 {
            _spell2 = get_summoner_spell(&map).expect("Unable to get summoner spell");
        }
        (_spell1, _spell2)
    };

    // Determine champion.
    let champ = /*match matches.value_of("champion") {
        Some(ch) => get_champion(ch).expect("No matching champion"),
        None =>*/ match random_champion() {
            Ok(c) => c,
            Err(_) => {
                println!("Invalid value given for champion. \
                          Remember that it's case sensitive.");
                std::process::exit(1);
            }
        // }
    };

    let mut items = Vec::new();

    items.push(random_item_from_category("boots").expect("Unable to get boots"));
    if matches.is_present("jungle") {
        // Force a jungle item.
        items.push(random_item_from_category("jungle").expect("Unable to get jungle item"));
        items.extend_from_slice(
            random_items(&map, 4, false)
                .expect("Unable to get items")
                .as_slice(),
        );
    } else if spells.0 == "Smite" || spells.1 == "Smite" {
        // Include jungle items only when it's possible to build them.
        items.extend_from_slice(
            random_items(&map, 5, true)
                .expect("Unable to get items")
                .as_slice(),
        );
    } else {
        items.extend_from_slice(
            random_items(&map, 5, false)
                .expect("Unable to get items")
                .as_slice(),
        );
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

    Map: {map: >23}    Summoners: {spells: >31}",
             map = String::from(map),
             champ = String::from(champ),
             spells = spells.0 + ", " + &spells.1};
    println!();

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
        println!(
            "   {itemt: ^72}",
            itemt = "Items",
        );

        println!(
            "    {item0: ^72}
    {item1: ^72}
    {item2: ^72}
    {item3: ^72}
    {item4: ^72}
    {item5: ^72}",
            item0 = items[0].name,
            item1 = items[1].name,
            item2 = items[2].name,
            item3 = items[3].name,
            item4 = items[4].name,
            item5 = items[5].name,
        );
    }
}
