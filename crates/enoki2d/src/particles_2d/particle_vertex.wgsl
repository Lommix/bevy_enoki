#import bevy_render::{ maths::affine3_to_square, view::View }
#import bevy_enoki::particle_vertex_out::{ VertexOutput }

@group(0) @binding(0) var<uniform> view: View;

struct VertexIn {
    @builtin(vertex_index) index: u32,
    @location(0) i_translation: vec4<f32>,
    @location(1) i_rotation: vec4<f32>,
    @location(2) i_scale: vec4<f32>,
    @location(3) i_color: vec4<f32>,
    @location(4) i_lifetime: vec4<f32>,
};

@vertex
fn vertex(in: VertexIn) -> VertexOutput {
    var out: VertexOutput;

    let vertex_position = vec3<f32>(
        f32(in.index & 0x1u),
        f32((in.index & 0x2u) >> 1u),
        0.0
    );

    out.clip_position = view.clip_from_world * affine3_to_square(mat3x4<f32>(
        in.i_translation,
        in.i_rotation,
        in.i_scale,
    )) * vec4<f32>(vertex_position - vec3(0.5,0.5,0.), 1.0);

    out.color = in.i_color;
	out.uv = vec2(vertex_position.x, 1.-vertex_position.y);

	out.lifetime_frac = in.i_lifetime.x;
	out.lifetime_total = in.i_lifetime.y;

    return out;
}
