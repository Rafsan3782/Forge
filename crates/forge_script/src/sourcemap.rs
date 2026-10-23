//! Source map decoder and location mapper.

use sourcemap::SourceMap;

/// Represents a mapped source location in original TypeScript source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    /// Source file path.
    pub file: String,
    /// 1-indexed line number.
    pub line: u32,
    /// 1-indexed column number.
    pub column: u32,
}

/// Helper for mapping runtime JavaScript line/column to original TypeScript locations.
pub struct ScriptSourceMapper {
    map: Option<SourceMap>,
    default_filename: String,
}

impl ScriptSourceMapper {
    /// Creates a new mapper from source map JSON (if available).
    pub fn new(default_filename: impl Into<String>, source_map_json: Option<&str>) -> Self {
        let default_filename = default_filename.into();
        let map = source_map_json.and_then(|json| SourceMap::from_reader(json.as_bytes()).ok());
        Self {
            map,
            default_filename,
        }
    }

    /// Maps a 1-indexed line and column from JavaScript output back to TypeScript.
    pub fn lookup(&self, line: u32, column: u32) -> SourceLocation {
        if let Some(ref sm) = self.map {
            // sourcemap crate lookup_token uses 0-indexed line and column
            let zero_line = line.saturating_sub(1);
            let zero_col = column.saturating_sub(1);

            if let Some(token) = sm.lookup_token(zero_line, zero_col) {
                let file = token
                    .get_source()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| self.default_filename.clone());
                return SourceLocation {
                    file,
                    // Convert back to 1-indexed
                    line: token.get_src_line() + 1,
                    column: token.get_src_col() + 1,
                };
            }
        }

        SourceLocation {
            file: self.default_filename.clone(),
            line,
            column,
        }
    }
}
