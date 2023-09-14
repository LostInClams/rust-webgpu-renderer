
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vertex_color: vec3<f32>,
};

@vertex
fn vs_main (
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.vertex_color = vec3<f32>(0.1, 1.0, 0.1);
    return out;
}

@vertex
fn vs_main_2 (
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.vertex_color = vec3<f32>(1.0 * f32(i32(in_vertex_index | 1u)), 0.1, 1.0 * f32(i32(in_vertex_index & 1u)));
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(in.vertex_color, 1.0);
}

@fragment
fn fs_main_2(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(0.1, 1.0, 0.1, 1.0);
}
