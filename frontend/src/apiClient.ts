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

export type StreakSummary = {
  status: 'active' | 'no_active_split';
  current_days: number;
  current_streak_started_on: string | null;
  last_missed_training_day: string | null;
  today: string;
  today_schedule_item: string | null;
  today_is_training_day: boolean | null;
};

export type TodayDashboardResponse = {
  workout: WorkoutSessionSummary | null;
  streak: StreakSummary;
};

export type StreakCalendarDayStatus =
  'before_active_split' | 'logged' | 'missed' | 'no_active_split' | 'pending' | 'rest';

export type StreakCalendarDay = {
  date: string;
  schedule_item: string | null;
  is_training_day: boolean | null;
  logged: boolean;
  status: StreakCalendarDayStatus;
};

export type StreakActiveSplit = {
  template_slug: string;
  template_name: string;
  depth_level: SplitDepthLevel;
  schedule: string[];
  selected_on: string;
};

export type StreakCalendarResponse = {
  summary: StreakSummary;
  active_split: StreakActiveSplit | null;
  days: StreakCalendarDay[];
};

export type SplitDepthLevel = 'simple' | 'advanced';

export type SplitTemplateItem = {
  id: number;
  slug: string;
  name: string;
  depth_level: SplitDepthLevel;
  schedule: string[];
  training_days_per_cycle: number;
  rest_days_per_cycle: number;
  rationale: string;
};

export type SplitsLibraryResponse = {
  count: number;
  templates: SplitTemplateItem[];
};

export type ActiveSplit = {
  user_sub: string;
  split_template_id: number;
  template_slug: string;
  template_name: string;
  depth_level: SplitDepthLevel;
  schedule: string[];
  training_days_per_cycle: number;
  rest_days_per_cycle: number;
  selected_at: string;
  updated_at: string;
};

export type ActiveSplitResponse = {
  active_split: ActiveSplit | null;
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

export type HistoryStrengthSet = {
  id: number;
  exercise_id: number;
  exercise_slug: string;
  exercise_name: string;
  set_number: number;
  reps: number;
  weight_kg: number;
  created_at: string;
  updated_at: string;
};

export type HistoryCardioEntry = {
  id: number;
  exercise_id: number;
  exercise_slug: string;
  exercise_name: string;
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

export type HistorySession = {
  id: number;
  user_sub: string;
  date: string;
  started_at: string;
  completed_at: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
  strength_sets: HistoryStrengthSet[];
  cardio_entries: HistoryCardioEntry[];
};

export type HistoryPage = {
  limit: number;
  offset: number;
  has_more: boolean;
  next_offset: number | null;
};

export type HistorySessionsResponse = {
  page: HistoryPage;
  sessions: HistorySession[];
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

export type SyncSessionPayload = {
  client_id: string;
  started_at?: string;
  completed_at?: string | null;
  notes?: string | null;
};

export type SyncStrengthSetPayload = AddStrengthSetPayload & {
  client_id: string;
  client_session_id: string;
};

export type SyncCardioEntryPayload = AddCardioEntryPayload & {
  client_id: string;
  client_session_id: string;
};

export type SyncBatchPayload = {
  sessions: SyncSessionPayload[];
  strength_sets: SyncStrengthSetPayload[];
  cardio_entries: SyncCardioEntryPayload[];
};

export type SyncBatchResponse = {
  status: 'synced';
  sessions: Array<{
    client_id: string;
    server_id: number;
    session: WorkoutSession;
    status: 'synced';
  }>;
  strength_sets: Array<{
    client_id: string;
    server_id: number;
    strength_set: StrengthSet;
    status: 'synced';
  }>;
  cardio_entries: Array<{
    client_id: string;
    server_id: number;
    cardio_entry: CardioEntry;
    status: 'synced';
  }>;
};

export class ApiError extends Error {
  readonly code: string;
  readonly loginUrl?: string;
  readonly rawMessage: string | null;
  readonly status: number;

  constructor(
    status: number,
    message: string,
    loginUrl?: string,
    code = 'request_failed',
    rawMessage: string | null = null
  ) {
    super(message);
    this.name = 'ApiError';
    this.code = code;
    this.status = status;
    this.loginUrl = loginUrl;
    this.rawMessage = rawMessage;
  }
}

export function redirectToLogin(loginUrl?: string) {
  window.location.assign(loginUrl ?? '/api/auth/login');
}

export function isAuthenticationError(caught: unknown): caught is ApiError {
  return caught instanceof ApiError && caught.status === 401;
}

export function isNetworkError(caught: unknown): caught is ApiError {
  return caught instanceof ApiError && caught.status === 0;
}

export function redirectIfAuthError(caught: unknown) {
  if (!isAuthenticationError(caught)) {
    return false;
  }

  redirectToLogin(caught.loginUrl);
  return true;
}

export function getUserFacingErrorMessage(caught: unknown, fallback: string) {
  if (caught instanceof ApiError) {
    return caught.message;
  }

  return fallback;
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

export async function fetchStreakCalendar(days = 35) {
  const params = new URLSearchParams({ days: days.toString() });

  return requestJson<StreakCalendarResponse>(`/api/streak?${params.toString()}`, {
    method: 'GET'
  });
}

export async function fetchHistorySessions(limit = 10, offset = 0) {
  const params = new URLSearchParams({
    limit: limit.toString(),
    offset: offset.toString()
  });

  return requestJson<HistorySessionsResponse>(
    `/api/history/sessions?${params.toString()}`,
    {
      method: 'GET'
    }
  );
}

export async function fetchSplitTemplates(depth: SplitDepthLevel) {
  const params = new URLSearchParams({
    depth,
    limit: '50'
  });

  return requestJson<SplitsLibraryResponse>(
    `/api/splits/templates?${params.toString()}`,
    {
      method: 'GET'
    }
  );
}

export async function fetchActiveSplit() {
  return requestJson<ActiveSplitResponse>('/api/splits/active', {
    method: 'GET'
  });
}

export async function selectActiveSplit(splitTemplateSlug: string) {
  return requestJson<ActiveSplitResponse>('/api/splits/active', {
    body: JSON.stringify({ split_template_slug: splitTemplateSlug }),
    method: 'PUT'
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

export async function syncOfflineBatch(payload: SyncBatchPayload) {
  return requestJson<SyncBatchResponse>('/api/sync/reconcile', {
    body: JSON.stringify(payload),
    method: 'POST'
  });
}

async function requestJson<T>(path: string, init: RequestInit): Promise<T> {
  let response: Response;
  try {
    response = await fetch(path, {
      ...init,
      credentials: 'include',
      headers: {
        Accept: 'application/json',
        'Content-Type': 'application/json',
        ...init.headers
      }
    });
  } catch {
    throw new ApiError(
      0,
      'Connection lost. Your changes are saved locally and will sync when you are back online.',
      undefined,
      'network_error'
    );
  }

  const payload = await readJson(response);

  if (!response.ok) {
    throw buildApiError(response.status, payload);
  }

  return payload as T;
}

async function readJson(response: Response) {
  const contentType = response.headers.get('content-type') ?? '';
  if (!contentType.includes('application/json')) {
    return null;
  }

  try {
    return await response.json();
  } catch {
    return null;
  }
}

function buildApiError(status: number, payload: unknown) {
  const errorPayload = payload as Partial<{
    code: string;
    error: string;
    login_url: string;
    message: string;
  }> | null;
  const rawMessage =
    typeof errorPayload?.message === 'string'
      ? errorPayload.message
      : typeof errorPayload?.error === 'string'
        ? errorPayload.error
        : null;
  const code =
    typeof errorPayload?.code === 'string'
      ? errorPayload.code
      : normalizeErrorCode(errorPayload?.error);

  return new ApiError(
    status,
    friendlyErrorMessage(status, code, rawMessage),
    errorPayload?.login_url,
    code,
    rawMessage
  );
}

function normalizeErrorCode(error: unknown) {
  if (typeof error !== 'string' || error.trim().length === 0) {
    return 'request_failed';
  }

  return error
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '_')
    .replace(/^_+|_+$/g, '');
}

function friendlyErrorMessage(status: number, code: string, rawMessage: string | null) {
  if (status === 401) {
    return 'Sign in to continue.';
  }
  if (status === 403) {
    return 'You do not have access to that action.';
  }
  if (status === 404) {
    return 'That workout data was not found. Refresh and try again.';
  }
  if (status === 429) {
    return 'Too many requests. Try again shortly.';
  }
  if (status >= 500) {
    return 'Something went wrong on our side. Try again shortly.';
  }

  if (code.includes('sync')) {
    return 'Some saved workout data could not sync. Check the entry and try again.';
  }
  if (code.includes('logging')) {
    return 'Check the workout details and try again.';
  }

  return rawMessage && rawMessage.length <= 140
    ? rawMessage
    : 'Check the request and try again.';
}
