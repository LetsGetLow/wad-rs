use crate::audio::{MidiSynthesizer, MusicSample, SoundSample};
use crate::error::WadError;
use crate::header::Header;
use crate::index::{index_tokens, LumpNode};
use crate::sprite::Sprite;
use crate::tokenizer::TokenIterator;
use std::collections::HashMap;

type Error = WadError;
type Result<T> = std::result::Result<T, Error>;


/// WadIndex represents an indexed WAD file, containing its header information and a hierarchical
/// index of lumps organized in namespaces. With no zero allocation for lump data access it provides
/// efficient retrieval of lumps by name and namespace.
///
/// Examples
/// ```no_run
/// use wad_rs::WadIndex;
///
/// let wad_data: &[u8] = include_bytes!("path/to/wadfile.wad");
/// let mut synthesizer = MidiSynthesizer::new(include_bytes!("../assets/microgm.sf2"), 16_000).unwrap();
/// let wad = WadIndex::from_bytes("wadfile.wad".to_string(), wad_data, &mut synthesizer).unwrap();
///
/// let header = wad.get_header();
/// println!("WAD Type: {}", header.get_wad_type());
///
/// let lump_index = wad.get_lump_index();
/// for (name, lump_node) in lump_index {
///     println!("Lump Name: {}", name);
/// }
/// let sound_sample = wad.get_sound_sample("DSPISTOL").unwrap();
/// if let Some(sample) = sound_sample {
///     println!("Sound Sample Size: {} bytes", sample.sample().len() * std::mem::size_of::<f32>());
/// }
/// let music_sample = wad.get_music_sample("D_E1M1").unwrap();
/// if let Some(sample) = music_sample {
///     println!("Music Sample Size: {} bytes", sample.sample().len() * std::mem::size_of::<f32>());
/// }
/// let sprite = wad.get_sprite("PLAYA1").unwrap();
/// if let Ok(sprite) = sprite {
///     println!("Sprite Width: {}, Height: {}", sprite.width(), sprite.height());
/// }
/// ```
pub struct WadIndex<'a, 'b> {
    header: Header,
    name: String,
    lump_index: HashMap<&'a str, LumpNode<'a>>,
    synthesizer: &'b mut MidiSynthesizer,
}

impl<'a, 'b> WadIndex<'a, 'b> {
    const HEADER_SIZE: usize = 12;

    /// Create a WadIndex from raw WAD file bytes
    /// # Errors
    /// Returns an WadError if the data is too small to contain a valid WAD header
    pub fn from_bytes(name: String, data: &'a [u8], synthesizer: &'b mut MidiSynthesizer) -> Result<Self> {
        let size = data.len();
        if size < Self::HEADER_SIZE {
            return Err(WadError::HeaderDataTooSmall);
        }
        let header_bytes: &[u8; 12] = data
            .first_chunk::<{ WadIndex::HEADER_SIZE }>()
            .ok_or(WadError::HeaderDataTooSmall)?;
        let header = Header::try_from(header_bytes)?;
        let lump_index = index_tokens(TokenIterator::new(header, data)?)?;

        let wad_index = WadIndex {
            header,
            name,
            lump_index,
            synthesizer,
        };

        Ok(wad_index)
    }

    /// Get the WAD header
    pub fn get_header(&self) -> &Header {
        &self.header
    }

    /// Get the full lump index
    pub fn get_lump_index(&self) -> &HashMap<&'a str, LumpNode<'a>> {
        &self.lump_index
    }

    /// Get a raw lump by its namespaces and name
    pub fn get_lump(&'_ self, namespaces: Vec<&str>, name: &str) -> Option<&LumpNode<'a>> {
        let mut current_index = &self.lump_index;
        for namespace in namespaces {
            if let Some(LumpNode::Namespace { children, .. }) = current_index.get(namespace) {
                current_index = children;
            } else {
                return None;
            }
        }
        current_index.get(name)
    }

    /// Get a sound sample by its lump name
    pub fn get_sound_sample(&self, name: &str) -> Result<Option<SoundSample>> {
        if let Some(lump_node) = self.lump_index.get(name)
            && let LumpNode::Lump { lump, .. } = lump_node
        {
            let lump_data = lump.data();
            Ok(Some(SoundSample::try_from(lump_data)?))
        } else {
            Ok(None)
        }
    }

    /// Get a music sample by its lump name
    pub fn get_music_sample(&mut self, name: &str) -> Result<Option<MusicSample>> {
        if let Some(lump_node) = self.lump_index.get(name)
            && let LumpNode::Lump { lump, .. } = lump_node
        {
            let music_sample = MusicSample::from_bytes(self.synthesizer, lump.data(), true)?;
            Ok(Some(music_sample))
        } else {
            Ok(None)
        }
    }

    /// Get a sprite/patch by its name
    pub fn get_sprite(&'a self, name: &str) -> Result<Option<Sprite<'a>>> {
        if let Some(LumpNode::Namespace { children, .. }) = self.lump_index.get("S_START")
            && let Some(LumpNode::Lump { lump, .. }) = children.get(name)
        {
            Sprite::new(lump.data()).map(Some)
        } else {
            Ok(None)
        }
    }

    /// Get the name of the index
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get all game maps in the WAD with their map lump nodes
    pub fn get_maps(&self) -> Option<&HashMap<&'a str, LumpNode<'a>>> {
        if let Some(LumpNode::Namespace { children, .. }) = self.lump_index.get("MAPS") {
            Some(children)
        } else {
            None
        }
    }
}
