use glfw::{
  Action,
  Key,
  Context,
  WindowEvent
};
use luminance::{
  context::GraphicsContext,
  framebuffer::Framebuffer,
  pipeline::PipelineState,
  render_state::RenderState,
  shader::{
    Program,
    Uniform
  },
  tess::{
    Mode,
    Tess
  },
  texture::{
    Dim2
  }
};
use luminance_derive::{
  Semantics,
  UniformInterface,
  Vertex
};
use luminance_gl::gl33::GL33;
use luminance_glfw::GlfwSurface;
use luminance_windowing::{
  WindowDim, 
  WindowOpt
};
use nalgebra_glm::Mat4;

const VS: &'static str = include_str!("shaders/cube-vertex.glsl");
const FS: &'static str = include_str!("shaders/cube-fragment.glsl");

#[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
pub enum Semantics {
  #[sem(name = "position", repr = "[f32; 3]", wrapper = "VertexPosition")]
  Position,

  #[sem(name = "color", repr = "[f32; 3]", wrapper = "VertexColor")]
  Color,
}

#[derive(Debug, UniformInterface)]
struct ShaderInterface {
  projection_matrix: Uniform<[[f32; 4]; 4]>,
  view_matrix: Uniform<[[f32; 4]; 4]>,
  world_matrix: Uniform<[[f32; 4]; 4]>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
struct Vertex {
  position: VertexPosition,
  color: VertexColor,
}

const CUBE_VERTICES: [Vertex; 8] = [
  Vertex::new(VertexPosition::new([-1.0,  1.0,  1.0]), VertexColor::new([0.0, 1.0, 1.0]) ),
  Vertex::new(VertexPosition::new([-1.0, -1.0,  1.0]), VertexColor::new([1.0, 1.0, 0.0]) ),
  Vertex::new(VertexPosition::new([ 1.0, -1.0,  1.0]), VertexColor::new([1.0, 0.0, 1.0]) ),
  Vertex::new(VertexPosition::new([ 1.0,  1.0,  1.0]), VertexColor::new([1.0, 1.0, 1.0]) ),
  Vertex::new(VertexPosition::new([ 1.0,  1.0, -1.0]), VertexColor::new([0.0, 1.0, 0.0]) ),
  Vertex::new(VertexPosition::new([ 1.0, -1.0, -1.0]), VertexColor::new([0.0, 0.0, 1.0]) ),
  Vertex::new(VertexPosition::new([-1.0, -1.0, -1.0]), VertexColor::new([1.0, 0.0, 0.0]) ),
  Vertex::new(VertexPosition::new([-1.0,  1.0, -1.0]), VertexColor::new([0.0, 0.0, 0.0]) ), 
];

const CUBE_INDICES: [u16; 36] = [
  0, 1, 2, 2, 3, 0,
  3, 2, 5, 5, 4, 3,
  4, 5, 6, 6, 7, 4,
  7, 6, 1, 1, 0, 7,
  2, 1, 6, 6, 5, 2,
  7, 0, 3, 3, 4, 7
];

pub struct Renderer {
  surface: GlfwSurface,
  program: Program<GL33, Semantics, (), ShaderInterface>,
  mesh: Tess<GL33, Vertex, u16>,
  back_buffer: Framebuffer<GL33, Dim2, (), ()>,

  spin_velocity: f32,
  spin_h_neg: bool,
  spin_h_pos: bool,
  spin_v_neg: bool,
  spin_v_pos: bool,
  x_angle: f32,
  y_angle: f32,
  world_matrix: [[f32; 4]; 4],
  projection_matrix: [[f32; 4]; 4],
  view_matrix: [[f32; 4]; 4],
  frame_rendered_count: u32,
  fps_start_at: u128
}

impl Renderer {
  // init a new renderer
  pub fn new() -> Renderer {
    let mut surface = GlfwSurface::new_gl33(
      "Desperate: Dig",
      WindowOpt::default()
        .set_dim(WindowDim::Windowed {
          width: 1280,
          height: 720,
        })
      ).unwrap();
    
    let mesh = Renderer::init_mesh(&mut surface);
    let program = Renderer::init_shaders(&mut surface);
    let matrices = Renderer::init_matrices(&mut surface);
    let back_buffer = surface.back_buffer().unwrap();

    Renderer {
      surface: surface,
      program: program,
      mesh: mesh,
      back_buffer: back_buffer,

      projection_matrix: matrices.0.into(),
      view_matrix: matrices.1.into(),
      world_matrix: matrices.2.into(),
      frame_rendered_count: 0,
      fps_start_at: 0,
      spin_velocity: 180.0_f32.to_radians(), // radians per sec
      spin_h_neg: false,
      spin_h_pos: false,
      spin_v_neg: false,
      spin_v_pos: false,
      x_angle: 0.0,
      y_angle: 0.0
    }
  }

  fn init_mesh(surface: &mut GlfwSurface) -> Tess<GL33, Vertex, u16> {
    surface
      .new_tess()
      .set_vertices(&CUBE_VERTICES[..])
      .set_indices(&CUBE_INDICES[..])
      .set_mode(Mode::Triangle)
      .build()
      .unwrap()
  }

  // init shaders
  fn init_shaders(surface: &mut GlfwSurface) -> Program<GL33, Semantics, (), ShaderInterface> {
    surface
      .new_shader_program::<Semantics, (), ShaderInterface>()
      .from_strings(VS, None, None, FS)
      .expect("shaders creation")
      .ignore_warnings()
  }

  // run application loop
  pub fn run(&mut self) {
    let start_time = std::time::Instant::now();
    let mut start_frame_time = std::time::Instant::now();
    let mut commands: Vec<(Key, bool)> = Vec::new();

    'app: loop {
      // deal with events
      self.surface.window.glfw.poll_events();
      for (_, event) in self.surface.events_rx.try_iter() {      
        match event {
          WindowEvent::Close => break 'app,
          WindowEvent::Key(key, _, action, _) => match action {
              Action::Repeat => (), // filters out repeat
              _ => commands.push((key, action == Action::Press))
            },
          _ => (),
        }
      }

      // deal with commands
      if commands.is_empty() {
        // no more commands to process, let's draw !
        let current_time = std::time::Instant::now();
        let game_time = current_time.duration_since(start_time).as_millis();
        let frame_time = current_time.duration_since(start_frame_time).as_millis();
        self.update(game_time, frame_time);
        self.draw(game_time, frame_time);
        start_frame_time = current_time;
      } else {
        //
        let command = commands.pop().unwrap();

        match command {
          (Key::Escape, _) => break 'app,
          (Key::Up, is_pressed) => {
            println!("up: {}", is_pressed);
            self.set_spin_v_neg(is_pressed)
          },
          (Key::Down, is_pressed) => {
            println!("down: {}", is_pressed);
            self.set_spin_v_pos(is_pressed)
          },
          _ => (),
        }
      }
    }
  }

  pub fn refresh_viewport_size(&mut self) {
    let matrices = Renderer::init_matrices(&self.surface);
    self.projection_matrix = matrices.0.into();
    self.back_buffer = self.surface.back_buffer().unwrap();
  }

  pub fn set_spin_h_neg(&mut self, on: bool) {
    self.spin_h_neg = on;
  }

  pub fn set_spin_h_pos(&mut self, on: bool) {
    self.spin_h_pos = on;
  }

  pub fn set_spin_v_neg(&mut self, on: bool) {
    self.spin_v_neg = on;
  }

  pub fn set_spin_v_pos(&mut self, on: bool) {
    self.spin_v_pos = on;
  }

  /// init projection, view and world matrices
  fn init_matrices(surface: &GlfwSurface) -> (Mat4, Mat4, Mat4) {
    // compute viewport
    let window_size = surface.window.get_size();
    let viewport_ratio: f32 = window_size.0 as f32 / window_size.1 as f32;
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

    let projection_matrix = nalgebra_glm::perspective_fov(
      fov.to_radians(), 
      viewport_width, 
      viewport_height, 
      0.1, 
      1000.0);

    // return
    (projection_matrix, view_matrix, world_matrix)
  }

  /// Update current states according to current time
  pub fn update(&mut self, game_time: u128, frame_time: u128) {
    let mut spin_h = 0.0;
    if self.spin_h_neg {
      spin_h -= 1.0;
    }

    if self.spin_h_pos {
      spin_h += 1.0;
    }

    let mut spin_v = 0.0;
    if self.spin_v_neg {
      spin_v -= 1.0;
    }

    if self.spin_v_pos {
      spin_v += 1.0;
    }

    let two_pi = 360.0_f32.to_radians();

    self.x_angle += self.spin_velocity * spin_v * (frame_time as f32 / 1000.0);
    self.y_angle += self.spin_velocity * spin_h * (frame_time as f32 / 1000.0);

    self.x_angle %= two_pi;
    self.y_angle %= two_pi;

    let rotate_x = nalgebra_glm::rotate_x(&Mat4::identity(), self.x_angle);
    let rotate_y = nalgebra_glm::rotate_y(&Mat4::identity(), self.y_angle);
/*
      let rotate_matrix = nalgebra_glm::rotate(
      &Mat4::identity(),
      //((time as f32 * 0.06) % 360.0).to_radians(),
      360.0_f32.to_radians(),
      &nalgebra_glm::vec3((self.x_angle+0.1) / two_pi, self.y_angle / two_pi, 0.0)
    );
*/
    let translate_matrix = nalgebra_glm::translate(
      &Mat4::identity(), 
      &nalgebra_glm::vec3(0.0_f32, 0.0, -5.0));

    self.view_matrix = (translate_matrix * rotate_x * rotate_y).into();
  }

  /// Draw next frame
  pub fn draw(&mut self, game_time: u128, frame_time: u128) {
    let mesh = &self.mesh;
    let program = &mut self.program;
    let pipeline_state = PipelineState::default()
      .set_clear_color([0.8, 0.9, 0.9, 1.0]);

    let projection_matrix: [[f32; 4]; 4] = self.projection_matrix;
    let view_matrix: [[f32; 4]; 4] = self.view_matrix;
    let world_matrix: [[f32; 4]; 4] = self.world_matrix;
    
    let render = self
      .surface
      .new_pipeline_gate()
      .pipeline(
        &self.back_buffer,
        &pipeline_state,
        |pipeline, mut shd_gate| {
          shd_gate.shade(program, |mut iface, uni, mut rdr_gate| {
            iface.set(&uni.projection_matrix, projection_matrix);
            iface.set(&uni.view_matrix, view_matrix);
            iface.set(&uni.world_matrix, world_matrix);

            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
              tess_gate.render(mesh)
            })
          })
        }
      )
      .assume();

    if render.is_ok() {
      self.surface.window.swap_buffers();
    } else {
      panic!();
    }

    // fps count
    self.frame_rendered_count += 1;
    //println!("frame #{}", self.frame_rendered_count);
    let duration = game_time - self.fps_start_at;
    if duration > 1000 {
      let fps = self.frame_rendered_count as f32 / duration as f32;
      println!("FPS: {}", 1000.0*fps);
      self.fps_start_at = game_time;
      self.frame_rendered_count = 0;
    }
  }
}