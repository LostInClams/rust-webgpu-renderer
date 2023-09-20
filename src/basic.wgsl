struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(1)@binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct InstanceInput {
    @location(8)  model_matrix_0: vec4<f32>,
    @location(9)  model_matrix_1: vec4<f32>,
    @location(10) model_matrix_2: vec4<f32>,
    @location(11) model_matrix_3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vertex_color: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main (
   vert: VertexInput,
   instance: InstanceInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(vert.position, 1.0);
    out.vertex_color = vert.color;
    out.uv = vert.uv;
    return out;
}

@vertex
fn vs_main_2 (
    vert: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(vert.position, 1.0);
    out.vertex_color = vert.color.rgb;
    out.uv = vert.uv;
    return out;
}

@group(0)@binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(in.vertex_color * textureSample(t_diffuse, s_diffuse, in.uv).rgb, 1.0);
}

@fragment
fn fs_main_2(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(in.vertex_color  * textureSample(t_diffuse, s_diffuse, in.uv).rgb, 1.0);
}
