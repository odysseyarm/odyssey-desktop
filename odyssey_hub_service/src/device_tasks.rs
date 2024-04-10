use std::net::Ipv4Addr;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{self, Sender};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Connect(odyssey_hub_common::device::Device),
    Disconnect(odyssey_hub_common::device::Device),
}

pub async fn device_tasks(message_channel: Sender<Message>) -> anyhow::Result<()> {
    let (sender, mut receiver) = mpsc::channel(12);
    tokio::select! {
        _ = device_ping_task(sender.clone()) => {},
        _ = device_hid_task(sender.clone()) => {},
        _ = device_cdc_task(sender.clone()) => {},
        _ = async {
            while let Some(message) = receiver.recv().await {
                message_channel.send(message).await.unwrap();
            }
        } => {},
    }
    Ok(())
}

async fn device_ping_task(message_channel: Sender<Message>) -> std::convert::Infallible {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 23456)).await.unwrap();
    socket.set_broadcast(true).unwrap();

    let mut old_list = vec![];
    loop {
        let mut new_list = vec![];
        let mut buf = [0; 1472];
        socket.send_to(&[255, 1], ("10.0.0.255", 23456)).await.unwrap();
        futures::future::select(
            std::pin::pin!(tokio::time::sleep(tokio::time::Duration::from_secs(2))),
            std::pin::pin!(async {
                loop {
                    let (len, addr) = socket.recv_from(&mut buf).await.unwrap();
                    if buf[0] == 255 { continue; }
                    if !old_list.contains(&odyssey_hub_common::device::Device::Udp((addr, buf[1]))) {
                        let _ = message_channel.send(Message::Connect(odyssey_hub_common::device::Device::Udp((addr, buf[1])))).await;
                    }
                    new_list.push(odyssey_hub_common::device::Device::Udp((addr, buf[1])));
                }
            })
        ).await;
        dbg!(&new_list);
        for v in &old_list {
            if !new_list.contains(v) {
                let _ = message_channel.send(Message::Disconnect(*v)).await;
            }
        }
        old_list = new_list;
    }
}

async fn device_hid_task(message_channel: Sender<Message>) -> std::convert::Infallible {
    let api = hidapi::HidApi::new().unwrap();

    let mut old_list = vec![];
    loop {
        let mut new_list = vec![];
        for device in api.device_list() {
            if device.vendor_id() == 0x1915 && device.product_id() == 0x48AB {
                if !old_list.contains(&odyssey_hub_common::device::Device::Hid) {
                    let _ = message_channel.send(Message::Connect(odyssey_hub_common::device::Device::Hid)).await;
                }
                new_list.push(odyssey_hub_common::device::Device::Hid);
            }
        }
        dbg!(&new_list);
        for v in &old_list {
            if !new_list.contains(v) {
                let _ = message_channel.send(Message::Disconnect(*v)).await;
            }
        }
        old_list = new_list;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}

async fn device_cdc_task(message_channel: Sender<Message>) -> std::convert::Infallible {
    let mut old_list = vec![];
    loop {
        let mut new_list = vec![];
        let ports = serialport::available_ports();
        let ports: Vec<_> = match ports {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to list serial ports {}", &e.to_string());
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                continue;
            }
        }.into_iter().filter(|port| {
            match &port.port_type {
                serialport::SerialPortType::UsbPort(port_info) => {
                    if port_info.vid != 0x1915 || port_info.pid != 0x520f {
                        return false;
                    }
                    if let Some(i) = port_info.interface {
                        // interface 0: cdc acm module
                        // interface 1: cdc acm module functional subordinate interface
                        // interface 2: cdc acm dfu
                        // interface 3: cdc acm dfu subordinate interface
                        i == 0
                    } else {
                        true
                    }
                },
                _ => false,
            }
        }).collect();
        for device in ports {
            if !old_list.contains(&odyssey_hub_common::device::Device::Cdc) {
                let _ = message_channel.send(Message::Connect(odyssey_hub_common::device::Device::Cdc)).await;
            }
            new_list.push(odyssey_hub_common::device::Device::Cdc);
        }
        dbg!(&new_list);
        for v in &old_list {
            if !new_list.contains(v) {
                let _ = message_channel.send(Message::Disconnect(*v)).await;
            }
        }
        old_list = new_list;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}
