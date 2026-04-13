import { useState, useEffect, useRef } from 'react';
import type { SavedQuery } from '../api';

interface Props {
  onSubmit: (question: string) => void;
  onDelete: (id: number) => void;
  onUpdate: (id: number, patch: Partial<SavedQuery> & { question: string; sql_query: string }) => void;
  onTableClick: (name: string) => void;
  onDirectExecute: (question: string, sql: string) => void;
  savedQueries: SavedQuery[];
  tables: string[];
  loading: boolean;
}

const EXAMPLES = [
  'Welke luchtschepen zijn gebouwd in Duitsland?',
  'Wat is het grootste luchtschip ooit gebouwd?',
  'Hoeveel passagiers vervoerde de Hindenburg in totaal?',
  'Welke incidenten hadden meer dan 10 slachtoffers?',
  'Toon alle vluchten van Friedrichshafen naar Lakehurst',
  'Welke luchtschepen gebruikten helium?',
];

function hslToHex(h: number, s: number, l: number): string {
  s /= 100; l /= 100;
  const a = s * Math.min(l, 1 - l);
  const f = (n: number) => {
    const k = (n + h / 30) % 12;
    const color = l - a * Math.max(Math.min(k - 3, 9 - k, 1), -1);
    return Math.round(255 * color).toString(16).padStart(2, '0');
  };
  return `#${f(0)}${f(8)}${f(4)}`;
}

function hueFromHex(hex: string | null | undefined): number {
  if (!hex) return 210;
  const r = parseInt(hex.slice(1, 3), 16) / 255;
  const g = parseInt(hex.slice(3, 5), 16) / 255;
  const b = parseInt(hex.slice(5, 7), 16) / 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b);
  const d = max - min;
  if (d === 0) return 0;
  let h = 0;
  if (max === r) h = ((g - b) / d) % 6;
  else if (max === g) h = (b - r) / d + 2;
  else h = (r - g) / d + 4;
  h = Math.round(h * 60);
  return h < 0 ? h + 360 : h;
}

const PRESET_COLORS = [
  '#ef4444', '#f97316', '#eab308', '#22c55e',
  '#06b6d4', '#3b82f6', '#8b5cf6', '#ec4899',
];

export default function QueryInput({ onSubmit, onDelete, onUpdate, onTableClick, onDirectExecute, savedQueries, tables, loading }: Props) {
  const [question, setQuestion] = useState('');
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editText, setEditText] = useState('');
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; query: SavedQuery } | null>(null);
  const [showColors, setShowColors] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!contextMenu) return;
    const close = (e: MouseEvent | TouchEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setContextMenu(null);
        setShowColors(false);
      }
    };
    document.addEventListener('mousedown', close);
    document.addEventListener('touchstart', close);
    return () => {
      document.removeEventListener('mousedown', close);
      document.removeEventListener('touchstart', close);
    };
  }, [contextMenu]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (question.trim() && !loading) {
      onSubmit(question.trim());
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      if (question.trim() && !loading) {
        onSubmit(question.trim());
      }
    }
  };

  const startEdit = (q: SavedQuery) => {
    setContextMenu(null);
    setShowColors(false);
    setEditingId(q.id!);
    setEditText(q.question);
  };

  const saveEdit = (q: SavedQuery) => {
    if (editText.trim()) {
      onUpdate(q.id!, { question: editText.trim(), sql_query: q.sql_query, color: q.color });
    }
    setEditingId(null);
  };

  const handleEditKeyDown = (e: React.KeyboardEvent<HTMLInputElement>, q: SavedQuery) => {
    if (e.key === 'Enter') {
      saveEdit(q);
    } else if (e.key === 'Escape') {
      setEditingId(null);
    }
  };

  const setColor = (q: SavedQuery, color: string | null) => {
    onUpdate(q.id!, { question: q.question, sql_query: q.sql_query, color });
    setContextMenu(null);
    setShowColors(false);
  };

  const chipStyle = (q: SavedQuery): React.CSSProperties => {
    if (!q.color) return {};
    return {
      backgroundColor: q.color + '18',
      borderColor: q.color + '66',
      color: q.color,
    };
  };

  const dotStyle = (q: SavedQuery): React.CSSProperties => {
    if (!q.color) return {};
    return { color: q.color };
  };

  return (
    <div className="query-input">
      <form onSubmit={handleSubmit}>
        <textarea
          value={question}
          onChange={(e) => setQuestion(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Stel een vraag over luchtschepen... (Enter = versturen, Shift+Enter = nieuwe regel)"
          rows={3}
          disabled={loading}
        />
        <button type="submit" disabled={loading || !question.trim()}>
          {loading ? 'Bezig...' : 'Vraag stellen'}
        </button>
      </form>
      <div className="examples">
        <span>Voorbeelden:</span>
        <div className="example-chips">
          {EXAMPLES.map((ex, i) => (
            <button
              key={`ex-${i}`}
              className="chip"
              onClick={() => {
                setQuestion(ex);
                onSubmit(ex);
              }}
              disabled={loading}
            >
              {ex}
            </button>
          ))}
        </div>
        {tables.length > 0 && (
          <>
            <span>Tabellen:</span>
            <div className="example-chips">
              {tables.map((t) => (
                <button
                  key={`table-${t}`}
                  className="chip table-chip"
                  onClick={() => onTableClick(t)}
                  disabled={loading}
                >
                  {t}
                </button>
              ))}
            </div>
          </>
        )}
        {savedQueries.length > 0 && (
          <>
            <span>Opgeslagen:</span>
            <div className="example-chips">
              {savedQueries.map((q) => (
                <span key={`saved-${q.id}`} className="chip saved-chip" style={chipStyle(q)}>
                  {editingId === q.id ? (
                    <input
                      className="chip-edit-input"
                      value={editText}
                      onChange={(e) => setEditText(e.target.value)}
                      onKeyDown={(e) => handleEditKeyDown(e, q)}
                      onBlur={() => saveEdit(q)}
                      autoFocus
                    />
                  ) : (
                    <>
                      <span
                        className="chip-text"
                        onClick={() => {
                          if (!loading) {
                            setQuestion(q.question);
                            onDirectExecute(q.question, q.sql_query);
                          }
                        }}
                      >
                        {q.question}
                      </span>
                      <span
                        className="chip-dots"
                        style={dotStyle(q)}
                        onClick={(e) => {
                          e.stopPropagation();
                          const rect = (e.target as HTMLElement).getBoundingClientRect();
                          setContextMenu({ x: rect.left, y: rect.bottom + 4, query: q });
                          setShowColors(false);
                        }}
                      >
                        &middot;&middot;&middot;
                      </span>
                    </>
                  )}
                </span>
              ))}
            </div>
          </>
        )}
      </div>

      {contextMenu && (
        <div
          ref={menuRef}
          className="context-menu"
          style={{ top: contextMenu.y, left: contextMenu.x }}
        >
          <button onClick={() => startEdit(contextMenu.query)}>Bewerken</button>
          <button onClick={() => setShowColors(!showColors)}>
            Kleur
            {contextMenu.query.color && (
              <span className="color-dot-preview" style={{ background: contextMenu.query.color }} />
            )}
          </button>
          {showColors && (
            <div className="color-picker-section">
              <div className="color-presets">
                {PRESET_COLORS.map((c) => (
                  <span
                    key={c}
                    className={'color-swatch' + (contextMenu.query.color === c ? ' active' : '')}
                    style={{ background: c }}
                    onClick={() => setColor(contextMenu.query, c)}
                  />
                ))}
                <span
                  className={'color-swatch color-swatch-none' + (!contextMenu.query.color ? ' active' : '')}
                  onClick={() => setColor(contextMenu.query, null)}
                  title="Geen kleur"
                />
              </div>
              <div className="color-custom">
                <span>Eigen:</span>
                <input
                  type="range"
                  className="hue-slider"
                  min="0"
                  max="360"
                  value={hueFromHex(contextMenu.query.color)}
                  onChange={(e) => {
                    const hue = parseInt(e.target.value);
                    setColor(contextMenu.query, hslToHex(hue, 70, 55));
                  }}
                />
              </div>
            </div>
          )}
          <button
            className="context-menu-delete"
            onClick={() => {
              onDelete(contextMenu.query.id!);
              setContextMenu(null);
              setShowColors(false);
            }}
          >
            Verwijderen
          </button>
        </div>
      )}
    </div>
  );
}
