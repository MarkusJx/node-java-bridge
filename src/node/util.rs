use crate::node::napi_error::NapiError;
use glob::glob;
use napi::{JsString, JsUnknown};

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

pub(crate) fn list_files(dirs: Vec<String>, ignore_unreadable: bool) -> napi::Result<Vec<String>> {
    let mut files = Vec::<String>::new();

    for file in dirs {
        let paths = glob(file.as_str()).map_err(|e| NapiError::from(e.to_string()).into_napi())?;

        for path in paths {
            match path {
                Ok(path) => files.push(
                    path.to_str()
                        .ok_or(NapiError::from("Failed to convert path to string").into_napi())?
                        .to_string(),
                ),
                Err(e) => {
                    if !ignore_unreadable {
                        return Err(NapiError::from(e.to_string()).into_napi());
                    }
                }
            }
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
        let other_cp = args.remove(other).split_at(18).1.to_string();
        cp.push(other_cp.replace(separator::OTHER_SEPARATOR, separator::CLASSPATH_SEPARATOR));
    }

    format!(
        "-Djava.class.path={}",
        cp.join(separator::CLASSPATH_SEPARATOR)
    )
}
