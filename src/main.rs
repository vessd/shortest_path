extern crate bincode;
extern crate cairo;
extern crate failure;
extern crate gdk;
extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use relm::Widget;

mod gui;
mod map;

fn main() {
    let map = map::Map::new(18, 32);
    gui::Win::run(()).expect("Win::run failed");
    //let mut gui = gui::GUI::new(map);
    //gui.exec().unwrap();
}
