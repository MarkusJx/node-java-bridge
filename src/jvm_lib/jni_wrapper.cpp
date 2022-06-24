#include <stdexcept>
#include <utility>
#include <sstream>

#include "jvm_lib/jni_wrapper.hpp"
#include "jvm_lib/java_exception.hpp"
#include "util/util.hpp"
#include "node_classes/jvm_container.hpp"

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

java_constructor::java_constructor(jobject object, const jni_wrapper &jni) : jobject_wrapper<jobject>(object,
                                                                                                      jni.env),
                                                                             parameterTypes() {
    parameterTypes = getParameterTypes();
}

std::vector<java_type> java_constructor::getParameterTypes() const {
    const auto jni = node_classes::jvm_container::attachJvm();
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

    std::vector<java_type> res;
    for (jint i = 0; i < numParams; i++) {
        jobject elem = jni->GetObjectArrayElement(parameters, i);
        JVM_CHECK_EXCEPTION(jni);

        jobject_wrapper<jobject> type(jni->CallObjectMethod(elem, getType), jni.env);
        JVM_CHECK_EXCEPTION(jni);

        jobject_wrapper<jstring> name(jni->CallObjectMethod(type, getName), jni.env);
        JVM_CHECK_EXCEPTION(jni);

        res.push_back(java_type::to_java_type(jni.jstring_to_string(name)));
    }

    return res;
}

jobject_wrapper<jobject> java_constructor::newInstance(const std::vector<jobject_wrapper<jobject>> &args) const {
    jni_wrapper env = node_classes::jvm_container::attachJvm();
    jclass constructor = env->FindClass("java/lang/reflect/Constructor");
    JVM_CHECK_EXCEPTION(env);

    jmethodID newInstance_m = env->GetMethodID(constructor, "newInstance", "([Ljava/lang/Object;)Ljava/lang/Object;");
    JVM_CHECK_EXCEPTION(env);

    jclass Object = env->FindClass("java/lang/Object");
    JVM_CHECK_EXCEPTION(env);
    jobject_wrapper<jobjectArray> argArr(env->NewObjectArray(static_cast<jsize>(args.size()), Object, nullptr),
                                         env.env);
    JVM_CHECK_EXCEPTION(env);

    for (size_t i = 0; i < args.size(); i++) {
        env->SetObjectArrayElement(argArr, static_cast<jsize>(i), args[i]);
        JVM_CHECK_EXCEPTION(env);
    }

    jobject instance = env->CallObjectMethod(obj, newInstance_m, argArr.obj);
    JVM_CHECK_EXCEPTION(env);

    return {instance, env.env};
}

std::string java_constructor::to_string() const {
    const auto jni = node_classes::jvm_container::attachJvm();
    jclass constructor = jni->FindClass("java/lang/reflect/Constructor");
    JVM_CHECK_EXCEPTION(jni);

    jmethodID toString = jni->GetMethodID(constructor, "toString", "()Ljava/lang/String;");
    JVM_CHECK_EXCEPTION(jni);

    jobject_wrapper<jstring> string(jni->CallObjectMethod(obj, toString), jni.env);
    JVM_CHECK_EXCEPTION(jni);

    return jni.jstring_to_string(string);
}

jobject_wrapper<jobject> jni_wrapper::classLoader;

jni_wrapper::jni_wrapper() noexcept: initialized(false) {}

jni_wrapper::jni_wrapper(jvm_env &&env) : env(std::move(env)), initialized(true) {
    if (!classLoader.ok()) {
        classLoader.assign(getSystemClassLoader());
    }
}

jvm_wrapper::jvm_wrapper() noexcept: jni_wrapper() {}

jvm_wrapper jvm_wrapper::create_jvm_wrapper(const std::string &jvmPath, jint version) {
    JavaVM *jvm = nullptr;
    JNIEnv *environment = nullptr;

    library = shared_library(jvmPath);
    std::function<JNI_CreateJavaVM_t> JNI_CreateJavaVM = library.getFunction<JNI_CreateJavaVM_t>("JNI_CreateJavaVM");

    JavaVMInitArgs vm_args;

    vm_args.version = version;
    vm_args.nOptions = 0;
    vm_args.options = nullptr;
    vm_args.ignoreUnrecognized = false;

    jint create_code = JNI_CreateJavaVM(&jvm, (void **) &environment, &vm_args);
    if (create_code != JNI_OK) {
        throw std::runtime_error("JNI_CreateJavaVM failed: " + util::jni_error_to_string(create_code));
    } else {
        return {jvm_env(std::make_shared<jvm_jvm>(jvm), environment, version), std::move(JNI_CreateJavaVM)};
    }
}

jvm_wrapper::jvm_wrapper(jvm_env &&env, std::function<jni_types::JNI_CreateJavaVM_t> &&createVm) : jni_wrapper(
        std::move(env)), JNI_CreateJavaVM(std::move(createVm)) {
    // The start class loader is the system default one.
    // It may evolve to a more potent one during the
    // execution of the program, just like a pokèmon.
    // Nah, I dunno either, never played those games.
    if (!classLoader) {
        classLoader.assign(getSystemClassLoader());
    }
}

shared_library jvm_wrapper::library;

jni_wrapper jni_wrapper::attachEnv() const {
    return jni_wrapper(env.attach_env());
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

    jmethodID forName = env->GetStaticMethodID(Class, "forName",
                                               "(Ljava/lang/String;ZLjava/lang/ClassLoader;)Ljava/lang/Class;");
    CHECK_EXCEPTION();

    jobject clazz = env->CallStaticObjectMethod(Class, forName, string_to_jstring(className).obj, true,
                                                classLoader.obj);
    CHECK_EXCEPTION();

    return {clazz, env};
}

std::vector<java_constructor> jni_wrapper::getClassConstructors(const std::string &className) const {
    /*
     * Java code:
     *
     * Class<?> clazz = Class.forName(className);
     * Constructor[] constructors = clazz.getConstructors();
     */
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
    /*
     * Java code:
     *
     * Class<?> clazz = Class.forName(className);
     * Field[] fields = clazz.getDeclaredFields();
     *
     * for (int i = 0; i < fields.length; i++) {
     *      String signature = fields[i].getType().getName();
     *      String fieldName = fields[i].getName();
     *
     *      int modifiers = fields[i].getModifiers();
     *      boolean is_static = Modifier.isStatic(modifiers);
     *      boolean is_public = Modifier.isPublic(modifiers);
     *      boolean is_final = Modifier.isFinal(modifiers);
     *
     *      // Get the fields id and store the information
     * }
     */
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

    // Get the fields signature as a string
    const auto getFieldSignature = [&](const jobject_wrapper<jobject> &field) -> std::string {
        jobject_wrapper<jobject> type(env->CallObjectMethod(field, field_getType), env);
        CHECK_EXCEPTION();

        jobject_wrapper<jstring> name(env->CallObjectMethod(type, class_getName), env);
        CHECK_EXCEPTION();

        return util::make_java_name_readable(jstring_to_string(name));
    };

    // Get the fields name
    const auto getFieldName = [&](const jobject_wrapper<jobject> &field) -> std::string {
        jobject_wrapper<jstring> name(env->CallObjectMethod(field, field_getName), env);
        CHECK_EXCEPTION();

        return jstring_to_string(name);
    };

    // Get the field id
    const auto getFieldId = [&](const jobject_wrapper<jobject> &field, const std::string &fieldName,
                                const std::string &sig) -> jfieldID {
        jclass javaClass = getJClass(className);
        CHECK_EXCEPTION();

        const std::string signature = util::java_type_to_jni_type(sig);

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
            res.emplace_back(signature, name, id, is_static, is_final);
        }
    }

    return res;
}

std::vector<java_function> jni_wrapper::getClassFunctions(const std::string &className, bool onlyStatic) const {
    /*
     * Java code:
     *
     * Class<?> clazz = Class.forName();
     * Method[] methods = class.getDeclaredMethods();
     *
     * for (int i = 0; i < methods.length; i++) {
     *      String name = methods[i].getName();
     *      String returnType = methods[i].getReturnType().getName();
     *
     *      Class[] types = methods[i].getParameterTypes();
     *      List<String> parameterTypes = new ArrayList<>(types.length);
     *      for (int i = 0; i < types.length; i++) {
     *          types.add(types[i].getName());
     *      }
     *
     *      int modifiers = methods[i].getModifiers();
     *      boolean is_static = Modifier.isStatic(modifiers);
     *      boolean is_public = Modifier.isPublic(modifiers);
     *
     *      // Get the methods id and store the information
     * }
     */
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

    // Get the function name
    const auto get_name = [&](const jobject_wrapper<jobject> &method) -> std::string {
        jobject_wrapper<jstring> name(env->CallObjectMethod(method, getName), env);
        CHECK_EXCEPTION();

        return jstring_to_string(name);
    };

    // Get the functions return type
    const auto get_return_type = [&](const jobject_wrapper<jobject> &method) -> java_type {
        jobject_wrapper<jobject> type(env->CallObjectMethod(method, getReturnType), env);
        CHECK_EXCEPTION();

        jobject_wrapper<jstring> str(env->CallObjectMethod(type, class_getName), env);
        CHECK_EXCEPTION();

        return java_type::to_java_type(jstring_to_string(str));
    };

    // Get the functions parameter type
    const auto get_parameter_types = [&](const jobject_wrapper<jobject> &method) -> std::vector<java_type> {
        jobject_wrapper<jobjectArray> types(env->CallObjectMethod(method, getParameterTypes), env);
        CHECK_EXCEPTION();

        const jsize numTypes = env->GetArrayLength(types);
        CHECK_EXCEPTION();

        std::vector<java_type> res;
        res.reserve(numTypes);

        for (jsize i = 0; i < numTypes; i++) {
            jobject_wrapper<jobject> type(env->GetObjectArrayElement(types, i), env);
            CHECK_EXCEPTION();

            jobject_wrapper<jstring> name(env->CallObjectMethod(type, class_getName), env);
            CHECK_EXCEPTION();

            res.push_back(java_type::to_java_type(jstring_to_string(name)));
        }

        return res;
    };

    // Get the functions id
    const auto get_id = [&](const std::string &name, const java_type &returnType,
                            const std::vector<java_type> &parameterTypes) -> jmethodID {
        jclass javaClass = getJClass(className);
        CHECK_EXCEPTION();

        std::string signature;
        signature += '(';
        for (const java_type &param: parameterTypes) {
            signature += util::java_type_to_jni_type(param.signature);
        }
        signature += ')';
        signature += util::java_type_to_jni_type(returnType.signature);

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
            const java_type returnType = get_return_type(method);
            const std::vector<java_type> parameterTypes = get_parameter_types(method);
            jmethodID id = get_id(name, returnType, parameterTypes);

            res.emplace_back(parameterTypes, returnType, name, id, is_static);
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
    jobject_wrapper<jclass> clazz(getJClass(className), env);

    return {staticFields, fields, staticFunctions, functions, constructors, clazz};
}

jclass jni_wrapper::getJClass(const std::string &className) const {
    /*
     * Java code:
     *
     * Class<?> clazz = Class.forName(className, true, this.classLoader);
     * return clazz;
     */
    jclass Class = getJavaLangClass();

    jmethodID forName = env->GetStaticMethodID(Class, "forName",
                                               "(Ljava/lang/String;ZLjava/lang/ClassLoader;)Ljava/lang/Class;");
    CHECK_EXCEPTION();

    jobject clazz = env->CallStaticObjectMethod(Class, forName, string_to_jstring(className).obj, true,
                                                classLoader.obj);
    CHECK_EXCEPTION();

    return reinterpret_cast<jclass>(clazz);
}

void jni_wrapper::throwLastException(int line) const {
    if (!env->ExceptionCheck()) {
        throw std::runtime_error("No exception occurred");
    } else {
        env->ExceptionDescribe();
        env->ExceptionClear();
        throw std::runtime_error("An exception occurred");
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
                    throw std::runtime_error("Could not convert a stack trace element to string");

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

std::string jni_wrapper::getObjectClassName(jobject obj) const {
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

void jni_wrapper::appendClasspath(const std::string &path) {
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

    jobject_wrapper<jobject> newClassLoader(env->NewObject(URLClassLoader, classLoaderInit, urls.obj, classLoader.obj),
                                            env);
    CHECK_EXCEPTION();

    classLoader.assign(newClassLoader);
}

void jni_wrapper::appendClasspath(const std::vector<std::string> &paths) {
    /*
     * This whole thing is based on this: https://stackoverflow.com/a/60775
     * Java code:
     *
     * URL[] urls = new URL[paths.size()];
     * for (int i = 0; i < paths.size(); i++) {
     *      File toLoad = new File(paths[i]);
     *      URI uri = toLoad.toURI();
     *      URL url = uri.toURL();
     *      urls[i] = url;
     * }
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

    jclass URL = env->FindClass("java/net/URL");
    CHECK_EXCEPTION();
    jobject_wrapper<jobjectArray> urls(env->NewObjectArray((jsize) paths.size(), URL, nullptr), env);
    CHECK_EXCEPTION();

    for (size_t i = 0; i < paths.size(); i++) {
        auto j_path = string_to_jstring(paths[i]);
        jobject_wrapper<jobject> file(env->NewObject(File, FileConstructor, j_path.obj), env);
        CHECK_EXCEPTION();

        jobject_wrapper<jobject> uri(env->CallObjectMethod(file, toURI), env);
        CHECK_EXCEPTION();
        jobject_wrapper<jobject> url(env->CallObjectMethod(uri, toURL), env);
        CHECK_EXCEPTION();

        env->SetObjectArrayElement(urls, (jsize) i, url);
    }

    jclass URLClassLoader = env->FindClass("java/net/URLClassLoader");
    CHECK_EXCEPTION();
    jmethodID classLoaderInit = env->GetMethodID(URLClassLoader, "<init>", "([Ljava/net/URL;Ljava/lang/ClassLoader;)V");
    CHECK_EXCEPTION();

    jobject_wrapper<jobject> newClassLoader(env->NewObject(URLClassLoader, classLoaderInit, urls.obj, classLoader.obj),
                                            env);
    CHECK_EXCEPTION();

    classLoader.assign(newClassLoader);
}

bool jni_wrapper::class_is_assignable(const std::string &sub, const std::string &sup) const {
    if (util::hasEnding(sub, "[]") || util::hasEnding(sup, "[]")) return false;
    if (sub == sup) return true;
    if (util::isPrimitive(sub) || util::isPrimitive(sup)) return false;

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
    jmethodID getSystemClassLoaderMethod = env->GetStaticMethodID(classLoaderCls, "getSystemClassLoader",
                                                                  "()Ljava/lang/ClassLoader;");
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

const jobject_wrapper<jobject> &jni_wrapper::getClassloader() {
    return classLoader;
}

jni_wrapper::operator bool() const {
    return initialized;
}

java_field::java_field(const std::string &signature, std::string name, jfieldID id, bool isStatic, bool isFinal)
        : signature(java_type::to_java_type(signature)), name(std::move(name)), id(id), isStatic(isStatic),
          isFinal(isFinal) {}

jvalue java_field::get(jobject classInstance, jobject_wrapper<jobject> &data) const {
    if (isStatic) {
        throw std::runtime_error("Tried to access a static field through a class instance");
    }

    if (classInstance == nullptr) {
        throw std::runtime_error(__FILE__ ":" + std::to_string(__LINE__) + " classInstance was nullptr");
    }

    jvalue val;
    const auto jvm = node_classes::jvm_container::attachJvm();
    if (signature.isInt()) {
        // Value is an integer
        val.i = jvm->GetIntField(classInstance, id);
    } else if (signature.isBool()) {
        // Value is a boolean
        val.z = jvm->GetBooleanField(classInstance, id);
    } else if (signature.isByte()) {
        // Value is a byte
        val.b = jvm->GetByteField(classInstance, id);
    } else if (signature.isChar()) {
        // Value is a char
        val.c = jvm->GetCharField(classInstance, id);
    } else if (signature.isShort()) {
        // Value is a short
        val.s = jvm->GetShortField(classInstance, id);
    } else if (signature.isLong()) {
        // Value is a long
        val.j = jvm->GetLongField(classInstance, id);
    } else if (signature.isFloat()) {
        // Value is a float
        val.f = jvm->GetFloatField(classInstance, id);
    } else if (signature.isDouble()) {
        // Value is a double
        val.d = jvm->GetDoubleField(classInstance, id);
    } else {
        data = jobject_wrapper<jobject>(jvm->GetObjectField(classInstance, id), jvm.env);
        val.l = data.obj;
    }
    JVM_CHECK_EXCEPTION(jvm);

    return val;
}

jvalue java_field::getStatic(jclass clazz, jobject_wrapper<jobject> &data) const {
    if (!isStatic) {
        throw std::runtime_error("Tried to access a non-static field through a static accessor");
    }

    jvalue val;
    const auto jvm = node_classes::jvm_container::attachJvm();
    if (signature.isInt()) {
        // Value is an integer
        val.i = jvm->GetStaticIntField(clazz, id);
    } else if (signature.isBool()) {
        // Value is a boolean
        val.z = jvm->GetStaticBooleanField(clazz, id);
    } else if (signature.isByte()) {
        // Value is a byte
        val.b = jvm->GetStaticByteField(clazz, id);
    } else if (signature.isChar()) {
        // Value is a char
        val.c = jvm->GetStaticCharField(clazz, id);
    } else if (signature.isShort()) {
        // Value is a short
        val.s = jvm->GetStaticShortField(clazz, id);
    } else if (signature.isLong()) {
        // Value is a long
        val.j = jvm->GetStaticLongField(clazz, id);
    } else if (signature.isFloat()) {
        // Value is a float
        val.f = jvm->GetStaticFloatField(clazz, id);
    } else if (signature.isDouble()) {
        // Value is a double
        val.d = jvm->GetStaticDoubleField(clazz, id);
    } else {
        data = jobject_wrapper<jobject>(jvm->GetStaticObjectField(clazz, id), jvm.env);
        val.l = data.obj;
    }
    JVM_CHECK_EXCEPTION(jvm);

    return val;
}

void java_field::set(jobject classInstance, jvalue data) const {
    if (isStatic) {
        throw std::runtime_error("Tried to access a static field through a class instance");
    }

    const auto env = node_classes::jvm_container::attachJvm();
    if (signature.isInt()) {
        // Value is an integer
        env->SetIntField(classInstance, id, data.i);
    } else if (signature.isBool()) {
        // Value is a boolean
        env->SetBooleanField(classInstance, id, data.z);
    } else if (signature.isByte()) {
        // Value is a byte
        env->SetByteField(classInstance, id, data.b);
    } else if (signature.isChar()) {
        // Value is a char
        env->SetCharField(classInstance, id, data.c);
    } else if (signature.isShort()) {
        // Value is a short
        env->SetShortField(classInstance, id, data.s);
    } else if (signature.isLong()) {
        // Value is a long
        env->SetLongField(classInstance, id, data.j);
    } else if (signature.isFloat()) {
        // Value is a float
        env->SetFloatField(classInstance, id, data.f);
    } else if (signature.isDouble()) {
        // Value is a double
        env->SetDoubleField(classInstance, id, data.d);
    } else {
        env->SetObjectField(classInstance, id, data.l);
    }
    JVM_CHECK_EXCEPTION(env);
}

void java_field::setStatic(jclass clazz, jvalue data) const {
    if (!isStatic) {
        throw std::runtime_error("Tried to access a non-static field through a static accessor");
    }

    const auto env = node_classes::jvm_container::attachJvm();
    if (signature.isInt()) {
        // Value is an integer
        env->SetStaticIntField(clazz, id, data.i);
    } else if (signature.isBool()) {
        // Value is a boolean
        env->SetStaticBooleanField(clazz, id, data.z);
    } else if (signature.isByte()) {
        // Value is a byte
        env->SetStaticByteField(clazz, id, data.b);
    } else if (signature.isChar()) {
        // Value is a char
        env->SetStaticCharField(clazz, id, data.c);
    } else if (signature.isShort()) {
        // Value is a short
        env->SetStaticShortField(clazz, id, data.s);
    } else if (signature.isLong()) {
        // Value is a long
        env->SetStaticLongField(clazz, id, data.j);
    } else if (signature.isFloat()) {
        // Value is a float
        env->SetStaticFloatField(clazz, id, data.f);
    } else if (signature.isDouble()) {
        // Value is a double
        env->SetStaticDoubleField(clazz, id, data.d);
    } else {
        env->SetStaticObjectField(clazz, id, data.l);
    }
    JVM_CHECK_EXCEPTION(env);
}

java_function::java_function(std::vector<java_type> parameterTypes, java_type returnType, std::string functionName,
                             jmethodID method, bool isStatic)
        : parameterTypes(std::move(parameterTypes)), returnType(std::move(returnType)), name(std::move(functionName)),
          method(method), isStatic(isStatic) {}

std::string java_function::to_string() const {
    std::stringstream ss;
    if (isStatic) {
        ss << "static ";
    }

    ss << util::make_java_name_readable(returnType.signature) << ' ' << name << '(';
    for (size_t i = 0; i < parameterTypes.size(); i++) {
        if (i > 0) {
            ss << ", ";
        }
        ss << util::make_java_name_readable(parameterTypes[i].signature);
    }

    ss << ')';
    return ss.str();
}

java_class::java_class() : clazz() {}

java_class::java_class(const std::vector<java_field> &static_fields, const std::vector<java_field> &fields,
                       const std::vector<java_function> &static_functions, const std::vector<java_function> &functions,
                       std::vector<java_constructor> constructors, jobject_wrapper<jclass> clazz)
        : static_fields(util::map_vector_values_to_map(static_fields)),
          fields(util::map_vector_values_to_map(fields)), static_functions(util::map_vector_to_map(static_functions)),
          functions(util::map_vector_to_map(functions)), constructors(std::move(constructors)),
          clazz(std::move(clazz)) {}