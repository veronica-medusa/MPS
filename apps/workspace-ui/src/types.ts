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
