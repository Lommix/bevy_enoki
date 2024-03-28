#import bevy_render::{ maths::affine_to_square, view::View }
#import bevy_enoki::particle_vertex_out::{ VertexOutput }

@group(0) @binding(0) var<uniform> view: View;

struct VertexIn {
    @builtin(vertex_index) index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) i_translation: vec4<f32>,
    @location(4) i_rotation: vec4<f32>,
    @location(5) i_scale: vec4<f32>,
    @location(6) i_color: vec4<f32>,
    @location(7) i_lifetime: vec4<f32>,
};

@vertex
fn vertex(vertex: VertexIn) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = view.view_proj * affine_to_square(mat3x4<f32>(
        vertex.i_translation,
        vertex.i_rotation,
        vertex.i_scale,
    )) * vec4<f32>(vertex.position, 1.0);

    out.color = vertex.i_color;
	out.uv = vertex.uv;

	out.lifetime_frac = vertex.i_lifetime.x;
	out.lifetime_total = vertex.i_lifetime.y;

    return out;
}
