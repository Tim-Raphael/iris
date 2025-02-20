use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

use crate::entities::{Device, DeviceId, UserId};

#[derive(Debug)]
pub enum ServerCommand {
    RegisterUser {
        conn_tx: mpsc::UnboundedSender<String>,
        res_tx: oneshot::Sender<UserId>,
    },

    UnregisterUser {
        user_id: UserId,
    },

    RegisterDevice {
        name: String,
        conn_tx: mpsc::UnboundedSender<String>,
        res_tx: oneshot::Sender<DeviceId>,
    },

    UnregisterDevice {
        device_id: DeviceId,
    },

    Connect {
        user_id: UserId,
        device_id: DeviceId,
    },

    Disconnect {
        user_id: UserId,
        device_id: DeviceId,
    },

    UserSignaling {
        user_id: UserId,
        device_id: DeviceId,
        signal: serde_json::Value,
    },

    DeviceSignaling {
        device_id: DeviceId,
        signal: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerCommandJson {
    Connect {
        device_id: DeviceId,
    },

    Disconnect {
        device_id: DeviceId,
    },

    DeviceSignaling {
        signal: serde_json::Value,
    },

    UserSignaling {
        device_id: DeviceId,
        signal: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum UserCommand<'a> {
    UpdateDevices {
        devices: Vec<&'a Device>,
    },

    RemoveDevice {
        device_id: &'a DeviceId,
    },

    UpdateConnectedDevice {
        device: &'a Device,
    },

    RemoveConnectedDevice {
        device_id: &'a DeviceId,
    },

    DeviceSignaling {
        device_id: &'a DeviceId,
        signal: serde_json::Value,
    },

    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum DeviceCommand {
    UserSignaling { signal: serde_json::Value },
    Error { message: String },
}
