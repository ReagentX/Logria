use std::fs::File;

struct Tape {
    history_tape: Vec<String>,
    current_index: usize,
    should_scroll_back: bool,
    history_tape_file: File,
    use_cache: bool
}

impl Tape {

}
