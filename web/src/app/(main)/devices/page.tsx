"use client";

import { DevicesTable, columns } from "~/src/components/devices-table";
import Header from "~/src/components/header";
import { useIris } from "~/src/contexts/iris-context";

export default function Home() {
    const { devices, connectedDevices } = useIris();
    const tableData = connectedDevices.concat(devices);

    return (
        <>
            <Header title={"Devices"} />
            <main className="flex flex-1 flex-col gap-4 p-4 pt-0">
                <DevicesTable columns={columns} data={tableData} />
            </main>
        </>
    );
}
