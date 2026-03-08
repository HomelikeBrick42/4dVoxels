struct Quad
{
    float x;
    float y;
    float width;
    float height;
}

[vk::binding(0, 0)]
StructuredBuffer<Quad> quads;

[vk::binding(0, 1)]
Texture2D texture;

[vk::binding(1, 1)]
SamplerState sampler;

struct VertexOutput
{
    float4 clip_position : SV_Position;
    float2 uv;
}

[shader("vertex")]
VertexOutput vertex(uint vertex_index: SV_VertexID, uint instance_index: SV_InstanceID)
{
    var out : VertexOutput;
    out.uv = float2(vertex_index & 1, (vertex_index >> 1) & 1);

    let quad = quads[instance_index];
    let position = float2(quad.x, quad.y);
    let size = float2(quad.width, quad.height);
    out.clip_position = float4(position + size * (out.uv - 0.5), 0.0, 1.0);

    return out;
}

struct FragmentOutput
{
    float4 color : SV_Target;
}

[shader("fragment")]
FragmentOutput fragment(VertexOutput in)
{
    var out : FragmentOutput;
    out.color = texture.Sample(sampler, in.uv);
    return out;
}
