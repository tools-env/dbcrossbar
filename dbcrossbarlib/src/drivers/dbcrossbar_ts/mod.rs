//! Support for `dbcrossbar-ts` locators.

use percent_encoding::percent_decode_str;
use std::{fmt, path::PathBuf, str::FromStr};

use crate::common::*;

mod ast;

use self::ast::SourceFile;

/// A file containing type definitions written in a subset of TypeScript.
#[derive(Clone, Debug)]
pub struct DbcrossbarTsLocator {
    path: PathOrStdio,
    fragment: String,
}

impl fmt::Display for DbcrossbarTsLocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encode = |s: &str| s.replace('%', "%25").replace('#', "%23");
        write!(
            f,
            "{}{}#{}",
            Self::scheme(),
            encode(&self.path.to_string()),
            encode(&self.fragment)
        )
    }
}

impl FromStr for DbcrossbarTsLocator {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if !s.starts_with(Self::scheme()) {
            return Err(format_err!(
                "expected {:?} to start with {}",
                s,
                Self::scheme()
            ));
        }
        let parts = s[Self::scheme().len()..].splitn(2, '#').collect::<Vec<_>>();
        if parts.len() != 2 {
            return Err(format_err!("expected '#' in {:?}", s));
        }
        let decode = |idx| {
            percent_decode_str(parts[idx])
                .decode_utf8()
                .with_context(|_| format!("error decoding {:?}", s))
        };
        let path = decode(0)?;
        let fragment = decode(1)?.into_owned();
        let path = if path == "-" {
            PathOrStdio::Stdio
        } else {
            PathOrStdio::Path(PathBuf::from(path.into_owned()))
        };
        Ok(DbcrossbarTsLocator { path, fragment })
    }
}

impl Locator for DbcrossbarTsLocator {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self, ctx: Context) -> BoxFuture<Option<Table>> {
        schema_helper(ctx, self.to_owned()).boxed()
    }
}

impl LocatorStatic for DbcrossbarTsLocator {
    fn scheme() -> &'static str {
        "dbcrossbar-ts:"
    }

    fn features() -> Features {
        Features {
            locator: LocatorFeatures::Schema.into(),
            write_schema_if_exists: EnumSet::empty(),
            source_args: EnumSet::empty(),
            dest_args: EnumSet::empty(),
            dest_if_exists: EnumSet::empty(),
            _placeholder: (),
        }
    }

    /// This locator type is currently unstable.
    fn is_unstable() -> bool {
        true
    }
}

/// Implementation of `schema`, but as a real `async` function.
async fn schema_helper(
    _ctx: Context,
    source: DbcrossbarTsLocator,
) -> Result<Option<Table>> {
    // Read our input.
    let input = source.path.open_async().await?;
    let data = async_read_to_end(input)
        .await
        .with_context(|_| format!("error reading {}", source.path))?;
    let data = String::from_utf8(data)
        .with_context(|_| format!("found non-UTF-8 data in {}", source.path))?;

    // Parse it as a TypeScript file.
    let source_file = SourceFile::parse(source.path.to_string(), data)?;
    let table = source_file.definition_to_table(&source.fragment)?;
    Ok(Some(table))
}
