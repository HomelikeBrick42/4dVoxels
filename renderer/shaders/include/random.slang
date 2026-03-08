float random_value(inout uint state)
{
    state = state * 747796405u + 2891336453u;
    var result = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    result = (result >> 22u) ^ result;
    return float(result) / 4294967295.0;
}

float random_value_normal_distribution(inout uint state)
{
    let theta = 2.0 * 3.1415926 * random_value(state);
    let rho = sqrt(-2.0 * log(random_value(state)));
    return rho * cos(theta);
}

vector<float, N> random_direction<uint N>(inout uint state)
{
    var result : vector<float, N>;
    for (var i = 0u; i < N; i++)
        result[i] = random_value_normal_distribution(state);
    return normalize(result);
}

vector<float, N> random_direction_in_hemisphere<uint N>(vector<float, N> normal, inout uint state)
{
    var direction = random_direction<N>(state);
    if (dot(direction, normal) < 0.0)
        direction = -direction;
    return direction;
}
