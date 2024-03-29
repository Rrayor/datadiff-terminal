use libdtf::core::diff_types::ValueDiff;
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
};

use crate::{
    dtfterminal_types::{LibWorkingContext, TableContext, TermTable}, prettify_data
};

pub struct ValueTable<'a> {
    context: TableContext<'a>,
}

impl<'a> TermTable<ValueDiff> for ValueTable<'a> {
    fn render(&self) -> String {
        self.context.render()
    }

    fn create_table(&mut self, data: &[ValueDiff]) {
        self.add_header();
        self.add_rows(data);
    }

    fn add_header(&mut self) {
        let (file_name_a_str, file_name_b_str) = self.get_file_names();
        let file_name_a = file_name_a_str.to_owned();
        let file_name_b = file_name_b_str.to_owned();
        self.context
            .add_row(Row::new(vec![TableCell::new_with_alignment(
                "Value Differences",
                3,
                Alignment::Center,
            )]));
        self.context.add_row(Row::new(vec![
            TableCell::new("Key"),
            TableCell::new(file_name_a),
            TableCell::new(file_name_b),
        ]));
    }

    fn add_rows(&mut self, data: &[ValueDiff]) {
        for vd in data {
            self.context.add_row(Row::new(vec![
                TableCell::new(&vd.key),
                TableCell::new(prettify_data(self.get_file_names(), &vd.value1)),
                TableCell::new(prettify_data(self.get_file_names(), &vd.value2)),
            ]));
        }
    }
}

impl<'a> ValueTable<'a> {
    pub fn new(data: &[ValueDiff], working_context: &'a LibWorkingContext) -> ValueTable<'a> {
        let mut table = ValueTable {
            context: TableContext::new(working_context),
        };
        table.create_table(data);
        table
    }

    fn get_file_names(&self) -> (&str, &str) {
        let file_name_a = self.context.working_context().file_a.name.as_str();
        let file_name_b = self.context.working_context().file_b.name.as_str();
        (file_name_a, file_name_b)
    }
}
