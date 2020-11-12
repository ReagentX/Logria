mod command_line {
    use cursive::view::SizeConstraint::{AtMost, Full};
    use cursive::views::{TextArea, ResizedView};

    pub fn build() -> ResizedView<TextArea> {
        let view = ResizedView::new(Full, AtMost(1), TextArea::new());
        view
    }
}

mod output_window {
    use cursive::align::VAlign::Bottom;
    use cursive::views::TextContent;
    use cursive::views::{ResizedView, TextView};

    pub fn build(content: TextContent, size: cursive::Vec2) -> ResizedView<TextView> {
        let view = ResizedView::with_fixed_size(
            (size.x, size.y - 3),
            TextView::new_with_content(content).v_align(Bottom),
        );
        view
    }
}

pub mod interface {
    use cursive::theme::{Color, PaletteColor, Theme, BorderStyle};
    use cursive::views::{LinearLayout, TextContent};
    use cursive::Cursive;

    use super::command_line;
    use super::output_window;

    fn terminal_theme() -> Theme {
        let mut theme = Theme::default();
        theme.borders = BorderStyle::None;
        theme.shadow = false;
        theme.palette[PaletteColor::Highlight] = Color::TerminalDefault;
        theme.palette[PaletteColor::HighlightInactive] = Color::TerminalDefault;
        theme.palette[PaletteColor::HighlightText] = Color::TerminalDefault;
        theme.palette[PaletteColor::Background] = Color::TerminalDefault;
        theme.palette[PaletteColor::Primary] = Color::TerminalDefault;
        theme.palette[PaletteColor::Secondary] = Color::TerminalDefault;
        theme.palette[PaletteColor::Shadow] = Color::TerminalDefault;
        theme.palette[PaletteColor::Tertiary] = Color::TerminalDefault;
        theme.palette[PaletteColor::TitlePrimary] = Color::TerminalDefault;
        theme.palette[PaletteColor::TitleSecondary] = Color::TerminalDefault;
        theme.palette[PaletteColor::View] = Color::TerminalDefault;
        return theme;
    }

    pub fn build(mut app: Cursive) {
        // Text content, used to send content to the output window
        let content = TextContent::new("I am Logria, and\nI\nAm\nALIVE!");

        // Set Theme
        app.set_theme(terminal_theme());

        // UI Elements
        let output_window = output_window::build(content, app.screen_size());
        let command_line = command_line::build();
        // We can quit by pressing `q`
        let layout = LinearLayout::vertical()
            .child(output_window)
            .child(command_line);
        app.add_global_callback('q', Cursive::quit);
        app.add_layer(layout);
        // Run the event loop
        app.run();
    }
}
