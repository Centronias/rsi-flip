use anyhow::{anyhow, bail};
use clap::Parser;
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageFormat, Rgba, open};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.directions != 1 && args.directions != 4 {
        bail!("Directions must be 1 or 4");
    }
    let image =
        open(args.path.as_path()).map_err(|_| anyhow!("Failed to open file={:?}", args.path))?;

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
            match args.directions {
                1 => map_coordinates_one_dir,
                4 => map_coordinates_four_dir,
                _ => panic!("Unreachable: we should've validated this value already"),
            },
        )
    });

    let new_path = args.path.with_file_name(format!(
        "{}-flipped.{}",
        args.path
            .file_stem()
            .ok_or(anyhow!("Given file doesn't have a filename"))?
            .to_string_lossy(),
        ImageFormat::Png
            .extensions_str()
            .first()
            .expect("No image extensions")
    ));
    flipped_image.save_with_format(&new_path, ImageFormat::Png)?;
    println!("Saved {}", new_path.to_string_lossy());

    Ok(())
}

/// The north and south quadrants are flipped across a vertical axis. The east and west quadrants
/// are flipped across a vertical axis and are transposed, meaning the east becomes west and west
/// becomes east.
fn map_pixel<F>(image: &DynamicImage, x: u32, y: u32, mapper: F) -> Rgba<u8>
where
    F: Fn(u32, u32, u32, u32) -> (u32, u32),
{
    let dimensions = image.dimensions();
    let (x, y) = mapper(dimensions.0, dimensions.1, x, y);
    image.get_pixel(x, y)
}

fn map_coordinates_one_dir(width: u32, _h: u32, x: u32, y: u32) -> (u32, u32) {
    (width - x - 1, y)
}

fn map_coordinates_four_dir(width: u32, height: u32, x: u32, y: u32) -> (u32, u32) {
    let hw = width / 2;
    let hh = height / 2;

    let x = if x < hw && y < hh {
        // North
        hw - (x + 1)
    } else if x >= hw && y < hh {
        // South
        width - ((x - hw) + 1)
    } else if x < hw && y >= hh {
        // East
        width - (x + 1)
    } else {
        // West
        width - (x + 1)
    };

    (x, y)
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// Horizontally flips the directions in a four-direction RSI.
/// See more at https://github.com/Centronias/rsi-flip
struct Args {
    /// The path of the image to flip.
    #[arg(short, long)]
    path: PathBuf,
    /// The number of direction in this RSI state. If one, flips only horizontally. If four, flips each as appropriate for its direction. Any other value is an error.
    #[arg(short, long, default_value_t = 4u32)]
    directions: u32,
    // TODO Output path
    // TODO Specify dimensions for non-square images
    // TODO Maybe 8 direction sprites? I've never seen one, though
}

#[cfg(test)]
mod test {
    use crate::map_coordinates_four_dir;

    #[test]
    fn test_map_coordinates() {
        assert_eq!(map_coordinates_four_dir(4, 4, 0, 0), (1, 0));
        assert_eq!(map_coordinates_four_dir(4, 4, 0, 1), (1, 1));
        assert_eq!(map_coordinates_four_dir(4, 4, 2, 0), (3, 0));
        assert_eq!(map_coordinates_four_dir(4, 4, 3, 0), (2, 0));

        assert_eq!(map_coordinates_four_dir(4, 4, 0, 2), (3, 2));
        assert_eq!(map_coordinates_four_dir(4, 4, 0, 3), (3, 3));
        assert_eq!(map_coordinates_four_dir(4, 4, 1, 3), (2, 3));
    }
}
