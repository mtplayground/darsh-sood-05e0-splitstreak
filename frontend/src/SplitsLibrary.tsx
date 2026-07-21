import React from 'react';

import {
  fetchActiveSplit,
  fetchSplitTemplates,
  getUserFacingErrorMessage,
  redirectIfAuthError,
  selectActiveSplit,
  type ActiveSplit,
  type SplitDepthLevel,
  type SplitTemplateItem
} from './apiClient';

type SplitsLibraryProps = {
  onStartLogging: () => void;
};

export function SplitsLibrary({ onStartLogging }: SplitsLibraryProps) {
  const [depth, setDepth] = React.useState<SplitDepthLevel>('simple');
  const [templates, setTemplates] = React.useState<SplitTemplateItem[]>([]);
  const [activeSplit, setActiveSplit] = React.useState<ActiveSplit | null>(null);
  const [isLoading, setIsLoading] = React.useState(true);
  const [isSelectingSlug, setIsSelectingSlug] = React.useState<string | null>(null);
  const [message, setMessage] = React.useState<string | null>(null);

  const loadSplits = React.useCallback(async () => {
    setIsLoading(true);
    setMessage(null);
    try {
      const [libraryResponse, activeResponse] = await Promise.all([
        fetchSplitTemplates(depth),
        fetchActiveSplit()
      ]);
      setTemplates(libraryResponse.templates);
      setActiveSplit(activeResponse.active_split);
    } catch (caught) {
      if (redirectIfAuthError(caught)) {
        return;
      }

      setMessage(getUserFacingErrorMessage(caught, 'Splits could not load.'));
    } finally {
      setIsLoading(false);
    }
  }, [depth]);

  React.useEffect(() => {
    void loadSplits();
  }, [loadSplits]);

  async function handleSelect(template: SplitTemplateItem) {
    setIsSelectingSlug(template.slug);
    setMessage(null);
    try {
      const response = await selectActiveSplit(template.slug);
      setActiveSplit(response.active_split);
      setMessage(`${template.name} is now your active split.`);
    } catch (caught) {
      if (redirectIfAuthError(caught)) {
        return;
      }

      setMessage(getUserFacingErrorMessage(caught, 'Active split could not be saved.'));
    } finally {
      setIsSelectingSlug(null);
    }
  }

  return (
    <section className="splits-layout" aria-labelledby="splits-heading">
      <div className="splits-library">
        <div className="splits-toolbar">
          <div className="section-heading">
            <p className="eyebrow">Split library</p>
            <h2 id="splits-heading">Choose training depth</h2>
          </div>
          <DepthToggle depth={depth} onChange={setDepth} />
        </div>

        {activeSplit && (
          <div className="active-split-strip" aria-live="polite">
            <div>
              <p className="dashboard-kicker">Active split</p>
              <strong>{activeSplit.template_name}</strong>
            </div>
            <span className="badge badge--good">
              {formatDepth(activeSplit.depth_level)}
            </span>
          </div>
        )}

        {isLoading ? (
          <p className="empty-state" aria-live="polite">
            Loading splits...
          </p>
        ) : (
          <div className="split-card-grid">
            {templates.map((template) => {
              const isActive = activeSplit?.template_slug === template.slug;
              return (
                <article className="split-card" key={template.slug}>
                  <div className="split-card__header">
                    <div>
                      <p className="dashboard-kicker">
                        {template.training_days_per_cycle} training /{' '}
                        {template.rest_days_per_cycle} rest
                      </p>
                      <h3>{template.name}</h3>
                    </div>
                    <span className="badge">{formatDepth(template.depth_level)}</span>
                  </div>

                  <p>{template.rationale}</p>

                  <ol
                    className="split-schedule"
                    aria-label={`${template.name} schedule`}
                  >
                    {template.schedule.map((day, index) => (
                      <li key={`${template.slug}-${index}`}>
                        <span>Day {index + 1}</span>
                        <strong>{day}</strong>
                      </li>
                    ))}
                  </ol>

                  <button
                    className={
                      isActive ? 'split-select split-select--active' : 'split-select'
                    }
                    disabled={isSelectingSlug !== null || isActive}
                    onClick={() => void handleSelect(template)}
                    type="button"
                  >
                    {isActive
                      ? 'Active split'
                      : isSelectingSlug === template.slug
                        ? 'Saving...'
                        : 'Set active split'}
                  </button>
                </article>
              );
            })}
          </div>
        )}

        {templates.length === 0 && !isLoading && (
          <p className="empty-state">No splits are available at this depth.</p>
        )}
        {message && <p className="form-message">{message}</p>}
      </div>

      <aside className="split-guidance" aria-labelledby="split-guidance-heading">
        <div className="section-heading">
          <p className="eyebrow">{formatDepth(depth)}</p>
          <h2 id="split-guidance-heading">Guidance</h2>
        </div>
        <p>{guidanceCopy[depth]}</p>
        <button className="primary-action" onClick={onStartLogging} type="button">
          Start logging
        </button>
      </aside>
    </section>
  );
}

function DepthToggle({
  depth,
  onChange
}: {
  depth: SplitDepthLevel;
  onChange: (depth: SplitDepthLevel) => void;
}) {
  return (
    <div className="segmented-control splits-depth-toggle" aria-label="Split depth">
      <button
        aria-pressed={depth === 'simple'}
        className={depth === 'simple' ? 'segment segment--active' : 'segment'}
        onClick={() => onChange('simple')}
        type="button"
      >
        Simple
      </button>
      <button
        aria-pressed={depth === 'advanced'}
        className={depth === 'advanced' ? 'segment segment--active' : 'segment'}
        onClick={() => onChange('advanced')}
        type="button"
      >
        Advanced
      </button>
    </div>
  );
}

function formatDepth(depth: SplitDepthLevel) {
  return depth === 'simple' ? 'Simple' : 'Advanced';
}

const guidanceCopy: Record<SplitDepthLevel, string> = {
  simple:
    'Simple splits keep the weekly pattern easy to repeat, with fewer moving parts and enough rest to make missed days easier to recover from.',
  advanced:
    'Advanced splits spread work across more specific training days, giving experienced lifters more room for volume, exercise variety, and recovery planning.'
};
