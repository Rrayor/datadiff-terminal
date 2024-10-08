use crate::{
    dtfterminal_types::{TableContext, TermTable, WorkingContext},
    utils::{CHECKMARK, MULTIPLY},
};
use colored::{Color, ColoredString, Colorize};
use libdtf::core::diff_types::KeyDiff;
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
};

/// Table to display key differences in the terminal
pub struct KeyTable<'a> {
    context: TableContext<'a>,
}

impl<'a> TermTable<KeyDiff> for KeyTable<'a> {
    fn render(&self) -> String {
        self.context.render()
    }

    fn create_table(&mut self, data: &[KeyDiff]) {
        self.add_header();
        self.add_rows(data);
    }

    fn add_header(&mut self) {
        let (file_name_a_str, file_name_b_str) = self.context.working_context().get_file_names();
        let file_name_a = file_name_a_str.to_owned();
        let file_name_b = file_name_b_str.to_owned();
        self.add_title_row();
        self.add_file_name_row(file_name_a, file_name_b);
    }

    fn add_rows(&mut self, data: &[KeyDiff]) {
        let (file_name_a_str, file_name_b_str) = self.context.working_context().get_file_names();
        let file_name_a = file_name_a_str.to_owned();
        let file_name_b = file_name_b_str.to_owned();
        for kd in data {
            let a_has = self.check_has(file_name_a.as_str(), kd);
            let b_has = self.check_has(file_name_b.as_str(), kd);
            self.context.add_row(Row::new(vec![
                TableCell::new(&kd.key),
                TableCell::new(a_has),
                TableCell::new(b_has),
            ]));
        }
    }
}

impl<'a> KeyTable<'a> {
    pub fn new(data: &[KeyDiff], working_context: &'a WorkingContext) -> KeyTable<'a> {
        let mut table = KeyTable {
            context: TableContext::new(working_context),
        };
        table.create_table(data);
        table
    }

    /// Check if the key is present in the file
    fn check_has(&self, file_name: &str, key_diff: &KeyDiff) -> ColoredString {
        if key_diff.has == file_name {
            CHECKMARK.color(Color::Green)
        } else {
            MULTIPLY.color(Color::Red)
        }
    }

    /// Adds the header row to the table
    fn add_title_row(&mut self) {
        self.context
            .add_row(Row::new(vec![TableCell::builder("Key Differences")
                .col_span(3)
                .alignment(Alignment::Center)]));
    }

    /// Adds the file names row to the table
    fn add_file_name_row(&mut self, file_name_a: String, file_name_b: String) {
        self.context.add_row(Row::new(vec![
            TableCell::new("Key"),
            TableCell::new(file_name_a),
            TableCell::new(file_name_b),
        ]));
    }
}

#[cfg(test)]
mod tests {
    use crate::dtfterminal_types::ConfigBuilder;

    use super::*;

    #[test]
    fn test_check_has() {
        let working_context = get_working_context();
        let key_diff = KeyDiff {
            key: "key1".to_owned(),
            has: "file_a.json".to_owned(),
            misses: "file_b.json".to_owned(),
        };
        let key_table = KeyTable::new(&[], &working_context);
        let result = key_table.check_has("file_a.json", &key_diff);
        assert_eq!(result, CHECKMARK.color(Color::Green));
    }

    fn get_working_context() -> WorkingContext {
        let working_file_a = libdtf::core::diff_types::WorkingFile::new("file_a.json".to_string());
        let working_file_b = libdtf::core::diff_types::WorkingFile::new("file_b.json".to_string());
        let lib_working_context = libdtf::core::diff_types::WorkingContext::new(
            working_file_a,
            working_file_b,
            libdtf::core::diff_types::Config {
                array_same_order: false,
            },
        );
        let working_context =
            WorkingContext::new(lib_working_context, ConfigBuilder::new().build());
        working_context
    }
}
