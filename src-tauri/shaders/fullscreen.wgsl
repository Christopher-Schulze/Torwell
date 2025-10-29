struct VertexOutput {
    @builtin(position) position : vec4<f32>;
    @location(0) uv : vec2<f32>;
};

@vertex
fn vs_main(@location(0) position : vec2<f32>, @location(1) uv : vec2<f32>) -> VertexOutput {
    var out : VertexOutput;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in : VertexOutput) -> @location(0) vec4<f32> {
    let gradient = vec3<f32>(in.uv.x, in.uv.y, max(0.0, 1.0 - 0.5 * in.uv.x));
    return vec4<f32>(gradient, 1.0);
}
