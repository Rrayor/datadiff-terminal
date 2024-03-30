use std::{error::Error, fmt};

use libdtf::core::diff_types::{ArrayDiff, Diff, KeyDiff, TypeDiff, ValueDiff};
use serde::{Deserialize, Serialize};
use term_table::{row::Row, Table, TableStyle};

pub type LibConfig = libdtf::core::diff_types::Config;
pub type LibWorkingContext = libdtf::core::diff_types::WorkingContext;

/// Stores the data required for rendering a table of the differences
pub struct TableContext<'a> {
    working_context: &'a LibWorkingContext,
    table: Table<'a>,
}

impl<'a> TableContext<'a> {
    pub fn new(working_context: &'a LibWorkingContext) -> TableContext {
        let mut table = Table::new();
        table.max_column_width = 80;
        table.style = TableStyle::extended();
        TableContext {
            working_context,
            table,
        }
    }

    /// Returns the current context of the table
    pub fn working_context(&self) -> &'a LibWorkingContext {
        self.working_context
    }

    /// Sets the actual table (term_table::Table)
    pub fn set_table(&mut self, table: Table<'a>) {
        self.table = table;
    }

    /// Adds a row to the table
    pub fn add_row(&mut self, row: Row<'a>) {
        self.table.add_row(row);
    }

    /// Returns the built table string
    pub fn render(&self) -> String {
        self.table.render()
    }
}

/// Gives tables the required functionality
pub trait TermTable<T: Diff> {
    fn render(&self) -> String;
    fn create_table(&mut self, data: &[T]);
    fn add_header(&mut self);
    fn add_rows(&mut self, data: &[T]);
}

/// The data structure arguments are needed to be stored in
pub type ParsedArgs = (
    Option<String>,
    Option<String>,
    Config,
);

/// How we move the result of diff checking around
pub type DiffCollection = (
    Option<Vec<KeyDiff>>,
    Option<Vec<TypeDiff>>,
    Option<Vec<ValueDiff>>,
    Option<Vec<ArrayDiff>>,
);

/// The structure a result set gets saved in for later re-use
#[derive(Serialize, Deserialize)]
pub struct SavedConfig {
    pub check_for_key_diffs: bool,
    pub check_for_type_diffs: bool,
    pub check_for_value_diffs: bool,
    pub check_for_array_diffs: bool,
    pub file_a: String,
    pub file_b: String,
    pub array_same_order: bool,
}

impl SavedConfig {
    pub fn new(
        check_for_key_diffs: bool,
        check_for_type_diffs: bool,
        check_for_value_diffs: bool,
        check_for_array_diffs: bool,
        file_a: String,
        file_b: String,
        array_same_order: bool,
    ) -> SavedConfig {
        SavedConfig {
            check_for_key_diffs,
            check_for_type_diffs,
            check_for_value_diffs,
            check_for_array_diffs,
            file_a,
            file_b,
            array_same_order,
        }
    }
}

/// The structure the runtime configurations are stored in
#[derive(Clone)]
pub struct Config {
    pub check_for_key_diffs: bool,
    pub check_for_type_diffs: bool,
    pub check_for_value_diffs: bool,
    pub check_for_array_diffs: bool,
    pub render_key_diffs: bool,
    pub render_type_diffs: bool,
    pub render_value_diffs: bool,
    pub render_array_diffs: bool,
    pub read_from_file: String,
    pub write_to_file: Option<String>,
    pub write_to_html: bool,
    pub file_a: Option<String>,
    pub file_b: Option<String>,
    pub array_same_order: bool,
}

/// Helper class for creating Config instances
#[derive(Default)]
pub struct ConfigBuilder {
    check_for_key_diffs: bool,
    check_for_type_diffs: bool,
    check_for_value_diffs: bool,
    check_for_array_diffs: bool,
    render_key_diffs: bool,
    render_type_diffs: bool,
    render_value_diffs: bool,
    render_array_diffs: bool,
    read_from_file: String,
    write_to_file: Option<String>,
    write_to_html: bool,
    file_a: Option<String>,
    file_b: Option<String>,
    array_same_order: bool,
}

impl ConfigBuilder {
    pub fn new() -> ConfigBuilder {
        ConfigBuilder {
            check_for_key_diffs: false,
            check_for_type_diffs: false,
            check_for_value_diffs: false,
            check_for_array_diffs: false,
            render_key_diffs: false,
            render_type_diffs: false,
            render_value_diffs: false,
            render_array_diffs: false,
            read_from_file: String::new(),
            write_to_file: None,
            write_to_html: false,
            file_a: None,
            file_b: None,
            array_same_order: false,
        }
    }

    pub fn check_for_key_diffs(mut self, check_for_key_diffs: bool) -> ConfigBuilder {
        self.check_for_key_diffs = check_for_key_diffs;
        self
    }

    pub fn check_for_type_diffs(mut self, check_for_type_diffs: bool) -> ConfigBuilder {
        self.check_for_type_diffs = check_for_type_diffs;
        self
    }

    pub fn check_for_value_diffs(mut self, check_for_value_diffs: bool) -> ConfigBuilder {
        self.check_for_value_diffs = check_for_value_diffs;
        self
    }

    pub fn check_for_array_diffs(mut self, check_for_array_diffs: bool) -> ConfigBuilder {
        self.check_for_array_diffs = check_for_array_diffs;
        self
    }

    pub fn render_key_diffs(mut self, render_key_diffs: bool) -> ConfigBuilder {
        self.render_key_diffs = render_key_diffs;
        self
    }

    pub fn render_type_diffs(mut self, render_type_diffs: bool) -> ConfigBuilder {
        self.render_type_diffs = render_type_diffs;
        self
    }

    pub fn render_value_diffs(mut self, render_value_diffs: bool) -> ConfigBuilder {
        self.render_value_diffs = render_value_diffs;
        self
    }

    pub fn render_array_diffs(mut self, render_array_diffs: bool) -> ConfigBuilder {
        self.render_array_diffs = render_array_diffs;
        self
    }

    pub fn read_from_file(mut self, read_from_file: String) -> ConfigBuilder {
        self.read_from_file = read_from_file;
        self
    }

    pub fn write_to_file(mut self, write_to_file: Option<String>) -> ConfigBuilder {
        self.write_to_file = write_to_file;
        self
    }

    pub fn write_to_html(mut self, write_to_html: bool) -> ConfigBuilder {
        self.write_to_html = write_to_html;
        self
    }

    pub fn file_a(mut self, file_a: Option<String>) -> ConfigBuilder {
        self.file_a = file_a;
        self
    }

    pub fn file_b(mut self, file_b: Option<String>) -> ConfigBuilder {
        self.file_b = file_b;
        self
    }

    pub fn array_same_order(mut self, array_same_order: bool) -> ConfigBuilder {
        self.array_same_order = array_same_order;
        self
    }

    pub fn build(self) -> Config {
        Config {
            check_for_key_diffs: self.check_for_key_diffs,
            check_for_type_diffs: self.check_for_type_diffs,
            check_for_value_diffs: self.check_for_value_diffs,
            check_for_array_diffs: self.check_for_array_diffs,
            render_key_diffs: self.render_key_diffs,
            render_type_diffs: self.render_type_diffs,
            render_value_diffs: self.render_value_diffs,
            render_array_diffs: self.render_array_diffs,
            read_from_file: self.read_from_file,
            write_to_file: self.write_to_file,
            write_to_html: self.write_to_html,
            file_a: self.file_a,
            file_b: self.file_b,
            array_same_order: self.array_same_order,
        }
    }
}

/// Contextual data for the current run
#[derive(Clone)]
pub struct WorkingContext {
    pub lib_working_context: LibWorkingContext,
    pub config: Config,
}

impl WorkingContext {
    pub fn new(lib_working_context: LibWorkingContext, config: Config) -> WorkingContext {
        WorkingContext {
            lib_working_context,
            config,
        }
    }
}

/// How a WorkingContext gets stored on disk
#[derive(Serialize, Deserialize)]
pub struct SavedContext {
    pub key_diff: Vec<KeyDiff>,
    pub type_diff: Vec<TypeDiff>,
    pub value_diff: Vec<ValueDiff>,
    pub array_diff: Vec<ArrayDiff>,
    pub config: SavedConfig,
}

impl SavedContext {
    pub fn new(
        key_diff: Vec<KeyDiff>,
        type_diff: Vec<TypeDiff>,
        value_diff: Vec<ValueDiff>,
        array_diff: Vec<ArrayDiff>,
        config: SavedConfig,
    ) -> SavedContext {
        SavedContext {
            key_diff,
            type_diff,
            value_diff,
            array_diff,
            config,
        }
    }
}

/// Custom Error type
#[derive(Debug)]
pub enum DtfError {
    IoError(std::io::Error),
    DiffError(String),
    GeneralError(Box<DtfError>),
}

impl fmt::Display for DtfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DtfError::IoError(err) => write!(f, "IO error: {}", err),
            DtfError::DiffError(msg) => write!(f, "Diff error: {}", msg),
            DtfError::GeneralError(err) => write!(f, "General error happened {}", err),
        }
    }
}

impl Error for DtfError {}
