# WorldEdit for Horizon, native

The WorldEdit editor as a **native mod** for [Horizon](https://github.com/horizon-rust/horizon): a
[horizon-mixins](https://github.com/horizon-rust/horizon-mixins) bundle that is compiled into the game binary
instead of running as a sandboxed WebAssembly mod. It edits the world the game owns directly — a fill writes
blocks through the game's own `set_block`, with no command, no packet, no block log and no command-length
limit — and it works in single player (server thread or in-frame), on a LAN host and on a guest of one.

```
//wand                       a wooden axe called "WorldEdit Wand"
left click / right click    position 1 / position 2, with that axe in hand
//pos1 //pos2 [x y z]        or without it, `~` for where you stand
//set stone,dirt,70%gravel   a region, a shape, a pattern
//cyl acacia_wood,minecraft:stone 10 20
//copy //paste //undo //redo
```

## Commands

```
//pos1 //pos2 [x y z]   //sel [clear]   //wand   //we
//expand //contract <amount> [up|down|north|south|east|west]   //shift <amount> <dir>
//set <pattern>         //replace <mask> [pattern]
//walls //outline //hollow //faces //overlay //center <pattern>
//sphere //hsphere <pattern> <radius>      //cyl //hcyl <pattern> <radius> [height]
//pyramid //hpyramid <pattern> <size>      //line <pattern> [thickness]
//copy //cut //paste //move <amount> <dir>
//count <mask> //distr                    //undo //redo //clearhistory
```

A **pattern** is one block (`stone`, `minecraft:oak_stairs[facing=east]`) or a comma-separated list picked at
random for every block: `//set stone,dirt`, `//set 70%stone,30%gravel`, `//cyl acacia_wood,minecraft:stone 4 3`.
Commas inside `[...]` stay with the block's properties. A mask (`//replace`) takes a list too.

One edit touches at most 250,000 blocks, the last 10 edits undo, an edit never loads a chunk, and editing
needs build access and permission level 2 (`//we` and `//wand` do not).

`//` completes in the chat screen: names, block names from the game's registry, directions, and the usage
line instead of vanilla's "Unknown or incomplete command".

## Install with the launcher

1. Download `worldedit-native-1.0.0.hnmod` (or pack it: `python3 tools/pack.py`).
2. Launcher → Mods → install the `.hnmod`, then Settings → Game → **Native mods (experimental)**, then **Build**.
3. Play the **Native mods** instance the build creates.

Native mods are native code with the player's rights: the launcher asks you to trust them before it builds.

## Build it into your own Horizon tree

```sh
git clone https://github.com/horizon-rust/horizon-mixins
cargo build --manifest-path horizon-mixins/Cargo.toml
horizon-mixins/target/debug/horizon-mixins plan \
  --workspace ~/horizon --mods "$PWD/mixins/worldedit" --out plan.json
horizon-mixins/target/debug/horizon-mixins check \
  --workspace ~/horizon --mods "$PWD/mixins/worldedit" --out ~/horizon-we --trust-native-mods
cd ~/horizon-we && cargo build --profile fast -p horizon-client --features horizon-client/mods
```

`plan` validates every hook against the tree without writing to it; `check` stages a patched copy and runs
its tests; the last line is a game with WorldEdit in it.

## Layout

```
horizon-native.json          the .hnmod manifest: id, version, game, the Horizon and horizon-mixins commits
mixins/worldedit/
  horizon-mixin.toml         two modules and four hooks
  runtime.rs                 `mod worldedit`: selection, patterns, masks, shapes, clipboard, undo
  command.rs                 a `//command` typed here (HEAD on `run_command`)
  guest-command.rs           a `//command` from a LAN guest or a server client (HEAD on `guest_command`)
  wand.rs                    the wand's clicks (HEAD on `play_mouse`)
  chat-suggestions.rs        `mod worldedit_chat`: the `//` command table and its completions
  chat-hook.rs               the chat screen's completion pass (HEAD on `update_command_info`)
tools/pack.py                packs the .hnmod, checking the manifest against the bundle
```

## Compatibility

Native hooks are raw function paths, so a bundle is tied to the Horizon source it is planned against. This
one pins **Nightly 2026-10-28** (`83f15fa`); it was planned, compiled and played on a tree of that Nightly.
Horizon moves fast: on another commit, change `compatibility.horizon_revision` in `mixins/worldedit/horizon-mixin.toml`
(and `horizon` in `horizon-native.json`) to that commit, and re-run `plan` — a hook whose function moved or
whose signature changed is refused, never silently skipped.

## Not in it

Brushes and tool bindings, `#clipboard`-style pattern sources, probability or expression masks, `//deform`,
`//smooth`, `//naturalize`, `//stack`, `//regen`, biome tools, `.schem`/`.schematic` files, and drawing the
selection in the world (there is no world-space line renderer to hook): `//sel`, `//pos1` and `//pos2` report
the selection in chat.

## License

MIT — see [LICENSE](LICENSE).

## Written with an agent

The bundle was written with an AI coding agent (Codebuff) against a Horizon 26.3 tree, at XnotCykoX's
direction: the commands, the patterns, the wand and the completions were each checked in an offscreen game
before landing here.
