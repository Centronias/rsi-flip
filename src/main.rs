#![warn(
    clippy::all,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    clippy::nursery
)]
#![allow(
    // Pedantic/nursery opt-outs for stylistic preferences.
    clippy::missing_docs_in_private_items,
    clippy::missing_panics_doc,
    clippy::pattern_type_mismatch,
    clippy::wildcard_enum_match_arm,
    clippy::implicit_return,
    clippy::question_mark_used,
    clippy::shadow_unrelated,
    clippy::shadow_reuse,
    clippy::shadow_same,
    clippy::too_many_lines,
    clippy::enum_glob_use
)]

use std::path::PathBuf;

use anyhow::{Result, anyhow, bail};
use clap::{Parser, ValueEnum};
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageFormat, Rgba, open};

use crate::{
    exts::i64Ext,
    offsets::{ArgsExt, Offsets, OffsetsExt},
};

mod exts;
mod offsets;

fn main() -> Result<()> {
    let args = Args::parse();

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

    let new_path = args.path.with_file_name(format!(
        "{}-flipped.{}",
        args.path
            .file_stem()
            .ok_or_else(|| anyhow!("Given file doesn't have a filename"))?
            .to_string_lossy(),
        "png",
    ));
    flipped_image.save_with_format(&new_path, ImageFormat::Png)?;
    println!("Saved {}", new_path.display());

    Ok(())
}

/// Retrieves a pixel from [image] to place at ([x], [y]) as determined by [mapper].
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

/// Simply flips the sprite horizontally.
const fn map_coordinates_one_dir(width: i64, _h: i64, x: i64, y: i64, _: &Offsets) -> (i64, i64) {
    (width - x - 1, y)
}

/// Flips each individual sprite horizontally as well as swaps the positions of the east and west
/// sprites. Additionally applies [offsets] as applicable.
fn map_coordinates_four_dir(
    width: i64,
    height: i64,
    x: i64,
    y: i64,
    offsets: &Offsets,
) -> (i64, i64) {
    use crate::Direction::*;

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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// Horizontally flips the directions in RSIs.
/// See more at <https://github.com/Centronias/rsi-flip>
struct Args {
    /// The path of the image to flip.
    #[arg(short, long)]
    path: PathBuf,
    /// The number of direction in this RSI state. If one, flips only horizontally. If four, flips
    /// each as appropriate for its direction. Any other value is an error.
    // TODO Maybe make this into an enum, arbitrary values don't make sense.
    #[arg(short, long, default_value_t = 4u8)]
    directions: u8,
    /// A list of directions and offsets to apply to sprites in those directions. Values of this
    /// argument must be of the form `nsew=123,456`, that is a list of directions (as single
    /// characters) and the x and y offsets.
    /// Has no effect when `directions` is 1.
    #[arg(short, long)]
    offsets: Option<Vec<String>>,
    /// What to do when a pixel's mapped location is outside of the original sprite. This usually happens when an offset is given.
    #[arg(short, long, value_enum, default_value_t = NoSourcePixelBehavior::Transparent)]
    no_source_pixel_behavior: NoSourcePixelBehavior,
    // TODO Output path
    // TODO Specify dimensions for non-square images
    // TODO Maybe 8 direction sprites? I've never seen one, though
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, ValueEnum, Default)]
enum NoSourcePixelBehavior {
    #[default]
    Transparent,
    Wrap,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum Direction {
    North,
    South,
    East,
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

    use crate::map_coordinates_four_dir;

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
        use crate::Direction::*;
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
