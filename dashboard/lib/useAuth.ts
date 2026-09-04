"use client";

import { useCallback, useEffect, useState } from "react";
import { fetchMe, login as apiLogin, logout as apiLogout, Me } from "./api";

export function useAuth() {
  const [me, setMe] = useState<Me | null | undefined>(undefined); // undefined = loading
  const [loginError, setLoginError] = useState<string | null>(null);
  const [loggingIn, setLoggingIn] = useState(false);

  const refresh = useCallback(async () => {
    try {
      setMe(await fetchMe());
    } catch {
      setMe(null);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const login = useCallback(async (username: string, password: string) => {
    setLoggingIn(true);
    setLoginError(null);
    try {
      const result = await apiLogin(username, password);
      setMe(result);
    } catch (e) {
      setLoginError((e as Error).message);
    } finally {
      setLoggingIn(false);
    }
  }, []);

  const logout = useCallback(async () => {
    await apiLogout();
    setMe(null);
  }, []);

  return { me, loginError, loggingIn, login, logout };
}
