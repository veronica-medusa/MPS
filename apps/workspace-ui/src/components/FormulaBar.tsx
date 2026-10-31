import React, { useState, useEffect, useRef } from 'react';
import { Check, X, FunctionSquare } from 'lucide-react';

interface FormulaBarProps {
  activeCell: string;
  value: string;
  onChangeValue: (val: string) => void;
  onSubmit: (val: string) => void;
  onOpenFunctionsModal?: () => void;
}

export const FormulaBar: React.FC<FormulaBarProps> = ({
  activeCell,
  value,
  onChangeValue,
  onSubmit,
  onOpenFunctionsModal,
}) => {
  const [internalValue, setInternalValue] = useState(value);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    setInternalValue(value);
  }, [value, activeCell]);

  const commonFunctions = [
    { name: 'PY', desc: 'Embedded Python Runtime: execute Python code with wp.cell/wp.range DAG bindings' },
    { name: 'SUM', desc: 'Adds all numbers in a range of cells' },
    { name: 'AVERAGE', desc: 'Returns the average (arithmetic mean) of arguments' },
    { name: 'XLOOKUP', desc: 'Searches a range or array for a match and returns corresponding item' },
    { name: 'VLOOKUP', desc: 'Looks for a value in the leftmost column of a table' },
    { name: 'IF', desc: 'Specifies a logical test to perform' },
    { name: 'SUMIFS', desc: 'Adds cells specified by a given set of conditions or criteria' },
    { name: 'COUNTIF', desc: 'Counts the number of cells that meet a given condition' },
    { name: 'FILTER', desc: 'Filters a range of data based on criteria you define' },
    { name: 'UNIQUE', desc: 'Returns a list of unique values in a list or range' },
    { name: 'SORT', desc: 'Sorts the contents of a range or array' },
    { name: 'SEQUENCE', desc: 'Generates a list of sequential numbers in an array' },
    { name: 'PMT', desc: 'Calculates payment for a loan based on constant payments and rate' },
    { name: 'IRR', desc: 'Returns the internal rate of return for a series of cash flows' },
    { name: 'NPV', desc: 'Returns net present value of an investment based on discount rate' },
    { name: 'DATE', desc: 'Returns serial number that represents a specific date' },
    { name: 'NETWORKDAYS', desc: 'Returns number of whole working days between two dates' },
  ];

  const filteredSuggestions = internalValue.startsWith('=')
    ? commonFunctions.filter((f) =>
        f.name.toLowerCase().startsWith(internalValue.slice(1).toUpperCase())
      )
    : [];

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      onSubmit(internalValue);
      setShowSuggestions(false);
    } else if (e.key === 'Escape') {
      setInternalValue(value);
      setShowSuggestions(false);
    }
  };

  const handleSelectFunction = (fnName: string) => {
    const newVal = `=${fnName}(`;
    setInternalValue(newVal);
    onChangeValue(newVal);
    setShowSuggestions(false);
    inputRef.current?.focus();
  };

  return (
    <div className="h-7 bg-white dark:bg-mercury-night border-b border-mercury-fog/30 flex items-center px-2 text-xs relative select-none">
      {/* Name Box */}
      <div className="w-16 h-5.5 px-2 flex items-center justify-center font-mono font-medium text-[11px] text-mercury-ink dark:text-mercury-on-dark bg-mercury-lavender/40 dark:bg-mercury-night-raised border border-mercury-fog/30 rounded-mercury-xs mr-1 shadow-inner">
        {activeCell || 'A1'}
      </div>

      {/* Action Buttons: Cancel / Accept */}
      <div className="flex items-center space-x-0.5 border-r border-mercury-fog/30 pr-1.5 mr-1.5">
        <button
          onClick={() => {
            setInternalValue(value);
            setShowSuggestions(false);
          }}
          className="p-1 rounded text-mercury-ink-secondary hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-950/30 transition-colors"
          title="Cancel (Esc)"
        >
          <X className="w-3 h-3" />
        </button>
        <button
          onClick={() => {
            onSubmit(internalValue);
            setShowSuggestions(false);
          }}
          className="p-1 rounded text-mercury-ink-secondary hover:text-mercury-sheets hover:bg-emerald-50 dark:hover:bg-emerald-950/30 transition-colors"
          title="Accept (Enter)"
        >
          <Check className="w-3 h-3" />
        </button>
        <button
          onClick={onOpenFunctionsModal}
          className="p-1 rounded text-mercury-sheets hover:bg-mercury-sheets/10 font-bold italic transition-colors"
          title="Insert Function (fx)"
        >
          <FunctionSquare className="w-3.5 h-3.5" />
        </button>
      </div>

      {/* Formula Bar Input */}
      <div className="flex-1 relative flex items-center h-full">
        <input
          ref={inputRef}
          type="text"
          value={internalValue}
          onChange={(e) => {
            setInternalValue(e.target.value);
            onChangeValue(e.target.value);
            setShowSuggestions(e.target.value.startsWith('=') && e.target.value.length > 1);
          }}
          onKeyDown={handleKeyDown}
          onFocus={() => {
            if (internalValue.startsWith('=')) setShowSuggestions(true);
          }}
          placeholder="Enter formula or value (e.g. =SUM(B2:D2) or 1500)"
          className="w-full h-5.5 px-2 font-mono text-[11px] bg-transparent text-mercury-ink dark:text-mercury-on-dark outline-none focus:bg-mercury-lavender/30 dark:focus:bg-mercury-night-raised rounded border border-transparent focus:border-mercury-sheets/50 transition-colors"
        />

        {/* Function Autocomplete Suggestions Dropdown */}
        {showSuggestions && filteredSuggestions.length > 0 && (
          <div className="absolute top-7 left-0 w-80 bg-white dark:bg-mercury-night-raised border border-mercury-fog/30 rounded-mercury-md shadow-2xl z-50 py-1 max-h-48 overflow-y-auto animate-in fade-in zoom-in-95">
            <div className="px-2 py-1 text-[10px] font-semibold text-mercury-sheets uppercase tracking-wider border-b border-mercury-fog/20">
              150+ Formula Library
            </div>
            {filteredSuggestions.map((item) => (
              <div
                key={item.name}
                onClick={() => handleSelectFunction(item.name)}
                className="px-2.5 py-1.5 hover:bg-mercury-sheets/10 cursor-pointer flex flex-col transition-colors"
              >
                <div className="flex items-center space-x-1.5">
                  <span className="font-mono font-bold text-mercury-sheets text-xs">{item.name}</span>
                </div>
                <span className="text-[10px] text-mercury-ink-secondary dark:text-mercury-fog truncate">
                  {item.desc}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
