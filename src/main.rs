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
    uv: [f32; 2] = "a_Uv",
});

impl Vertex {
    fn new(p: [f32; 2], u: [f32; 2]) -> Vertex {
        Vertex {
            pos: p,
            uv: u,
        }
    }
}

gfx_pipeline!( pipe {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    dif: gfx::TextureSampler<[f32; 4]> = "t_Dif",
    out: gfx::RenderTarget<Srgba8> = "Target0",
});

const CLEAR_COLOR: [f32; 4] = [0.02, 0.02, 0.02, 1.0];

fn load_texture<R, F>(factory: &mut F, data: &[u8])
                -> Result<gfx::handle::ShaderResourceView<R, [f32; 4]>, String> where
                R: gfx::Resources, F: gfx::Factory<R> {
    use std::io::Cursor;
    use gfx::texture as t;
    let img = image::load(Cursor::new(data), image::PNG).unwrap().to_rgba();
    let (width, height) = img.dimensions();
    let kind = t::Kind::D2(width as t::Size, height as t::Size, t::AaMode::Single);
    let (_, view) = factory.create_texture_immutable_u8::<Rgba8>(kind, t::Mipmap::Provided, &[&img]).unwrap();
    Ok(view)
}

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

    let dif_texture = load_texture(&mut factory, &include_bytes!("../res/concrete.png")[..]).unwrap();
    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, ());
    let sampler = factory.create_sampler_linear();
    let data = pipe::Data {
        vbuf: vertex_buffer,
        dif: (dif_texture, sampler),
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
        // println!("{:?}", time::now());
        device.cleanup();
    }
}
