//! Provides an interface to output sound to the user's speakers.
//!
//! This departs from the LÖVE API a bit because `SDL2_mixer` is opinionated
//! about the difference between samples and music files, and also makes channel
//! management and such more explicit.
//! This seems a bit awkward but we'll roll with it for now.

use std::fmt;
use std::io;
use std::io::Read;
use std::path;

use std::sync::Arc;

use rodio;

use context::Context;
use GameError;
use GameResult;

/// An object representing a channel that may be playing a particular Sound.
pub type Channel = rodio::Sink;


/// A struct that contains all information for tracking sound info.
pub struct AudioContext {
    endpoint: rodio::Endpoint,
}

impl AudioContext {
    pub fn new() -> GameResult<AudioContext> {
        let error = GameError::AudioError(String::from("Could not initialize sound system (for \
                                                        some reason)"));
        let e = rodio::get_default_endpoint().ok_or(error)?;
        Ok(AudioContext { endpoint: e })
    }
}

/// Static sound data stored in memory.
/// It is Arc'ed, so cheap to clone.
#[derive(Clone)]
pub struct SoundData(Arc<Vec<u8>>);

impl SoundData {
    /// Copies the data in the given slice into a new SoundData object.
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut buffer = Vec::with_capacity(data.len());
        buffer.extend(data);
        SoundData::from(buffer)

    }

    pub fn from_read<R>(reader: &mut R) -> GameResult<Self> where R: Read {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        Ok(SoundData::from(buffer))

    }
}

impl From<Vec<u8>> for SoundData {
    fn from(v: Vec<u8>) -> Self {
        SoundData(Arc::new(v))
    }
}

impl AsRef<[u8]> for SoundData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref().as_ref()
    }
}

/// A source of audio data.
//
// TODO: Will stop when dropped.  Check and see if this matches Love2d's semantics!
// Eventually it might read from a streaming decoder of some kind,
// but for now it is just an in-memory SoundData structure.
// The source of a rodio decoder must be Send, which something
// that contains a reference to a ZipFile is not, so we are going
// to just slurp all the data into memory for now.
pub struct Source {
    data: SoundData,
    sink: rodio::Sink,
}

impl Source {
    /// Create a new Source from the given file.
    pub fn new<P: AsRef<path::Path>>(context: &mut Context, path: P) -> GameResult<Self> {
        let path = path.as_ref();
        let data = {
        let file = &mut context.filesystem.open(path)?;
            let data = SoundData::from_read(file)?;
            data
        };
        Source::from_data(context, data)
    }

    /// Creates a new Source using the given SoundData object.
    pub fn from_data(context: &mut Context, data: SoundData) -> GameResult<Self> {
        let sink = rodio::Sink::new(&context.audio_context.endpoint);
        Ok(Source {
            data: data,
            sink: sink,
        })
    }

    /// Plays the Source.
    pub fn play(&self) -> GameResult<()> {
        // Creating a new Decoder each time seems a little messy,
        // since it may do checking and data-type detection that is
        // redundant, but it's fine for now.
        let cursor = io::Cursor::new(self.data.clone());
        let decoder = rodio::Decoder::new(cursor)?;
        self.sink.append(decoder);
        Ok(())
    }

    pub fn pause(&self) {
        self.sink.pause()
    }
    pub fn resume(&self) {
        self.sink.play()
    }
    

    pub fn stop(&self) {
    }
    pub fn set_looping() {}
    pub fn set_volume(&mut self, value: f32) {
        self.sink.set_volume(value)
    }

    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }
    pub fn stopped(&self) -> bool {
        false
    }
    pub fn paused(&self) -> bool {
        self.sink.is_paused()
    }
    pub fn playing(&self) -> bool {
        !self.paused() && !self.stopped()
    }
    pub fn looping(&self) -> bool {
        false
    }
    
    // TODO: maybe seek(), tell(), rewind()?
}


impl fmt::Debug for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Audio source: {:p}>", self)
    }
}


// TODO: global start, stop, volume?
