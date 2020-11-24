use crossterm::{cursor, execute, queue, style, terminal, Result};
use std::io::{Stdout, Write};

use crate::communication::reader::main::MainWindow;

fn rect(stdout: &mut Stdout, start: u16, height: u16, width: u16) -> Result<()> {
    for y in start..height {
        for x in 0..width {
            if y == start || y == height - 1 {
                queue!(stdout, cursor::MoveTo(x, y), style::Print("─"))?;
            } else if x == 0 || x == width - 1 {
                queue!(stdout, cursor::MoveTo(x, y), style::Print("│"))?;
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
    execute!(app.output, terminal::Clear(terminal::ClearType::All))?;
    execute!(app.output, cursor::Hide)?;
    terminal::enable_raw_mode()?;
    // panic!("{}, {}", config.height - config.last_row, config.width);
    rect(
        &mut app.output,
        app.config.last_row,
        app.config.height,
        app.config.width,
    )?;
    app.output.flush()?;
    Ok(())
}
