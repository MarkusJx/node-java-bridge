use crate::node::napi_error::NapiError;
use napi::{JsString, JsUnknown};
use std::collections::VecDeque;
use std::path::Path;

#[cfg(windows)]
mod separator {
    pub const CLASSPATH_SEPARATOR: &str = ";";
    pub const OTHER_SEPARATOR: &str = ":";
}
#[cfg(unix)]
mod separator {
    pub const CLASSPATH_SEPARATOR: &str = ":";
    pub const OTHER_SEPARATOR: &str = ";";
}

/// Parse an JsUnknown that is either a JsString or a JsArray into a String
pub(crate) fn parse_array_or_string(value: JsUnknown) -> napi::Result<Vec<String>> {
    let mut res = Vec::<String>::new();
    if value.is_array()? {
        let obj = value.coerce_to_object()?;
        for i in 0..obj.get_array_length()? {
            let path: JsString = obj.get_element(i)?;
            res.push(path.into_utf16()?.as_str()?);
        }
    } else {
        let path = value.coerce_to_string()?;
        res.push(path.into_utf16()?.as_str()?);
    }

    Ok(res)
}

pub(crate) fn list_files(dirs: Vec<String>, recursive: bool) -> napi::Result<Vec<String>> {
    let mut files = Vec::<String>::new();
    let mut q = VecDeque::from(dirs);

    while let Some(dir) = q.pop_back() {
        let path = Path::new(&dir);
        if !path.exists() {
            return Err(NapiError::from(format!("Path '{}' does not exist", dir)).into_napi());
        } else if path.is_dir() {
            let inner = std::fs::read_dir(path)
                .map_err(|e| NapiError::to_napi_error(e.into()))?
                .filter_map(|e| e.ok())
                .filter_map(|e| e.path().to_str().map(|s| s.to_string()))
                .collect::<Vec<String>>();

            if recursive {
                q.extend(inner);
            } else {
                files.extend(inner);
            }
        } else {
            files.push(dir);
        }
    }

    Ok(files)
}

pub fn parse_classpath_args(cp: &Vec<String>, args: &mut Vec<String>) -> String {
    let mut cp = cp.clone();
    if let Some(other) = args
        .into_iter()
        .position(|e| e.starts_with("-Djava.class.path="))
    {
        let other_cp = args.remove(other).clone().split_at(18).1.to_string();
        cp.push(other_cp.replace(separator::OTHER_SEPARATOR, separator::CLASSPATH_SEPARATOR));
    }

    format!(
        "-Djava.class.path={}",
        cp.join(separator::CLASSPATH_SEPARATOR)
    )
}