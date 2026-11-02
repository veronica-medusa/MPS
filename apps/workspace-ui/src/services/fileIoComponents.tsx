import React, { useState, useRef, useCallback } from 'react';
import {
  UploadCloud,
  FileText,
  Table,
  Presentation,
  FileCheck,
  CheckCircle2,
  AlertCircle,
  X,
  Download,
} from 'lucide-react';
import {
  OfficeFileFormat,
  OfficeDocumentPayload,
  parseOfficeFileBytes,
} from './fileIo';

// ============================================================================
// MERCURY FILE DROPZONE COMPONENT
// ============================================================================

export interface MercuryFileDropzoneProps {
  onFileLoaded: (payload: OfficeDocumentPayload) => void;
  className?: string;
  isCompact?: boolean;
}

export const MercuryFileDropzone: React.FC<MercuryFileDropzoneProps> = ({
  onFileLoaded,
  className = '',
  isCompact = false,
}) => {
  const [isDragOver, setIsDragOver] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [successName, setSuccessName] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const handleProcessFile = useCallback(
    async (file: File) => {
      setIsProcessing(true);
      setErrorMsg(null);
      setSuccessName(null);

      try {
        const buffer = await file.arrayBuffer();
        const uint8 = new Uint8Array(buffer);
        const payload = await parseOfficeFileBytes(file.name, uint8);
        setSuccessName(file.name);
        onFileLoaded(payload);
      } catch (err: any) {
        console.error('File parsing error:', err);
        setErrorMsg(err?.message || 'Failed to parse file.');
      } finally {
        setIsProcessing(false);
      }
    },
    [onFileLoaded]
  );

  const handleDrop = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    setIsDragOver(false);
    if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
      handleProcessFile(e.dataTransfer.files[0]);
    }
  };

  const handleDragOver = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    setIsDragOver(true);
  };

  const handleDragLeave = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    setIsDragOver(false);
  };

  const handleClick = () => {
    if (inputRef.current) {
      inputRef.current.value = '';
      inputRef.current.click();
    }
  };

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length > 0) {
      handleProcessFile(e.target.files[0]);
    }
  };

  return (
    <div
      onDrop={handleDrop}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onClick={handleClick}
      className={`relative group cursor-pointer transition-all duration-200 select-none ${
        isDragOver
          ? 'bg-[#edecfb] border-[#5266eb] shadow-md scale-[1.005]'
          : 'bg-[#fbfcfd] hover:bg-[#f6f5fb] border-[#e2e1ec]'
      } border-2 border-dashed rounded-2xl flex flex-col items-center justify-center text-center ${
        isCompact ? 'p-6' : 'p-10'
      } ${className}`}
    >
      <input
        type="file"
        ref={inputRef}
        onChange={handleInputChange}
        className="hidden"
        accept=".docx,.xlsx,.csv,.pptx,.pdf"
      />

      {/* Center Icon */}
      <div
        className={`w-14 h-14 rounded-full flex items-center justify-center mb-4 transition-transform duration-200 group-hover:scale-105 ${
          isDragOver
            ? 'bg-[#5266eb] text-white'
            : 'bg-[#edecfb] text-[#5266eb]'
        }`}
      >
        {isProcessing ? (
          <div className="w-6 h-6 border-2 border-white/30 border-t-white rounded-full animate-spin" />
        ) : successName ? (
          <CheckCircle2 className="w-7 h-7 text-emerald-600" />
        ) : (
          <UploadCloud className="w-7 h-7" />
        )}
      </div>

      {/* Main Copy */}
      <h3 className="text-base font-semibold text-[#171721] mb-1">
        {successName ? `Loaded ${successName}` : 'Drop documents here or click to browse'}
      </h3>
      <p className="text-xs text-[#706e82] mb-4 max-w-sm">
        Supports native Microsoft Word (.docx), Excel (.xlsx), CSV, PowerPoint (.pptx), and PDF documents in pure memory.
      </p>

      {/* Format Pills */}
      <div className="flex flex-wrap items-center justify-center gap-2 mb-2">
        <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-[11px] font-medium bg-[#edecfb] text-[#5266eb]">
          <FileText className="w-3.5 h-3.5" /> .DOCX
        </span>
        <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-[11px] font-medium bg-[#eef7ee] text-emerald-700">
          <Table className="w-3.5 h-3.5" /> .XLSX / .CSV
        </span>
        <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-[11px] font-medium bg-[#fcf0e8] text-amber-700">
          <Presentation className="w-3.5 h-3.5" /> .PPTX
        </span>
        <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-[11px] font-medium bg-[#fbeeed] text-rose-700">
          <FileCheck className="w-3.5 h-3.5" /> .PDF
        </span>
      </div>

      {/* Primary Action Button */}
      <div className="mt-3">
        <button
          type="button"
          className="button-primary px-5 py-2 rounded-full text-xs font-medium shadow-sm transition-transform active:scale-95"
          onClick={(e) => {
            e.stopPropagation();
            handleClick();
          }}
        >
          Choose Local File
        </button>
      </div>

      {/* Error Badge */}
      {errorMsg && (
        <div className="mt-3 flex items-center gap-1.5 text-xs text-rose-600 bg-rose-50 px-3 py-1.5 rounded-lg border border-rose-200">
          <AlertCircle className="w-3.5 h-3.5" />
          <span>{errorMsg}</span>
        </div>
      )}
    </div>
  );
};

// ============================================================================
// MERCURY FILE MODAL (OPEN & EXPORT DIALOG)
// ============================================================================

export interface MercuryFileModalProps {
  isOpen: boolean;
  mode: 'open' | 'export';
  currentTitle?: string;
  onClose: () => void;
  onFileLoaded?: (payload: OfficeDocumentPayload) => void;
  onExportConfirm?: (format: OfficeFileFormat) => void;
}

export const MercuryFileModal: React.FC<MercuryFileModalProps> = ({
  isOpen,
  mode,
  currentTitle = 'document',
  onClose,
  onFileLoaded,
  onExportConfirm,
}) => {
  const [selectedFormat, setSelectedFormat] = useState<OfficeFileFormat>('docx');

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-[#171721]/50 backdrop-blur-sm animate-fade-in">
      <div className="relative w-full max-w-lg bg-[#ffffff] rounded-3xl p-6 sm:p-8 shadow-2xl border border-[#e2e1ec] animate-scale-up">
        {/* Header */}
        <div className="flex items-center justify-between pb-4 mb-6 border-b border-[#f0eff8]">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-full bg-[#edecfb] flex items-center justify-center text-[#5266eb]">
              {mode === 'open' ? <UploadCloud className="w-5 h-5" /> : <Download className="w-5 h-5" />}
            </div>
            <div>
              <h2 className="text-lg font-semibold text-[#171721]">
                {mode === 'open' ? 'Open Document' : 'Export & Save File'}
              </h2>
              <p className="text-xs text-[#706e82]">
                {mode === 'open'
                  ? 'Import binary office documents directly into your active workspace'
                  : 'Save in authentic binary format for Microsoft Office, Google Docs, or PDF'}
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="w-8 h-8 rounded-full flex items-center justify-center text-[#706e82] hover:bg-[#f0eff8] transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content Body */}
        {mode === 'open' ? (
          <div>
            <MercuryFileDropzone
              onFileLoaded={(payload) => {
                onFileLoaded?.(payload);
                onClose();
              }}
            />
          </div>
        ) : (
          <div className="space-y-6">
            <div>
              <label className="block text-xs font-semibold text-[#3b384e] mb-2 uppercase tracking-wider">
                Select Export Format
              </label>
              <div className="grid grid-cols-2 sm:grid-cols-3 gap-2.5">
                {[
                  { fmt: 'docx' as OfficeFileFormat, label: 'Word (.docx)', icon: FileText, color: 'text-indigo-600' },
                  { fmt: 'xlsx' as OfficeFileFormat, label: 'Excel (.xlsx)', icon: Table, color: 'text-emerald-600' },
                  { fmt: 'csv' as OfficeFileFormat, label: 'CSV (.csv)', icon: Table, color: 'text-teal-600' },
                  { fmt: 'pptx' as OfficeFileFormat, label: 'PowerPoint (.pptx)', icon: Presentation, color: 'text-amber-600' },
                  { fmt: 'pdf' as OfficeFileFormat, label: 'PDF Document (.pdf)', icon: FileCheck, color: 'text-rose-600' },
                ].map(({ fmt, label, icon: Icon, color }) => (
                  <button
                    key={fmt}
                    type="button"
                    onClick={() => setSelectedFormat(fmt)}
                    className={`flex items-center gap-2.5 p-3 rounded-2xl border text-left transition-all ${
                      selectedFormat === fmt
                        ? 'border-[#5266eb] bg-[#edecfb] text-[#171721] font-medium shadow-sm'
                        : 'border-[#e2e1ec] hover:bg-[#fbfcfd] text-[#706e82]'
                    }`}
                  >
                    <Icon className={`w-4 h-4 ${color}`} />
                    <span className="text-xs">{label}</span>
                  </button>
                ))}
              </div>
            </div>

            <div className="p-3.5 bg-[#f6f5fb] rounded-2xl border border-[#e2e1ec] flex items-center justify-between">
              <div>
                <span className="text-[11px] text-[#706e82] block">Target Output File</span>
                <span className="text-xs font-semibold text-[#171721]">
                  {currentTitle.replace(/\.[^/.]+$/, '')}.{selectedFormat}
                </span>
              </div>
              <span className="text-[10px] font-medium uppercase px-2 py-0.5 rounded-full bg-[#edecfb] text-[#5266eb]">
                In-Memory Codec
              </span>
            </div>

            <div className="flex items-center justify-end gap-3 pt-2">
              <button
                type="button"
                onClick={onClose}
                className="px-4 py-2 rounded-full text-xs font-medium text-[#706e82] hover:bg-[#f0eff8] transition-colors"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={() => {
                  onExportConfirm?.(selectedFormat);
                  onClose();
                }}
                className="button-primary px-6 py-2 rounded-full text-xs font-medium shadow-sm"
              >
                Download File
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
