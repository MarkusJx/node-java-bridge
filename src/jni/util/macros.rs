#[macro_export]
macro_rules! define_call_methods {
    ($name: ident, $static_name: ident, $method: ident, $static_method: ident, $result_type: ty, $result_name: ident, $converter: expr) => {
        pub fn $name<'b>(
            &'a self,
            object: JavaObject<'b>,
            method: &'b JavaMethod<'b>,
            args: JavaArgs,
        ) -> ResultType<$result_type> {
            unsafe {
                let args = self.convert_args(args);
                let $result_name = self.methods.$method.unwrap()(
                    self.env,
                    object.get_raw(),
                    method.id(),
                    args.as_ptr(),
                );
                if self.is_err() {
                    return Err(self.get_last_error(
                        file!(),
                        line!(),
                        true,
                        concat!(stringify!($method), " failed"),
                    )?);
                }

                Ok($converter)
            }
        }

        pub fn $static_name(
            &'a self,
            class: &'a JavaClass<'a>,
            method: &'a JavaMethod<'a>,
            args: JavaArgs,
        ) -> ResultType<$result_type> {
            unsafe {
                let args = self.convert_args(args);
                let $result_name = self.methods.$static_method.unwrap()(
                    self.env,
                    class.class(),
                    method.id(),
                    args.as_ptr(),
                );
                if self.is_err() {
                    return Err(self.get_last_error(
                        file!(),
                        line!(),
                        true,
                        concat!(stringify!($static_method), " failed"),
                    )?);
                }

                Ok($converter)
            }
        }
    };
}

#[macro_export]
macro_rules! define_array_methods {
    ($create_name: ident, $get_name: ident, $jni_type: ty, $rust_type: ty, $result_type: ty, $create_method: ident, $set_method: ident, $get_method: ident, $release_elements_method: ident) => {
        pub fn $create_name(&'a self, data: &Vec<$rust_type>) -> ResultType<$result_type> {
            let arr = unsafe { self.methods.$create_method.unwrap()(self.env, data.len() as i32) };
            if self.is_err() {
                return Err(self.get_last_error(
                    file!(),
                    line!(),
                    true,
                    concat!(stringify!($create_method), " failed"),
                )?);
            }

            unsafe {
                self.methods.$set_method.unwrap()(
                    self.env,
                    arr,
                    0,
                    data.len() as i32,
                    data.as_ptr() as $jni_type,
                );
            }
            if self.is_err() {
                self.delete_local_ref(arr);
                return Err(self.get_last_error(
                    file!(),
                    line!(),
                    true,
                    concat!(stringify!($set_method), " failed"),
                )?);
            }

            Ok(<$result_type>::from(LocalJavaObject::new(arr, self)))
        }

        pub fn $get_name(&'a self, array: sys::jobject) -> ResultType<Vec<$rust_type>> {
            let is_copy = std::ptr::null_mut();
            let elements = unsafe { self.methods.$get_method.unwrap()(self.env, array, is_copy) };
            if self.is_err() || elements == std::ptr::null_mut() {
                return Err(self.get_last_error(
                    file!(),
                    line!(),
                    true,
                    concat!(stringify!($get_method), " failed"),
                )?);
            }

            let length_res = self.get_array_length(array);
            if length_res.is_err() {
                unsafe {
                    self.methods.$release_elements_method.unwrap()(self.env, array, elements, 0);
                }

                if self.is_err() {
                    self.clear_err();
                }

                return Err(length_res.unwrap_err());
            }

            let length = length_res.unwrap();
            let mut data = Vec::with_capacity(length as usize);

            for i in 0..length {
                let element = unsafe { *elements.offset(i as isize) };
                data.push(element);
            }

            unsafe {
                self.methods.$release_elements_method.unwrap()(self.env, array, elements, 0);
            }
            Ok(data)
        }
    };
}

#[macro_export]
macro_rules! define_java_methods {
    ($name: ident, $bound_name: ident, $static_name: ident, $method: ident, $static_method: ident, $result_type: ty, $allowed_types: expr) => {
        pub struct $name<'a>(JavaMethod<'a>);

        impl<'a> $name<'a> {
            #[allow(dead_code)]
            pub fn new(method: JavaMethod<'a>) -> Self {
                Self(method)
            }

            pub fn bind(self, object: JavaObject<'a>) -> $bound_name<'a> {
                $bound_name::new(self, object)
            }

            pub fn call(
                &self,
                object: JavaObject<'_>,
                args: JavaArgs<'_>,
            ) -> ResultType<$result_type> {
                self.0.class.env().$method(object, &self.0, args)
            }

            pub fn from_global(
                method: GlobalJavaMethod,
                class: &'a JavaClass<'a>,
            ) -> ResultType<Self> {
                if method.is_static {
                    return Err("Tried creating a non-static method from a static method".into());
                }

                let t = method.return_type.type_enum();
                let allowed_types = $allowed_types;
                if !allowed_types.contains(&t) {
                    return Err(format!(
                        "{} is not a valid return type for this method, allowed types are: {}",
                        t,
                        allowed_types
                            .iter()
                            .map(|t| t.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                    .into());
                }

                Ok(Self(JavaMethod::new(
                    method.method.load(Ordering::Relaxed),
                    &class,
                    method.return_type,
                    method.is_static,
                )))
            }
        }

        impl<'a> Into<JavaMethod<'a>> for $name<'a> {
            fn into(self) -> JavaMethod<'a> {
                self.0
            }
        }

        pub struct $bound_name<'a> {
            method: $name<'a>,
            object: JavaObject<'a>,
        }

        impl<'a> $bound_name<'a> {
            pub fn new(method: $name<'a>, object: JavaObject<'a>) -> Self {
                Self { method, object }
            }

            pub fn call(&'a self, args: JavaArgs<'_>) -> ResultType<$result_type> {
                self.method.call(self.object.clone(), args)
            }
        }

        impl<'a> Into<JavaMethod<'a>> for $bound_name<'a> {
            fn into(self) -> JavaMethod<'a> {
                self.method.0
            }
        }

        pub struct $static_name<'a>(JavaMethod<'a>);

        impl<'a> $static_name<'a> {
            #[allow(dead_code)]
            pub fn new(method: JavaMethod<'a>) -> Self {
                Self(method)
            }

            pub fn call(&self, args: JavaArgs<'_>) -> ResultType<$result_type> {
                self.0
                    .class
                    .env()
                    .$static_method(self.0.class, &self.0, args)
            }

            pub fn from_global(
                method: GlobalJavaMethod,
                class: &'a JavaClass<'a>,
            ) -> ResultType<Self> {
                if !method.is_static {
                    return Err("Tried creating a static method from a non-static method".into());
                }

                let t = method.return_type.type_enum();
                let allowed_types = $allowed_types;
                if !allowed_types.contains(&t) {
                    return Err(format!(
                        "{} is not a valid return type for this method, allowed types are: {}",
                        t,
                        allowed_types
                            .iter()
                            .map(|t| t.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                    .into());
                }

                Ok(Self(JavaMethod::new(
                    method.method.load(Ordering::Relaxed),
                    &class,
                    method.return_type,
                    method.is_static,
                )))
            }
        }

        impl<'a> Into<JavaMethod<'a>> for $static_name<'a> {
            fn into(self) -> JavaMethod<'a> {
                self.0
            }
        }
    };
}

#[macro_export]
macro_rules! define_array {
    ($name: ident, $new_fn: ident, $get_fn: ident, $type: ident) => {
        pub struct $name<'a>(JavaArray<'a>);

        impl<'a> $name<'a> {
            pub fn new(env: &'a JavaEnv<'a>, data: &Vec<$type>) -> ResultType<Self> {
                env.get_env().$new_fn(data)
            }

            pub fn len(&self) -> ResultType<i32> {
                self.0.len()
            }

            pub fn get_data(&self) -> ResultType<Vec<$type>> {
                self.0.object.env().$get_fn(unsafe {
                    self.0
                        .object
                        .get_raw()
                })
            }
        }

        impl<'a> ToJavaValue<'a> for $name<'a> {
            fn to_java_value(&'a self) -> JavaValue<'a> {
                JavaValue::new(sys::jvalue {
                    l: unsafe { self.0.get_raw() },
                })
            }
        }

        impl<'a> From<LocalJavaObject<'a>> for $name<'a> {
            fn from(object: LocalJavaObject<'a>) -> Self {
                Self(JavaArray::from(object))
            }
        }

        impl<'a> From<JavaArray<'a>> for $name<'a> {
            fn from(array: JavaArray<'a>) -> Self {
                Self(array)
            }
        }

        impl<'a> Into<JavaObject<'a>> for $name<'a> {
            fn into(self) -> JavaObject<'a> {
                JavaObject::from(self.0.object)
            }
        }
    };
}

#[macro_export]
macro_rules! define_java_value {
    ($name: ident, $type: ty, $union_name: ident) => {
        pub struct $name($type);

        impl $name {
            pub fn new(value: $type) -> Self {
                Self(value)
            }
        }

        impl<'a> ToJavaValue<'a> for $name {
            fn to_java_value(&'a self) -> JavaValue<'a> {
                JavaValue::new(sys::jvalue {
                    $union_name: self.0 as _,
                })
            }
        }

        impl<'a> Into<JavaValue<'a>> for $name {
            fn into(self) -> JavaValue<'a> {
                JavaValue::new(sys::jvalue {
                    $union_name: self.0 as _,
                })
            }
        }
    };
}

#[macro_export]
macro_rules! define_get_method_method {
    ($name: ident, $getter: ident, $result_type: ident) => {
        pub fn $name(&'a self, method_name: &str, signature: &str) -> ResultType<$result_type<'a>> {
            let method = self.0.env().$getter(&self, method_name, signature)?;

            Ok($result_type::new(method))
        }
    };
}

#[macro_export]
macro_rules! define_object_to_val_method {
    ($name: ident, $result_type: ty, $class_name: expr, $method_name: expr, $signature: expr, $method_getter: ident) => {
        pub fn $name(&self, object: &LocalJavaObject) -> ResultType<$result_type> {
            let class = self.find_class($class_name)?;
            let method = class.$method_getter($method_name, $signature)?;

            method.call(JavaObject::from(object), vec![])
        }
    };
}

/// Define a method to create a new Java object from a primitive value.
/// # Example
/// Create a method to create a `java.lang.Integer` from a `i32`.
/// ```rust
/// define_object_value_of_method!(
///    /// Some doc comment
///    => from_i32, "java/lang/Integer", "I", i32, JavaInt
/// );
/// ```
///
/// # Arguments
/// * `name` - The name of the method.
/// * `class_name` - The signature of the class to create the object from.
/// * `java_input_type` - The java primitive input type signature.
/// * `value_type` - The input rust primitive type
/// * `java_value_type` - The java primitive wrapper type.
#[macro_export]
macro_rules! define_object_value_of_method {
    ($(#[$attr:meta])* => $name: ident, $class_name: expr, $java_input_type: expr, $value_type: ty, $java_value_type: ty) => {
        /// Create a new Java object from a value using the `valueOf` method.
        /// Use this if java objects rather than primitives are required as inputs.
        ///
        $(#[$attr])*
        /// # Arguments
        /// * `env` - The environment to use.
        /// * `value` - The value to convert.
        /// # Returns
        /// A new Java object.
        pub fn $name(env: &'a JavaEnv<'a>, value: $value_type) -> ResultType<LocalJavaObject<'a>> {
            let class = JavaClass::by_name($class_name, env)?;
            let method = class.get_static_object_method("valueOf", format!("({})L{};", $java_input_type, $class_name).as_str())?;

            let val = <$java_value_type>::new(value);
            let res = method.call(vec![Box::new(&val)])?.ok_or(format!("{}.valueOf() returned null", $class_name))?;

            Ok(res.assign_env(env.get_env()))
        }
    };
}

#[macro_export]
macro_rules! define_field {
    ($name: ident, $static_name: ident, $value_type: ty, $getter: ident, $setter: ident, $static_getter: ident, $static_setter: ident, $call_res_type: ident, $value: ident, $set_converter: expr, $get_converter: expr, $allowed_types: expr) => {
        pub struct $name<'a>(JavaField<'a>);

        impl<'a> $name<'a> {
            pub fn new(field: JavaField<'a>) -> Self {
                Self(field)
            }

            pub fn get(&self, object: &JavaObject<'_>) -> ResultType<$value_type> {
                self.0.class.env().$getter(&self, object)
            }

            pub fn set(&self, object: &JavaObject<'_>, value: $value_type) -> ResultType<()> {
                self.0.class.env().$setter(self, object, value)
            }

            pub fn from_global(
                field: GlobalJavaField,
                class: &'a JavaClass<'a>,
            ) -> ResultType<Self> {
                if field.is_static {
                    return Err("Tried creating a non-static field from a static field".into());
                }

                let t = field.field_type.type_enum();
                if !$allowed_types.contains(&t) {
                    return Err(format!("{} is not a valid type for this field", t).into());
                }

                unsafe {
                    Ok(Self(JavaField::new(
                        field.field.load(Ordering::Relaxed),
                        field.field_type,
                        &class,
                        field.is_static,
                    )))
                }
            }

            pub(in crate::jni) unsafe fn id(&self) -> sys::jfieldID {
                self.0.id()
            }
        }

        impl<'a> JavaFieldValues for $name<'a> {
            fn set(&self, object: &JavaObject<'_>, value: JavaCallResult) -> ResultType<()> {
                match value {
                    JavaCallResult::$call_res_type($value) => self.set(object, $set_converter),
                    _ => Err(
                        concat!("Invalid value type supplied for field ", stringify!($name)).into(),
                    ),
                }
            }

            fn get(&self, object: &JavaObject<'_>) -> ResultType<JavaCallResult> {
                let $value = self.get(object)?;
                Ok(JavaCallResult::$call_res_type($get_converter))
            }
        }

        pub struct $static_name<'a>(JavaField<'a>);

        impl<'a> $static_name<'a> {
            pub fn new(field: JavaField<'a>) -> Self {
                Self(field)
            }

            pub fn get(&self) -> ResultType<$value_type> {
                self.0.class.env().$static_getter(self, self.0.class)
            }

            pub fn set(&self, value: $value_type) -> ResultType<()> {
                self.0.class.env().$static_setter(self, self.0.class, value)
            }

            pub fn from_global(
                field: GlobalJavaField,
                class: &'a JavaClass<'a>,
            ) -> ResultType<Self> {
                if !field.is_static {
                    return Err("Tried creating a static field from a non-static field".into());
                }

                let t = field.field_type.type_enum();
                if !$allowed_types.contains(&t) {
                    return Err(format!("{} is not a valid type for this field", t).into());
                }

                unsafe {
                    Ok(Self(JavaField::new(
                        field.field.load(Ordering::Relaxed),
                        field.field_type,
                        &class,
                        field.is_static,
                    )))
                }
            }

            pub(in crate::jni) unsafe fn id(&self) -> sys::jfieldID {
                self.0.id()
            }
        }

        impl<'a> StaticJavaFieldValues for $static_name<'a> {
            fn set(&self, value: JavaCallResult) -> ResultType<()> {
                match value {
                    JavaCallResult::$call_res_type($value) => self.set($set_converter),
                    _ => Err(concat!(
                        "Invalid value type supplied for field ",
                        stringify!($static_name)
                    )
                    .into()),
                }
            }

            fn get(&self) -> ResultType<JavaCallResult> {
                let $value = self.get()?;
                Ok(JavaCallResult::$call_res_type($get_converter))
            }
        }
    };
}

#[macro_export]
macro_rules! define_field_methods {
    ($getter: ident, $setter: ident, $static_getter: ident, $static_setter: ident, $value_type: ty, $static_value_type: ty, $result_type: ty, $getter_method: ident, $setter_method: ident, $static_getter_method: ident, $static_setter_method: ident) => {
        pub fn $getter(
            &self,
            field: &$value_type,
            object: &JavaObject,
        ) -> ResultType<$result_type> {
            unsafe {
                let res = self.methods.$getter_method.unwrap()(
                    self.env,
                    object.get_raw(),
                    field.id(),
                );
                if self.is_err() {
                    Err(self.get_last_error(
                        file!(),
                        line!(),
                        true,
                        concat!(stringify!($getter), " failed"),
                    )?)
                } else {
                    Ok(res)
                }
            }
        }

        pub fn $setter(
            &self,
            field: &$value_type,
            object: &JavaObject,
            value: $result_type,
        ) -> ResultType<()> {
            unsafe {
                self.methods.$setter_method.unwrap()(
                    self.env,
                    object.get_raw(),
                    field.id(),
                    value,
                );
                if self.is_err() {
                    Err(self.get_last_error(
                        file!(),
                        line!(),
                        true,
                        concat!(stringify!($setter), " failed"),
                    )?)
                } else {
                    Ok(())
                }
            }
        }

        pub fn $static_getter(
            &self,
            field: &$static_value_type,
            class: &JavaClass<'_>,
        ) -> ResultType<$result_type> {
            unsafe {
                let res = self.methods.$static_getter_method.unwrap()(
                    self.env,
                    class.class(),
                    field.id(),
                );
                if self.is_err() {
                    Err(self.get_last_error(
                        file!(),
                        line!(),
                        true,
                        concat!(stringify!($static_getter), " failed"),
                    )?)
                } else {
                    Ok(res)
                }
            }
        }

        pub fn $static_setter(
            &self,
            field: &$static_value_type,
            class: &JavaClass<'_>,
            value: $result_type,
        ) -> ResultType<()> {
            unsafe {
                self.methods.$static_setter_method.unwrap()(
                    self.env,
                    class.class(),
                    field.id(),
                    value,
                );
                if self.is_err() {
                    Err(self.get_last_error(
                        file!(),
                        line!(),
                        true,
                        concat!(stringify!($static_setter), " failed"),
                    )?)
                } else {
                    Ok(())
                }
            }
        }
    };
}

/// Source: https://stackoverflow.com/a/40234666
#[macro_export]
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}
