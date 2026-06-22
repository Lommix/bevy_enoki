#import bevy_render::view::View
#import bevy_enoki::particle_vertex_out::{ VertexOutput }

@group(0) @binding(0) var<uniform> view: View;

struct VertexIn {
    @builtin(vertex_index) index: u32,
    @location(0) i_transform: vec4<f32>,
    @location(1) i_scale_lifetime: vec4<f32>,
    @location(2) i_color: vec4<f32>,
};

@vertex
fn vertex(in: VertexIn) -> VertexOutput {
    var out: VertexOutput;

    let vertex_position = vec3<f32>(
        f32(in.index & 0x1u),
        f32((in.index & 0x2u) >> 1u),
        0.0
    );

    let local = vertex_position.xy - vec2(0.5, 0.5);
    let scaled = local * in.i_scale_lifetime.xy;
    let angle = in.i_transform.w;
    let sine = sin(angle);
    let cosine = cos(angle);
    let rotated = vec2(
        cosine * scaled.x - sine * scaled.y,
        sine * scaled.x + cosine * scaled.y,
    );
    let world_position = vec3(in.i_transform.xy + rotated, in.i_transform.z);

    out.clip_position = view.clip_from_world * vec4(world_position, 1.0);

    out.color = in.i_color;
	out.uv = vec2(vertex_position.x, 1.-vertex_position.y);

	out.lifetime_frac = in.i_scale_lifetime.z;
	out.lifetime_total = in.i_scale_lifetime.w;

    return out;
}
