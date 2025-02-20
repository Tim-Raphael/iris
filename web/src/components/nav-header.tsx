import {
    Avatar,
    AvatarImage,
    AvatarFallback,
} from "~/src/components/ui/avatar";

export default function NavHeader({
    appIcon,
    appName,
}: { appIcon: string; appName: string }) {
    return (
        <Avatar className="h-8 w-8">
            <AvatarImage src={appIcon} alt={appName} />
            <AvatarFallback>{appName}</AvatarFallback>
        </Avatar>
    );
}
