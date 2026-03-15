use std::fmt;

/// Write a string with indented lines
pub(crate) fn write_indented(
    f: &mut fmt::Formatter<'_>,
    s: &str,
    num_indent: usize,
) -> fmt::Result {
    for line in s.lines() {
        writeln!(f, "{:>num_indent$}{line}", "", num_indent = num_indent)?;
    }
    Ok(())
}
