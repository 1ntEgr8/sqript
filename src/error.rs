#[derive(Debug, Clone)]
pub enum SrError {
    NixError(nix::errno::Errno),
}

impl From<nix::errno::Errno> for SrError {
    fn from(err: nix::errno::Errno) -> SrError {
        SrError::NixError(err)
    }
}


