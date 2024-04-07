use std::{future::ready, pin::Pin};

use tokio::io::{AsyncRead, AsyncWrite};
use interprocess::local_socket::{tokio::prelude::LocalSocketListener, traits::tokio::{Listener, Stream}, NameTypeSupport, ToFsName, ToNsName};
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, sync::mpsc, try_join};
use odyssey_hub_service_interface::{greeter_server::{Greeter, GreeterServer}, HelloReply, HelloRequest};
use tonic::transport::server::Connected;


#[derive(Debug, Default)]
struct Server {}

#[tonic::async_trait]
impl Greeter for Server {
    async fn say_hello(
        &self,
        request: tonic::Request<HelloRequest>,
    ) -> Result<tonic::Response<HelloReply>, tonic::Status> {
        println!("Got a request: {:?}", request);
        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name), // We must use .into_inner() as the fields of gRPC requests and responses are private
        };

        Ok(tonic::Response::new(reply)) // Send back our formatted greeting}
    }
}

pub enum Message {
    ServerInit(Result<(), std::io::Error>),
    Stop,
}

struct LocalSocketStream(interprocess::local_socket::tokio::prelude::LocalSocketStream);

impl LocalSocketStream {
    fn split(self) -> (interprocess::local_socket::tokio::RecvHalf, interprocess::local_socket::tokio::SendHalf) {
        self.0.split()
    }
    fn inner_pin(self: Pin<&mut Self>) -> Pin<&mut interprocess::local_socket::tokio::prelude::LocalSocketStream> {
        unsafe {
            self.map_unchecked_mut(|s| &mut s.0)
        }
    }
}

impl Connected for LocalSocketStream {
    type ConnectInfo = ();

    fn connect_info(&self) -> Self::ConnectInfo {
        ()
    }
}

impl AsyncWrite for LocalSocketStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        self.inner_pin().poll_write(cx, buf)
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        self.inner_pin().poll_flush(cx)
    }

    fn poll_shutdown(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        self.inner_pin().poll_shutdown(cx)
    }

    fn poll_write_vectored(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        self.inner_pin().poll_write_vectored(cx, bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }
}

impl AsyncRead for LocalSocketStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        self.inner_pin().poll_read(cx, buf)
    }
}

pub async fn run_service(sender: mpsc::UnboundedSender<Message>) -> anyhow::Result<()> {
    // Pick a name. There isn't a helper function for this, mostly because it's largely unnecessary:
    // in Rust, `match` is your concise, readable and expressive decision making construct.
    let name = {
        // This scoping trick allows us to nicely contain the import inside the `match`, so that if
        // any imports of variants named `Both` happen down the line, they won't collide with the
        // enum we're working with here. Maybe someone should make a macro for this.
        use NameTypeSupport::*;
        match NameTypeSupport::query() {
            OnlyFs => "/tmp/odyhub.sock".to_fs_name(),
            OnlyNs | Both => "@odyhub.sock".to_ns_name(),
        }
    };
    let name = name?;
    // Create our listener. In a more robust program, we'd check for an
    // existing socket file that has not been deleted for whatever reason,
    // ensure it's a socket file and not a normal file, and delete it.
    let listener = LocalSocketListener::bind(name.clone())?;
    let listener = futures::stream::unfold((), |()| {
        async { Some((listener.accept().await.map(LocalSocketStream), ())) }
    });
    println!("Server running at {:?}", name.clone());

    sender.send(Message::ServerInit(Ok(()))).unwrap();

    tonic::transport::Server::builder()
        .add_service(GreeterServer::new(Server::default()))
        .serve_with_incoming(listener)
        .await?;

    Ok(())
}
