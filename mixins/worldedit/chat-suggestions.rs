// The editor's commands in the chat screen: the vanilla tree parses `//set` as
// the literal `/set` and the screen answers with "Unknown or incomplete
// command", so nothing completes and the line is red. This pass takes over the
// whole `//` prefix in `update_command_info` (which returns early), completing
// command names and their arguments, and showing the usage line instead of a
// parse error. `//commands` never reach the vanilla parser.
use super::worldedit_paths;
use super::*;

/// What a position in a command's arguments completes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ArgKind {
    /// A block id (`stone`, `oak_stairs[facing=east]`'s name).
    Block,
    /// A count, radius or height: nothing to suggest.
    Number,
    /// `up`, `down`, `north`, `south`, `east`, `west`.
    Direction,
    /// `clear`.
    Clear,
    /// `//schem`'s own words.
    Schem,
    /// A file in the world's schematics folder, and a name to save as.
    SchemFile,
}

/// What `//schem` takes after its name.
const SCHEM_WORDS: [&str; 5] = ["list", "load", "save", "delete", "folder"];

/// The schematics of the world this game last opened, for `//schem load` and
/// `//load`: the names without their extensions, once each.
fn schematic_names() -> Vec<String> {
    worldedit_paths::schematics()
        .map(|dir| worldedit_paths::names(&dir))
        .unwrap_or_default()
}

/// One of the editor's commands: its name (without slashes), its usage line and
/// its arguments in order.
struct Command {
    name: &'static str,
    usage: &'static str,
    args: &'static [ArgKind],
    /// Takes `-h` before its first argument (`//sphere -h stone 4`).
    hollow: bool,
}

const COMMANDS: &[Command] = &[
    Command {
        name: "we",
        usage: "//we — every WorldEdit command",
        args: &[],
        hollow: false,
    },
    Command {
        name: "wand",
        usage: "//wand — the wooden axe that selects with a click",
        args: &[],
        hollow: false,
    },
    Command {
        name: "pos1",
        usage: "//pos1 [x y z] (~ for where you stand)",
        args: &[ArgKind::Number; 3],
        hollow: false,
    },
    Command {
        name: "pos2",
        usage: "//pos2 [x y z] (~ for where you stand)",
        args: &[ArgKind::Number; 3],
        hollow: false,
    },
    Command {
        name: "sel",
        usage: "//sel [clear]",
        args: &[ArgKind::Clear],
        hollow: false,
    },
    Command {
        name: "expand",
        usage: "//expand <amount> [up|down|north|south|east|west]",
        args: &[ArgKind::Number, ArgKind::Direction],
        hollow: false,
    },
    Command {
        name: "contract",
        usage: "//contract <amount> [up|down|north|south|east|west]",
        args: &[ArgKind::Number, ArgKind::Direction],
        hollow: false,
    },
    Command {
        name: "shift",
        usage: "//shift <amount> <up|down|north|south|east|west>",
        args: &[ArgKind::Number, ArgKind::Direction],
        hollow: false,
    },
    Command {
        name: "set",
        usage: "//set <block>[,<block>]",
        args: &[ArgKind::Block],
        hollow: false,
    },
    Command {
        name: "replace",
        usage: "//replace <from>[,<from>] [to]",
        args: &[ArgKind::Block, ArgKind::Block],
        hollow: false,
    },
    Command {
        name: "walls",
        usage: "//walls <block>[,<block>]",
        args: &[ArgKind::Block],
        hollow: false,
    },
    Command {
        name: "outline",
        usage: "//outline <block>[,<block>]",
        args: &[ArgKind::Block],
        hollow: false,
    },
    Command {
        name: "hollow",
        usage: "//hollow <block>[,<block>]",
        args: &[ArgKind::Block],
        hollow: false,
    },
    Command {
        name: "faces",
        usage: "//faces <block>[,<block>]",
        args: &[ArgKind::Block],
        hollow: false,
    },
    Command {
        name: "overlay",
        usage: "//overlay <block>[,<block>]",
        args: &[ArgKind::Block],
        hollow: false,
    },
    Command {
        name: "center",
        usage: "//center <block>[,<block>]",
        args: &[ArgKind::Block],
        hollow: false,
    },
    Command {
        name: "count",
        usage: "//count <block>[,<block>]",
        args: &[ArgKind::Block],
        hollow: false,
    },
    Command {
        name: "distr",
        usage: "//distr — the blocks of the region by type",
        args: &[],
        hollow: false,
    },
    Command {
        name: "sphere",
        usage: "//sphere [-h] <block>[,<block>] <radius>",
        args: &[ArgKind::Block, ArgKind::Number],
        hollow: true,
    },
    Command {
        name: "hsphere",
        usage: "//hsphere [-h] <block>[,<block>] <radius>",
        args: &[ArgKind::Block, ArgKind::Number],
        hollow: true,
    },
    Command {
        name: "cyl",
        usage: "//cyl [-h] <block>[,<block>] <radius> [height]",
        args: &[ArgKind::Block, ArgKind::Number, ArgKind::Number],
        hollow: true,
    },
    Command {
        name: "hcyl",
        usage: "//hcyl [-h] <block>[,<block>] <radius> [height]",
        args: &[ArgKind::Block, ArgKind::Number, ArgKind::Number],
        hollow: true,
    },
    Command {
        name: "pyramid",
        usage: "//pyramid [-h] <block>[,<block>] <size>",
        args: &[ArgKind::Block, ArgKind::Number],
        hollow: true,
    },
    Command {
        name: "hpyramid",
        usage: "//hpyramid [-h] <block>[,<block>] <size>",
        args: &[ArgKind::Block, ArgKind::Number],
        hollow: true,
    },
    Command {
        name: "line",
        usage: "//line <block>[,<block>] [thickness]",
        args: &[ArgKind::Block, ArgKind::Number],
        hollow: false,
    },
    Command {
        name: "copy",
        usage: "//copy — the selection to the clipboard",
        args: &[],
        hollow: false,
    },
    Command {
        name: "cut",
        usage: "//cut — copy, then clear the selection",
        args: &[],
        hollow: false,
    },
    Command {
        name: "paste",
        usage: "//paste — the clipboard, from where you stand",
        args: &[],
        hollow: false,
    },
    Command {
        name: "move",
        usage: "//move <amount> <up|down|north|south|east|west>",
        args: &[ArgKind::Number, ArgKind::Direction],
        hollow: false,
    },
    Command {
        name: "schem",
        usage: "//schem — the menu; //schem <list|load <file>|save [name]|delete <file>|folder>",
        args: &[ArgKind::Schem, ArgKind::SchemFile],
        hollow: false,
    },
    Command {
        name: "load",
        usage: "//load <file> — a schematic into the clipboard",
        args: &[ArgKind::SchemFile],
        hollow: false,
    },
    Command {
        name: "undo",
        usage: "//undo — the last edit",
        args: &[],
        hollow: false,
    },
    Command {
        name: "redo",
        usage: "//redo — the undone edit",
        args: &[],
        hollow: false,
    },
    Command {
        name: "clearhistory",
        usage: "//clearhistory — forget every undo step",
        args: &[],
        hollow: false,
    },
];

const DIRECTIONS: [&str; 6] = ["up", "down", "north", "south", "east", "west"];
/// What the screen says for a `//` line no command starts.
const HINT: &str = "WorldEdit: //we lists every command";

/// The block ids to complete with: naked (`stone`) unless the word already has
/// a namespace, as the game's own block arguments suggest them.
fn block_names<'a>(c: &'a Commands<'_>, typed: &str) -> Vec<&'a str> {
    let namespaced = typed.contains(':');
    c.registries
        .blocks
        .keys()
        .map(String::as_str)
        .map(|id| {
            if namespaced {
                id
            } else {
                id.strip_prefix("minecraft:").unwrap_or(id)
            }
        })
        .collect()
}

/// The block being typed: after the last `,` outside `[...]`, and after a
/// `70%` weight when there is one. Returns its byte offset in `word`.
fn block_segment(word: &str) -> (usize, &str) {
    let mut depth = 0usize;
    let mut start = 0usize;
    for (i, ch) in word.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => start = i + 1,
            _ => {}
        }
    }
    let segment = &word[start..];
    let trimmed = segment.trim_start();
    let lead = segment.len() - trimmed.len();
    match trimmed.split_once('%') {
        Some((weight, rest))
            if !weight.is_empty() && weight.chars().all(|c| c.is_ascii_digit()) =>
        {
            (start + lead + trimmed.len() - rest.len(), rest)
        }
        _ => (start + lead, trimmed),
    }
}

/// Completes a `//command` typed in the chat screen. Returns false for anything
/// else, which the vanilla parser keeps.
pub(super) fn suggest(screen: &mut ChatScreen, m: &Measure, c: &Commands<'_>) -> bool {
    let value = screen.input.value.clone();
    if !value.starts_with("//") {
        return false;
    }
    // `//set` is not in the vanilla tree: its parse is an unknown literal and
    // its error is what the screen used to show. Drop it for this prefix.
    screen.parse = None;
    screen.awaiting = None;
    if !screen.keep_suggestions {
        screen.input.suggestion = None;
        screen.suggestions = None;
    }
    screen.usage.clear();
    let cursor_b = byte_at(&value, screen.input.cursor).clamp(2, value.len());
    let typed = &value[..cursor_b];
    let after = &typed[2..];
    // The word at the cursor, and the words before it.
    let word = last_word_index(after).min(after.len());
    let before: Vec<&str> = after[..word].split_whitespace().collect();
    let current = &after[word..];
    let mut builder = SuggestionsBuilder::new(typed, 2 + word);
    let mut usage = HINT;
    if before.is_empty() {
        builder.suggest_matching(COMMANDS.iter().map(|command| command.name));
        // A complete name that takes arguments completes with the space that
        // starts them, so Tab again lists them: `//schem` → `//schem `.
        if COMMANDS
            .iter()
            .any(|command| command.name.eq_ignore_ascii_case(current) && !command.args.is_empty())
        {
            builder.suggest(format!("{current} "));
        }
    } else {
        let name = before[0].to_ascii_lowercase();
        let spec = COMMANDS.iter().find(|command| command.name == name);
        let index = before.len() - 1;
        // A name the editor does not have: no error, just the hint.
        if let Some(spec) = spec {
            usage = spec.usage;
            if spec.hollow && index == 0 && current.starts_with('-') {
                builder.suggest("-h");
            }
            match spec.args.get(index) {
                Some(&ArgKind::Block) => {
                    let (offset, typed_block) = block_segment(current);
                    let mut rest = builder.offset(2 + word + offset);
                    rest.suggest_matching(block_names(c, typed_block));
                    builder.add(rest);
                }
                Some(&ArgKind::Direction) => builder.suggest_matching(DIRECTIONS),
                Some(&ArgKind::Clear) => builder.suggest_matching(["clear"]),
                Some(&ArgKind::Schem) => builder.suggest_matching(SCHEM_WORDS),
                Some(&ArgKind::SchemFile) => {
                    let names = schematic_names();
                    builder.suggest_matching(names.iter().map(String::as_str));
                }
                Some(&ArgKind::Number) | None => {}
            }
        }
    }
    screen.usage.push(Text::colored(usage, text::GRAY));
    let (range, list) = merge_suggestions(typed, builder.out);
    screen.pending = Some((range.0, list));
    screen.usage_width = screen
        .usage
        .iter()
        .map(|line| m.width(line))
        .max()
        .unwrap_or(0);
    let left = screen.input.screen_x(m, 0);
    screen.usage_position = clamp(
        left,
        0,
        (left + screen.input.inner_width() - screen.usage_width).max(0),
    );
    screen.suggestions = None;
    if screen.allow_suggestions && screen.auto_suggestions {
        screen.show_suggestions(m);
    }
    true
}
