use gdk::EventMask;
use gtk::WidgetExt;
use gtk::{DrawingArea, Inhibit};
use map::{Cell, Map, MapPos};
use relm::{DrawHandler, Widget};
use relm_attributes::widget;

use self::Msg::{ButtonPress, ButtonRelease, MoveCursor, UpdateDrawBuffer};

#[derive(Debug, Clone)]
struct Color {
    red: f64,
    green: f64,
    blue: f64,
}

impl Color {
    fn white() -> Self {
        Self {
            red: 1f64,
            green: 1f64,
            blue: 1f64,
        }
    }

    fn red() -> Self {
        Self {
            red: 1f64,
            green: 0f64,
            blue: 0f64,
        }
    }

    fn green() -> Self {
        Self {
            red: 0f64,
            green: 1f64,
            blue: 0f64,
        }
    }

    fn grey() -> Self {
        Self {
            red: 0.5f64,
            green: 0.5f64,
            blue: 0.5f64,
        }
    }

    fn yellow() -> Self {
        Self {
            red: 1f64,
            green: 1f64,
            blue: 0f64,
        }
    }
}

#[derive(Debug)]
struct PointPath {
    coordinates: Vec<MapPos>,
    color: Color,
}

struct Cursor {
    position: (f64, f64),
    button_pressed: bool,
    cell: Cell,
}

pub struct Model {
    draw_handler: DrawHandler<DrawingArea>,
    map: Map,
    path: Option<PointPath>,
    cursor: Cursor,
}

#[derive(Msg)]
pub enum Msg {
    UpdateDrawBuffer,
    MoveCursor((f64, f64)),
    ButtonPress,
    ButtonRelease,
}

impl MapGrid {
    fn get_cursor_pos(&self) -> MapPos {
        let allocation = self.drawing_area.get_allocation();
        let x = match self.model.cursor.position.1.round() {
            x if x < 0f64 => 0,
            x if x >= f64::from(allocation.height) => self.model.map.rows() - 1,
            x => (x / f64::from(allocation.height) * self.model.map.rows() as f64) as usize,
        };
        let y = match self.model.cursor.position.0.round() {
            y if y < 0f64 => 0,
            y if y >= f64::from(allocation.width) => self.model.map.cols() - 1,
            y => (y / f64::from(allocation.width) * self.model.map.cols() as f64) as usize,
        };
        MapPos::new(x, y)
    }
}

#[widget]
impl Widget for MapGrid {
    fn init_view(&mut self) {
        self.model.draw_handler.init(&self.drawing_area);
        self.drawing_area.add_events(
            (EventMask::BUTTON_PRESS_MASK
                | EventMask::BUTTON_RELEASE_MASK
                | EventMask::POINTER_MOTION_MASK)
                .bits() as i32,
        );
    }

    fn model(size: (usize, usize)) -> Model {
        Model {
            draw_handler: DrawHandler::new().expect("draw handler"),
            map: Map::new(size.0, size.1),
            path: None,
            cursor: Cursor {
                position: (0f64, 0f64),
                button_pressed: false,
                cell: Cell::Passable,
            },
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            UpdateDrawBuffer => {
                let allocation = self.drawing_area.get_allocation();
                let context = self.model.draw_handler.get_context();
                context.rectangle(
                    0f64,
                    0f64,
                    f64::from(allocation.width),
                    f64::from(allocation.height),
                );
                context.set_source_rgb(0.0, 0.0, 0.0);
                context.fill();
                let cell_width = f64::from(allocation.width) / self.model.map.cols() as f64;
                let cell_height = f64::from(allocation.height) / self.model.map.rows() as f64;

                // draw grid
                let border = 1f64;
                for i in 0..self.model.map.rows() {
                    for j in 0..self.model.map.cols() {
                        let color = match self.model.map[i][j] {
                            Cell::Passable => Color::white(),
                            Cell::Impassable => Color::grey(),
                            Cell::Start => Color::green(),
                            Cell::Finish => Color::red(),
                        };
                        context.set_source_rgb(color.red, color.green, color.blue);
                        context.rectangle(
                            j as f64 * cell_width + border,
                            i as f64 * cell_height + border,
                            cell_width - 2f64 * border,
                            cell_height - 2f64 * border,
                        );
                        context.fill();
                    }
                }

                // draw path
                if let Some(ref path) = self.model.path {
                    if path.coordinates.len() > 1 {
                        context.set_line_width(3f64);
                        context.set_source_rgb(path.color.red, path.color.green, path.color.blue);
                        context.move_to(
                            (path.coordinates[0].x as f64 + 0.5f64) * cell_width,
                            (path.coordinates[0].y as f64 + 0.5f64) * cell_height,
                        );
                        for c in path.coordinates.iter().skip(1) {
                            context.line_to(
                                (c.x as f64 + 0.5f64) * cell_width,
                                (c.y as f64 + 0.5f64) * cell_height,
                            );
                        }
                        context.stroke();
                    }
                }
            }
            MoveCursor(pos) => {
                self.model.cursor.position = pos;
                if self.model.cursor.button_pressed {
                    let pos = self.get_cursor_pos();
                    self.model.map.set_cell(self.model.cursor.cell, pos);
                }
            }
            ButtonPress => {
                self.model.cursor.button_pressed = true;
                let pos = self.get_cursor_pos();
                self.model.cursor.cell = match self.model.map[pos.x][pos.y] {
                    Cell::Passable => Cell::Impassable,
                    Cell::Impassable => Cell::Passable,
                    Cell::Start => Cell::Start,
                    Cell::Finish => Cell::Finish,
                };
                self.model.map[pos.x][pos.y] = self.model.cursor.cell;
            }
            ButtonRelease => self.model.cursor.button_pressed = false,
        }
    }

    view! {
        #[name="drawing_area"]
        gtk::DrawingArea {
            draw(_, _) => (UpdateDrawBuffer, Inhibit(false)),
            motion_notify_event(_, event) => (MoveCursor(event.get_position()), Inhibit(false)),
            button_press_event(_, _) => (ButtonPress, Inhibit(false)),
            button_release_event(_, _) => (ButtonRelease, Inhibit(false)),
        }
    }
}
