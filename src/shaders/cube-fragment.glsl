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