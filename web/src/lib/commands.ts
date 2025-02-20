import { Device } from "~/src/lib/entities";

interface UpdateDevices {
    type: "UpdateDevices";
    devices: Device[];
}

interface RemoveDevice {
    type: "RemoveDevice";
    device_id: string;
}

interface UpdateConnectedDevice {
    type: "UpdateConnectedDevice";
    device: Device;
}

interface RemoveConnectedDevice {
    type: "RemoveConnectedDevice";
    device_id: string;
}

interface DeviceSignaling {
    type: "DeviceSignaling";
    device_id: string;
    signal: RTCSessionDescription | RTCIceCandidate;
}

interface Error {
    type: "Error";
    message: string;
}

export type UserCommand =
    | UpdateDevices
    | RemoveDevice
    | UpdateConnectedDevice
    | RemoveConnectedDevice
    | DeviceSignaling
    | Error;

interface Connect {
    type: "Connect";
    device_id: string;
}

interface Disconnect {
    type: "Disconnect";
    device_id: string;
}

interface UserSignaling {
    type: "UserSignaling";
    device_id: string;
    signal: RTCSessionDescription | RTCIceCandidate;
}

export type ServerCommand = Connect | Disconnect | UserSignaling;
