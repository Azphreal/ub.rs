# Ultimate Bravery

Ultimate Bravery is a "fun" custom game mode for League of Legends.

It has the player build a randomised build, including champion, summoner spells, runes, and ability order. The most well-known site for it is [here](http://www.ultimate-bravery.com/); however, it has not been updated in a long time, and contains many un-purchasable items.

# ub.rs

`ub.rs` is a Rust implementation for Ultimate Bravery. It features random builds, season 7 runes, and summoner spells.

It takes into account the following when deciding a build:

- map, configurable with the `-m` flag: Some items are locked to certain maps, so it won't give you Face of the Mountain on Howling Abyss.
- unique items: Jungle and support items are one-only, and you will only ever receive one (maybe of each!).
- suggestive smite: Jungle items won't be given unless you have also been given smite. In those rare games where your team decides you need a jungler, you can gamble and force the build to give smite and a jungle item with `-j`.

All of the program's runtime data is stored in JSON files in the resources folder. This makes it easily updatable when item costs are changed or new champions are added.

# Usage

Extremely simple. `./ub` or `ub.exe` to get a fully random build.

Optional flags:

- `-m` / `--map`: Specify a map, between one of `rift`, `abyss`, or `treeline`.
- `-c` / `--champion`: Specify a champion, mostly for ARAM. Currently not working due to me and Rust having a disagreement. **Case- and punctuation-sensitive**. Escape those apostrophes and spaces.
- `-j` / `--force_jungle`: Force a build to be made with smite and a jungle item.
- `-R` / `--no_runes`: Don't generate a rune page. For less fun?
- `-S` / `--no_skill`: Don't generate a skill order.

# What's currently not available

- Terminal app, terminal restrictions. No pretty pictures, unfortunately. The library can be extended if someone wants to implement a GUI frontend for it.
- The original site had a fun little adjective to describe your build. This can be added but has not yet been.

# What's currently a bit odd

- Random skill order currently includes all abilities. This will be, in future, a flag to use one (default), basic, or all abilities in the order.
- Champion specification isn't working.
- Some champions have some special items or restrictions that aren't in place. This include:
    - Viktor must have a Hex Core item.
    - Casseopeia cannot buy boots.
    - Gangplank has upgrades for his ult.
- Trinkets currently aren't generated. This is opinionated and some people believe you shouldn't have a trinket, to preserve the "no wards" playstyle of the original. However, it will be added as an option.
- Melee/ranged-only items (e.g., Sterak's Gage or Runaan's Hurricane) aren't filtered. This requires a restructure of the data format to allow and may or may not happen given my motivation.
- It may be possible to end up with two smites when forcing jungle.
