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
            <span>{tab.wordCount || 84} words</span>
            <span>English (US)</span>
          </>
        )}
        {tab.type === 'sheets' && (
          <>
            <span>Ready</span>
            <span className="font-mono">Cell: {tab.activeCell || 'B2'}</span>
            <span>Average: $1,202,850</span>
            <span>Count: 2</span>
            <span>Sum: $2,405,700</span>
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
