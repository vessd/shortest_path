extern crate bincode;
#[macro_use]
extern crate conrod;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate failure;

mod gui;
mod map;

fn main() {
    let map = map::Map::new(18, 32);
    let mut gui = gui::GUI::new(map);
    gui.exec().unwrap();
}
