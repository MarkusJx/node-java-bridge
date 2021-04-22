#include <stdexcept>
#include <sstream>
#include <iostream>

#include "jni_wrapper.hpp"

#define CHECK_EXCEPTION() if (env->ExceptionCheck()) throw getLastException()
#define JVM_CHECK_EXCEPTION(jvm) if (jvm->ExceptionCheck()) throw jvm.getLastException()

using namespace jni_types;

std::string jni_error_to_string(jint code) {
    switch (code) {
        case JNI_EDETACHED:
            return "Thread detached from the vm";
        case JNI_EVERSION:
            return "JNI version error";
        case JNI_ENOMEM:
            return "Not enough memory";
        case JNI_EEXIST:
            return "VM already created";
        case JNI_EINVAL:
            return "Invalid arguments";
        default:
            return "Unknown error";
    }
}

java_exception::java_exception(std::vector<std::string> causes, std::vector<std::string> frames) : causes(
        std::move(causes)), frames(std::move(frames)), std::exception(), message() {
    std::stringstream ss;
    for (size_t i = 0; i < this->causes.size(); i++) {
        if (i == 0) {
            ss << this->causes[i];

            if (!this->frames.empty()) {
                ss << std::endl;
            }

            for (const std::string &frame : this->frames) {
                ss << '\t' << "at " << frame << std::endl;
            }
        } else {
            ss << "Caused by: " << this->causes[i];
        }

        if ((i + 1) < causes.size()) {
            ss << std::endl;
        }
    }

    message = ss.str();
}

java_exception::java_exception(const java_exception &other) = default;

const char *java_exception::what() const {
    return message.c_str();
}

java_constructor::java_constructor(jobject object, const jni_wrapper &jni) : jobject_wrapper<jobject>(object, jni.env),
                                                                             jni(jni) {}

jint java_constructor::numArguments() const {
    jclass constructor = jni->FindClass("java/lang/reflect/Constructor");
    JVM_CHECK_EXCEPTION(jni);

    jmethodID getParameterCount = jni->GetMethodID(constructor, "getParameterCount", "()I");
    JVM_CHECK_EXCEPTION(jni);

    jint res = jni->CallIntMethod(obj, getParameterCount);
    JVM_CHECK_EXCEPTION(jni);

    return res;
}

std::vector<std::string> java_constructor::getParameterTypes() const {
    jclass Constructor = jni->FindClass("java/lang/reflect/Constructor");
    JVM_CHECK_EXCEPTION(jni);

    jmethodID getParameters = jni->GetMethodID(Constructor, "getParameters", "()[Ljava/lang/reflect/Parameter;");
    JVM_CHECK_EXCEPTION(jni);

    jclass Parameter = jni->FindClass("java/lang/reflect/Parameter");
    JVM_CHECK_EXCEPTION(jni);

    jmethodID getType = jni->GetMethodID(Parameter, "getType", "()Ljava/lang/Class;");
    JVM_CHECK_EXCEPTION(jni);

    jclass Class = jni->FindClass("java/lang/Class");
    JVM_CHECK_EXCEPTION(jni);

    jmethodID getName = jni->GetMethodID(Class, "getName", "()Ljava/lang/String;");
    JVM_CHECK_EXCEPTION(jni);

    jobject_wrapper<jobjectArray> parameters(jni->CallObjectMethod(obj, getParameters), jni.env);
    JVM_CHECK_EXCEPTION(jni);

    jint numParams = jni->GetArrayLength(parameters);
    JVM_CHECK_EXCEPTION(jni);

    std::vector<std::string> res;
    for (jint i = 0; i < numParams; i++) {
        jobject elem = jni->GetObjectArrayElement(parameters, i);
        JVM_CHECK_EXCEPTION(jni);

        jobject_wrapper<jobject> type(jni->CallObjectMethod(elem, getType), jni.env);
        JVM_CHECK_EXCEPTION(jni);

        jobject_wrapper<jstring> name(jni->CallObjectMethod(type, getName), jni.env);
        JVM_CHECK_EXCEPTION(jni);

        res.push_back(jni.jstring_to_string(name));
    }

    return res;
}

jobject_wrapper<jobject> java_constructor::newInstance(const std::vector<jobject> &args) const {
    jclass constructor = jni->FindClass("java/lang/reflect/Constructor");
    JVM_CHECK_EXCEPTION(jni);

    jmethodID newInstance_m = jni->GetMethodID(constructor, "newInstance", "([Ljava/lang/Object;)Ljava/lang/Object;");
    JVM_CHECK_EXCEPTION(jni);

    jclass Object = jni->FindClass("java/lang/Object");
    JVM_CHECK_EXCEPTION(jni);
    jobject_wrapper<jobjectArray> argArr(jni->NewObjectArray(static_cast<jsize>(args.size()), Object, nullptr),
                                         jni.env);
    JVM_CHECK_EXCEPTION(jni);

    for (size_t i = 0; i < args.size(); i++) {
        jni->SetObjectArrayElement(argArr, static_cast<jsize>(i), args[i]);
        JVM_CHECK_EXCEPTION(jni);
    }

    jobject instance = jni->CallObjectMethod(obj, newInstance_m, argArr.obj);
    JVM_CHECK_EXCEPTION(jni);
    return jobject_wrapper(instance, jni.env);
}

std::string java_constructor::to_string() const {
    jclass constructor = jni->FindClass("java/lang/reflect/Constructor");
    JVM_CHECK_EXCEPTION(jni);

    jmethodID toString = jni->GetMethodID(constructor, "toString", "()Ljava/lang/String;");
    JVM_CHECK_EXCEPTION(jni);

    jobject_wrapper<jstring> string(jni->CallObjectMethod(obj, toString), jni.env);
    JVM_CHECK_EXCEPTION(jni);

    return jni.jstring_to_string(string);
}

jni_wrapper::jni_wrapper(const std::string &jvmPath, JavaVMInitArgs &jvm_args) : env() {
    library = shared_library(jvmPath);
    JNI_CreateJavaVM = library.getFunction<JNI_CreateJavaVM_t>("JNI_CreateJavaVM");

    JavaVM *jvm = nullptr;
    JNIEnv *environment = nullptr;

    jint create_code = JNI_CreateJavaVM(&jvm, (void **) &environment, &jvm_args);
    if (create_code != JNI_OK) {
        throw std::runtime_error("JNI_CreateJavaVM failed: " + jni_error_to_string(create_code));
    } else {
        env = jvm_env(jvm, environment);
    }
}

jobject_wrapper<jstring> jni_wrapper::string_to_jstring(const std::string &str) const {
    auto res = jobject_wrapper<jstring>(env->NewStringUTF(str.c_str()), env);
    if (env->ExceptionCheck()) {
        throw getLastException();
    } else if (res == nullptr) {
        throw std::runtime_error("Could not get the string");
    }

    return res;
}

std::string jni_wrapper::jstring_to_string(jstring str) const {
    const char *chars = env->GetStringUTFChars(str, nullptr);
    if (env->ExceptionCheck()) {
        throw getLastException();
    } else if (chars == nullptr) {
        throw std::runtime_error("Could not get the characters");
    }

    std::string res(chars);
    env->ReleaseStringUTFChars(str, chars);

    return res;
}

std::vector<java_constructor> jni_wrapper::getClassConstructors(const std::string &className) const {
    jclass Class = env->FindClass("java/lang/Class");
    CHECK_EXCEPTION();

    jmethodID forName = env->GetStaticMethodID(Class, "forName", "(Ljava/lang/String;)Ljava/lang/Class;");
    CHECK_EXCEPTION();

    jobject_wrapper<jobjectArray> clazz(env->CallStaticObjectMethod(Class, forName, string_to_jstring(className).obj),
                                        env);
    CHECK_EXCEPTION();

    jmethodID getConstructors = env->GetMethodID(Class, "getConstructors", "()[Ljava/lang/reflect/Constructor;");
    CHECK_EXCEPTION();

    jobject_wrapper<jobjectArray> constructors(env->CallObjectMethod(clazz, getConstructors), env);
    CHECK_EXCEPTION();

    jsize numConstructors = env->GetArrayLength(constructors);
    std::vector<java_constructor> java_constructors;

    for (jsize i = 0; i < numConstructors; i++) {
        jobject c = env->GetObjectArrayElement(constructors, i);
        java_constructors.emplace_back(c, *this);
    }

    return java_constructors;
}

java_exception jni_wrapper::getLastException() const {
    if (!env->ExceptionCheck()) {
        throw std::runtime_error("No exception occurred");
    }

    auto throwable = jobject_wrapper(env->ExceptionOccurred(), env);
    env->ExceptionClear();

    jclass throwable_class = env->FindClass("java/lang/Throwable");
    jmethodID throwable_getCause = env->GetMethodID(throwable_class, "getCause", "()Ljava/lang/Throwable;");

    jmethodID throwable_getStackTrace = env->GetMethodID(throwable_class, "getStackTrace",
                                                         "()[Ljava/lang/StackTraceElement;");

    jmethodID throwable_toString = env->GetMethodID(throwable_class, "toString", "()Ljava/lang/String;");

    jclass stacktrace_element_class = env->FindClass("java/lang/StackTraceElement");
    jmethodID stacktrace_toString = env->GetMethodID(stacktrace_element_class, "toString", "()Ljava/lang/String;");

    auto frames = jobject_wrapper<jobjectArray>(env->CallObjectMethod(throwable, throwable_getStackTrace), env);
    jsize numFrames = env->GetArrayLength(frames);

    std::vector<std::string> causes;
    std::vector<std::string> stackFrames;
    while (frames != nullptr && throwable.ok()) {
        auto throwable_string = jobject_wrapper<jstring>(env->CallObjectMethod(throwable, throwable_toString), env);
        causes.push_back(jstring_to_string(throwable_string));

        if (numFrames > 0) {
            for (jsize i = 0; i < numFrames; i++) {
                auto frame = jobject_wrapper(env->GetObjectArrayElement(frames, i), env);
                auto frame_string = jobject_wrapper<jstring>(env->CallObjectMethod(frame, stacktrace_toString), env);

                stackFrames.push_back(jstring_to_string(frame_string));
            }
        }

        throwable = env->CallObjectMethod(throwable, throwable_getCause);

        if (throwable != nullptr) {
            frames = env->CallObjectMethod(throwable, throwable_getStackTrace);
            numFrames = env->GetArrayLength(frames);
        }
    }

    return java_exception(causes, stackFrames);
}

std::string jni_wrapper::get_object_class_name(jobject obj) const {
    jclass Object = env->FindClass("java/lang/Object");
    CHECK_EXCEPTION();

    jmethodID  getClass = env->GetMethodID(Object, "getClass", "()Ljava/lang/Class;");
    CHECK_EXCEPTION();

    jobject_wrapper<jobject> clazz(env->CallObjectMethod(obj, getClass), env);
    CHECK_EXCEPTION();

    jclass Class = env->FindClass("java/lang/Class");
    CHECK_EXCEPTION();

    jmethodID getName = env->GetMethodID(Class, "getName", "()Ljava/lang/String;");
    CHECK_EXCEPTION();

    jobject_wrapper<jstring> name(env->CallObjectMethod(clazz, getName), env);
    CHECK_EXCEPTION();

    return jstring_to_string(name);
}

JNIEnv *jni_wrapper::operator->() const {
    return env.env;
}

jvm_env::jvm_env() : env(nullptr), jvm(nullptr), shared_releaser(nullptr) {}

jvm_env::jvm_env(JavaVM *vm, JNIEnv *environment) : env(environment), jvm(vm), shared_releaser([vm] {
    vm->DestroyJavaVM();
}) {}

JNIEnv *jvm_env::operator->() const {
    return env;
}