import { useCallback } from "react";

// Detect if we're running inside Tauri
const isTauri = typeof window !== "undefined" && "__TAURI__" in window;

export function useTauri() {
  const invoke = useCallback(async (command, args) => {
    if (!isTauri) {
      // Mock responses for browser preview
      console.log(`[Tauri mock] invoke("${command}", ${JSON.stringify(args)})`);
      if (command === "get_state") return { connected: false, effect: "Static" };
      return null;
    }
    try {
      const { invoke: tauriInvoke } = await import("@tauri-apps/api/tauri");
      return await tauriInvoke(command, args);
    } catch (e) {
      console.error(`Tauri invoke error (${command}):`, e);
      throw e;
    }
  }, []);

  return { invoke, isAvailable: isTauri };
}
