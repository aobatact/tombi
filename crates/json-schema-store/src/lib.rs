mod accessor;
mod schema;
mod store;
mod value_type;

pub use accessor::{Accessor, Accessors};
pub use schema::ValueSchema;
use schema::{DocumentSchema, SchemaType};
pub use store::Store;
pub use value_type::ValueType;

#[derive(Debug)]
pub struct KeysValueInfo {
    accessors: Accessors,
    value_type: ValueType,
}

impl KeysValueInfo {
    pub fn accessors(&self) -> &Accessors {
        &self.accessors
    }

    pub fn value_type(&self) -> &ValueType {
        &self.value_type
    }
}

pub fn get_keys_value_info(
    root: document_tree::Root,
    keys: &[document_tree::Key],
    position: text::Position,
) -> Option<KeysValueInfo> {
    let mut accessors = Vec::new();
    let mut value_type = None;
    let table: document_tree::Table = root.into();
    let mut table_ref = &table;

    for key in keys {
        accessors.push(Accessor::Key(key.to_string()));
        if let Some(value) = table_ref.get(key) {
            if let Some(table) = get_item_table(value, &mut accessors, &mut value_type, position) {
                table_ref = table;
            }
        }
    }

    if let Some(value_type) = value_type {
        Some(KeysValueInfo {
            accessors: Accessors::new(accessors),
            value_type,
        })
    } else {
        None
    }
}

fn get_item_table<'a>(
    value: &'a document_tree::Value,
    accessors: &mut Vec<Accessor>,
    value_type: &mut Option<ValueType>,
    position: text::Position,
) -> Option<&'a document_tree::Table> {
    use document_tree::ArrayKind::*;
    use document_tree::Value;

    match value {
        Value::Boolean(_) => {
            *value_type = Some(ValueType::Boolean);
            None
        }
        Value::Integer(_) => {
            *value_type = Some(ValueType::Integer);
            None
        }
        Value::Float(_) => {
            *value_type = Some(ValueType::Float);
            None
        }
        Value::String(_) => {
            *value_type = Some(ValueType::String);
            None
        }
        Value::OffsetDateTime(_) => {
            *value_type = Some(ValueType::OffsetDateTime);
            None
        }
        Value::LocalDateTime(_) => {
            *value_type = Some(ValueType::LocalDateTime);
            None
        }
        Value::LocalDate(_) => {
            *value_type = Some(ValueType::LocalDate);
            None
        }
        Value::LocalTime(_) => {
            *value_type = Some(ValueType::LocalTime);
            None
        }
        Value::Array(array) => {
            let mut index = 0;
            *value_type = Some(ValueType::Array);

            for value in array.values() {
                if value.range().contains(position) {
                    accessors.push(Accessor::Index(index));
                    let table_ref = get_item_table(value, accessors, value_type, position);

                    match array.kind() {
                        ArrayOfTables | ParentArrayOfTables => {
                            *value_type = Some(ValueType::Array);
                        }
                        Array => {}
                    }

                    return table_ref;
                }
                index += 1;
            }
            None
        }
        Value::Table(tbl) => {
            *value_type = Some(ValueType::Table);
            Some(tbl)
        }
    }
}

pub fn parse_document_schema(mut content: serde_json::Value) -> DocumentSchema {
    let mut schema = DocumentSchema::default();

    if content.get("properties").is_some() {
        if let serde_json::Value::Object(object) = content["properties"].take() {
            for (key, value) in object.into_iter() {
                if let Some(value_schema) = parse_value_schema(value) {
                    schema.insert_property(Accessor::Key(key.to_string()), value_schema);
                }
            }
        }
    }
    if content.get("definitions").is_some() {
        if let serde_json::Value::Object(object) = content["definitions"].take() {
            for (key, value) in object.into_iter() {
                if let Some(value_schema) = parse_value_schema(value) {
                    schema.insert_definition(key, value_schema);
                }
            }
        }
    }
    schema
}

fn parse_value_schema(object: serde_json::Value) -> Option<ValueSchema> {
    match object {
        serde_json::Value::Object(object) => {
            let mut schema = ValueSchema::default();

            for (key, value) in object {
                match key.as_str() {
                    "title" => {
                        if let serde_json::Value::String(title) = value {
                            schema.title = Some(title);
                        }
                    }
                    "description" => {
                        if let serde_json::Value::String(description) = value {
                            schema.description = Some(description);
                        }
                    }
                    "type" => {
                        if let serde_json::Value::String(type_str) = value {
                            let schema_type = match type_str.as_str() {
                                "null" => SchemaType::Null,
                                "boolean" => SchemaType::Boolean,
                                "number" => SchemaType::Numeric,
                                "string" => SchemaType::String,
                                "array" => SchemaType::Array,
                                "object" => SchemaType::Object,
                                _ => continue,
                            };
                            schema.types.push(schema_type);
                        }
                    }
                    "default" => {
                        if let serde_json::Value::String(default) = value {
                            schema.default = Some(default);
                        }
                    }
                    "enum" => {
                        if let serde_json::Value::Array(array) = value {
                            for value in array {
                                if let serde_json::Value::String(enum_value) = value {
                                    schema.enum_values.push(enum_value);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            Some(schema)
        }
        _ => None,
    }
}
