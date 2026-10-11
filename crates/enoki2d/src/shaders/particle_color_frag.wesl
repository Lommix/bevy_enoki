import bevy_enoki::shaders::particle_vertex_out::VertexOutput;

@group(1) @binding(0) var<uniform> color: vec4<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
	return in.color * color;
}
