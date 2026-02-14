use rodio::buffer::SamplesBuffer;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::{Duration, Instant};

pub struct AudioManager {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Sink,
    // --- 新增：音效缓存 ---
    hit_data: Option<(Vec<f32>, u16, u32)>,

    start_instant: Option<Instant>,
    accumulated_time: Duration,
    is_playing: bool,
}
impl AudioManager {
    pub fn new() -> Self {
        // 恢复最简单的初始化，保证编译通过
        let (stream, handle) = OutputStream::try_default().expect("无法打开音频输出设备");
        let sink = Sink::try_new(&handle).expect("无法创建音频 Sink");

        let hit_data = Self::load_hit_file("assets/sounds/hit.wav")
            .unwrap_or_else(|| Self::generate_beep());

        Self {
            _stream: stream,
            handle,
            sink,
            hit_data: Some(hit_data),
            start_instant: None,
            accumulated_time: Duration::ZERO,
            is_playing: false,
        }
    }


    // 辅助函数：从文件加载
    fn load_hit_file(path: &str) -> Option<(Vec<f32>, u16, u32)> {
        File::open(path).ok().and_then(|file| {
            let decoder = Decoder::new(BufReader::new(file)).ok()?;
            let channels = decoder.channels();
            let sample_rate = decoder.sample_rate();
            let samples = decoder.convert_samples::<f32>().collect();
            Some((samples, channels, sample_rate))
        })
    }

    // 🚩 核心逻辑：生成一个 100ms 的电子打击音
    fn generate_beep() -> (Vec<f32>, u16, u32) {
        let sample_rate = 44100;
        let duration_ms = 100;
        let num_samples = (sample_rate * duration_ms / 1000) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        let frequency = 880.0; // A5 调，比较清脆

        for i in 0..num_samples {
            let t = i as f32 / sample_rate as f32;
            // 基础正弦波
            let mut s = (t * frequency * 2.0 * std::f32::consts::PI).sin();

            // 指数级振幅衰减 (让声音从响到静，产生打击感)
            let envelope = (-15.0 * t).exp();
            s *= envelope;

            samples.push(s);
        }

        (samples, 1, sample_rate)
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


    pub fn play_hit_effect(&self) {
        if let Some((samples, channels, rate)) = &self.hit_data {
            // 直接从内存构建 buffer，省去每一击的解码开销
            let source = SamplesBuffer::new(*channels, *rate, samples.clone());
            let _ = self.handle.play_raw(source.convert_samples());
        }
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