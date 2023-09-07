#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use serde_json::Value;
    use crate::delta::Operation;
    use crate::differ::Differ;

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
        assert_eq!(deltas[0].operation, Operation::Change);
        assert_eq!(deltas[1].operation, Operation::Change);
        assert_eq!(deltas[2].operation, Operation::Change);
        assert_eq!(deltas[3].operation, Operation::Delete);
        assert_eq!(deltas[4].operation, Operation::Add);
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
}
