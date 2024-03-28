#import bevy_enoki::particle_vertex_out::{ VertexOutput }


@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
