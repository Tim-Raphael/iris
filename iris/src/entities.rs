use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

pub type DeviceId = Uuid;
pub type UserId = Uuid;

#[derive(PartialEq, Deserialize, Serialize, Debug, Clone)]
pub enum DeviceState {
    Open,
    Connected,
}

#[derive(Serialize, Debug, Clone)]
pub struct Device {
    pub id: DeviceId,
    pub name: String,
    pub state: DeviceState,

    #[serde(skip)]
    pub conn_tx: mpsc::UnboundedSender<String>,
}

impl Device {
    pub fn new(name: String, conn_tx: mpsc::UnboundedSender<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            state: DeviceState::Open,
            conn_tx,
        }
    }
}

#[derive(Debug)]
pub struct User {
    pub id: UserId,
    pub conn_tx: mpsc::UnboundedSender<String>,
}

impl User {
    pub fn new(conn_tx: mpsc::UnboundedSender<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            conn_tx,
        }
    }
}
