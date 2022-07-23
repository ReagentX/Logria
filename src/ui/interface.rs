use crossterm::{cursor, execute, queue, style, terminal, tty::IsTty, Result};
use std::io::{stdin, stdout, Stdout, Write};

use crate::communication::reader::MainWindow;

fn rect(stdout: &mut Stdout, start: u16, height: u16, width: u16) -> Result<()> {
    for y in start..height {
        for x in 0..width {
            if y == start || y == height - 1 {
                queue!(stdout, cursor::MoveTo(x, y), style::Print("─"))?; // left side
            } else if x == 0 || x == width - 1 {
                queue!(stdout, cursor::MoveTo(x, y), style::Print("│"))?; // right side
            }
            queue!(stdout, cursor::MoveTo(width - 1, start), style::Print("┐"))?; // top right
            queue!(stdout, cursor::MoveTo(0, start), style::Print("┌"))?; // top left
            queue!(stdout, cursor::MoveTo(width - 1, height), style::Print("┘"))?; // bottom right
            queue!(stdout, cursor::MoveTo(0, height), style::Print("└"))?; // bottom left
        }
    }
    Ok(())
}

pub fn build(app: &mut MainWindow) -> Result<()> {
    let mut stdout = stdout();
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    execute!(stdout, cursor::Hide)?;
    terminal::enable_raw_mode()?;
    rect(
        &mut stdout,
        app.config.last_row,
        app.config.height,
        app.config.width,
    )?;
    stdout.flush()?;
    Ok(())
}

/// Ensure both stdin and stdout are controlled by the terminal emulator
pub fn valid_tty() -> bool {
    stdin().is_tty() && stdout().is_tty()
}
