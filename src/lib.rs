use tokio_util::bytes::Bytes;

pub enum Frame {
    Err(Bytes),
    Out(Bytes),
}

impl Frame {
    pub fn kind(&self) -> FrameKind {
        match self {
            Self::Err(_) => FrameKind::Err,
            Self::Out(_) => FrameKind::Out,
        }
    }
}

pub enum FrameKind {
    Err = 0,
    Out = 1,
}

pub const MAX: usize = 8 * 1024 * 1024;
