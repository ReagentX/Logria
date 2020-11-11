mod command_line {
    use cursive::views::{EditView, ResizedView};

    pub fn build() -> ResizedView<EditView> {
        ResizedView::with_fixed_size((50, 20), EditView::new())
    }
}

mod output_window {
    use cursive::views::{ResizedView, TextView};

    pub fn build() -> ResizedView<TextView> {
        ResizedView::with_fixed_size((20, 4), TextView::new("i am the output window!"))
    }
}

pub mod interface {
    use cursive::views::TextView;
    use cursive::Cursive;

    use super::command_line;
    use super::output_window;

    pub fn build(mut app: Cursive) {
        // We can quit by pressing `q`
        app.add_global_callback('q', Cursive::quit);
        // Add a simple view
        app.add_layer(command_line::build());
        app.add_layer(output_window::build());
        // Run the event loop
        app.run();
    }
}
