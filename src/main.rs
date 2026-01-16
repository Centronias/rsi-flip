use anyhow::{anyhow, bail};
use clap::Parser;
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageFormat, Rgba, open};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let image_path = Args::parse().path;

    let image =
        open(image_path.as_path()).map_err(|_| anyhow!("Failed to open file={image_path:?}"))?;

    let (width, height) = image.dimensions();
    if width != height {
        bail!("Image must be square");
    }
    if width % 2 != 0 {
        bail!("Image must be composed of four quadrants");
    }
    let flipped_image = ImageBuffer::from_par_fn(width, height, |x, y| map_pixel(&image, x, y));

    let new_path = image_path.with_file_name(format!(
        "{}-flipped.{}",
        image_path
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
fn map_pixel(image: &DynamicImage, x: u32, y: u32) -> Rgba<u8> {
    let dimensions = image.dimensions();
    let (x, y) = map_coordinates(dimensions.0, dimensions.1, x, y);
    image.get_pixel(x, y)
}

fn map_coordinates(width: u32, height: u32, x: u32, y: u32) -> (u32, u32) {
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
struct Args {
    #[arg(short, long)]
    path: PathBuf,
}

#[cfg(test)]
mod test {
    use crate::map_coordinates;

    #[test]
    fn test_map_coordinates() {
        assert_eq!(map_coordinates(4, 4, 0, 0), (1, 0));
        assert_eq!(map_coordinates(4, 4, 0, 1), (1, 1));
        assert_eq!(map_coordinates(4, 4, 2, 0), (3, 0));
        assert_eq!(map_coordinates(4, 4, 3, 0), (2, 0));

        assert_eq!(map_coordinates(4, 4, 0, 2), (3, 2));
        assert_eq!(map_coordinates(4, 4, 0, 3), (3, 3));
        assert_eq!(map_coordinates(4, 4, 1, 3), (2, 3));
    }
}
