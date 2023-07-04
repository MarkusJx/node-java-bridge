# java-rs

Bindings to the JVM for rust focussing on calling java methods from rust.

This is mainly used by [`node-java-bridge`](https://github.com/MarkusJx/node-java-bridge).

## Usage

### Creating a new jvm

```rust
use java_rs::java_vm::{JavaVM, InternalJavaOptions};

fn create_jvm() -> ResultType<JavaVM> {
    JavaVM::new(
        &"1.8".to_string(),
        None,
        vec![],
        InternalJavaOptions::default()
    )
}
```

### Calling a java method

```rust
fn method_call() -> ResultType<()> {
    let env = jvm.attach_thread()?;

    let class = env.find_class("java/lang/String")?;
    let method = class.get_static_method_id(class, "valueOf", "(I)Ljava/lang/String;")?;
    let res = method.call(&[JavaInt::new(123).as_arg()])?;
    let str = JavaString::try_from(res)?;

    println!("{}", str.to_string()?);
}
```
