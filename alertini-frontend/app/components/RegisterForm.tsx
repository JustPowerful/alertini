import { useState } from "react";
import { useAuthStore } from "../stores/auth-store";

interface RegisterFormProps {
  onSwitchToLogin: () => void;
}

export default function RegisterForm({ onSwitchToLogin }: RegisterFormProps) {
  const [firstname, setFirstname] = useState("");
  const [lastname, setLastname] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [success, setSuccess] = useState(false);
  const { isLoading, error, register, clearError } = useAuthStore();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    clearError();
    const success = await register({ firstname, lastname, email, password });
    if (success) {
      setSuccess(true);
      setTimeout(() => {
        setSuccess(false);
        onSwitchToLogin();
      }, 2000);
    }
  };

  return (
    <div className="w-full max-w-md rounded-2xl border border-white/10 bg-slate-900/80 p-8 ring-1 ring-white/10">
      <div>
        <h2 className="text-2xl font-semibold text-white">Create account</h2>
        <p className="mt-2 text-sm text-slate-400">
          Sign up to get started with Alertini
        </p>
      </div>

      <form className="mt-6 space-y-4" onSubmit={handleSubmit}>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-1">
              First name
            </label>
            <input
              type="text"
              value={firstname}
              onChange={(e) => setFirstname(e.target.value)}
              className="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-2 text-white focus:border-cyan-400 focus:outline-none transition"
              placeholder="John"
              required
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-slate-300 mb-1">
              Last name
            </label>
            <input
              type="text"
              value={lastname}
              onChange={(e) => setLastname(e.target.value)}
              className="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-2 text-white focus:border-cyan-400 focus:outline-none transition"
              placeholder="Doe"
              required
            />
          </div>
        </div>

        <div>
          <label className="block text-sm font-medium text-slate-300 mb-1">
            Email
          </label>
          <input
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            className="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-2 text-white focus:border-cyan-400 focus:outline-none transition"
            placeholder="you@example.com"
            autoComplete="email"
            required
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-slate-300 mb-1">
            Password
          </label>
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            className="w-full rounded-lg border border-white/10 bg-white/5 px-4 py-2 text-white focus:border-cyan-400 focus:outline-none transition"
            placeholder="••••••••"
            autoComplete="new-password"
            required
          />
        </div>

        {error && (
          <div className="rounded-lg border border-rose-500/30 bg-rose-500/10 px-4 py-3 text-sm text-rose-200">
            {error}
          </div>
        )}

        {success && (
          <div className="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-3 text-sm text-emerald-200">
            Account created successfully! Redirecting to login...
          </div>
        )}

        <button
          type="submit"
          disabled={isLoading}
          className="w-full rounded-lg bg-cyan-400 px-4 py-2 font-medium text-slate-950 hover:bg-cyan-300 disabled:opacity-60 transition"
        >
          {isLoading ? "Creating account..." : "Sign up"}
        </button>
      </form>

      <div className="mt-6 text-center text-sm">
        <span className="text-slate-400">Already have an account? </span>
        <button
          onClick={onSwitchToLogin}
          className="text-cyan-400 hover:text-cyan-300 font-medium"
        >
          Sign in
        </button>
      </div>
    </div>
  );
}
