
struct VertexOutput {
    @builtin(position) clip_positon: vec4<f32>,
    @location(0) position: vec2<f32>,
}


@vertex
fn vertex_main(
    @builtin(vertex_index)
    vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    var x = f32(vertex_index & 1u);
    var y = f32(vertex_index & 2u) / 2.;

    out.clip_positon = vec4<f32>(
        x * 2. - 1.,
        y * 2. - 1.,
        0.0,
        1.0,
    );

    out.position = vec2<f32>(
        x * 2. - 1.,
        y * 2. - 1.,
    );

    return out;
}


struct View {
    position: vec2<f32>,
    size: vec2<f32>,
    max_iterations: u32,
}

@group(0) @binding(0)
var<uniform> view: View;

@fragment
fn fragment_main(
    vertex_output: VertexOutput
) -> @location(0) vec4<f32> {

    let x = view.position.x + vertex_output.position.x * view.size.x;
    let y = view.position.y + vertex_output.position.y * view.size.y;

    var r = 0.;
    var i = 0.;

    for (var n = 0u; n < view.max_iterations; n ++) {
        var next_r = r*r - i*i + x;
        var next_i = 2. * r * i + y;

        r = next_r;
        i = next_i;

        if (r*r + i*i >= 4.) {
            return vec4<f32>(f32(n % 20u) / 20., 1., 1., 1.);
        }
    }

    return vec4<f32>(0., 0., 0., 1.0);
}
