import React from 'react';
import { createRoot } from 'react-dom/client';
import './styles.css';

type HealthState =
  | { status: 'loading' }
  | { status: 'ready'; service: string }
  | { status: 'error'; message: string };

function App() {
  const [health, setHealth] = React.useState<HealthState>({ status: 'loading' });

  React.useEffect(() => {
    let isMounted = true;

    async function loadHealth() {
      try {
        const response = await fetch('/api/health', {
          headers: { Accept: 'application/json' }
        });

        if (!response.ok) {
          throw new Error(`Health check failed with ${response.status}`);
        }

        const payload = (await response.json()) as { status?: string; service?: string };
        if (payload.status !== 'ok' || typeof payload.service !== 'string') {
          throw new Error('Health check response was malformed');
        }

        if (isMounted) {
          setHealth({ status: 'ready', service: payload.service });
        }
      } catch (error) {
        if (isMounted) {
          const message = error instanceof Error ? error.message : 'Health check failed';
          setHealth({ status: 'error', message });
        }
      }
    }

    void loadHealth();

    return () => {
      isMounted = false;
    };
  }, []);

  return (
    <main className="shell">
      <section className="intro" aria-labelledby="app-title">
        <div className="mark" aria-hidden="true">
          SS
        </div>
        <div>
          <p className="eyebrow">Workout logging PWA</p>
          <h1 id="app-title">SplitStreak</h1>
          <p className="summary">
            A React SPA and Rust API foundation for tracking sessions, splits, streaks,
            and workout history.
          </p>
        </div>
      </section>

      <section className="status-panel" aria-live="polite">
        <span className={`status-dot status-dot--${health.status}`} />
        <div>
          <h2>API health</h2>
          {health.status === 'loading' && <p>Checking backend availability...</p>}
          {health.status === 'ready' && <p>{health.service} is responding.</p>}
          {health.status === 'error' && <p>{health.message}</p>}
        </div>
      </section>
    </main>
  );
}

const rootElement = document.getElementById('root');

if (!rootElement) {
  throw new Error('Root element was not found');
}

createRoot(rootElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
