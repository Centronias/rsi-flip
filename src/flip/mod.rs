pub mod offsets;

use std::path::PathBuf;

use anyhow::{Result, anyhow, bail};
use clap::ValueEnum;
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageFormat, Rgba, open};
use offsets::{Offsets, OffsetsExt};

use crate::common::i64Ext;

#[derive(clap::Args, Debug)]
/// Horizontally flips the directions in RSIs.
/// See more at <https://github.com/Centronias/rsi-flip>
pub struct FlipArgs {
    /// The path of the image to flip.
    #[arg(short, long)]
    pub path: PathBuf,
    /// The number of directions in this RSI state. If one, flips only horizontally. If four, flips
    /// each as appropriate for its direction. Any other value is an error.
    #[arg(short, long, default_value_t = 4u8)]
    pub directions: u8,
    /// Per-direction pixel offsets applied after flipping. Has no effect when `directions` is 1.
    ///
    /// Format: `<dirs>=<x>,<y>`. `<dirs>` is one or more direction letters (n/s/e/w,
    /// case-insensitive); grouping them applies the same offset to each (e.g. `nsew=0,2`).
    /// `<x>` is the horizontal offset (positive = right), `<y>` is vertical (positive = down).
    ///
    /// Repeat the flag to set different offsets per direction:
    /// `--offsets n=0,2 --offsets s=0,-2`
    #[arg(long)]
    pub offsets: Option<Vec<String>>,
    /// What to do when a pixel's mapped location is outside of the original sprite. This usually happens when an offset is given.
    #[arg(short, long, value_enum, default_value_t = NoSourcePixelBehavior::Transparent)]
    pub no_source_pixel_behavior: NoSourcePixelBehavior,
    /// Output path for the flipped image. Defaults to `<stem>-flipped.png` next to the input.
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

/// Validates args, opens the image, and writes the flipped result to the output path.
pub fn run(args: FlipArgs) -> Result<()> {
    let offsets = args.get_offsets()?;

    if args.directions != 1 && args.directions != 4 {
        bail!("Directions must be 1 or 4");
    }
    if args.directions == 1 && args.offsets.is_some() {
        eprintln!("'offsets' will be ignored when 'directions' is 1");
    }
    let image = open(args.path.as_path())
        .map_err(|e| anyhow!("Failed to open file={}: {e}", args.path.display()))?;

    let (width, height) = image.dimensions();
    if width != height {
        bail!("Image must be square");
    }
    if args.directions == 4 && width % 2 != 0 {
        bail!("Image must be composed of four quadrants");
    }
    let flipped_image = ImageBuffer::from_par_fn(width, height, |x, y| {
        map_pixel(
            &image,
            x,
            y,
            &offsets,
            args.no_source_pixel_behavior,
            match args.directions {
                1 => map_coordinates_one_dir,
                4 => map_coordinates_four_dir,
                _ => unreachable!("We should've validated this value already"),
            },
        )
    });

    let new_path = match args.output {
        Some(p) => p,
        None => args.path.with_file_name(format!(
            "{}-flipped.png",
            args.path
                .file_stem()
                .ok_or_else(|| anyhow!("Given file doesn't have a filename"))?
                .to_string_lossy(),
        )),
    };
    flipped_image.save_with_format(&new_path, ImageFormat::Png)?;
    println!("Saved {}", new_path.display());

    Ok(())
}

/// Returns the source pixel that belongs at destination `(x, y)` after applying `mapper`.
/// Coordinates are promoted to `i64` for arithmetic (offsets can go negative), then cast back
/// to `u32` for pixel lookup. Out-of-bounds results are handled by `no_source_behavior`.
fn map_pixel(
    image: &DynamicImage,
    x: u32,
    y: u32,
    offsets: &Offsets,
    no_source_behavior: NoSourcePixelBehavior,
    mapper: impl Fn(i64, i64, i64, i64, &Offsets) -> (i64, i64),
) -> Rgba<u8> {
    let dimensions = image.dimensions();
    let dim_x = i64::from(dimensions.0);
    let dim_y = i64::from(dimensions.1);

    let (x, y) = mapper(dim_x, dim_y, i64::from(x), i64::from(y), offsets);
    if x >= dim_x || y >= dim_y || x < 0 || y < 0 {
        match no_source_behavior {
            NoSourcePixelBehavior::Transparent => Rgba([0, 0, 0, 0]),
            NoSourcePixelBehavior::Wrap => {
                image.get_pixel(x.rem_euclid_asserting(dim_x), y.rem_euclid_asserting(dim_y))
            }
        }
    } else {
        image.get_pixel(x.assert_positive(), y.assert_positive())
    }
}

/// Maps pixel `x` to its horizontally mirrored position: `width - x - 1`.
const fn map_coordinates_one_dir(width: i64, _h: i64, x: i64, y: i64, _: &Offsets) -> (i64, i64) {
    (width - x - 1, y)
}

/// Maps a pixel to its post-flip position for a 4-direction RSI sprite sheet.
///
/// The sheet is divided into four equal quadrants arranged as:
/// ```text
///   North | South
///    East | West
/// ```
/// Each directional sprite is mirrored horizontally within its own quadrant. Additionally,
/// the East and West sprites swap quadrant positions with each other -- a sprite facing
/// East becomes a sprite facing West after mirroring, so it belongs in the West slot.
fn map_coordinates_four_dir(
    width: i64,
    height: i64,
    x: i64,
    y: i64,
    offsets: &Offsets,
) -> (i64, i64) {
    use Direction::*;

    let hw = width / 2;
    let hh = height / 2;

    if x < hw && y < hh {
        let (ox, oy) = offsets.get_dir(North);
        (hw - (x + 1) + ox, y + oy)
    } else if x >= hw && y < hh {
        let (ox, oy) = offsets.get_dir(South);
        (width - ((x - hw) + 1) + ox, y + oy)
    } else if x < hw && y >= hh {
        let (ox, oy) = offsets.get_dir(East);
        (width - (x + 1) + ox, y + oy)
    } else {
        let (ox, oy) = offsets.get_dir(West);
        (width - (x + 1) + ox, y + oy)
    }
}

/// What to do when a mapped pixel coordinate falls outside the image bounds.
/// This can happen when a direction has a non-zero offset applied.
#[derive(Debug, Copy, Clone, Eq, PartialEq, ValueEnum, Default)]
pub enum NoSourcePixelBehavior {
    /// Out-of-bounds pixels become fully transparent.
    #[default]
    Transparent,
    /// Out-of-bounds coordinates wrap around using Euclidean modulo, tiling the image.
    Wrap,
}

/// A cardinal direction corresponding to one of the four RSI sprite quadrants.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Direction {
    /// Top-left quadrant.
    North,
    /// Top-right quadrant.
    South,
    /// Bottom-left quadrant.
    East,
    /// Bottom-right quadrant.
    West,
}

impl TryFrom<char> for Direction {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'N' | 'n' => Ok(Self::North),
            'S' | 's' => Ok(Self::South),
            'E' | 'e' => Ok(Self::East),
            'W' | 'w' => Ok(Self::West),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use super::map_coordinates_four_dir;

    #[rstest]
    // North
    #[case((0, 0), (1, 0))]
    #[case((0, 1), (1, 1))]
    #[case((1, 0), (0, 0))]
    #[case((1, 1), (0, 1))]
    // South
    #[case((2, 0), (3, 0))]
    #[case((2, 1), (3, 1))]
    #[case((3, 0), (2, 0))]
    #[case((3, 1), (2, 1))]
    // East
    #[case((0, 2), (3, 2))]
    #[case((0, 3), (3, 3))]
    #[case((1, 2), (2, 2))]
    #[case((1, 3), (2, 3))]
    // West
    #[case((2, 2), (1, 2))]
    #[case((2, 3), (1, 3))]
    #[case((3, 2), (0, 2))]
    #[case((3, 3), (0, 3))]
    fn test_map_coordinates(#[case] (x, y): (i64, i64), #[case] expected: (i64, i64)) {
        let offsets = Default::default();
        assert_eq!(map_coordinates_four_dir(4, 4, x, y, &offsets), expected);
    }

    #[rstest]
    // North
    #[case((0, 0), (1, 1))]
    #[case((0, 1), (1, 2))]
    #[case((1, 0), (0, 1))]
    #[case((1, 1), (0, 2))]
    // South
    #[case((2, 0), (3, -1))]
    #[case((2, 1), (3, 0))]
    #[case((3, 0), (2, -1))]
    #[case((3, 1), (2, 0))]
    // East
    #[case((0, 2), (4, 2))]
    #[case((0, 3), (4, 3))]
    #[case((1, 2), (3, 2))]
    #[case((1, 3), (3, 3))]
    // West
    #[case((2, 2), (0, 2))]
    #[case((2, 3), (0, 3))]
    #[case((3, 2), (-1, 2))]
    #[case((3, 3), (-1, 3))]
    fn test_map_coordinates_with_offsets(#[case] (x, y): (i64, i64), #[case] expected: (i64, i64)) {
        use super::Direction::*;
        let offsets = vec![
            (North, (0, 1)),
            (South, (0, -1)),
            (East, (1, 0)),
            (West, (-1, 0)),
        ]
        .into_iter()
        .collect();
        assert_eq!(map_coordinates_four_dir(4, 4, x, y, &offsets), expected);
    }
}
