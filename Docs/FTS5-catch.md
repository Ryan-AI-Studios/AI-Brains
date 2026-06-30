# FTS5 Catch: Defensive Query Handling for SQLite Full-Text Search

**Author:** Systematic Command Audit  
**Date:** 2026-05-20  
**Scope:** `ai-brains recall` and any downstream consumer that forwards raw user queries into SQLite FTS5

---

## Problem Statement

When downstream tools (e.g., Ledgerful `bridge query`) pass raw user queries into `ai-brains recall`, unescaped SQLite FTS5 special characters cause a hard syntax error. The error propagates as raw text instead of structured JSON, which in turn confuses the NDJSON parser in Ledgerful's bridge client.

### Manifestation
```
WARN changeguard::bridge::client::client_cli:
  ai-brains CLI returned error:
  Failed to parse bridge search response line:
    expected value at line 1 column 1
  Error: database error: fts5: syntax error near "?"
```

The `?` character is a valid character in natural-language questions but is **invalid** as a bare token in FTS5 query syntax.

---

## FTS5 Special Characters

SQLite FTS5 (Module Version 5) treats the following characters as syntax operators when they appear unquoted in a query string:

| Character | Meaning in FTS5 | Example of breakage |
|-----------|-----------------|---------------------|
| `"`       | Phrase delimiter | `"hello` → unterminated phrase |
| `*`       | Prefix match operator | `foo*` works, `*` alone → syntax error |
| `?`       | **Not an operator**, but bare `?` is interpreted as an empty token / placeholder in some FTS5 builds | `what is?` → `fts5: syntax error near "?"` |
| `AND`     | Binary AND operator | `cats AND dogs` → valid; `AND` alone → error |
| `OR`      | Binary OR operator | same as above |
| `NOT`     | Unary NOT operator | same as above |
| `NEAR`    | Proximity operator | `NEAR/5` syntax |
| `(` `)`   | Grouping | unbalanced parens → error |

> **Note:** The exact set of reserved tokens depends on the FTS5 tokenizer. The default `unicode61` tokenizer treats `?` as a separator (non-alphanumeric), but when it appears isolated it can still trigger a syntax error in the query parser.

---

## Root Cause in Ledgerful → AI-Brains Flow

1. Ledgerful `ask` receives a natural-language question like:  
   `"What is the main purpose of ChangeGuard?"`
2. Ledgerful's bridge client forwards this **verbatim** to `ai-brains recall` via the named-pipe IPC.
3. AI-Brains constructs an FTS5 query using the raw string:
   ```sql
   SELECT * FROM memories WHERE content MATCH 'What is the main purpose of ChangeGuard?'
   ```
4. The trailing `?` is parsed as a token. Depending on build flags and tokenizer, it either:
   - Is stripped by the tokenizer (safe), **or**
   - Is passed to the query parser as a bare token, causing `fts5: syntax error near "?"`.
5. The error bubbles up as a plain string, not JSON. Ledgerful's `BridgeRecord` NDJSON parser chokes on it.

---

## Recommended Defensive Strategies

### Strategy 1: Sanitize Before FTS5 (Recommended)

In the consumer (Ledgerful) or at the AI-Brains API boundary, sanitize the query:

```rust
fn sanitize_for_fts5(input: &str) -> String {
    // Remove or replace characters that break FTS5 bare-token parsing
    let cleaned = input
        .replace('?', "")           // Remove question marks
        .replace('"', "")           // Remove double quotes
        .replace('*', "")           // Remove asterisks
        .replace("  ", " ");        // Collapse double spaces
    
    // Trim and ensure non-empty
    cleaned.trim().to_string()
}
```

> **Pros:** Simple, fast, keeps query semantics mostly intact.  
> **Cons:** Slightly alters user intent (question marks are semantically meaningful).

### Strategy 2: Quote the Entire Query

Wrap the entire query in double quotes so FTS5 treats it as a single phrase:

```rust
fn quote_for_fts5(input: &str) -> String {
    // Escape any existing double quotes first
    let escaped = input.replace('"', "\"\"");
    format!("\"{}\"", escaped)
}
```

Example: `What is ChangeGuard?` → `"What is ChangeGuard?"`

> **Pros:** Zero semantic loss.  
> **Cons:** FTS5 phrase matching is stricter; it looks for the exact sequence of words in order. This may reduce recall for fuzzy searches.

### Strategy 3: Hybrid (Recommended for Production)

Use a two-stage approach:

1. **Primary:** Try the raw (or lightly sanitized) query with the default tokenizer.
2. **Fallback:** If a syntax error is returned, fall back to a quoted phrase or a keyword-only extraction.

```rust
pub fn safe_recall(query: &str, db: &Connection) -> Result<Vec<Memory>, Error> {
    // Attempt 1: sanitized keyword query
    let sanitized = sanitize_for_fts5(query);
    if let Ok(results) = query_fts5(&sanitized, db) {
        return Ok(results);
    }
    
    // Attempt 2: exact phrase
    let quoted = quote_for_fts5(query);
    if let Ok(results) = query_fts5(&quoted, db) {
        return Ok(results);
    }
    
    // Attempt 3: keyword-only (strip all non-alphanumeric)
    let keywords: String = query
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect();
    query_fts5(&keywords, db)
}
```

---

## AI-Brains Internal Recommendation

### Catch & Convert

In `ai-brains recall` (or the IPC handler that receives external queries), implement the following guard:

```rust
use sqlite::Error as SqliteError;

fn execute_fts5_safe(db: &Connection, raw_query: &str) -> Result<Vec<Row>, Error> {
    // Fast path: try raw query (user may intentionally use FTS5 operators)
    match try_fts5(db, raw_query) {
        Ok(rows) => return Ok(rows),
        Err(SqliteError::Fts5Syntax(_)) => {
            tracing::warn!("FTS5 syntax error on raw query, falling back to quoted phrase");
        }
        Err(e) => return Err(e.into()),
    }
    
    // Fallback: quote entire string as a single phrase
    let quoted = format!("\"{}\"", raw_query.replace('"', "\"\""));
    try_fts5(db, &quoted)
}
```

### Error Response Contract

When FTS5 fails **even after** the fallback, return a structured JSON error so downstream NDJSON parsers don't choke:

```json
{
  "status": "error",
  "error_type": "fts5_syntax",
  "message": "Query contains characters that cannot be interpreted by the search engine.",
  "suggestion": "Remove punctuation such as ? * \" and retry.",
  "original_query": "What is ChangeGuard?"
}
```

This ensures Ledgerful's `BridgeRecord` deserialization succeeds and can present a meaningful error to the user.

---

## Verification Checklist

After implementing a fix, verify with these queries:

```bash
# Contains ?
ai-brains recall "What is the main purpose of ChangeGuard?"

# Contains "
ai-brains recall 'He said "hello" to the system'

# Contains *
ai-brains recall "find all files matching *.rs"

# Contains AND/OR/NOT as words
ai-brains recall "difference between AND and OR operators"

# Empty string
ai-brains recall ""

# Only special chars
ai-brains recall "? * \""
```

All should return either valid results or a structured JSON error — **never** raw SQLite error text.

---

## Related Issues

- **Ledgerful CG-2:** FTS5 syntax error degrades dual-retrieval in `ask` and `bridge query`.  
- **Ledgerful CG-1:** Local model unreachable due to `localhost` → `::1` resolution.  
- **AI-Brains IPC Protocol:** BridgeRecord v0.2 schema assumes every line is valid NDJSON. Error text breaks this invariant.

---

## References

- [SQLite FTS5 Query Syntax](https://www.sqlite.org/fts5.html#full_text_query_syntax)
- [FTS5 Tokenizers](https://www.sqlite.org/fts5.html#tokenizers)
- Ledgerful BridgeRecord Schema: `src/bridge/model.rs`
- Ledgerful Bridge Client: `src/bridge/client.rs`, `src/bridge/client/client_cli.rs`
