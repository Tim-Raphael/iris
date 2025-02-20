"use client";

import React, {
    createContext,
    useContext,
    useEffect,
    useRef,
    useState,
    RefObject,
    ReactNode,
} from "react";

import { Device } from "~/src/lib/entities";
import { UserCommand, ServerCommand } from "~/src/lib/commands";

interface IrisContextType {
    sendCommand: (command: ServerCommand) => void;
    setActiveDevice: (device: Device) => void;
    ondevicesignaling: RefObject<(signal: RTCSessionDescription | RTCIceCandidate) => void>;
    devices: Device[];
    connectedDevices: Device[];
    activeDevice: Device | null;
}

const IrisContext = createContext<IrisContextType | null>(null);

export function useIris() {
    const context = useContext(IrisContext);
    if (!context) {
        throw new Error("useWebSocket must be used within a WebSocketProvider");
    }
    return context;
}

export function IrisProvider({
    path,
    children,
}: { path: string; children: ReactNode }) {
    const [devices, setDevices] = useState<IrisContextType["devices"]>([]);
    const [connectedDevices, setConnectedDevices] =
        useState<IrisContextType["connectedDevices"]>([]);
    const [activeDevice, setActiveDevice] =
        useState<IrisContextType["activeDevice"]>(null);
    const ondevicesignaling =
        useRef<(signal: RTCSessionDescription | RTCIceCandidate) => void>((): void => { });
    const ws = useRef<WebSocket | null>(null);

    function sendCommand(cmd: ServerCommand) {
        if (ws.current?.readyState === WebSocket.OPEN) {
            ws.current.send(JSON.stringify(cmd));
        } else {
            console.warn("WebSocket is not open. Command not sent.");
        }
    }

    function parseCommand(commandString: string) {
        const cmd: UserCommand = JSON.parse(commandString);
        console.log(cmd);

        switch (cmd.type) {
            case "DeviceSignaling":
                if (!cmd.signal) return;
                ondevicesignaling.current(cmd.signal);
                break;

            case "UpdateDevices":
                setDevices(devices => {
                    cmd.devices.forEach(updatedDevice => {
                        const index = devices.findIndex(
                            device => device.id === updatedDevice.id,
                        );

                        if (index !== -1) devices[index] = updatedDevice;
                        else devices.push(updatedDevice);
                    });

                    return [...devices];
                });
                break;

            case "RemoveDevice":
                setDevices(devices =>
                    devices.filter(device => device.id !== cmd.device_id),
                );
                break;

            case "UpdateConnectedDevice":
                setConnectedDevices(devices => {
                    const index = devices.findIndex(
                        device => device.id === cmd.device.id,
                    );

                    if (index !== -1) devices[index] = cmd.device;
                    else devices.push(cmd.device);

                    return [...devices];
                });

                if (!activeDevice) setActiveDevice(() => cmd.device);
                break;

            case "RemoveConnectedDevice":
                setConnectedDevices(connectedDevices =>
                    connectedDevices.filter(
                        device => device.id !== cmd.device_id,
                    ),
                );

                setActiveDevice(activeDevice => {
                    if (activeDevice)
                        return cmd.device_id !== activeDevice.id
                            ? activeDevice
                            : null;
                    else return null;
                });
                break;

            case "Error":
                break;
            default:
                console.error("Could not parse command");
        }
    }

    useEffect(() => {
        if (!process.env.BASE_API_URL) {
            console.error(
                "BASE_API_URL is not set in environment variables.",
            );
            return;
        }

        ws.current = new WebSocket(`ws://${process.env.BASE_API_URL}${path}`);
        ws.current.onmessage = event => parseCommand(event.data);

        return () => { ws.current?.close() };
    }, []);

    return (
        <IrisContext.Provider
            value={{
                sendCommand,
                setActiveDevice,
                ondevicesignaling,
                devices,
                connectedDevices,
                activeDevice,
            }}
        >
            {children}
        </IrisContext.Provider>
    );
}
