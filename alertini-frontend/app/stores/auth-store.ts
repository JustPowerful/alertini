import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

type LoginCredentials = {
  email: string;
  password: string;
};

type RegisterCredentials = {
  firstname: string;
  lastname: string;
  email: string;
  password: string;
};

type AuthState = {
  token: string | null;
  isLoading: boolean;
  error: string | null;
  login: (credentials: LoginCredentials) => Promise<boolean>;
  register: (credentials: RegisterCredentials) => Promise<boolean>;
  logout: () => void;
  clearError: () => void;
};

const apiBaseUrl = import.meta.env.VITE_API_URL ?? "http://localhost:3000";

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      token: null,
      isLoading: false,
      error: null,

      login: async ({ email, password }) => {
        set({ isLoading: true, error: null });
        try {
          const response = await fetch(`${apiBaseUrl}/auth/login`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ email, password }),
          });

          const data = await response.json();

          if (!response.ok || !data.success || !data.data?.token) {
            throw new Error(data.message || "Login failed");
          }

          set({ token: data.data.token, isLoading: false, error: null });
          return true;
        } catch (error) {
          const message =
            error instanceof Error ? error.message : "Login failed";
          set({ isLoading: false, error: message });
          return false;
        }
      },

      register: async ({ firstname, lastname, email, password }) => {
        set({ isLoading: true, error: null });
        try {
          const response = await fetch(`${apiBaseUrl}/auth/register`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              firstname,
              lastname,
              email,
              password,
            }),
          });

          const data = await response.json();

          if (!response.ok || !data.success) {
            throw new Error(data.message || "Registration failed");
          }

          set({ isLoading: false, error: null });
          return true;
        } catch (error) {
          const message =
            error instanceof Error ? error.message : "Registration failed";
          set({ isLoading: false, error: message });
          return false;
        }
      },

      logout: () => set({ token: null, error: null }),
      clearError: () => set({ error: null }),
    }),
    {
      name: "alertini-auth",
      storage: createJSONStorage(() => ({
        getItem: (name: string) => {
          if (typeof window === "undefined") return null;
          return window.localStorage.getItem(name);
        },
        setItem: (name: string, value: string) => {
          if (typeof window === "undefined") return;
          window.localStorage.setItem(name, value);
        },
        removeItem: (name: string) => {
          if (typeof window === "undefined") return;
          window.localStorage.removeItem(name);
        },
      })),
    },
  ),
);
