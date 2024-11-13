#import bevy_enoki::particle_vertex_out::{ VertexOutput }

@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;
@group(1) @binding(2) var<uniform> frame_data: vec4<u32>;


@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
	var out = in.color;

	let max_hframe = f32(frame_data.x);
	let max_vframe = f32(frame_data.y);

    let total_frames = max_hframe * max_vframe;
    let current_frame = floor(in.lifetime_frac * total_frames);

    let hframe = current_frame % max_hframe;
    let vframe = floor(current_frame / max_hframe);

    let frame_width = 1.0 / max_hframe;
    let frame_height = 1.0 / max_vframe;

    let u_offset = hframe * frame_width;
    let v_offset = (max_vframe - vframe - 1.0) * frame_height;

    let uv = in.uv * vec2<f32>(frame_width, frame_height) + vec2<f32>(u_offset, v_offset);
	return out * textureSample(texture, texture_sampler, uv);
}
