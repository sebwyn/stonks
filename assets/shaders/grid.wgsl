#import bevy_sprite::mesh2d_vertex_output::VertexOutput

#import bevy_sprite::

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(0.5, 0.5, 0.5, 1.0);
}