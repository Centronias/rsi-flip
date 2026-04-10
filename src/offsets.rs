use std::collections::HashMap;

use anyhow::{anyhow, bail};

use crate::{Args, Direction};

pub type Offset = (i64, i64);
pub type Offsets = HashMap<Direction, Offset>;

#[extend::ext]
pub impl Offsets {
    fn get_dir(&self, key: Direction) -> Offset {
        self.get(&key).copied().unwrap_or_default()
    }
}

#[extend::ext]
pub impl Args {
    fn get_offsets(&self) -> anyhow::Result<Offsets> {
        let Some(offsets) = self.offsets.as_ref() else {
            return Ok(Offsets::default());
        };

        get_offsets(offsets)
    }
}

fn get_offsets(offsets: &[String]) -> anyhow::Result<Offsets> {
    {
        let directions_and_offsets: Vec<(Vec<Direction>, Offset)> = offsets
            .iter()
            .map(|s| {
                let &[directions, offset] = &s.split('=').collect::<Vec<_>>()[..] else {
                    bail!("Bad offset \"{s}\": expected two values separated by an equals sign")
                };

                let directions = directions_string_to_directions(directions)?;
                let offset = offsets_string_to_offset(offset)?;
                Ok((directions, offset))
            })
            .collect::<anyhow::Result<Vec<(Vec<_>, Offset)>>>()?;
        Ok(directions_and_offsets
            .into_iter()
            .flat_map(|(directions, offset)| {
                directions
                    .into_iter()
                    .map(move |direction| (direction, offset))
            })
            .collect())
    }
}

/// Parses `"nw"` to `vec![North, West]`.
fn directions_string_to_directions(s: &str) -> anyhow::Result<Vec<Direction>> {
    s.chars()
        .map(|d| Direction::try_from(d).map_err(|()| anyhow!("Invalid direction: {d}")))
        .collect::<anyhow::Result<Vec<_>, _>>()
}

/// Parses `"123,456"` to `(123, 456)`.
fn offsets_string_to_offset(s: &str) -> anyhow::Result<Offset> {
    let &[x, y] = &s.split(',').collect::<Vec<_>>()[..] else {
        bail!("Bad offset \"{s}\": expected two numbers separated by a comma")
    };

    let x = x
        .parse::<i64>()
        .map_err(|e| anyhow!("Bad offset \"{s}\": {e}"))?;
    let y = y
        .parse::<i64>()
        .map_err(|e| anyhow!("Bad offset \"{s}\": {e}"))?;

    Ok((x, y))
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use super::{directions_string_to_directions, get_offsets, offsets_string_to_offset};
    use crate::Direction::{self, *};

    #[rstest]
    #[case("n", vec![North])]
    #[case("s", vec![South])]
    #[case("e", vec![East])]
    #[case("w", vec![West])]
    #[case("N", vec![North])]
    #[case("S", vec![South])]
    #[case("E", vec![East])]
    #[case("W", vec![West])]
    #[case("nsew", vec![North, South, East, West])]
    #[case("nw", vec![North, West])]
    fn test_directions_string_valid(#[case] input: &str, #[case] expected: Vec<Direction>) {
        assert_eq!(directions_string_to_directions(input).unwrap(), expected);
    }

    #[rstest]
    #[case("x")]
    #[case("nq")]
    #[case("1")]
    fn test_directions_string_invalid(#[case] input: &str) {
        assert!(directions_string_to_directions(input).is_err());
    }

    #[rstest]
    #[case("10,20", (10, 20))]
    #[case("-10,-20", (-10, -20))]
    #[case("0,0", (0, 0))]
    #[case("1,0", (1, 0))]
    #[case("0,-99", (0, -99))]
    fn test_offsets_string_valid(#[case] input: &str, #[case] expected: (i64, i64)) {
        assert_eq!(offsets_string_to_offset(input).unwrap(), expected);
    }

    #[rstest]
    #[case("1020")] // missing comma
    #[case("a,b")] // non-numeric
    #[case("1,2,3")] // too many parts
    #[case("1,")] // missing second number
    #[case(",2")] // missing first number
    fn test_offsets_string_invalid(#[case] input: &str) {
        assert!(offsets_string_to_offset(input).is_err());
    }

    // --- get_offsets ---

    #[test]
    fn test_get_offsets_single() {
        let result = get_offsets(&["n=10,20".to_owned()]).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[&North], (10, 20));
    }

    #[test]
    fn test_get_offsets_multiple_args() {
        let result = get_offsets(&["n=10,20".to_owned(), "s=30,40".to_owned()]).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[&North], (10, 20));
        assert_eq!(result[&South], (30, 40));
    }

    #[test]
    fn test_get_offsets_multiple_dirs_one_arg() {
        let result = get_offsets(&["nsew=5,-3".to_owned()]).unwrap();
        assert_eq!(result.len(), 4);
        assert!(result.values().all(|&v| v == (5, -3)));
    }

    #[test]
    fn test_get_offsets_empty() {
        let result = get_offsets(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[rstest]
    #[case("n10,20")] // missing equals sign
    #[case("x=10,20")] // invalid direction
    #[case("n=abc")] // non-numeric offset
    #[case("n=1,2,3")] // too many offset parts
    fn test_get_offsets_invalid(#[case] input: &str) {
        assert!(get_offsets(&[input.to_owned()]).is_err());
    }
}
