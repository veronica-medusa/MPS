import React, { useEffect, useRef, useState } from 'react';
import { Bold, Italic, Underline, Highlighter, Copy, Sparkles, Wand2 } from 'lucide-react';
import { DocumentTab } from '../types';

interface CanvasViewportProps {
  tab: DocumentTab;
  onUpdateActiveCell?: (coord: string) => void;
  onOpenAiAssist?: (selection: string) => void;
}

export const CanvasViewport: React.FC<CanvasViewportProps> = ({ tab, onUpdateActiveCell, onOpenAiAssist }) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [selectedCell, setSelectedCell] = useState('B2');
  const [hasSelection, setHasSelection] = useState(false);
  const [selectionPos, setSelectionPos] = useState({ x: 0, y: 0 });

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Handle high DPI Retina scaling
    const dpr = window.devicePixelRatio || 1;
    const width = canvas.clientWidth;
    const height = canvas.clientHeight;
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    ctx.scale(dpr, dpr);

    // Clear canvas
    ctx.clearRect(0, 0, width, height);

    if (tab.type === 'writer') {
      // Render A4 Document Page in Canvas
      const pageWidth = 640 * (tab.zoom / 100);
      const pageHeight = 880 * (tab.zoom / 100);
      const pageX = Math.max(40, (width - pageWidth) / 2);
      const pageY = 40;

      // Page Shadow & Fill
      ctx.fillStyle = '#ffffff';
      ctx.shadowColor = 'rgba(40, 37, 59, 0.08)';
      ctx.shadowBlur = 12;
      ctx.shadowOffsetY = 4;
      ctx.fillRect(pageX, pageY, pageWidth, pageHeight);
      ctx.shadowColor = 'transparent';

      // Page Border Hairline
      ctx.strokeStyle = '#e2e4ea';
      ctx.lineWidth = 1;
      ctx.strokeRect(pageX, pageY, pageWidth, pageHeight);

      // Render Sample Document Typography
      ctx.fillStyle = '#28253b';
      ctx.font = `600 ${20 * (tab.zoom / 100)}px Inter`;
      ctx.fillText('OpenWPS Executive Specification', pageX + 50, pageY + 70);

      ctx.fillStyle = '#554e7b';
      ctx.font = `400 ${12 * (tab.zoom / 100)}px Inter`;
      ctx.fillText('Precision Office Suite • High-Performance Rust & Wasm Engine', pageX + 50, pageY + 95);

      // Paragraph lines
      ctx.fillStyle = '#28253b';
      ctx.font = `400 ${11 * (tab.zoom / 100)}px Inter`;
      const lines = [
        'OpenWPS provides bit-for-bit layout determinism across Desktop and WebAssembly targets.',
        'Document pages, spreadsheets grids, slide stages, and PDF annotations are rendered via',
        'a direct hardware-accelerated DisplayList pipeline, bypassing the browser DOM for zero layout jitter.',
        'Track Changes revisions, Mail Merge fields, and auto-generated Table of Contents are managed in Rust.',
      ];
      lines.forEach((line, i) => {
        ctx.fillText(line, pageX + 50, pageY + 140 + i * 22 * (tab.zoom / 100));
      });
    } else if (tab.type === 'sheets') {
      // Render Spreadsheet Grid
      const cellWidth = 90 * (tab.zoom / 100);
      const cellHeight = 24 * (tab.zoom / 100);
      const headerWidth = 44;
      const headerHeight = 24;

      // Top-Left corner
      ctx.fillStyle = '#f4f5f8';
      ctx.fillRect(0, 0, headerWidth, headerHeight);

      // Column Headers (A to N)
      ctx.font = '500 11px Inter';
      ctx.fillStyle = '#554e7b';
      ctx.textAlign = 'center';
      for (let col = 0; col < 15; col++) {
        const colLetter = String.fromCharCode(65 + col);
        const colX = headerWidth + col * cellWidth;
        ctx.fillStyle = '#f4f5f8';
        ctx.fillRect(colX, 0, cellWidth, headerHeight);
        ctx.strokeStyle = '#e2e4ea';
        ctx.strokeRect(colX, 0, cellWidth, headerHeight);
        ctx.fillStyle = '#554e7b';
        ctx.fillText(colLetter, colX + cellWidth / 2, 16);
      }

      // Row Headers & Grid Cells (1 to 40)
      ctx.textAlign = 'left';
      for (let row = 1; row <= 35; row++) {
        const rowY = headerHeight + (row - 1) * cellHeight;

        // Row Header
        ctx.fillStyle = '#f4f5f8';
        ctx.fillRect(0, rowY, headerWidth, cellHeight);
        ctx.strokeStyle = '#e2e4ea';
        ctx.strokeRect(0, rowY, headerWidth, cellHeight);
        ctx.fillStyle = '#554e7b';
        ctx.textAlign = 'center';
        ctx.fillText(row.toString(), headerWidth / 2, rowY + 16);
        ctx.textAlign = 'left';

        // Cells in row
        for (let col = 0; col < 15; col++) {
          const colX = headerWidth + col * cellWidth;
          ctx.strokeStyle = '#f0f1f4';
          ctx.strokeRect(colX, rowY, cellWidth, cellHeight);

          // Sample Values
          if (row === 1 && col === 0) {
            ctx.fillStyle = '#0ca678';
            ctx.fillText('Region', colX + 8, rowY + 16);
          } else if (row === 1 && col === 1) {
            ctx.fillStyle = '#0ca678';
            ctx.fillText('Q1 Revenue', colX + 8, rowY + 16);
          } else if (row === 2 && col === 0) {
            ctx.fillStyle = '#28253b';
            ctx.fillText('North America', colX + 8, rowY + 16);
          } else if (row === 2 && col === 1) {
            ctx.fillStyle = '#28253b';
            ctx.fillText('$1,420,500', colX + 8, rowY + 16);
          } else if (row === 3 && col === 0) {
            ctx.fillStyle = '#28253b';
            ctx.fillText('EMEA', colX + 8, rowY + 16);
          } else if (row === 3 && col === 1) {
            ctx.fillStyle = '#28253b';
            ctx.fillText('$985,200', colX + 8, rowY + 16);
          }
        }
      }

      // Active Selected Cell Outline (Emerald #0ca678)
      const selCol = 1;
      const selRow = 2;
      const selX = headerWidth + selCol * cellWidth;
      const selY = headerHeight + (selRow - 1) * cellHeight;
      ctx.strokeStyle = '#0ca678';
      ctx.lineWidth = 2;
      ctx.strokeRect(selX, selY, cellWidth, cellHeight);
      ctx.fillStyle = '#0ca678';
      ctx.fillRect(selX + cellWidth - 5, selY + cellHeight - 5, 5, 5); // Drag handle
      ctx.lineWidth = 1;
    } else if (tab.type === 'slides') {
      // Render 16:9 Slide Canvas Stage
      const slideW = 760 * (tab.zoom / 100);
      const slideH = 427.5 * (tab.zoom / 100);
      const slideX = Math.max(40, (width - slideW) / 2);
      const slideY = 40;

      ctx.fillStyle = '#ffffff';
      ctx.shadowColor = 'rgba(40, 37, 59, 0.12)';
      ctx.shadowBlur = 16;
      ctx.shadowOffsetY = 6;
      ctx.fillRect(slideX, slideY, slideW, slideH);
      ctx.shadowColor = 'transparent';

      // Slide Title & Content
      ctx.fillStyle = '#e8590c';
      ctx.font = `700 ${28 * (tab.zoom / 100)}px Inter`;
      ctx.fillText('WPS Presentation Engine', slideX + 60, slideY + 100);

      ctx.fillStyle = '#554e7b';
      ctx.font = `400 ${14 * (tab.zoom / 100)}px Inter`;
      ctx.fillText('3-Tier Master Slide Inheritance & DrawingML Connectors', slideX + 60, slideY + 140);
    } else if (tab.type === 'pdf') {
      // Render PDF Page
      const pdfW = 600 * (tab.zoom / 100);
      const pdfH = 800 * (tab.zoom / 100);
      const pdfX = Math.max(40, (width - pdfW) / 2);
      const pdfY = 40;

      ctx.fillStyle = '#ffffff';
      ctx.shadowColor = 'rgba(40, 37, 59, 0.1)';
      ctx.shadowBlur = 14;
      ctx.fillRect(pdfX, pdfY, pdfW, pdfH);
      ctx.shadowColor = 'transparent';

      ctx.fillStyle = '#e03131';
      ctx.font = `600 ${18 * (tab.zoom / 100)}px Inter`;
      ctx.fillText('ISO 32000 Conforming PDF Document', pdfX + 50, pdfY + 70);

      // Reconstructed Paragraph Box Demo
      ctx.strokeStyle = 'rgba(224, 49, 49, 0.3)';
      ctx.setLineDash([4, 2]);
      ctx.strokeRect(pdfX + 45, pdfY + 95, pdfW - 90, 80);
      ctx.setLineDash([]);

      ctx.fillStyle = '#28253b';
      ctx.font = `400 ${12 * (tab.zoom / 100)}px Inter`;
      ctx.fillText('[Editable Paragraph Box Reconstructed via DBSCAN]', pdfX + 50, pdfY + 120);
      ctx.fillText('Click to edit text directly inline with font-matching reflow.', pdfX + 50, pdfY + 145);
    }
  }, [tab]);

  return (
    <div className="relative flex-1 bg-mercury-canvas dark:bg-mercury-night overflow-auto flex items-center justify-center p-4 transition-colors">
      <canvas
        ref={canvasRef}
        className="w-full h-full cursor-text"
        onClick={(e) => {
          setHasSelection(true);
          setSelectionPos({ x: e.clientX, y: e.clientY });
        }}
      />

      {/* Floating Mercury Micro-Toolbar */}
      {hasSelection && (
        <div
          className="absolute z-40 bg-white dark:bg-mercury-night-raised border border-mercury-fog/30 rounded-mercury-pill shadow-xl px-2 py-1 flex items-center space-x-1 animate-in fade-in zoom-in-95"
          style={{
            top: Math.max(10, selectionPos.y - 120),
            left: Math.max(20, selectionPos.x - 100),
          }}
        >
          <button className="p-1 hover:bg-mercury-lavender dark:hover:bg-mercury-night rounded-full text-mercury-ink dark:text-mercury-on-dark">
            <Bold className="w-3.5 h-3.5" />
          </button>
          <button className="p-1 hover:bg-mercury-lavender dark:hover:bg-mercury-night rounded-full text-mercury-ink dark:text-mercury-on-dark">
            <Italic className="w-3.5 h-3.5" />
          </button>
          <button className="p-1 hover:bg-mercury-lavender dark:hover:bg-mercury-night rounded-full text-mercury-ink dark:text-mercury-on-dark">
            <Highlighter className="w-3.5 h-3.5 text-yellow-500" />
          </button>
          <div className="w-[1px] h-3.5 bg-mercury-fog/30 mx-0.5" />
          <button
            onClick={() => onOpenAiAssist && onOpenAiAssist('selected text')}
            className="flex items-center space-x-1 px-2 py-0.5 rounded-full bg-mercury-ai/10 text-mercury-ai hover:bg-mercury-ai/20 text-xs font-medium"
          >
            <Sparkles className="w-3 h-3" />
            <span>AI Assist</span>
          </button>
        </div>
      )}
    </div>
  );
};
