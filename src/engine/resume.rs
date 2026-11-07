use crate::engine::http::RemoteFileInfo;

pub struct ResumeValidator;

impl ResumeValidator {
    pub fn can_resume(
        previous_etag: Option<&str>,
        previous_size: Option<u64>,
        current_info: &RemoteFileInfo,
    ) -> bool {
        if !current_info.supports_range {
            return false;
        }

        // If ETag matches
        if let (Some(prev_tag), Some(curr_tag)) = (previous_etag, current_info.etag.as_deref()) {
            return prev_tag == curr_tag;
        }

        // If Content-Length matches
        if let (Some(prev_len), Some(curr_len)) = (previous_size, current_info.content_length) {
            return prev_len == curr_len;
        }

        // If no metadata exists to compare, assume caution
        false
    }
}
