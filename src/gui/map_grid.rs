use std::cell::RefCell;
use std::rc::Rc;

use cairo::Context;
use gdk::EventMask;
use gtk::WidgetExt;
use gtk::{DrawingArea, Inhibit};
use relm::{Relm, Update, Widget};

#[derive(Debug, Clone)]
struct Color {
    red: f64,
    green: f64,
    blue: f64,
}

#[derive(Debug)]
struct Point {
    x: usize,
    y: usize,
}

#[derive(Debug)]
struct PointPath {
    points: Vec<Point>,
    color: Color,
    width: f64,
}

#[derive(Debug)]
pub struct Model {
    state: Rc<RefCell<State>>,
}

#[derive(Debug)]
pub struct State {
    cell: Vec<Color>,
    width: usize,
    path: Option<PointPath>,
}

impl State {
    fn new(size: (usize, usize)) -> Self {
        State {
            cell: vec![
                Color {
                    red: 1f64,
                    green: 1f64,
                    blue: 1f64
                };
                size.0 * size.1
            ],
            width: size.1,
            path: None,
        }
    }

    fn draw(&self, drawing_area: &DrawingArea, cr: &Context) {
        cr.rectangle(0.0, 0.0, 8.0, 8.0);
        cr.set_source_rgb(0.55, 0.64, 0.68); // dark
        cr.fill();
    }
}

#[derive(Msg)]
pub enum Msg {
    Quit,
}

#[derive(Debug)]
pub struct MapGrid {
    drawing_area: DrawingArea,
    model: Model,
}

impl Update for MapGrid {
    type Model = Model;
    type ModelParam = (usize, usize);
    type Msg = Msg;

    fn model(_: &Relm<Self>, size: (usize, usize)) -> Model {
        Model {
            state: Rc::new(RefCell::new(State::new(size))),
        }
    }

    fn update(&mut self, event: Msg) {}
}

impl Widget for MapGrid {
    type Root = DrawingArea;

    fn root(&self) -> Self::Root {
        self.drawing_area.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let drawing_area = DrawingArea::new();
        drawing_area.add_events(
            (EventMask::BUTTON_PRESS_MASK
                | EventMask::BUTTON_RELEASE_MASK
                | EventMask::POINTER_MOTION_MASK)
                .bits() as i32,
        );
        {
            let weak_state = Rc::downgrade(&model.state);
            drawing_area.connect_draw(move |widget, cr| {
                if let Some(state) = weak_state.upgrade() {
                    let state = state.borrow();
                    state.draw(widget, cr);
                }
                Inhibit(false)
            });
        }

        drawing_area.show();

        MapGrid {
            drawing_area,
            model,
        }
    }
}
