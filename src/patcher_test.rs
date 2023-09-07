#[cfg(test)]
mod tests {
    use serde_json::json;
    use serde_json::Value::Null;
    use crate::delta::{Delta, Operation};
    use crate::patcher::{patch, set_property_by_path};

    #[test]
    fn test_patch_add_list() {
        let mut base_json = json!({});
        let path = "$.list";
        let value = json!([1,2,3]);

        set_property_by_path(
            &mut base_json, path, &value, Operation::Add, false,
        ).unwrap();

        assert_eq!(base_json, json!({
            "list": [1,2,3]
        }))
    }

    #[test]
    fn test_patch_add_element_to_list() {
        let mut base_json = json!({});
        let path = "$.list[0]";
        let value = json!(1);

        set_property_by_path(
            &mut base_json, path, &value, Operation::Add, false,
        ).unwrap();

        assert_eq!(base_json, json!({
            "list": [1]
        }));

        let path = "$.list[0]";
        let value = json!(2);
        set_property_by_path(
            &mut base_json, path, &value, Operation::Change, false,
        ).unwrap();

        assert_eq!(base_json, json!({
            "list": [2]
        }));


        let path = "$.list[1]";
        let value = json!(3);
        set_property_by_path(
            &mut base_json, path, &value, Operation::Add, false,
        ).unwrap();

        assert_eq!(base_json, json!({
            "list": [2,3]
        }));

        let path = "$.list[1]";
        let value = json!(Null);
        set_property_by_path(
            &mut base_json, path, &value, Operation::Delete, false,
        ).unwrap();

        assert_eq!(base_json, json!({
            "list": [2]
        }));

        let path = "$.list[0]";
        let value = json!(Null);
        set_property_by_path(
            &mut base_json, path, &value, Operation::Delete, false,
        ).unwrap();

        assert_eq!(base_json, json!({
            "list": []
        }));

        let path = "$.list";
        let value = json!(Null);
        set_property_by_path(
            &mut base_json, path, &value, Operation::Delete, false,
        ).unwrap();

        assert_eq!(base_json, json!({}));
    }

    #[test]
    fn test_crud_number() {
        let mut base_json = json!({});
        let path = "$.age";
        let value = json!(1);

        set_property_by_path(
            &mut base_json, path, &value, Operation::Add, false,
        ).unwrap();
        assert_eq!(base_json, json!({"age": 1}));

        let value = json!(2);
        set_property_by_path(
            &mut base_json, path, &value, Operation::Change, false,
        ).unwrap();
        assert_eq!(base_json, json!({"age": 2}));


        set_property_by_path(
            &mut base_json, path, &value, Operation::Delete, false,
        ).unwrap();
        assert_eq!(base_json, json!({}));
    }

    #[test]
    fn test_crud_string() {
        let mut base_json = json!({});
        let path = "$.first_name";
        let value = json!("first name");

        set_property_by_path(
            &mut base_json, path, &value, Operation::Add, false,
        ).unwrap();
        assert_eq!(base_json, json!({"first_name": "first name"}));

        let value = json!("changed name");
        set_property_by_path(
            &mut base_json, path, &value, Operation::Change, false,
        ).unwrap();
        assert_eq!(base_json, json!({"first_name": "changed name"}));


        set_property_by_path(
            &mut base_json, path, &value, Operation::Delete, false,
        ).unwrap();
        assert_eq!(base_json, json!({}));
    }

    #[test]
    fn test_crud_nested_json() {
        let mut base_json = json!({});
        let path = "$.gdpr.first_name";
        let value = json!("first name");

        set_property_by_path(
            &mut base_json, path, &value, Operation::Add, false,
        ).unwrap();
        assert_eq!(base_json, json!({"gdpr": {"first_name": "first name"}}));

        let value = json!("changed name");
        set_property_by_path(
            &mut base_json, path, &value, Operation::Change, false,
        ).unwrap();
        assert_eq!(base_json, json!({"gdpr": {"first_name": "changed name"}}));

        set_property_by_path(
            &mut base_json, path, &value, Operation::Delete, false,
        ).unwrap();
        assert_eq!(base_json, json!({"gdpr":{}}));
    }

    #[test]
    fn test_path() {
        let deltas = vec![
            Delta {
                operation: Operation::Add,
                path: "$.age".parse().unwrap(),
                old_value: json!(Null),
                new_value: json!(1),
            },
            Delta {
                operation: Operation::Add,
                path: "$.personal_information.first_name".parse().unwrap(),
                old_value: json!(Null),
                new_value: json!("first name"),
            },
            Delta {
                operation: Operation::Change,
                path: "$.age".parse().unwrap(),
                old_value: json!(Null),
                new_value: json!(20),
            },
            Delta {
                operation: Operation::Add,
                path: "$.tags".parse().unwrap(),
                old_value: json!(Null),
                new_value: json!(["test","test1"]),
            },
            Delta {
                operation: Operation::Change,
                path: "$.tags[1]".parse().unwrap(),
                old_value: json!(Null),
                new_value: json!("test2"),
            },
        ];

        let patched = patch(json!({}), &deltas);

        assert_eq!(patched, json!(
            {
                "age":20 ,
                "personal_information": {"first_name": "first name"},
                "tags":["test","test2"]
            }
        ))
    }
}
