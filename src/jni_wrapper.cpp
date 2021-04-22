#include <stdexcept>
#include <sstream>
#include <iostream>

#include "jni_wrapper.hpp"

#define CHECK_EXCEPTION() if (env->ExceptionCheck()) throw getLastException(__LINE__)
#define JVM_CHECK_EXCEPTION(jvm) if (jvm->ExceptionCheck()) throw jvm.getLastException(__LINE__)

using namespace jni_types;

/*
 * Types:
 *
 * +---+---------+
 * | Z | boolean |
 * | B | byte    |
 * | C | char    |
 * | S | short   |
 * | I | int     |
 * | J | long    |
 * | F | float   |
 * | D | double  |
 * | V | void    |
 * +-------------+
 */

std::string string_replace(std::string val, char to_replace, char replacement) {
    for (char &c : val) {
        if (c == to_replace) {
            c = replacement;
        }
    }

    return val;
}

std::string java_type_to_jni_type(const std::string &to_convert) {
    if (to_convert == "boolean") {
        return "Z";
    } else if (to_convert == "byte") {
        return "B";
    } else if (to_convert == "char") {
        return "C";
    } else if (to_convert == "short") {
        return "S";
    } else if (to_convert == "int") {
        return "I";
    } else if (to_convert == "long") {
        return "J";
    } else if (to_convert == "float") {
        return "F";
    } else if (to_convert == "double") {
        return "D";
    } else if (to_convert == "void") {
        return "V";
    } else {
        if (!to_convert.empty() && to_convert[0] != '[' && to_convert[0] != 'L') {
            return 'L' + string_replace(to_convert, '.', '/') + ';';
        } else {
            return string_replace(to_convert, '.', '/');
        }
    }
}

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

jclass jni_wrapper::getJavaLangClass() const {
    jclass Class = env->FindClass("java/lang/Class");
    CHECK_EXCEPTION();

    return Class;
}

jobject_wrapper<jobject> jni_wrapper::getClassByName(const std::string &className) const {
    jclass Class = getJavaLangClass();

    jmethodID forName = env->GetStaticMethodID(Class, "forName", "(Ljava/lang/String;)Ljava/lang/Class;");
    CHECK_EXCEPTION();

    jobject clazz = env->CallStaticObjectMethod(Class, forName, string_to_jstring(className).obj);
    CHECK_EXCEPTION();

    return jobject_wrapper<jobject>(clazz, env);
}

std::vector<java_constructor> jni_wrapper::getClassConstructors(const std::string &className) const {
    jobject_wrapper<jobject> clazz = getClassByName(className);

    jmethodID getConstructors = env->GetMethodID(getJavaLangClass(), "getConstructors",
                                                 "()[Ljava/lang/reflect/Constructor;");
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

std::vector<java_field> jni_wrapper::getClassFields(const std::string &className, bool onlyStatic) const {
    jclass Class = getJavaLangClass();
    jobject_wrapper<jobject> clazz = getClassByName(className);

    jmethodID getDeclaredFields = env->GetMethodID(Class, "getDeclaredFields", "()[Ljava/lang/reflect/Field;");
    CHECK_EXCEPTION();

    jclass Field = env->FindClass("java/lang/reflect/Field");
    CHECK_EXCEPTION();

    jmethodID getModifiers = env->GetMethodID(Field, "getModifiers", "()I");
    CHECK_EXCEPTION();

    jmethodID field_getType = env->GetMethodID(Field, "getType", "()Ljava/lang/Class;");
    CHECK_EXCEPTION();

    jmethodID field_getName = env->GetMethodID(Field, "getName", "()Ljava/lang/String;");
    CHECK_EXCEPTION();

    jmethodID class_getName = env->GetMethodID(Class, "getName", "()Ljava/lang/String;");
    CHECK_EXCEPTION();

    jclass Modifier = env->FindClass("java/lang/reflect/Modifier");
    CHECK_EXCEPTION();

    jmethodID isStatic = env->GetStaticMethodID(Modifier, "isStatic", "(I)Z");
    CHECK_EXCEPTION();

    jmethodID isPublic = env->GetStaticMethodID(Modifier, "isPublic", "(I)Z");
    CHECK_EXCEPTION();

    jobject_wrapper<jobjectArray> fields(env->CallObjectMethod(clazz, getDeclaredFields), env);
    CHECK_EXCEPTION();

    const jsize numFields = env->GetArrayLength(fields);
    CHECK_EXCEPTION();

    const auto getFieldSignature = [&](const jobject_wrapper<jobject> &field) -> std::string {
        jobject_wrapper<jobject> type(env->CallObjectMethod(field, field_getType), env);
        CHECK_EXCEPTION();

        jobject_wrapper<jstring> name(env->CallObjectMethod(type, class_getName), env);
        CHECK_EXCEPTION();

        return java_type_to_jni_type(jstring_to_string(name));
    };

    const auto getFieldName = [&](const jobject_wrapper<jobject> &field) -> std::string {
        jobject_wrapper<jstring> name(env->CallObjectMethod(field, field_getName), env);
        CHECK_EXCEPTION();

        return jstring_to_string(name);
    };

    const auto getFieldId = [&](const jobject_wrapper<jobject> &field, const std::string &fieldName,
                                const std::string &signature) -> jfieldID {
        jclass javaClass = env->FindClass(string_replace(className, '.', '/').c_str());
        CHECK_EXCEPTION();

        jfieldID id;
        if (onlyStatic) {
            id = env->GetStaticFieldID(javaClass, fieldName.c_str(), signature.c_str());
        } else {
            id = env->GetFieldID(javaClass, fieldName.c_str(), signature.c_str());
        }
        CHECK_EXCEPTION();

        return id;
    };

    std::vector<java_field> res;
    for (jsize i = 0; i < numFields; i++) {
        jobject_wrapper<jobject> field(env->GetObjectArrayElement(fields, i), env);
        CHECK_EXCEPTION();

        jint modifiers = env->CallIntMethod(field, getModifiers);
        CHECK_EXCEPTION();

        jboolean is_static = env->CallStaticBooleanMethod(Modifier, isStatic, modifiers);
        CHECK_EXCEPTION();

        jboolean is_public = env->CallStaticBooleanMethod(Modifier, isPublic, modifiers);
        CHECK_EXCEPTION();

        if (((onlyStatic && is_static) || (!onlyStatic && !is_static)) && is_public) {
            const std::string signature = getFieldSignature(field);
            const std::string name = getFieldName(field);
            jfieldID id = getFieldId(field, name, signature);
            res.emplace_back(signature, name, id);
        }
    }

    return res;
}

std::vector<java_function> jni_wrapper::getClassFunctions(const std::string &className, bool onlyStatic) const {
    jclass Class = getJavaLangClass();
    jobject_wrapper<jobject> clazz = getClassByName(className);

    jmethodID getDeclaredMethods = env->GetMethodID(Class, "getDeclaredMethods", "()[Ljava/lang/reflect/Method;");
    CHECK_EXCEPTION();

    jclass Method = env->FindClass("java/lang/reflect/Method");
    CHECK_EXCEPTION();

    jmethodID getName = env->GetMethodID(Method, "getName", "()Ljava/lang/String;");
    CHECK_EXCEPTION();

    jmethodID getReturnType = env->GetMethodID(Method, "getReturnType", "()Ljava/lang/Class;");
    CHECK_EXCEPTION();

    jmethodID getModifiers = env->GetMethodID(Method, "getModifiers", "()I");
    CHECK_EXCEPTION();

    jmethodID class_getName = env->GetMethodID(Class, "getName", "()Ljava/lang/String;");
    CHECK_EXCEPTION();

    jmethodID getParameterTypes = env->GetMethodID(Method, "getParameterTypes", "()[Ljava/lang/Class;");
    CHECK_EXCEPTION();

    jclass Modifier = env->FindClass("java/lang/reflect/Modifier");
    CHECK_EXCEPTION();

    jmethodID isStatic = env->GetStaticMethodID(Modifier, "isStatic", "(I)Z");
    CHECK_EXCEPTION();

    jmethodID isPublic = env->GetStaticMethodID(Modifier, "isPublic", "(I)Z");
    CHECK_EXCEPTION();

    jobject_wrapper<jobjectArray> methods(env->CallObjectMethod(clazz, getDeclaredMethods), env);
    CHECK_EXCEPTION();

    const jsize numMethods = env->GetArrayLength(methods);
    CHECK_EXCEPTION();

    const auto get_name = [&](const jobject_wrapper<jobject> &method) -> std::string {
        jobject_wrapper<jstring> name(env->CallObjectMethod(method, getName), env);
        CHECK_EXCEPTION();

        return jstring_to_string(name);
    };

    const auto get_return_type = [&](const jobject_wrapper<jobject> &method) -> std::string {
        jobject_wrapper<jobject> type(env->CallObjectMethod(method, getReturnType), env);
        CHECK_EXCEPTION();

        jobject_wrapper<jstring> str(env->CallObjectMethod(type, class_getName), env);
        CHECK_EXCEPTION();

        return java_type_to_jni_type(jstring_to_string(str));
    };

    const auto get_parameter_types = [&](const jobject_wrapper<jobject> &method) -> std::vector<std::string> {
        jobject_wrapper<jobjectArray> types(env->CallObjectMethod(method, getParameterTypes), env);
        CHECK_EXCEPTION();

        const jsize numTypes = env->GetArrayLength(types);
        CHECK_EXCEPTION();

        std::vector<std::string> res;
        res.reserve(numTypes);

        for (jsize i = 0; i < numTypes; i++) {
            jobject_wrapper<jobject> type(env->GetObjectArrayElement(types, i), env);
            CHECK_EXCEPTION();

            jobject_wrapper<jstring> name(env->CallObjectMethod(type, class_getName), env);
            CHECK_EXCEPTION();

            res.push_back(java_type_to_jni_type(jstring_to_string(name)));
        }

        return res;
    };

    const auto get_id = [&](const std::string &name, const std::string &returnType,
                            const std::vector<std::string> &parameterTypes) -> jmethodID {
        jclass javaClass = env->FindClass(string_replace(className, '.', '/').c_str());
        CHECK_EXCEPTION();

        std::stringstream signature;
        signature << '(';
        for (const std::string &param : parameterTypes) {
            signature << param;
        }
        signature << ')' << returnType;

        const std::string sig = signature.str();
        jmethodID id;
        if (onlyStatic) {
            id = env->GetStaticMethodID(javaClass, name.c_str(), sig.c_str());
        } else {
            id = env->GetMethodID(javaClass, name.c_str(), sig.c_str());
        }
        CHECK_EXCEPTION();

        return id;
    };

    std::vector<java_function> res;
    res.reserve(numMethods);

    for (jsize i = 0; i < numMethods; i++) {
        jobject_wrapper<jobject> method(env->GetObjectArrayElement(methods, i), env);
        CHECK_EXCEPTION();

        const jint modifiers = env->CallIntMethod(method, getModifiers);
        CHECK_EXCEPTION();

        const jboolean is_static = env->CallStaticBooleanMethod(Modifier, isStatic, modifiers);
        CHECK_EXCEPTION();

        const jboolean is_public = env->CallStaticBooleanMethod(Modifier, isPublic, modifiers);
        CHECK_EXCEPTION();

        if (((onlyStatic && is_static) || (!onlyStatic && !is_static)) && is_public) {
            const std::string name = get_name(method);
            const std::string returnType = get_return_type(method);
            const std::vector<std::string> parameterTypes = get_parameter_types(method);
            jmethodID id = get_id(name, returnType, parameterTypes);

            res.emplace_back(parameterTypes, returnType, name, id);
        }
    }

    return res;
}

java_class jni_wrapper::getClass(const std::string &className) const {
    const std::vector<java_field> fields = getClassFields(className, false);
    const std::vector<java_field> staticFields = getClassFields(className, true);
    const std::vector<java_function> functions = getClassFunctions(className, false);
    const std::vector<java_function> staticFunctions = getClassFunctions(className, true);
    const std::vector<java_constructor> constructors = getClassConstructors(className);

    return java_class(staticFields, fields, staticFunctions, functions, constructors);
}

java_exception jni_wrapper::getLastException(int line) const {
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
    if (line >= 0) {
        stackFrames.push_back("jni_wrapper.cpp:" + std::to_string(line));
    }

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

    jmethodID getClass = env->GetMethodID(Object, "getClass", "()Ljava/lang/Class;");
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

java_field::java_field(std::string signature, std::string name, jfieldID id) : signature(std::move(signature)),
                                                                               name(std::move(name)), id(id) {}

java_function::java_function(std::vector<std::string> parameterTypes, std::string returnType, std::string functionName,
                             jmethodID method) : parameterTypes(std::move(parameterTypes)),
                                                 returnType(std::move(returnType)),
                                                 functionName(std::move(functionName)), method(method) {}

java_class::java_class(std::vector<java_field> static_fields, std::vector<java_field> fields,
                       std::vector<java_function> static_functions, std::vector<java_function> functions,
                       std::vector<java_constructor> constructors) : static_fields(std::move(static_fields)),
                                                                     fields(std::move(fields)),
                                                                     static_functions(std::move(static_functions)),
                                                                     functions(std::move(functions)),
                                                                     constructors(std::move(constructors)) {}

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