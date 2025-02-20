"use client";

import { ReactNode } from "react";

import {
    ColumnDef,
    flexRender,
    getCoreRowModel,
    useReactTable,
} from "@tanstack/react-table";

import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "~/src/components/ui/table";

import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from "~/src/components/ui/dropdown-menu";

import { Badge } from "~/src/components/ui/badge";
import { Device } from "~/src/lib/entities";
import { MoreHorizontal } from "lucide-react";
import { Button } from "~/src/components/ui/button";
import { useIris } from "~/src/contexts/iris-context";

interface DataTableProps<TData, TValue> {
    columns: ColumnDef<TData, TValue>[];
    data: TData[];
}

const ConnectButton = ({
    deviceId,
    children,
}: { deviceId: string; children: ReactNode }) => {
    const { sendCommand } = useIris();
    return (
        <button
            className="flex flex-grow"
            onClick={() =>
                sendCommand({ type: "Connect", device_id: deviceId })
            }
        >
            {children}
        </button>
    );
};

const DisconnectButton = ({
    deviceId,
    children,
}: { deviceId: string; children: ReactNode }) => {
    const { sendCommand } = useIris();
    return (
        <button
            className="flex flex-grow"
            onClick={() =>
                sendCommand({ type: "Disconnect", device_id: deviceId })
            }
        >
            {children}
        </button>
    );
};

export const columns: ColumnDef<Device>[] = [
    {
        accessorKey: "id",
        header: "ID",
    },
    {
        accessorKey: "name",
        header: "Name",
    },
    {
        accessorKey: "state",
        header: "State",
        cell: ({ row }) => {
            const state: string | object = row.getValue("state");
            const formatted: string =
                typeof state === "string" ? state : Object.keys(state)[0];

            return <Badge variant="outline">{formatted}</Badge>;
        },
    },
    {
        header: "Actions",
        id: "actions",
        enableHiding: false,
        cell: ({ row }) => {
            const device = row.original;

            return (
                <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                        <Button variant="ghost" className="h-8 w-8 p-0">
                            <span className="sr-only">Open menu</span>
                            <MoreHorizontal />
                        </Button>
                    </DropdownMenuTrigger>

                    <DropdownMenuContent align="end">
                        <DropdownMenuLabel>Actions</DropdownMenuLabel>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem>
                            <ConnectButton deviceId={String(device.id)}>
                                Connect
                            </ConnectButton>
                        </DropdownMenuItem>
                        <DropdownMenuItem>
                            <DisconnectButton deviceId={String(device.id)}>
                                Disconnect
                            </DisconnectButton>
                        </DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>
            );
        },
    },
];

export const DevicesTable = <TData, TValue>({
    columns,
    data,
}: DataTableProps<TData, TValue>) => {
    const table = useReactTable({
        data,
        columns,
        getCoreRowModel: getCoreRowModel(),
    });

    return (
        <div className="rounded-md border">
            <Table>
                <TableHeader>
                    {table.getHeaderGroups().map(headerGroup => (
                        <TableRow key={headerGroup.id}>
                            {headerGroup.headers.map(header => {
                                return (
                                    <TableHead key={header.id}>
                                        {header.isPlaceholder
                                            ? null
                                            : flexRender(
                                                header.column.columnDef
                                                    .header,
                                                header.getContext(),
                                            )}
                                    </TableHead>
                                );
                            })}
                        </TableRow>
                    ))}
                </TableHeader>
                <TableBody>
                    {table.getRowModel()?.rows?.length ? (
                        table.getRowModel().rows.map(row => (
                            <TableRow
                                key={row.id}
                                data-state={row.getIsSelected() && "selected"}
                            >
                                {row.getVisibleCells().map(cell => (
                                    <TableCell key={cell.id}>
                                        {flexRender(
                                            cell.column.columnDef.cell,
                                            cell.getContext(),
                                        )}
                                    </TableCell>
                                ))}
                            </TableRow>
                        ))
                    ) : (
                        <TableRow>
                            <TableCell
                                colSpan={columns.length}
                                className="h-24 text-center"
                            >
                                No results.
                            </TableCell>
                        </TableRow>
                    )}
                </TableBody>
            </Table>
        </div>
    );
};
