"use client";

import * as React from "react";
import { SquareTerminal } from "lucide-react";

import {
    Sidebar,
    SidebarHeader,
    SidebarContent,
    SidebarFooter,
    SidebarRail,
} from "~/src/components/ui/sidebar";

import NavHeader from "~/src/components/nav-header";
import NavMain from "~/src/components/nav-main";
import NavDevices from "~/src/components/nav-devices";

const data = {
    appName: "Iris",
    appIcon: "/img/branding_avatar.png",
    navMain: [
        {
            title: "Playground",
            url: "/",
            icon: SquareTerminal,
            isActive: true,
            items: [
                {
                    title: "Controls",
                    url: "/",
                },
                {
                    title: "Devices",
                    url: "/devices",
                },
            ],
        },
    ],
};

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
    return (
        <Sidebar collapsible="icon" {...props}>
            <SidebarHeader>
                <NavHeader appIcon={data.appIcon} appName={data.appName} />
            </SidebarHeader>
            <SidebarContent>
                <NavMain items={data.navMain} />
            </SidebarContent>
            <SidebarFooter>
                <NavDevices />
            </SidebarFooter>
            <SidebarRail />
        </Sidebar>
    );
}
