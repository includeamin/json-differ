#[cfg(test)]
mod tests {
    use crate::delta::Delta;
    use crate::delta::Operation::{Add, Change, Delete};
    use crate::patcher::{patch, set_by_path, PatchOptions};
    use serde_json::json;
    use serde_json::Value::Null;

    #[test]
    fn test_patch_add_list() {
        let mut base_json = json!({});
        let path = "$.list";
        let value = json!([1, 2, 3]);

        set_by_path(&mut base_json, path, &value, Add, PatchOptions::new()).unwrap();

        assert_eq!(
            base_json,
            json!({
                "list": [1,2,3]
            })
        )
    }

    #[test]
    fn test_patch_add_element_to_list() {
        let mut base_json = json!({});
        let path = "$.list[0]";
        let value = json!(1);

        set_by_path(&mut base_json, path, &value, Add, PatchOptions::new()).unwrap();

        assert_eq!(
            base_json,
            json!({
                "list": [1]
            })
        );

        let path = "$.list[0]";
        let value = json!(2);
        set_by_path(&mut base_json, path, &value, Change, PatchOptions::new()).unwrap();

        assert_eq!(
            base_json,
            json!({
                "list": [2]
            })
        );

        let path = "$.list[1]";
        let value = json!(3);
        set_by_path(&mut base_json, path, &value, Add, PatchOptions::new()).unwrap();

        assert_eq!(
            base_json,
            json!({
                "list": [2,3]
            })
        );

        let path = "$.list[1]";
        let value = json!(Null);
        set_by_path(&mut base_json, path, &value, Delete, PatchOptions::new()).unwrap();

        assert_eq!(
            base_json,
            json!({
                "list": [2]
            })
        );

        let path = "$.list[0]";
        let value = json!(Null);
        set_by_path(&mut base_json, path, &value, Delete, PatchOptions::new()).unwrap();

        assert_eq!(
            base_json,
            json!({
                "list": []
            })
        );

        let path = "$.list";
        let value = json!(Null);
        set_by_path(&mut base_json, path, &value, Delete, PatchOptions::new()).unwrap();

        assert_eq!(base_json, json!({}));
    }

    #[test]
    fn test_crud_number() {
        let mut base_json = json!({});
        let path = "$.age";
        let value = json!(1);

        set_by_path(&mut base_json, path, &value, Add, PatchOptions::new()).unwrap();
        assert_eq!(base_json, json!({"age": 1}));

        let value = json!(2);
        set_by_path(&mut base_json, path, &value, Change, PatchOptions::new()).unwrap();
        assert_eq!(base_json, json!({"age": 2}));

        set_by_path(&mut base_json, path, &value, Delete, PatchOptions::new()).unwrap();
        assert_eq!(base_json, json!({}));
    }

    #[test]
    fn test_crud_string() {
        let mut base_json = json!({});
        let path = "$.first_name";
        let value = json!("first name");

        set_by_path(&mut base_json, path, &value, Add, PatchOptions::new()).unwrap();
        assert_eq!(base_json, json!({"first_name": "first name"}));

        let value = json!("changed name");
        set_by_path(&mut base_json, path, &value, Change, PatchOptions::new()).unwrap();
        assert_eq!(base_json, json!({"first_name": "changed name"}));

        set_by_path(&mut base_json, path, &value, Delete, PatchOptions::new()).unwrap();
        assert_eq!(base_json, json!({}));
    }

    #[test]
    fn test_crud_nested_json() {
        let mut base_json = json!({});
        let path = "$.gdpr.first_name";
        let value = json!("first name");

        set_by_path(&mut base_json, path, &value, Add, PatchOptions::new()).unwrap();

        assert_eq!(base_json, json!({"gdpr": {"first_name": "first name"}}));

        let value = json!("changed name");
        set_by_path(&mut base_json, path, &value, Change, PatchOptions::new()).unwrap();
        assert_eq!(base_json, json!({"gdpr": {"first_name": "changed name"}}));

        set_by_path(&mut base_json, path, &value, Delete, PatchOptions::new()).unwrap();
        assert_eq!(base_json, json!({"gdpr":{}}));
    }

    #[test]
    fn test_path() {
        let deltas = vec![
            Delta {
                operation: Add,
                path: "$.age".parse().unwrap(),
                old_value: json!(Null),
                new_value: json!(1),
                hash: String::default(),
            },
            Delta {
                operation: Add,
                path: "$.personal_information.first_name".parse().unwrap(),
                old_value: json!(Null),
                new_value: json!("first name"),
                hash: String::default(),
            },
            Delta {
                operation: Change,
                path: "$.age".parse().unwrap(),
                old_value: json!(Null),
                new_value: json!(20),
                hash: String::default(),
            },
            Delta {
                operation: Add,
                path: "$.tags".parse().unwrap(),
                old_value: json!(Null),
                new_value: json!(["test", "test1"]),
                hash: String::default(),
            },
            Delta {
                operation: Change,
                path: "$.tags[1]".parse().unwrap(),
                old_value: json!(Null),
                new_value: json!("test2"),
                hash: String::default(),
            },
        ];

        let patched = patch(json!({}), &deltas, PatchOptions::default().force(false));

        assert_eq!(
            patched,
            json!(
                {
                    "age":20 ,
                    "personal_information": {"first_name": "first name"},
                    "tags":["test","test2"]
                }
            )
        )
    }

    #[test]
    fn test_2d_array() {
        let deltas = vec![
            Delta {
                operation: Add,
                path: "$.test[0][0]".parse().unwrap(),
                old_value: Null,
                new_value: json!(1),
                hash: "4437996877722456100".parse().unwrap(),
            },
            Delta {
                operation: Add,
                path: "$.test[1][0]".parse().unwrap(),
                old_value: Null,
                new_value: json!(2),
                hash: "4437996877722456100".parse().unwrap(),
            },
        ];

        let patched = patch(json!({}), &deltas, PatchOptions::default().force(true));

        assert_eq!(
            patched,
            json!(
                {
                    "test": [[1], [2]]
                }
            )
        );

        let deltas = vec![Delta {
            operation: Add,
            path: "$.test[1][1]".parse().unwrap(),
            old_value: Null,
            new_value: json!(3),
            hash: "4437996877722456100".parse().unwrap(),
        }];

        let patched = patch(patched, &deltas, PatchOptions::default());

        assert_eq!(
            patched,
            json!(
                {
                    "test": [[1], [2, 3]]
                }
            )
        );

        let deltas = vec![Delta {
            operation: Change,
            path: "$.test[1][1]".parse().unwrap(),
            old_value: Null,
            new_value: json!(4),
            hash: "4437996877722456100".parse().unwrap(),
        }];

        let patched = patch(patched, &deltas, PatchOptions::default());

        assert_eq!(
            patched,
            json!(
                {
                    "test": [[1], [2, 4]]
                }
            )
        );

        let deltas = vec![Delta {
            operation: Delete,
            path: "$.test[1][1]".parse().unwrap(),
            old_value: Null,
            new_value: Null,
            hash: "4437996877722456100".parse().unwrap(),
        }];

        let patched = patch(patched, &deltas, PatchOptions::default().force(false));

        assert_eq!(
            patched,
            json!(
                {
                    "test": [[1], [2]]
                }
            )
        );

        let deltas = vec![Delta {
            operation: Delete,
            path: "$.test[1][0]".parse().unwrap(),
            old_value: Null,
            new_value: Null,
            hash: "4437996877722456100".parse().unwrap(),
        }];

        let patched = patch(
            patched,
            &deltas,
            PatchOptions::default().force(false).omit_empty(true),
        );

        assert_eq!(
            patched,
            json!(
                {
                    "test": [[1]]
                }
            )
        );
    }

    #[test]
    fn test_object_and_nested_array() {
        let deltas = vec![Delta {
            operation: Add,
            path: "$.list_of_objects[0][0].id".parse().unwrap(),
            old_value: Null,
            new_value: json!(1),
            hash: "4437996877722456100".parse().unwrap(),
        }];

        let patched = patch(
            json!({}),
            &deltas,
            PatchOptions::default().force(false).omit_empty(true),
        );

        assert_eq!(
            patched,
            json!(
                {
                    "list_of_objects": [[{"id": 1}]]
                }
            )
        );
    }

    #[test]
    fn test_set_by_path() {
        let mut json = json!({
            "a": [
                [
                    {
                        "b": [
                            [
                                "1",
                                "2"
                            ]
                        ]
                    }
                ]
            ]
        });

        let path = "$.a[0][0].b[0][1]";
        let value = json!("3");

        let result = set_by_path(&mut json, path, &value, Change, PatchOptions::new());

        assert!(result.is_ok());
        assert_eq!(
            json,
            json!({
                "a": [
                    [
                        {
                            "b": [
                                [
                                    "1",
                                    "3"
                                ]
                            ]
                        }
                    ]
                ]
            })
        );

        let path = "$.a[0][0].b[0][2]";
        let value = json!("4");

        let result = set_by_path(&mut json, path, &value, Add, PatchOptions::new());

        assert!(result.is_ok());
        assert_eq!(
            json,
            json!({
                "a": [
                    [
                        {
                            "b": [
                                [
                                    "1",
                                    "3",
                                    "4"
                                ]
                            ]
                        }
                    ]
                ]
            })
        );

        let path = "$.a[0][0].c";
        let value = json!("test");

        let result = set_by_path(&mut json, path, &value, Add, PatchOptions::new());

        assert!(result.is_ok());
        assert_eq!(
            json,
            json!({
                "a": [
                    [
                        {
                            "b": [
                                [
                                    "1",
                                    "3",
                                    "4"
                                ]
                            ],
                            "c": "test"
                        }
                    ]
                ]
            })
        );

        let path = "$.a[0][0].c";
        let value = json!("test2");

        let result = set_by_path(&mut json, path, &value, Change, PatchOptions::new());

        assert!(result.is_ok());
        assert_eq!(
            json,
            json!({
                "a": [
                    [
                        {
                            "b": [
                                [
                                    "1",
                                    "3",
                                    "4"
                                ]
                            ],
                            "c": "test2"
                        }
                    ]
                ]
            })
        );

        let path = "$.a[0][0].c";
        let value = json!(Null);

        let result = set_by_path(&mut json, path, &value, Delete, PatchOptions::new());

        assert!(result.is_ok());
        assert_eq!(
            json,
            json!({
                "a": [
                    [
                        {
                            "b": [
                                [
                                    "1",
                                    "3",
                                    "4"
                                ]
                            ]
                        }
                    ]
                ]
            })
        );

        let path = "$.a[0][0].b[0][1]";
        let value = json!(Null);

        let result = set_by_path(&mut json, path, &value, Delete, PatchOptions::new());

        assert!(result.is_ok());
        assert_eq!(
            json,
            json!({
                "a": [
                    [
                        {
                            "b": [
                                [
                                    "1",
                                    "4"
                                ]
                            ]
                        }
                    ]
                ]
            })
        );
    }

    #[test]
    fn test_patcher_omit_empty() {
        let base = json!({
            "a": [
                [
                    {
                        "b": [
                            [
                                "1"
                            ]
                        ]
                    }
                ]
            ]
        });

        let deltas = vec![Delta {
            operation: Delete,
            path: "$.a[0][0].b[0][1]".parse().unwrap(),
            old_value: json!("1"),
            new_value: Null,
            hash: "4437996877722456100".parse().unwrap(),
        }];

        let patcher = patch(
            base,
            &deltas,
            PatchOptions::default().force(false).omit_empty(true),
        );

        assert_eq!(patcher, json!({}));
    }
}
