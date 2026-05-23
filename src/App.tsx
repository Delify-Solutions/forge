import { Routes, Route, Navigate } from 'react-router-dom';
import { Sidebar } from '@/components/Sidebar';
import { General } from '@/pages/General';
import { Sites } from '@/pages/Sites';
import { Php } from '@/pages/Php';
import { Services } from '@/pages/Services';
import { About } from '@/pages/About';

export function App() {
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
                    <Route path="/services" element={<Services />} />
                    <Route path="/about" element={<About />} />
                </Routes>
            </main>
        </div>
    );
}
