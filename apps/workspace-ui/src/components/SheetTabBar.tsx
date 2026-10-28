import React, { useState } from 'react';
import { Plus, ChevronLeft, ChevronRight, MoreVertical } from 'lucide-react';
import { SheetInfo } from '../types';

interface SheetTabBarProps {
  sheets: SheetInfo[];
  activeSheetIndex: number;
  onSelectSheet: (index: number) => void;
  onAddSheet: () => void;
  onRenameSheet: (index: number, newName: string) => void;
  onDeleteSheet?: (index: number) => void;
}

export const SheetTabBar: React.FC<SheetTabBarProps> = ({
  sheets,
  activeSheetIndex,
  onSelectSheet,
  onAddSheet,
  onRenameSheet,
  onDeleteSheet,
}) => {
  const [editingIndex, setEditingIndex] = useState<number | null>(null);
  const [editName, setEditName] = useState('');

  const handleStartRename = (index: number, name: string) => {
    setEditingIndex(index);
    setEditName(name);
  };

  const handleSaveRename = (index: number) => {
    if (editName.trim()) {
      onRenameSheet(index, editName.trim());
    }
    setEditingIndex(null);
  };

  return (
    <div className="h-7 bg-mercury-canvas dark:bg-mercury-night-raised border-t border-mercury-fog/30 flex items-center justify-between px-2 select-none text-xs">
      {/* Left: Navigation & Sheet Tabs */}
      <div className="flex items-center space-x-1 overflow-x-auto scrollbar-none py-0.5">
        {/* Scroll Buttons */}
        <div className="flex items-center space-x-0.5 text-mercury-ink-secondary dark:text-mercury-fog mr-1">
          <button className="p-0.5 rounded hover:bg-mercury-lavender dark:hover:bg-mercury-night" title="Scroll Left">
            <ChevronLeft className="w-3.5 h-3.5" />
          </button>
          <button className="p-0.5 rounded hover:bg-mercury-lavender dark:hover:bg-mercury-night" title="Scroll Right">
            <ChevronRight className="w-3.5 h-3.5" />
          </button>
        </div>

        {/* Tab Items */}
        {sheets.map((sheet, idx) => {
          const isActive = idx === activeSheetIndex;
          return (
            <div
              key={sheet.id || idx}
              onClick={() => onSelectSheet(idx)}
              onDoubleClick={() => handleStartRename(idx, sheet.name)}
              className={`h-6 px-3 flex items-center space-x-1.5 rounded-t cursor-pointer border-t-2 transition-all ${
                isActive
                  ? 'bg-white dark:bg-mercury-night border-mercury-sheets text-mercury-sheets font-medium shadow-sm'
                  : 'bg-transparent border-transparent text-mercury-ink-secondary dark:text-mercury-fog hover:bg-mercury-lavender/50 dark:hover:bg-mercury-night'
              }`}
            >
              {editingIndex === idx ? (
                <input
                  type="text"
                  value={editName}
                  autoFocus
                  onChange={(e) => setEditName(e.target.value)}
                  onBlur={() => handleSaveRename(idx)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') handleSaveRename(idx);
                    if (e.key === 'Escape') setEditingIndex(null);
                  }}
                  className="w-24 h-4 px-1 text-xs bg-white dark:bg-mercury-night border border-mercury-sheets rounded outline-none"
                />
              ) : (
                <span className="truncate max-w-[120px]">{sheet.name}</span>
              )}

              {/* Sheet count indicator if available */}
              {sheet.rows && sheet.rows > 1 && (
                <span className="text-[9px] opacity-60">({sheet.rows}r)</span>
              )}
            </div>
          );
        })}

        {/* Add Sheet Button */}
        <button
          onClick={onAddSheet}
          className="p-1 rounded-full text-mercury-sheets hover:bg-mercury-sheets/10 transition-colors ml-1"
          title="Add New Worksheet (+)"
        >
          <Plus className="w-3.5 h-3.5" />
        </button>
      </div>

      {/* Right side helper info */}
      <div className="flex items-center space-x-2 text-[10px] text-mercury-ink-secondary dark:text-mercury-fog font-mono pr-2">
        <span>Sheet {activeSheetIndex + 1} of {sheets.length}</span>
      </div>
    </div>
  );
};
