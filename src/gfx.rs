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
  //view_matrix: [[f32; 4]; 4],
  shaders: glium::Program,
  frame_rendered_count: u32,
  fps_start_at: u128
}

impl Renderer {
  /// Instanciate a new renderer.
  pub fn new(event_loop: &EventLoop<()>) -> Renderer {
    let window_builder = WindowBuilder::new()
      .with_title("Desperate Wolf 🐺");
    let context_builder = ContextBuilder::new()
      .with_vsync(true)
      .with_double_buffer(Some(true))
      .with_hardware_acceleration(Some(true));
      //.with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3,3)));
    let display = Display::new(window_builder, context_builder, event_loop).unwrap();

    println!("Found GL {}", display.get_opengl_version_string());
    let matrices = Renderer::init_matrices(&display);
    let program = Renderer::init_shaders(&display);
    
    // init geometry
    let cube_buffers = Renderer::init_vertices(&display);
    let renderer = Renderer {
      display: display,
      cube_vertex_buffer: cube_buffers.0,
      cube_index_buffer: cube_buffers.1,
      projection_matrix: matrices.0.into(),
      //view_matrix: matrices.1.into(),
      world_matrix: matrices.2.into(),
      shaders: program,
      frame_rendered_count: 0,
      fps_start_at: 0
    };
   
    renderer
  }

  pub fn refresh_viewport_size(&mut self) {
    let matrices = Renderer::init_matrices(&self.display);
    self.projection_matrix = matrices.0.into();
  }

  fn init_shaders(display: &Display) -> glium::Program {
    let vertex_shader_source = "
    #version 140

    in vec3 position;
    in vec3 color;
    out vec3 dest_color;
    out vec3 original_position;
 
    uniform mat4 projection_matrix;
    uniform mat4 view_matrix;
    uniform mat4 world_matrix;

    void main() {
      dest_color = vec3(1.0, 1.0, 1.0);
      original_position = position;
      gl_Position = projection_matrix * view_matrix * world_matrix * vec4(position, 1.0);
    }
    ";

    let fragment_shader_source = "
    #version 140

    in vec3 dest_color;
    in vec3 original_position;
    out vec4 color;

    void main() {
      float fract_x;
      float fract_y;
      float fract_z;
      int is_x_near_edge;
      int is_y_near_edge;
      int is_z_near_edge;
      float edge_threshold;
      float low_threshold;
      float high_threshold;
      vec3 position;

      edge_threshold = 0.01;
      low_threshold = edge_threshold;
      high_threshold = 1.0 - edge_threshold;

      // detect edges using fract parts; only works for cubes
      position = (original_position + vec3(1.0, 1.0, 1.0)) / 2;
      fract_x = fract(position.x);
      fract_y = fract(position.y);
      fract_z = fract(position.z);

      is_x_near_edge = fract_x < low_threshold || fract_x > high_threshold ? 1 : 0;
      is_y_near_edge = fract_y < low_threshold || fract_y > high_threshold ? 1 : 0;
      is_z_near_edge = fract_z < low_threshold || fract_z > high_threshold ? 1 : 0;

      color = is_x_near_edge + is_y_near_edge + is_z_near_edge >= 2 ? vec4(0.1, 0.1, 0.1, 1.0) : vec4(0.8, 0.8, 0.8, 1.0);
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
  pub fn draw(&mut self, time: u128) {
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

    self.display.gl_window().swap_buffers().unwrap();

    // fps count
    self.frame_rendered_count += 1;
    //println!("frame #{}", self.frame_rendered_count);
    let duration = time - self.fps_start_at;
    if duration > 10000 {
      let fps = self.frame_rendered_count as f32 / duration as f32;
      println!("FPS: {}", 1000.0*fps);
      self.fps_start_at = time;
      self.frame_rendered_count = 0;
    }
  }
}