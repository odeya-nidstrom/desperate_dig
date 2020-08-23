use glutin::window::{WindowBuilder};
use glutin::{ContextBuilder};
use glutin::event_loop::{EventLoop};
use glium::{
  Display, 
  IndexBuffer,
  Surface, 
  VertexBuffer
};
//use glium::glutin::window::{Fullscreen};
use nalgebra_glm::{Mat4,perspective_fov};

#[derive(Copy, Clone)]
pub struct Vertex {
  position: [f32; 3],
  color: [f32; 3]
}

glium::implement_vertex!(Vertex, position, color);

pub struct Renderer {
  pub display: Display,
  cube_vertex_buffer: VertexBuffer<Vertex>,
  cube_index_buffer: IndexBuffer<u16>,
  //uniforms: glium::uniforms::UniformsStorage<f32, glium::uniforms::EmptyUniforms>,
  world_matrix: [[f32; 4]; 4],
  projection_matrix: [[f32; 4]; 4],
  view_matrix: [[f32; 4]; 4],
  shaders: glium::Program
}

impl Renderer {
  /// Instanciate a new renderer.
  pub fn new(event_loop: &EventLoop<()>) -> Renderer {
    let window_builder = WindowBuilder::new()
      .with_title("Desperate Wolf 🐺");
    let context_builder = ContextBuilder::new();
    let display = Display::new(window_builder, context_builder, event_loop).unwrap();

    println!("Found GL {}", display.get_opengl_version_string());
    
    /*let monitor_handle = display.gl_window().window().available_monitors().next().unwrap();
    let fs = Fullscreen::Borderless(monitor_handle);
    display.gl_window().window().set_fullscreen(Some(fs));
    */

    let matrices = Renderer::init_matrices(&display);
    let program = Renderer::init_shaders(&display);
    
    // init geometry
    let cube_buffers = Renderer::init_vertices(&display);
    let renderer = Renderer {
      display: display,
      cube_vertex_buffer: cube_buffers.0,
      cube_index_buffer: cube_buffers.1,
      projection_matrix: matrices.0.into(),
      view_matrix: matrices.1.into(),
      world_matrix: matrices.2.into(),
      shaders: program
    };
   
    renderer
  }

  fn init_shaders(display: &Display) -> glium::Program {
    let vertex_shader_source = "
    #version 140

    in vec3 position;
    in vec3 color;
    out vec3 dest_color;
 
    uniform mat4 projection_matrix;
    uniform mat4 view_matrix;
    uniform mat4 world_matrix;

    void main() {
      dest_color = color;
      gl_Position = projection_matrix * view_matrix * world_matrix * vec4(position, 1.0);
    }
    ";

    let fragment_shader_source = "
    #version 140

    in vec3 dest_color;
    out vec4 color;

    void main() {
      color = vec4(dest_color, 1.0);
    }
    ";

    let program = glium::Program::from_source(
      display, 
      vertex_shader_source, 
      fragment_shader_source, 
      None
    ).unwrap();

    // return
    program
  }

  /// init projection, view and world matrices
  fn init_matrices(display: &Display) -> (Mat4, Mat4, Mat4) {
    // compute viewport
    let window_size = display.gl_window().window().inner_size();
    let viewport_ratio: f32 = window_size.width as f32 / window_size.height as f32;
    let reference_viewport_ratio: f32 = 4.0 / 3.0;

    let viewport_width: f32;
    let viewport_height: f32;

    if reference_viewport_ratio < viewport_ratio {
      // width larger than expected
      viewport_width = 8.0;
      viewport_height = viewport_width / viewport_ratio;
    } else {
      viewport_height = 6.0;
      viewport_width = viewport_height * viewport_ratio;
    }

    // init matrices
    let fov = 60.0_f32;
    let world_matrix = Mat4::identity();

    let rotate_matrix = nalgebra_glm::rotate_y(
      &Mat4::identity(),
      60.0_f32.to_radians()
    );

    let translate_matrix = nalgebra_glm::translate(
      &Mat4::identity(), 
      &nalgebra_glm::vec3(0.0_f32, 0.0, -5.0));

    let view_matrix = translate_matrix * rotate_matrix;

    let projection_matrix = perspective_fov(
      fov.to_radians(), 
      viewport_width, 
      viewport_height, 
      0.1, 
      1000.0);

    // return
    (projection_matrix, view_matrix, world_matrix)
  }

  /// Init GL
  fn init_vertices(display: &Display) -> (VertexBuffer<Vertex>, IndexBuffer<u16>) {    
    let vertex_data: [Vertex; 8] = [
      Vertex { position: [-1.0,  1.0,  1.0], color: [0.0, 1.0, 1.0] },
      Vertex { position: [-1.0, -1.0,  1.0], color: [1.0, 1.0, 0.0] },
      Vertex { position: [ 1.0, -1.0,  1.0], color: [1.0, 0.0, 1.0] },
      Vertex { position: [ 1.0,  1.0,  1.0], color: [1.0, 1.0, 1.0] },
      Vertex { position: [ 1.0,  1.0, -1.0], color: [0.0, 1.0, 0.0] },
      Vertex { position: [ 1.0, -1.0, -1.0], color: [0.0, 0.0, 1.0] },
      Vertex { position: [-1.0, -1.0, -1.0], color: [1.0, 0.0, 0.0] },
      Vertex { position: [-1.0,  1.0, -1.0], color: [0.0, 0.0, 0.0] } 
    ];

    let vertex_buffer = glium::VertexBuffer::new(display, &vertex_data).unwrap();

    let index_data: [u16; 36] = [
      0, 1, 2, 2, 3, 0,
      3, 2, 5, 5, 4, 3,
      4, 5, 6, 6, 7, 4,
      7, 6, 1, 1, 0, 7,
      2, 1, 6, 6, 5, 2,
      7, 0, 3, 3, 4, 7
    ];

    let index_buffer = glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &index_data).unwrap();
    (vertex_buffer, index_buffer)
  }

  /// Draw next frame
  pub fn draw(&self, time: u128) {
    let mut target = self.display.draw();
    target.clear_color(0.7, 0.8, 0.85, 1.0);
    
    let params = glium::DrawParameters {
      backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
      ..Default::default()
    };

    let rotate_matrix = nalgebra_glm::rotate(
      &Mat4::identity(),
      ((time as f32 * 0.06) % 360.0).to_radians(),
      &nalgebra_glm::vec3(0.4, 1.0, 0.2)
    );

    let translate_matrix = nalgebra_glm::translate(
      &Mat4::identity(), 
      &nalgebra_glm::vec3(0.0_f32, 0.0, -5.0));

    let view_matrix: [[f32;4];4] = (translate_matrix * rotate_matrix).into();

    target.draw(
      &self.cube_vertex_buffer, 
      &self.cube_index_buffer, 
      &self.shaders,
      &glium::uniform! {
        projection_matrix: self.projection_matrix,
        view_matrix: view_matrix,
        world_matrix: self.world_matrix
      },
      &params).unwrap();
    
    target.finish().unwrap();
  }
}