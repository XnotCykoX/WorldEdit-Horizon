fn guest_command_head(
    game: &mut crate::game::Game<'_>,
    host: &mut crate::game::lan::LanHost,
    conn: u32,
    command: &str,
) -> core::ops::ControlFlow<()> {
    let _ = game;
    if !crate::game::worldedit::is_command(command) {
        return core::ops::ControlFlow::Continue(());
    }
    let Some(permission) = host.hub.guest(conn).map(|g| g.permission) else {
        return core::ops::ControlFlow::Continue(());
    };
    let integrated = host.hub.config.integrated;
    // Queued as typed, so `run_command` sees `/set ...` and the editor runs
    // with this guest's state swapped in (`as_guest`).
    host.commands.push(crate::game::lan_actions::GuestCommand {
        conn,
        command: command.to_owned(),
        permission,
        integrated,
        quiet: false,
    });
    core::ops::ControlFlow::Break(())
}
