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
    jobject_wrapper(T object, jvm_env env) : obj(object), shared_releaser([object, env]{
        if (object != nullptr) {
            env->DeleteLocalRef(object);
        }
    }) {}

    template<class = int> requires std::negation_v<std::is_same<T, jobject>>
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

class java_function {

};

class java_constructor;

class jni_wrapper {
public:
    jni_wrapper(const std::string &jvmPath, JavaVMInitArgs &jvm_args);

    [[nodiscard]] jobject_wrapper<jstring> string_to_jstring(const std::string &str) const;

    std::string jstring_to_string(jstring str) const;

    [[nodiscard]] std::vector<java_constructor> getClassConstructors(const std::string &className) const;

    [[nodiscard]] java_exception getLastException() const;

    std::string get_object_class_name(jobject obj) const;

    JNIEnv *operator->() const;

    std::function<jni_types::JNI_CreateJavaVM_t> JNI_CreateJavaVM = nullptr;

    jvm_env env;
private:
    shared_library library;
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

#endif //NODE_JAVA_BRIDGE_JNI_WRAPPER_HPP
