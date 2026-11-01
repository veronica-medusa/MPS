export type DocumentType = 'writer' | 'sheets' | 'slides' | 'pdf';

export interface Collaborator {
  id: string;
  name: string;
  color: string;
  avatar: string;
}

export interface SheetInfo {
  id: string;
  name: string;
  rows?: number;
  cols?: number;
}

export interface SelectionStats {
  count: number;
  num_count: number;
  sum: number;
  average: number;
  min: number;
  max: number;
}

export type PdfSidebarTab = 'thumbnails' | 'outline' | 'annotations' | 'search_redact';

export type PdfMode = 'read' | 'edit_content' | 'annotate' | 'reflow' | 'organize';

export interface PdfPageInfo {
  index: number;
  width: number;
  height: number;
  rotation: number;
  textElementsCount: number;
  vectorPathsCount: number;
  annotationCount: number;
}

export interface PdfAnnotationInfo {
  id: string;
  pageIndex: number;
  kind: string;
  author: string;
  content: string;
  bounds: [number, number, number, number];
  color: string;
}

export interface DocumentTab {
  id: string;
  title: string;
  type: DocumentType;
  isDirty: boolean;
  filePath?: string;
  zoom: number;
  collaborators: Collaborator[];
  activeSheet?: string;
  activeSheetIndex?: number;
  sheetsList?: SheetInfo[];
  currentPage?: number;
  totalPages?: number;
  currentSlide?: number;
  totalSlides?: number;
  wordCount?: number;
  charCount?: number;
  trackChanges?: boolean;
  pendingRevisions?: number;
  activeCell?: string;
  formulaBarValue?: string;
  readingView?: boolean;
  freezePane?: boolean;
  selectionSummary?: string;
  stats?: SelectionStats;
  pdfMode?: PdfMode;
  pdfSidebarTab?: PdfSidebarTab;
  pdfSearchQuery?: string;
  pdfWatermark?: string;
  pdfBatesPrefix?: string;
  pdfIsEncrypted?: boolean;
}

export type RibbonMode = 'classic' | 'compact';

export type ActiveRibbonTab =
  | 'home'
  | 'insert'
  | 'layout'
  | 'references'
  | 'formulas'
  | 'data'
  | 'toolbox'
  | 'slides'
  | 'transitions'
  | 'animations'
  | 'pdf_annotate'
  | 'pdf_edit'
  | 'pdf_organize'
  | 'pdf_convert'
  | 'pdf_protect'
  | 'review'
  | 'view'
  | 'ai';

export interface CommandItem {
  id: string;
  title: string;
  category: string;
  shortcut?: string;
  icon?: string;
  action: () => void;
}

export type SmartToolboxAction =
  | 'merge_no_loss'
  | 'unmerge_fill'
  | 'split_sheet'
  | 'combine_sheets'
  | 'compare_sheets'
  | 'data_hygiene'
  | 'range_arithmetic'
  | 'add_prefix_suffix'
  | 'case_conversion'
  | 'regex_extract'
  | 'formula_to_value'
  | 'pivot_wizard'
  | 'data_validation';

export interface SlideShape {
  id: string;
  name: string;
  geometry: string;
  x: number;
  y: number;
  width: number;
  height: number;
  rotation?: number;
  fillColor?: string;
  strokeColor?: string;
  strokeWidth?: number;
  adjustmentValues?: number[];
  text?: string;
}

export interface SlideConnector {
  id: string;
  startId: string;
  endId: string;
  startPort?: 'top' | 'right' | 'bottom' | 'left';
  endPort?: 'top' | 'right' | 'bottom' | 'left';
  routing: 'straight' | 'orthogonal' | 'curved';
  color: string;
  strokeWidth: number;
}

export interface SlideInfo {
  id: string;
  title: string;
  layoutId: string;
  shapes: SlideShape[];
  connectors: SlideConnector[];
  transitionName?: string;
  transitionDuration?: number;
  speakerNotes: string;
}

export interface PresenterCockpitState {
  isOpen: boolean;
  slideIndex: number;
  isTimerRunning: boolean;
  elapsedSeconds: number;
  activeTool: 'pointer' | 'pen' | 'highlighter';
  penColor: string;
  isBlackout: boolean;
  isWhiteout: boolean;
}


// ============================================================================
// WPS OFFICE 5-ZONE WORKSPACE EXTENDED TYPES (MERCURY DESIGN)
// ============================================================================

export interface OfficeTab {
  id: string;
  title: string;
  type: DocumentType;
  isDirty: boolean;
  lastModified: string;
  filePath?: string;
  zoom?: number;
}

// ─── WPS Writer Models ───
export interface WriterParagraph {
  id: string;
  style: 'Normal' | 'Title' | 'Heading 1' | 'Heading 2' | 'Heading 3' | 'Quote';
  text: string;
  fontFamily?: string;
  fontSize?: number;
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  strike?: boolean;
  align?: 'left' | 'center' | 'right' | 'justify';
  color?: string;
  highlight?: string;
  revision?: {
    type: 'delete' | 'insert';
    originalText: string;
    revisedText: string;
    author: string;
    date: string;
  };
}

export interface WriterTable {
  id: string;
  rows: number;
  cols: number;
  data: string[][];
}

export interface WriterComment {
  id: string;
  author: string;
  date: string;
  text: string;
  targetText: string;
  resolved: boolean;
}

export interface WriterDocumentState {
  paragraphs: WriterParagraph[];
  tables: WriterTable[];
  comments: WriterComment[];
  trackChanges: boolean;
  markupView: 'all' | 'final';
  pageSize: 'Letter' | 'A4';
  orientation: 'Portrait' | 'Landscape';
  margins: 'Normal' | 'Narrow' | 'Wide';
  columns: 1 | 2;
  zoom: number;
}

// ─── WPS Spreadsheets Models ───
export interface SheetCell {
  raw: string;
  value: string;
  format?: 'general' | 'currency' | 'percent' | 'number';
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  fillColor?: string;
  textColor?: string;
  align?: 'left' | 'center' | 'right';
}

export interface SingleSheet {
  id: string;
  name: string;
  cells: Record<string, SheetCell>;
  colWidths?: Record<string, number>;
}

export interface SheetsWorkbookState {
  sheets: SingleSheet[];
  activeSheetIndex: number;
  activeCell: string;
  selectedRange?: string;
  formulaBarValue: string;
  readingSpotlight: boolean;
  showFormulas: boolean;
  chart?: {
    type: 'bar' | 'line' | 'pie' | 'area';
    title: string;
    range: string;
    labels: string[];
    values: number[];
  } | null;
}

// ─── WPS Presentation Models ───
export interface SlideElement {
  id: string;
  type: 'title' | 'subtitle' | 'text' | 'shape' | 'image' | 'table';
  x: number;
  y: number;
  width: number;
  height: number;
  text?: string;
  shapeType?: 'rectangle' | 'rounded' | 'circle' | 'arrow' | 'callout' | 'star';
  fillColor?: string;
  strokeColor?: string;
  textColor?: string;
  fontSize?: number;
  bold?: boolean;
  align?: 'left' | 'center' | 'right';
  tableData?: string[][];
}

export interface SlideData {
  id: string;
  title: string;
  layout: 'title' | 'title_content' | 'two_column' | 'blank';
  elements: SlideElement[];
  speakerNotes: string;
  transition?: 'none' | 'fade' | 'push' | 'wipe' | 'morph';
}

export interface PresentationState {
  slides: SlideData[];
  activeSlideIndex: number;
  theme: 'mercury-night' | 'mercury-canvas' | 'mercury-lavender' | 'executive-indigo';
  selectedElementId?: string | null;
  isPlayingSlideshow: boolean;
  isPresenterCockpit: boolean;
}

// ─── WPS PDF Models ───
export interface PdfParagraphBlock {
  id: string;
  page: number;
  role: 'title' | 'heading' | 'paragraph' | 'table';
  text: string;
  highlight?: 'yellow' | 'lavender' | 'periwinkle' | null;
  redacted?: boolean;
  underline?: boolean;
  strikethrough?: boolean;
  tableData?: string[][];
}

export interface PdfStickyNote {
  id: string;
  page: number;
  x: number;
  y: number;
  text: string;
  author: string;
  date: string;
}

export interface PdfSignature {
  id: string;
  page: number;
  x: number;
  y: number;
  dataUrl: string;
  author: string;
  date: string;
}

export interface PdfDocumentState {
  blocks: PdfParagraphBlock[];
  notes: PdfStickyNote[];
  signatures: PdfSignature[];
  isEditingText: boolean;
  activeTool: 'select' | 'pan' | 'highlight' | 'note' | 'redact' | 'sign';
  highlightColor: 'yellow' | 'lavender' | 'periwinkle';
  searchQuery: string;
  currentPage: number;
  totalPages: number;
  zoom: number;
}

// ─── Version Snapshot & AI Models ───
export interface VersionSnapshot {
  id: string;
  timestamp: string;
  author: string;
  description: string;
  type: 'auto' | 'manual' | 'ai' | 'convert';
  targetDocTitle: string;
}

export interface AiChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: string;
  actionTag?: string;
  citationPage?: number;
}
