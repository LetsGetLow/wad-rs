use thiserror::Error;

#[derive(Debug, Error)]
pub enum WADError {
    #[error("General error: {0}")]
    GeneralError(#[from] Box<dyn std::error::Error>),
    #[error("Invalid WAD header identification")]
    InvalidHeaderIdentification,
    #[error("Data too small to contain valid WAD header")]
    HeaderDataTooSmall,
    #[error("Data too small to contain valid lump directory")]
    TokenDataTooSmall,
    #[error("Unknown marker type encountered")]
    UnknownMarkerType,
    #[error("Unexpected end of tokens while indexing")]
    UnexpectedEndOfTokens,
    #[error("Marker end found without matching start: {name}")]
    EndMarkerWithoutStart { name:String },
    #[error("Dangling end marker: expected '{expected}', found '{found}'")]
    DanglingEndMarker { expected:String, found:String },
    #[error("Dangling start marker: no matching end marker for '{name}'")]
    DanglingStartMarker { name:String },
    #[error("Invalid start marker name: {name}")]
    InvalidStartMarkerName { name:String },
    #[error("Invalid end marker name: {name}")]
    InvalidEndMarkerName { name:String },
    #[error("Token type not allowed in namespace: {name}")]
    TokenTypeNotAllowedInNamespace { name:String },
    #[error("Palette data too short")]
    PaletteDataTooShort,
    #[error("Palette data malformed")]
    PaletteDataMalformed,
    #[error("Color map data too short")]
    ColorMapDataTooShort,
    #[error("Color map data malformed")]
    ColorMapDataMalformed,
    #[error("Color index out of bounds: {index}")]
    ColorIndexOutOfBounds { index:usize },
}