use gdk::EventMask;
use gtk::WidgetExt;
use gtk::{DrawingArea, Inhibit};
use relm::{DrawHandler, Widget};
use relm_attributes::widget;

use self::Msg::UpdateDrawBuffer;

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

pub struct Model {
    draw_handler: DrawHandler<DrawingArea>,
    cell: Vec<Color>,
    width: usize,
    path: Option<PointPath>,
}

#[derive(Msg)]
pub enum Msg {
    UpdateDrawBuffer,
}

#[widget]
impl Widget for MapGrid {
    fn init_view(&mut self) {
        self.model.draw_handler.init(&self.drawing_area);
        self.drawing_area
            .add_events(EventMask::POINTER_MOTION_MASK.bits() as i32);
    }

    fn model(size: (usize, usize)) -> Model {
        Model {
            draw_handler: DrawHandler::new().expect("draw handler"),
            cell: vec![
                Color {
                    red: 1f64,
                    green: 1f64,
                    blue: 1f64
                };
                size.0 * size.1
            ],
            width: size.0,
            path: None,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            UpdateDrawBuffer => {
                let allocation = self.drawing_area.get_allocation();
                let context = self.model.draw_handler.get_context();

                context.rectangle(0.0, 0.0, allocation.width as f64, allocation.height as f64);
                context.set_source_rgb(0.0, 0.0, 0.0);
                context.fill();
                let cell_width = allocation.width as f64 / self.model.width as f64;
                let cell_height =
                    allocation.height as f64 / (self.model.cell.len() / self.model.width) as f64;
                let mut cell_x = 0f64;
                let mut cell_y = 0f64;
                let border = 1f64;
                for (i, c) in self.model.cell.iter().enumerate() {
                    if i % self.model.width == 0 && i != 0 {
                        cell_y += cell_height;
                        cell_x = 0f64;
                    }
                    context.rectangle(
                        cell_x + border,
                        cell_y + border,
                        cell_width - 2f64 * border,
                        cell_height - 2f64 * border,
                    );
                    context.set_source_rgb(c.red, c.green, c.blue);
                    context.fill();
                    cell_x += cell_width;
                }
            }
        }
    }

    view! {
        #[name="drawing_area"]
        gtk::DrawingArea {
            draw(_, _) => (UpdateDrawBuffer, Inhibit(false)),
        }
    }
}
