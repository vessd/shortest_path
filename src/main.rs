extern crate bincode;
extern crate conrod;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate winit;

mod map;

use conrod::gfx::Device;
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

    renderer.clear(&mut encoder, CLEAR_COLOR);
    encoder.flush(&mut device);
    window.swap_buffers().unwrap();
    device.cleanup();

    'main: loop {
        let mut should_quit = false;
        events_loop.poll_events(|event| {
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
        });
        if should_quit {
            break 'main;
        }
    }
}
