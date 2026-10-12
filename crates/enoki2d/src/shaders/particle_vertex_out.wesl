struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @interpolate(flat)
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) lifetime_frac: f32,
    @location(3) lifetime_total: f32,
}
