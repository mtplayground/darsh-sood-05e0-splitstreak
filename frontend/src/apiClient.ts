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

export type ExerciseSearchItem = {
  id: number;
  slug: string;
  name: string;
  modality: 'strength' | 'cardio';
  primary_muscle_group: string | null;
  equipment: string | null;
  aliases: string[];
  is_bodyweight: boolean;
};

export type ExerciseSearchResponse = {
  query: string;
  count: number;
  exercises: ExerciseSearchItem[];
};

export type WorkoutSession = {
  id: number;
  user_sub: string;
  started_at: string;
  completed_at: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
};

export type WorkoutSessionSummary = {
  session: WorkoutSession;
  strength_set_count: number;
  cardio_entry_count: number;
};

export type TodayDashboardResponse = {
  workout: WorkoutSessionSummary | null;
  streak: {
    status: 'pending';
    current_days: number | null;
  };
};

export type StrengthSet = {
  id: number;
  session_id: number;
  exercise_id: number;
  set_number: number;
  reps: number;
  weight_kg: number;
  created_at: string;
  updated_at: string;
};

export type CardioEntry = {
  id: number;
  session_id: number;
  exercise_id: number;
  cardio_type: string;
  duration_seconds: number;
  distance_meters: number | null;
  intensity_level: number | null;
  speed_kph: number | null;
  incline_percent: number | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
};

export type CreateSessionResponse = {
  session: WorkoutSession;
};

export type CreateWorkoutSessionPayload = {
  notes?: string;
  started_at?: string;
};

export type AddStrengthSetPayload = {
  exercise_id: number;
  set_number: number;
  reps: number;
  weight_kg: number;
};

export type AddStrengthSetResponse = {
  strength_set: StrengthSet;
};

export type AddCardioEntryPayload = {
  exercise_id: number;
  cardio_type: string;
  duration_seconds: number;
  distance_meters?: number;
  intensity_level?: number;
  speed_kph?: number;
  incline_percent?: number;
  notes?: string;
};

export type AddCardioEntryResponse = {
  cardio_entry: CardioEntry;
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

export async function searchExercises(
  query: string,
  modality: ExerciseSearchItem['modality'] = 'strength'
) {
  const params = new URLSearchParams({
    limit: '8',
    modality,
    q: query
  });

  return requestJson<ExerciseSearchResponse>(
    `/api/exercises/search?${params.toString()}`,
    {
      method: 'GET'
    }
  );
}

export async function createWorkoutSession(payload: CreateWorkoutSessionPayload = {}) {
  return requestJson<CreateSessionResponse>('/api/logging/sessions', {
    body: JSON.stringify(payload),
    method: 'POST'
  });
}

export async function fetchTodayDashboard() {
  return requestJson<TodayDashboardResponse>('/api/dashboard/today', {
    method: 'GET'
  });
}

export async function addStrengthSet(
  sessionId: number,
  payload: AddStrengthSetPayload
) {
  return requestJson<AddStrengthSetResponse>(
    `/api/logging/sessions/${sessionId}/strength-sets`,
    {
      body: JSON.stringify(payload),
      method: 'POST'
    }
  );
}

export async function addCardioEntry(
  sessionId: number,
  payload: AddCardioEntryPayload
) {
  return requestJson<AddCardioEntryResponse>(
    `/api/logging/sessions/${sessionId}/cardio-entries`,
    {
      body: JSON.stringify(payload),
      method: 'POST'
    }
  );
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
