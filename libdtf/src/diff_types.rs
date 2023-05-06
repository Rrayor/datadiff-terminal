use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ValueType {
    Null,
    Boolean,
    Number,
    String,
    Array,
    Object,
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value_type_str = match self {
            ValueType::Null => "null",
            ValueType::Boolean => "bool",
            ValueType::Number => "number",
            ValueType::String => "string",
            ValueType::Array => "array",
            ValueType::Object => "object",
        };
        write!(f, "{}", value_type_str)
    }
}

#[derive(PartialEq, Debug)]
pub enum ArrayDiffDesc {
    AHas,
    AMisses,
    BHas,
    BMisses,
}

pub struct WorkingFile {
    pub name: String,
}

pub struct WorkingContext {
    pub file_a: WorkingFile,
    pub file_b: WorkingFile,
}

#[derive(PartialEq, Debug)]
pub struct KeyDiff {
    pub key: String,
    pub has: String,
    pub misses: String,
}

#[derive(PartialEq, Debug)]
pub struct ValueDiff {
    pub key: String,
    pub value1: String, // TODO: would be better as Option
    pub value2: String,
}

#[derive(PartialEq, Debug)]
pub struct ArrayDiff {
    pub key: String,
    pub descriptor: ArrayDiffDesc,
    pub value: String,
}

#[derive(PartialEq)]
pub struct TypeDiff {
    pub key: String,
    pub type1: String,
    pub type2: String,
}

pub type ComparisionResult = (Vec<KeyDiff>, Vec<TypeDiff>, Vec<ValueDiff>, Vec<ArrayDiff>);
