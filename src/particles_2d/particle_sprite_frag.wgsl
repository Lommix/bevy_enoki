#import bevy_enoki::particle_vertex_out::{ VertexOutput }

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;
@group(2) @binding(2) var<uniform> uni: Uniform;


struct Uniform{
	frames : u32,
	_padding : vec3<f32>,
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
	var out = in.color;
	var uv = in.uv;
	let max_frame = f32(uni.frames + 1);
	let frame_step = 1. / max_frame;
	let current_frame = u32(max_frame * (in.lifetime_frac));
	uv.x = uv.x / max_frame + f32(current_frame) * frame_step;
	out = out * textureSample(texture, texture_sampler, uv);

    return out;
}
