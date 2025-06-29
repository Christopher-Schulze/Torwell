use async_trait::async_trait;
use futures::io::{AsyncRead, AsyncWrite};
use futures::Stream;
use pin_project::pin_project;
use std::io::Result as IoResult;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tor_rtcompat::{NetStreamListener, NetStreamProvider, StreamOps};

#[derive(Debug, Clone, Default)]
pub struct TrafficCounters {
    pub bytes_sent: usize,
    pub bytes_received: usize,
}

#[pin_project]
#[derive(Clone)]
pub struct Counting<R> {
    #[pin]
    inner: R,
    counters: Arc<Mutex<TrafficCounters>>,
}

impl<R> Counting<R> {
    pub fn new(inner: R) -> Self
    where
        R: NetStreamProvider,
    {
        Self {
            inner,
            counters: Arc::new(Mutex::new(TrafficCounters::default())),
        }
    }

    pub fn counters(&self) -> TrafficCounters {
        self.counters.lock().expect("lock poisoned").clone()
    }
}

#[async_trait]
impl<R> NetStreamProvider for Counting<R>
where
    R: NetStreamProvider + Send + Sync,
{
    type Stream = Counting<R::Stream>;
    type Listener = Counting<R::Listener>;

    async fn connect(&self, addr: &SocketAddr) -> IoResult<Self::Stream> {
        let inner = self.inner.connect(addr).await?;
        Ok(Counting {
            inner,
            counters: self.counters.clone(),
        })
    }

    async fn listen(&self, addr: &SocketAddr) -> IoResult<Self::Listener> {
        let inner = self.inner.listen(addr).await?;
        Ok(Counting {
            inner,
            counters: self.counters.clone(),
        })
    }
}

impl<S: AsyncRead> AsyncRead for Counting<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<IoResult<usize>> {
        let this = self.project();
        let outcome = this.inner.poll_read(cx, buf);
        if let Poll::Ready(Ok(n)) = outcome {
            this.counters.lock().expect("poisoned").bytes_received += n;
        }
        outcome
    }
}

impl<S: AsyncWrite> AsyncWrite for Counting<S> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<IoResult<usize>> {
        let this = self.project();
        let outcome = this.inner.poll_write(cx, buf);
        if let Poll::Ready(Ok(n)) = outcome {
            this.counters.lock().expect("poisoned").bytes_sent += n;
        }
        outcome
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        self.project().inner.poll_close(cx)
    }
}

impl<S: StreamOps> StreamOps for Counting<S> {
    fn set_tcp_notsent_lowat(&self, notsent_lowat: u32) -> IoResult<()> {
        self.inner.set_tcp_notsent_lowat(notsent_lowat)
    }

    fn new_handle(&self) -> Box<dyn StreamOps + Send + Unpin> {
        self.inner.new_handle()
    }
}

impl<S, T> Stream for Counting<S>
where
    S: Stream<Item = IoResult<(T, SocketAddr)>>,
{
    type Item = IoResult<(Counting<T>, SocketAddr)>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this.inner.poll_next(cx) {
            Poll::Ready(Some(Ok((inner, addr)))) => Poll::Ready(Some(Ok((
                Counting {
                    inner,
                    counters: this.counters.clone(),
                },
                addr,
            )))),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
