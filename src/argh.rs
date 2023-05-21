use std::marker::PhantomData;

#[macro_export]
macro_rules! read_args_with_version {
    ($ty:ty) => {{
        struct VersionProvider;
        impl $crate::AppVersionProvider for VersionProvider {
            fn version() -> &'static str {
                env!("CARGO_PKG_VERSION")
            }
        }

        ::argh::from_env::<$crate::ArgsOrVersion<$ty, VersionProvider>>().app
    }};
}

pub trait AppVersionProvider {
    fn version() -> &'static str;
}

pub struct ArgsOrVersion<T: argh::FromArgs, V> {
    pub app: T,
    _version_marker: PhantomData<V>,
}

impl<T: argh::FromArgs, V: AppVersionProvider> argh::TopLevelCommand for ArgsOrVersion<T, V> {}

impl<T: argh::FromArgs, V: AppVersionProvider> argh::FromArgs for ArgsOrVersion<T, V> {
    fn from_args(command_name: &[&str], args: &[&str]) -> Result<Self, argh::EarlyExit> {
        /// Also use argh for catching `--version`-only invocations
        #[derive(argh::FromArgs)]
        struct Version {
            /// print version information and exit
            #[argh(switch, short = 'v')]
            pub version: bool,
        }

        match Version::from_args(command_name, args) {
            Ok(v) if v.version => Err(argh::EarlyExit {
                output: format!("{} {}", command_name.first().unwrap_or(&""), V::version()),
                status: Ok(()),
            }),
            Err(exit) if exit.status.is_ok() => {
                let help = match T::from_args(command_name, &["--help"]) {
                    Ok(_) => unreachable!(),
                    Err(exit) => exit.output,
                };
                Err(argh::EarlyExit {
                    output: format!("{help}  -v, --version     print version information and exit"),
                    status: Ok(()),
                })
            }
            _ => T::from_args(command_name, args).map(|app| Self {
                app,
                _version_marker: PhantomData,
            }),
        }
    }
}
