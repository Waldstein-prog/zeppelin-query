import { useState, useEffect, useCallback } from 'react';
import QueryInput from './components/QueryInput';
import ResultsTable from './components/ResultsTable';
import {
  submitQuery,
  getSavedQueries,
  createSavedQuery,
  updateSavedQuery,
  deleteSavedQuery,
  getTables,
  getTableData,
  executeDirect,
} from './api';
import type { QueryResponse, SavedQuery } from './api';

function App() {
  const [result, setResult] = useState<QueryResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [savedQueries, setSavedQueries] = useState<SavedQuery[]>([]);
  const [tables, setTables] = useState<string[]>([]);

  const loadSaved = useCallback(async () => {
    try {
      const queries = await getSavedQueries();
      setSavedQueries(queries);
    } catch (e) {
      console.error('Failed to load saved queries:', e);
    }
  }, []);

  useEffect(() => {
    loadSaved();
    getTables().then(setTables).catch(() => {});
  }, [loadSaved]);

  const handleDirectExecute = async (question: string, sql: string) => {
    setLoading(true);
    try {
      const res = await executeDirect(question, sql);
      setResult(res);
    } catch (e) {
      setResult({
        question,
        sql,
        columns: [],
        rows: [],
        error: `Netwerk fout: ${e}`,
      });
    } finally {
      setLoading(false);
    }
  };

  const handleTableClick = async (name: string) => {
    setLoading(true);
    try {
      const res = await getTableData(name);
      setResult(res);
    } catch (e) {
      setResult({
        question: `Tabel: ${name}`,
        sql: '',
        columns: [],
        rows: [],
        error: `Netwerk fout: ${e}`,
      });
    } finally {
      setLoading(false);
    }
  };

  const handleQuery = async (question: string) => {
    setLoading(true);
    try {
      const res = await submitQuery(question);
      setResult(res);
    } catch (e) {
      setResult({
        question,
        sql: '',
        columns: [],
        rows: [],
        error: `Netwerk fout: ${e}`,
      });
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    if (!result || !result.sql) return;
    try {
      await createSavedQuery({
        question: result.question,
        sql_query: result.sql,
      });
      await loadSaved();
    } catch (e) {
      console.error('Failed to save query:', e);
    }
  };

  const handleDelete = async (id: number) => {
    try {
      await deleteSavedQuery(id);
      await loadSaved();
    } catch (e) {
      console.error('Failed to delete query:', e);
    }
  };

  const handleUpdate = async (id: number, patch: Partial<SavedQuery> & { question: string; sql_query: string }) => {
    try {
      await updateSavedQuery(id, patch);
      await loadSaved();
    } catch (e) {
      console.error('Failed to update query:', e);
    }
  };

  return (
    <div className="app">
      <header>
        <h1>Zeppelin Query</h1>
        <p className="llm-status">LLM via WWW</p>
        <p>Stel vragen over historische luchtschepen in natuurlijke taal</p>
      </header>

      <main>
        <QueryInput
          onSubmit={handleQuery}
          onDelete={handleDelete}
          onUpdate={handleUpdate}
          onTableClick={handleTableClick}
          onDirectExecute={handleDirectExecute}
          savedQueries={savedQueries}
          tables={tables}
          loading={loading}
        />

        {result && !result.error && result.sql && (
          <button className="save-btn" onClick={handleSave}>
            Query opslaan
          </button>
        )}

        <ResultsTable result={result} />
      </main>
    </div>
  );
}

export default App;
