#[derive(Debug)]
pub enum AssetLoadError {
    LoadFileError,
    FormatError,
    LoadImageError,
    UploadImageError,
    NotFoundLoader,
    FindDepAssetError
}