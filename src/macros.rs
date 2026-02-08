//! This module contains the macros used in the project

/// automatically generate the input path
macro_rules! config_path {
    ($config:ident, $setting_name: ident, $struct_name: ident, $key_name: ident, $string: expr) => {
        match &$config.config_data.$setting_name {
            Some($struct_name {
                $key_name: Some(path),
                ..
            }) => PathBuf::from(path),
            _ => {
                println!(concat!(
                    "Please enter the path to the folder where to save ",
                    $string,
                    ":"
                ));
                let (file_path, path_string) = input_path()?;
                $config.update(|config_data| {
                    if let Some(local_config) = config_data.$setting_name.as_mut() {
                        local_config.$key_name = Some(path_string);
                    } else {
                        config_data.$setting_name = Some($struct_name {
                            $key_name: Some(path_string),
                            ..Default::default()
                        });
                    }
                })?;
                file_path
            }
        }
    };
}

pub(crate) use config_path;
/// automatically generate the input path
macro_rules! get_config_path {
    ($config:ident, $setting_name: ident, $struct_name: ident, $key_name: ident, $string: expr) => {
        match &$config.config_data.$setting_name {
            Some($struct_name {
                $key_name: Some(path),
                ..
            }) => Ok(PathBuf::from(path)),
            _ => Err(GeneralError::new(concat!(
                "The path ",
                $string,
                " is not set"
            ))),
        }
    };
}
pub(crate) use get_config_path;

/// automatically generate the input path
macro_rules! config_sub_path {
    ($config:ident, $setting_name: ident, $struct_name: ident, $key_name: ident, $sub_struct_name: ident, $sub_key_name: ident, $string: expr) => {{
        match &$config.config_data.$setting_name {
            Some($struct_name {
                $key_name:
                    Some($sub_struct_name {
                        $sub_key_name: Some(path),
                        ..
                    }),
                ..
            }) => PathBuf::from(path),
            _ => {
                println!(concat!(
                    "Please enter the path to the folder where to save ",
                    $string,
                    ":"
                ));
                let (file_path, path_string) = input_path()?;
                $config.update(|config_data| match config_data.$setting_name {
                    Some($struct_name {
                        $key_name: Some(ref mut local_config),
                        ..
                    }) => {
                        local_config.$sub_key_name = Some(path_string);
                    }
                    _ => {
                        config_data.$setting_name = Some($struct_name {
                            $key_name: Some($sub_struct_name {
                                $sub_key_name: Some(path_string),
                                ..Default::default()
                            }),
                            ..Default::default()
                        });
                    }
                })?;
                file_path
            }
        }
    }};
}
pub(crate) use config_sub_path;
