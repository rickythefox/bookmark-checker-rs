use crate::Bookmark;
use serde_json::Value;

pub(crate) fn parse_bookmarks(data: &str) -> Result<Vec<Bookmark>, serde_json::Error> {
    let value: Value = serde_json::from_str(data)?;
    Ok(extract_bookmarks(&value))
}

fn extract_bookmarks(value: &Value) -> Vec<Bookmark> {
    let mut collected = Vec::new();
    collect_nodes(value, &mut collected);
    collected
}

fn collect_nodes(node: &Value, collected: &mut Vec<Bookmark>) {
    if let Some(object) = node.as_object() {
        if object.get("type").and_then(Value::as_str) == Some("url") {
            if let (Some(name), Some(url)) = (
                object.get("name").and_then(Value::as_str),
                object.get("url").and_then(Value::as_str),
            ) {
                collected.push(Bookmark {
                    name: name.to_string(),
                    url: url.to_string(),
                });
            }
        }

        if let Some(children) = object.get("children").and_then(Value::as_array) {
            for child in children {
                collect_nodes(child, collected);
            }
        }

        for (key, value) in object {
            if key != "children" {
                collect_nodes(value, collected);
            }
        }
    } else if let Some(array) = node.as_array() {
        for value in array {
            collect_nodes(value, collected);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_nodes() {
        let data = r#"{
            "roots": {
                "bookmark_bar": {
                    "children": [
                        {
                            "type": "url",
                            "name": "Example",
                            "url": "https://example.com"
                        },
                        {
                            "type": "folder",
                            "children": [
                                {
                                    "type": "url",
                                    "name": "Nested",
                                    "url": "https://nested.example.com"
                                }
                            ]
                        }
                    ]
                }
            }
        }"#;

        let bookmarks = parse_bookmarks(data).expect("should parse");
        assert_eq!(
            bookmarks,
            vec![
                Bookmark {
                    name: "Example".into(),
                    url: "https://example.com".into(),
                },
                Bookmark {
                    name: "Nested".into(),
                    url: "https://nested.example.com".into(),
                }
            ]
        );
    }

    #[test]
    fn collects_from_arrays() {
        let data = serde_json::json!([{
            "type": "url",
            "name": "Array Example",
            "url": "https://array.example.com"
        }]);

        let mut collected = Vec::new();
        collect_nodes(&data, &mut collected);
        assert_eq!(
            collected,
            vec![Bookmark {
                name: "Array Example".into(),
                url: "https://array.example.com".into(),
            }]
        );
    }

    #[test]
    fn parsing_invalid_json_returns_error() {
        let result = parse_bookmarks("not json");
        assert!(result.is_err());
    }
}
