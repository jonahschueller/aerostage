use crate::{
    aerospace::Aerospace,
    arrangement::{Arrangement, ArrangementWindow},
    restore::resolution::WindowResolution,
};

pub fn restore_arrangement(aerospace: &Aerospace, arrangement: &Arrangement) -> Result<(), String> {
    let windows = aerospace.list_windows()?;

    let resolution = WindowResolution::resolve(arrangement, windows);

    todo!("Properly implement arrangement restore");
    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restore_arrangement_successfully() {}
}
