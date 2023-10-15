#[cfg(test)]
mod tests {
    use crate::delta::Operation;
    use crate::differ::Differ;
    use serde_json::{json, Value};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn diff_from_serde_values_success() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("src/testdata/small_json_1.json");
        let data = fs::read_to_string(d).expect("Unable to read file");
        let json_a = serde_json::from_str(&data).expect("Unable to parse");

        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("src/testdata/small_json_2.json");
        let data = fs::read_to_string(d).expect("Unable to read file");
        let json_b = serde_json::from_str(&data).expect("Unable to parse");

        let mut differ = Differ::new_from_json_values(json_a, json_b);
        let deltas = differ.diff().get_deltas();

        assert_eq!(deltas.len(), 5, "Expected 4 deltas, got {}", deltas.len());
    }

    #[test]
    fn diff_from_json_value() {
        let a = serde_json::from_str(
            r#"
         {
           "username": "admin",
           "password": "admin",
           "email": "foo@bar.com",
           "name": "Foo Bar",
           "address": "Dhaka",
           "roles": [
             "admin"
           ],
           "status": "active",
           "created_at": "2019-12-12T12:12:12.000Z",
           "nested": {
             "foo": "bar",
             "nested2": {
               "foo": "bar",
                "nested3": {
                  "foo": "bar"
                }
             }
           }
         }
        "#,
        )
        .unwrap();

        let b = serde_json::from_str(
            r#"
         {
           "username": "admin",
           "password": "admin",
           "email": "foo@bar.com",
           "name": "Foo Bar",
           "address": "Dhaka",
           "phone": "123456789",
           "roles": [
             "admin"
           ],
           "status": "active",
           "created_at": "2019-12-12T12:12:13.000Z",
           "nested": {
             "foo": "bar",
                "nested2": {
                "foo": "bar",
                    "nested3": {
                    "foo": "bar2"
                    }
                }
           }
         }
        "#,
        )
        .unwrap();

        let mut differ = Differ::new_from_json_values(a, b);
        let differ = differ.diff();

        assert_eq!(
            differ.get_deltas().len(),
            3,
            "Expected 1 deltas, got {}",
            differ.get_deltas().len()
        );
        assert!(differ.has_changes());
        assert!(differ.has_path_changed("$.phone", Operation::Add));
        assert!(differ.has_path_changed("$.created_at", Operation::Change));

        let delta = differ.get_delta_by_path("$.phone").unwrap();
        assert_eq!(delta.operation, Operation::Add);
        assert_eq!(delta.path, "$.phone");
        assert_eq!(delta.old_value, Value::Null);
        assert_eq!(delta.new_value, Value::String("123456789".to_string()));

        let delta = differ.get_delta_by_path("$.created_at").unwrap();
        assert_eq!(delta.operation, Operation::Change);
        assert_eq!(delta.path, "$.created_at");
        assert_eq!(
            delta.old_value,
            Value::String("2019-12-12T12:12:12.000Z".to_string())
        );
        assert_eq!(
            delta.new_value,
            Value::String("2019-12-12T12:12:13.000Z".to_string())
        );

        let delta = differ
            .get_delta_by_path("$.nested.nested2.nested3.foo")
            .unwrap();
        assert_eq!(delta.operation, Operation::Change);
        assert_eq!(delta.path, "$.nested.nested2.nested3.foo");
        assert_eq!(delta.old_value, Value::String("bar".to_string()));
        assert_eq!(delta.new_value, Value::String("bar2".to_string()));
    }

    #[test]
    fn diff_arrays_remove_change() {
        let a = serde_json::from_str(
            r#"
         {
          "test":[1,2,3]
         }
        "#,
        )
        .unwrap();

        let b = serde_json::from_str(
            r#"
         {
          "test":[1,2]
         }
        "#,
        )
        .unwrap();

        let mut differ = Differ::new_from_json_values(a, b);
        let differ = differ.diff();

        let detlas = differ.get_deltas();

        assert_eq!(detlas.len(), 1, "Expected 1 deltas, got {}", detlas.len());

        assert_eq!(detlas[0].path, "$.test[2]");
        assert_eq!(detlas[0].operation, Operation::Delete);
        assert_eq!(detlas[0].old_value, Value::Number(3.into()));
    }

    #[test]
    fn diff_nested_arrays_remove_change() {
        let a = serde_json::from_str(
            r#"
         {
          "test":[[1,2,3]]
         }
        "#,
        )
        .unwrap();

        let b = serde_json::from_str(
            r#"
         {
          "test":[[1,2]]
         }
        "#,
        )
        .unwrap();

        let mut differ = Differ::new_from_json_values(a, b);
        let differ = differ.diff();

        let deltas = differ.get_deltas();

        assert_eq!(deltas.len(), 1, "Expected 1 deltas, got {}", deltas.len());

        assert_eq!(deltas[0].path, "$.test[0][2]");
        assert_eq!(deltas[0].operation, Operation::Delete);
        assert_eq!(deltas[0].old_value, Value::Number(3.into()));
    }

    #[test]
    fn diff_arrays_remove() {
        let a = serde_json::from_str(
            r#"
         {
         "test1": "test",
          "test":[1,2,3]
         }
        "#,
        )
        .unwrap();

        let b = serde_json::from_str(
            r#"
         {
           "test1": "test"
         }
        "#,
        )
        .unwrap();

        let mut differ = Differ::new_from_json_values(a, b);
        let differ = differ.diff();
        let deltas = differ.get_deltas();

        assert_eq!(deltas.len(), 3, "Expected 1 deltas, got {}", deltas.len());

        let delta = deltas.first().unwrap();
        assert_eq!(delta.path, "$.test[0]");
        assert_eq!(delta.operation, Operation::Delete);
        assert_eq!(delta.old_value, Value::Number(1.into()));
        assert_eq!(delta.new_value, Value::Null);

        let delta = deltas.get(1).unwrap();
        assert_eq!(delta.path, "$.test[1]");
        assert_eq!(delta.operation, Operation::Delete);
        assert_eq!(delta.old_value, Value::Number(2.into()));
        assert_eq!(delta.new_value, Value::Null);

        let delta = deltas.get(2).unwrap();
        assert_eq!(delta.path, "$.test[2]");
        assert_eq!(delta.operation, Operation::Delete);
        assert_eq!(delta.old_value, Value::Number(3.into()));
        assert_eq!(delta.new_value, Value::Null);
    }

    #[test]
    fn diff_arrays_add() {
        let a = serde_json::from_str(
            r#"
         {
         "test1": "test"
         }
        "#,
        )
        .unwrap();

        let b = serde_json::from_str(
            r#"
         {
           "test1": "test",
           "test":[1,2,3]
         }
        "#,
        )
        .unwrap();

        let mut differ = Differ::new_from_json_values(a, b);
        let diff = differ.diff();

        assert_eq!(
            diff.get_deltas().len(),
            3,
            "Expected 1 deltas, got {}",
            diff.get_deltas().len()
        );

        let delta = diff.get_deltas().first().unwrap();
        assert_eq!(delta.path, "$.test[0]");
        assert_eq!(delta.operation, Operation::Add);
        assert_eq!(delta.old_value, Value::Null);
        assert_eq!(delta.new_value, Value::Number(1.into()));

        let delta = diff.get_deltas().get(1).unwrap();
        assert_eq!(delta.path, "$.test[1]");
        assert_eq!(delta.operation, Operation::Add);
        assert_eq!(delta.old_value, Value::Null);
        assert_eq!(delta.new_value, Value::Number(2.into()));

        let delta = diff.get_deltas().get(2).unwrap();
        assert_eq!(delta.path, "$.test[2]");
        assert_eq!(delta.operation, Operation::Add);
        assert_eq!(delta.old_value, Value::Null);
        assert_eq!(delta.new_value, Value::Number(3.into()));
    }

    #[test]
    fn test_super_nested_object() {
        let a = json!({
            "test":[
                [
                    {
                        "test": [
                         [
                             "1",
                                "2"

                         ]
                    ]
                    }
                ]
            ]
        });

        let b = json!({
            "test":[
                [
                    {
                        "test": [
                         [
                             "1",
                                "3"

                         ]
                    ]
                    }
                ]
            ]
        });

        let mut differ = Differ::new_from_json_values(a, b);
        let diff = differ.diff();

        assert_eq!(
            diff.get_deltas().len(),
            1,
            "Expected 1 deltas, got {}",
            diff.get_deltas().len()
        );

        let delta = diff.get_deltas().first().unwrap();
        assert_eq!(delta.path, "$.test[0][0].test[0][1]");
        assert_eq!(delta.operation, Operation::Change);
        assert_eq!(delta.old_value, Value::String("2".to_string()));
        assert_eq!(delta.new_value, Value::String("3".to_string()));
    }
}
