/// automaticallly generate the input path
#[macro_export]
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
                let file_path = input_path();
                let cloned_path = file_path.1.clone();
                $config.update(|config_data| {
                    if let Some(local_config) = config_data.$setting_name.as_mut() {
                        local_config.$key_name = Some(cloned_path);
                    } else {
                        config_data.$setting_name = Some($struct_name {
                            $key_name: Some(cloned_path),
                            ..Default::default()
                        });
                    }
                    config_data
                });
                file_path.0
            }
        }
    };
}

/// automatically generate the input path
#[macro_export]
macro_rules! config_sub_path {
    ($config:ident, $setting_name: ident, $struct_name: ident, $key_name: ident, $sub_struct_name: ident, $sub_key_name: ident, $string: expr) => {
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
                let file_path = input_path();
                let cloned_path = file_path.1.clone();
                $config.update(|config_data| {
                    match config_data.$setting_name {
                        Some($struct_name {
                            $key_name: Some(ref mut local_config),
                            ..
                        }) => {
                            local_config.$sub_key_name = Some(cloned_path);
                        }
                        _ => {
                            config_data.$setting_name = Some($struct_name {
                                $key_name: Some($sub_struct_name {
                                    $sub_key_name: Some(cloned_path),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            });
                        }
                    }
                    config_data
                });
                file_path.0
            }
        }
    };
}
