import React from 'react';

import {
  ApiError,
  confirmEmailVerification,
  fetchSession,
  redirectToLogin,
  registerAccount,
  requestPasswordReset as requestPasswordResetEmail,
  sendVerificationEmail
} from '../apiClient';
import {
  AuthContext,
  type AuthContextValue,
  type AuthStatus,
  type AuthUser
} from './authState';

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [status, setStatus] = React.useState<AuthStatus>('loading');
  const [user, setUser] = React.useState<AuthUser | null>(null);
  const [error, setError] = React.useState<string | null>(null);

  const refresh = React.useCallback(async () => {
    setError(null);
    try {
      const session = await fetchSession();
      setUser(session.user);
      setStatus('authenticated');
    } catch (caught) {
      if (caught instanceof ApiError && caught.status === 401) {
        setUser(null);
        setStatus('unauthenticated');
        return;
      }

      setUser(null);
      setStatus('unauthenticated');
      setError(caught instanceof Error ? caught.message : 'Session check failed');
    }
  }, []);

  React.useEffect(() => {
    void refresh();
  }, [refresh]);

  const value = React.useMemo<AuthContextValue>(
    () => ({
      status,
      user,
      error,
      refresh,
      signIn() {
        redirectToLogin();
      },
      async register() {
        try {
          const registration = await registerAccount();
          setUser(registration.user);
          setStatus('authenticated');
        } catch (caught) {
          if (caught instanceof ApiError && caught.status === 401) {
            redirectToLogin(caught.loginUrl);
            return;
          }

          throw caught;
        }
      },
      async requestPasswordReset(email: string) {
        await requestPasswordResetEmail(email);
      },
      async sendVerification() {
        try {
          return await sendVerificationEmail();
        } catch (caught) {
          if (caught instanceof ApiError && caught.status === 401) {
            redirectToLogin(caught.loginUrl);
          }

          throw caught;
        }
      },
      async confirmVerification() {
        try {
          return await confirmEmailVerification();
        } catch (caught) {
          if (caught instanceof ApiError && caught.status === 401) {
            redirectToLogin(caught.loginUrl);
          }

          throw caught;
        }
      }
    }),
    [error, refresh, status, user]
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
