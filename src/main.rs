extern crate time;
extern crate image;
extern crate cgmath;

#[macro_use]
extern crate gfx;
extern crate gfx_core;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;

use gfx::Device;
use gfx::traits::FactoryExt;
use gfx_core::format::{DepthStencil, Rgba8, Srgba8};

use cgmath::{Point3, Vector3, Matrix4};

gfx_vertex_struct!( Vertex {
    pos: [f32; 4] = "a_Pos",
    uv: [f32; 2] = "a_Uv",
});

impl Vertex {
    fn new(p: [f32; 3], u: [f32; 2]) -> Vertex {
        Vertex {
            pos: [p[0], p[1], p[2], 1.0],
            uv: u,
        }
    }
}

gfx_constant_struct!( Locals {
    transform: [[f32; 4]; 4] = "u_Transform",
});

gfx_pipeline!( pipe {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    transform: gfx::Global<[[f32; 4]; 4]> = "u_Transform",
    dif: gfx::TextureSampler<[f32; 4]> = "t_Dif",
    out: gfx::RenderTarget<Srgba8> = "Target0",
    out_depth: gfx::DepthTarget<DepthStencil> =
        gfx::preset::depth::LESS_EQUAL_WRITE,
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
    let window_size = glutin::dpi::LogicalSize::new(1024.0, 768.0);
    let window_builder = glutin::WindowBuilder::new()
        .with_title("Darkness")
        .with_dimensions(window_size);
    let context = glutin::ContextBuilder::new();
    let (window, mut device, mut factory, rtv, stv) =
        gfx_window_glutin::init::<Srgba8, DepthStencil>(window_builder, context, &events_loop)
        .expect("Failed!");
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let pso = factory.create_pipeline_simple(
        include_bytes!("../shader/vert.glsl"),
        include_bytes!("../shader/frag.glsl"),
        pipe::new()
    ).unwrap();

    let vertex_data = [
        // top (0, 0, 1.0)
        Vertex::new([-1.0, -1.0,  1.0], [0.0, 0.0]),
        Vertex::new([ 1.0, -1.0,  1.0], [1.0, 0.0]),
        Vertex::new([ 1.0,  1.0,  1.0], [1.0, 1.0]),
        Vertex::new([-1.0,  1.0,  1.0], [0.0, 1.0]),
        // bottom (0.0, 0.0, -1.0)
        Vertex::new([-1.0,  1.0, -1.0], [1.0, 0.0]),
        Vertex::new([ 1.0,  1.0, -1.0], [0.0, 0.0]),
        Vertex::new([ 1.0, -1.0, -1.0], [0.0, 1.0]),
        Vertex::new([-1.0, -1.0, -1.0], [1.0, 1.0]),
        // right (1.0, 0.0, 0.0)
        Vertex::new([ 1.0, -1.0, -1.0], [0.0, 0.0]),
        Vertex::new([ 1.0,  1.0, -1.0], [1.0, 0.0]),
        Vertex::new([ 1.0,  1.0,  1.0], [1.0, 1.0]),
        Vertex::new([ 1.0, -1.0,  1.0], [0.0, 1.0]),
        // left (-1.0, 0.0, 0.0)
        Vertex::new([-1.0, -1.0,  1.0], [1.0, 0.0]),
        Vertex::new([-1.0,  1.0,  1.0], [0.0, 0.0]),
        Vertex::new([-1.0,  1.0, -1.0], [0.0, 1.0]),
        Vertex::new([-1.0, -1.0, -1.0], [1.0, 1.0]),
        // front (0.0, 1.0, 0.0)
        Vertex::new([ 1.0,  1.0, -1.0], [1.0, 0.0]),
        Vertex::new([-1.0,  1.0, -1.0], [0.0, 0.0]),
        Vertex::new([-1.0,  1.0,  1.0], [0.0, 1.0]),
        Vertex::new([ 1.0,  1.0,  1.0], [1.0, 1.0]),
        // back (0.0, -1.0, 0.0)
        Vertex::new([ 1.0, -1.0,  1.0], [0.0, 0.0]),
        Vertex::new([-1.0, -1.0,  1.0], [1.0, 0.0]),
        Vertex::new([-1.0, -1.0, -1.0], [1.0, 1.0]),
        Vertex::new([ 1.0, -1.0, -1.0], [0.0, 1.0]),
    ];

    let index_data: &[u16] = &[
         0,  1,  2,  2,  3,  0, // top
         4,  5,  6,  6,  7,  4, // bottom
         8,  9, 10, 10, 11,  8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    let dif_texture = load_texture(&mut factory, &include_bytes!("../res/wall.png")[..]).unwrap();
    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, index_data);
    let sampler = factory.create_sampler_linear();

    let view = Matrix4::look_at(
        Point3::new(1.5f32, -5.0, 3.0),
        Point3::new(0f32, 0.0, 0.0),
        Vector3::unit_z(),
    );
    let proj = cgmath::perspective(cgmath::Deg(50.0f32), 1.33, 1.0, 10.0);

    let data = pipe::Data {
        vbuf: vertex_buffer,
        transform: (proj * view).into(),
        dif: (dif_texture, sampler),
        out: rtv,
        out_depth: stv,
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
        encoder.clear_depth(&data.out_depth, 1.0);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        // println!("{:?}", time::now());
        device.cleanup();
    }
}
