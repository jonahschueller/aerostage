use crate::{
    aerospace::Aerospace,
    arrangement::{Arrangement, ArrangementWindow},
    restore::resolution::WindowResolution,
};

pub fn restore_arrangement(aerospace: &Aerospace, arrangement: &Arrangement) -> Result<(), String> {
    let windows = dbg!(aerospace.list_windows()?);

    let resolution = dbg!(WindowResolution::resolve(arrangement, windows));
    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restore_arrangement_successfully() {}
}
