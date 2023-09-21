use tdlib::types::File;
use FileStatus::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FileStatus {
    Downloading(f64),
    Uploading(f64),
    CanBeDownloaded,
    Downloaded,
}

impl From<&File> for FileStatus {
    fn from(file: &File) -> Self {
        let local = &file.local;
        let remote = &file.remote;

        let size = file.size.max(file.expected_size) as u64;

        if local.is_downloading_active {
            let progress = local.downloaded_size as f64 / size as f64;
            Downloading(progress)
        } else if remote.is_uploading_active {
            let progress = remote.uploaded_size as f64 / size as f64;
            Uploading(progress)
        } else if local.is_downloading_completed {
            Downloaded
        } else if local.can_be_downloaded {
            CanBeDownloaded
        } else {
            dbg!(file);
            unimplemented!("unknown file status");
        }
    }
}
