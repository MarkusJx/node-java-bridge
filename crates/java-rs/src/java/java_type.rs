use crate::java::java_env::JavaEnv;
use crate::java::objects::class::JavaClass;
use crate::java::util::util::{jni_type_to_java_type, ResultType};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

/// A java type.
/// Used to represent the type of an java object.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum Type {
    Void = 0,
    Object = 1,
    Array = 2,
    Integer = 3,
    Boolean = 4,
    Byte = 5,
    Character = 6,
    Short = 7,
    Long = 8,
    Float = 9,
    Double = 10,
    LangInteger = 11,
    LangBoolean = 12,
    LangByte = 13,
    LangCharacter = 14,
    LangShort = 15,
    LangLong = 16,
    LangFloat = 17,
    LangDouble = 18,
    LangObject = 19,
    String = 20,
}

impl Type {
    pub fn is_object(&self) -> bool {
        match self {
            Type::Object
            | Type::Array
            | Type::LangBoolean
            | Type::LangInteger
            | Type::LangByte
            | Type::LangCharacter
            | Type::LangShort
            | Type::LangLong
            | Type::LangFloat
            | Type::LangDouble
            | Type::LangObject
            | Type::String => true,
            _ => false,
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let status_string = format!("{:?}", self);
        write!(f, "{}", status_string)
    }
}

/// A wrapper around a java type.
/// This contains the [`Type`](Type), its internal
/// signature and possible inner types, when this is an array.
#[derive(Clone)]
pub struct JavaType {
    inner: Option<Arc<Mutex<JavaType>>>,
    signature: String,
    type_enum: Type,
    hash: Option<u64>,
}

impl JavaType {
    /// Create a new [`JavaType`](JavaType) from a signature.
    /// Either accepts java or jni signatures.
    /// When passing jni signatures, pass `true` to the `convert` argument,
    /// as this allows converting the jni type to a java type signature.
    /// When passing java signatures, you may pass `true` to the `convert` argument,
    /// as this will do nothing.
    ///
    /// # Examples
    /// ```rust
    /// use java_rs::java_type::JavaType;
    ///
    /// // Get the type from a java signature
    /// let java_type = JavaType::new("java.lang.String".into(), true);
    ///
    /// // Get the type from a jni signature
    /// let jni_type = JavaType::new("Ljava/lang/String;".into(), true);
    ///
    /// // Both should be the same
    /// assert_eq!(java_type.to_string(), jni_type.to_string());
    /// ```
    pub fn new(mut signature: String, convert: bool) -> Self {
        if convert {
            signature = jni_type_to_java_type(&signature);
        }

        let type_enum: Type;
        let mut inner: Option<Arc<Mutex<JavaType>>> = None;

        match signature.as_str() {
            "void" => {
                type_enum = Type::Void;
            }
            "int" => {
                type_enum = Type::Integer;
            }
            "boolean" => {
                type_enum = Type::Boolean;
            }
            "byte" => {
                type_enum = Type::Byte;
            }
            "char" => {
                type_enum = Type::Character;
            }
            "short" => {
                type_enum = Type::Short;
            }
            "long" => {
                type_enum = Type::Long;
            }
            "float" => {
                type_enum = Type::Float;
            }
            "double" => {
                type_enum = Type::Double;
            }
            "java.lang.Integer" => {
                type_enum = Type::LangInteger;
            }
            "java.lang.Boolean" => {
                type_enum = Type::LangBoolean;
            }
            "java.lang.Byte" => {
                type_enum = Type::LangByte;
            }
            "java.lang.Character" => {
                type_enum = Type::LangCharacter;
            }
            "java.lang.Short" => {
                type_enum = Type::LangShort;
            }
            "java.lang.Long" => {
                type_enum = Type::LangLong;
            }
            "java.lang.Float" => {
                type_enum = Type::LangFloat;
            }
            "java.lang.Double" => {
                type_enum = Type::LangDouble;
            }
            "java.lang.String" => {
                type_enum = Type::String;
            }
            "java.lang.Object" => {
                type_enum = Type::LangObject;
            }
            _ => {
                if signature.ends_with("[]") {
                    type_enum = Type::Array;
                    inner = Some(Arc::new(Mutex::new(JavaType::new(
                        signature.clone()[0..(signature.len() - 2)].to_string(),
                        false,
                    ))));
                } else {
                    type_enum = Type::Object;
                }
            }
        }

        Self {
            inner,
            signature,
            type_enum,
            hash: None,
        }
    }

    pub fn from_existing(inner: Option<JavaType>, signature: String, type_enum: Type) -> Self {
        Self {
            inner: inner.map(|inner| Arc::new(Mutex::new(inner))),
            signature,
            type_enum,
            hash: None,
        }
    }

    pub fn array(inner: JavaType) -> Self {
        Self {
            signature: format!("{}[]", inner.signature),
            inner: Some(Arc::new(Mutex::new(inner))),
            type_enum: Type::Array,
            hash: None,
        }
    }

    pub fn object() -> Self {
        Self::new("java.lang.Object".to_string(), false)
    }

    pub fn integer() -> Self {
        Self::new("java.lang.Integer".to_string(), false)
    }

    pub fn boolean() -> Self {
        Self::new("java.lang.Boolean".to_string(), false)
    }

    pub fn byte() -> Self {
        Self::new("java.lang.Byte".to_string(), false)
    }

    pub fn character() -> Self {
        Self::new("java.lang.Character".to_string(), false)
    }

    pub fn short() -> Self {
        Self::new("java.lang.Short".to_string(), false)
    }

    pub fn long() -> Self {
        Self::new("java.lang.Long".to_string(), false)
    }

    pub fn float() -> Self {
        Self::new("java.lang.Float".to_string(), false)
    }

    pub fn double() -> Self {
        Self::new("java.lang.Double".to_string(), false)
    }

    pub fn string() -> Self {
        Self::new("java.lang.String".to_string(), false)
    }

    pub fn void() -> Self {
        Self::new("void".to_string(), false)
    }

    /// Get the return type of a jni method signature. This is used by
    /// [`get_method_id`](crate::java::java_env_wrapper::JavaEnvWrapper::get_method_id) and
    /// [`get_static_method_id`](crate::java::java_env_wrapper::JavaEnvWrapper::get_static_method_id).
    ///
    /// # Examples
    /// ```rust
    /// use java_rs::java_type::JavaType;
    ///
    /// // Get the return type of a method signature
    /// let java_type = JavaType::from_method_return_type("()I").unwrap();
    ///
    /// let expected = JavaType::new("int".into(), false);
    /// assert_eq!(java_type.to_string(), expected.to_string());
    /// ```
    pub fn from_method_return_type(method_signature: &str) -> ResultType<Self> {
        let signature = method_signature.split(")").last().ok_or(format!(
            "Could not get the return type of signature '{}'",
            method_signature
        ))?;

        if signature.len() == 0 || signature.contains('(') || signature.contains(')') {
            Err(format!(
                "Could not get the return type of signature '{}'",
                method_signature
            )
            .into())
        } else {
            Ok(JavaType::new(signature.to_string(), true))
        }
    }

    /// Get the inner type of this java type.
    /// The option is only set if this is an array.
    ///
    /// # Examples
    /// ```rust
    /// use java_rs::java_type::{JavaType, Type};
    ///
    /// let t = JavaType::new("java.lang.String[]".into(), false);
    /// let inner = t.inner();
    ///
    /// assert_eq!(inner.is_some(), t.is_array());
    /// assert_eq!(inner.unwrap().lock().unwrap().type_enum(), Type::String);
    /// ```
    pub fn inner(&self) -> Option<Arc<Mutex<JavaType>>> {
        self.inner.clone()
    }

    /// Get the type enum of this java type.
    pub fn type_enum(&self) -> Type {
        self.type_enum
    }

    /// Get a hash of the signature of this type.
    /// If the signature is not yet hashed, it will be hashed and stored.
    /// Therefore `self` must be mutable.
    ///
    /// This is used by [`equals`](Self::equals) to compare two (java) types.
    pub fn get_hash(&mut self) -> u64 {
        if self.hash.is_none() {
            let mut hasher = DefaultHasher::new();
            self.signature.hash(&mut hasher);
            self.hash = Some(hasher.finish());
        }

        self.hash.unwrap()
    }

    /// Check if this equals another type.
    /// This is done by comparing the hashes of the signatures
    /// of both types. The signatures are not computed when calling
    /// [`new`](Self::new) but rather when calling [`get_hash`](Self::get_hash),
    /// which this does. The hashes are stored for later use though, thus
    /// `self` and `other` must be mutable.
    ///
    /// This is the same as [`JavaType == JavaType`](PartialEq<JavaType>)
    /// but with (cached) hashes, thus, faster, when called multiple times.
    ///
    /// # Examples
    /// ```rust
    /// use java_rs::java_type::JavaType;
    ///
    /// let mut type1 = JavaType::new("java.lang.String".to_string(), true);
    /// let mut type2 = JavaType::new("java.lang.String".to_string(), true);
    ///
    /// // Note: This call to equals will compute and
    /// // store the hashes of type1 and type2
    /// assert!(type1.equals(&mut type2));
    ///
    /// let mut type3 = JavaType::new("java.lang.Object".to_string(), true);
    /// // At this point, the cached hash of type1 will be used but
    /// // the hash of type3 will be computed and stored.
    /// assert!(!type1.equals(&mut type3));
    /// ```
    pub fn equals(&mut self, other: &mut JavaType) -> bool {
        self.get_hash() == other.get_hash()
    }

    /// Get this as a JNI type instead of the java signature.
    pub fn to_jni_type(&self) -> String {
        match self.type_enum {
            Type::Void => "V".to_string(),
            Type::Array => format!(
                "[{}",
                self.inner.as_ref().unwrap().lock().unwrap().to_jni_type()
            ),
            Type::Integer => "I".to_string(),
            Type::Boolean => "Z".to_string(),
            Type::Byte => "B".to_string(),
            Type::Character => "C".to_string(),
            Type::Short => "S".to_string(),
            Type::Long => "J".to_string(),
            Type::Float => "F".to_string(),
            Type::Double => "D".to_string(),
            _ => format!("L{};", self.signature.replace(".", "/")),
        }
    }

    pub fn get_most_inner_signature(&self) -> String {
        match self.inner {
            Some(ref inner) => inner.lock().unwrap().get_most_inner_signature(),
            None => self.signature.clone(),
        }
    }

    pub fn is_int(&self) -> bool {
        self.type_enum == Type::Integer || self.type_enum == Type::LangInteger
    }

    pub fn is_long(&self) -> bool {
        self.type_enum == Type::Long || self.type_enum == Type::LangLong
    }

    pub fn is_float(&self) -> bool {
        self.type_enum == Type::Float || self.type_enum == Type::LangFloat
    }

    pub fn is_double(&self) -> bool {
        self.type_enum == Type::Double || self.type_enum == Type::LangDouble
    }

    pub fn is_boolean(&self) -> bool {
        self.type_enum == Type::Boolean || self.type_enum == Type::LangBoolean
    }

    pub fn is_char(&self) -> bool {
        self.type_enum == Type::Character || self.type_enum == Type::LangCharacter
    }

    pub fn is_byte(&self) -> bool {
        self.type_enum == Type::Byte || self.type_enum == Type::LangByte
    }

    pub fn is_byte_array(&self) -> bool {
        self.type_enum == Type::Array && self.inner.as_ref().unwrap().lock().unwrap().is_byte()
    }

    pub fn is_array(&self) -> bool {
        self.type_enum == Type::Array
    }

    pub fn is_primitive(&self) -> bool {
        self.type_enum() == Type::Void
            || self.type_enum() == Type::Integer
            || self.type_enum() == Type::Byte
            || self.type_enum() == Type::Boolean
            || self.type_enum() == Type::Character
            || self.type_enum() == Type::Short
            || self.type_enum() == Type::Long
            || self.type_enum() == Type::Float
            || self.type_enum() == Type::Double
    }

    pub fn as_class<'a>(&self, env: &'a JavaEnv<'a>) -> ResultType<JavaClass<'a>> {
        let class_name = match self.type_enum {
            Type::Void => "java.lang.Void",
            Type::Long => "java.lang.Long",
            Type::Integer => "java.lang.Integer",
            Type::Boolean => "java.lang.Boolean",
            Type::Byte => "java.lang.Byte",
            Type::Character => "java.lang.Character",
            Type::Short => "java.lang.Short",
            Type::Float => "java.lang.Float",
            Type::Double => "java.lang.Double",
            _ => &self.signature,
        };

        JavaClass::by_java_name(class_name.to_string(), env)
    }
}

impl PartialEq<Type> for JavaType {
    fn eq(&self, other: &Type) -> bool {
        self.type_enum == *other
    }
}

impl PartialEq<&JavaType> for Type {
    fn eq(&self, other: &&JavaType) -> bool {
        other.type_enum == *self
    }
}

impl PartialEq<Self> for JavaType {
    fn eq(&self, other: &Self) -> bool {
        self.signature == other.signature
    }
}

impl Display for JavaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.signature)
    }
}

unsafe impl Sync for JavaType {}
