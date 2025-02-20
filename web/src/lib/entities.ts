export type Device = {
    id: string;
    name: string;
    state: "Open" | { Connected: string };
};
