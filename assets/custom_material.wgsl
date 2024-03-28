#import bevy_enoki::particle_vertex_out::{ VertexOutput }

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;


@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
	var out = in.color;

	let speed = 5.0;

	let r_uv = ( in.uv + vec2(0.1,0.) * in.lifetime * speed ) % 1.;
	let b_uv = ( in.uv + vec2(0.5,0.1) * in.lifetime * speed) % 1.;
	let g_uv = ( in.uv + vec2(0.2,-0.1) * in.lifetime * speed) % 1.;

	let r = textureSample(texture, texture_sampler, r_uv).r * 2.;
	let b = textureSample(texture, texture_sampler, b_uv).b * 2.;
	let g = textureSample(texture, texture_sampler, g_uv).g * 2.;

	let dist_center = distance(in.uv, vec2(0.5));
	let energy = (r + b + g) / 3.;

	let fade_out =  1. - dist_center * 2.;
	out.a =
	smoothstep(dist_center, dist_center + 0.4, 0.4)
	* smoothstep(energy,energy + 0.2,.3)
	* fade_out;



    return out;
}


fn noise(uv: vec2<f32>) -> f32 {
    return fract(sin(dot(uv, vec2<f32>(12.9898, 58.233))) * 43758.5453);
}
