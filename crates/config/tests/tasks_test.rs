use figment::{
    providers::{Format, Yaml},
    Figment,
};
use moon_config::{TaskCommandArgs, TaskConfig};
use moon_utils::string_vec;
use std::path::PathBuf;

const CONFIG_FILENAME: &str = "tasks.yml";

// Not a config file, but we want to test in isolation
fn load_jailed_config() -> Result<TaskConfig, figment::Error> {
    Figment::new()
        .merge(Yaml::file(&PathBuf::from(CONFIG_FILENAME)))
        .extract()
}

mod command {
    use super::*;

    #[test]
    #[should_panic(
        expected = "expected a string or a sequence of strings for key \"default.command\""
    )]
    fn invalid_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(super::CONFIG_FILENAME, "command: 123")?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    fn returns_empty_string() {
        let config = TaskConfig::default();

        assert_eq!(config.get_command(), "");
    }

    #[test]
    fn returns_only_from_string() {
        let config = TaskConfig {
            command: Some(TaskCommandArgs::String("foo".to_owned())),
            ..TaskConfig::default()
        };

        assert_eq!(config.get_command(), "foo");
    }

    #[test]
    fn returns_first_from_string() {
        let config = TaskConfig {
            command: Some(TaskCommandArgs::String("foo --bar baz".to_owned())),
            ..TaskConfig::default()
        };

        assert_eq!(config.get_command(), "foo");
    }

    #[test]
    fn returns_only_from_list() {
        let config = TaskConfig {
            command: Some(TaskCommandArgs::Sequence(string_vec!["foo"])),
            ..TaskConfig::default()
        };

        assert_eq!(config.get_command(), "foo");
    }

    #[test]
    fn returns_first_from_list() {
        let config = TaskConfig {
            command: Some(TaskCommandArgs::Sequence(string_vec![
                "foo", "--bar", "baz"
            ])),
            ..TaskConfig::default()
        };

        assert_eq!(config.get_command(), "foo");
    }
}

mod command_args {
    use super::*;

    #[test]
    fn cmd_string_no_args() {
        let config = TaskConfig {
            command: Some(TaskCommandArgs::String("foo --bar baz".to_owned())),
            ..TaskConfig::default()
        };

        assert_eq!(
            config.get_command_and_args().unwrap(),
            (Some("foo".to_owned()), string_vec!["--bar", "baz"])
        );
    }

    #[test]
    fn cmd_string_with_args_string() {
        let config = TaskConfig {
            command: Some(TaskCommandArgs::String("foo --bar baz".to_owned())),
            args: Some(TaskCommandArgs::String("--qux -x".to_owned())),
            ..TaskConfig::default()
        };

        assert_eq!(
            config.get_command_and_args().unwrap(),
            (
                Some("foo".to_owned()),
                string_vec!["--bar", "baz", "--qux", "-x"]
            )
        );
    }

    #[test]
    fn cmd_string_with_args_list() {
        let config = TaskConfig {
            command: Some(TaskCommandArgs::String("foo --bar baz".to_owned())),
            args: Some(TaskCommandArgs::Sequence(string_vec!["--qux", "-x"])),
            ..TaskConfig::default()
        };

        assert_eq!(
            config.get_command_and_args().unwrap(),
            (
                Some("foo".to_owned()),
                string_vec!["--bar", "baz", "--qux", "-x"]
            )
        );
    }

    #[test]
    fn cmd_list_no_args() {
        let config = TaskConfig {
            command: Some(TaskCommandArgs::Sequence(string_vec![
                "foo", "--bar", "baz"
            ])),
            ..TaskConfig::default()
        };

        assert_eq!(
            config.get_command_and_args().unwrap(),
            (Some("foo".to_owned()), string_vec!["--bar", "baz"])
        );
    }

    #[test]
    fn cmd_list_with_args_string() {
        let config = TaskConfig {
            command: Some(TaskCommandArgs::Sequence(string_vec![
                "foo", "--bar", "baz"
            ])),
            args: Some(TaskCommandArgs::String("--qux -x".to_owned())),
            ..TaskConfig::default()
        };

        assert_eq!(
            config.get_command_and_args().unwrap(),
            (
                Some("foo".to_owned()),
                string_vec!["--bar", "baz", "--qux", "-x"]
            )
        );
    }

    #[test]
    fn cmd_list_with_args_list() {
        let config = TaskConfig {
            command: Some(TaskCommandArgs::Sequence(string_vec![
                "foo", "--bar", "baz"
            ])),
            args: Some(TaskCommandArgs::Sequence(string_vec!["--qux", "-x"])),
            ..TaskConfig::default()
        };

        assert_eq!(
            config.get_command_and_args().unwrap(),
            (
                Some("foo".to_owned()),
                string_vec!["--bar", "baz", "--qux", "-x"]
            )
        );
    }

    #[test]
    fn args_string_no_cmd() {
        let config = TaskConfig {
            command: None,
            args: Some(TaskCommandArgs::String("--foo bar".to_owned())),
            ..TaskConfig::default()
        };

        assert_eq!(
            config.get_command_and_args().unwrap(),
            (None, string_vec!["--foo", "bar"])
        );
    }

    #[test]
    fn args_list_no_cmd() {
        let config = TaskConfig {
            command: None,
            args: Some(TaskCommandArgs::Sequence(string_vec!["--foo", "bar"])),
            ..TaskConfig::default()
        };

        assert_eq!(
            config.get_command_and_args().unwrap(),
            (None, string_vec!["--foo", "bar"])
        );
    }
}

mod args {
    use super::*;

    #[test]
    #[should_panic(
        expected = "expected a string or a sequence of strings for key \"default.args\""
    )]
    fn invalid_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_FILENAME,
                r#"
command: foo
args: 123
"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "expected a string or a sequence of strings for key \"default.args\""
    )]
    fn invalid_value_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_FILENAME,
                r#"
command: foo
args:
    - 123
"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    fn supports_vec_strings() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_FILENAME,
                r#"
command: foo
args:
    - arg
    - -o
    - '@token(0)'
    - --opt
    - value
    - 'quoted arg'
"#,
            )?;

            let config = super::load_jailed_config()?;

            assert_eq!(
                config,
                TaskConfig {
                    command: Some(TaskCommandArgs::String("foo".to_owned())),
                    args: Some(TaskCommandArgs::Sequence(string_vec![
                        "arg",
                        "-o",
                        "@token(0)",
                        "--opt",
                        "value",
                        "quoted arg"
                    ])),
                    ..TaskConfig::default()
                }
            );

            Ok(())
        });
    }

    #[test]
    fn supports_string() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_FILENAME,
                r#"
command: foo
args: 'arg -o @token(0) --opt value "quoted arg"'
"#,
            )?;

            let config = super::load_jailed_config()?;

            assert_eq!(
                config,
                TaskConfig {
                    command: Some(TaskCommandArgs::String("foo".to_owned())),
                    args: Some(TaskCommandArgs::String(
                        "arg -o @token(0) --opt value \"quoted arg\"".to_owned()
                    )),
                    ..TaskConfig::default()
                }
            );

            Ok(())
        });
    }
}

mod deps {
    #[test]
    #[should_panic(
        expected = "invalid type: found string \"abc\", expected a sequence for key \"default.deps\""
    )]
    fn invalid_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_FILENAME,
                r#"
command: foo
deps: abc
"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "invalid type: found unsigned int `123`, expected a string for key \"default.deps.0\""
    )]
    fn invalid_value_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_FILENAME,
                r#"
command: foo
deps:
    - 123
"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    //         #[test]
    //         #[should_panic(
    //             expected = "Invalid field <id>deps.0</id>: Expected a string type, received unsigned int `123`."
    //         )]
    //         fn invalid_format() {
    //             figment::Jail::expect_with(|jail| {
    //                 jail.create_file(
    //                     super::CONFIG_FILENAME,
    //                     r#"
    // command: foo
    // deps:
    //     - foo
    // "#,
    //                 )?;

    //                 super::load_jailed_config()?;

    //                 Ok(())
    //             });
    //         }
}

mod env {
    #[test]
    #[should_panic(
        expected = "invalid type: found string \"abc\", expected a map for key \"default.env\""
    )]
    fn invalid_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_FILENAME,
                r#"
command: foo
env: abc
"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "invalid type: found unsigned int `123`, expected a string for key \"default.env.KEY\""
    )]
    fn invalid_value_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_FILENAME,
                r#"
command: foo
env:
  KEY: 123
"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }
}

mod inputs {
    #[test]
    #[should_panic(
        expected = "invalid type: found string \"abc\", expected a sequence for key \"default.inputs\""
    )]
    fn invalid_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_FILENAME,
                r#"
command: foo
inputs: abc
"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "invalid type: found unsigned int `123`, expected a string for key \"default.inputs.0\""
    )]
    fn invalid_value_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_FILENAME,
                r#"
command: foo
inputs:
    - 123
"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    fn supports_env_vars() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_FILENAME,
                r#"
command: foo
inputs:
  - $FOO
  - file.js
  - /file.js
"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }
}

mod outputs {
    #[test]
    #[should_panic(
        expected = "invalid type: found string \"abc\", expected a sequence for key \"default.outputs\""
    )]
    fn invalid_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_FILENAME,
                r#"
command: foo
outputs: abc
"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "invalid type: found unsigned int `123`, expected a string for key \"default.outputs.0\""
    )]
    fn invalid_value_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_FILENAME,
                r#"
command: foo
outputs:
    - 123
"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }
}

mod type_of {
    #[test]
    #[should_panic(
        expected = "unknown variant: found `whatisthis`, expected `one of `node`, `system`, `unknown`` for key \"default.type\""
    )]
    fn invalid_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_FILENAME,
                r#"
command: foo
type: whatisthis
"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }
}

mod options {
    #[test]
    #[should_panic(
        expected = "invalid type: found unsigned int `123`, expected struct TaskOptionsConfig for key \"default.options\""
    )]
    fn invalid_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_FILENAME,
                r#"
command: foo
options: 123
"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "unknown variant: found `bubble`, expected `one of `append`, `prepend`, `replace`` for key \"default.options.mergeArgs\""
    )]
    fn invalid_merge_strategy_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_FILENAME,
                r#"
command: foo
options:
    mergeArgs: bubble
"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "invalid type: found string \"abc\", expected u8 for key \"default.options.retryCount\""
    )]
    fn invalid_retry_count_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_FILENAME,
                r#"
command: foo
options:
    retryCount: abc
"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    #[test]
    #[should_panic(
        expected = "expected a boolean or a relative file system path for key \"default.options.envFile\""
    )]
    fn invalid_env_file_type() {
        figment::Jail::expect_with(|jail| {
            jail.create_file(
                super::CONFIG_FILENAME,
                r#"
command: foo
options:
    envFile: 123
"#,
            )?;

            super::load_jailed_config()?;

            Ok(())
        });
    }

    // Enums validation is currently not supported:
    // https://github.com/Keats/validator/issues/77
    //         #[test]
    //         #[should_panic(expected = "todo")]
    //         fn invalid_env_file_path() {
    //             figment::Jail::expect_with(|jail| {
    //                 jail.create_file(
    //                     super::CONFIG_FILENAME,
    //                     r#"
    // command: foo
    // options:
    //     envFile: '../.env'
    // "#,
    //                 )?;

    //                 super::load_jailed_config()?;

    //                 Ok(())
    //             });
    //         }
}