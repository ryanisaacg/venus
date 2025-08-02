use std::{io::Cursor, sync::Arc};

use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use slotmap::SlotMap;

slotmap::new_key_type! {
    pub struct PlayingAudio;
}

pub struct AudioPlayer {
    output: OutputStream,
    slotmap: SlotMap<PlayingAudio, Sink>,
}

impl AudioPlayer {
    pub fn new() -> AudioPlayer {
        let output = OutputStreamBuilder::open_default_stream().expect("open default audio stream");

        AudioPlayer {
            output,
            slotmap: SlotMap::with_key(),
        }
    }

    pub fn start(&mut self, source: impl Source + Send + 'static) -> PlayingAudio {
        let sink = Sink::connect_new(self.output.mixer());
        sink.append(source);
        self.slotmap.insert(sink)
    }

    pub fn pause(&self, audio: PlayingAudio) {
        let Some(sink) = self.slotmap.get(audio) else {
            return;
        };
        sink.pause();
    }

    pub fn play(&self, audio: PlayingAudio) {
        let Some(sink) = self.slotmap.get(audio) else {
            return;
        };
        sink.play();
    }

    pub fn stop(&self, audio: PlayingAudio) {
        let Some(sink) = self.slotmap.get(audio) else {
            return;
        };
        sink.stop();
    }

    pub fn gc(&mut self) {
        self.slotmap.retain(|_, sink| !sink.empty());
    }
}

#[derive(Clone)]
pub struct Audio {
    contents: Arc<[u8]>,
}

impl Audio {
    pub fn new(contents: Arc<[u8]>) -> Result<Audio, rodio::decoder::DecoderError> {
        let audio = Audio { contents };
        audio.source()?;
        Ok(audio)
    }

    pub(crate) fn source(
        &self,
    ) -> Result<Decoder<Cursor<Arc<[u8]>>>, rodio::decoder::DecoderError> {
        Decoder::new(Cursor::new(self.contents.clone()))
    }
}
