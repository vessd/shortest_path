extern crate bincode;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod map;

use map::{Map, MapPos};

fn main() {
    let mut m = Map::new(5, 5);
    m.set_wall(MapPos::new(1, 2));
    m.set_wall(MapPos::new(1, 3));
    //m.set_wall(MapPos::new(2, 2));
    m.set_wall(MapPos::new(3, 1));
    //m.set_wall(MapPos::new(3, 2));
    m.set_wall(MapPos::new(3, 3));
    m.set_wall(MapPos::new(4, 3));
    m.print_map();
    let vec = m.shortest_path(MapPos::new(0, 0), MapPos::new(4, 4));
    println!("");
    m.print_path(vec);
}
