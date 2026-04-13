import { useState } from 'react';
import type { SavedQuery } from '../api';

interface Props {
  queries: SavedQuery[];
  onRun: (question: string) => void;
  onDelete: (id: number) => void;
  onUpdate: (id: number, question: string, sql: string) => void;
}

export default function SavedQueries({ queries, onRun, onDelete, onUpdate }: Props) {
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editQuestion, setEditQuestion] = useState('');
  const [editSql, setEditSql] = useState('');

  const startEdit = (q: SavedQuery) => {
    setEditingId(q.id!);
    setEditQuestion(q.question);
    setEditSql(q.sql_query);
  };

  const saveEdit = () => {
    if (editingId !== null) {
      onUpdate(editingId, editQuestion, editSql);
      setEditingId(null);
    }
  };

  if (queries.length === 0) {
    return (
      <div className="saved-queries">
        <h3>Opgeslagen queries</h3>
        <p className="empty">Nog geen opgeslagen queries. Stel een vraag en sla het resultaat op.</p>
      </div>
    );
  }

  return (
    <div className="saved-queries">
      <h3>Opgeslagen queries ({queries.length})</h3>
      <ul>
        {queries.map((q) => (
          <li key={q.id}>
            {editingId === q.id ? (
              <div className="edit-form">
                <input
                  value={editQuestion}
                  onChange={(e) => setEditQuestion(e.target.value)}
                  placeholder="Vraag"
                />
                <textarea
                  value={editSql}
                  onChange={(e) => setEditSql(e.target.value)}
                  placeholder="SQL"
                  rows={2}
                />
                <div className="edit-actions">
                  <button onClick={saveEdit}>Opslaan</button>
                  <button onClick={() => setEditingId(null)}>Annuleren</button>
                </div>
              </div>
            ) : (
              <>
                <div className="query-text" onClick={() => onRun(q.question)}>
                  {q.question}
                </div>
                <code className="query-sql">{q.sql_query}</code>
                <div className="query-actions">
                  <button onClick={() => onRun(q.question)} title="Opnieuw uitvoeren">Uitvoeren</button>
                  <button onClick={() => startEdit(q)} title="Bewerken">Bewerken</button>
                  <button onClick={() => onDelete(q.id!)} title="Verwijderen" className="delete">Verwijderen</button>
                </div>
              </>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
}
