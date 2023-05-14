use std::{fs::File, io::BufReader};

use clap::{ArgGroup, Parser};
use colored::{Color, ColoredString, Colorize};
use libdtf::{
    diff_types, find_array_diffs, find_key_diffs, find_type_diffs, find_value_diffs, read_json_file,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableStyle,
};

use diff_types::{ArrayDiff, ArrayDiffDesc, KeyDiff, TypeDiff, ValueDiff, WorkingFile};

pub type LibConfig = libdtf::diff_types::Config;
pub type LibWorkingContext = libdtf::diff_types::WorkingContext;

#[derive(Serialize, Deserialize)]
pub struct SavedConfig {
    check_for_key_diffs: bool,
    check_for_type_diffs: bool,
    check_for_value_diffs: bool,
    check_for_array_diffs: bool,
    file_a: String,
    file_b: String,
    array_same_order: bool,
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

#[derive(Clone)]
pub struct Config {
    check_for_key_diffs: bool,
    check_for_type_diffs: bool,
    check_for_value_diffs: bool,
    check_for_array_diffs: bool,
    pub read_from_file: String,
    pub write_to_file: Option<String>,
    pub file_a: Option<String>,
    pub file_b: Option<String>,
    pub array_same_order: bool,
}

impl Config {
    pub fn new(
        check_for_key_diffs: bool,
        check_for_type_diffs: bool,
        check_for_value_diffs: bool,
        check_for_array_diffs: bool,
        read_from_file: String,
        write_to_file: Option<String>,
        file_a: Option<String>,
        file_b: Option<String>,
        array_same_order: bool,
    ) -> Config {
        Config {
            check_for_key_diffs,
            check_for_type_diffs,
            check_for_value_diffs,
            check_for_array_diffs,
            read_from_file,
            write_to_file,
            file_a,
            file_b,
            array_same_order,
        }
    }
}

#[derive(Clone)]
pub struct WorkingContext {
    lib_working_context: LibWorkingContext,
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

#[derive(Default, Parser, Debug)]
#[clap(
    version,
    about,
    group(
        ArgGroup::new("diff-options")
            .required(true)
            .multiple(true)
            .args(&["key_diffs", "type_diffs", "value_diffs", "array_diffs"]),
    ),
    group(
        ArgGroup::new("file-options")
        .required(true)
        .args(&["check_files", "read_from_file"])
    )
)]
/// Find the difference in your data structures
struct Arguments {
    /// The files to check if not reading from saved check
    #[clap(short, value_delimiter = ' ', num_args = 2)]
    check_files: Vec<String>,
    /// Read from a JSON file created on previous check instead of checking again
    #[clap(short, default_value_t = String::new())]
    read_from_file: String,

    /// Output to json file instead of rendering tables in the terminal
    #[clap(short)]
    write_to_file: Option<String>,

    /// Check for Key differences
    #[clap(short, default_value_t = false)]
    key_diffs: bool,
    /// Check for Type differences
    #[clap(short, default_value_t = false)]
    type_diffs: bool,
    /// Check for Value differences
    #[clap(short, default_value_t = false)]
    value_diffs: bool,
    /// Check for Array differences
    #[clap(short, default_value_t = false)]
    array_diffs: bool,

    /// Do you want arrays to be the same order? If defined you will get Value differences with indexes, otherwise you will get array differences, that tell you which object contains or misses values.
    #[clap(short = 'o', default_value_t = false)]
    array_same_order: bool,
}

const CHECKMARK: &str = "\u{2713}";
const MULTIPLY: &str = "\u{00D7}";

pub fn parse_args() -> (
    Option<Map<String, Value>>,
    Option<Map<String, Value>>,
    Config,
) {
    let args = Arguments::parse();

    let data1 = if args.read_from_file.is_empty() {
        Some(
            read_json_file(&args.check_files[0])
                .unwrap_or_else(|_| panic!("Couldn't read file: {}", &args.check_files[0])),
        )
    } else {
        None
    };

    let data2 = if args.read_from_file.is_empty() {
        Some(
            read_json_file(&args.check_files[1])
                .unwrap_or_else(|_| panic!("Couldn't read file: {}", &args.check_files[1])),
        )
    } else {
        None
    };

    let file_a = if args.read_from_file.is_empty() {
        Some(args.check_files[0].clone())
    } else {
        None
    };

    let file_b = if args.read_from_file.is_empty() {
        Some(args.check_files[1].clone())
    } else {
        None
    };

    let config = Config::new(
        args.key_diffs,
        args.type_diffs,
        args.value_diffs,
        args.array_diffs,
        args.read_from_file,
        args.write_to_file,
        file_a,
        file_b,
        args.array_same_order,
    );

    (data1, data2, config)
}

pub fn create_working_context(config: &Config) -> WorkingContext {
    if config.read_from_file.is_empty() {
        let file_a = WorkingFile::new(config.file_a.clone().unwrap());
        let file_b = WorkingFile::new(config.file_b.clone().unwrap());

        let lib_working_context =
            LibWorkingContext::new(file_a, file_b, LibConfig::new(config.array_same_order));

        WorkingContext::new(lib_working_context, config.clone())
    } else {
        let saved_config = read_from_file(&config.read_from_file).unwrap().config;
        let file_a = WorkingFile::new(saved_config.file_a.clone());
        let file_b = WorkingFile::new(saved_config.file_b.clone());
        let lib_working_context = LibWorkingContext::new(
            file_a,
            file_b,
            LibConfig::new(saved_config.array_same_order),
        );

        WorkingContext::new(
            lib_working_context,
            Config::new(
                saved_config.check_for_key_diffs,
                saved_config.check_for_type_diffs,
                saved_config.check_for_value_diffs,
                saved_config.check_for_array_diffs,
                config.read_from_file.clone(),
                config.write_to_file.clone(),
                Some(saved_config.file_a),
                Some(saved_config.file_b),
                saved_config.array_same_order,
            ),
        )
    }
}

pub fn collect_data(
    data1: &Map<String, Value>,
    data2: &Map<String, Value>,
    working_context: &WorkingContext,
) -> (
    Option<Vec<KeyDiff>>,
    Option<Vec<TypeDiff>>,
    Option<Vec<ValueDiff>>,
    Option<Vec<ArrayDiff>>,
) {
    let key_diff = working_context
        .config
        .check_for_key_diffs
        .then(|| find_key_diffs("", data1, data2, &working_context.lib_working_context));
    let type_diff = working_context
        .config
        .check_for_type_diffs
        .then(|| find_type_diffs("", data1, data2, &working_context.lib_working_context));
    let value_diff = working_context
        .config
        .check_for_value_diffs
        .then(|| find_value_diffs("", data1, data2, &working_context.lib_working_context));
    let array_diff = working_context
        .config
        .check_for_array_diffs
        .then(|| find_array_diffs("", data1, data2, &working_context.lib_working_context));

    (key_diff, type_diff, value_diff, array_diff)
}

pub fn read_from_file(file_path: &str) -> serde_json::Result<SavedContext> {
    let file =
        File::open(&file_path).unwrap_or_else(|_| panic!("Could not open file {}", &file_path));
    let reader = BufReader::new(file);
    serde_json::from_reader(reader)
}

pub fn write_to_file(
    key_diff_option: Option<Vec<KeyDiff>>,
    type_diff_option: Option<Vec<TypeDiff>>,
    value_diff_option: Option<Vec<ValueDiff>>,
    array_diff_option: Option<Vec<ArrayDiff>>,
    working_context: &WorkingContext,
) -> Result<(), ()> {
    println!("HERE");
    let key_diff = key_diff_option.unwrap_or(vec![]);
    let type_diff = type_diff_option.unwrap_or(vec![]);
    let value_diff = value_diff_option.unwrap_or(vec![]);
    let array_diff = array_diff_option.unwrap_or(vec![]);

    if let Some(write_to_file) = working_context.config.write_to_file.clone() {
        let file = File::create(write_to_file);
        let config = &working_context.config;

        match serde_json::to_writer(
            &mut file.unwrap(),
            &SavedContext::new(
                key_diff,
                type_diff,
                value_diff,
                array_diff,
                SavedConfig::new(
                    config.check_for_key_diffs,
                    config.check_for_type_diffs,
                    config.check_for_value_diffs,
                    config.check_for_array_diffs,
                    config.file_a.clone().unwrap(),
                    config.file_b.clone().unwrap(),
                    config.array_same_order,
                ),
            ),
        ) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    } else {
        Err(())
    }
}

pub fn render_tables(
    key_diff: Option<Vec<KeyDiff>>,
    type_diff: Option<Vec<TypeDiff>>,
    value_diff: Option<Vec<ValueDiff>>,
    array_diff: Option<Vec<ArrayDiff>>,
    working_context: &WorkingContext,
) -> Result<(), ()> {
    key_diff.filter(|kd| !kd.is_empty()).map(|diffs| {
        let table = create_table_key_diff(&diffs, &working_context.lib_working_context);
        println!("{}", table.render());
    });

    type_diff.filter(|td| !td.is_empty()).map(|diffs| {
        let table = create_table_type_diff(&diffs, &working_context.lib_working_context);
        println!("{}", table.render());
    });

    value_diff.filter(|vd| !vd.is_empty()).map(|diffs| {
        let table = create_table_value_diff(&diffs, &working_context.lib_working_context);
        println!("{}", table.render());
    });

    array_diff.filter(|ad| !ad.is_empty()).map(|diffs| {
        let table = create_table_array_diff(&diffs, &working_context.lib_working_context);
        println!("{}", table.render());
    });
    Ok(())
}

// Key table

fn create_table_key_diff<'a>(
    data: &Vec<KeyDiff>,
    working_context: &LibWorkingContext,
) -> Table<'a> {
    let mut table = Table::new();
    table.max_column_width = 80;
    table.style = TableStyle::extended();

    add_key_table_header(&mut table, working_context);
    add_key_table_rows(&mut table, &data, working_context);

    table
}

fn add_key_table_header(table: &mut Table, working_context: &LibWorkingContext) {
    table.add_row(Row::new(vec![TableCell::new_with_alignment(
        "Key Differences",
        3,
        Alignment::Center,
    )]));
    table.add_row(Row::new(vec![
        TableCell::new("Key"),
        TableCell::new(&working_context.file_a.name),
        TableCell::new(&working_context.file_b.name),
    ]));
}

fn add_key_table_rows(table: &mut Table, data: &[KeyDiff], working_context: &LibWorkingContext) {
    for kd in data {
        table.add_row(Row::new(vec![
            TableCell::new(&kd.key),
            TableCell::new(check_has(&working_context.file_a.name, kd)),
            TableCell::new(check_has(&working_context.file_b.name, kd)),
        ]));
    }
}

fn check_has(file_name: &str, key_diff: &KeyDiff) -> ColoredString {
    if key_diff.has == file_name {
        CHECKMARK.color(Color::Green)
    } else {
        MULTIPLY.color(Color::Red)
    }
}

// Type table

fn create_table_type_diff<'a>(
    data: &Vec<TypeDiff>,
    working_context: &LibWorkingContext,
) -> Table<'a> {
    let mut table = Table::new();
    table.max_column_width = 80;
    table.style = TableStyle::extended();

    add_type_table_header(&mut table, working_context);
    add_type_table_rows(&mut table, &data);

    table
}

fn add_type_table_header(table: &mut Table, working_context: &LibWorkingContext) {
    table.add_row(Row::new(vec![TableCell::new_with_alignment(
        "Type Differences",
        3,
        Alignment::Center,
    )]));
    table.add_row(Row::new(vec![
        TableCell::new("Key"),
        TableCell::new(&working_context.file_a.name),
        TableCell::new(&working_context.file_b.name),
    ]));
}

fn add_type_table_rows(table: &mut Table, data: &[TypeDiff]) {
    for td in data {
        table.add_row(Row::new(vec![
            TableCell::new(&td.key),
            TableCell::new(&td.type1),
            TableCell::new(&td.type2),
        ]));
    }
}

// Value table

fn create_table_value_diff<'a>(
    data: &Vec<ValueDiff>,
    working_context: &LibWorkingContext,
) -> Table<'a> {
    let mut table = Table::new();
    table.max_column_width = 80;
    table.style = TableStyle::extended();

    add_value_table_header(&mut table, working_context);
    add_value_table_rows(&mut table, &data);

    table
}

fn add_value_table_header(table: &mut Table, working_context: &LibWorkingContext) {
    table.add_row(Row::new(vec![TableCell::new_with_alignment(
        "Value Differences",
        3,
        Alignment::Center,
    )]));
    table.add_row(Row::new(vec![
        TableCell::new("Key"),
        TableCell::new(&working_context.file_a.name),
        TableCell::new(&working_context.file_b.name),
    ]));
}

fn add_value_table_rows(table: &mut Table, data: &Vec<ValueDiff>) {
    for vd in data {
        table.add_row(Row::new(vec![
            TableCell::new(&vd.key),
            TableCell::new(&sanitize_json_str(&vd.value1)),
            TableCell::new(&sanitize_json_str(&vd.value2)),
        ]));
    }
}

// Array table

fn create_table_array_diff<'a>(
    data: &Vec<ArrayDiff>,
    working_context: &LibWorkingContext,
) -> Table<'a> {
    let mut table = Table::new();
    table.max_column_width = 80;
    table.style = TableStyle::extended();

    add_array_table_header(&mut table, working_context);
    add_array_table_rows(&mut table, &data);

    table
}

fn add_array_table_header(table: &mut Table, working_context: &LibWorkingContext) {
    table.add_row(Row::new(vec![TableCell::new_with_alignment(
        "Array Differences",
        3,
        Alignment::Center,
    )]));
    table.add_row(Row::new(vec![
        TableCell::new("Key"),
        TableCell::new(&working_context.file_a.name),
        TableCell::new(&working_context.file_b.name),
    ]));
}

fn add_array_table_rows(table: &mut Table, data: &Vec<ArrayDiff>) {
    for ad in data {
        let value_str = sanitize_json_str(&ad.value);
        table.add_row(Row::new(vec![
            TableCell::new(&ad.key),
            TableCell::new(get_array_table_cell_value(&ad.descriptor, &value_str)),
            TableCell::new(get_array_table_cell_value(&ad.descriptor, &value_str)),
        ]));
    }
}

fn get_array_table_cell_value<'a>(descriptor: &'a ArrayDiffDesc, value_str: &'a str) -> &'a str {
    match descriptor {
        ArrayDiffDesc::AHas => value_str,
        ArrayDiffDesc::AMisses => value_str,
        ArrayDiffDesc::BHas => value_str,
        ArrayDiffDesc::BMisses => value_str,
    }
}

// Utils

fn sanitize_json_str(json_str: &str) -> String {
    match serde_json::from_str::<Value>(json_str) {
        Ok(json_value) => serde_json::to_string_pretty(&json_value).unwrap_or(json_str.to_owned()),
        Err(_) => json_str.to_owned(),
    }
}
