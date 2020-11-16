mod command_line {
    use cursive::view::SizeConstraint::{AtMost, Full};
    use cursive::views::{ResizedView, TextArea};

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
    use cursive::event::EventTrigger;
    use cursive::theme::{BorderStyle, Color, PaletteColor, Theme};
    use cursive::views::{LinearLayout, TextContent};
    use cursive::Cursive;

    use super::command_line;
    use super::output_window;
    use crate::communication::reader::main::MainWindow;

    /// Default theme that respects the user's terminal color scheme.
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

    fn custom_event_trigger() -> EventTrigger {
        EventTrigger::from_fn_and_tag(|e| true, "logria")
    }

    pub fn build(window: &mut MainWindow) {
        // Text content, used to send content to the output window
        let content = TextContent::new("I am Logria, and\nI\nAm\nALIVE!");

        // Set Theme
        window.logria.set_theme(terminal_theme());
        // Create UI Elements
        let output_window = output_window::build(content, window.logria.screen_size());
        let command_line = command_line::build();
        let layout = LinearLayout::vertical()
            .child(output_window)
            .child(command_line);

        // We can quit by pressing `q`
        window.logria.add_global_callback('q', Cursive::quit);
        window
            .logria
            .set_on_post_event(custom_event_trigger(), |_| {
                // streams.iter().map(|s| {println!("{:?}", s)});
                // Cannot reference logria as an app here, need to find a way to get the stream into this part of the app
                println!("{:?}", true)
            });
        window.logria.add_layer(layout);
        window.logria.set_autorefresh(true);
        // Run the event loop
        window.logria.run();
        println!("{:?}", window.logria.active_screen())
    }
}
