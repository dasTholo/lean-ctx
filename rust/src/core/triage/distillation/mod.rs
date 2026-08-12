//! Offline distillation flow: Gold Set -> Teacher Expansion -> Training Data -> Model.

pub mod data_pipeline;
pub mod model_config;
pub mod teacher_labeling;

pub use data_pipeline::{
    AugmentationHint, DatasetSplit, DistributionReport, TrainingConfig, augmentation_hints,
    export_fine_tuning_jsonl, split_dataset, split_dataset_with_seed, validate_distribution,
};
pub use model_config::{ModelManifest, TinyModelConfig, model_pack_path};
pub use teacher_labeling::{
    LabeledSample, TeacherConfig, TeacherLabelError, TeacherLabels, TeacherPrompt,
    prepare_teacher_batch, validate_teacher_labels, with_teacher_retries,
};
