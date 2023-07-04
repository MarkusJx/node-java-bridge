use crate::java_type::JavaType;
use crate::objects::args::JavaArgs;
use crate::util::util::ResultType;
use regex::Regex;
use std::fmt::Display;

#[derive(Clone)]
pub struct Signature {
    return_type: JavaType,
    args: Vec<JavaType>,
}

impl Signature {
    pub fn new(return_type: JavaType, args: Vec<JavaType>) -> Self {
        Self { return_type, args }
    }

    pub fn from_jni(sig: &str) -> ResultType<Self> {
        let captures = Regex::new(r"\((.*)\)(.+)")?
            .captures(sig)
            .ok_or("Failed to get captures".to_string())?;

        let args = captures
            .get(1)
            .ok_or("Failed to get arguments".to_string())?
            .as_str();
        let return_type = captures
            .get(2)
            .ok_or("Failed to get return type".to_string())?
            .as_str();

        Ok(Self {
            return_type: JavaType::new(return_type.to_string(), true),
            args: Self::parse_args(args)?,
        })
    }

    fn parse_args(args: &str) -> ResultType<Vec<JavaType>> {
        let chars = args.chars().collect::<Vec<char>>();
        let mut res = Vec::new();

        let char_at = |pos: usize| {
            chars
                .get(pos)
                .ok_or(format!("Failed to get char at position {}", pos))
                .map(|c| *c)
        };

        let mut i = 0;
        while i < chars.len() {
            let c = char_at(i)?;
            if c == 'L' {
                let end = args[i..].find(';').ok_or("Failed to find ;".to_string())?;
                let sig = args[i..i + end + 1].to_string();
                res.push(JavaType::new(sig, true));

                i += end;
            } else if c == '[' {
                let mut offset = 1;
                while char_at(i + offset)? == '[' {
                    offset += 1;
                }

                let next = char_at(i + offset)?;
                if next == 'L' {
                    let end = args[i + offset..]
                        .find(';')
                        .ok_or("Failed to find ;".to_string())?;
                    let sig = args[i..i + end + offset + 1].to_string();
                    res.push(JavaType::new(sig, true));

                    i += end + offset;
                } else {
                    let sig = args[i..i + offset + 1].to_string();
                    res.push(JavaType::new(sig, true));

                    i += offset;
                }
            } else {
                res.push(JavaType::new(c.to_string(), true));
            }

            i += 1;
        }

        Ok(res)
    }

    pub fn get_return_type(&self) -> &JavaType {
        &self.return_type
    }

    pub fn get_args(&self) -> &Vec<JavaType> {
        &self.args
    }

    pub fn num_args(&self) -> usize {
        self.args.len()
    }

    pub fn to_jni_signature(&self) -> String {
        format!(
            "({}){}",
            self.args
                .iter()
                .map(|arg| arg.to_jni_type())
                .collect::<String>(),
            self.return_type.to_jni_type()
        )
    }

    pub fn matches(&self, args: &JavaArgs) -> bool {
        if self.num_args() != args.len() {
            return false;
        }

        for (i, arg) in self.args.iter().enumerate() {
            let s = arg.type_enum();
            let t = args.get(i).unwrap().get_type();
            if s.is_object() != t.is_object() && s != t {
                return false;
            }
        }

        true
    }

    pub fn as_method_signature(&self, method_name: &String) -> String {
        format!(
            "{} {}({})",
            self.return_type.to_string(),
            method_name,
            self.args
                .iter()
                .map(|arg| arg.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({})",
            self.return_type.to_string(),
            self.args
                .iter()
                .map(|arg| arg.to_string())
                .collect::<Vec<String>>()
                .join(", "),
        )
    }
}
