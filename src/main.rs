extern crate time;
extern crate image;
extern crate cgmath;
extern crate edn;

#[macro_use]
extern crate gfx;
extern crate gfx_core;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;

use std::fs::File;
use std::io::prelude::*;
use std::error::Error;
use std::iter::Map;

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
                -> Result<gfx::handle::ShaderResourceView<R, [f32; 4]>, String>
                where R: gfx::Resources, F: gfx::Factory<R>
{
    use std::io::Cursor;
    use gfx::texture as t;
    let img = image::load(Cursor::new(data), image::PNG).unwrap().to_rgba();
    let (width, height) = img.dimensions();
    let kind = t::Kind::D2(width as t::Size, height as t::Size, t::AaMode::Single);
    let (_, view) = factory
        .create_texture_immutable_u8::<Rgba8>(kind, t::Mipmap::Provided, &[&img])
        .unwrap();
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

    let mut model_file = match File::open("assets/cube.model.edn") {
        Err(why) => panic!("couldn't open: {}", why.description()),
        Ok(file) => file,
    };
    let mut model_edn = String::new();
    model_file.read_to_string(&mut model_edn);

    let mut vertices_data: &[f32];
    let mut indices_data: &[f32];

    let mut parser = edn::parser::Parser::new(&model_edn);
    match parser.read() {
      Some(dt) => match dt {
          Ok(edn::Value::Map(model)) => {
              match model[&edn::Value::Keyword("indices".into())] {
                  edn::Value::Vector(indices) =>
                      indices_data = indices
                      .into_itter()
                      .map(|x| x as f32),
                  _ => (),
              }
              match model[&edn::Value::Keyword("vertices".into())] {
                  edn::Value(vertices) => vertices_data = &vertices[..],
                  _ => (),
              }
          },
          _ => (),
      },
      None => {},
    }

    let dif_texture = load_texture(&mut factory, &include_bytes!("../res/wall.png")[..]).unwrap();
    let (vertex_buffer, slice) = factory
        .create_vertex_buffer_with_slice(vertices_data, indices_data);
    let sampler = factory.create_sampler_linear();

    let mut view = Matrix4::look_at(
        Point3::new(1.5f32, -5.0, 3.0),
        Point3::new(0f32, 0.0, 0.0),
        Vector3::unit_z(),
    );
    let proj = cgmath::perspective(cgmath::Deg(50.0f32), 1.33, 1.0, 10.0);

    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        transform: (proj * view).into(),
        dif: (dif_texture, sampler),
        out: rtv,
        out_depth: stv,
    };

    let mut x = 0f32;
    let mut y = 0f32;
    let sensetivity = 0.02f32;
    let distance = 4.0f32;

    let mut pressed_w = false;
    let mut pressed_a = false;
    let mut pressed_s = false;
    let mut pressed_d = false;

    let mut running = true;
    while running
    {
        events_loop.poll_events(|event| {
            match event {
              glutin::Event::WindowEvent{ event, .. } => match event {
                glutin::WindowEvent::KeyboardInput { input, .. } => {
                    match input.virtual_keycode {
                        Some(glutin::VirtualKeyCode::Escape) => running = false,
                        Some(glutin::VirtualKeyCode::W) => {
                            match input.state {
                                glutin::ElementState::Pressed => pressed_w = true,
                                glutin::ElementState::Released => pressed_w = false,
                            }
                        },
                        Some(glutin::VirtualKeyCode::A) => {
                            match input.state {
                                glutin::ElementState::Pressed => pressed_a = true,
                                glutin::ElementState::Released => pressed_a = false,
                            }
                        },
                        Some(glutin::VirtualKeyCode::S) => {
                            match input.state {
                                glutin::ElementState::Pressed => pressed_s = true,
                                glutin::ElementState::Released => pressed_s = false,
                            }
                        },
                        Some(glutin::VirtualKeyCode::D) => {
                            match input.state {
                                glutin::ElementState::Pressed => pressed_d = true,
                                glutin::ElementState::Released => pressed_d = false,
                            }
                        },
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

        if pressed_w { x = x + sensetivity }
        if pressed_a { y = y + sensetivity }
        if pressed_s { x = x - sensetivity }
        if pressed_d { y = y - sensetivity }

        view = Matrix4::look_at(
            Point3::new(x + distance, y + distance, distance),
            Point3::new(x, y, 0.0),
            Vector3::unit_z(),
        );
        data.transform = (proj * view).into();

        encoder.clear(&data.out, CLEAR_COLOR);
        encoder.clear_depth(&data.out_depth, 1.0);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
