use crate::node::util::util::ResultType;
use java_rs::function;
use java_rs::java_env::JavaEnv;
use java_rs::objects::args::AsJavaArg;
use java_rs::objects::array::JavaObjectArray;
use java_rs::objects::class::JavaClass;
use java_rs::objects::java_object::JavaObject;
use java_rs::objects::string::JavaString;

pub struct JsError {
    message: String,
    stack: Vec<String>,
}

impl JsError {
    pub fn new(message: String, mut stack: Vec<String>) -> Self {
        Self::push_stack(&mut stack, function!(), file!(), line!());
        Self { message, stack }
    }

    pub fn push(&mut self, method: &str, file: &str, line: u32) {
        Self::push_stack(&mut self.stack, method, file, line);
    }

    pub fn push_stack(stack: &mut Vec<String>, method: &str, file: &str, line: u32) {
        stack.insert(0, format!("\tat {} ({}:{})", method, file, line));
    }

    pub fn throw(&self, env: &JavaEnv) -> ResultType<()> {
        let utils = JavaClass::by_name("io/github/markusjx/bridge/Util", &env)?;
        let exception_from_js_error = utils.get_static_object_method(
            "exceptionFromJsError",
            "(Ljava/lang/String;[Ljava/lang/String;)Ljava/lang/Exception;",
        )?;

        let mut stack = self.stack.clone();
        Self::push_stack(&mut stack, function!(), file!(), line!());

        let string_class = JavaClass::by_name("java/lang/String", &env)?;
        let mut java_stack = JavaObjectArray::new(&string_class, stack.len() as _)?;

        for i in 0..stack.len() {
            java_stack.set(
                i as _,
                Some(JavaObject::from(JavaString::from_string(
                    stack.get(i).unwrap().clone(),
                    &env,
                )?)),
            )?;
        }

        let exception = exception_from_js_error
            .call(&[
                JavaString::from_string(self.message.clone(), &env)?.as_arg(),
                java_stack.as_arg(),
            ])?
            .ok_or(
                "io/github/markusjx/bridge/Util.exceptionFromJsError returned null".to_string(),
            )?;
        env.throw(JavaObject::from(exception));
        Ok(())
    }

    pub fn message(&self) -> String {
        self.message.clone()
    }
}
