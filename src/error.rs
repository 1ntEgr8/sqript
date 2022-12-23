#[derive(Debug)]
pub enum SrError {
    NixError(nix::errno::Errno),
    IoError(std::io::Error),
}

impl From<nix::errno::Errno> for SrError {
    fn from(err: nix::errno::Errno) -> SrError {
        SrError::NixError(err)
    }
}

impl From<std::io::Error> for SrError {
    fn from(err: std::io::Error) -> SrError {
        SrError::IoError(err)
    }
}
