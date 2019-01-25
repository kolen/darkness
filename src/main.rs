extern crate time;
#[macro_use]
extern crate gfx;
extern crate gfx_core;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;

use gfx::Device;
use gfx::traits::FactoryExt;
use gfx_core::format::{DepthStencil, Rgba8};

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        out: gfx::RenderTarget<Rgba8> = "Target0",
    }
}

const TRIANGLE: [Vertex; 3] = [
    Vertex { pos: [ -0.5, -0.5 ], color: [1.0, 0.1, 0.1] },
    Vertex { pos: [  0.5, -0.5 ], color: [0.1, 1.0, 0.1] },
    Vertex { pos: [  0.0,  0.5 ], color: [0.1, 0.1, 1.0] }
];

const CLEAR_COLOR: [f32; 4] = [0.02, 0.02, 0.02, 1.0];

fn main()
{
    let mut events_loop = glutin::EventsLoop::new();

    let window_size = glutin::dpi::LogicalSize::new(1024.0, 768.0);
    let window_builder = glutin::WindowBuilder::new()
        .with_title("Triangle example")
        .with_dimensions(window_size);

    let context = glutin::ContextBuilder::new();

    let (window, mut device, mut factory, rtv, _stv) =
        gfx_window_glutin::init::<Rgba8, DepthStencil>(window_builder, context, &events_loop)
        .expect("Failed!");
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let pso = factory.create_pipeline_simple(
        include_bytes!("../shader/triangle_150.glslv"),
        include_bytes!("../shader/triangle_150.glslf"),
        pipe::new()
    ).unwrap();

    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&TRIANGLE, ());
    let data = pipe::Data {
        vbuf: vertex_buffer,
        out: rtv
    };


    let mut running = true;
    while running
    {
        events_loop.poll_events(|event| {
            match event {
              glutin::Event::WindowEvent{ event, .. } => match event {
                glutin::WindowEvent::KeyboardInput { input, .. } => {
                    match input.virtual_keycode {
                        Some(glutin::VirtualKeyCode::Escape) => running = false,
                        _ => (),
                    }
                },
                _ => (),
              },
              _ => (),
            }
        });

        encoder.clear(&data.out, CLEAR_COLOR);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        println!("{:?}", time::now());
        device.cleanup();
    }
}
