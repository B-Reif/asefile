use crate::{pixel, reader::AseReader, AsepriteParseError, Result};
use nohash::IntMap;

/// The color palette embedded in the file.
#[derive(Debug)]
pub struct ColorPalette {
    //entries: Vec<ColorPaletteEntry>,
    entries: IntMap<u32, ColorPaletteEntry>,
}

/// A single entry in a [ColorPalette].
#[derive(Debug)]
pub struct ColorPaletteEntry {
    id: u32,
    rgba8: [u8; 4],
    name: Option<String>,
}

impl ColorPalette {
    /// Total number of colors in the palette.
    pub fn num_colors(&self) -> u32 {
        self.entries.len() as u32
    }

    /// Look up entry at given index.
    ///
    /// The Aseprite file format spec does not guarantee the color indices to
    /// go from `0..num_colors()` but there doesn't seem to be a way to violate
    /// this constraint using the Aseprite GUI.
    pub fn color(&self, index: u32) -> Option<&ColorPaletteEntry> {
        self.entries.get(&index)
    }

    pub(crate) fn validate_indexed_pixels(&self, indexed_pixels: &[pixel::Indexed]) -> Result<()> {
        for pixel in indexed_pixels {
            let color = self.color(pixel.value().into());
            color.ok_or_else(|| {
                AsepriteParseError::InvalidInput(format!(
                    "Index out of range: {} (max: {})",
                    pixel.value(),
                    self.num_colors()
                ))
            })?;
        }
        Ok(())
    }
}

impl ColorPaletteEntry {
    /// The id of this entry is the same as its index in the palette.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Get the RGBA components as an array. Most color libraries allow you to
    /// build an instance of their color type from such an array.
    pub fn raw_rgba8(&self) -> [u8; 4] {
        self.rgba8
    }

    /// The red channel of the color.
    pub fn red(&self) -> u8 {
        self.rgba8[0]
    }

    /// The green channel of the color.
    pub fn green(&self) -> u8 {
        self.rgba8[1]
    }

    /// The blue channel of the color.
    pub fn blue(&self) -> u8 {
        self.rgba8[2]
    }

    /// Alpha value of this color (0 = fully transparent, 255 = fully opaque).
    pub fn alpha(&self) -> u8 {
        self.rgba8[3]
    }

    /// The color name. Seems to be usually empty in practice.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

pub(crate) fn parse_chunk(data: &[u8]) -> Result<ColorPalette> {
    let mut reader = AseReader::new(data);

    let _num_total_entries = reader.dword()?;
    let first_color_index = reader.dword()?;
    let last_color_index = reader.dword()?;
    reader.skip_reserved(8)?;

    if last_color_index < first_color_index {
        return Err(AsepriteParseError::InvalidInput(format!(
            "Bad palette color indices: first={} last={}",
            first_color_index, last_color_index,
        )));
    }

    let count = last_color_index - first_color_index + 1;
    //let mut entries = Vec::with_capacity(count as usize);
    let mut entries = IntMap::default();

    for id in 0..count {
        let flags = reader.word()?;
        let red = reader.byte()?;
        let green = reader.byte()?;
        let blue = reader.byte()?;
        let alpha = reader.byte()?;
        let name = if flags & 1 == 1 {
            let s = reader.string()?;
            Some(s)
        } else {
            None
        };
        let id = id + first_color_index;
        entries.insert(
            id,
            ColorPaletteEntry {
                id,
                rgba8: [red, green, blue, alpha],
                name,
            },
        );
    }

    Ok(ColorPalette { entries })
}
