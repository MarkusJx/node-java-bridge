#include <stdexcept>
#include <iostream>
#include <utility>

#include "jvm_lib/jni_wrapper.hpp"
#include "jvm_lib/java_exception.hpp"
#include "util/util.hpp"

#define CHECK_EXCEPTION() if (env->ExceptionCheck()) throwLastException(__LINE__)
#define JVM_CHECK_EXCEPTION(jvm) if (jvm->ExceptionCheck()) jvm.throwLastException(__LINE__)

using namespace jni;
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

java_constructor::java_constructor(jobject object, const jni_wrapper &jni) : jobject_wrapper<jobject>(object, jni),
                                                                             jni(jni), parameterTypes() {
    parameterTypes = getParameterTypes();
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

    jobject_wrapper<jobjectArray> parameters(jni->CallObjectMethod(obj, getParameters), jni);
    JVM_CHECK_EXCEPTION(jni);

    jint numParams = jni->GetArrayLength(parameters);
    JVM_CHECK_EXCEPTION(jni);

    std::vector<std::string> res;
    for (jint i = 0; i < numParams; i++) {
        jobject elem = jni->GetObjectArrayElement(parameters, i);
        JVM_CHECK_EXCEPTION(jni);

        jobject_wrapper<jobject> type(jni->CallObjectMethod(elem, getType), jni);
        JVM_CHECK_EXCEPTION(jni);

        jobject_wrapper<jstring> name(jni->CallObjectMethod(type, getName), jni);
        JVM_CHECK_EXCEPTION(jni);

        res.push_back(jni.jstring_to_string(name));
    }

    return res;
}

jobject_wrapper<jobject> java_constructor::newInstance(const std::vector<jobject_wrapper<jobject>> &args) const {
    jni_wrapper env = jni.attachEnv();
    jclass constructor = env->FindClass("java/lang/reflect/Constructor");
    JVM_CHECK_EXCEPTION(env);

    jmethodID newInstance_m = env->GetMethodID(constructor, "newInstance", "([Ljava/lang/Object;)Ljava/lang/Object;");
    JVM_CHECK_EXCEPTION(env);

    jclass Object = env->FindClass("java/lang/Object");
    JVM_CHECK_EXCEPTION(env);
    jobject_wrapper<jobjectArray> argArr(env->NewObjectArray(static_cast<jsize>(args.size()), Object, nullptr),
                                         jni);
    JVM_CHECK_EXCEPTION(env);

    for (size_t i = 0; i < args.size(); i++) {
        env->SetObjectArrayElement(argArr, static_cast<jsize>(i), args[i]);
        JVM_CHECK_EXCEPTION(env);
    }

    jobject instance = env->CallObjectMethod(obj, newInstance_m, argArr.obj);
    JVM_CHECK_EXCEPTION(env);
    return jobject_wrapper(instance, env);
}

std::string java_constructor::to_string() const {
    jclass constructor = jni->FindClass("java/lang/reflect/Constructor");
    JVM_CHECK_EXCEPTION(jni);

    jmethodID toString = jni->GetMethodID(constructor, "toString", "()Ljava/lang/String;");
    JVM_CHECK_EXCEPTION(jni);

    jobject_wrapper<jstring> string(jni->CallObjectMethod(obj, toString), jni);
    JVM_CHECK_EXCEPTION(jni);

    return jni.jstring_to_string(string);
}

jobject_wrapper<jobject> jni_wrapper::classLoader;

jni_wrapper::jni_wrapper() noexcept: env(), initialized(false), version(0) {}

jni_wrapper::jni_wrapper(jvm_env env, jint version) : env(std::move(env)), initialized(true), version(version) {
    if (!classLoader.ok()) {
        classLoader.assign(getSystemClassLoader());
    }
}

jvm_wrapper::jvm_wrapper() noexcept: jni_wrapper(), library() {}

jvm_wrapper::jvm_wrapper(const std::string &jvmPath, jint version) : jni_wrapper() {
    this->version = version;
    initialized = true;
    library = shared_library(jvmPath);
    JNI_CreateJavaVM = library.getFunction<JNI_CreateJavaVM_t>("JNI_CreateJavaVM");

    JavaVM *jvm = nullptr;
    JNIEnv *environment = nullptr;

    JavaVMInitArgs vm_args;

    vm_args.version = version;
    vm_args.nOptions = 0;
    vm_args.options = nullptr;
    vm_args.ignoreUnrecognized = false;

    jint create_code = JNI_CreateJavaVM(&jvm, (void **) &environment, &vm_args);
    if (create_code != JNI_OK) {
        throw std::runtime_error("JNI_CreateJavaVM failed: " + util::jni_error_to_string(create_code));
    } else {
        env = jvm_env(std::make_shared<jvm_jvm>(jvm), environment, version);
    }

    // The start class loader is the system default one.
    // It may evolve to a more potent one during the
    // execution of the program, just like a pokèmon.
    // Nah, I dunno either, never played those games.
    if (!classLoader) {
        classLoader.assign(getSystemClassLoader());
    }
}

jni_wrapper jni_wrapper::attachEnv() const {
    JNIEnv *environment = nullptr;
    jint create_result = env.jvm->GetEnv(reinterpret_cast<void **>(&environment), version);

    if (create_result == JNI_EDETACHED) {
        create_result = env.jvm->AttachCurrentThread(reinterpret_cast<void **>(&environment), nullptr);
        if (create_result == JNI_OK) {
            return jni_wrapper(jvm_env(env.jvm, environment, version, true), version);
        } else {
            throw std::runtime_error("AttachCurrentThread failed: " + util::jni_error_to_string(create_result));
        }
    } else if (create_result == JNI_OK) {
        return *this;
    } else {
        throw std::runtime_error("GetEnv failed: " + util::jni_error_to_string(create_result));
    }
}

void jni_wrapper::checkForError() const {
    CHECK_EXCEPTION();
}

jobject_wrapper<jstring> jni_wrapper::string_to_jstring(const std::string &str) const {
    auto res = jobject_wrapper<jstring>(env->NewStringUTF(str.c_str()), env);
    if (env->ExceptionCheck()) {
        throwLastException(__LINE__);
    } else if (res == nullptr) {
        throw std::runtime_error("Could not get the string");
    }

    return res;
}

std::string jni_wrapper::jstring_to_string(jstring str, bool convertErrors) const {
    const char *chars = env->GetStringUTFChars(str, nullptr);
    if (env->ExceptionCheck()) {
        if (convertErrors) {
            throwLastException(__LINE__);
        } else {
            throw std::runtime_error("Could not get the string characters");
        }
    } else if (chars == nullptr) {
        throw std::runtime_error("Could not get the string characters");
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
    /*
     * Java code:
     *
     * Class<?> clazz = Class.forName(className, true, this.classLoader);
     * return clazz;
     */
    jclass Class = getJavaLangClass();

    jmethodID forName = env->GetStaticMethodID(Class, "forName", "(Ljava/lang/String;ZLjava/lang/ClassLoader;)Ljava/lang/Class;");
    CHECK_EXCEPTION();

    jobject clazz = env->CallStaticObjectMethod(Class, forName, string_to_jstring(className).obj, true, classLoader.obj);
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

    jmethodID isFinal = env->GetStaticMethodID(Modifier, "isFinal", "(I)Z");
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

        return util::java_type_to_jni_type(jstring_to_string(name));
    };

    const auto getFieldName = [&](const jobject_wrapper<jobject> &field) -> std::string {
        jobject_wrapper<jstring> name(env->CallObjectMethod(field, field_getName), env);
        CHECK_EXCEPTION();

        return jstring_to_string(name);
    };

    const auto getFieldId = [&](const jobject_wrapper<jobject> &field, const std::string &fieldName,
                                const std::string &signature) -> jfieldID {
        jclass javaClass = getJClass(className);
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

        jboolean is_final = env->CallStaticBooleanMethod(Modifier, isFinal, modifiers);
        CHECK_EXCEPTION();

        if (((onlyStatic && is_static) || (!onlyStatic && !is_static)) && is_public) {
            const std::string signature = getFieldSignature(field);
            const std::string name = getFieldName(field);
            jfieldID id = getFieldId(field, name, signature);
            res.emplace_back(signature, name, id, is_static, is_final, *this);
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

        return util::java_type_to_jni_type(jstring_to_string(str));
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

            res.push_back(util::java_type_to_jni_type(jstring_to_string(name)));
        }

        return res;
    };

    const auto get_id = [&](const std::string &name, const std::string &returnType,
                            const std::vector<std::string> &parameterTypes) -> jmethodID {
        jclass javaClass = getJClass(className);
        CHECK_EXCEPTION();

        std::string signature;
        signature += '(';
        for (const std::string &param : parameterTypes) {
            signature += param;
        }
        signature += ')';
        signature += returnType;

        jmethodID id;
        if (onlyStatic) {
            id = env->GetStaticMethodID(javaClass, name.c_str(), signature.c_str());
        } else {
            id = env->GetMethodID(javaClass, name.c_str(), signature.c_str());
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

            res.emplace_back(parameterTypes, returnType, name, id, is_static, *this);
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
    jclass clazz = getJClass(className);

    return java_class(staticFields, fields, staticFunctions, functions, constructors, clazz);
}

jclass jni_wrapper::getJClass(const std::string &className) const {
    /*
     * Java code:
     *
     * Class<?> clazz = Class.forName(className, true, this.classLoader);
     * return clazz;
     */
    jclass Class = getJavaLangClass();

    jmethodID forName = env->GetStaticMethodID(Class, "forName", "(Ljava/lang/String;ZLjava/lang/ClassLoader;)Ljava/lang/Class;");
    CHECK_EXCEPTION();

    jobject clazz = env->CallStaticObjectMethod(Class, forName, string_to_jstring(className).obj, true, classLoader.obj);
    CHECK_EXCEPTION();

    return reinterpret_cast<jclass>(clazz);
}

void jni_wrapper::throwLastException(int line) const {
    if (!env->ExceptionCheck()) {
        throw std::runtime_error("No exception occurred");
    }

    auto throwable = jobject_wrapper(env->ExceptionOccurred(), env);
    env->ExceptionClear();

    jclass throwable_class = env->FindClass("java/lang/Throwable");
    if (env->ExceptionCheck()) throw std::runtime_error("Could not get java.lang.Throwable");

    jmethodID throwable_getCause = env->GetMethodID(throwable_class, "getCause", "()Ljava/lang/Throwable;");
    if (env->ExceptionCheck()) throw std::runtime_error("Could not get java.lang.Throwable#getCause");

    jmethodID throwable_getStackTrace = env->GetMethodID(throwable_class, "getStackTrace",
                                                         "()[Ljava/lang/StackTraceElement;");
    if (env->ExceptionCheck()) throw std::runtime_error("Could not get java.lang.Throwable#getStackTrace");

    jmethodID throwable_toString = env->GetMethodID(throwable_class, "toString", "()Ljava/lang/String;");
    if (env->ExceptionCheck()) throw std::runtime_error("Could not get java.lang.Throwable#toString");

    jclass stacktrace_element_class = env->FindClass("java/lang/StackTraceElement");
    if (env->ExceptionCheck()) throw std::runtime_error("Could not get java.lang.StackTraceElement");
    jmethodID stacktrace_toString = env->GetMethodID(stacktrace_element_class, "toString", "()Ljava/lang/String;");
    if (env->ExceptionCheck()) throw std::runtime_error("Could not get java.lang.StackTraceElement#toString");

    auto frames = jobject_wrapper<jobjectArray>(env->CallObjectMethod(throwable, throwable_getStackTrace), env);
    if (env->ExceptionCheck()) throw std::runtime_error("Could not get the stack trace");
    jsize numFrames = env->GetArrayLength(frames);
    if (env->ExceptionCheck()) throw std::runtime_error("Could not get the stack trace length");

    std::vector<std::string> causes;
    std::vector<std::string> stackFrames;
    if (line >= 0) {
        stackFrames.push_back("jni_wrapper.cpp:" + std::to_string(line));
    }

    while (frames != nullptr && throwable.ok()) {
        jobject_wrapper<jstring> throwable_string(env->CallObjectMethod(throwable, throwable_toString), env);
        if (env->ExceptionCheck()) throw std::runtime_error("Could not get the stack trace length");
        causes.push_back(jstring_to_string(throwable_string, false));

        if (numFrames > 0) {
            for (jsize i = 0; i < numFrames; i++) {
                jobject_wrapper frame(env->GetObjectArrayElement(frames, i), env);
                if (env->ExceptionCheck()) throw std::runtime_error("Could not get a stack trace element");
                jobject_wrapper<jstring> frame_string(env->CallObjectMethod(frame, stacktrace_toString), env);
                if (env->ExceptionCheck())
                    throw std::runtime_error("Could not convert a  stack trace element to string");

                stackFrames.push_back(jstring_to_string(frame_string, false));
            }
        }

        throwable.assign(env->CallObjectMethod(throwable, throwable_getCause), env);
        if (env->ExceptionCheck()) throw std::runtime_error("Could not get the throwable cause");

        if (throwable != nullptr) {
            frames.assign(env->CallObjectMethod(throwable, throwable_getStackTrace), env);
            if (env->ExceptionCheck()) throw std::runtime_error("Could not get the frames");
            numFrames = env->GetArrayLength(frames);
            if (env->ExceptionCheck()) throw std::runtime_error("Could not get the number of frames");
        }
    }

    throw java_exception(causes, stackFrames);
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

void jni_wrapper::appendClasspath(const std::string& path) {
    // This. was. torture.
    /*
     * This whole thing is based on this: https://stackoverflow.com/a/60775
     * Java code:
     *
     * File toLoad = new File(path);
     * URI uri = toLoad.toURI();
     * URL url = uri.toURL();
     * URL[] urls = new URL[]{url};
     * URLClassLoader newClassLoader = new URLClassLoader(urls, this.classLoader);
     * this.classLoader = newClassLoader;
     */
    jclass File = env->FindClass("java/io/File");
    CHECK_EXCEPTION();
    jmethodID FileConstructor = env->GetMethodID(File, "<init>", "(Ljava/lang/String;)V");
    CHECK_EXCEPTION();
    jmethodID toURI = env->GetMethodID(File, "toURI", "()Ljava/net/URI;");
    CHECK_EXCEPTION();
    jclass URI = env->FindClass("java/net/URI");
    CHECK_EXCEPTION();
    jmethodID toURL = env->GetMethodID(URI, "toURL", "()Ljava/net/URL;");
    CHECK_EXCEPTION();

    auto j_path = string_to_jstring(path);
    jobject_wrapper<jobject> file(env->NewObject(File, FileConstructor, j_path.obj), env);
    CHECK_EXCEPTION();

    jobject_wrapper<jobject> uri(env->CallObjectMethod(file, toURI), env);
    CHECK_EXCEPTION();
    jobject_wrapper<jobject> url(env->CallObjectMethod(uri, toURL), env);
    CHECK_EXCEPTION();

    jclass URLClassLoader = env->FindClass("java/net/URLClassLoader");
    CHECK_EXCEPTION();
    jmethodID classLoaderInit = env->GetMethodID(URLClassLoader, "<init>", "([Ljava/net/URL;Ljava/lang/ClassLoader;)V");
    CHECK_EXCEPTION();

    jclass URL = env->FindClass("java/net/URL");
    CHECK_EXCEPTION();
    jobject_wrapper<jobjectArray> urls(env->NewObjectArray(1, URL, url), env);
    CHECK_EXCEPTION();

    jobject_wrapper<jobject> newClassLoader(env->NewObject(URLClassLoader, classLoaderInit, urls.obj, classLoader.obj), env);
    CHECK_EXCEPTION();

    classLoader.assign(newClassLoader);
}

bool jni_wrapper::class_is_assignable(const std::string &sub, const std::string &sup) const {
    if (util::hasEnding(sub, "[]") || util::hasEnding(sup, "[]")) return false;
    if (sub == sup) return true;

    jclass clazz1 = getJClass(sub);
    jclass clazz2 = getJClass(sup);

    return env->IsAssignableFrom(clazz1, clazz2);
}

jobject_wrapper<jobject> jni_wrapper::getSystemClassLoader() {
    /*
     * Java code:
     *
     * return ClassLoader.getSystemClassLoader();
     */
    jclass classLoaderCls = env->FindClass("java/lang/ClassLoader");
    CHECK_EXCEPTION();
    jmethodID getSystemClassLoaderMethod = env->GetStaticMethodID(classLoaderCls, "getSystemClassLoader", "()Ljava/lang/ClassLoader;");
    CHECK_EXCEPTION();
    jobject_wrapper res(env->CallStaticObjectMethod(classLoaderCls, getSystemClassLoaderMethod), env);
    CHECK_EXCEPTION();

    return res;
}

#define JOBJECT_TO_TEMPLATE(className, valueFunc, signature, method) \
if (!obj) throw std::runtime_error("The object was null");\
jclass cls = env->GetObjectClass(obj);\
CHECK_EXCEPTION();\
\
if(env->IsInstanceOf(obj, env->FindClass(className)) ) {\
    CHECK_EXCEPTION();\
    \
    jmethodID vFunction = env->GetMethodID(cls, valueFunc, signature);\
    CHECK_EXCEPTION();\
    auto result = env->method(obj, vFunction);\
    CHECK_EXCEPTION();\
    \
    return result;\
} else \
    throw std::runtime_error("Mismatched types: The passed value is not of type " className)

jint jni_wrapper::jobject_to_jint(jobject obj) const {
    JOBJECT_TO_TEMPLATE("java/lang/Integer", "intValue", "()I", CallIntMethod);
}

jboolean jni_wrapper::jobject_to_jboolean(jobject obj) const {
    JOBJECT_TO_TEMPLATE("java/lang/Boolean", "booleanValue", "()Z", CallBooleanMethod);
}

jbyte jni_wrapper::jobject_to_jbyte(jobject obj) const {
    JOBJECT_TO_TEMPLATE("java/lang/Byte", "byteValue", "()B", CallByteMethod);
}

jchar jni_wrapper::jobject_to_jchar(jobject obj) const {
    JOBJECT_TO_TEMPLATE("java/lang/Character", "charValue", "()C", CallCharMethod);
}

jshort jni_wrapper::jobject_to_jshort(jobject obj) const {
    JOBJECT_TO_TEMPLATE("java/lang/Short", "shortValue", "()S", CallShortMethod);
}

jlong jni_wrapper::jobject_to_jlong(jobject obj) const {
    JOBJECT_TO_TEMPLATE("java/lang/Long", "longValue", "()J", CallLongMethod);
}

jfloat jni_wrapper::jobject_to_jfloat(jobject obj) const {
    JOBJECT_TO_TEMPLATE("java/lang/Float", "floatValue", "()F", CallFloatMethod);
}

jdouble jni_wrapper::jobject_to_jdouble(jobject obj) const {
    JOBJECT_TO_TEMPLATE("java/lang/Double", "doubleValue", "()D", CallDoubleMethod);
}

#undef JOBJECT_TO_TEMPLATE

#define CREATE_JOBJECT(className, signature) \
    jclass clazz = env->FindClass(className);\
    CHECK_EXCEPTION();\
    jmethodID constructor = env->GetMethodID(clazz, "<init>", signature); \
    CHECK_EXCEPTION();\
    jobject obj = env->NewObject(clazz, constructor, e);\
    CHECK_EXCEPTION();\
    return jobject_wrapper<jobject>(obj, env)

jobject_wrapper<jobject> jni_wrapper::create_jint(jint e) const {
    CREATE_JOBJECT("java/lang/Integer", "(I)V");
}

jobject_wrapper<jobject> jni_wrapper::create_jshort(jshort e) const {
    CREATE_JOBJECT("java/lang/Short", "(S)V");
}

jobject_wrapper<jobject> jni_wrapper::create_jdouble(jdouble e) const {
    CREATE_JOBJECT("java/lang/Double", "(D)V");
}

jobject_wrapper<jobject> jni_wrapper::create_jfloat(jfloat e) const {
    CREATE_JOBJECT("java/lang/Float", "(F)V");
}

jobject_wrapper<jobject> jni_wrapper::create_jlong(jlong e) const {
    CREATE_JOBJECT("java/lang/Long", "(J)V");
}

jobject_wrapper<jobject> jni_wrapper::create_jbyte(jbyte e) const {
    CREATE_JOBJECT("java/lang/Byte", "(B)V");
}

jobject_wrapper<jobject> jni_wrapper::create_jchar(jchar e) const {
    CREATE_JOBJECT("java/lang/Character", "(C)V");
}

jobject_wrapper<jobject> jni_wrapper::create_jboolean(jboolean e) const {
    CREATE_JOBJECT("java/lang/Boolean", "(Z)V");
}

#undef CREATE_JOBJECT

JNIEnv *jni_wrapper::operator->() const {
    return env.operator->();
}

jvm_env &jni_wrapper::getEnv() {
    return env;
}

jni_wrapper::operator jvm_env() const {
    return env;
}

jni_wrapper::operator bool() const {
    return initialized;
}

java_field::java_field(std::string signature, std::string name, jfieldID id, bool isStatic, bool isFinal,
                       jni_wrapper env) : signature(std::move(signature)), name(std::move(name)), id(id),
                                          isStatic(isStatic), isFinal(isFinal), env(std::move(env)) {}

jobject_wrapper<jobject> java_field::get(jobject classInstance) const {
    if (isStatic) {
        throw std::runtime_error("Tried to access a static field through a class instance");
    }

    jobject_wrapper<jobject> data(env->GetObjectField(classInstance, id), env);
    JVM_CHECK_EXCEPTION(env);

    return data;
}

jobject_wrapper<jobject> java_field::getStatic(jclass clazz) const {
    if (!isStatic) {
        throw std::runtime_error("Tried to access a non-static field through a static accessor");
    }

    jobject_wrapper<jobject> data(env->GetStaticObjectField(clazz, id), env);
    JVM_CHECK_EXCEPTION(env);

    return data;
}

void java_field::set(jobject classInstance, jobject data) const {
    if (isStatic) {
        throw std::runtime_error("Tried to access a static field through a class instance");
    }

    env->SetObjectField(classInstance, id, data);
    JVM_CHECK_EXCEPTION(env);
}

void java_field::setStatic(jclass clazz, jobject data) const {
    if (!isStatic) {
        throw std::runtime_error("Tried to access a non-static field through a static accessor");
    }

    env->SetStaticObjectField(clazz, id, data);
    JVM_CHECK_EXCEPTION(env);
}

java_function::java_function(std::vector<std::string> parameterTypes, std::string returnType, std::string functionName,
                             jmethodID method, bool isStatic, jni_wrapper env) : parameterTypes(
        std::move(parameterTypes)),
                                                                                 returnType(std::move(returnType)),
                                                                                 name(std::move(functionName)),
                                                                                 method(method),
                                                                                 isStatic(isStatic),
                                                                                 env(std::move(env)) {}

std::string java_function::to_string() const {
    std::stringstream ss;
    if (isStatic) {
        ss << "static ";
    }

    ss << util::make_java_name_readable(returnType) << ' ' << name << '(';
    for (size_t i = 0; i < parameterTypes.size(); i++) {
        if (i > 0) {
            ss << ", ";
        }
        ss << util::make_java_name_readable(parameterTypes[i]);
    }

    ss << ')';
    return ss.str();
}

java_class::java_class() : clazz(nullptr) {}

java_class::java_class(const std::vector<java_field> &static_fields, const std::vector<java_field> &fields,
                       const std::vector<java_function> &static_functions, const std::vector<java_function> &functions,
                       std::vector<java_constructor> constructors, jclass clazz) : static_fields(
        util::map_vector_values_to_map(static_fields)), fields(util::map_vector_values_to_map(fields)),
                                                                                   static_functions(
                                                                                           util::map_vector_to_map(
                                                                                                   static_functions)),
                                                                                   functions(util::map_vector_to_map(
                                                                                           functions)), constructors(
                std::move(constructors)), clazz(clazz) {}