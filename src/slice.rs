use std::io::Read;

use crate::{reader::AseReader, user_data::UserData, Result};

/// A slice is a region of the sprite with a name and optional [UserData].
#[derive(Debug, Clone)]
pub struct Slice {
    /// The name of the slice. Not guaranteed to be unique.
    pub name: String,
    /// A set of [SliceKey] structs. Together, these describe the shape and position of a slice during animation.
    pub keys: Vec<SliceKey>,
    /// Optional [UserData] associated with this slice.
    pub user_data: Option<UserData>,
}

/// A Slice9 divides a [Slice] into nine regions for 9-slice scaling.
#[derive(Debug, Clone, Copy)]
pub struct Slice9 {
    /// Center X position (relative to slice bounds).
    pub center_x: i32,
    /// Center Y position (relative to slice bounds).
    pub center_y: i32,
    /// Center width.
    pub center_width: u32,
    /// Center height.
    pub center_height: u32,
}
impl Slice9 {
    fn read<R: Read>(reader: &mut AseReader<R>) -> Result<Self> {
        let center_x = reader.long()?;
        let center_y = reader.long()?;
        let center_width = reader.dword()?;
        let center_height = reader.dword()?;
        Ok(Self {
            center_x,
            center_y,
            center_width,
            center_height,
        })
    }
}

/// A SliceOrigin describes the position of a [Slice] within the sprite.
#[derive(Debug, Clone, Copy)]
pub struct SliceOrigin {
    /// A [Slice]'s x origin coordinate in the sprite.
    pub x: i32,
    /// A [Slice]'s y origin coordinate in the sprite.
    pub y: i32,
}
impl SliceOrigin {
    fn read<R: Read>(reader: &mut AseReader<R>) -> Result<Self> {
        let x = reader.long()?;
        let y = reader.long()?;
        Ok(Self { x, y })
    }
}

/// SliceSize describes the size of a [Slice] in pixels.
#[derive(Debug, Clone, Copy)]
pub struct SliceSize {
    /// Slice width. This can be 0 if this slice is hidden in the animation from the given frame.
    pub width: u32,
    /// Slice height.
    pub height: u32,
}
impl SliceSize {
    fn read<R: Read>(reader: &mut AseReader<R>) -> Result<Self> {
        let width = reader.dword()?;
        let height = reader.dword()?;
        Ok(Self { width, height })
    }
}

/// SlicePivot describes a [Slice]'s pivot position relative to the Slice's origin.
#[derive(Debug, Clone, Copy)]
pub struct SlicePivot {
    /// Pivot X position (relative to the slice origin).
    pub x: i32,
    /// Pivot Y position (relative to the slice origin).
    pub y: i32,
}
impl SlicePivot {
    fn read<R: Read>(reader: &mut AseReader<R>) -> Result<Self> {
        let x = reader.long()?;
        let y = reader.long()?;
        Ok(Self { x, y })
    }
}

/// SliceKey describes the position and shape of a [Slice], starting at a given frame.
#[derive(Debug, Clone, Copy)]
pub struct SliceKey {
    /// Starting frame number for this slice key. (This slice is valid from this frame to the end of the animation.)
    pub from_frame: u32,
    /// Origin of the slice.
    pub origin: SliceOrigin,
    /// Size of the slice.
    pub size: SliceSize,
    /// Optional 9-slicing information.
    pub slice9: Option<Slice9>,
    /// Optional pivot information.
    pub pivot: Option<SlicePivot>,
}
impl SliceKey {
    fn read<R: Read>(reader: &mut AseReader<R>, flags: u32) -> Result<Self> {
        let from_frame = reader.dword()?;
        let origin = SliceOrigin::read(reader)?;
        let size = SliceSize::read(reader)?;
        let slice9 = if flags & 1 != 0 {
            Some(Slice9::read(reader)?)
        } else {
            None
        };
        let pivot = if flags & 2 != 0 {
            Some(SlicePivot::read(reader)?)
        } else {
            None
        };

        Ok(Self {
            from_frame,
            origin,
            size,
            slice9,
            pivot,
        })
    }
}

pub(crate) fn parse_chunk(data: &[u8]) -> Result<Slice> {
    let mut reader = AseReader::new(data);

    let num_slice_keys = reader.dword()?;
    let flags = reader.dword()?;
    let _reserved = reader.dword()?;
    let name = reader.string()?;
    let slice_keys: Result<Vec<SliceKey>> = (0..num_slice_keys)
        .map(|_id| SliceKey::read(&mut reader, flags))
        .collect();

    Ok(Slice {
        name,
        keys: slice_keys?,
        user_data: None,
    })
}
