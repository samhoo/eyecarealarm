//! Audio playback via rodio. One output stream for the app lifetime; playback
//! is serialized through a single sink slot. Fades are driven by a helper
//! thread stepping the sink volume.

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use parking_lot::Mutex;
use rodio::mixer::Mixer;
use rodio::{Decoder, OutputStreamBuilder, Sink, Source};

pub struct Audio {
    mixer: Mixer, // Send+Sync clone of the stream's mixer; the stream itself is leaked
    sink: Mutex<Option<Sink>>,
    /// Generation counter: bumped on every start/stop so a stale fade thread
    /// from a previous playback never touches the current sink.
    generation: AtomicU64,
    /// Per-file loudness cache for gain normalization.
    rms_cache: Mutex<std::collections::HashMap<PathBuf, f32>>,
}

/// Whole-file RMS via the same decoder used for playback. Files are tens of
/// seconds at most; measuring costs a few ms and is cached per path.
fn measure_rms(path: &PathBuf) -> Option<f32> {
    let file = File::open(path).ok()?;
    let dec = Decoder::new(BufReader::new(file)).ok()?;
    let mut sum = 0f64;
    let mut n = 0u64;
    for s in dec {
        sum += (s as f64) * (s as f64);
        n += 1;
    }
    (n > 0).then(|| (sum / n as f64).sqrt() as f32)
}

/// Boost-only normalization: quiet recordings (e.g. a singing bowl that
/// decays into a near-silent tail) are lifted toward a reference loudness so
/// the user volume setting stays meaningful across files. Never attenuates.
fn norm_gain(rms: f32) -> f32 {
    const TARGET_RMS: f32 = 0.12;
    (TARGET_RMS / rms.max(0.005)).clamp(1.0, 4.0)
}

impl Audio {
    pub fn new() -> Option<Self> {
        let stream = OutputStreamBuilder::open_default_stream().ok()?;
        let mixer = stream.mixer().clone();
        // OutputStream is !Send on macOS (cpal CoreAudio property-listener
        // callback), so it cannot live in AppState. Leak it to keep the device
        // open for the app lifetime and keep only the Send+Sync Mixer here.
        std::mem::forget(stream);
        Some(Self {
            mixer,
            sink: Mutex::new(None),
            generation: AtomicU64::new(0),
            rms_cache: Default::default(),
        })
    }

    /// Play `path` for up to `play_sec` seconds at `volume_pct`, with a
    /// `fade_sec` linear fade-out at the end. Short files loop to fill the
    /// window. Any previous playback is stopped first.
    pub fn play(self: &Arc<Self>, path: PathBuf, volume_pct: u32, play_sec: u64, fade_sec: u64) {
        self.stop();
        let gen = self.generation.fetch_add(1, Ordering::SeqCst) + 1;

        let rms = *self
            .rms_cache
            .lock()
            .entry(path.clone())
            .or_insert_with(|| measure_rms(&path).unwrap_or(0.12));
        let gain = norm_gain(rms);
        let base_vol = volume_pct as f32 / 100.0 * gain;
        eprintln!(
            "[audio] play {:?} vol={volume_pct}% rms={rms:.3} gain={gain:.2} play={play_sec}s fade={fade_sec}s",
            path
        );
        let Ok(file) = File::open(&path) else {
            eprintln!("[audio] cannot open {:?}", path);
            return;
        };
        let Ok(source) = Decoder::new(BufReader::new(file)) else {
            eprintln!("[audio] cannot decode {:?}", path);
            return;
        };
        let sink = Sink::connect_new(&self.mixer);
        sink.set_volume(base_vol);
        sink.append(source.repeat_infinite());
        sink.play();
        *self.sink.lock() = Some(sink);

        // Fade thread: sleeps until fade start, then steps volume to 0.
        let fade_start = play_sec.saturating_sub(fade_sec);
        let steps = (fade_sec * 10).max(1) as u32; // 100ms per step
        let this = Arc::clone(self);
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(fade_start));
            for i in 1..=steps {
                thread::sleep(Duration::from_millis(100));
                if this.generation.load(Ordering::SeqCst) != gen {
                    return; // superseded
                }
                let guard = this.sink.lock();
                let Some(sink) = guard.as_ref() else { return };
                let remaining = 1.0 - i as f32 / steps as f32;
                sink.set_volume(base_vol * remaining);
                if i == steps {
                    sink.stop();
                }
            }
        });
    }

    pub fn stop(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        if let Some(sink) = self.sink.lock().take() {
            sink.stop();
        }
    }
}
