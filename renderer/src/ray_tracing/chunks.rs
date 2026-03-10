use bytemuck::{NoUninit, Pod, Zeroable};
use math::{Vector3, Vector4};

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(transparent)]
pub struct BlockId(u32);

impl BlockId {
    pub const AIR: Self = Self(0);
}

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(u32)]
pub enum BlockType {
    Transparent,
    Solid,
}

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
pub struct Block {
    pub color: Vector3<f32>,
    pub typ: BlockType,
}

#[derive(Debug, Clone, Copy, NoUninit)]
#[repr(C)]
pub struct Chunk {
    blocks: [[[[BlockId; Chunk::SIZE]; Chunk::SIZE]; Chunk::SIZE]; Chunk::SIZE],
}

impl Chunk {
    pub const SIZE: usize = 8;

    #[inline]
    pub fn new(mut f: impl FnMut(Vector4<u8>) -> BlockId) -> Self {
        Self {
            blocks: std::array::from_fn(|x| {
                std::array::from_fn(|y| {
                    std::array::from_fn(|z| {
                        std::array::from_fn(|w| {
                            f(Vector4 {
                                x: x as _,
                                y: y as _,
                                z: z as _,
                                w: w as _,
                            })
                        })
                    })
                })
            }),
        }
    }

    #[inline]
    pub fn get(&self, position: Vector4<u8>) -> &BlockId {
        const { assert!(Self::SIZE.is_power_of_two()) };
        &self.blocks[position.x as usize & (Self::SIZE - 1)][position.y as usize & (Self::SIZE - 1)]
            [position.z as usize & (Self::SIZE - 1)][position.w as usize & (Self::SIZE - 1)]
    }

    #[inline]
    pub fn get_mut(&mut self, position: Vector4<u8>) -> &mut BlockId {
        const { assert!(Self::SIZE.is_power_of_two()) };
        &mut self.blocks[position.x as usize & (Self::SIZE - 1)]
            [position.y as usize & (Self::SIZE - 1)][position.z as usize & (Self::SIZE - 1)]
            [position.w as usize & (Self::SIZE - 1)]
    }
}

pub struct Chunks {
    block_list: Vec<Block>,
    chunk: Chunk,
}

impl Chunks {
    #[expect(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            block_list: vec![Block {
                color: Vector3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                },
                typ: BlockType::Transparent,
            }],
            chunk: Chunk::new(|_| BlockId::AIR),
        }
    }

    pub fn block_list(&self) -> &[Block] {
        &self.block_list
    }

    pub fn push_block(&mut self, block: Block) -> BlockId {
        let id = BlockId(self.block_list.len() as _);
        self.block_list.push(block);
        id
    }

    pub fn get_chunk(&self) -> &Chunk {
        &self.chunk
    }

    pub fn get_chunk_mut(&mut self) -> &mut Chunk {
        &mut self.chunk
    }
}
