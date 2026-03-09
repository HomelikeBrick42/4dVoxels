struct BlockId
{
    uint index;
}

const static BlockId AIR = BlockId(0);

enum BlockType
{
    Transparent,
    Solid,
}

struct Block
{
    float3 color;
    BlockType typ;
}

const static uint CHUNK_SIZE = 8;

struct Chunk
{
    Array<Array<Array<Array<BlockId, CHUNK_SIZE>, CHUNK_SIZE>, CHUNK_SIZE>, CHUNK_SIZE> blocks;
}
