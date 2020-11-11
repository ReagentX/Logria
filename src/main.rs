mod util;
mod ui;
mod strings;

fn main() {
    let options = util::options::from_command_line();
    let cache = options.is_present("cache");
    let smart_speed = options.is_present("smart-speed");
    let exec = options.value_of("exec");
    println!("history disabled? {:?}", cache);
    println!("smart speed disabled? {:?}", smart_speed);
    println!("exec stream? {:?}", exec);

    // Build ui
    let app = cursive::default();
    ui::windows::interface::build(app);
}
