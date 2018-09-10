extern crate bincode;
#[macro_use]
extern crate conrod;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate winit;

mod map;

use conrod::color::Color;
use conrod::gfx::Device;
use conrod::widget;
use conrod::Labelable;
use conrod::Positionable;
use conrod::Sizeable;
use conrod::Widget;
use glutin::GlContext;
use map::{Map, MapPos};

const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

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

    let builder = glutin::WindowBuilder::new()
        .with_title("Поиск кратчайшего пути")
        .with_resizable(false)
        .with_dimensions((800, 600).into());
    let context = glutin::ContextBuilder::new().with_multisampling(4);

    let mut events_loop = winit::EventsLoop::new();

    // Initialize gfx things
    let (window, mut device, mut factory, rtv, _) = gfx_window_glutin::init::<
        conrod::backend::gfx::ColorFormat,
        conrod::gfx_core::format::DepthStencil,
    >(builder, context, &events_loop);

    let mut encoder: conrod::gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let mut renderer =
        conrod::backend::gfx::Renderer::new(&mut factory, &rtv, window.get_hidpi_factor() as f64)
            .unwrap();

    let mut ui = conrod::UiBuilder::new([800 as f64, 600 as f64]).build();

    // Instantiate the generated list of widget identifiers.
    widget_ids!(struct Ids { canvas, counter });
    let ids = Ids::new(ui.widget_id_generator());

    let mut image_map = conrod::image::Map::new();

    renderer.clear(&mut encoder, CLEAR_COLOR);
    encoder.flush(&mut device);
    window.swap_buffers().unwrap();
    device.cleanup();

    let mut count = 0;

    'main: loop {
        let mut should_quit = false;
        let (win_w, win_h): (u32, u32) = match window.get_inner_size() {
            Some(s) => s.into(),
            None => break 'main,
        };
        let dpi_factor = window.get_hidpi_factor() as f32;
        events_loop.poll_events(|event| {
            // Convert winit event to conrod event, requires conrod to be built with the `winit` feature
            if let Some(event) =
                conrod::backend::winit::convert_event(event.clone(), window.window())
            {
                ui.handle_event(event);
            }
            // Close window if the escape key or the exit button is pressed
            match event {
                winit::Event::WindowEvent { event, .. } => match event {
                    winit::WindowEvent::KeyboardInput {
                        input:
                            winit::KeyboardInput {
                                virtual_keycode: Some(winit::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    }
                    | winit::WindowEvent::CloseRequested => should_quit = true,
                    _ => {}
                },
                _ => {}
            }

            // Instantiate all widgets in the GUI.
            {
                let ui = &mut ui.set_widgets();

                // Create a background canvas upon which we'll place the button.
                widget::Canvas::new().pad(40.0).set(ids.canvas, ui);

                // Draw the button and increment `count` if pressed.
                for _click in widget::Button::new()
                    .middle_of(ids.canvas)
                    .w_h(80.0, 80.0)
                    .label(&count.to_string())
                    .label_color(Color::Rgba(255f32, 0f32, 0f32, 0f32))
                    .set(ids.counter, ui)
                {
                    count += 1;
                }
            }

            // Render the `Ui` and then display it on the screen.
            /* if let Some(primitives) = ui.draw_if_changed() {
                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            } */
            if let Some(primitives) = ui.draw_if_changed() {
                let dims = (win_w as f32 * dpi_factor, win_h as f32 * dpi_factor);

                //Clear the window
                renderer.clear(&mut encoder, CLEAR_COLOR);

                renderer.fill(
                    &mut encoder,
                    dims,
                    dpi_factor as f64,
                    primitives,
                    &image_map,
                );

                renderer.draw(&mut factory, &mut encoder, &image_map);

                encoder.flush(&mut device);
                window.swap_buffers().unwrap();
                device.cleanup();
            }
        });
        if should_quit {
            break 'main;
        }
    }
    println!("{}", count);
}
