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