use derive_more::From;

pub type Result<T> = core::result::Result<T, Error>;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, From)]
pub enum Error {
    // // -- String errors
    #[from]
    Message(String),
    #[from]
    IoError(std::io::Error),
    #[from]
    TokioPostgres(tokio_postgres::Error),
    #[from]
    ChecksumMismatchError(ChecksumMismatchError),
    #[from]
    MissingVariableTemplateError(MissingVariableTemplateError),
}

#[derive(Debug)]
pub struct ChecksumMismatchError {
    pub file_name: String,
    pub file_checksum: i32,
    pub applied_checksum: i32,
}

#[derive(Debug)]
pub struct MissingVariableTemplateError {
    pub name: String,
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate

// converting string to out error when calling .into()
impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Message(value.to_string())
    }
}
