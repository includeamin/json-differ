#[cfg(test)]
mod lib_tests {
    use serde_json::json;
    use crate::differ::Differ;
    use crate::patcher::patch;

    #[test]
    fn test_lib() {
        let left = json!({});
        let right = json!({"age": 20, "tags":["test"]});
        let mut differ = Differ::new_from_json_values(left, right);
        let diff = differ.diff();

        assert_eq!(diff.get_deltas().len(), 2);

        let new_base = json!({});
        let deltas = diff.get_deltas();
        let patched = patch(new_base, deltas);

        assert_eq!(patched, json!({"age": 20, "tags":["test"]}))
    }
}