use std::net::SocketAddr;

mod proto {
    tonic::include_proto!("odyssey.service_interface");
}

pub use proto::*;

impl From<nalgebra::Matrix3x1<f64>> for proto::Matrix3x1 {
    fn from(value: nalgebra::Matrix3x1<f64>) -> Self {
        Self {
            m11: value.x,
            m21: value.y,
            m31: value.z,
        }
    }
}

impl Into<nalgebra::Matrix3x1<f64>> for proto::Matrix3x1 {
    fn into(self) -> nalgebra::Matrix3x1<f64> {
        nalgebra::Matrix3x1::new(self.m11, self.m21, self.m31)
    }
}

impl From<nalgebra::Matrix3<f64>> for proto::Matrix3x3 {
    fn from(value: nalgebra::Matrix3<f64>) -> Self {
        Self {
            m11: value.m11,
            m12: value.m12,
            m13: value.m13,
            m21: value.m21,
            m22: value.m22,
            m23: value.m23,
            m31: value.m31,
            m32: value.m32,
            m33: value.m33,
        }
    }
}

impl Into<nalgebra::Matrix3<f64>> for proto::Matrix3x3 {
    fn into(self) -> nalgebra::Matrix3<f64> {
        nalgebra::Matrix3::new(
            self.m11, self.m12, self.m13,
            self.m21, self.m22, self.m23,
            self.m31, self.m32, self.m33,
        )
    }
}

impl From<odyssey_hub_common::device::Device> for proto::Device {
    fn from(value: odyssey_hub_common::device::Device) -> Self {
        match value {
            odyssey_hub_common::device::Device::Udp(udp_device) => {
                proto::Device {
                    device_oneof: Some(proto::device::DeviceOneof::UdpDevice(proto::UdpDevice {
                        id: udp_device.id as i32,
                        ip: udp_device.addr.ip().to_string(),
                        port: udp_device.addr.port() as i32,
                        uuid: u64::from_le_bytes([udp_device.uuid[0], udp_device.uuid[1], udp_device.uuid[2], udp_device.uuid[3], udp_device.uuid[4], udp_device.uuid[5], 0, 0]),
                    })),
                }
            },
            odyssey_hub_common::device::Device::Hid(d) => proto::Device {
                device_oneof: Some(proto::device::DeviceOneof::HidDevice(proto::HidDevice { path: d.path, uuid: u64::from_le_bytes([d.uuid[0], d.uuid[1], d.uuid[2], d.uuid[3], d.uuid[4], d.uuid[5], 0, 0]), })),
            },
            odyssey_hub_common::device::Device::Cdc(d) => proto::Device {
                device_oneof: Some(proto::device::DeviceOneof::CdcDevice(proto::CdcDevice { path: d.path, uuid: u64::from_le_bytes([d.uuid[0], d.uuid[1], d.uuid[2], d.uuid[3], d.uuid[4], d.uuid[5], 0, 0]), })),
            },
        }
    }
}

impl From<proto::Device> for odyssey_hub_common::device::Device {
    fn from(value: proto::Device) -> Self {
        match value.device_oneof.unwrap() {
            proto::device::DeviceOneof::UdpDevice(proto::UdpDevice { id, ip, port, uuid }) => {
                odyssey_hub_common::device::Device::Udp(
                    odyssey_hub_common::device::UdpDevice {
                        id: id as u8,
                        addr: SocketAddr::new(ip.parse().unwrap(), port as u16),
                        uuid: uuid.to_le_bytes()[0..6].try_into().unwrap(),
                    }
                )
            },
            proto::device::DeviceOneof::HidDevice(proto::HidDevice { path, uuid }) => {
                odyssey_hub_common::device::Device::Hid(
                    odyssey_hub_common::device::HidDevice {
                        path,
                        uuid: uuid.to_le_bytes()[0..6].try_into().unwrap(),
                    }
                )
            },
            proto::device::DeviceOneof::CdcDevice(proto::CdcDevice { path, uuid }) => {
                odyssey_hub_common::device::Device::Cdc(
                    odyssey_hub_common::device::CdcDevice {
                        path,
                        uuid: uuid.to_le_bytes()[0..6].try_into().unwrap(),
                    }
                )
            },
        }
    }
}

impl From<odyssey_hub_common::events::Event> for proto::Event {
    fn from(value: odyssey_hub_common::events::Event) -> Self {
        match value {
            odyssey_hub_common::events::Event::None => proto::Event {
                event_oneof: None,
            },
            odyssey_hub_common::events::Event::DeviceEvent(odyssey_hub_common::events::DeviceEvent { device, kind }) => {
                proto::Event {
                    event_oneof: Some(
                        proto::event::EventOneof::Device(
                            proto::DeviceEvent {
                                device: Some(device.into()),
                                device_event_oneof: Some(match kind {
                                    odyssey_hub_common::events::DeviceEventKind::TrackingEvent(odyssey_hub_common::events::TrackingEvent { timestamp, aimpoint, pose, screen_id }) => {
                                        proto::device_event::DeviceEventOneof::Tracking(
                                            proto::device_event::TrackingEvent {
                                                timestamp,
                                                aimpoint: Some(proto::Vector2 {
                                                    x: aimpoint.x,
                                                    y: aimpoint.y,
                                                }),
                                                pose: Some(proto::Pose {
                                                    rotation: Some(pose.unwrap().rotation.into()),
                                                    translation: Some(pose.unwrap().translation.into()),
                                                }),
                                                screen_id,
                                            },
                                        )
                                    }
                                    odyssey_hub_common::events::DeviceEventKind::ImpactEvent(odyssey_hub_common::events::ImpactEvent { timestamp, }) => {
                                        proto::device_event::DeviceEventOneof::Impact(
                                            proto::device_event::ImpactEvent {
                                                timestamp,
                                            },
                                        )
                                    }
                                })
                            }
                        ),
                    )
                }
            },
        }
    }
}

impl From<proto::Event> for odyssey_hub_common::events::Event {
    fn from(event: proto::Event) -> Self {
        match event.event_oneof.unwrap() {
            proto::event::EventOneof::Device(proto::DeviceEvent { device, device_event_oneof }) => {
                odyssey_hub_common::events::Event::DeviceEvent(
                    odyssey_hub_common::events::DeviceEvent {
                        device: device.unwrap().into(),
                        kind: match device_event_oneof.unwrap() {
                            proto::device_event::DeviceEventOneof::Tracking(proto::device_event::TrackingEvent { timestamp, aimpoint, pose, screen_id }) => {
                                odyssey_hub_common::events::DeviceEventKind::TrackingEvent(
                                    odyssey_hub_common::events::TrackingEvent {
                                        timestamp,
                                        aimpoint: nalgebra::Vector2::new(aimpoint.clone().unwrap().x, aimpoint.clone().unwrap().y),
                                        pose: if let Some(pose) = pose {
                                            Some(odyssey_hub_common::events::Pose {
                                                rotation: pose.rotation.unwrap().into(),
                                                translation: pose.translation.unwrap().into(),
                                            })
                                        } else {
                                            None
                                        },
                                        screen_id,
                                    },
                                )
                            },
                            proto::device_event::DeviceEventOneof::Impact(proto::device_event::ImpactEvent { timestamp, }) => {
                                odyssey_hub_common::events::DeviceEventKind::ImpactEvent(
                                    odyssey_hub_common::events::ImpactEvent {
                                        timestamp,
                                    },
                                )
                            },
                        }
                    }
                )
            }
        }
    }
}
