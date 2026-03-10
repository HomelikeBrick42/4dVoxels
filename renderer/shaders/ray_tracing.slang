import include.random;
import include.ray;
import include.chunks;

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
    uint chunks_count;
}

[vk::binding(0, 2)]
ConstantBuffer<ObjectsInfo> objects_info;

[vk::binding(1, 2)]
StructuredBuffer<Block> block_list;

[vk::binding(2, 2)]
StructuredBuffer<Chunk> chunk_list;

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
        var random_offset = float2(random_value(random_state), random_value(random_state));
        if (camera.samples == 1u)
            random_offset = 0.5;

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
    if (objects_info.chunks_count > 0)
    {
        let color = hit_chunk(ray, random_state);
        if (color.hasValue)
            return color.value;
    }

    return skybox(ray.direction);
}

Optional<float3> hit_chunk(Ray ray, inout uint random_state)
{
    // if its exact then issues with the divisions can happen
    for (var i = 0u; i < 4u; i++)
        if (ray.direction[i] == 0.0)
            ray.direction[i] = 0.000001;

    let step_sizes = 1.0 / abs(ray.direction);
    let step_direction = sign(ray.direction);
    var next_dist = (step_direction * 0.5 + 0.5 - fract(ray.origin)) / ray.direction;

    let max_distance = 20.0;
    var distance = 0.0;
    var voxel_pos = floor(ray.origin);

    for (var i = 0u; i < uint(max_distance * 4); i++)
    {
        let closest_distance = min(min(min(next_dist.x, next_dist.y), next_dist.z), next_dist.w);

        // could use a comparison operator here but that can skip through blocks sometimes
        // maybe this could be allowed if it becomes a performance issue
        var step_axis : float4;
        if (next_dist.x == closest_distance)
            step_axis = float4(1.0, 0.0, 0.0, 0.0);
        else if (next_dist.y == closest_distance)
            step_axis = float4(0.0, 1.0, 0.0, 0.0);
        else if (next_dist.z == closest_distance)
            step_axis = float4(0.0, 0.0, 1.0, 0.0);
        else if (next_dist.w == closest_distance)
            step_axis = float4(0.0, 0.0, 0.0, 1.0);

        voxel_pos += step_axis * step_direction;
        distance += closest_distance;
        next_dist -= float4(closest_distance);
        next_dist += step_sizes * step_axis;

        if (distance > max_distance)
            break;

        if (all(voxel_pos >= float4(0.0) && voxel_pos < float4(CHUNK_SIZE)))
        {
            let voxel_pos = uint4(voxel_pos);
            let block_id = chunk_list[0].blocks[voxel_pos.x][voxel_pos.y][voxel_pos.z][voxel_pos.w];
            let block = block_list[block_id.index];

            let light = max(0.3, dot(step_axis * step_direction, -SunDirection) * 0.5 + 0.5);
            if (block.typ == BlockType.Solid)
                return block.color * light;
        }
    }

    return none;
}
