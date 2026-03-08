import include.random;
import include.ray;

[vk::binding(0, 0)]
[format("rgba32f")]
WTexture2D output_texture;

struct Camera
{
    float4 position;
    float4 forward;
    float4 up;
    float4 right;
    float fov;
    uint samples;
    uint max_bounces;
    uint random_seed;
}

[vk::binding(0, 1)]
ConstantBuffer<Camera> camera;

struct ObjectsInfo
{
    uint foo;
}

[vk::binding(0, 2)]
ConstantBuffer<ObjectsInfo> objects_info;

[shader("compute")]
[numthreads(16, 16, 1)]
void trace_rays(uint3 global_index: SV_DispatchThreadID)
{
    var width : uint;
    var height : uint;
    output_texture.GetDimensions(width, height);

    if (global_index.x >= width || global_index.y >= height)
        return;

    let aspect = float(width) / float(height);
    let tan_fov = tan(camera.fov * 0.5);

    var random_state = camera.random_seed + uint(global_index.x * 190146947u + global_index.y * 703548749u);

    var color : float3;
    for (var sample = 0u; sample < camera.samples; sample++)
    {
        let random_offset = float2(random_value(random_state), random_value(random_state));
        let uv = ((float2(global_index.xy) + random_offset) / float2(width, height)) * 2.0 - 1.0;

        let position = camera.position;
        let forward = camera.forward;
        let up = camera.up;
        let right = camera.right;

        var ray : Ray;
        ray.origin = camera.position;
        ray.direction = normalize(forward + (up * uv.y * tan_fov) + (right * uv.x * tan_fov * aspect));

        color += trace_ray(ray, random_state);
    }
    color /= camera.samples;
    output_texture.Store(global_index.xy, float4(color, 1.0));
}

float3 trace_ray(Ray ray, inout uint random_state)
{
    return skybox(ray.direction);
}
