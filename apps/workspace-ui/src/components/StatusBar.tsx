import React from 'react';
import { ZoomIn, ZoomOut, Maximize2 } from 'lucide-react';
import { DocumentTab } from '../types';

interface StatusBarProps {
  tab: DocumentTab;
  onZoomChange: (newZoom: number) => void;
}

export const StatusBar: React.FC<StatusBarProps> = ({ tab, onZoomChange }) => {
  return (
    <footer className="h-6 bg-white dark:bg-mercury-night border-t border-mercury-fog/30 flex items-center justify-between px-3 text-[11px] text-mercury-ink-secondary dark:text-mercury-fog select-none transition-colors">
      {/* Left: Metrics */}
      <div className="flex items-center space-x-4 tabular-nums">
        {tab.type === 'writer' && (
          <>
            <span>Page {tab.currentPage || 1} of {tab.totalPages || 1}</span>
            <span>{tab.wordCount || 842} words</span>
            <span>{tab.charCount || 4820} chars</span>
            {tab.trackChanges ? (
              <span className="flex items-center space-x-1 px-1.5 py-0.5 rounded text-[10px] bg-red-100 text-red-700 dark:bg-red-950 dark:text-red-300 font-medium">
                <span className="w-1.5 h-1.5 rounded-full bg-red-500 animate-pulse" />
                <span>Track Changes: ON ({tab.pendingRevisions || 0})</span>
              </span>
            ) : (
              <span className="px-1.5 py-0.5 rounded text-[10px] text-mercury-ink-secondary dark:text-mercury-fog">
                Track Changes: Off
              </span>
            )}
            <span>English (US)</span>
          </>
        )}
        {tab.type === 'sheets' && (
          <>
            <span className="flex items-center space-x-1">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-500" />
              <span>Ready</span>
            </span>
            <span className="font-mono font-medium text-mercury-sheets">Cell: {tab.activeCell || 'B2'}</span>
            {tab.stats ? (
              <>
                <span>Average: {tab.stats.average.toLocaleString(undefined, { minimumFractionDigits: 1, maximumFractionDigits: 2 })}</span>
                <span>Count: {tab.stats.count}</span>
                <span>Sum: {tab.stats.sum.toLocaleString()}</span>
                {tab.stats.min !== 0 && <span>Min: {tab.stats.min.toLocaleString()}</span>}
                {tab.stats.max !== 0 && <span>Max: {tab.stats.max.toLocaleString()}</span>}
              </>
            ) : (
              <>
                <span>Average: 52,666.7</span>
                <span>Count: 3</span>
                <span>Sum: 158,000</span>
              </>
            )}
            {tab.readingView && (
              <span className="flex items-center space-x-1 px-1.5 py-0.5 rounded text-[10px] bg-emerald-100 text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300 font-medium">
                <span>Reading View: ON</span>
              </span>
            )}
          </>
        )}
        {tab.type === 'slides' && (
          <>
            <span>Slide {tab.currentSlide || 1} of {tab.totalSlides || 4}</span>
            <span>Theme: Mercury Modern</span>
          </>
        )}
        {tab.type === 'pdf' && (
          <>
            <span>Page {tab.currentPage || 1} of {tab.totalPages || 12}</span>
            <span>Letter (8.5 × 11 in)</span>
          </>
        )}
      </div>

      {/* Right: Zoom Controls */}
      <div className="flex items-center space-x-2">
        <button
          onClick={() => onZoomChange(Math.max(50, tab.zoom - 10))}
          className="p-0.5 hover:bg-mercury-lavender dark:hover:bg-mercury-night-raised rounded"
          title="Zoom Out"
        >
          <ZoomOut className="w-3 h-3" />
        </button>

        <input
          type="range"
          min="50"
          max="200"
          step="5"
          value={tab.zoom}
          onChange={(e) => onZoomChange(Number(e.target.value))}
          className="w-20 h-1 bg-mercury-fog/40 rounded-lg appearance-none cursor-pointer accent-mercury-primary"
        />

        <button
          onClick={() => onZoomChange(Math.min(200, tab.zoom + 10))}
          className="p-0.5 hover:bg-mercury-lavender dark:hover:bg-mercury-night-raised rounded"
          title="Zoom In"
        >
          <ZoomIn className="w-3 h-3" />
        </button>

        <span className="w-9 text-right font-mono tabular-nums">{tab.zoom}%</span>

        <button
          onClick={() => onZoomChange(100)}
          className="p-0.5 hover:bg-mercury-lavender dark:hover:bg-mercury-night-raised rounded ml-1"
          title="Reset Zoom to 100%"
        >
          <Maximize2 className="w-3 h-3" />
        </button>
      </div>
    </footer>
  );
};
