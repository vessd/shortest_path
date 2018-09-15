extern crate bincode;
#[macro_use]
extern crate conrod;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod map;

use conrod::backend::glium::glium::{self, Surface};
use conrod::position::Point;
use conrod::widget::envelope_editor::EnvelopePoint;
use conrod::{color, widget, Colorable, Positionable, Widget};
use glium::glutin::dpi::LogicalPosition;
use glium::glutin::ElementState;
use map::{Cell, Map, MapPos};

const COLS: usize = 32;
const ROWS: usize = 18;
const CELL_SIZE: usize = 32;

const WIDTH: usize = COLS * CELL_SIZE;
const HEIGHT: usize = ROWS * CELL_SIZE;

/// In most of the examples the `glutin` crate is used for providing the window context and
/// events while the `glium` crate is used for displaying `conrod::render::Primitives` to the
/// screen.
///
/// This `Iterator`-like type simplifies some of the boilerplate involved in setting up a
/// glutin+glium event loop that works efficiently with conrod.
pub struct EventLoop {
    ui_needs_update: bool,
    last_update: std::time::Instant,
}

impl EventLoop {
    pub fn new() -> Self {
        EventLoop {
            last_update: std::time::Instant::now(),
            ui_needs_update: true,
        }
    }

    /// Produce an iterator yielding all available events.
    pub fn next(
        &mut self,
        events_loop: &mut glium::glutin::EventsLoop,
    ) -> Vec<glium::glutin::Event> {
        // We don't want to loop any faster than 60 FPS, so wait until it has been at least 16ms
        // since the last yield.
        let last_update = self.last_update;
        let sixteen_ms = std::time::Duration::from_millis(16);
        let duration_since_last_update = std::time::Instant::now().duration_since(last_update);
        if duration_since_last_update < sixteen_ms {
            std::thread::sleep(sixteen_ms - duration_since_last_update);
        }

        // Collect all pending events.
        let mut events = Vec::new();
        events_loop.poll_events(|event| events.push(event));

        // If there are no events and the `Ui` does not need updating, wait for the next event.
        if events.is_empty() && !self.ui_needs_update {
            events_loop.run_forever(|event| {
                events.push(event);
                glium::glutin::ControlFlow::Break
            });
        }

        self.ui_needs_update = false;
        self.last_update = std::time::Instant::now();

        events
    }

    /// Notifies the event loop that the `Ui` requires another update whether or not there are any
    /// pending events.
    ///
    /// This is primarily used on the occasion that some part of the `Ui` is still animating and
    /// requires further updates to do so.
    pub fn needs_update(&mut self) {
        self.ui_needs_update = true;
    }
}

struct Cursor {
    position: LogicalPosition,
    state: ElementState,
    cell: Cell,
}

impl Cursor {
    fn new() -> Self {
        Cursor {
            position: (0f64, 0f64).into(),
            state: ElementState::Released,
            cell: Cell::Passable,
        }
    }
}

impl From<LogicalPosition> for MapPos {
    fn from(pos: LogicalPosition) -> Self {
        let (mut x, mut y): (i32, i32) = pos.into();
        if x < 0 {
            x = 0;
        }
        if y < 0 {
            y = 0;
        }
        MapPos::new(y as usize / CELL_SIZE, x as usize / CELL_SIZE)
    }
}

fn main() {
    let mut m = Map::new(COLS, ROWS);

    // Build the window.
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_title("Поиск кратчайшего пути")
        .with_dimensions((WIDTH as f64, HEIGHT as f64).into());
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    // construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // Generate the widget identifiers.
    widget_ids!(struct Ids { line, matrix });
    let ids = Ids::new(ui.widget_id_generator());

    // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

    // Poll events from the window.
    let mut event_loop = EventLoop::new();

    let mut cur = Cursor::new();
    'main: loop {
        //std::thread::sleep(std::time::Duration::from_millis(25));
        // Handle all events.
        for event in event_loop.next(&mut events_loop) {
            // Use the `winit` backend feature to convert the winit event to a conrod one.
            if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &display) {
                ui.handle_event(event);
                event_loop.needs_update();
            }

            match event {
                glium::glutin::Event::WindowEvent { event, .. } => match event {
                    // Break from the loop upon `Escape`.
                    glium::glutin::WindowEvent::CloseRequested
                    | glium::glutin::WindowEvent::KeyboardInput {
                        input:
                            glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => break 'main,
                    glium::glutin::WindowEvent::CursorMoved { position, .. } => {
                        cur.position = position;
                        if cur.state == ElementState::Pressed {
                            m.set_cell(cur.cell, cur.position.into())
                        }
                    }
                    glium::glutin::WindowEvent::MouseInput {
                        state,
                        button: glium::glutin::MouseButton::Left,
                        ..
                    } => {
                        if state == ElementState::Pressed {
                            let pos = MapPos::from(cur.position);
                            cur.cell = match m[pos.x][pos.y] {
                                Cell::Passable => Cell::Impassable,
                                Cell::Impassable => Cell::Passable,
                                Cell::Start => Cell::Start,
                                Cell::Finish => Cell::Finish,
                            };
                            m[pos.x][pos.y] = cur.cell;
                        }
                        cur.state = state;
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        let points: Vec<Point> = m
            .shortest_path()
            .into_iter()
            .map(|pos| {
                Point::new(
                    (pos.y as f64 - COLS as f64 / 2f64 + 0.5f64) * 32f64,
                    (ROWS as f64 / 2f64 - pos.x as f64 - 0.5f64) * 32f64,
                )
            }).collect();

        // Set the widgets.
        let ui = &mut ui.set_widgets();

        let mut elements = widget::Matrix::new(COLS, ROWS)
            .middle_of(ui.window)
            .set(ids.matrix, ui);

        while let Some(elem) = elements.next(ui) {
            let c = match m[elem.row][elem.col] {
                Cell::Passable => color::WHITE,
                Cell::Impassable => color::DARK_GREY,
                Cell::Start => color::GREEN,
                Cell::Finish => color::RED,
            };
            let canvas = widget::Canvas::new().color(c);
            elem.set(canvas, ui);
        }

        widget::primitive::point_path::PointPath::abs(points)
            .color(color::YELLOW)
            .xy_relative_to(ui.window, Point::new(0f64, 0f64))
            .set(ids.line, ui);

        // Render the `Ui` and then display it on the screen.
        if let Some(primitives) = ui.draw_if_changed() {
            renderer.fill(&display, primitives, &image_map);
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&display, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }
    }
}
