use std::error::Error;

use crate::{
    array_table::ArrayTable,
    dtfterminal_types::{
        Config, ConfigBuilder, DiffCollection, DtfError, LibConfig, LibWorkingContext, ParsedArgs,
        TermTable, WorkingContext,
    },
    file_handler::FileHandler,
    key_table::KeyTable,
    type_table::TypeTable,
    value_table::ValueTable,
    Arguments,
};

use ::clap::Parser;
use serde_json::{Map, Value};

/// Responsible for the main functionality of the app. Makes sure everything runs in the correct order.
pub struct App {
    json_data1: Option<Map<String, Value>>,
    json_data2: Option<Map<String, Value>>,
    yaml_data1: Option<serde_yaml::Mapping>,
    yaml_data2: Option<serde_yaml::Mapping>,
    diffs: DiffCollection,
    json_context: libdtf::json::diff_types::WorkingContext,
    yaml_context: libdtf::yaml::diff_types::WorkingContext,
    file_handler: FileHandler,
}

impl App {
    /// Creates a new App instance
    /// 1. Parses the command line arguments
    /// 2. Checks for differences and stores them
    pub fn new() -> App {
        let parsed_args = App::parse_args();
        let mut file_handler = FileHandler::new(parsed_args.config.clone(), None);
        let (diffs, context) = if parsed_args.config.read_from_file.is_empty() {
            (
                (None, None, None, None),
                App::create_working_context(&parsed_args.config),
            )
        } else {
            file_handler
                .load_saved_results()
                .expect("Could not load saved file!")
        };
        let mut app = App {
            diffs,
            json_context: context.lib_working_context.clone(),
            yaml_context: context.lib_working_context.clone(),
            file_handler,
            json_data1: parsed_args.json_data1,
            json_data2: parsed_args.json_data2,
            yaml_data1: parsed_args.yaml_data1,
            yaml_data2: parsed_args.yaml_data2,
        };

        app.collect_data(&parsed_args.config);

        app
    }

    /// Handles the output into file or to the terminal
    pub fn execute(&self) -> Result<(), DtfError> {
        if self.context.config.write_to_file.is_some() {
            self.file_handler
                .write_to_file(self.diffs.clone())
                .map_err(|e| DtfError::GeneralError(Box::new(e)))?;
        } else {
            self.render_tables()
                .map_err(|e| DtfError::DiffError(format!("{}", e)))?;
        }

        Ok(())
    }

    fn parse_args() -> ParsedArgs {
        let args = Arguments::parse();

        let (json_data1, json_data2, yaml_data1, yaml_data2) = if args.read_from_file.is_empty() {
            // TODO: Handle JSON and YAML
            let mut yaml_data1;
            let mut yaml_data2;

            let mut json_data1;
            let mut json_data2;

            if args.check_files[0].ends_with(".yaml") || args.check_files[0].ends_with(".yml") {
                yaml_data1 = FileHandler::read_yaml_file(&args.check_files[0])
                    .unwrap_or_else(|_| panic!("Couldn't read file: {}", &args.check_files[0]));
                yaml_data2 = FileHandler::read_yaml_file(&args.check_files[1])
                    .unwrap_or_else(|_| panic!("Couldn't read file: {}", &args.check_files[1]));
            } else {
                json_data1 = FileHandler::read_json_file(&args.check_files[0])
                    .unwrap_or_else(|_| panic!("Couldn't read file: {}", &args.check_files[0]));
                json_data2 = FileHandler::read_json_file(&args.check_files[1])
                    .unwrap_or_else(|_| panic!("Couldn't read file: {}", &args.check_files[1]));
            }


            (Some(json_data1), Some(json_data2), Some(yaml_data1), Some(yaml_data2))
        } else {
            (None, None, None, None)
        };

        let file_a = if json_data1.is_some() {
            Some(args.check_files[0].clone())
        } else if yaml_data1.is_some() {
            Some(args.check_files[0].clone())
        } else {
            None
        };

        let file_b = if json_data2.is_some() {
            Some(args.check_files[1].clone())
        } else if yaml_data2.is_some() {
            Some(args.check_files[1].clone())
        } else {
            None
        };

        let config = ConfigBuilder::new()
            .check_for_key_diffs(args.key_diffs)
            .check_for_type_diffs(args.type_diffs)
            .check_for_value_diffs(args.value_diffs)
            .check_for_array_diffs(args.array_diffs)
            .render_key_diffs(args.key_diffs)
            .render_type_diffs(args.type_diffs)
            .render_value_diffs(args.value_diffs)
            .render_array_diffs(args.array_diffs)
            .read_from_file(args.read_from_file)
            .write_to_file(args.write_to_file)
            .file_a(file_a)
            .file_b(file_b)
            .array_same_order(args.array_same_order)
            .build();

        ParsedArgs::new(json_data1, json_data2, yaml_data1, yaml_data2, config)
    }

    fn collect_data(&mut self, user_config: &Config) {
        if user_config.read_from_file.is_empty() {
            self.diffs = self.perform_new_check().expect("Data check failed!")
        } else {
            self.diffs = self
                .file_handler
                .load_saved_results()
                .expect("Could not load saved file!")
                .0;
        }
    }

    fn perform_new_check(&self) -> Result<DiffCollection, Box<dyn Error>> {
        let diffs = self.check_for_json_diffs(
            self.data1
                .as_ref()
                .ok_or("Contents of first file are missing")?,
            self.data2
                .as_ref()
                .ok_or("Contents of second file are missing")?,
        );

        Ok(diffs)
    }

    fn check_for_json_diffs(
        &self,
        data1: &Map<String, Value>,
        data2: &Map<String, Value>,
    ) -> DiffCollection {
        let key_diff = if self.context.config.check_for_key_diffs {
            let mut checking_data: libdtf::json::diff_types::CheckingData<libdtf::json::diff_types::KeyDiff> =
                libdtf::json::diff_types::CheckingData::new("", data1, data2, &self.context.lib_working_context);
            libdtf::json::diff_types::Checker::check(&mut checking_data);
            Some(libdtf::json::diff_types::Checker::diffs(&checking_data)).cloned()
        } else {
            None
        };
        let type_diff = if self.context.config.check_for_type_diffs {
            let mut checking_data: libdtf::json::diff_types::CheckingData<libdtf::json::diff_types::TypeDiff> =
                libdtf::json::diff_types::CheckingData::new("", data1, data2, &self.context.lib_working_context);
            libdtf::json::diff_types::Checker::check(&mut checking_data);
            Some(libdtf::json::diff_types::Checker::diffs(&checking_data)).cloned()
        } else {
            None
        };
        let value_diff = if self.context.config.check_for_value_diffs {
            let mut checking_data: libdtf::json::diff_types::CheckingData<libdtf::json::diff_types::ValueDiff> =
                libdtf::json::diff_types::CheckingData::new("", data1, data2, &self.context.lib_working_context);
            libdtf::json::diff_types::Checker::check(&mut checking_data);
            Some(libdtf::json::diff_types::Checker::diffs(&checking_data)).cloned()
        } else {
            None
        };
        let array_diff = if self.context.config.check_for_array_diffs {
            let mut checking_data: libdtf::json::diff_types::CheckingData<libdtf::json::diff_types::ArrayDiff> =
                libdtf::json::diff_types::CheckingData::new("", data1, data2, &self.context.lib_working_context);
            libdtf::json::diff_types::Checker::check(&mut checking_data);
            Some(libdtf::json::diff_types::Checker::diffs(&checking_data)).cloned()
        } else {
            None
        };

        (key_diff, type_diff, value_diff, array_diff)
    }

    fn check_for_yaml_diffs(
        &self,
        data1: &serde_yaml::Mapping,
        data2: &serde_yaml::Mapping,
    ) -> DiffCollection {
        let key_diff = if self.context.config.check_for_key_diffs {
            let mut checking_data: libdtf::yaml::diff_types::CheckingData<libdtf::yaml::diff_types::KeyDiff> =
                libdtf::yaml::diff_types::CheckingData::new("", data1, data2, &self.context.lib_working_context);
            checking_data.check();
            Some(checking_data.diffs()).cloned()
        } else {
            None
        };
        let type_diff = if self.context.config.check_for_type_diffs {
            let mut checking_data: libdtf::yaml::diff_types::CheckingData<libdtf::yaml::diff_types::TypeDiff> =
                libdtf::yaml::diff_types::CheckingData::new("", data1, data2, &self.context.lib_working_context);
            checking_data.check();
            Some(checking_data.diffs()).cloned()
        } else {
            None
        };
        let value_diff = if self.context.config.check_for_value_diffs {
            let mut checking_data: libdtf::yaml::diff_types::CheckingData<libdtf::yaml::diff_types::ValueDiff> =
                libdtf::yaml::diff_types::CheckingData::new("", data1, data2, &self.context.lib_working_context);
            checking_data.check();
            Some(checking_data.diffs()).cloned()
        } else {
            None
        };
        let array_diff = if self.context.config.check_for_array_diffs {
            let mut checking_data: libdtf::yaml::diff_types::CheckingData<libdtf::yaml::diff_types::ArrayDiff> =
                libdtf::yaml::diff_types::CheckingData::new("", data1, data2, &self.context.lib_working_context);
            checking_data.check();
            Some(checking_data.diffs()).cloned()
        } else {
            None
        };

        (key_diff, type_diff, value_diff, array_diff)
    }

    fn render_tables(&self) -> Result<(), DtfError> {
        let (key_diff, type_diff, value_diff, array_diff) = &self.diffs;

        let mut rendered_tables = vec![];
        if self.context.config.render_key_diffs {
            if let Some(diffs) = key_diff.as_ref().filter(|kd| !kd.is_empty()) {
                let table = KeyTable::new(diffs, &self.context.lib_working_context);
                rendered_tables.push(table.render());
            }
        }

        if self.context.config.render_type_diffs {
            if let Some(diffs) = type_diff.as_ref().filter(|td| !td.is_empty()) {
                let table = TypeTable::new(diffs, &self.context.lib_working_context);
                rendered_tables.push(table.render());
            }
        }

        if self.context.config.render_value_diffs {
            if let Some(diffs) = value_diff.as_ref().filter(|vd| !vd.is_empty()) {
                let table = ValueTable::new(diffs, &self.context.lib_working_context);
                rendered_tables.push(table.render());
            }
        }

        if self.context.config.render_array_diffs {
            if let Some(diffs) = array_diff.as_ref().filter(|ad| !ad.is_empty()) {
                let table = ArrayTable::new(diffs, &self.context.lib_working_context);
                rendered_tables.push(table.render());
            }
        }

        for table in rendered_tables {
            println!("{}", table);
        }

        Ok(())
    }

    fn create_working_context(config: &Config) -> WorkingContext {
        let file_a = WorkingFile::new(config.file_a.as_ref().unwrap().clone());
        let file_b = WorkingFile::new(config.file_b.as_ref().unwrap().clone());

        let lib_working_context =
            LibWorkingContext::new(file_a, file_b, LibConfig::new(config.array_same_order));

        WorkingContext::new(lib_working_context, config.clone())
    }
}
