import React from 'react';

import { getUserFacingErrorMessage } from './apiClient';
import { AuthProvider } from './auth/AuthContext';
import { useAuth } from './auth/useAuth';
import { HistoryScreen } from './HistoryScreen';
import { HomeDashboard } from './HomeDashboard';
import { LogScreen } from './logging/LogScreen';
import { SplitsLibrary } from './SplitsLibrary';
import { SyncStatusIndicator } from './SyncStatus';

type AuthPanel = 'login' | 'register';
type AuthenticatedView = 'home' | 'log' | 'history' | 'splits';

export function App() {
  return (
    <AuthProvider>
      <AuthGate />
    </AuthProvider>
  );
}

function AuthGate() {
  const auth = useAuth();
  const [panel, setPanel] = React.useState<AuthPanel>('login');
  const [email, setEmail] = React.useState('');
  const [formMessage, setFormMessage] = React.useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = React.useState(false);

  if (auth.status === 'loading') {
    return (
      <main className="shell shell--centered">
        <section className="loading-panel" aria-live="polite">
          <span className="loader" aria-hidden="true" />
          <p>Opening SplitStreak...</p>
        </section>
      </main>
    );
  }

  if (auth.status === 'authenticated') {
    return <AuthenticatedApp />;
  }

  async function handleRegister() {
    setIsSubmitting(true);
    setFormMessage(null);
    try {
      await auth.register();
      setFormMessage('Registration is complete. Checking your session...');
      await auth.refresh();
    } catch (error) {
      setFormMessage(getUserFacingErrorMessage(error, 'Registration failed'));
    } finally {
      setIsSubmitting(false);
    }
  }

  async function handlePasswordReset(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsSubmitting(true);
    setFormMessage(null);
    try {
      await auth.requestPasswordReset(email);
      setFormMessage(
        'If that email matches an account, a recovery link is on the way.'
      );
      setEmail('');
    } catch (error) {
      setFormMessage(getUserFacingErrorMessage(error, 'Recovery request failed'));
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <main className="auth-layout">
      <section className="auth-hero" aria-labelledby="app-title">
        <div className="brand-row">
          <div className="mark" aria-hidden="true">
            SS
          </div>
          <p className="eyebrow">SplitStreak</p>
        </div>
        <h1 id="app-title">Keep your split moving.</h1>
        <p className="summary">
          Sign in to track workouts, keep streaks honest around rest days, and keep your
          training history synced as the app grows.
        </p>
      </section>

      <section className="auth-panel" aria-labelledby="auth-heading">
        <div
          className="segmented-control"
          role="tablist"
          aria-label="Authentication screens"
        >
          <button
            aria-selected={panel === 'login'}
            className={panel === 'login' ? 'segment segment--active' : 'segment'}
            onClick={() => {
              setPanel('login');
              setFormMessage(null);
            }}
            role="tab"
            type="button"
          >
            Login
          </button>
          <button
            aria-selected={panel === 'register'}
            className={panel === 'register' ? 'segment segment--active' : 'segment'}
            onClick={() => {
              setPanel('register');
              setFormMessage(null);
            }}
            role="tab"
            type="button"
          >
            Register
          </button>
        </div>

        {panel === 'login' ? (
          <div className="auth-form" role="tabpanel">
            <h2 id="auth-heading">Welcome back</h2>
            <p>
              Continue through secure sign-in. SplitStreak uses the platform session
              cookie, so there is no app password to manage here.
            </p>
            <button className="primary-action" onClick={auth.signIn} type="button">
              Continue to SplitStreak
            </button>
            <form className="inline-form" onSubmit={handlePasswordReset}>
              <label htmlFor="reset-email">Recover account access</label>
              <div className="input-row">
                <input
                  autoComplete="email"
                  id="reset-email"
                  inputMode="email"
                  onChange={(event) => setEmail(event.target.value)}
                  placeholder="you@example.com"
                  type="email"
                  value={email}
                />
                <button disabled={isSubmitting} type="submit">
                  Send
                </button>
              </div>
            </form>
          </div>
        ) : (
          <div className="auth-form" role="tabpanel">
            <h2 id="auth-heading">Create your account</h2>
            <p>
              Registration starts with secure sign-in, then SplitStreak stores your
              training profile in its own Postgres database.
            </p>
            <button
              className="primary-action"
              disabled={isSubmitting}
              onClick={handleRegister}
              type="button"
            >
              Start registration
            </button>
          </div>
        )}

        {auth.error && <p className="form-message form-message--error">{auth.error}</p>}
        {formMessage && <p className="form-message">{formMessage}</p>}
      </section>
    </main>
  );
}

function AuthenticatedApp() {
  const auth = useAuth();
  const [view, setView] = React.useState<AuthenticatedView>('home');
  const [message, setMessage] = React.useState<string | null>(null);
  const [isBusy, setIsBusy] = React.useState(false);
  const user = auth.user;

  if (!user) {
    return null;
  }

  async function handleVerificationEmail() {
    setIsBusy(true);
    setMessage(null);
    try {
      const response = await auth.sendVerification();
      if (response.delivery.status === 'rate_limited') {
        setMessage('Verification email is rate limited. Try again shortly.');
      } else if (response.delivery.status === 'sent') {
        setMessage('Verification email sent.');
      } else {
        setMessage(response.delivery.reason);
      }
    } catch (error) {
      setMessage(getUserFacingErrorMessage(error, 'Verification email failed'));
    } finally {
      setIsBusy(false);
    }
  }

  async function handleConfirmVerification() {
    setIsBusy(true);
    setMessage(null);
    try {
      const response = await auth.confirmVerification();
      setMessage(
        response.email_verified
          ? 'Email verified.'
          : 'Email verification is still pending.'
      );
      await auth.refresh();
    } catch (error) {
      setMessage(getUserFacingErrorMessage(error, 'Verification check failed'));
    } finally {
      setIsBusy(false);
    }
  }

  return (
    <main className="app-shell">
      <header className="top-bar">
        <div className="brand-row">
          <div className="mark mark--small" aria-hidden="true">
            SS
          </div>
          <div>
            <p className="eyebrow">SplitStreak</p>
            <h1>{viewTitle(view)}</h1>
          </div>
        </div>
        <nav className="app-nav" aria-label="Primary">
          <button
            aria-current={view === 'home' ? 'page' : undefined}
            className={view === 'home' ? 'nav-button nav-button--active' : 'nav-button'}
            onClick={() => setView('home')}
            type="button"
          >
            Home
          </button>
          <button
            aria-current={view === 'log' ? 'page' : undefined}
            className={view === 'log' ? 'nav-button nav-button--active' : 'nav-button'}
            onClick={() => setView('log')}
            type="button"
          >
            Log
          </button>
          <button
            aria-current={view === 'history' ? 'page' : undefined}
            className={
              view === 'history' ? 'nav-button nav-button--active' : 'nav-button'
            }
            onClick={() => setView('history')}
            type="button"
          >
            History
          </button>
          <button
            aria-current={view === 'splits' ? 'page' : undefined}
            className={
              view === 'splits' ? 'nav-button nav-button--active' : 'nav-button'
            }
            onClick={() => setView('splits')}
            type="button"
          >
            Splits
          </button>
        </nav>
        <SyncStatusIndicator userSub={user.sub} />
        {user.picture_url ? (
          <img alt="" className="avatar" src={user.picture_url} />
        ) : (
          <div className="avatar avatar--fallback" aria-hidden="true">
            {initials(user.name ?? user.email)}
          </div>
        )}
      </header>

      {view === 'home' && (
        <HomeDashboard onQuickStart={() => setView('log')} userSub={user.sub} />
      )}
      {view === 'log' && <LogScreen userSub={user.sub} />}
      {view === 'history' && <HistoryScreen />}
      {view === 'splits' && <SplitsLibrary onStartLogging={() => setView('log')} />}

      <section
        className="profile-panel account-strip"
        aria-labelledby="profile-heading"
      >
        <div>
          <p className="eyebrow">Signed in</p>
          <h2 id="profile-heading">{user.name ?? user.email}</h2>
          <p>{user.email}</p>
        </div>
        <span
          className={user.email_verified ? 'badge badge--good' : 'badge badge--warn'}
        >
          {user.email_verified ? 'Email verified' : 'Verification pending'}
        </span>
      </section>

      <section className="action-grid account-actions" aria-label="Account actions">
        <button
          disabled={isBusy || user.email_verified}
          onClick={handleVerificationEmail}
          type="button"
        >
          Send verification email
        </button>
        <button disabled={isBusy} onClick={handleConfirmVerification} type="button">
          Check verification
        </button>
      </section>

      {message && <p className="form-message">{message}</p>}
    </main>
  );
}

function viewTitle(view: AuthenticatedView) {
  if (view === 'log') {
    return 'Log workout';
  }

  if (view === 'history') {
    return 'History';
  }

  if (view === 'splits') {
    return 'Splits';
  }

  return 'Dashboard';
}

function initials(value: string) {
  return value
    .split(/\s+|@/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase() ?? '')
    .join('');
}
