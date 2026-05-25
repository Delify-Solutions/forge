import { useEffect, useState } from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { Sidebar } from '@/components/Sidebar';
import { FirstRunWizard } from '@/components/Wizard/FirstRunWizard';
import { General } from '@/pages/General';
import { Sites } from '@/pages/Sites';
import { Php } from '@/pages/Php';
import { Services } from '@/pages/Services';
import { About } from '@/pages/About';
import { tauri } from '@/lib/tauri';

export function App() {
    const [wizardOpen, setWizardOpen] = useState(true);

    useEffect(() => {
        const onKey = (e: KeyboardEvent) => {
            const isMac = navigator.platform.toLowerCase().includes('mac');
            const mod = isMac ? e.metaKey && e.altKey : e.ctrlKey && e.shiftKey;
            if (mod && (e.key === 'i' || e.key === 'I')) {
                e.preventDefault();
                void tauri.openDevtools();
            }
        };
        window.addEventListener('keydown', onKey);
        return () => window.removeEventListener('keydown', onKey);
    }, []);

    const openWizard = () => setWizardOpen(true);

    return (
        <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground">
            <Sidebar />
            <main className="flex-1 overflow-auto p-6">
                <Routes>
                    <Route
                        path="/"
                        element={<Navigate to="/sites" replace />}
                    />
                    <Route path="/general" element={<General />} />
                    <Route path="/sites" element={<Sites />} />
                    <Route path="/php" element={<Php />} />
                    <Route
                        path="/services"
                        element={<Services onOpenWizard={openWizard} />}
                    />
                    <Route path="/about" element={<About />} />
                </Routes>
            </main>
            <FirstRunWizard
                open={wizardOpen}
                onComplete={() => setWizardOpen(false)}
            />
        </div>
    );
}
