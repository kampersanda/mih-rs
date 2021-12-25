use std::io::{Read, Write};

use anyhow::Result;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use num_traits::int::PrimInt;
use num_traits::{FromPrimitive, ToPrimitive};

/// Generic trait of binary codes.
pub trait CodeInt: PrimInt + FromPrimitive + ToPrimitive + Popcnt + Default {
    fn dimensions() -> usize;
    fn serialize_into<W: Write>(&self, writer: W) -> Result<()>;
    fn deserialize_from<R: Read>(reader: R) -> Result<Self>;
}

impl CodeInt for u8 {
    fn dimensions() -> usize {
        8
    }

    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<()> {
        writer.write_u8(*self)?;
        Ok(())
    }

    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self> {
        let x = reader.read_u8()?;
        Ok(x)
    }
}

impl CodeInt for u16 {
    fn dimensions() -> usize {
        16
    }

    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<()> {
        writer.write_u16::<LittleEndian>(*self)?;
        Ok(())
    }

    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self> {
        let x = reader.read_u16::<LittleEndian>()?;
        Ok(x)
    }
}

impl CodeInt for u32 {
    fn dimensions() -> usize {
        32
    }

    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<()> {
        writer.write_u32::<LittleEndian>(*self)?;
        Ok(())
    }

    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self> {
        let x = reader.read_u32::<LittleEndian>()?;
        Ok(x)
    }
}

impl CodeInt for u64 {
    fn dimensions() -> usize {
        64
    }

    fn serialize_into<W: Write>(&self, mut writer: W) -> Result<()> {
        writer.write_u64::<LittleEndian>(*self)?;
        Ok(())
    }

    fn deserialize_from<R: Read>(mut reader: R) -> Result<Self> {
        let x = reader.read_u64::<LittleEndian>()?;
        Ok(x)
    }
}

/// Generic trait for pop-countable integers.
pub trait Popcnt {
    fn popcnt(&self) -> u32;
}

impl Popcnt for u8 {
    fn popcnt(&self) -> u32 {
        self.count_ones()
    }
}

impl Popcnt for u16 {
    fn popcnt(&self) -> u32 {
        self.count_ones()
    }
}

impl Popcnt for u32 {
    fn popcnt(&self) -> u32 {
        self.count_ones()
    }
}

impl Popcnt for u64 {
    fn popcnt(&self) -> u32 {
        self.count_ones()
    }
}
