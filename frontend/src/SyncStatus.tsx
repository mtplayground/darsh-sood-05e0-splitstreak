import React from 'react';

import { redirectIfAuthError } from './apiClient';
import { getLocalSyncSnapshot } from './localStore';
import { syncQueuedMutations, syncQueueStatusEvent } from './syncQueue';

type SyncStatusIndicatorProps = {
  userSub: string;
};

type SyncEventDetail = {
  status: 'idle' | 'syncing';
  userSub: string;
};

export function SyncStatusIndicator({ userSub }: SyncStatusIndicatorProps) {
  const [isOnline, setIsOnline] = React.useState(readOnline);
  const [isSyncing, setIsSyncing] = React.useState(false);
  const [snapshot, setSnapshot] = React.useState(() => getLocalSyncSnapshot(userSub));

  const refresh = React.useCallback(() => {
    setIsOnline(readOnline());
    setSnapshot(getLocalSyncSnapshot(userSub));
  }, [userSub]);

  React.useEffect(() => {
    refresh();

    function handleOnlineStateChange() {
      refresh();
    }

    function handleSyncEvent(event: Event) {
      const detail = (event as CustomEvent<SyncEventDetail>).detail;
      if (detail?.userSub !== userSub) {
        return;
      }

      setIsSyncing(detail.status === 'syncing');
      refresh();
    }

    window.addEventListener('online', handleOnlineStateChange);
    window.addEventListener('offline', handleOnlineStateChange);
    window.addEventListener('splitstreak-local-workouts-updated', refresh);
    window.addEventListener(syncQueueStatusEvent, handleSyncEvent);

    return () => {
      window.removeEventListener('online', handleOnlineStateChange);
      window.removeEventListener('offline', handleOnlineStateChange);
      window.removeEventListener('splitstreak-local-workouts-updated', refresh);
      window.removeEventListener(syncQueueStatusEvent, handleSyncEvent);
    };
  }, [refresh, userSub]);

  React.useEffect(() => {
    if (!isOnline || isSyncing || snapshot.pendingCount === 0) {
      return;
    }

    void syncQueuedMutations(userSub).catch(redirectIfAuthError);
  }, [isOnline, isSyncing, snapshot.pendingCount, userSub]);

  const status = getDisplayStatus(isOnline, isSyncing, snapshot);

  return (
    <div
      className={`sync-indicator sync-indicator--${status.tone}`}
      aria-live="polite"
      aria-label={status.label}
    >
      <span className="sync-indicator__dot" aria-hidden="true" />
      <span>{status.label}</span>
    </div>
  );
}

function getDisplayStatus(
  isOnline: boolean,
  isSyncing: boolean,
  snapshot: ReturnType<typeof getLocalSyncSnapshot>
) {
  if (!isOnline) {
    return {
      label:
        snapshot.pendingCount > 0
          ? `Offline - ${snapshot.pendingCount} pending`
          : 'Offline',
      tone: 'offline'
    };
  }

  if (isSyncing) {
    return {
      label: 'Syncing',
      tone: 'syncing'
    };
  }

  if (snapshot.pendingCount > 0) {
    return {
      label: `${snapshot.pendingCount} pending`,
      tone: 'pending'
    };
  }

  if (snapshot.failedCount > 0) {
    return {
      label: `${snapshot.failedCount} failed`,
      tone: 'failed'
    };
  }

  return {
    label: 'Synced',
    tone: 'synced'
  };
}

function readOnline() {
  return typeof navigator === 'undefined' ? true : navigator.onLine;
}
