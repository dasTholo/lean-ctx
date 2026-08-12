//! Offline distillation flow: Gold Set -> Teacher Expansion -> Training Data -> Model.

pub mod data_pipeline;
pub mod model_config;
pub mod teacher_labeling;

pub use data_pipeline::{DistributionReport, TrainingConfig, split_dataset, validate_distribution};
pub use model_config::{ModelManifest, TinyModelConfig, model_pack_path};
pub use teacher_labeling::{
    LabeledSample, TeacherConfig, TeacherLabels, TeacherPrompt, prepare_teacher_batch,
};
