#![windows_subsystem = "windows"]

extern crate bincode;
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
    gui::Win::run((18, 32)).expect("Win::run failed");
}
