use std::path::PathBuf;

use anyhow::{Result, anyhow, bail};
use image::{ImageFormat, image_dimensions, open};

#[derive(clap::Args, Debug)]
/// Overlays a foreground PNG on top of a background PNG of the same size.
pub struct OverlayArgs {
    /// Path to the foreground image.
    #[arg(short, long)]
    foreground: PathBuf,
    /// Path to the background image.
    #[arg(short, long)]
    background: PathBuf,
    /// Output path for the composited image. Defaults to `<background>-<foreground>-overlaid.png`.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

/// Validates that both images share the same dimensions, then alpha-composites the foreground over the background.
pub fn run(args: OverlayArgs) -> Result<()> {
    let (fw, fh) = image_dimensions(&args.foreground).map_err(|e| {
        anyhow!(
            "Failed to read foreground={}: {e}",
            args.foreground.display()
        )
    })?;
    let (bw, bh) = image_dimensions(&args.background).map_err(|e| {
        anyhow!(
            "Failed to read background={}: {e}",
            args.background.display()
        )
    })?;

    if (fw, fh) != (bw, bh) {
        bail!("Images must be the same size: foreground is {fw}x{fh} but background is {bw}x{bh}");
    }

    let fg = open(&args.foreground).map_err(|e| {
        anyhow!(
            "Failed to open foreground={}: {e}",
            args.foreground.display()
        )
    })?;
    let bg = open(&args.background).map_err(|e| {
        anyhow!(
            "Failed to open background={}: {e}",
            args.background.display()
        )
    })?;

    let output = if let Some(p) = args.output {
        p
    } else {
        let bg_stem = args
            .background
            .file_stem()
            .ok_or_else(|| anyhow!("Background file doesn't have a filename"))?
            .to_string_lossy();
        let fg_stem = args
            .foreground
            .file_stem()
            .ok_or_else(|| anyhow!("Foreground file doesn't have a filename"))?
            .to_string_lossy();
        args.background
            .with_file_name(format!("{bg_stem}-{fg_stem}-overlaid.png"))
    };

    let mut bg = bg.into_rgba8();
    image::imageops::overlay(&mut bg, &fg.into_rgba8(), 0, 0);

    bg.save_with_format(&output, ImageFormat::Png)?;
    println!("Saved {}", output.display());

    Ok(())
}
