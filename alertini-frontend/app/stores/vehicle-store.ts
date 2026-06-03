import { create } from "zustand";

type Vehicle = {
  id: string;
  license_plate: string;
  car_desc: string | null;
  user_id: string;
  created_at: string | null;
};

type VehicleState = {
  vehicles: Vehicle[];
  isLoading: boolean;
  error: string | null;
  fetchVehicles: (token: string) => Promise<void>;
  createVehicle: (
    token: string,
    licensePlate: string,
    carDesc: string | null,
  ) => Promise<boolean>;
  deleteVehicle: (token: string, vehicleId: string) => Promise<boolean>;
  updateVehicle: (
    token: string,
    vehicleId: string,
    licensePlate: string,
    carDesc: string | null,
  ) => Promise<boolean>;
  clearError: () => void;
};

const apiBaseUrl = import.meta.env.VITE_API_URL ?? "http://localhost:3000";

export const useVehicleStore = create<VehicleState>((set) => ({
  vehicles: [],
  isLoading: false,
  error: null,

  fetchVehicles: async (token: string) => {
    set({ isLoading: true, error: null });
    try {
      const response = await fetch(`${apiBaseUrl}/vehicle/getall`, {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${token}`,
        },
      });

      const data = await response.json();

      if (!response.ok || !data.success) {
        throw new Error(data.message || "Failed to fetch vehicles");
      }

      set({ vehicles: data.data || [], isLoading: false });
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Failed to fetch vehicles";
      set({ error: message, isLoading: false });
    }
  },

  createVehicle: async (
    token: string,
    licensePlate: string,
    carDesc: string | null,
  ) => {
    set({ isLoading: true, error: null });
    try {
      const response = await fetch(`${apiBaseUrl}/vehicle/create`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify({
          license_plate: licensePlate,
          car_desc: carDesc,
        }),
      });

      const data = await response.json();

      if (!response.ok || !data.success) {
        throw new Error(data.message || "Failed to create vehicle");
      }

      set((state) => ({
        vehicles: [...state.vehicles, data.data],
        isLoading: false,
      }));
      return true;
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Failed to create vehicle";
      set({ error: message, isLoading: false });
      return false;
    }
  },

  deleteVehicle: async (token: string, vehicleId: string) => {
    set({ isLoading: true, error: null });
    try {
      const response = await fetch(
        `${apiBaseUrl}/vehicle/delete/${vehicleId}`,
        {
          method: "DELETE",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${token}`,
          },
        },
      );

      const data = await response.json();

      if (!response.ok || !data.success) {
        throw new Error(data.message || "Failed to delete vehicle");
      }

      set((state) => ({
        vehicles: state.vehicles.filter((v) => v.id !== vehicleId),
        isLoading: false,
      }));
      return true;
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Failed to delete vehicle";
      set({ error: message, isLoading: false });
      return false;
    }
  },

  updateVehicle: async (
    token: string,
    vehicleId: string,
    licensePlate: string,
    carDesc: string | null,
  ) => {
    set({ isLoading: true, error: null });
    try {
      const response = await fetch(
        `${apiBaseUrl}/vehicle/update/${vehicleId}`,
        {
          method: "PATCH",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${token}`,
          },
          body: JSON.stringify({
            license_plate: licensePlate,
            car_desc: carDesc,
          }),
        },
      );

      const data = await response.json();

      if (!response.ok || !data.success) {
        throw new Error(data.message || "Failed to update vehicle");
      }

      set((state) => ({
        vehicles: state.vehicles.map((v) =>
          v.id === vehicleId ? data.data : v,
        ),
        isLoading: false,
      }));
      return true;
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Failed to update vehicle";
      set({ error: message, isLoading: false });
      return false;
    }
  },

  clearError: () => set({ error: null }),
}));
