use tokio_util::bytes::Bytes;

pub enum Frame {
    Err(Bytes),
    ExitStatusCode(i32),
    Out(Bytes),
}

impl Frame {
    pub fn kind(&self) -> FrameKind {
        match self {
            Self::Err(_) => FrameKind::Err,
            Self::ExitStatusCode(_) => FrameKind::ExitStatusCode,
            Self::Out(_) => FrameKind::Out,
        }
    }
}

pub enum FrameKind {
    Err = 0,
    ExitStatusCode = 1,
    Out = 2,
}

pub const MAX: usize = 8 * 1024 * 1024;
