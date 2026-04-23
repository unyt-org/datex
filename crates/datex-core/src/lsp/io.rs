use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures::{AsyncRead, AsyncWrite, StreamExt, channel::mpsc};

pub struct Writer {
    tx: mpsc::UnboundedSender<Vec<u8>>,
}
impl Writer {
    pub fn new(tx: mpsc::UnboundedSender<Vec<u8>>) -> Self {
        Self { tx }
    }
}

impl AsyncWrite for Writer {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        let _ = self.tx.unbounded_send(buf.to_vec());
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
}

pub struct Reader {
    rx: mpsc::UnboundedReceiver<Vec<u8>>,
    buffer: Vec<u8>,
    pos: usize,
}

impl Reader {
    pub fn new(rx: mpsc::UnboundedReceiver<Vec<u8>>) -> Self {
        Self {
            rx,
            buffer: Vec::new(),
            pos: 0,
        }
    }
}

impl AsyncRead for Reader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        if self.pos >= self.buffer.len() {
            match Pin::new(&mut self.rx).poll_next_unpin(cx) {
                Poll::Ready(Some(vec)) => {
                    self.buffer = vec;
                    self.pos = 0;
                }
                Poll::Ready(None) => return Poll::Ready(Ok(0)),
                Poll::Pending => return Poll::Pending,
            }
        }

        let n = core::cmp::min(buf.len(), self.buffer.len() - self.pos);
        buf[..n].copy_from_slice(&self.buffer[self.pos..self.pos + n]);
        self.pos += n;
        Poll::Ready(Ok(n))
    }
}
