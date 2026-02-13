use rodio::{Decoder, OutputStream, Sink, OutputStreamHandle};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::{Duration, Instant};

pub struct AudioManager {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Sink,
    // --- 新增追踪字段 ---
    start_instant: Option<Instant>,
    accumulated_time: Duration,
    is_playing: bool,
}

impl AudioManager {
    pub fn new() -> Self {
        let (stream, handle) = OutputStream::try_default().expect("无法打开音频输出设备");
        let sink = Sink::try_new(&handle).expect("无法创建音频 Sink");

        Self {
            _stream: stream,
            handle,
            sink,
            start_instant: None,
            accumulated_time: Duration::ZERO,
            is_playing: false,
        }
    }

    pub fn play_music<T>(&mut self, path: T) -> anyhow::Result<()>
    where T: AsRef<Path>
    {
        let file = BufReader::new(File::open(path)?);
        let source = Decoder::new(file)?;

        self.sink.stop();
        self.sink.append(source);

        // 重置计时器
        self.start_instant = Some(Instant::now());
        self.accumulated_time = Duration::ZERO;
        self.is_playing = true;

        self.sink.play();
        Ok(())
    }

    pub fn pause(&mut self) {
        if self.is_playing {
            if let Some(start) = self.start_instant {
                self.accumulated_time += start.elapsed();
            }
            self.start_instant = None;
            self.is_playing = false;
            self.sink.pause();
        }
    }

    pub fn resume(&mut self) {
        if !self.is_playing {
            self.start_instant = Some(Instant::now());
            self.is_playing = true;
            self.sink.play();
        }
    }

    pub fn get_pos(&self) -> Duration {
        if !self.is_playing {
            return self.accumulated_time;
        }

        match self.start_instant {
            Some(start) => self.accumulated_time + start.elapsed(),
            None => self.accumulated_time,
        }
    }

    pub fn stop(&mut self) {
        self.sink.stop();
        self.start_instant = None;
        self.accumulated_time = Duration::ZERO;
        self.is_playing = false;
    }

    pub fn is_finished(&self) -> bool {
        self.sink.empty()
    }
}