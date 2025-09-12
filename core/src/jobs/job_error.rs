// SPDX-FileCopyrightText: 2025 Simon Repp
// SPDX-License-Identifier: AGPL-3.0-or-later

use hyper8_translations::Translations;

use crate::SitePath;

/// Jobs run and fail asynchronously, potentially over long periods of time,
/// in which the state of the interface can change greatly. In addition, the
/// Hyper 8 editor can be used by multiple persons in different languages at
/// the same time. Error messages must therefore be translated at the time of
/// viewing, in the context of the viewer, rather than "baked" into a single
/// language at the time of failure. Long story short: This enum represents
/// error variants and attached data that can be displayed in a localized
/// form at arbitrary times.
#[derive(Clone, Debug)]
pub enum JobError {
    BackgroundJobOutputNotLoadedTargetHasInvalidManifest,
    BackgroundJobOutputNotLoadedTargetMoved,
    BackgroundJobOutputNotLoadedTargetRemoved,
    ChildProcessCouldNotBeExecuted {
        err: String,
        program: &'static str
    },
    ChildProcessTerminatedUnsuccessfully {
        code: Option<i32>,
        program: &'static str
    },
    /// Used for messages that we cannot even translate, e.g. error output
    /// returned from ffprobe, ffmpeg, etc. - use sparingly and/or
    /// iteratively wrap it into more contextualized variants as we go along
    Generic {
        err: String
    },
    ThereWasNoSourceVideoToComputeAPosterFrom {
        site_path: SitePath
    }
}

impl JobError {
    pub fn render(&self, translations: &Translations) -> String {
        match self {
            JobError::BackgroundJobOutputNotLoadedTargetHasInvalidManifest => {
                translations.background_job_output_not_loaded_target_has_invalid_manifest.to_string()
            }
            JobError::BackgroundJobOutputNotLoadedTargetMoved => {
                translations.background_job_output_not_loaded_target_moved.to_string()
            }
            JobError::BackgroundJobOutputNotLoadedTargetRemoved => {
                translations.background_job_output_not_loaded_target_removed.to_string()
            }
            JobError::ChildProcessCouldNotBeExecuted { err, program } => {
                let the_xxx_child_process_could_not_be_executed = translations.the_xxx_child_process_could_not_be_executed(program);
                format!("{the_xxx_child_process_could_not_be_executed} ({err})")
            }
            JobError::ChildProcessTerminatedUnsuccessfully { code, program } => {
                // TODO: Translate
                let error_code = match code {
                    Some(code) => format!("error code {code}"),
                    None => String::from("probably signal-terminated")
                };

                let the_xxx_child_process_terminated_unsuccessfully = translations.the_xxx_child_process_terminated_unsuccessfully(program);
                format!("{the_xxx_child_process_terminated_unsuccessfully} ({error_code})")
            }
            JobError::Generic { err } => {
                err.clone()
            }
            JobError::ThereWasNoSourceVideoToComputeAPosterFrom { site_path } => {
                let site_path = site_path.normalized();
                let t_there_was_no_source_video_to_compute_a_poster_from = translations.there_was_no_source_video_to_compute_a_poster_from;
                format!("{t_there_was_no_source_video_to_compute_a_poster_from} ({site_path})")
            }
        }
    }
}
