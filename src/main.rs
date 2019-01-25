extern crate time;
extern crate image;
#[macro_use]
extern crate gfx;
extern crate gfx_core;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;

use gfx::Device;
use gfx::traits::FactoryExt;
use gfx_core::format::{DepthStencil, Rgba8, Srgba8};

gfx_vertex_struct!( Vertex {
    pos: [f32; 2] = "a_Pos",
    color: [f32; 2] = "a_Color",
});

impl Vertex {
    fn new(p: [f32; 2], c: [f32; 2]) -> Vertex {
        Vertex {
            pos: p,
            color: c,
        }
    }
}

gfx_pipeline!( pipe {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    out: gfx::RenderTarget<Srgba8> = "Target0",
});

const CLEAR_COLOR: [f32; 4] = [0.02, 0.02, 0.02, 1.0];

fn main()
{
    let mut events_loop = glutin::EventsLoop::new();

    let window_size = glutin::dpi::LogicalSize::new(768.0, 768.0);

    let window_builder = glutin::WindowBuilder::new()
        .with_title("Darkness")
        .with_dimensions(window_size);

    let context = glutin::ContextBuilder::new();

    let (window, mut device, mut factory, rtv, _stv) =
        gfx_window_glutin::init::<Srgba8, DepthStencil>(window_builder, context, &events_loop)
        .expect("Failed!");
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let pso = factory.create_pipeline_simple(
        include_bytes!("../shader/simple_150.glslv"),
        include_bytes!("../shader/simple_150.glslf"),
        pipe::new()
    ).unwrap();

    let vertex_data = [
        Vertex::new([-0.4, -0.4], [0.0, 0.4]),
        Vertex::new([ 0.4, -0.4], [0.4, 0.4]),
        Vertex::new([ 0.4,  0.4], [0.4, 0.0]),

        Vertex::new([-0.4, -0.4], [0.0, 0.4]),
        Vertex::new([ 0.4,  0.4], [0.4, 0.0]),
        Vertex::new([-0.4,  0.4], [0.0, 0.0]),
    ];

    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, ());

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
                glutin::WindowEvent::Resized(logical_size) => {
                    let dpi_factor = window.get_hidpi_factor();
                    window.resize(logical_size.to_physical(dpi_factor));
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
