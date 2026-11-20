//! `Bases/` folder-name conventions: a trailing ` [UL]`, ` [DB]` or
//! ` [<item>]` binds a folder to the Uploads, Drop Box, or a named Folder
//! Access item. See `docs/formats/bases.md`.

#[derive(Debug, Clone, PartialEq, Eq)]
/// What a folder name's trailing bracketed tag binds it to.
pub enum BaseTag {
    /// ` [UL]`: the Uploads folder.
    Uploads,
    /// ` [DB]`: the Drop Box.
    DropBox,
    /// ` [<item>]`: the Folder Access item of that name.
    Item(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A folder name under `Bases/`, split into what the user sees and its tag.
pub struct BaseName {
    /// The name without the tag.
    pub display: Vec<u8>,
    /// The tag, if the name ends in one.
    pub tag: Option<BaseTag>,
}

/// Splits a folder name into display name and tag.
pub fn parse_base_name(name: &[u8]) -> BaseName {
    let tagged = name.strip_suffix(b"]").and_then(|stripped| {
        let open = stripped.iter().rposition(|&b| b == b'[')?;
        (open >= 2 && stripped[open - 1] == b' ').then_some((stripped, open))
    });
    if let Some((stripped, open)) = tagged {
        let display = &stripped[..open - 1];
        let item = &stripped[open + 1..];
        if !item.is_empty() && !display.is_empty() {
            let tag = match item {
                b"UL" => BaseTag::Uploads,
                b"DB" => BaseTag::DropBox,
                other => BaseTag::Item(other.to_vec()),
            };
            return BaseName {
                display: display.to_vec(),
                tag: Some(tag),
            };
        }
    }
    BaseName {
        display: name.to_vec(),
        tag: None,
    }
}

/// The folder name for a display name and tag.
pub fn format_base_name(name: &BaseName) -> Vec<u8> {
    let mut out = name.display.clone();
    match &name.tag {
        None => {}
        Some(BaseTag::Uploads) => out.extend_from_slice(b" [UL]"),
        Some(BaseTag::DropBox) => out.extend_from_slice(b" [DB]"),
        Some(BaseTag::Item(item)) => {
            out.extend_from_slice(b" [");
            out.extend_from_slice(item);
            out.push(b']');
        }
    }
    out
}
