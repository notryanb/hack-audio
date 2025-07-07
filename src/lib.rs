use nih_plug::prelude::*;
use nih_plug_egui::{
    create_egui_editor,
    egui::{Vec2},
    resizable_window::ResizableWindow,
    widgets, EguiState,
};
use std::sync::Arc;

/// This is mostly identical to the gain example, minus some fluff, and with a GUI.
pub struct HackAudio {
    params: Arc<PluginParams>,

    // Needed to normalize the peak meter's response based on the sample rate.
    // peak_meter_decay_weight: f32,
    //  The current data for the peak meter. This is stored as an [`Arc`] so we can share it between
    //  the GUI and the audio processing parts. If you have more state to share, then it's a good
    // idea to put all of that in a struct behind a single `Arc`.
    
    //  This is stored as voltage gain.
    // peak_meter: Arc<AtomicF32>,
}


#[derive(Params)]
pub struct PluginParams {
    /// The editor state, saved together with the parameter state so the custom scaling can be
    /// restored.
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "gain"]
    pub gain: FloatParam,

    #[id = "pan"]
    pub pan: FloatParam,

    // TODO: Remove this parameter when we're done implementing the widgets
    #[id = "foobar"]
    pub some_int: IntParam,
}

impl Default for HackAudio {
    fn default() -> Self {
        Self {
            params: Arc::new(PluginParams::default()),

            // peak_meter_decay_weight: 1.0,
            // peak_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),
        }
    }
}


impl Default for PluginParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(600, 800),

            // See the main gain example for more details
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            pan: FloatParam::new(
              "Pan",
              0.0,
              FloatRange::Linear { min: -100.0, max: 100.0 }  
            ).with_unit(" %"),

            
            some_int: IntParam::new("Something", 3, IntRange::Linear { min: 0, max: 3 }),
        }
    }
}

impl Plugin for HackAudio {
    const NAME: &'static str = "Hack Audio FX";
    const VENDOR: &'static str = "notryanb plugins";
    const URL: &'static str = "https://youtu.be/dQw4w9WgXcQ";
    const EMAIL: &'static str = "info@example.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        // let peak_meter = self.peak_meter.clone();
        let egui_state = params.editor_state.clone();
        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                ResizableWindow::new("res-wind")
                    .min_size(Vec2::new(400.0, 400.0))
                    .show(egui_ctx, egui_state.as_ref(), |ui| {
                        // NOTE: See `plugins/diopser/src/editor.rs` for an example using the generic UI widget

                        // This is a fancy widget that can get all the information it needs to properly
                        // display and modify the parameter from the parametr itself
                        // It's not yet fully implemented, as the text is missing.
                        ui.label("Some random integer");
                        ui.add(widgets::ParamSlider::for_param(&params.some_int, setter));

                        ui.label("Gain");
                        ui.add(widgets::ParamSlider::for_param(&params.gain, setter));

                        ui.label("Pan");
                        ui.add(widgets::ParamSlider::for_param(&params.pan, setter));


                        // // TODO: Add a proper custom widget instead of reusing a progress bar
                        // let peak_meter =
                        //     util::gain_to_db(peak_meter.load(std::sync::atomic::Ordering::Relaxed));
                        // let peak_meter_text = if peak_meter > util::MINUS_INFINITY_DB {
                        //     format!("{peak_meter:.1} dBFS")
                        // } else {
                        //     String::from("-inf dBFS")
                        // };

                        // let peak_meter_normalized = (peak_meter + 60.0) / 60.0;
                        // ui.allocate_space(egui::Vec2::splat(2.0));
                        // ui.add(
                        //     egui::widgets::ProgressBar::new(peak_meter_normalized)
                        //         .text(peak_meter_text),
                        // );
                    });
            },
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // self.peak_meter_decay_weight = 0.25f64
        //     .powf((buffer_config.sample_rate as f64 * PEAK_METER_DECAY_MS / 1000.0).recip())
        //     as f32;

        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Linear panning from Hack Audio book
        let pan_value = self.params.pan.value();
        let pan_transform = (pan_value / 200.0) + 0.5;

        for channel_samples in buffer.iter_samples() {
            // let mut amplitude = 0.0;
            // let num_samples = channel_samples.len();
            // let gain = self.params.gain.smoothed.next();

            for (channel, sample) in channel_samples.into_iter().enumerate() {
                if channel == 0 { // Assumes only left and right channels
                    *sample *= 1.0 - pan_transform;
                } else {
                    *sample *= pan_transform;
                }

                // *sample *= gain;
                // amplitude += *sample;
            }

            // To save resources, a plugin can (and probably should!) only perform expensive
            // calculations that are only displayed on the GUI while the GUI is open
            if self.params.editor_state.is_open() {
                // amplitude = (amplitude / num_samples as f32).abs();
                // let current_peak_meter = self.peak_meter.load(std::sync::atomic::Ordering::Relaxed);
                // let new_peak_meter = if amplitude > current_peak_meter {
                //     amplitude
                // } else {
                //     current_peak_meter * self.peak_meter_decay_weight
                //         + amplitude * (1.0 - self.peak_meter_decay_weight)
                // };

                // self.peak_meter
                //     .store(new_peak_meter, std::sync::atomic::Ordering::Relaxed)
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for HackAudio {
    const CLAP_ID: &'static str = "com.notryanb-plugins-.hack-audio";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Implementations of FX in HackAudio book");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for HackAudio {
    const VST3_CLASS_ID: [u8; 16] = *b"HackAudioWhoaWho";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(HackAudio);
nih_export_vst3!(HackAudio);
