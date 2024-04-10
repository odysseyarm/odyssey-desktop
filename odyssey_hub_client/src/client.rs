use interprocess::local_socket::{tokio::prelude::LocalSocketStream, traits::tokio::Stream, NameTypeSupport, ToFsName, ToNsName};
use odyssey_hub_service_interface::{service_client::ServiceClient, DeviceListRequest};
use tokio_util::sync::CancellationToken;
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;

#[derive(Default)]
pub struct Client {
    pub end_token: CancellationToken,
    pub service_client: Option<ServiceClient<tonic::transport::Channel>>,
}

impl Client {
    pub async fn connect(&mut self) -> anyhow::Result<()> {
        let name = {
            use NameTypeSupport as Nts;
            match NameTypeSupport::query() {
                Nts::OnlyFs => "/tmp/odyhub.sock".to_fs_name().unwrap(),
                Nts::OnlyNs | Nts::Both => "@odyhub.sock".to_ns_name().unwrap(),
            }
        };

        // Await this here since we can't do a whole lot without a connection.
        // URI is ignored
        let channel = Endpoint::try_from("http://[::]:50051")?
            .connect_with_connector(service_fn(move |_: Uri| {
                let name = name.clone();
                async move {
                    let r = LocalSocketStream::connect(name).await?;
                    let r = odyssey_hub_service::service::LocalSocketStream::new(r);
                    dbg!(std::io::Result::Ok(r))
                }
            })).await.unwrap();

        self.service_client = Some(ServiceClient::new(channel));
        println!("connected");

        Ok(())
    }

    pub async fn get_device_list(&mut self) -> anyhow::Result<Vec<odyssey_hub_common::device::Device>> {
        if let Some(service_client) = &mut self.service_client {
            let request = tonic::Request::new(DeviceListRequest {});
            // for whatever insane reason get_device_list requires mutable service_client
            let response = service_client.get_device_list(request).await.unwrap();
            println!("RESPONSE={:?}", response);
            Ok(response.into_inner().device_list.into_iter().map(|d| {
                d.into()
            }).collect::<Vec<odyssey_hub_common::device::Device>>())
        } else {
            Err(anyhow::anyhow!("No service client"))
        }
    }
}
