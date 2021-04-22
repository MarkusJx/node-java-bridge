#ifndef NODE_JAVA_BRIDGE_JNI_WRAPPER_HPP
#define NODE_JAVA_BRIDGE_JNI_WRAPPER_HPP

#include <functional>
#include <jni.h>
#include <shared_releaser.hpp>

#include "shared_library.hpp"

namespace jni_types {
    using JNI_CreateJavaVM_t = decltype(::JNI_CreateJavaVM);
}

class java_exception : public std::exception {
public:
    java_exception(std::vector<std::string> causes, std::vector<std::string> frames);

    java_exception(const java_exception &other);

    [[nodiscard]] const char *what() const override;

    const std::vector<std::string> causes, frames;

private:
    std::string message;
};

class jvm_env : public shared_releaser {
public:
    jvm_env();

    jvm_env(JavaVM *vm, JNIEnv *env);

    JNIEnv *operator->() const;

    JavaVM *jvm;
    JNIEnv *env;
};

template<class T>
class jobject_wrapper : public shared_releaser {
public:
    jobject_wrapper(T object, jvm_env env) : obj(object), shared_releaser([object, env] {
        if (object != nullptr) {
            env->DeleteLocalRef(object);
        }
    }) {}

    template<class = int>
    requires std::negation_v<std::is_same<T, jobject>>
    jobject_wrapper(jobject object, jvm_env env) : shared_releaser([object, env] {
        if (object != nullptr) {
            env->DeleteLocalRef(object);
        }
    }) {
        obj = reinterpret_cast<T>(object);
    }

    jobject_wrapper &operator=(jobject newObject) {
        this->reset();

        obj = reinterpret_cast<T>(newObject);
        return *this;
    }

    operator T() const {
        return obj;
    }

    [[nodiscard]] bool ok() const {
        return obj != nullptr;
    }

    T obj;
};

class java_constructor;

class java_field;

class java_function;

class java_class;

class jni_wrapper {
public:
    jni_wrapper(const std::string &jvmPath, JavaVMInitArgs &jvm_args);

    [[nodiscard]] jobject_wrapper<jstring> string_to_jstring(const std::string &str) const;

    std::string jstring_to_string(jstring str) const;

    [[nodiscard]] std::vector<java_constructor> getClassConstructors(const std::string &className) const;

    [[nodiscard]] jclass getJavaLangClass() const;

    [[nodiscard]] jobject_wrapper<jobject> getClassByName(const std::string &className) const;

    [[nodiscard]] std::vector<java_field> getClassFields(const std::string &className, bool onlyStatic) const;

    [[nodiscard]] std::vector<java_function> getClassFunctions(const std::string &className, bool onlyStatic) const;

    [[nodiscard]] java_class getClass(const std::string &className) const;

    [[nodiscard]] java_exception getLastException(int line = -1) const;

    std::string get_object_class_name(jobject obj) const;

    JNIEnv *operator->() const;

    std::function<jni_types::JNI_CreateJavaVM_t> JNI_CreateJavaVM = nullptr;

    jvm_env env;
private:
    shared_library library;
};

class java_field {
public:
    java_field(std::string signature, std::string name, jfieldID id);

    const std::string signature;
    const std::string name;
    jfieldID id;
};

class java_function {
public:
    java_function(std::vector<std::string> parameterTypes, std::string returnType, std::string functionName,
                  jmethodID method);

    const std::vector<std::string> parameterTypes;
    const std::string returnType;
    const std::string functionName;
    jmethodID method;
};

class java_constructor : public jobject_wrapper<jobject> {
public:
    java_constructor(jobject object, const jni_wrapper &jni);

    [[nodiscard]] jint numArguments() const;

    [[nodiscard]] std::vector<std::string> getParameterTypes() const;

    [[nodiscard]] jobject_wrapper<jobject> newInstance(const std::vector<jobject> &args) const;

    [[nodiscard]] std::string to_string() const;

private:
    jni_wrapper jni;
};

class java_class {
public:
    java_class(std::vector<java_field> static_fields, std::vector<java_field> fields,
               std::vector<java_function> static_functions, std::vector<java_function> functions,
               std::vector<java_constructor> constructors);

    const std::vector<java_field> static_fields, fields;
    const std::vector<java_function> static_functions, functions;
    const std::vector<java_constructor> constructors;
};

#endif //NODE_JAVA_BRIDGE_JNI_WRAPPER_HPP
