import { useEffect, useState } from "react";
import type { Route } from "./+types/home";
import { useAuthStore } from "../stores/auth-store";
import { useVehicleStore } from "../stores/vehicle-store";
import LoginForm from "../components/LoginForm";
import RegisterForm from "../components/RegisterForm";
import VehicleManager from "../components/VehicleManager";
import AlertManager from "../components/AlertManager";

export function meta({}: Route.MetaArgs) {
  return [
    { title: "Alertini" },
    { name: "description", content: "Vehicle alerts dashboard" },
  ];
}

export default function Home() {
  const [mounted, setMounted] = useState(false);
  const [showRegister, setShowRegister] = useState(false);
  const { token } = useAuthStore();

  useEffect(() => {
    setMounted(true);
  }, []);

  if (!mounted) {
    return (
      <main className="min-h-screen bg-slate-950 text-white flex items-center justify-center">
        <div className="text-slate-400">Loading...</div>
      </main>
    );
  }

  if (!token) {
    return (
      <main className="min-h-screen bg-[radial-gradient(circle_at_top,_#1e293b,_#020617_55%)] text-white px-4 py-10">
        <div className="mx-auto flex min-h-[calc(100vh-5rem)] max-w-4xl items-center justify-center">
          {showRegister ? (
            <RegisterForm onSwitchToLogin={() => setShowRegister(false)} />
          ) : (
            <LoginForm onSwitchToRegister={() => setShowRegister(true)} />
          )}
        </div>
      </main>
    );
  }

  return (
    <main className="min-h-screen bg-slate-950 text-white">
      <nav className="border-b border-white/10 bg-slate-900/50 backdrop-blur">
        <div className="mx-auto max-w-6xl px-4 py-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between">
            <h1 className="text-2xl font-bold text-cyan-400">Alertini</h1>
            <button
              onClick={() => useAuthStore.getState().logout()}
              className="rounded-lg bg-slate-700 px-4 py-2 text-sm font-medium hover:bg-slate-600 transition"
            >
              Logout
            </button>
          </div>
        </div>
      </nav>

      <div className="mx-auto max-w-6xl px-4 py-8 sm:px-6 lg:px-8">
        <div className="grid gap-8 lg:grid-cols-3">
          <div className="lg:col-span-1 space-y-8">
            <VehicleManager />
            <AlertManager />
          </div>
        </div>
      </div>
    </main>
  );
}
