#[macro_use]
extern crate glium;
extern crate x11;
extern crate clap;
extern crate rand;

use std::io::prelude::*;
use std::process;
use std::fs::File;
use glium::Surface;
use clap::{Arg, App};
use std::{thread, time};
use rand::distributions::{IndependentSample, Range};


#[derive(Copy, Clone)]
struct Vertex {
    position: (f32, f32),
    tex_coords: (f32, f32),
}

implement_vertex!(Vertex, position, tex_coords);


fn main() {
    let matches = App::new("backr")
        .version("0.1")
        .author("Adam Krupicka <krupicka.adam@gmail.com>")
        .about("Renders pretty things. If you have any.")
        .arg(Arg::with_name("INPUT")
             .help("GLSL Fragment shader to use")
             .index(1))
        .arg(Arg::with_name("quality")
             .short("q")
             .long("quality")
             .value_name("FLOAT")
             .help("At least one. Powers of two are best. Default: 2.")
             .takes_value(true))
        .arg(Arg::with_name("fps")
             .short("f")
             .long("fps")
             .value_name("FLOAT")
             .help("How many frames per second to draw. Default: 2.")
             .takes_value(true))
        .get_matches();

    let shader_src_name = matches.value_of("INPUT").unwrap_or("shaders/skyline.frag");
    let quality: f32 = matches.value_of("quality").unwrap_or("2").parse().unwrap_or(2.);
    let fps: f32 = matches.value_of("fps").unwrap_or("2").parse().unwrap_or(2.);

    // Load the shader.
    let mut shader_src_file = File::open(shader_src_name).unwrap();
    let mut shader_src = String::new();
    shader_src_file.read_to_string(&mut shader_src).unwrap();

    // Initialize OpenGL stuff.
    let mut events_loop = glium::glutin::EventsLoop::new();
    let context = glium::glutin::ContextBuilder::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_dimensions(800, 600)
        .with_title("Backr");
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    // Set the X11 flag to be a desktop window.
    // This is by far the hardest part.
    unsafe {
        use x11::xlib;

        let x_display = display.gl_window().platform_display() as *mut xlib::Display;
        let x_window = display.gl_window().platform_window() as u64;
        xlib::XChangeProperty(
            x_display,
            x_window,
            xlib::XInternAtom(x_display, "_NET_WM_WINDOW_TYPE".as_ptr() as *const i8, xlib::False),
            xlib::XA_ATOM,
            32,
            xlib::PropModeReplace,
            xlib::XInternAtom(x_display, "_NET_WM_WINDOW_TYPE_DESKTOP".as_ptr() as *const i8, xlib::False) as *const u8,
            1);
    }

    // Prepare stuff for rendering.
    let vertex1 = Vertex { position: (-1.0, -1.0), tex_coords: (0.0, 0.0) };
    let vertex2 = Vertex { position: (-1.0,  1.0), tex_coords: (0.0, 1.0) };
    let vertex3 = Vertex { position: ( 1.0,  1.0), tex_coords: (1.0, 1.0) };
    let vertex4 = Vertex { position: ( 1.0, -1.0), tex_coords: (1.0, 0.0) };
    let quad = vec![vertex1, vertex2, vertex3, vertex3, vertex4, vertex1];

    let vertex_buffer = glium::VertexBuffer::new(&display, &quad).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let vertex_shader_src = r#"
        #version 140

        in vec2 position;
        in vec2 tex_coords;

        out vec2 v_tex_coords;

        void main() {
            v_tex_coords = tex_coords;
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, shader_src.as_str(), None).unwrap();

    let texture_shader_src = r#"
        #version 140

        in vec2 v_tex_coords;
        uniform sampler2D tex;

        out vec4 color;

        void main() {
            color = texture(tex, v_tex_coords);
        }
    "#;

    // Setup render-to-texture.
    let texture_program = glium::Program::from_source(&display, vertex_shader_src, texture_shader_src, None).unwrap();
    let mut rtt = glium::texture::texture2d::Texture2d::empty(&display, 800, 800).unwrap();

    // Initialize the time uniform to a random value, so that we don't always look at the same thing at startup.
    let mut rng = rand::thread_rng();
    let offset = Range::new(0, 1000).ind_sample(&mut rng);
    let time = time::SystemTime::now() - time::Duration::new(offset, 0);
    let mut res = (800 as f32, 800 as f32);

    // Render. Update. Shine.
    loop {
        let time_start = time::SystemTime::now();

        // Handle events.
        events_loop.poll_events(
            |ev|
            match ev {
                glium::glutin::Event::WindowEvent{window_id: _, event: we} => match we {
                    glium::glutin::WindowEvent::Resized(x, y) => {
                        res = (x as f32, y as f32);
                        let (sx, sy) = ((res.0 / quality) as u32, (res.1 / quality) as u32);
                        rtt = glium::texture::texture2d::Texture2d::empty(&display, sx, sy).unwrap();
                    }
                    glium::glutin::WindowEvent::Closed => process::exit(0),
                    _ => ()
                }
                _ => ()
            }
        );

        let mut target = rtt.as_surface();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        // Uniforms.
        let time_elapsed = time.elapsed().unwrap();
        let ms = time_elapsed.as_secs() as f32 + (time_elapsed.subsec_nanos() as f32 / (10 as f32).powi(9));
        let uniforms = uniform!{
            time: ms,
            resolution: (res.0 / quality, res.1 / quality)
        };

        // Draw fancy shader.
        target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();

        // Draw to screen.
        let mut screen = display.draw();
        screen.draw(&vertex_buffer, &indices, &texture_program, &uniform! {tex: &rtt}, &Default::default()).unwrap();
        screen.finish().unwrap();

        let elapsed = time_start.elapsed().unwrap();
        let to_sleep = (1000. / fps) as i64 - (elapsed.subsec_nanos() / 1000000) as i64;
        if to_sleep > 0 {
            let duration = time::Duration::from_millis(to_sleep as u64);
            thread::sleep(duration);
        }
    }
}
