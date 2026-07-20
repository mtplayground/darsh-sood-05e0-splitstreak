import React from 'react';

import type {
  AuthUser,
  VerificationConfirmResponse,
  VerificationEmailResponse
} from '../apiClient';

export type { AuthUser } from '../apiClient';

export type AuthStatus = 'loading' | 'authenticated' | 'unauthenticated';

export type AuthContextValue = {
  status: AuthStatus;
  user: AuthUser | null;
  error: string | null;
  refresh: () => Promise<void>;
  signIn: () => void;
  register: () => Promise<void>;
  requestPasswordReset: (email: string) => Promise<void>;
  sendVerification: () => Promise<VerificationEmailResponse>;
  confirmVerification: () => Promise<VerificationConfirmResponse>;
};

export const AuthContext = React.createContext<AuthContextValue | null>(null);
