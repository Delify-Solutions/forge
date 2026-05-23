import { NavLink } from 'react-router-dom';
import {
    Settings,
    FolderOpen,
    Code2,
    Activity,
    Info,
    Hammer,
} from 'lucide-react';
import { cn } from '@/lib/utils';

const navItems = [
    { to: '/general', label: 'General', icon: Settings },
    { to: '/sites', label: 'Sites', icon: FolderOpen },
    { to: '/php', label: 'PHP', icon: Code2 },
    { to: '/services', label: 'Services', icon: Activity },
    { to: '/about', label: 'About', icon: Info },
];

export function Sidebar() {
    return (
        <aside className="flex w-60 flex-col border-r border-sidebar-border bg-sidebar text-sidebar-foreground">
            <div className="flex items-center gap-2 px-5 py-5">
                <Hammer className="h-5 w-5 text-sidebar-primary" />
                <span className="text-base font-semibold tracking-tight">
                    Delify Forge
                </span>
            </div>
            <nav className="flex-1 px-2">
                <ul className="space-y-1">
                    {navItems.map((item) => (
                        <li key={item.to}>
                            <NavLink
                                to={item.to}
                                className={({ isActive }) =>
                                    cn(
                                        'flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors',
                                        isActive
                                            ? 'bg-sidebar-accent text-sidebar-accent-foreground'
                                            : 'text-sidebar-foreground/70 hover:bg-sidebar-accent/50 hover:text-sidebar-foreground',
                                    )
                                }
                            >
                                <item.icon className="h-4 w-4" />
                                {item.label}
                            </NavLink>
                        </li>
                    ))}
                </ul>
            </nav>
            <div className="border-t border-sidebar-border px-5 py-3 text-xs text-muted-foreground">
                v0.0.0 · pre-MVP
            </div>
        </aside>
    );
}
