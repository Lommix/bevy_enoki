#define_import_path bevy_enoki::particle_vertex_out

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
	@location(1) uv : vec2<f32>,
	@location(2) lifetime : f32,
};
