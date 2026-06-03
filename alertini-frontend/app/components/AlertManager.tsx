import { useEffect, useState } from "react";
import { useAuthStore } from "../stores/auth-store";
import { useVehicleStore } from "../stores/vehicle-store";

type Alert = {
  id: string;
  car_id: string;
  note: string;
  reporter_id: string;
};

export default function AlertManager() {
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [selectedPlate, setSelectedPlate] = useState("");
  const [alertMessage, setAlertMessage] = useState("");
  const [wsConnected, setWsConnected] = useState(false);
  const [wsRef, setWsRef] = useState<WebSocket | null>(null);
  const [error, setError] = useState<string | null>(null);
  const { token } = useAuthStore();
  const { vehicles } = useVehicleStore();

  useEffect(() => {
    if (!token) return;

    // WebSocket doesn't support custom headers natively in browser
    // The backend should either:
    // 1. Remove auth middleware from WebSocket endpoint
    // 2. Accept token as query parameter
    // For now, we'll attempt connection and send auth in first message
    const wsProtocol = import.meta.env.VITE_WS_URL?.startsWith("wss")
      ? "wss"
      : "ws";
    const wsHost = import.meta.env.VITE_WS_HOST ?? "localhost:3000";
    const wsUrl = `${wsProtocol}://${wsHost}/alert/ws`;

    const ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      setWsConnected(true);
      setError(null);
      // Send auth token as first message if backend doesn't support headers
      ws.send(
        JSON.stringify({
          action: "auth",
          token,
        }),
      );
      // attempt to subscribe to user's vehicles immediately
      if (vehicles.length > 0) {
        vehicles.forEach((v) => {
          try {
            ws.send(
              JSON.stringify({
                action: "subscribe",
                license_plate: v.license_plate,
              }),
            );
          } catch (e) {
            // ignore
          }
        });
      }
    };

    ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        if (message.type === "AlertCreated") {
          // server may send the alert as `data.alert` or directly as `data`
          const alertPayload = message.data?.alert ?? message.data;
          if (alertPayload) {
            setAlerts((prev) => [alertPayload, ...prev].slice(0, 50));
          } else {
            console.debug(
              "AlertCreated message without alert payload:",
              message,
            );
          }
        } else if (message.type === "Error") {
          setError(message.data?.message || "WebSocket error");
        } else if (message.type === "Subscribed") {
          console.debug("Subscribed to:", message.data);
        } else if (message.type === "Info") {
          console.debug("WS info:", message.data?.message ?? message.data);
        }
      } catch (e) {
        console.error("Failed to parse message:", e);
      }
    };

    ws.onerror = (event) => {
      setWsConnected(false);
      setError(
        "WebSocket connection failed - ensure backend allows connections",
      );
      console.error("WebSocket error:", event);
    };

    ws.onclose = () => {
      setWsConnected(false);
    };

    setWsRef(ws);

    return () => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.close();
      }
    };
  }, [token]);

  // When vehicles load (or change) and websocket is open, ensure subscriptions are in place
  useEffect(() => {
    if (!wsRef || wsRef.readyState !== WebSocket.OPEN) return;
    if (!vehicles || vehicles.length === 0) return;

    vehicles.forEach((v) => {
      try {
        wsRef.send(
          JSON.stringify({
            action: "subscribe",
            license_plate: v.license_plate,
          }),
        );
      } catch (e) {
        // ignore
      }
    });
  }, [wsRef, vehicles]);

  const handleSendAlert = () => {
    if (
      !selectedPlate ||
      !alertMessage ||
      !wsRef ||
      wsRef.readyState !== WebSocket.OPEN
    )
      return;

    wsRef.send(
      JSON.stringify({
        action: "alert",
        license_plate: selectedPlate,
        message: alertMessage,
      }),
    );
    setAlertMessage("");
  };

  return (
    <div className="rounded-2xl border border-white/10 bg-slate-900 p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-semibold text-white">Alerts</h2>
        <div className="flex items-center gap-2">
          <div
            className={`w-2 h-2 rounded-full ${
              wsConnected ? "bg-emerald-500" : "bg-rose-500"
            }`}
          />
          <span className="text-xs text-slate-400">
            {wsConnected ? "Connected" : "Disconnected"}
          </span>
        </div>
      </div>

      {error && (
        <div className="rounded-lg border border-rose-500/30 bg-rose-500/10 px-4 py-3 text-sm text-rose-200 mb-4">
          {error}
        </div>
      )}

      <div className="space-y-4 mb-6 pb-6 border-b border-white/10">
        <div>
          <label className="block text-sm font-medium text-slate-300 mb-2">
            License Plate (enter manually)
          </label>
          <input
            type="text"
            value={selectedPlate}
            onChange={(e) => setSelectedPlate(e.target.value)}
            className="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-2 text-white focus:border-cyan-400 focus:outline-none transition"
            placeholder="e.g., ABC123"
          />
        </div>

        <div className="grid grid-cols-1 gap-2">
          <button
            onClick={handleSendAlert}
            disabled={!selectedPlate || !wsConnected}
            className="rounded-lg bg-amber-500 px-4 py-2 text-sm font-medium text-white hover:bg-amber-600 disabled:opacity-60 transition"
          >
            Send Alert
          </button>
        </div>

        <div>
          <label className="block text-sm font-medium text-slate-300 mb-1">
            Alert Message
          </label>
          <textarea
            value={alertMessage}
            onChange={(e) => setAlertMessage(e.target.value)}
            className="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-2 text-white focus:border-cyan-400 focus:outline-none transition resize-none"
            placeholder="e.g., Suspicious activity detected"
            rows={3}
          />
        </div>
      </div>

      {/* Subscriptions are automatic for owner's vehicles; no manual subscribe UI */}

      <div>
        <h3 className="text-sm font-medium text-slate-300 mb-3">
          Recent Alerts ({alerts.length})
        </h3>
        <div className="space-y-2 max-h-48 overflow-y-auto">
          {alerts.length === 0 ? (
            <p className="text-center text-slate-400 py-4 text-xs">
              No alerts yet
            </p>
          ) : (
            alerts.map((alert) => (
              <div
                key={alert.id ?? Math.random().toString(36).slice(2, 9)}
                className="rounded-lg border border-amber-500/30 bg-amber-500/10 p-3 text-xs"
              >
                <p className="font-semibold text-amber-200">
                  {alert.note ?? JSON.stringify(alert)}
                </p>
                <p className="text-amber-300/60 mt-1">
                  Car:{" "}
                  {(alert.car_id ?? "").toString().slice(0, 8) || "unknown"}...
                </p>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
