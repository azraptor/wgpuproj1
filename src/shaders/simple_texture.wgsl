struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_uv: vec2<f32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(2) tex_uv: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    // Setup output struct
    var out: VertexOutput;

    // Assign color and texture UVs
    out.color = model.color;
    out.tex_uv = model.tex_uv;

    // Clip position adjusted by perspective
    out.clip_position = camera.view_proj * model.position;
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_uv) * in.color;
}
