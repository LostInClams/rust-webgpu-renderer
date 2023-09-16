struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vertex_color: vec3<f32>,
};

@vertex
fn vs_main (
   vert: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(vert.position, 1.0);
    out.vertex_color = vert.color;
    return out;
}

@vertex
fn vs_main_2 (
    vert: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(vert.position, 1.0);
    out.vertex_color = vec3<f32>(1.0 - vert.color.rgb);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(in.vertex_color, 1.0);
}

@fragment
fn fs_main_2(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(in.vertex_color, 1.0);
}
