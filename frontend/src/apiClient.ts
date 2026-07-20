export type AuthUser = {
  sub: string;
  email: string;
  email_verified: boolean;
  name: string | null;
  picture_url: string | null;
};

export type EmailDelivery =
  | { status: 'sent'; message_id: string }
  | { status: 'skipped'; reason: string }
  | { status: 'rate_limited' };

export type SessionResponse = {
  status: 'authenticated';
  session: 'mctai_session';
  user: AuthUser;
};

export type RegistrationResponse = {
  status: 'registered';
  user: AuthUser;
  email: EmailDelivery;
};

export type VerificationEmailResponse = {
  status: 'verification_email_processed';
  email_verified: boolean;
  delivery: EmailDelivery;
};

export type VerificationConfirmResponse = {
  status: 'verified' | 'pending';
  email_verified: boolean;
};

export type PasswordResetResponse = {
  status: 'accepted';
  message: string;
};

export class ApiError extends Error {
  readonly loginUrl?: string;
  readonly status: number;

  constructor(status: number, message: string, loginUrl?: string) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.loginUrl = loginUrl;
  }
}

export function redirectToLogin(loginUrl?: string) {
  window.location.assign(loginUrl ?? '/api/auth/login');
}

export async function fetchSession() {
  return requestJson<SessionResponse>('/api/auth/login', {
    method: 'POST'
  });
}

export async function registerAccount() {
  return requestJson<RegistrationResponse>('/api/auth/register', {
    method: 'POST'
  });
}

export async function sendVerificationEmail() {
  return requestJson<VerificationEmailResponse>('/api/auth/email-verification', {
    method: 'POST'
  });
}

export async function confirmEmailVerification() {
  return requestJson<VerificationConfirmResponse>(
    '/api/auth/email-verification/confirm',
    {
      method: 'POST'
    }
  );
}

export async function requestPasswordReset(email: string) {
  return requestJson<PasswordResetResponse>('/api/auth/password-reset', {
    body: JSON.stringify({ email }),
    method: 'POST'
  });
}

async function requestJson<T>(path: string, init: RequestInit): Promise<T> {
  const response = await fetch(path, {
    ...init,
    credentials: 'include',
    headers: {
      Accept: 'application/json',
      'Content-Type': 'application/json',
      ...init.headers
    }
  });

  const payload = await readJson(response);

  if (!response.ok) {
    const errorPayload = payload as Partial<{
      error: string;
      login_url: string;
    }> | null;
    throw new ApiError(
      response.status,
      errorPayload?.error ?? `Request failed with ${response.status}`,
      errorPayload?.login_url
    );
  }

  return payload as T;
}

async function readJson(response: Response) {
  const contentType = response.headers.get('content-type') ?? '';
  if (!contentType.includes('application/json')) {
    return null;
  }

  return response.json();
}
