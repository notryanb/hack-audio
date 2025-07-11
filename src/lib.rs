use nih_plug::prelude::*;
use nih_plug_egui::{
    EguiState, create_egui_editor, egui, egui::Vec2, resizable_window::ResizableWindow, widgets,
};
use std::sync::Arc;

#[derive(Clone, Enum, PartialEq)]
pub enum Fx {
    #[id = "panning"]
    Panning,

    #[id = "mid-side-encode"]
    MidSideEncode,

    #[id = "mid-side-decode"]
    MidSideDecode,
}

impl Fx {
    pub fn to_f32(fx: Fx) -> f32 {
        match fx {
            Fx::Panning => 0.0,
            Fx::MidSideEncode => 1.0,
            Fx::MidSideDecode=> 2.0,
        }
    }

    pub fn from_f32(i: f32) -> Self {
        match i {
            2.0 => Fx::MidSideDecode,
            1.0 => Fx::MidSideEncode,
            _ => Fx::Panning,
        }
    }
}

#[derive(Clone)]
pub struct UiState {}

#[derive(Clone, Enum, PartialEq)]
pub enum PanningMode {
    #[id = "linear"]
    Linear,

    #[id = "square"]
    Square,

    #[id = "sine"]
    Sine,
}

impl PanningMode {
    pub fn to_f32(gm: PanningMode) -> f32 {
        match gm {
            PanningMode::Linear => 0.0,
            PanningMode::Square => 1.0,
            PanningMode::Sine => 2.0,
        }
    }

    pub fn from_f32(i: f32) -> Self {
        match i {
            2.0 => PanningMode::Sine,
            1.0 => PanningMode::Square,
            _ => PanningMode::Linear,
        }
    }
}

/// This is mostly identical to the gain example, minus some fluff, and with a GUI.
pub struct HackAudio {
    params: Arc<PluginParams>,
    ui_state: UiState,
}

#[derive(Params)]
pub struct PluginParams {
    /// The editor state, saved together with the parameter state so the custom scaling can be
    /// restored.
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "selected_fx"]
    pub selected_fx: EnumParam<Fx>,

    #[id = "pan"]
    pub pan: FloatParam,

    #[id = "panning_mode"]
    pub panning_mode: EnumParam<PanningMode>,

    #[id = "mid-side-encoding-stereo-width"]
    pub mid_side_enc_stereo_width: FloatParam,
}

impl Default for HackAudio {
    fn default() -> Self {
        Self {
            params: Arc::new(PluginParams::default()),
            ui_state: UiState {},
        }
    }
}

impl Default for PluginParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(600, 800),

            selected_fx: EnumParam::new("Selected Fx", Fx::Panning),
            panning_mode: EnumParam::new("Panning Mode", PanningMode::Linear),
            pan: FloatParam::new(
                "Pan",
                0.0,
                FloatRange::Linear {
                    min: -100.0,
                    max: 100.0,
                },
            )
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            mid_side_enc_stereo_width: FloatParam::new(
                "Stereo Width",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 2.0,
                },
            )
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
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
        let egui_state = params.editor_state.clone();
        create_egui_editor(
            self.params.editor_state.clone(),
            self.ui_state.clone(),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                ResizableWindow::new("resizable-window")
                    .min_size(Vec2::new(400.0, 400.0))
                    .show(egui_ctx, egui_state.as_ref(), |_ui| {
                        let selected_fx = &params.selected_fx.value();
                        let panning_mode = &params.panning_mode.value();

                        egui::TopBottomPanel::top("menu").show(egui_ctx, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("FX");

                                if ui
                                    .add(egui::widgets::SelectableLabel::new(
                                        *selected_fx == Fx::Panning,
                                        "Panning",
                                    ))
                                    .clicked()
                                {
                                    setter.begin_set_parameter(&params.selected_fx);
                                    setter.set_parameter(&params.selected_fx, Fx::Panning);
                                    setter.end_set_parameter(&params.selected_fx);
                                }
                                if ui
                                    .add(egui::widgets::SelectableLabel::new(
                                        *selected_fx == Fx::MidSideEncode,
                                        "Mid-Side Encode",
                                    ))
                                    .clicked()
                                {
                                    setter.begin_set_parameter(&params.selected_fx);
                                    setter.set_parameter(&params.selected_fx, Fx::MidSideEncode);
                                    setter.end_set_parameter(&params.selected_fx);
                                }
                                if ui
                                    .add(egui::widgets::SelectableLabel::new(
                                        *selected_fx == Fx::MidSideDecode,
                                        "Mid-Side Decode",
                                    ))
                                    .clicked()
                                {
                                    setter.begin_set_parameter(&params.selected_fx);
                                    setter.set_parameter(&params.selected_fx, Fx::MidSideDecode);
                                    setter.end_set_parameter(&params.selected_fx);
                                }
                            });
                        });

                        egui::CentralPanel::default().show(egui_ctx, |ui| match selected_fx {
                            Fx::Panning => {
                                ui.horizontal(|ui| {
                                    if ui
                                        .add(egui::widgets::SelectableLabel::new(
                                            *panning_mode == PanningMode::Linear,
                                            "Linear",
                                        ))
                                        .clicked()
                                    {
                                        setter.begin_set_parameter(&params.panning_mode);
                                        setter.set_parameter(
                                            &params.panning_mode,
                                            PanningMode::Linear,
                                        );
                                        setter.end_set_parameter(&params.panning_mode);
                                    }

                                    if ui
                                        .add(egui::widgets::SelectableLabel::new(
                                            *panning_mode == PanningMode::Square,
                                            "Square",
                                        ))
                                        .clicked()
                                    {
                                        setter.begin_set_parameter(&params.panning_mode);
                                        setter.set_parameter(
                                            &params.panning_mode,
                                            PanningMode::Square,
                                        );
                                        setter.end_set_parameter(&params.panning_mode);
                                    }
                                    if ui
                                        .add(egui::widgets::SelectableLabel::new(
                                            *panning_mode == PanningMode::Sine,
                                            "Sine",
                                        ))
                                        .clicked()
                                    {
                                        setter.begin_set_parameter(&params.panning_mode);
                                        setter
                                            .set_parameter(&params.panning_mode, PanningMode::Sine);
                                        setter.end_set_parameter(&params.panning_mode);
                                    }
                                });

                                ui.label("Pan");
                                ui.add(widgets::ParamSlider::for_param(&params.pan, setter));
                            }
                            Fx::MidSideEncode => { 
                                ui.label("MidSideEncode"); 
                                ui.separator();
                                
                                ui.label("Stereo Width");
                                ui.add(widgets::ParamSlider::for_param(&params.mid_side_enc_stereo_width, setter));
                            }
                            Fx::MidSideDecode => { ui.label("MidSideDecode"); }
                        });
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
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let selected_fx = self.params.selected_fx.value();

        match selected_fx {
            Fx::Panning => panning_plugin_process(buffer, &self.params),
            Fx::MidSideEncode => mid_side_encode_plugin_process(buffer, &self.params),
            Fx::MidSideDecode => mid_side_decode_plugin_process(buffer, &self.params),
        }
    }
}

/// Linear panning from Hack Audio Book
pub fn panning_plugin_process(buffer: &mut Buffer, params: &Arc<PluginParams>) -> ProcessStatus {
    let pan_value = params.pan.value();
    let panning_mode = params.panning_mode.value();
    let pan_transform = (pan_value / 200.0) + 0.5;

    for channel_samples in buffer.iter_samples() {
        for (channel, sample) in channel_samples.into_iter().enumerate() {
            if channel == 0 {
                // Assumes only left and right channels
                let new_sample = match panning_mode {
                    PanningMode::Linear => 1.0 - pan_transform,
                    PanningMode::Square => (1.0 - pan_transform).sqrt(),
                    PanningMode::Sine => {
                        ((1.0 - pan_transform) * (std::f32::consts::PI / 2.0)).sin()
                    }
                };

                *sample *= new_sample;
            } else {
                let new_sample = match panning_mode {
                    PanningMode::Linear => pan_transform,
                    PanningMode::Square => (pan_transform).sqrt(),
                    PanningMode::Sine => (pan_transform * (std::f32::consts::PI / 2.0)).sin(),
                };

                *sample *= new_sample;
            }
        }
    }

    ProcessStatus::Normal
}

pub fn mid_side_encode_plugin_process(buffer: &mut Buffer, params: &Arc<PluginParams>) -> ProcessStatus {
    let num_samples = buffer.samples();
    let stereo_width = params.mid_side_enc_stereo_width.value();
    let output = buffer.as_slice();

    for sample_idx in 0..num_samples {
        let mid = (2.0 - stereo_width) * (output[0][sample_idx] + output[1][sample_idx]) * 0.5;
        let side = stereo_width * (output[0][sample_idx] - output[1][sample_idx]) * 0.5;

        output[0][sample_idx] = mid;
        output[1][sample_idx] = side;
    }

    ProcessStatus::Normal
}

pub fn mid_side_decode_plugin_process(buffer: &mut Buffer, _params: &Arc<PluginParams>) -> ProcessStatus {
    let num_samples = buffer.samples();
    let output = buffer.as_slice();

    for sample_idx in 0..num_samples {
        let left = output[0][sample_idx] + output[1][sample_idx];
        let right = output[0][sample_idx] - output[1][sample_idx];

        output[0][sample_idx] = left;
        output[1][sample_idx] = right;
    }

    ProcessStatus::Normal
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
