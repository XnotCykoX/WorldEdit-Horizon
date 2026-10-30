# WorldEdit, native

The WorldEdit editor as a **native mod**: a [horizon-mixins](https://github.com/horizon-rust/horizon-mixins) bundle that becomes
part of the game binary instead of running as a sandboxed WebAssembly mod. It is the same command set as
`mods/official/worldedit` (the `.hmod`), but it runs inside the game that owns the world, so a fill writes
blocks through `Game::level_set` directly: no command, no packet, no block log, no command-length or
command-count limit.

## What the bundle does

`horizon-mixin.toml` declares two modules and four hooks:

| Piece | What it is |
|---|---|
| `runtime.rs` | `mod worldedit` staged into `crates/horizon-client/src/game.rs`: the editor (selection, region maths, clipboard, shapes, undo). |
| `command.rs` | HEAD on `Game<'_>::run_command`: a `//command` typed on this machine. |
| `guest-command.rs` | HEAD on `Game<'_>::guest_command`: a `//command` from a guest of a hosted world, queued as typed so the server runs it with that player's state in place (`as_guest`). |
| `wand.rs` | HEAD on `Game<'_>::play_mouse`: the wand, taken before mining or item use starts. |
| `chat-suggestions.rs` | `mod worldedit_chat` staged into `crates/horizon-client/src/chat.rs`: the command table and the completion pass for the `//` prefix. |
| `chat-hook.rs` | HEAD on `ChatScreen::update_command_info`: completes `//commands` instead of parsing them as vanilla's unknown literal. |

The helpers are a few lines each: all the logic is in the modules, which are compiled inside the game
crate and therefore see the game's private fields and methods (no game to mod Cargo dependency).

## The wand

`//wand` puts a wooden axe named **WorldEdit Wand** in your inventory. With that axe held, **left click**
set position 1 and **right click** sets position 2, on the block you are looking at (`Game::pick`, so it
uses the game's own reach and outline raycast); the click does not mine, place or open anything. An
ordinary wooden axe is an ordinary axe, and the name is what marks the wand, so a renamed axe is a new
item to the editor. In single player on the server thread and on a LAN client, the click becomes the
command the player would have typed: a client sends it, the server's copy of the module runs it.

## Completions

The vanilla command tree does not know `//set`, so the chat screen used to answer `//` with "Unknown or
incomplete command" and completed nothing. The bundle takes over that prefix: names complete
(`//po` + Tab → `//pos1`), block arguments complete from the game's own block registry (`//set sto` + Tab →
`//set stone`), the direction arguments complete their six words, and the usage line is shown instead of a
parse error. Single-slash commands stay with the game.

## Commands

```
//pos1 //pos2 [x y z]   (~ for where you stand; no arguments: your feet)
//sel [clear]           //wand, //we
//expand //contract <amount> [up|down|north|south|east|west]
//shift <amount> <dir>
//set <block>            //replace <from> [to]
//walls //outline //hollow //faces //overlay //center <block>
//sphere //hsphere <block> <radius>
//cyl //hcyl <block> <radius> [height]
//pyramid //hpyramid <block> <size>
//line <block> [thickness]
//copy //cut //paste //move <amount> <dir>
//count <block> //distr
//undo //redo //clearhistory
```

Block names are `stone` or `minecraft:oak_stairs[facing=east]`; properties are optional and an exact state
is matched when you give them.

## Patterns

Every command that takes a block takes a **pattern**: a comma-separated list, picked at random for each
block of the region, so `//set stone,dirt` or `//cyl acacia_wood,minecraft:stone 4 3` mixes them.

```
//set stone,dirt                  the two, half and half
//set 70%stone,30%gravel          weighted
//replace stone,dirt air          the mask takes a list too (weights ignored)
//cyl acacia_wood,minecraft:stone 4 3
```

Commas inside `[...]` belong to the block's properties, not to the list:
`oak_stairs[facing=east,half=top]` is one block. Each edit bumps a salt, so the same pattern spreads
differently every time, and the pick is a hash of the position, so it never depends on the order the region
is walked in. A single block is placed for every position as before.

One operation may touch at most 250,000 blocks (`WorldEdit`'s default), the last 10 edits can be undone,
and an edit never loads a chunk: unloaded chunks are skipped. Editing needs build access and permission
level 2 (`//we` and `//wand` do not).

## Build

The bundle is built by `horizon-mixins`, which stages a separate copy of Horizon and never edits the
checkout you are in:

```sh
git clone https://github.com/horizon-rust/horizon-mixins ../horizon-mixins
cargo build --manifest-path ../horizon-mixins/Cargo.toml
# Validate the plan against this Horizon tree (writes nothing into it):
../horizon-mixins/target/debug/horizon-mixins plan   --workspace "$PWD" --mods "$PWD/mixins/worldedit" --out ../we-plan.json
# Stage a patched copy and run the game crate's tests on it:
../horizon-mixins/target/debug/horizon-mixins check   --workspace "$PWD" --mods "$PWD/mixins/worldedit" --out ../we-stage   --trust-native-mods
```

`check` runs `cargo test --locked --offline`, so the workspace's dependencies must already be cached
(`cargo fetch --locked`). The stage is published atomically and the original checkout keeps its bytes and
mtimes.

The `.hnmod` the launcher installs is packed from this folder and the manifest beside it:

```sh
python3 tools/pack.py --check          # manifest, id, version and pin agree
python3 tools/pack.py --out ~/Desktop  # worldedit-native-1.0.0.hnmod
```

## Verified

Planned, staged, compiled (`cargo check -p horizon-client`) and run against this tree: in an offscreen single-player
game on its server thread, `//pos1 0 64 0`, `//pos2 2 66 2` and `//set stone` changed 27 blocks (confirmed with
vanilla `execute if block`), `//count stone` found them and `//undo` put them back; left click with the wand held
set position 1 to the block under the crosshair and did not mine it, while a plain wooden axe mined;
`//sphere glass 3` placed 123 blocks; `//set oak_stairs[facing=east]` wrote the state with that property.
Typed in the chat screen, `//po` + Tab became `//pos1`, `//set sto` + Tab became `//set stone` and
`//set stone,gl` + Tab completed only the second entry.

Patterns, over a 5x1x5 region: `//set stone,dirt` wrote 25 blocks as 13 stone and 12 dirt;
`//set 90%glass,10%gold_block` wrote 20 glass and 5 gold; `//replace glass,gold_block air` cleared all 25;
`//cyl acacia_wood,minecraft:stone 4 3` wrote 147 blocks as 76 acacia and 71 stone; and
`oak_stairs[facing=east,half=top]` stayed one block, its property comma intact.

## Compatibility

Native hooks are raw function paths (`Game<'_>::run_command`, `Game<'_>::guest_command`,
`Game<'_>::play_mouse`, `ChatScreen::update_command_info`), so a bundle is tied to the Horizon revision it was
planned against, and the manifest requires one: `horizon-native.json`'s `horizon` must equal
`compatibility.horizon_revision`, or the launcher refuses the archive.

This bundle pins **Nightly 2026-10-28**, `83f15fa276c44711276b6ceb5f0fe0c9c534dc66`: `plan` there resolved
its two modules and four hooks, and it was compiled and played on a tree of that Nightly. On another commit,
set both fields to that commit and run `plan`; a hook whose function moved or whose signature changed is
reported, never skipped.

The `native_*` hook points in `crates/horizon-client/src/game/native_hooks.rs` are not compiled into this
tree (no `mod native_hooks;` declares them and there is no `native-hooks.json` catalogue), so this bundle
hooks the game's own functions instead of named hooks.

The chat input, the console and the guest command packet each drop a different number of leading slashes
before a command reaches `run_command`, so commands are matched by name (`//set` and `set` both work). A
name the vanilla dispatcher already has at its root is left alone; `move` belongs to `//clone`, not the
root, so `//move` is this mod's.

## Not in it

Brushes and tool bindings, `#clipboard`/`#existing`-style pattern sources, probability masks, `//deform`,
`//smooth`, `//naturalize`, `//stack`, `//regen`, biome tools, and `.schem`/`.schematic` files. The selection is not
drawn in the world (there is no world-space line renderer to hook yet): `//sel`, `//pos1` and `//pos2`
report it in chat instead. The `.hmod` in `mods/official/worldedit` keeps `//save`/`//load` into the mod's
own storage.
