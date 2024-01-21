use serde_json::Value;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct PrimitiveField {
    pub name: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct ObjectField {
    pub name: String,
    pub fields: Vec<FieldType>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct ArrayField {
    pub name: String,
    // TODO: replace FieldType with something that does not require a name
    pub field: Box<FieldType>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum FieldType {
    Any(PrimitiveField),
    Bool(PrimitiveField),
    String(PrimitiveField),
    Int(PrimitiveField),
    Float(PrimitiveField),
    Object(ObjectField),
    Array(ArrayField),
}

fn same_type<'a>(a: &FieldType, b: &'a FieldType) -> Option<&'a FieldType> {
    match (a, b) {
        (FieldType::Any(a), out @ FieldType::Any(b))
        | (FieldType::Bool(a), out @ FieldType::Bool(b))
        | (FieldType::String(a), out @ FieldType::String(b))
        | (FieldType::Int(a), out @ FieldType::Int(b))
        | (FieldType::Float(a), out @ FieldType::Float(b)) => (a.name == b.name).then_some(out),
        (FieldType::Array(a), out @ FieldType::Array(b)) => (a.name == b.name)
            .then_some(out)
            .and_then(|_| same_type(&a.field, &b.field)),
        (FieldType::Object(a), out @ FieldType::Object(b)) => (a.name == b.name
            && a.fields.len() == b.fields.len()
            && a.fields
                .iter()
                .zip(b.fields.iter())
                .all(|(a, b)| same_type(a, b).is_some()))
        .then_some(out),
        _ => None,
    }
}

pub(crate) fn parse_value(name: &str, value: serde_json::Value) -> FieldType {
    match value {
        Value::Object(entries) => FieldType::Object(ObjectField {
            name: name.to_string(),
            fields: entries
                .iter()
                .map(|(name, value)| parse_value(name, value.clone()))
                .collect::<Vec<FieldType>>(),
        }),
        Value::Array(elements) => {
            if elements.is_empty() {
                return FieldType::Array(ArrayField {
                    name: name.to_string(),
                    field: Box::new(FieldType::Any(PrimitiveField {
                        name: String::new(),
                    })),
                });
            }

            let parsed_elements = elements
                .iter()
                .map(|e| parse_value("", e.clone()))
                .collect::<Vec<FieldType>>();

            let split = parsed_elements.split_first();
            let Some((first_element, parsed_elements)) = split else {
                return FieldType::Array(ArrayField {
                    name: name.to_string(),
                    field: Box::new(FieldType::Any(PrimitiveField {
                        name: String::new(),
                    })),
                });
            };
            parsed_elements
                .iter()
                .try_fold(first_element, |acc, a| same_type(acc, a))
                .map_or_else(
                    || {
                        FieldType::Array(ArrayField {
                            name: name.to_string(),
                            field: Box::new(FieldType::Any(PrimitiveField {
                                name: String::new(),
                            })),
                        })
                    },
                    |array_type| {
                        FieldType::Array(ArrayField {
                            name: name.to_string(),
                            field: Box::new(array_type.clone()),
                        })
                    },
                )
        }
        Value::Null => FieldType::Any(PrimitiveField {
            name: name.to_string(),
        }),
        Value::Bool(_) => FieldType::Bool(PrimitiveField {
            name: name.to_string(),
        }),
        Value::Number(n) => {
            if n.is_i64() {
                FieldType::Int(PrimitiveField {
                    name: name.to_string(),
                })
            } else {
                FieldType::Float(PrimitiveField {
                    name: name.to_string(),
                })
            }
        }
        Value::String(_) => FieldType::String(PrimitiveField {
            name: name.to_string(),
        }),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    #[rstest]
    #[case::string(
        serde_json::json!(""),
        FieldType::String(PrimitiveField {
            name: "test".to_string(),
        })
    )]
    #[case::empty_array(
        serde_json::json!([]),
        FieldType::Array(ArrayField{
            name: "test".to_string(),
            field: Box::new(FieldType::Any(PrimitiveField {
                name: "".to_string()
            }))
        })
    )]
    #[case::heterogenous_array(
        serde_json::json!([1, "yes", true]),
        FieldType::Array(ArrayField{
            name: "test".to_string(),
            field: Box::new(FieldType::Any(PrimitiveField {
                name: "".to_string()
            }))
        })
    )]
    #[case::object(
        serde_json::json!({
            "name": "John Doe",
            "age": 43,
            "height": 1.8,
            "is_admin": true,
            "phones": [
                "+61 123 456 789",
                "+61 234 567 890"
            ],
            "spouse": {
                "name": "Jill Doe",
                "age": 44,
                "height": 1.7,
                "is_admin": true,
                "phones": [
                    "+61 987 654 321",
                    "+61 876 543 210"
                ]
            },
            "children": [
                {
                    "name": "Jill Doe",
                    "age": 13,
                    "height": 1.5,
                    "phones": [
                        "+61 987 654 321",
                        "+61 876 543 210"
                    ]
                },
                {
                    "name": "Jim Doe",
                    "age": 11,
                    "height": 1.2,
                    "phones": [
                        "+61 111 222 333",
                        "+61 444 555 666"
                    ]
                }
            ],
        }),
        FieldType::Object(ObjectField {
            name: "test".to_string(),
            fields: vec![
                FieldType::String(PrimitiveField {
                    name: "name".to_string(),
                }),
                FieldType::Int(PrimitiveField {
                    name: "age".to_string(),
                }),
                FieldType::Float(PrimitiveField {
                    name: "height".to_string(),
                }),
                FieldType::Bool(PrimitiveField {
                    name: "is_admin".to_string(),
                }),
                FieldType::Array(ArrayField {
                    name: "phones".to_string(),
                    field: Box::new(FieldType::String(PrimitiveField {
                        name: "".to_string()
                    })),
                }),
                FieldType::Object(ObjectField {
                    name: "spouse".to_string(),
                    fields: vec![
                        FieldType::String(PrimitiveField {
                            name: "name".to_string(),
                        }),
                        FieldType::Int(PrimitiveField {
                            name: "age".to_string(),
                        }),
                        FieldType::Float(PrimitiveField {
                            name: "height".to_string(),
                        }),
                        FieldType::Bool(PrimitiveField {
                            name: "is_admin".to_string(),
                        }),
                        FieldType::Array(ArrayField {
                            name: "phones".to_string(),
                            field: Box::new(FieldType::String(PrimitiveField {
                                name: "".to_string()
                            })),
                        }),
                    ],
                }),
                FieldType::Array(ArrayField {
                    name: "children".to_string(),
                    field: Box::new(FieldType::Object(ObjectField {
                        name: "".to_string(),
                        fields: vec![
                            FieldType::String(PrimitiveField {
                                name: "name".to_string(),
                            }),
                            FieldType::Int(PrimitiveField {
                                name: "age".to_string(),
                            }),
                            FieldType::Float(PrimitiveField {
                                name: "height".to_string(),
                            }),
                            FieldType::Array(ArrayField {
                                name: "phones".to_string(),
                                field: Box::new(FieldType::String(PrimitiveField {
                                    name: "".to_string()
                                })),
                            }),
                        ]
                    }))
                })
            ]
        })
    )]
    fn test_parse_json(#[case] value: serde_json::Value, #[case] expected: FieldType) {
        let parsed_json = parse_value("test", value);
        assert_eq!(parsed_json, expected);
    }
}
