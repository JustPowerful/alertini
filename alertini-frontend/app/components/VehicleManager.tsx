import { useEffect, useState } from "react";
import { useAuthStore } from "../stores/auth-store";
import { useVehicleStore } from "../stores/vehicle-store";

export default function VehicleManager() {
  const [licensePlate, setLicensePlate] = useState("");
  const [carDesc, setCarDesc] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const { token } = useAuthStore();
  const {
    vehicles,
    isLoading,
    error,
    fetchVehicles,
    createVehicle,
    deleteVehicle,
    updateVehicle,
    clearError,
  } = useVehicleStore();

  useEffect(() => {
    if (token) {
      fetchVehicles(token);
    }
  }, [token, fetchVehicles]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    clearError();

    if (!token) return;

    if (editingId) {
      const success = await updateVehicle(
        token,
        editingId,
        licensePlate,
        carDesc || null,
      );
      if (success) {
        setLicensePlate("");
        setCarDesc("");
        setEditingId(null);
      }
    } else {
      const success = await createVehicle(token, licensePlate, carDesc || null);
      if (success) {
        setLicensePlate("");
        setCarDesc("");
      }
    }
  };

  const handleEdit = (
    vehicleId: string,
    plate: string,
    desc: string | null,
  ) => {
    setEditingId(vehicleId);
    setLicensePlate(plate);
    setCarDesc(desc || "");
  };

  const handleDelete = async (vehicleId: string) => {
    if (!token) return;
    if (confirm("Are you sure you want to delete this vehicle?")) {
      await deleteVehicle(token, vehicleId);
    }
  };

  const handleCancel = () => {
    setEditingId(null);
    setLicensePlate("");
    setCarDesc("");
    clearError();
  };

  return (
    <div className="rounded-2xl border border-white/10 bg-slate-900 p-6">
      <h2 className="text-xl font-semibold text-white mb-6">Vehicles</h2>

      <form
        onSubmit={handleSubmit}
        className="space-y-4 mb-6 pb-6 border-b border-white/10"
      >
        <div>
          <label className="block text-sm font-medium text-slate-300 mb-1">
            License Plate
          </label>
          <input
            type="text"
            value={licensePlate}
            onChange={(e) => setLicensePlate(e.target.value)}
            className="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-2 text-white focus:border-cyan-400 focus:outline-none transition"
            placeholder="ABC123"
            required
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-slate-300 mb-1">
            Car Description
          </label>
          <input
            type="text"
            value={carDesc}
            onChange={(e) => setCarDesc(e.target.value)}
            className="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-2 text-white focus:border-cyan-400 focus:outline-none transition"
            placeholder="e.g., Red Toyota Camry"
          />
        </div>

        {error && (
          <div className="rounded-lg border border-rose-500/30 bg-rose-500/10 px-4 py-3 text-sm text-rose-200">
            {error}
          </div>
        )}

        <div className="flex gap-2">
          <button
            type="submit"
            disabled={isLoading}
            className="flex-1 rounded-lg bg-cyan-400 px-4 py-2 font-medium text-slate-950 hover:bg-cyan-300 disabled:opacity-60 transition"
          >
            {isLoading
              ? "Processing..."
              : editingId
                ? "Update Vehicle"
                : "Add Vehicle"}
          </button>
          {editingId && (
            <button
              type="button"
              onClick={handleCancel}
              className="flex-1 rounded-lg border border-white/20 px-4 py-2 font-medium text-slate-300 hover:bg-white/5 transition"
            >
              Cancel
            </button>
          )}
        </div>
      </form>

      <div className="space-y-2 max-h-96 overflow-y-auto">
        {vehicles.length === 0 ? (
          <p className="text-center text-slate-400 py-8">
            No vehicles yet. Add one above.
          </p>
        ) : (
          vehicles.map((vehicle) => (
            <div
              key={vehicle.id}
              className="rounded-lg border border-white/10 bg-white/5 p-4 flex items-start justify-between hover:bg-white/10 transition"
            >
              <div className="flex-1">
                <p className="font-semibold text-white">
                  {vehicle.license_plate}
                </p>
                {vehicle.car_desc && (
                  <p className="text-sm text-slate-400">{vehicle.car_desc}</p>
                )}
              </div>
              <div className="flex gap-2">
                <button
                  onClick={() =>
                    handleEdit(
                      vehicle.id,
                      vehicle.license_plate,
                      vehicle.car_desc,
                    )
                  }
                  className="px-3 py-1 text-xs rounded bg-blue-500/20 text-blue-300 hover:bg-blue-500/30 transition"
                >
                  Edit
                </button>
                <button
                  onClick={() => handleDelete(vehicle.id)}
                  className="px-3 py-1 text-xs rounded bg-rose-500/20 text-rose-300 hover:bg-rose-500/30 transition"
                >
                  Delete
                </button>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
