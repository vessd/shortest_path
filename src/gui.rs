use conrod;
use conrod::glium::glutin::dpi::LogicalPosition;
use conrod::glium::glutin::ElementState;
use conrod::glium::{self, Surface};
use conrod::position::{Dimensions, Point};
use conrod::widget::envelope_editor::EnvelopePoint;
use conrod::{color, widget, Colorable, Positionable, Widget};
use conrod::{Labelable, Sizeable};
use map::{Cell, Map, MapPos};
use std;

type Result<T> = std::result::Result<T, ::failure::Error>;

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

    fn get_map_pos(&self, map: &Map, dimensions: Dimensions) -> MapPos {
        let x = match self.position.y.round() {
            x if x < 0f64 => 0,
            x if x >= dimensions[1] => map.rows() - 1,
            x => (x / dimensions[1] * map.rows() as f64) as usize,
        };
        let y = match self.position.x.round() {
            y if y < 0f64 => 0,
            y if y >= dimensions[0] => map.cols() - 1,
            y => (y / dimensions[0] * map.cols() as f64) as usize,
        };
        MapPos::new(x, y)
    }
}

pub struct GUI {
    map: Map,
}

impl GUI {
    pub fn new(map: Map) -> Self {
        GUI { map }
    }

    pub fn exec(&mut self) -> Result<()> {
        let height = 600f64;
        let width = 1024f64;

        // Build the window.
        let mut events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_title("Поиск кратчайшего пути")
            .with_dimensions((width, height).into());
        let context = glium::glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4);
        let display = glium::Display::new(window, context, &events_loop).unwrap();

        // construct our `Ui`.
        let mut ui = conrod::UiBuilder::new([width, height]).build();

        // Generate the widget identifiers.
        widget_ids!(struct Ids { canvas, line, matrix_canvas, matrix, button_start });
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
            // Handle all events.
            for event in event_loop.next(&mut events_loop) {
                // Use the `winit` backend feature to convert the winit event to a conrod one.
                if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &display)
                {
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
                                if let Some(matrix_dimensions) = ui.wh_of(ids.matrix) {
                                    let pos = cur.get_map_pos(&self.map, matrix_dimensions);
                                    self.map.set_cell(cur.cell, pos);
                                }
                            }
                        }
                        glium::glutin::WindowEvent::MouseInput {
                            state,
                            button: glium::glutin::MouseButton::Left,
                            ..
                        } => {
                            if state == ElementState::Pressed {
                                if let Some(matrix_dimensions) = ui.wh_of(ids.matrix) {
                                    let pos = cur.get_map_pos(&self.map, matrix_dimensions);
                                    cur.cell = match self.map[pos.x][pos.y] {
                                        Cell::Passable => Cell::Impassable,
                                        Cell::Impassable => Cell::Passable,
                                        Cell::Start => Cell::Start,
                                        Cell::Finish => Cell::Finish,
                                    };
                                    self.map[pos.x][pos.y] = cur.cell;
                                }
                            }
                            cur.state = state;
                        }
                        _ => (),
                    },
                    _ => (),
                }
            }

            // Set the widgets.
            let ui = &mut ui.set_widgets();

            widget::Canvas::new().set(ids.canvas, ui);

            widget::Canvas::new()
                .mid_top_of(ids.canvas)
                .h(ui.h_of(ids.canvas).unwrap_or(600f64) - 24f64)
                .set(ids.matrix_canvas, ui);

            let mut elements = widget::Matrix::new(self.map.cols(), self.map.rows())
                .middle_of(ids.matrix_canvas)
                .set(ids.matrix, ui);

            while let Some(elem) = elements.next(ui) {
                let c = match self.map[elem.row][elem.col] {
                    Cell::Passable => color::WHITE,
                    Cell::Impassable => color::DARK_GREY,
                    Cell::Start => color::GREEN,
                    Cell::Finish => color::RED,
                };
                let canvas = widget::Canvas::new().color(c);
                elem.set(canvas, ui);
            }

            if let Some(matrix_dimensions) = ui.wh_of(ids.matrix_canvas) {
                let cell_w = matrix_dimensions[0] / self.map.cols() as f64;
                let cell_h = matrix_dimensions[1] / self.map.rows() as f64;
                let points: Vec<Point> = self
                    .map
                    .shortest_path()
                    .into_iter()
                    .map(|pos| {
                        Point::new(
                            -matrix_dimensions[0] / 2f64 + cell_w * (pos.y as f64 + 0.5f64),
                            matrix_dimensions[1] / 2f64 - cell_h * (pos.x as f64 + 0.5f64),
                        )
                    }).collect();
                println!("{:#?}", ui.xy_of(ids.matrix_canvas));
                widget::primitive::point_path::PointPath::new(points)
                    .w_h(100f64, 100f64)
                    //.xy(ui.xy_of(ids.matrix).unwrap())
                    //.top_left_of(ids.matrix_canvas)
                    .color(color::YELLOW)
                    .set(ids.line, ui);
            }

            for _press in widget::Button::new()
                .label("PRESS ME")
                .down_from(ids.matrix_canvas, 0.0)
                .align_middle_x_of(ids.canvas)
                .w_h(30f64, 30f64)
                .set(ids.button_start, ui)
            {}

            // Render the `Ui` and then display it on the screen.
            if let Some(primitives) = ui.draw_if_changed() {
                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }
        }
        Ok(())
    }
}
