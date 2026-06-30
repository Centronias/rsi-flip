# rsi-flip
CLI tools for working with [RSI](https://docs.spacestation14.com/en/specifications/robust-station-image.html) sprite sheets.

## Installation

```sh
cargo install --git https://github.com/Centronias/rsi-flip
```

Or build from source:

```sh
git clone https://github.com/Centronias/rsi-flip
cd rsi-flip
cargo build --release
```

## Commands

### `flip`

Horizontally mirrors an RSI sprite sheet. In 4-direction mode (the default), each directional sprite is mirrored and the east and west sprites swap quadrant positions. In 1-direction mode the entire image is simply flipped horizontally.

For example, this image

![](./demo/flip/input.png)

becomes this with 4 directions (the default):

![](./demo/flip/output-4-dir.png)

or this with 1 direction:

![](./demo/flip/output-1-dir.png)

```sh
rsi-flip flip -p sprite.png
rsi-flip flip -p sprite.png -d 1
rsi-flip flip -p sprite.png -o out.png --offsets n=0,2
```

Run `rsi-flip flip --help` for the full list of options.

### `overlay`

Alpha-composites a foreground PNG on top of a background PNG of the same dimensions, producing a single merged image.

For example, using this image as the background

![](./demo/overlay/background.png)

and this image as the foreground

![](./demo/overlay/foreground.png)

produces this result:

![](./demo/overlay/output.png)

```sh
rsi-flip overlay -b background.png -f foreground.png
rsi-flip overlay -b background.png -f foreground.png -o out.png
```

Run `rsi-flip overlay --help` for the full list of options.

## Contact

I'll probably notice issues or whatever you create here, but feel free to beam me a message on Discord `@Centronias`.
