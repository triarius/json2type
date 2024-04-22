use crate::parse::{FieldType, ObjectField};

pub(crate) fn type_string(obj: &ObjectField) -> String {
    let mut s = format!("type {} struct {{\n", obj.name);
    for field in &obj.fields {
        s.push_str(&field_node("", field));
    }
    s.push_str("}\n");
    s
}

fn field_node(indent: &str, field: &FieldType) -> String {
    match field {
        FieldType::Any(field) => format!(
            "{indent}\t{} any `json:\"{}\"`\n",
            field_name(&field.name),
            field.name
        ),
        FieldType::Bool(field) => format!(
            "{indent}\t{} bool `json:\"{}\"`\n",
            field_name(&field.name),
            field.name
        ),
        FieldType::String(field) => format!(
            "{indent}\t{} string `json:\"{}\"`\n",
            field_name(&field.name),
            field.name
        ),
        FieldType::Int(field) => format!(
            "{indent}\t{} int `json:\"{}\"`\n",
            field_name(&field.name),
            field.name
        ),
        FieldType::Float(field) => format!(
            "{indent}\t{} float64 `json:\"{}\"`\n",
            field_name(&field.name),
            field.name
        ),
        FieldType::Array(field) => format!(
            "{indent}\t{} []{} `json:\"{}\"`\n",
            field_name(&field.name),
            field_type(&field.field),
            field.name,
        ),
        FieldType::Object(field) => {
            let mut s = if field.name.is_empty() {
                "struct {\n".to_string()
            } else {
                format!("{indent}\t{} struct {{\n", field_name(&field.name))
            };
            for field in &field.fields {
                s.push_str(&field_node(&format!("{indent}\t"), field));
            }
            s.push_str(&format!("{indent}\t}}"));
            if !field.name.is_empty() {
                s.push_str(&format!(" `json:\"{}\"`\n", field.name));
            }
            s
        }
    }
}

fn field_type(field: &FieldType) -> String {
    match field {
        FieldType::Any(_) => "any".to_string(),
        FieldType::Bool(_) => "bool".to_string(),
        FieldType::String(_) => "string".to_string(),
        FieldType::Int(_) => "int".to_string(),
        FieldType::Float(_) => "float64".to_string(),
        FieldType::Array(field) => format!("[]{}", field_type(&field.field)),
        FieldType::Object(_) => field_node("", field),
    }
}

fn field_name(name: &str) -> String {
    name.split('_')
        .map(|s| {
            if s.is_empty() {
                s.to_string()
            } else {
                let mut c = s.chars();
                match c.next() {
                    None => s.to_string(),
                    Some(f) => {
                        let mut s = c.as_str().to_string();
                        s.insert(0, f.to_ascii_uppercase());
                        format_initialism(&s)
                    }
                }
            }
        })
        .collect()
}

// TODO: account for mixed case initialism, e.g. DDoS
fn format_initialism(s: &str) -> String {
    match s.to_ascii_lowercase().as_str() {
        "id" | "url" | "ip" | "json" | "html" | "xml" | "css" | "jwt" | "uuid" | "uri" => {
            s.to_ascii_uppercase()
        }
        _ => s.to_string(),
    }
}

#[cfg(test)]
mod test {
    use crate::parse;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    #[rstest]
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
        r#"type test struct {
	Name string `json:"name"`
	Age int `json:"age"`
	Height float64 `json:"height"`
	IsAdmin bool `json:"is_admin"`
	Phones []string `json:"phones"`
	Spouse struct {
		Name string `json:"name"`
		Age int `json:"age"`
		Height float64 `json:"height"`
		IsAdmin bool `json:"is_admin"`
		Phones []string `json:"phones"`
	} `json:"spouse"`
	Children []struct {
		Name string `json:"name"`
		Age int `json:"age"`
		Height float64 `json:"height"`
		Phones []string `json:"phones"`
	} `json:"children"`
}
"#
    )]
    fn test_parse_json(#[case] value: serde_json::Value, #[case] expected: &str) {
        let s = parse::parse_value("test", value);
        if let parse::FieldType::Object(obj) = s {
            let go_struct = super::type_string(&obj);
            assert_eq!(go_struct, expected.to_string());
        } else {
            panic!("Top level must be an object");
        }
    }
}
