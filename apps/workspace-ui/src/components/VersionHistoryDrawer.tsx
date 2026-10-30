import React from 'react';
import { History, X, Clock, User, HardDrive, RotateCcw, CheckCircle } from 'lucide-react';

export interface VersionRecord {
  versionId: number;
  timestamp: string;
  author: string;
  summary: string;
  byteSize: number;
  blobHash: string;
}

interface VersionHistoryDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  onRestoreVersion?: (versionId: number) => void;
}

export const VersionHistoryDrawer: React.FC<VersionHistoryDrawerProps> = ({
  isOpen,
  onClose,
  onRestoreVersion,
}) => {
  if (!isOpen) return null;

  const versions: VersionRecord[] = [
    {
      versionId: 4,
      timestamp: 'Today at 18:45',
      author: 'Current User',
      summary: 'Added Phase 6 Python Studio & WPS AI Co-Pilot integration',
      byteSize: 18452,
      blobHash: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    },
    {
      versionId: 3,
      timestamp: 'Today at 16:30',
      author: 'Alice Chen',
      summary: 'Automated Z-score outlier detection and Pearson correlation formulas',
      byteSize: 16210,
      blobHash: '8f434346648f6b96df89dda901c5176b10a6d83961dd3c1ac88b59b2dc327aa4',
    },
    {
      versionId: 2,
      timestamp: 'Yesterday at 14:15',
      author: 'Bob Smith',
      summary: 'Inserted ProcessTimeline shapes and Morph transitions',
      byteSize: 14120,
      blobHash: '2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae',
    },
    {
      versionId: 1,
      timestamp: '2 days ago',
      author: 'System',
      summary: 'Initial document check-in with baseline AST nodes',
      byteSize: 10450,
      blobHash: 'fcde2b2edba56bf408601fb721fe9b5c338d10ee429ea04fae5511b68fbf8fb9',
    },
  ];

  return (
    <div className="fixed inset-y-0 right-0 w-80 bg-white dark:bg-mercury-night-raised border-l border-mercury-fog/30 shadow-2xl z-50 flex flex-col select-none animate-in slide-in-from-right-3 duration-200">
      {/* Header */}
      <div className="h-12 border-b border-mercury-fog/20 px-3 flex items-center justify-between bg-mercury-lavender/30 dark:bg-mercury-night">
        <div className="flex items-center space-x-2">
          <History className="w-4 h-4 text-mercury-primary" />
          <h2 className="text-xs font-bold text-mercury-ink dark:text-mercury-on-dark">
            File Roaming Version History
          </h2>
        </div>
        <button
          onClick={onClose}
          className="p-1 rounded-mercury-sm text-mercury-ink-secondary dark:text-mercury-fog hover:bg-mercury-lavender dark:hover:bg-mercury-night"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      {/* Subheader Info */}
      <div className="p-3 border-b border-mercury-fog/10 bg-mercury-surface-tint dark:bg-mercury-night/50 text-[11px] text-mercury-slate">
        <span>FIPS-180-4 SHA-256 deduplicated snapshot store with millisecond time-travel.</span>
      </div>

      {/* Version List */}
      <div className="flex-1 overflow-y-auto p-3 space-y-3">
        {versions.map((v, idx) => (
          <div
            key={v.versionId}
            className={`p-2.5 rounded-mercury-lg border text-xs space-y-1.5 transition-all ${
              idx === 0
                ? 'bg-mercury-primary/5 border-mercury-primary/40'
                : 'bg-white dark:bg-mercury-night border-mercury-fog/20 hover:border-mercury-primary/30'
            }`}
          >
            <div className="flex items-center justify-between">
              <span className="font-bold text-mercury-ink dark:text-mercury-on-dark flex items-center space-x-1">
                {idx === 0 && <CheckCircle className="w-3 h-3 text-mercury-sheets" />}
                <span>Version {v.versionId} {idx === 0 ? '(Current)' : ''}</span>
              </span>
              <span className="text-[10px] text-mercury-slate flex items-center space-x-1">
                <Clock className="w-3 h-3" />
                <span>{v.timestamp}</span>
              </span>
            </div>

            <p className="text-mercury-ink-secondary dark:text-mercury-fog text-[11px]">
              {v.summary}
            </p>

            <div className="flex items-center justify-between text-[10px] text-mercury-slate pt-1 border-t border-mercury-fog/10">
              <span className="flex items-center space-x-1">
                <User className="w-3 h-3" />
                <span>{v.author}</span>
              </span>
              <span className="font-mono" title={v.blobHash}>
                {v.blobHash.slice(0, 7)}...
              </span>
            </div>

            {idx !== 0 && (
              <button
                onClick={() => onRestoreVersion && onRestoreVersion(v.versionId)}
                className="w-full mt-1.5 py-1 px-2 rounded-mercury-md bg-mercury-lavender dark:bg-mercury-night-raised text-mercury-primary text-[11px] font-semibold flex items-center justify-center space-x-1 hover:bg-mercury-primary hover:text-white transition-colors"
              >
                <RotateCcw className="w-3 h-3" />
                <span>Restore This Version</span>
              </button>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};
