use std::process;

use clap::Parser;
use futures::TryStreamExt;
use muxeo::{Frame, FrameKind, MAX};
use tokio::io::{self, AsyncWriteExt};
use tokio_util::{
    bytes::{Buf, BytesMut},
    codec::{Decoder, FramedRead},
};

/// Demultiplexer for standard error and standard output
#[derive(Parser)]
#[command(version)]
struct Cli;

struct EoDecoder;

impl Decoder for EoDecoder {
    type Item = Frame;

    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 1 + 4 {
            // Not enough data to read kind marker and length marker / exit
            // status code.
            return Ok(None);
        }

        // Read kind marker.
        let kind = match src[0] {
            0 => FrameKind::Err,
            1 => FrameKind::ExitStatusCode,
            2 => FrameKind::Out,
            k => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid frame kind {}.", k),
                ));
            }
        };

        if let FrameKind::ExitStatusCode = kind {
            // Read exit status code.
            let mut code_bytes = [0; 4];
            code_bytes.copy_from_slice(&src[1..1 + 4]);
            let code = i32::from_be_bytes(code_bytes);

            // Use advance to modify src such that it no longer contains this
            // frame.
            src.advance(1 + 4);

            return Ok(Some(Frame::ExitStatusCode(code)));
        }

        // Read length marker.
        let mut len_bytes = [0; 4];
        len_bytes.copy_from_slice(&src[1..1 + 4]);
        let bytes_len = u32::from_be_bytes(len_bytes) as usize;

        // Check that the length is not too large to avoid a denial of
        // service attack where the server runs out of memory.
        if bytes_len > MAX {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Frame of length {} is too large.", bytes_len),
            ));
        }

        if src.len() < 1 + 4 + bytes_len {
            // The full frame has not yet arrived.
            //
            // We reserve more space in the buffer. This is not strictly
            // necessary, but is a good idea performance-wise.
            src.reserve(1 + 4 + bytes_len - src.len());

            // We inform the Framed that we need more bytes to form the next
            // frame.
            return Ok(None);
        }

        // Use advance and split_to to modify src such that it no longer
        // contains this frame.
        src.advance(1 + 4);
        let bytes = src.split_to(bytes_len).freeze();

        Ok(Some(match kind {
            FrameKind::Err => Frame::Err(bytes),
            FrameKind::Out => Frame::Out(bytes),
            _ => unreachable!(),
        }))
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    Cli::parse();

    let mut stderr = io::stderr();
    let mut stdout = io::stdout();
    let mut frames = FramedRead::new(io::stdin(), EoDecoder);

    while let Some(frame) = frames.try_next().await? {
        match frame {
            Frame::Err(mut bytes) => {
                stderr.write_all_buf(&mut bytes).await?;
            }
            Frame::ExitStatusCode(code) => process::exit(code),
            Frame::Out(mut bytes) => {
                stdout.write_all_buf(&mut bytes).await?;
            }
        }
    }

    Ok(())
}
